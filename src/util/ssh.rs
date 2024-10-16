use crate::{
    get_or_prompt_passphrase, load_config, prompt_yes_no, read_password_from_tty, save_config,
    Connection,
};
use std::io::{stdin, Write};
use std::process::Command;

pub fn handle_ssh(args: &[String]) {
    if args.is_empty() {
        println!("Usage: velo ssh <connection_name>");
        return;
    }

    let connection_name = &args[0];
    let username = whoami::username();
    let passphrase = get_or_prompt_passphrase(&username);
    let config = load_config(&passphrase);

    match config.connections.get(connection_name) {
        Some(conn) => {
            println!("Connecting to {}...", connection_name);

            let mut ssh_command = format!(
                "ssh -o StrictHostKeyChecking=no -p {} {}@{}",
                conn.port, conn.user, conn.host
            );

            if let Some(password) = &conn.password {
                // Use sshpass if available, otherwise prompt for password
                if Command::new("sshpass").arg("-h").output().is_ok() {
                    ssh_command = format!(
                        "sshpass -p {} ssh -o StrictHostKeyChecking=no -p {} {}@{}",
                        password, conn.port, conn.user, conn.host
                    );
                } else {
                    println!("Password is stored, but sshpass is not available.");
                    println!("You will be prompted to enter the password manually.");
                }
            }

            // Check if tmux is available and run ssh inside tmux
            if Command::new("tmux").arg("-V").output().is_ok() {
                println!("Launching ssh in a split tmux pane...");

                let split_option = if prompt_yes_no("Do you want to split horizontally? (y/n): ") {
                    "-h"
                } else {
                    "-v"
                };

                let status = Command::new("tmux")
                    .arg("split-window")
                    .arg(split_option)
                    .arg(ssh_command)
                    .status()
                    .expect("Failed to split tmux window with SSH");

                if !status.success() {
                    println!("Failed to split tmux window with SSH");
                } else {
                    println!("SSH is running in a new tmux pane. You can switch panes with Ctrl-b and the arrow keys.");
                }
            } else {
                // If tmux is not available, just run the ssh command directly
                let status = Command::new("sh")
                    .arg("-c")
                    .arg(ssh_command)
                    .status()
                    .expect("Failed to execute ssh command");

                if !status.success() {
                    println!("SSH connection failed");
                }
            }
        }
        None => println!("Connection '{}' not found", connection_name),
    }
}

pub fn handle_add_connection(args: &[String], passphrase: &str) {
    if args.len() != 3 {
        println!("Usage: velo add <name> <host> <user>");
        return;
    }

    let name = &args[0];
    let host = &args[1];
    let user = &args[2];

    let mut config = load_config(passphrase);

    if config.connections.contains_key(name) {
        println!("Connection '{}' already exists.", name);
        return;
    }

    let port = prompt_port();
    let password = prompt_password();

    let connection = Connection {
        host: host.to_string(),
        user: user.to_string(),
        port,
        password,
    };

    config.connections.insert(name.to_string(), connection);
    save_config(&config, passphrase);

    println!("Connection '{}' added successfully.", name);
}

pub fn handle_remove_connection(args: &[String], passphrase: &str) {
    if args.is_empty() {
        println!("Usage: velo remove <connection_name>");
        return;
    }

    let connection_name = &args[0];
    let mut config = load_config(passphrase);

    match config.connections.remove(connection_name) {
        Some(_) => {
            save_config(&config, passphrase);
            println!("Connection '{}' removed successfully", connection_name);
        }
        None => println!("Connection '{}' not found", connection_name),
    }
}

pub fn handle_list_connections(passphrase: &str) {
    let config = load_config(passphrase);
    if config.connections.is_empty() {
        println!("No connections stored.");
    } else {
        println!("Stored connections:");
        for (name, conn) in config.connections {
            println!("{}: {}@{} (port {})", name, conn.user, conn.host, conn.port);
            if conn.password.is_some() {
                println!("  (Password stored)");
            }
        }
    }
}

fn prompt_port() -> u16 {
    loop {
        print!("Enter port (default: 22): ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read input");
        let input = input.trim();

        if input.is_empty() {
            return 22;
        }

        match input.parse() {
            Ok(port) => return port,
            Err(_) => println!("Invalid port number. Please try again."),
        }
    }
}

fn prompt_password() -> Option<String> {
    if prompt_yes_no("Do you want to save the password? (y/n): ") {
        Some(read_password_from_tty("Enter password: ").unwrap())
    } else {
        None
    }
}
