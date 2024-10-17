use std::fs::{File, OpenOptions};
use std::io::{Read, Write, BufReader, BufWriter, stdin};
use std::path::{PathBuf, Path};
use std::process::Command;
use regex::Regex;
use crate::util::zellij::*;

pub struct SSHConfig {
    path: PathBuf,
    content: String,
}

impl SSHConfig {
    pub fn new() -> std::io::Result<Self> {
        let home_dir = dirs::home_dir().expect("Unable to determine home directory");
        let config_path = home_dir.join(".ssh").join("config");
        let mut content = String::new();

        if config_path.exists() {
            let mut file = File::open(&config_path)?;
            file.read_to_string(&mut content)?;
        }

        Ok(SSHConfig {
            path: config_path,
            content,
        })
    }

    pub fn add_connection(&mut self, name: &str, host: &str, user: &str, port: u16) -> std::io::Result<()> {
        let entry = format!("\nHost {}\n    HostName {}\n    User {}\n    Port {}\n", name, host, user, port);
        self.content.push_str(&entry);
        self.save()?;
        println!("Connection '{}' added successfully.", name);
        Ok(())
    }


    pub fn remove_connection(&mut self, name: &str) -> std::io::Result<bool> {
        let host_pattern = format!(r"(?m)^Host\s+{}\s*$", regex::escape(name));
        let re = Regex::new(&host_pattern).unwrap();

        if re.is_match(&self.content) {
            let mut new_content = String::new();
            let mut skip_block = false;
            let mut removed = false;

            for line in self.content.lines() {
                if re.is_match(line) {
                    skip_block = true;
                    removed = true;
                    continue;
                }
                if skip_block {
                    if line.trim().starts_with("Host ") {
                        skip_block = false;
                    } else {
                        continue;
                    }
                }
                new_content.push_str(line);
                new_content.push('\n');
            }

            if removed {
                self.content = new_content;
                self.save()?;
            }
            Ok(removed)
        } else {
            Ok(false)
        }
    }

    pub fn list_connections(&self) -> Vec<String> {
        let re = Regex::new(r"(?m)^Host (.+)$").unwrap();
        re.captures_iter(&self.content)
            .map(|cap| cap[1].to_string())
            .collect()
    }


    pub fn add_key(&mut self) -> std::io::Result<()> {
        let ssh_dir = dirs::home_dir().unwrap().join(".ssh");
        let pub_keys: Vec<PathBuf> = std::fs::read_dir(&ssh_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("pub") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        if pub_keys.is_empty() {
            println!("No SSH public keys found in {}.", ssh_dir.display());
            return Ok(());
        }

        println!("Select an SSH public key:");
        for (i, key) in pub_keys.iter().enumerate() {
            println!("  {}) {}", i + 1, key.file_name().unwrap().to_string_lossy());
        }
        println!("  {}) Enter a custom path", pub_keys.len() + 1);

        let mut choice = String::new();
        stdin().read_line(&mut choice)?;
        let choice: usize = choice.trim().parse().expect("Please enter a number");

        let selected_key = if choice <= pub_keys.len() {
            pub_keys[choice - 1].clone()
        } else {
            println!("Enter the path to your existing private key:");
            let mut custom_path = String::new();
            stdin().read_line(&mut custom_path)?;
            PathBuf::from(custom_path.trim())
        };

        let private_key_path = if selected_key.extension().and_then(|s| s.to_str()) == Some("pub") {
            selected_key.with_extension("")
        } else {
            selected_key
        };

        // Check and fix permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&private_key_path)?;
            let mode = metadata.permissions().mode();
            if mode & 0o777 != 0o600 {
                std::fs::set_permissions(&private_key_path, PermissionsExt::from_mode(0o600))?;
                println!("Fixed permissions on {}", private_key_path.display());
            }
        }

        // Add key to ssh-agent
        self.add_key_to_agent(&private_key_path)?;

        // Ask for the SSH connection to add the key to
        println!("Enter the name of the SSH connection to add this key to:");
        let mut connection_name = String::new();
        stdin().read_line(&mut connection_name)?;
        let connection_name = connection_name.trim();

        // Update SSH config file
        self.update_config_with_key(connection_name, &private_key_path)?;

        Ok(())
    }

    fn add_key_to_agent(&self, key_path: &Path) -> std::io::Result<()> {
        let output = if cfg!(target_os = "macos") {
            Command::new("ssh-add")
                .arg("--apple-use-keychain")
                .arg(key_path)
                .output()?
        } else {
            Command::new("ssh-add")
                .arg(key_path)
                .output()?
        };

        if output.status.success() {
            println!("SSH key added to ssh-agent successfully.");
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to add SSH key to ssh-agent: {}", error)))
        }
    }

