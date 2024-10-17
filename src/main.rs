mod util;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{stdin, Read, Write};
use std::path::PathBuf;
use util::help::*;
use util::ssh::*;
use util::zellij::*;
use util::completion::run_interactive_shell;
use util::ui::launch_tui;

fn main() {
    launch_tui().unwrap();
    env::set_var("RUST_BACKTRACE", "1");

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        match run_interactive_shell() {
            Ok(_) => println!("Interactive shell exited successfully."),
            Err(e) => eprintln!("Error in interactive mode: {}", e),
        }
        return;
    }
    if args[1] == "-h" {
        print_main_help();
        return;
    }

    let command = &args[1];
    let rest_args = &args[2..];

    match command.as_str() {
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
        "add-key" => {
            if rest_args.contains(&"-h".to_string()) {
                print_add_key_help();
            } else {
                if let Err(e) = handle_add_key() {
                    eprintln!("Error adding SSH key: {}", e);
                }
            }
        }
        "zellij" => {
            if rest_args.is_empty() || rest_args.contains(&"-h".to_string()) || rest_args.contains(&"--help".to_string()) {
                print_zellij_help();
                return;
            }
            let subcommand = &rest_args[0];
            match subcommand.as_str() {
                "new" => {
                    if rest_args.len() != 2 {
                        println!("Usage: velo zellij new <session_name>");
                        return;
                    }
                    match create_session(&rest_args[1]) {
                        Ok(_) => {
                            println!("Zellij session '{}' created successfully.", rest_args[1]);
                            println!("To attach to this session, run: velo zellij attach {}", rest_args[1]);
                        },
                        Err(e) => eprintln!("Error: {}", e),
                    }
                },
                "list" => {
                    match list_sessions() {
                        Ok(sessions) => {
                            println!("Zellij sessions:");
                            for session in sessions {
                                println!("  {}", session);
                            }
                        },
                        Err(e) => eprintln!("Error listing Zellij sessions: {}", e),
                    }
                },
                "attach" => {
                    if rest_args.len() != 2 {
                        println!("Usage: velo zellij attach <session_name>");
                        return;
                    }
                    match attach_session(&rest_args[1]) {
                        Ok(_) => println!("Attached to Zellij session: {}", rest_args[1]),
                        Err(e) => eprintln!("Error attaching to Zellij session: {}", e),
                    }
                },
                "kill" => {
                    if rest_args.len() != 2 {
                        println!("Usage: velo zellij kill <session_name>");
                        return;
                    }
                    match kill_session(&rest_args[1]) {
                        Ok(_) => println!("Killed Zellij session: {}", rest_args[1]),
                        Err(e) => eprintln!("Error killing Zellij session: {}", e),
                    }
                },
                "create-layout" => {
                    if rest_args.len() != 3 {
                        println!("Usage: velo zellij create-layout <layout_name> <layout_file_path>");
                        return;
                    }
                    let layout_name = &rest_args[1];
                    let layout_file_path = &rest_args[2];
                    match fs::read_to_string(layout_file_path) {
                        Ok(content) => {
                            match create_layout(layout_name, &content) {
                                Ok(_) => println!("Layout '{}' created successfully.", layout_name),
                                Err(e) => eprintln!("Error creating layout: {}", e),
                            }
                        },
                        Err(e) => eprintln!("Error reading layout file: {}", e),
                    }
                },
                "list-layouts" => {
                    match list_layouts() {
                        Ok(layouts) => {
                            println!("Available Zellij layouts:");
                            for layout in layouts {
                                println!("  {}", layout);
                            }
                        },
                        Err(e) => eprintln!("Error listing layouts: {}", e),
                    }
                },
                _ => println!("Unknown Zellij subcommand: {}. Use -h for help.", subcommand),
            }
        },
        _ => println!("Unknown command: {}. Use 'velo -h' for help.", command),
    }
}