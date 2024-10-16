mod util;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{stdin, Read, Write};
use std::path::PathBuf;
use util::help::*;
use util::ssh::*;
use util::tmux::handle_tmux; // Import the `handle_tmux` function from the `tmux` module // Import the `handle_ssh` function and `Connection` struct from the `ssh` module

#[derive(Serialize, Deserialize)]
struct Connection {
    host: String,
    user: String,
    port: u16,
    password: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Config {
    connections: HashMap<String, Connection>,
}

const CONFIG_FILE: &str = ".velo_config";
const NONCE_SIZE: usize = 12;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args[1] == "-h" {
        // If there are no arguments or the first argument is `-h`, show global help
        print_main_help();
        return;
    }

    let command = &args[1];
    let rest_args = &args[2..];

    match command.as_str() {
        "tmux" => {
            if rest_args.contains(&"-h".to_string()) {
                print_tmux_help();
            } else {
                handle_tmux(rest_args);
            }
        }
        "ssh" => {
            if rest_args.contains(&"-h".to_string()) {
                print_ssh_help();
            } else {
                handle_ssh(rest_args);
            }
        }
        "add" => {
            if rest_args.contains(&"-h".to_string()) {
                print_add_help();
            } else {
                handle_add_connection(rest_args);
            }
        }
        "list" | "ls" => {
            if rest_args.contains(&"-h".to_string()) {
                print_list_help();
            } else {
                handle_list_connections();
            }
        }
        "remove" | "rm" => {
            if rest_args.contains(&"-h".to_string()) {
                print_remove_help();
            } else {
                handle_remove_connection(rest_args);
            }
        }
        _ => println!("Unknown command: {}. Use -h for help.", command),
    }
}

fn load_config() -> Config {
    let home_dir = env::var("HOME").expect("Unable to determine home directory");
    let config_path = PathBuf::from(home_dir).join(CONFIG_FILE);

    if !config_path.exists() {
        return Config {
            connections: HashMap::new(),
        };
    }

    let password = get_password("Enter password to decrypt config: ");
    let mut file = File::open(config_path).expect("Unable to open config file");
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .expect("Unable to read config file");

    let nonce = Nonce::from_slice(&contents[..NONCE_SIZE]);
    let ciphertext = &contents[NONCE_SIZE..];

    let key = derive_key(&password);
    let cipher = Aes256Gcm::new_from_slice(&key).expect("Invalid key length");
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .expect("Decryption failed");

    serde_json::from_slice(&plaintext).expect("Unable to deserialize config")
}

fn save_config(config: &Config) {
    let home_dir = env::var("HOME").expect("Unable to determine home directory");
    let config_path = PathBuf::from(home_dir).join(CONFIG_FILE);

    let password = get_password("Enter password to encrypt config: ");
    let plaintext = serde_json::to_vec(config).expect("Unable to serialize config");

    let key = derive_key(&password);
    let cipher = Aes256Gcm::new_from_slice(&key).expect("Invalid key length");
    let mut rng = rand::thread_rng();
    let mut nonce = [0u8; NONCE_SIZE];
    rng.fill(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .expect("Encryption failed");

    let mut file = File::create(config_path).expect("Unable to create config file");
    file.write_all(nonce).expect("Unable to write nonce");
    file.write_all(&ciphertext)
        .expect("Unable to write encrypted data");
}

fn load_config_with_password(password: &str) -> Config {
    let home_dir = env::var("HOME").expect("Unable to determine home directory");
    let config_path = PathBuf::from(home_dir).join(CONFIG_FILE);

    if !config_path.exists() {
        return Config {
            connections: HashMap::new(),
        };
    }

    let mut file = File::open(config_path).expect("Unable to open config file");
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .expect("Unable to read config file");

    let nonce = Nonce::from_slice(&contents[..NONCE_SIZE]);
    let ciphertext = &contents[NONCE_SIZE..];

    let key = derive_key(password);
    let cipher = Aes256Gcm::new_from_slice(&key).expect("Invalid key length");
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .expect("Decryption failed");

    serde_json::from_slice(&plaintext).expect("Unable to deserialize config")
}

fn save_config_with_password(config: &Config, password: &str) {
    let home_dir = env::var("HOME").expect("Unable to determine home directory");
    let config_path = PathBuf::from(home_dir).join(CONFIG_FILE);

    let plaintext = serde_json::to_vec(config).expect("Unable to serialize config");

    let key = derive_key(password);
    let cipher = Aes256Gcm::new_from_slice(&key).expect("Invalid key length");
    let mut rng = rand::thread_rng();
    let mut nonce = [0u8; NONCE_SIZE];
    rng.fill(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .expect("Encryption failed");

    let mut file = File::create(config_path).expect("Unable to create config file");
    file.write_all(nonce).expect("Unable to write nonce");
    file.write_all(&ciphertext)
        .expect("Unable to write encrypted data");
}

fn get_password(prompt: &str) -> String {
    use std::io::Write;
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();
    let mut password = String::new();
    stdin()
        .read_line(&mut password)
        .expect("Failed to read password");
    password.trim().to_string()
}

fn derive_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}

fn prompt_yes_no(prompt: &str) -> bool {
    loop {
        print!("{}", prompt);
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read input");
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please answer with 'y' or 'n'"),
        }
    }
}

fn read_password_from_tty(prompt: &str) -> std::io::Result<String> {
    print!("{}", prompt);
    std::io::stdout().flush()?;
    read_password()
}
