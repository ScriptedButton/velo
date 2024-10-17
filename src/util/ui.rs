use std::io::{self, stdout};

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    widgets::{Block, Paragraph, Borders, List, ListItem, ListState},
    prelude::{Constraint, Direction, Layout},
    Frame, Terminal,
    style::{Color, Modifier, Style},
};

use crate::util::ssh::{get_connections, handle_ssh, handle_add_connection};

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
                            app_state
                                .ssh_connections_state
                                .select(Some(
                                    (i + 1).min(app_state.ssh_connections.len().saturating_sub(1)),
                                ));
                        }
                    }
                    KeyCode::Tab => {
                        app_state.focused_section = 1 - app_state.focused_section;
                    }
                    KeyCode::Enter => {
                        if app_state.focused_section == 0 {
                            match app_state.main_menu_state.selected() {
                                Some(3) => {
                                    // Start editing when Add Connection is selected
                                    app_state.input_mode = InputMode::Editing;
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Esc => {
                        app_state.input_mode = InputMode::Normal;
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
                    KeyCode::Tab => {
                        app_state.add_connection_form.next_field();
                    }
                    KeyCode::BackTab => {
                        app_state.add_connection_form.prev_field();
                    }
                    _ => {}
                },
            }
        }
    }
    Ok(false)
}

fn ui(frame: &mut Frame, app_state: &mut AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.area());

    let main_menu_items = vec![
        ListItem::new("SSH"),
        ListItem::new("Tmux"),
        ListItem::new("Zellij"),
        ListItem::new("Add Connection"),
        ListItem::new("Add Key"),
    ];

    let main_menu_block = Block::new()
        .borders(Borders::ALL)
        .title("Main Menu")
        .border_style(Style::default().add_modifier(
            if app_state.focused_section == 0 {
                Modifier::BOLD
            } else {
                Modifier::empty()
            },
        ));

    let main_menu = List::new(main_menu_items)
        .block(main_menu_block)
        .highlight_symbol("=> ")
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(main_menu, layout[0], &mut app_state.main_menu_state);

    let bottom_block = Block::new()
        .borders(Borders::ALL)
        .title("Details")
        .border_style(Style::default().add_modifier(
            if app_state.focused_section == 1 {
                Modifier::BOLD
            } else {
                Modifier::empty()
            },
        ));

    match app_state.main_menu_state.selected() {
        Some(3) => {
            // Add Connection Form
            let add_connection_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Length(3), Constraint::Length(3)])
                .split(layout[1]);

            let field_names = ["Name", "Host", "User"];
            for (i, field) in app_state.add_connection_form.fields.iter().enumerate() {
                let color = if app_state.input_mode == InputMode::Editing
                    && app_state.add_connection_form.current_field == i
                {
                    Color::Yellow
                } else {
                    Color::White
                };
                let input = Paragraph::new(field.as_str())
                    .style(Style::default().fg(color))
                    .block(Block::default().borders(Borders::ALL).title(field_names[i]));
                frame.render_widget(input, add_connection_layout[i]);
            }
        }
        _ => {
            frame.render_widget(Paragraph::new("").block(bottom_block), layout[1]);
        }
    }
}
