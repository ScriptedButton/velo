use std::io::{self, stdout};

use crate::util::ssh::{get_connections, handle_add_connection, handle_ssh_from_tui};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, BorderType},
    Frame, Terminal,
};

#[derive(Clone, PartialEq)]
enum InputMode {
    Normal,
    Editing,
}

pub struct AppState {
    focused_section: usize,
    main_menu_state: ListState,
    ssh_connections_state: ListState,
    ssh_connections: Vec<String>,
    input_mode: InputMode,
    add_connection_form: AddConnectionForm,
}

struct AddConnectionForm {
    fields: Vec<String>,
    current_field: usize,
}

impl AddConnectionForm {
    fn new() -> Self {
        Self {
            fields: vec![String::new(); 3], // 3 fields: name, host, user
            current_field: 0,
        }
    }

    fn next_field(&mut self) {
        self.current_field = (self.current_field + 1) % self.fields.len();
    }

    fn prev_field(&mut self) {
        self.current_field = (self.current_field + self.fields.len() - 1) % self.fields.len();
    }
}

impl AppState {
    fn new() -> Self {
        let mut main_menu_state = ListState::default();
        main_menu_state.select(Some(0));
        Self {
            focused_section: 0,
            main_menu_state,
            ssh_connections_state: ListState::default(),
            ssh_connections: get_connections(),
            input_mode: InputMode::Normal,
            add_connection_form: AddConnectionForm::new(),
        }
    }
}

pub fn launch_tui() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app_state = AppState::new();

    loop {
        terminal.draw(|f| ui(f, &mut app_state))?;
        if handle_events(&mut app_state)? {
            break;
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events(app_state: &mut AppState) -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match app_state.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(true),
                        KeyCode::Up => {
                            if app_state.focused_section == 0 {
                                let i = app_state.main_menu_state.selected().unwrap_or(0);
                                app_state.main_menu_state.select(Some(i.saturating_sub(1)));
                            } else {
                                let i = app_state.ssh_connections_state.selected().unwrap_or(0);
                                app_state
                                    .ssh_connections_state
                                    .select(Some(i.saturating_sub(1)));
                            }
                        }
                        KeyCode::Down => {
                            if app_state.focused_section == 0 {
                                let i = app_state.main_menu_state.selected().unwrap_or(0);
                                app_state.main_menu_state.select(Some((i + 1).min(4)));
                            } else {
                                let i = app_state.ssh_connections_state.selected().unwrap_or(0);
                                app_state.ssh_connections_state.select(Some(
                                    (i + 1).min(app_state.ssh_connections.len().saturating_sub(1)),
                                ));
                            }
                        }
                        KeyCode::Tab => {
                            app_state.focused_section = 1 - app_state.focused_section;
                            if app_state.focused_section == 1
                                && app_state.main_menu_state.selected() == Some(3)
                            {
                                app_state.input_mode = InputMode::Editing;
                            }
                        }
                        KeyCode::Enter => {
                            if app_state.focused_section == 0 {
                                match app_state.main_menu_state.selected() {
                                    Some(3) => {
                                        app_state.input_mode = InputMode::Editing;
                                    }
                                    _ => {}
                                }
                            } else if app_state.main_menu_state.selected() == Some(0) {
                                // SSH connection is selected
                                if let Some(selected_index) = app_state.ssh_connections_state.selected() {
                                    if let Some(selected_connection) = app_state.ssh_connections.get(selected_index) {
                                        if let Err(e) = handle_ssh_from_tui(selected_connection) {
                                            eprintln!("Failed to connect: {}", e);
                                        }
                                        // Indicate that a full redraw is needed
                                        return Ok(true);
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Esc => {
                            app_state.input_mode = InputMode::Normal;
                            app_state.focused_section = app_state.focused_section - 1;
                        }
                        KeyCode::Enter => {
                            if app_state.add_connection_form.current_field == 2 {
                                let args: Vec<String> =
                                    app_state.add_connection_form.fields.clone();
                                if handle_add_connection(&args).is_ok() {
                                    app_state.ssh_connections = get_connections();
                                    app_state.input_mode = InputMode::Normal;
                                    app_state.add_connection_form = AddConnectionForm::new();
                                }
                            } else {
                                app_state.add_connection_form.next_field();
                            }
                        }
                        KeyCode::Backspace => {
                            let current_field = &mut app_state.add_connection_form.fields
                                [app_state.add_connection_form.current_field];
                            current_field.pop();
                        }
                        KeyCode::Char(c) => {
                            let current_field = &mut app_state.add_connection_form.fields
                                [app_state.add_connection_form.current_field];
                            current_field.push(c);
                        }
                        KeyCode::Tab | KeyCode::Down => {
                            app_state.add_connection_form.next_field();
                        }
                        KeyCode::BackTab | KeyCode::Up => {
                            app_state.add_connection_form.prev_field();
                        }
                        _ => {}
                    },
                }
            }
        }
    }
    Ok(false)
}

