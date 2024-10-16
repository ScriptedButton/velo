mod util;

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    Frame,
    text::{Span, Text, Line},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    layout::{Layout, Constraint, Direction},
    style::{Style, Color},
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, error::Error};
use regex::Regex;
use std::process::Command;
use crate::util::ssh::{ensure_ssh_agent_running, SSHConfig};
// Include the SSHConfig struct and its implementation from the provided code

enum InputMode {
    Normal,
    AddConnection,
    RemoveConnection,
    AddKey,
}

struct App {
    ssh_config: SSHConfig,
    connections: Vec<String>,
    input: String,
    input_mode: InputMode,
    selected_index: usize,
    error_message: Option<String>,
}

impl App {
    fn new() -> Result<Self, Box<dyn Error>> {
        let ssh_config = SSHConfig::new()?;
        let connections = ssh_config.list_connections();
        Ok(App {
            ssh_config,
            connections,
            input: String::new(),
            input_mode: InputMode::Normal,
            selected_index: 0,
            error_message: None,
        })
    }

    fn refresh_connections(&mut self) {
        self.connections = self.ssh_config.list_connections();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('a') => app.input_mode = InputMode::AddConnection,
                    KeyCode::Char('r') => app.input_mode = InputMode::RemoveConnection,
                    KeyCode::Char('k') => app.input_mode = InputMode::AddKey,
                    KeyCode::Down => {
                        if !app.connections.is_empty() {
                            app.selected_index = (app.selected_index + 1) % app.connections.len();
                        }
                    },
                    KeyCode::Up => {
                        if !app.connections.is_empty() {
                            if app.selected_index > 0 {
                                app.selected_index -= 1;
                            } else {
                                app.selected_index = app.connections.len() - 1;
                            }
                        }
                    },
                    KeyCode::Enter => {
                        if let Some(connection) = app.connections.get(app.selected_index) {
                            ensure_ssh_agent_running();
                            let status = Command::new("ssh")
                                .arg(connection)
                                .status()?;
                            if !status.success() {
                                app.error_message = Some("SSH connection failed".to_string());
                            }
                        }
                    },
                    _ => {}
                },
                InputMode::AddConnection => {
                    match key.code {
                        KeyCode::Enter => {
                            let parts: Vec<&str> = app.input.split_whitespace().collect();
                            if parts.len() == 3 {
                                let name = parts[0];
                                let host = parts[1];
                                let user = parts[2];
                                let port = 22; // Default port, you might want to add a way to specify this
                                if let Err(e) = app.ssh_config.add_connection(name, host, user, port) {
                                    app.error_message = Some(format!("Failed to add connection: {}", e));
                                } else {
                                    app.refresh_connections();
                                }
                                app.input.clear();
                                app.input_mode = InputMode::Normal;
                            } else {
                                app.error_message = Some("Invalid input. Use format: <name> <host> <user>".to_string());
                            }
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.input.clear();
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                },
                InputMode::RemoveConnection => {
                    match key.code {
                        KeyCode::Enter => {
                            if let Err(e) = app.ssh_config.remove_connection(&app.input) {
                                app.error_message = Some(format!("Failed to remove connection: {}", e));
                            } else {
                                app.refresh_connections();
                            }
                            app.input.clear();
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.input.clear();
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                },
                InputMode::AddKey => {
                    match key.code {
                        KeyCode::Enter => {
                            if let Err(e) = app.ssh_config.add_key() {
                                app.error_message = Some(format!("Failed to add key: {}", e));
                            }
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                },
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ]
                .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Line::from(Span::from("Press 'a' to add a connection")),
                Line::from(Span::from("Press 'r' to remove a connection")),
                Line::from(Span::from("Press 'k' to add an SSH key")),
                Line::from(Span::from("Use arrow keys to navigate")),
            ],
            Style::default().fg(Color::Yellow),
        ),
        InputMode::AddConnection => (
            vec![Line::from(Span::from("Enter new connection details: <name> <host> <user>"))],
            Style::default().fg(Color::Green),
        ),
        InputMode::RemoveConnection => (
            vec![Line::from(Span::from("Enter the name of the connection to remove:"))],
            Style::default().fg(Color::Red),
        ),
        InputMode::AddKey => (
            vec![Line::from(Span::from("Press Enter to add an SSH key, or Esc to cancel"))],
            Style::default().fg(Color::Cyan),
        ),
    };

    let help_message = Paragraph::new(Text::from(msg))
        .style(style)
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help_message, chunks[0]);

    let connections: Vec<ListItem> = app
        .connections
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = Span::from(Span::raw(m));
            let style = if i == app.selected_index {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            ListItem::new(content).style(style)
        })
        .collect();
    let connections = List::new(connections)
        .block(Block::default().borders(Borders::ALL).title("Connections"));
    f.render_widget(connections, chunks[1]);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[2]);

    if let Some(error) = &app.error_message {
        let error_message = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Error"));
        let error_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3)].as_ref())
            .split(f.size())[0];
        f.render_widget(error_message, error_area);
    }
}

// Include the helper functions from the provided code (ensure_ssh_agent_running, prompt_port, prompt_yes_no)