mod util;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{stdin, Read, Write};
use std::path::PathBuf;
use util::help::*;
use util::ssh::*;
use util::tmux::handle_tmux;

#[derive(Serialize, Deserialize)]
struct Connection {
    host: String,
    user: String,
    port: u16,
}

#[derive(Serialize, Deserialize)]
struct Config {
    connections: HashMap<String, Connection>,
}

const CONFIG_FILE: &str = ".velo_config";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args[1] == "-h" {
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
        "add-key" => {
            if rest_args.contains(&"-h".to_string()) {
                print!("Not implemented yet.");
            } else {
                if let Err(e) = handle_add_key() {
                    eprintln!("Error adding SSH key: {}", e);
                }
            }
        }
        _ => println!("Unknown command: {}. Use -h for help.", command),
    }
}