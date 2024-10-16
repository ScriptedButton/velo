use crate::util::help::*;
use crate::util::ssh::*;
use crate::util::tmux::*;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Helper;
use rustyline::{CompletionType, Config, Context, EditMode, Editor};
use std::borrow::Cow::{self, Borrowed, Owned};
use std::io::Error as IoError;

pub struct VeloCompleter {
    commands: Vec<String>,
    tmux_subcommands: Vec<String>,
}

impl VeloCompleter {
    pub fn new() -> Self {
        VeloCompleter {
            commands: vec![
                "tmux".to_string(),
                "ssh".to_string(),
                "add".to_string(),
                "list".to_string(),
                "remove".to_string(),
                "add-key".to_string(),
            ],
            tmux_subcommands: vec![
                "new".to_string(),
                "list".to_string(),
                "ls".to_string(),
                "attach".to_string(),
                "kill".to_string(),
            ],
        }
    }

    fn get_ssh_connections(&self) -> Vec<String> {
        match SSHConfig::new() {
            Ok(ssh_config) => ssh_config.list_connections(),
            Err(_) => Vec::new(),
        }
    }
}

impl Completer for VeloCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let mut completions = vec![];
        let words: Vec<&str> = line[..pos].split_whitespace().collect();

        if words.is_empty() {
            return Ok((0, completions));
        }

        let start = if words.len() == 1 {
            0
        } else {
            line[..pos].rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0)
        };

        let word_to_complete = words.last().unwrap();

        if words.len() == 1 {
            // Complete main commands
            for command in &self.commands {
                if command.starts_with(word_to_complete) {
                    completions.push(Pair {
                        display: command.clone(),
                        replacement: command.clone(),
                    });
                }
            }
        } else if words[0] == "tmux" && words.len() == 2 {
            // Complete tmux subcommands
            for subcommand in &self.tmux_subcommands {
                if subcommand.starts_with(word_to_complete) {
                    completions.push(Pair {
                        display: subcommand.clone(),
                        replacement: subcommand.clone(),
                    });
                }
            }
        } else if words[0] == "ssh" && words.len() == 2 {
            // Complete SSH connections
            for connection in self.get_ssh_connections() {
                if connection.starts_with(word_to_complete) {
                    completions.push(Pair {
                        display: connection.clone(),
                        replacement: connection.clone(),
                    });
                }
            }
        }

        Ok((start, completions))
    }
}

impl Hinter for VeloCompleter {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for VeloCompleter {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Borrowed(line)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, forced: bool) -> bool {
        forced
    }
}

impl Validator for VeloCompleter {}

// Implement the Helper trait for VeloCompleter
impl Helper for VeloCompleter {}

pub fn run_interactive_shell() -> rustyline::Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();

    let h = VeloCompleter::new();

    let mut rl = Editor::with_config(config).map_err(|err| {
        eprintln!("Error creating editor: {:?}", err);
        err
    })?;

    rl.set_helper(Some(h));

    loop {
        let readline = rl.readline("velo> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let args: Vec<String> = line.split_whitespace().map(String::from).collect();
                if !args.is_empty() {
                    if let Err(e) = handle_command(&args) {
                        eprintln!("Error: {}", e);
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn handle_command(args: &[String]) -> Result<(), IoError> {
    match args[0].as_str() {
        "exit" => std::process::exit(0),
        "tmux" => {
            handle_tmux(&args[1..]);
            Ok(())
        }
        "ssh" => handle_ssh(&args[1..]),
        "add" => handle_add_connection(&args[1..]),
        "list" | "ls" => handle_list_connections(),
        "remove" | "rm" => handle_remove_connection(&args[1..]),
        "add-key" => handle_add_key(),
        "help" => {
            print_main_help();
            Ok(())
        }
        _ => {
            println!(
                "Unknown command: {}. Use 'help' for a list of commands.",
                args[0]
            );
            Ok(())
        }
    }
}