fn ui(frame: &mut Frame, app_state: &mut AppState) {
    // Define theme colors
    const NEON_GREEN: Color = Color::Rgb(0, 255, 136);
    const DARKER_GREEN: Color = Color::Rgb(0, 180, 96);
    const BACKGROUND: Color = Color::Rgb(16, 24, 24);
    const HIGHLIGHT: Color = Color::Rgb(255, 110, 199);

    // Set terminal background
    frame.render_widget(
        Block::default().style(Style::default().bg(BACKGROUND)),
        frame.area(),
    );

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(1),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(frame.area());

    let main_menu_items = vec![
        ListItem::new("[ SSH ]").style(Style::default().fg(NEON_GREEN)),
        ListItem::new("[ ZELLIJ ]").style(Style::default().fg(NEON_GREEN)),
        ListItem::new("[ ADD CONNECTION ]").style(Style::default().fg(NEON_GREEN)),
        ListItem::new("[ ADD KEY ]").style(Style::default().fg(NEON_GREEN)),
    ];

    // Title with matrix-like styling
    let title = Paragraph::new("// VELO //")
        .style(
            Style::default()
                .fg(HIGHLIGHT)
                .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK)
        )
        .alignment(Alignment::Center);

    frame.render_widget(title, layout[0]);

    let main_menu_block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title("[ MAIN_MENU ]")
        .title_alignment(Alignment::Center)
        .border_style(
            Style::default()
                .fg(if app_state.focused_section == 0 {
                    HIGHLIGHT
                } else {
                    DARKER_GREEN
                })
                .add_modifier(if app_state.focused_section == 0 {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        );

    let main_menu = List::new(main_menu_items)
        .block(main_menu_block)
        .highlight_symbol(">> ")
        .highlight_style(
            Style::default()
                .fg(HIGHLIGHT)
                .add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK),
        );

    frame.render_stateful_widget(main_menu, layout[1], &mut app_state.main_menu_state);

    let bottom_block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title("[ DETAILS ]")
        .title_alignment(Alignment::Center)
        .border_style(
            Style::default()
                .fg(if app_state.focused_section == 1 {
                    HIGHLIGHT
                } else {
                    DARKER_GREEN
                })
                .add_modifier(if app_state.focused_section == 1 {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        );

    match app_state.main_menu_state.selected() {
        Some(0) => {
            // SSH option with matrix-style decoration
            let connections_block = Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .title("[ SSH_CONNECTIONS ]")
                .title_alignment(Alignment::Center)
                .border_style(
                    Style::default()
                        .fg(if app_state.focused_section == 1 {
                            HIGHLIGHT
                        } else {
                            DARKER_GREEN
                        })
                        .add_modifier(if app_state.focused_section == 1 {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                );

            let connections: Vec<ListItem> = app_state
                .ssh_connections
                .iter()
                .map(|c| {
                    ListItem::new(format!("< {} >", c))
                        .style(Style::default().fg(NEON_GREEN))
                })
                .collect();

            let connections_list = List::new(connections)
                .block(connections_block)
                .highlight_style(
                    Style::default()
                        .fg(HIGHLIGHT)
                        .add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK),
                )
                .highlight_symbol(">> ");

            frame.render_stateful_widget(
                connections_list,
                layout[2],
                &mut app_state.ssh_connections_state,
            );
        }
        Some(3) => {
            // Add Connection Form with cyberpunk styling
            let add_connection_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                ])
                .split(layout[2]);

            let field_names = ["[ NAME ]", "[ HOST ]", "[ USER ]"];
            for (i, field) in app_state.add_connection_form.fields.iter().enumerate() {
                let is_active = app_state.input_mode == InputMode::Editing
                    && app_state.add_connection_form.current_field == i;

                let input = Paragraph::new(field.as_str())
                    .style(
                        Style::default().fg(if is_active {
                            HIGHLIGHT
                        } else {
                            NEON_GREEN
                        }),
                    )
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Double)
                            .title(field_names[i])
                            .border_style(
                                Style::default()
                                    .fg(if is_active {
                                        HIGHLIGHT
                                    } else {
                                        DARKER_GREEN
                                    })
                                    .add_modifier(if is_active {
                                        Modifier::BOLD
                                    } else {
                                        Modifier::empty()
                                    }),
                            ),
                    );
                frame.render_widget(input, add_connection_layout[i]);
            }
        }
        _ => {
            frame.render_widget(
                Paragraph::new("")
                    .block(bottom_block)
                    .style(Style::default().fg(NEON_GREEN)),
                layout[2],
            );
        }
    }
}

