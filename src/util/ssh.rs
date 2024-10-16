use crate::{
    get_password, load_config, load_config_with_password, prompt_yes_no, read_password_from_tty,
    save_config_with_password, Connection,
};
use std::process::Command;

pub fn handle_ssh(args: &[String]) {
    if args.is_empty() {
        println!("Usage: velo ssh <connection_name>");
        return;
    }

    let connection_name = &args[0];
    let config_password = get_password("Enter password to decrypt config: ");
    let config = load_config_with_password(&config_password);

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

                // Choose whether to split horizontally or vertically
                // Example: Split horizontally with the '-h' option or vertically with '-v'
                let split_option = if prompt_yes_no("Do you want to split horizontally? (y/n): ") {
                    "-h"
                } else {
                    "-v"
                };

                // Split the current tmux window and run the ssh command in the new pane
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

pub fn handle_add_connection(args: &[String]) {
    if args.len() != 4 {
        println!("Usage: velo add <name> <host> <user> <port>");
        return;
    }

    let name = &args[0];
    let host = &args[1];
    let user = &args[2];
    let port = args[3].parse::<u16>().expect("Invalid port number");

    let store_password = prompt_yes_no("Do you want to store the SSH password? (y/n): ");

    let password = if store_password {
        Some(read_password_from_tty("Enter SSH password: ").expect("Failed to read password"))
    } else {
        None
    };

    let config_password = get_password("Enter password for config encryption: ");
    let mut config = load_config_with_password(&config_password);

    config.connections.insert(
        name.to_string(),
        Connection {
            host: host.to_string(),
            user: user.to_string(),
            port,
            password,
        },
    );

    save_config_with_password(&config, &config_password);
    println!("Connection '{}' added successfully", name);
}

pub fn handle_remove_connection(args: &[String]) {
    if args.is_empty() {
        println!("Usage: velo remove <connection_name>");
        return;
    }

    let connection_name = &args[0];
    let password = get_password("Enter password: ");
    let mut config = load_config_with_password(&password);

    match config.connections.remove(connection_name) {
        Some(_) => {
            save_config_with_password(&config, &password);
            println!("Connection '{}' removed successfully", connection_name);
        }
        None => println!("Connection '{}' not found", connection_name),
    }
}

pub fn handle_list_connections() {
    let config = load_config();
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