    fn update_config_with_key(&mut self, connection_name: &str, key_path: &Path) -> std::io::Result<()> {
        let host_pattern = format!(r"(?m)^Host\s+{}\s*$", regex::escape(connection_name));
        let re = Regex::new(&host_pattern).unwrap();

        let key_path_str = key_path.to_str().unwrap();
        let new_identity_line = format!("    IdentityFile {}", key_path_str);

        if re.is_match(&self.content) {
            let mut new_content = String::new();
            let mut in_host_block = false;
            let mut identity_added = false;

            for line in self.content.lines() {
                if re.is_match(line) {
                    in_host_block = true;
                    new_content.push_str(line);
                    new_content.push('\n');
                    continue;
                }

                if in_host_block {
                    if line.trim().starts_with("IdentityFile") {
                        if !identity_added {
                            new_content.push_str(&new_identity_line);
                            new_content.push('\n');
                            identity_added = true;
                        }
                    } else if line.trim().is_empty() || line.trim().starts_with("Host ") {
                        if !identity_added {
                            new_content.push_str(&new_identity_line);
                            new_content.push('\n');
                        }
                        in_host_block = false;
                    } else {
                        new_content.push_str(line);
                        new_content.push('\n');
                    }
                } else {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }

            if in_host_block && !identity_added {
                new_content.push_str(&new_identity_line);
                new_content.push('\n');
            }

            self.content = new_content;
        } else {
            self.content.push_str(&format!("\nHost {}\n{}\n", connection_name, new_identity_line));
        }

        self.save()?;
        println!("SSH config updated for connection '{}'. Added key: {}", connection_name, key_path_str);
        Ok(())
    }

    fn save(&self) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.path)?;
        file.write_all(self.content.as_bytes())?;
        Ok(())
    }
}

pub fn handle_add_key() -> std::io::Result<()> {
    ensure_ssh_agent_running()?;
    let mut ssh_config = SSHConfig::new()?;
    ssh_config.add_key()
}

pub fn ensure_ssh_agent_running() -> std::io::Result<()> {
    let output = Command::new("ssh-add")
        .arg("-l")
        .output()?;

    if !output.status.success() {
        println!("Starting ssh-agent...");
        let output = Command::new("ssh-agent")
            .arg("-s")
            .output()?;

        if output.status.success() {
            let agent_output = String::from_utf8_lossy(&output.stdout);
            for line in agent_output.lines() {
                if line.starts_with("SSH_AUTH_SOCK=") || line.starts_with("SSH_AGENT_PID=") {
                    let parts: Vec<&str> = line.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        std::env::set_var(parts[0], parts[1].trim_matches(|c| c == ';' || c == '"'));
                    }
                }
            }
            println!("ssh-agent started successfully.");
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to start ssh-agent"));
        }
    } else {
        println!("ssh-agent is already running.");
    }

    Ok(())
}

pub fn handle_add_connection(args: &[String]) -> std::io::Result<()> {
    if args.len() < 3 {
        println!("Usage: velo add <name> <host> <user> [port]");
        return Ok(());
    }

    let name = &args[0];
    let host = &args[1];
    let user = &args[2];
    let port = if args.len() > 3 {
        args[3].parse().unwrap_or(22)
    } else {
        22
    };

    let mut ssh_config = SSHConfig::new()?;

    if ssh_config.list_connections().contains(&name.to_string()) {
        println!("Connection '{}' already exists.", name);
        return Ok(());
    }

    ssh_config.add_connection(name, host, user, port)?;

    println!("Connection '{}' added successfully.", name);
    println!("To add an SSH key to this connection, use: velo add-key");

    Ok(())
}

pub fn handle_remove_connection(args: &[String]) -> std::io::Result<()> {
    if args.is_empty() {
        println!("Usage: velo remove <connection_name>");
        return Ok(());
    }

    let connection_name = &args[0];
    let mut ssh_config = SSHConfig::new()?;

    if ssh_config.remove_connection(connection_name)? {
        println!("Connection '{}' removed successfully", connection_name);
    } else {
        println!("Connection '{}' not found", connection_name);
    }
    Ok(())
}

pub fn handle_list_connections() -> std::io::Result<()> {
    let ssh_config = SSHConfig::new()?;
    let connections = ssh_config.list_connections();

    if connections.is_empty() {
        println!("No connections stored.");
    } else {
        println!("Stored connections:");
        for name in connections {
            println!("  {}", name);
        }
    }
    Ok(())
}


pub fn get_connections() -> Vec<String> {
    let ssh_config = SSHConfig::new().unwrap();
    ssh_config.list_connections()
}

pub fn handle_ssh(args: &[String]) -> std::io::Result<()> {
    if args.is_empty() {
        println!("Usage: velo ssh <connection_name>");
        return Ok(());
    }

    let connection_name = &args[0];
    ensure_ssh_agent_running();

    // Create or attach to a Zellij session
    let session_name = format!("ssh-{}", connection_name);
    match create_session(&session_name) {
        Ok(_) => println!("Created new Zellij session: {}", session_name),
        Err(_) => println!("Attaching to existing Zellij session: {}", session_name),
    }

    let status = std::process::Command::new("zellij")
        .args(&["run", "--", "ssh", connection_name])
        .status()?;

    if !status.success() {
        println!("SSH connection failed");
    }
    Ok(())
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