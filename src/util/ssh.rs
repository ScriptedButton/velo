use std::fs::{File, OpenOptions};
use std::io::{Read, Write, BufReader, BufWriter, stdin};
use std::path::{PathBuf, Path};
use regex::Regex;
use std::process::Command;

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
        self.save()
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
        if cfg!(target_os = "macos") {
            Command::new("ssh-add")
                .arg("--apple-use-keychain")
                .arg(&private_key_path)
                .status()?;
            println!("SSH key added to ssh-agent and passphrase stored in Apple Keychain.");
        } else {
            Command::new("ssh-add")
                .arg(&private_key_path)
                .status()?;
            println!("SSH key added to ssh-agent.");
        }

        // Update SSH config file
        self.update_config_with_key(&private_key_path)?;

        Ok(())
    }

    fn update_config_with_key(&mut self, key_path: &Path) -> std::io::Result<()> {
        const START_MARKER: &str = "### SSH Configurations Managed by Script Start ###";
        const END_MARKER: &str = "### SSH Configurations Managed by Script End ###";

        // Check if delimiters exist, if not add them
        if !self.content.contains(START_MARKER) {
            self.content.push_str(&format!("\n{}\n{}\n", START_MARKER, END_MARKER));
        }

        // Split the content into sections
        let mut sections: Vec<&str> = self.content.split(START_MARKER).collect();
        let managed_section = sections.pop().unwrap().split(END_MARKER).next().unwrap();

        // Parse existing groups and hosts
        let mut groups: Vec<(String, Vec<String>)> = Vec::new();
        let mut current_group = String::new();
        let mut current_hosts = Vec::new();

        for line in managed_section.lines() {
            if line.starts_with("# Group:") {
                if !current_group.is_empty() {
                    groups.push((current_group, current_hosts));
                    current_hosts = Vec::new();
                }
                current_group = line.trim_start_matches("# Group:").trim().to_string();
            } else if line.trim().starts_with("Host ") {
                current_hosts.push(line.trim().to_string());
            }
        }
        if !current_group.is_empty() {
            groups.push((current_group, current_hosts));
        }

        // Prompt for group information
        println!("Enter the group code (e.g., '470'):");
        let mut group_code = String::new();
        stdin().read_line(&mut group_code)?;
        let group_code = group_code.trim();

        println!("Enter the group name (e.g., 'CNIT 470 - Incident Response'):");
        let mut group_name = String::new();
        stdin().read_line(&mut group_name)?;
        let group_name = group_name.trim();

        // Find or create the group
        let group_index = groups.iter().position(|(name, _)| name == group_name);
        let group_entry = format!("# Group: {}\n    Host {}-*\n      IdentityFile {}\n      AddKeysToAgent yes",
                                  group_name, group_code, key_path.display());

        if let Some(index) = group_index {
            groups[index].1.push(group_entry);
        } else {
            groups.push((group_name.to_string(), vec![group_entry]));
        }

        // Rebuild the managed section
        let mut new_managed_section = String::new();
        for (_, hosts) in groups {
            for host in hosts {
                new_managed_section.push_str(&format!("{}\n", host));
            }
            new_managed_section.push('\n');
        }

        // Rebuild the entire config content
        let mut new_content = sections[0].to_string();
        new_content.push_str(&format!("{}\n{}\n{}\n", START_MARKER, new_managed_section.trim(), END_MARKER));
        if sections.len() > 1 {
            new_content.push_str(sections[1]);
        }

        self.content = new_content;
        self.save()?;

        println!("SSH config file updated successfully.");
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

pub fn handle_add_connection(args: &[String]) -> std::io::Result<()> {
    if args.len() != 3 {
        println!("Usage: velo add <name> <host> <user>");
        return Ok(());
    }

    let name = &args[0];
    let host = &args[1];
    let user = &args[2];

    let mut ssh_config = SSHConfig::new()?;

    if ssh_config.list_connections().contains(&name.to_string()) {
        println!("Connection '{}' already exists.", name);
        return Ok(());
    }

    let port = prompt_port();

    ssh_config.add_connection(name, host, user, port)?;

    println!("Connection '{}' added successfully.", name);
    println!("Note: Make sure to add your SSH key to ssh-agent before connecting.");
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

    let status = std::process::Command::new("ssh")
        .arg(connection_name)
        .status()?;

    if !status.success() {
        println!("SSH connection failed");
    }
    Ok(())
}

pub fn handle_add_key() -> std::io::Result<()> {
    let mut ssh_config = SSHConfig::new()?;
    ssh_config.add_key()
}


pub fn ensure_ssh_agent_running() {
    let output = std::process::Command::new("ssh-add")
        .arg("-l")
        .output()
        .expect("Failed to execute ssh-add");

    if !output.status.success() {
        println!("Starting ssh-agent...");
        let _ = std::process::Command::new("ssh-agent")
            .status()
            .expect("Failed to start ssh-agent");
        println!("ssh-agent started. Please add your SSH key using 'ssh-add <path_to_private_key>'");

        if prompt_yes_no("Would you like to add an SSH key now? (y/n): ") {
            let _ = std::process::Command::new("ssh-add")
                .status()
                .expect("Failed to run ssh-add");
        }
    } else {
        println!("ssh-agent is running and has keys loaded.");
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