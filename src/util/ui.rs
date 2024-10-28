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
    layout::{
        Flex
    },
    Frame, Terminal,
};
use ratatui::layout::Position;

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
    cursor_positions: Vec<usize>, // Track cursor position for each field
}

impl AddConnectionForm {
    fn new() -> Self {
        Self {
            fields: vec![String::new(); 3], // 3 fields: name, host, user
            current_field: 0,
            cursor_positions: vec![0; 3], // Cursor position for each field
        }
    }

    fn next_field(&mut self) {
        self.current_field = (self.current_field + 1) % self.fields.len();
    }

    fn prev_field(&mut self) {
        self.current_field = (self.current_field + self.fields.len() - 1) % self.fields.len();
    }

    fn move_cursor_left(&mut self) {
        let current_cursor = &mut self.cursor_positions[self.current_field];
        *current_cursor = current_cursor.saturating_sub(1);
    }

    fn move_cursor_right(&mut self) {
        let field_len = self.fields[self.current_field].chars().count();
        let current_cursor = &mut self.cursor_positions[self.current_field];
        *current_cursor = (*current_cursor + 1).min(field_len);
    }

    fn enter_char(&mut self, c: char) {
        let field = &mut self.fields[self.current_field];
        let cursor_pos = self.cursor_positions[self.current_field];

        // Convert cursor position to byte index
        let byte_index = field
            .char_indices()
            .map(|(i, _)| i)
            .nth(cursor_pos)
            .unwrap_or(field.len());

        field.insert(byte_index, c);
        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let field = &mut self.fields[self.current_field];
        let cursor_pos = &mut self.cursor_positions[self.current_field];

        if *cursor_pos > 0 {
            let from_left_to_current_index = *cursor_pos - 1;

            // Getting all characters before and after the cursor
            let before_char = field.chars().take(from_left_to_current_index);
            let after_char = field.chars().skip(*cursor_pos);

            // Combine all characters except the deleted one
            *field = before_char.chain(after_char).collect();
            self.move_cursor_left();
        }
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
                                if let Some(selected_index) = app_state.ssh_connections_state.selected() {
                                    if let Some(selected_connection) = app_state.ssh_connections.get(selected_index) {
                                        if let Err(e) = handle_ssh_from_tui(selected_connection) {
                                            eprintln!("Failed to connect: {}", e);
                                        }
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
                            app_state.focused_section = 0;
                        }
                        KeyCode::Enter => {
                            if app_state.add_connection_form.current_field == 2 {
                                let args: Vec<String> = app_state.add_connection_form.fields.clone();
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
                            app_state.add_connection_form.delete_char();
                        }
                        KeyCode::Char(c) => {
                            app_state.add_connection_form.enter_char(c);
                        }
                        KeyCode::Left => {
                            app_state.add_connection_form.move_cursor_left();
                        }
                        KeyCode::Right => {
                            app_state.add_connection_form.move_cursor_right();
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

    // Main vertical layout with flex
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .flex(Flex::Start)
        .constraints([
            Constraint::Length(1),      // Title bar
            Constraint::Min(10),        // Content area
        ])
        .split(frame.area());

    // Title with matrix-like styling
    let title = Paragraph::new("// VELO //")
        .style(Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK))
        .alignment(Alignment::Center);
    frame.render_widget(title, main_layout[0]);

    // Content area split into two columns with flex
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .flex(Flex::SpaceBetween)
        .constraints([
            Constraint::Percentage(30),  // Menu column
            Constraint::Percentage(70),  // Details column
        ])
        .split(main_layout[1]);

    // Main menu items
    let main_menu_items = vec![
        ListItem::new("[ SSH ]").style(Style::default().fg(NEON_GREEN)),
        ListItem::new("[ ZELLIJ ]").style(Style::default().fg(NEON_GREEN)),
        ListItem::new("[ ADD CONNECTION ]").style(Style::default().fg(NEON_GREEN)),
        ListItem::new("[ ADD KEY ]").style(Style::default().fg(NEON_GREEN)),
    ];

    let main_menu_block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title("[ MAIN_MENU ]")
        .title_alignment(Alignment::Center)
        .border_style(
            Style::default()
                .fg(if app_state.focused_section == 0 { HIGHLIGHT } else { DARKER_GREEN })
                .add_modifier(if app_state.focused_section == 0 { Modifier::BOLD } else { Modifier::empty() }),
        );

    let main_menu = List::new(main_menu_items)
        .block(main_menu_block)
        .highlight_symbol(">> ")
        .highlight_style(
            Style::default()
                .fg(HIGHLIGHT)
                .add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK),
        );

    frame.render_stateful_widget(main_menu, content_layout[0], &mut app_state.main_menu_state);

    // Details section with flex
    match app_state.main_menu_state.selected() {
        Some(0) => {
            // SSH Connections list
            let connections_block = Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .title("[ SSH_CONNECTIONS ]")
                .title_alignment(Alignment::Center)
                .border_style(
                    Style::default()
                        .fg(if app_state.focused_section == 1 { HIGHLIGHT } else { DARKER_GREEN })
                        .add_modifier(if app_state.focused_section == 1 { Modifier::BOLD } else { Modifier::empty() }),
                );

            let connections: Vec<ListItem> = app_state
                .ssh_connections
                .iter()
                .map(|c| ListItem::new(format!("< {} >", c)).style(Style::default().fg(NEON_GREEN)))
                .collect();

            let connections_list = List::new(connections)
                .block(connections_block)
                .highlight_style(Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK))
                .highlight_symbol(">> ");

            frame.render_stateful_widget(connections_list, content_layout[1], &mut app_state.ssh_connections_state);
        }
        Some(3) => {
            // Add Connection Form
            let form_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .title("[ ADD_CONNECTION ]")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(DARKER_GREEN));

            let inner_area = form_block.inner(content_layout[1]);
            frame.render_widget(form_block, content_layout[1]);

            // Form layout with flex
            let form_layout = Layout::default()
                .direction(Direction::Vertical)
                .flex(Flex::SpaceBetween)
                .constraints([
                    Constraint::Min(3),    // Form fields area
                    Constraint::Length(1), // Helper text
                ])
                .split(inner_area);

            // Form fields area with flex
            let fields_layout = Layout::default()
                .direction(Direction::Vertical)
                .flex(Flex::SpaceAround)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .split(form_layout[0]);

            let field_names = ["[ NAME ]", "[ HOST ]", "[ USER ]"];
            for (i, field) in app_state.add_connection_form.fields.iter().enumerate() {
                let is_active = app_state.input_mode == InputMode::Editing
                    && app_state.add_connection_form.current_field == i;

                // Row layout with flex
                let row = Layout::default()
                    .direction(Direction::Horizontal)
                    .flex(Flex::Start)
                    .constraints([
                        Constraint::Length(15),
                        Constraint::Min(30),
                        Constraint::Length(2),
                    ])
                    .split(fields_layout[i]);

                let label = Paragraph::new(field_names[i])
                    .style(Style::default().fg(if is_active { HIGHLIGHT } else { DARKER_GREEN }))
                    .alignment(Alignment::Right);

                let input = Paragraph::new(field.as_str())
                    .style(Style::default().fg(if is_active { HIGHLIGHT } else { NEON_GREEN }))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Double)
                            .border_style(
                                Style::default()
                                    .fg(if is_active { HIGHLIGHT } else { DARKER_GREEN })
                                    .add_modifier(if is_active { Modifier::BOLD } else { Modifier::empty() })
                            ),
                    );

                frame.render_widget(label, row[0]);
                frame.render_widget(input, row[1]);

                if is_active && app_state.input_mode == InputMode::Editing {
                    frame.set_cursor_position(Position::new(
                        row[1].x + app_state.add_connection_form.cursor_positions[i] as u16 + 1,
                        row[1].y + 1,
                    ));
                }
            }

            // Helper text
            let helper_text = if app_state.input_mode == InputMode::Editing {
                "[ TAB: next field | SHIFT+TAB: prev field | ENTER: submit | ESC: cancel ]"
            } else {
                "[ ENTER: start editing ]"
            };

            let helper = Paragraph::new(helper_text)
                .style(Style::default().fg(DARKER_GREEN))
                .alignment(Alignment::Center);
            frame.render_widget(helper, form_layout[1]);
        }
        _ => {
            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .title("[ DETAILS ]")
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg(DARKER_GREEN)),
                content_layout[1],
            );
        }
    }
}