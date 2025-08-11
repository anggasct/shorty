use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    fs,
    io::{self, Stdout},
};

use crate::utils::get_aliases_path;

#[derive(Debug, Clone)]
struct Alias {
    name: String,
    command: String,
    note: Option<String>,
    tags: Vec<String>,
    line_number: usize,
}

#[derive(Debug, PartialEq)]
enum Screen {
    MainMenu,
    AliasBrowser,
    Settings,
    EditAlias,
    AddAlias,
    ConfirmDelete,
    #[allow(dead_code)]
    Help,
}

#[derive(Debug)]
struct App {
    screen: Screen,
    main_menu_state: ListState,
    alias_list_state: ListState,
    aliases: Vec<Alias>,
    filtered_aliases: Vec<usize>,
    search_input: String,
    edit_name: String,
    edit_command: String,
    edit_note: String,
    edit_tags: String,
    edit_index: Option<usize>,
    delete_index: Option<usize>,
    current_edit_field: usize,
    status_message: Option<String>,
    should_quit: bool,
    show_help: bool,
    search_focused: bool,
}

impl App {
    fn new() -> anyhow::Result<Self> {
        let mut app = Self {
            screen: Screen::MainMenu,
            main_menu_state: ListState::default(),
            alias_list_state: ListState::default(),
            aliases: Vec::new(),
            filtered_aliases: Vec::new(),
            search_input: String::new(),
            edit_name: String::new(),
            edit_command: String::new(),
            edit_note: String::new(),
            edit_tags: String::new(),
            edit_index: None,
            delete_index: None,
            current_edit_field: 0,
            status_message: None,
            should_quit: false,
            show_help: false,
            search_focused: false,
        };

        app.main_menu_state.select(Some(0));
        app.load_aliases()?;
        app.reset_filter();

        Ok(app)
    }

    fn load_aliases(&mut self) -> anyhow::Result<()> {
        let aliases_path = get_aliases_path();

        if !aliases_path.exists() {
            self.status_message =
                Some("No aliases file found. Create some aliases first.".to_string());
            return Ok(());
        }

        let content = fs::read_to_string(&aliases_path)?;
        self.aliases.clear();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(alias) = parse_alias_line(line, line_num + 1) {
                self.aliases.push(alias);
            }
        }

        Ok(())
    }

    fn reset_filter(&mut self) {
        self.filtered_aliases = (0..self.aliases.len()).collect();
    }

    fn apply_search_filter(&mut self) {
        if self.search_input.is_empty() {
            self.reset_filter();
        } else {
            let query = self.search_input.to_lowercase();

            // Check for field-specific search
            if let Some((field, search_term)) = parse_search_query(&query) {
                self.filtered_aliases = self
                    .aliases
                    .iter()
                    .enumerate()
                    .filter(|(_, alias)| match field {
                        "name" => alias.name.to_lowercase().contains(&search_term),
                        "cmd" | "command" => alias.command.to_lowercase().contains(&search_term),
                        "note" => alias
                            .note
                            .as_ref()
                            .is_some_and(|n| n.to_lowercase().contains(&search_term)),
                        "tag" | "tags" => alias
                            .tags
                            .iter()
                            .any(|t| t.to_lowercase().contains(&search_term)),
                        _ => false,
                    })
                    .map(|(i, _)| i)
                    .collect();
            } else {
                // Regular search across all fields
                self.filtered_aliases = self
                    .aliases
                    .iter()
                    .enumerate()
                    .filter(|(_, alias)| {
                        alias.name.to_lowercase().contains(&query)
                            || alias.command.to_lowercase().contains(&query)
                            || alias
                                .note
                                .as_ref()
                                .is_some_and(|n| n.to_lowercase().contains(&query))
                            || alias.tags.iter().any(|t| t.to_lowercase().contains(&query))
                    })
                    .map(|(i, _)| i)
                    .collect();
            }
        }
        self.alias_list_state
            .select(if self.filtered_aliases.is_empty() {
                None
            } else {
                Some(0)
            });
    }

    fn handle_main_menu_input(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                let current = self.main_menu_state.selected().unwrap_or(0);
                let new_index = if current > 0 { current - 1 } else { 2 };
                self.main_menu_state.select(Some(new_index));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let current = self.main_menu_state.selected().unwrap_or(0);
                let new_index = if current < 2 { current + 1 } else { 0 };
                self.main_menu_state.select(Some(new_index));
            }
            KeyCode::Enter | KeyCode::Char(' ') => match self.main_menu_state.selected() {
                Some(0) => {
                    self.screen = Screen::AliasBrowser;
                    if !self.filtered_aliases.is_empty() {
                        self.alias_list_state.select(Some(0));
                    }
                    self.search_focused = false;
                }
                Some(1) => {
                    self.screen = Screen::Settings;
                }
                Some(2) => {
                    self.should_quit = true;
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_alias_browser_input(&mut self, key: KeyCode) -> anyhow::Result<()> {
        if self.search_focused {
            match key {
                KeyCode::Esc => {
                    self.search_focused = false;
                    self.search_input.clear();
                    self.reset_filter();
                }
                KeyCode::Backspace => {
                    self.search_input.pop();
                    self.apply_search_filter();
                }
                KeyCode::Char(c) => {
                    self.search_input.push(c);
                    self.apply_search_filter();
                }
                KeyCode::Enter => {
                    self.search_focused = false;
                }
                _ => {}
            }
        } else {
            match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    if !self.filtered_aliases.is_empty() {
                        let current = self.alias_list_state.selected().unwrap_or(0);
                        let new_index = if current > 0 {
                            current - 1
                        } else {
                            self.filtered_aliases.len() - 1
                        };
                        self.alias_list_state.select(Some(new_index));
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.filtered_aliases.is_empty() {
                        let current = self.alias_list_state.selected().unwrap_or(0);
                        let new_index = if current < self.filtered_aliases.len() - 1 {
                            current + 1
                        } else {
                            0
                        };
                        self.alias_list_state.select(Some(new_index));
                    }
                }
                KeyCode::Char('/') | KeyCode::F(3) => {
                    self.search_focused = true;
                    self.search_input.clear();
                }
                KeyCode::Char('e') => {
                    if let Some(selected) = self.alias_list_state.selected() {
                        if selected < self.filtered_aliases.len() {
                            let alias_idx = self.filtered_aliases[selected];
                            self.edit_index = Some(alias_idx);
                            let alias = &self.aliases[alias_idx];
                            self.edit_name = alias.name.clone();
                            self.edit_command = alias.command.clone();
                            self.edit_note = alias.note.clone().unwrap_or_default();
                            self.edit_tags = alias.tags.join(", ");
                            self.current_edit_field = 0;
                            self.screen = Screen::EditAlias;
                        }
                    }
                }
                KeyCode::Char('a') => {
                    self.edit_name.clear();
                    self.edit_command.clear();
                    self.edit_note.clear();
                    self.edit_tags.clear();
                    self.current_edit_field = 0;
                    self.edit_index = None;
                    self.screen = Screen::AddAlias;
                }
                KeyCode::Char('d') => {
                    if let Some(selected) = self.alias_list_state.selected() {
                        if selected < self.filtered_aliases.len() {
                            let alias_idx = self.filtered_aliases[selected];
                            self.delete_index = Some(alias_idx);
                            self.screen = Screen::ConfirmDelete;
                        }
                    }
                }
                KeyCode::Char('r') => {
                    self.load_aliases()?;
                    self.reset_filter();
                }
                KeyCode::Esc => {
                    self.screen = Screen::MainMenu;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_edit_alias_input(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Tab => {
                self.current_edit_field = (self.current_edit_field + 1) % 4;
            }
            KeyCode::BackTab => {
                self.current_edit_field = if self.current_edit_field > 0 {
                    self.current_edit_field - 1
                } else {
                    3
                };
            }
            KeyCode::Enter => {
                if self.edit_name.trim().is_empty() || self.edit_command.trim().is_empty() {
                    self.status_message = Some("Name and command are required".to_string());
                    return Ok(());
                }

                if let Some(idx) = self.edit_index {
                    self.save_edit_alias(idx)?;
                } else {
                    self.save_new_alias()?;
                }

                self.screen = Screen::AliasBrowser;
            }
            KeyCode::Esc => {
                self.screen = Screen::AliasBrowser;
            }
            KeyCode::Backspace => match self.current_edit_field {
                0 => {
                    self.edit_name.pop();
                }
                1 => {
                    self.edit_command.pop();
                }
                2 => {
                    self.edit_note.pop();
                }
                3 => {
                    self.edit_tags.pop();
                }
                _ => {}
            },
            KeyCode::Char(c) => match self.current_edit_field {
                0 => self.edit_name.push(c),
                1 => self.edit_command.push(c),
                2 => self.edit_note.push(c),
                3 => self.edit_tags.push(c),
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_confirm_delete_input(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(idx) = self.delete_index {
                    self.delete_alias(idx)?;
                }
                self.delete_index = None;
                self.screen = Screen::AliasBrowser;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.delete_index = None;
                self.screen = Screen::AliasBrowser;
            }
            _ => {}
        }
        Ok(())
    }

    fn save_edit_alias(&mut self, index: usize) -> anyhow::Result<()> {
        let aliases_path = get_aliases_path();
        let content = std::fs::read_to_string(&aliases_path)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        let alias = &self.aliases[index];
        let line_idx = alias.line_number - 1;

        let tags_part = if self.edit_tags.trim().is_empty() {
            String::new()
        } else {
            format!(" #tags:{}", self.edit_tags.trim())
        };

        let note_part = if self.edit_note.trim().is_empty() {
            String::new()
        } else {
            format!(" # {}", self.edit_note.trim())
        };

        let new_line = format!(
            "alias {}='{}'{}{}",
            self.edit_name.trim(),
            self.edit_command.trim(),
            note_part,
            tags_part
        );

        if line_idx < lines.len() {
            lines[line_idx] = new_line;
        }

        std::fs::write(&aliases_path, lines.join("\n"))?;
        self.load_aliases()?;
        self.reset_filter();
        self.status_message = Some("Alias updated successfully".to_string());

        Ok(())
    }

    fn save_new_alias(&mut self) -> anyhow::Result<()> {
        let aliases_path = get_aliases_path();

        let tags_part = if self.edit_tags.trim().is_empty() {
            String::new()
        } else {
            format!(" #tags:{}", self.edit_tags.trim())
        };

        let note_part = if self.edit_note.trim().is_empty() {
            String::new()
        } else {
            format!(" # {}", self.edit_note.trim())
        };

        let new_line = format!(
            "alias {}='{}'{}{}",
            self.edit_name.trim(),
            self.edit_command.trim(),
            note_part,
            tags_part
        );

        let mut content = if aliases_path.exists() {
            std::fs::read_to_string(&aliases_path)?
        } else {
            String::new()
        };

        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&new_line);
        content.push('\n');

        std::fs::write(&aliases_path, content)?;
        self.load_aliases()?;
        self.reset_filter();
        self.status_message = Some("Alias added successfully".to_string());

        Ok(())
    }

    fn delete_alias(&mut self, index: usize) -> anyhow::Result<()> {
        let aliases_path = get_aliases_path();
        let content = std::fs::read_to_string(&aliases_path)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        let alias = &self.aliases[index];
        let line_idx = alias.line_number - 1;

        if line_idx < lines.len() {
            lines.remove(line_idx);
        }

        std::fs::write(&aliases_path, lines.join("\n"))?;
        self.load_aliases()?;
        self.reset_filter();
        self.status_message = Some("Alias deleted successfully".to_string());

        Ok(())
    }
}

pub fn run_interactive_mode() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        println!("Error: {err}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if app.should_quit {
            break;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // Handle global Ctrl shortcuts
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('f') if app.screen == Screen::AliasBrowser => {
                            app.search_focused = !app.search_focused;
                            if app.search_focused {
                                app.search_input.clear();
                            }
                        }
                        KeyCode::Char('n') if app.screen == Screen::AliasBrowser => {
                            app.edit_name.clear();
                            app.edit_command.clear();
                            app.edit_note.clear();
                            app.edit_tags.clear();
                            app.current_edit_field = 0;
                            app.edit_index = None;
                            app.screen = Screen::AddAlias;
                        }
                        KeyCode::Char('r') if app.screen == Screen::AliasBrowser => {
                            app.load_aliases()?;
                            app.reset_filter();
                        }
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') if app.screen == Screen::MainMenu => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('?') | KeyCode::F(1) => {
                            app.show_help = !app.show_help;
                        }
                        _ => match app.screen {
                            Screen::MainMenu => app.handle_main_menu_input(key.code)?,
                            Screen::AliasBrowser => app.handle_alias_browser_input(key.code)?,
                            Screen::EditAlias | Screen::AddAlias => {
                                app.handle_edit_alias_input(key.code)?
                            }
                            Screen::ConfirmDelete => app.handle_confirm_delete_input(key.code)?,
                            Screen::Settings => {
                                if key.code == KeyCode::Esc {
                                    app.screen = Screen::MainMenu;
                                }
                            }
                            Screen::Help => {
                                app.show_help = false;
                                app.screen = Screen::MainMenu;
                            }
                        },
                    }
                }
            }
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(size);

    match app.screen {
        Screen::MainMenu => render_main_menu(f, chunks[0], app),
        Screen::AliasBrowser => render_alias_browser(f, chunks[0], app),
        Screen::EditAlias => render_edit_screen(f, chunks[0], app, "Edit Alias"),
        Screen::AddAlias => render_edit_screen(f, chunks[0], app, "Add New Alias"),
        Screen::ConfirmDelete => render_delete_confirm(f, chunks[0], app),
        Screen::Settings => render_settings_screen(f, chunks[0], app),
        Screen::Help => render_help_screen(f, chunks[0], app),
    }

    render_status_bar(f, chunks[1], app);

    if app.show_help {
        render_help_popup(f, app);
    }
}

fn render_main_menu(f: &mut Frame, area: Rect, app: &mut App) {
    let menu_items = vec![
        ListItem::new("Browse Aliases"),
        ListItem::new("Settings"),
        ListItem::new("Exit"),
    ];

    let list = List::new(menu_items)
        .block(
            Block::default()
                .title(" Shorty Interactive Mode ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Blue)
                .fg(Color::White),
        );

    f.render_stateful_widget(list, area, &mut app.main_menu_state);
}

fn render_alias_browser(f: &mut Frame, area: Rect, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Render search bar
    let search_title = if app.search_focused {
        " Search (ESC to cancel) "
    } else {
        " Search (/ to focus) "
    };

    let search_style = if app.search_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let search_input = Paragraph::new(app.search_input.as_str()).block(
        Block::default()
            .title(search_title)
            .borders(Borders::ALL)
            .border_style(search_style),
    );

    f.render_widget(search_input, main_chunks[0]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    let items: Vec<ListItem> = app
        .filtered_aliases
        .iter()
        .map(|&i| {
            let alias = &app.aliases[i];
            let command_display = if alias.command.len() > 30 {
                format!("{}...", &alias.command[..27])
            } else {
                alias.command.clone()
            };

            if !app.search_input.is_empty() {
                // Create highlighted text
                let name_spans = highlight_text(&alias.name, &app.search_input);
                let cmd_spans = highlight_text(&command_display, &app.search_input);

                let mut spans = name_spans;
                spans.push(Span::raw(" → "));
                spans.extend(cmd_spans);

                ListItem::new(Line::from(spans))
            } else {
                let display = format!("{} → {}", alias.name, command_display);
                ListItem::new(display)
            }
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" Aliases ({}) ", app.filtered_aliases.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Blue)
                .fg(Color::White),
        );

    f.render_stateful_widget(list, chunks[0], &mut app.alias_list_state);

    render_alias_details(f, chunks[1], app);
}

fn render_alias_details(f: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(selected) = app.alias_list_state.selected() {
        if selected < app.filtered_aliases.len() {
            let alias_idx = app.filtered_aliases[selected];
            let alias = &app.aliases[alias_idx];

            let mut lines = vec![
                Line::from(vec![
                    Span::styled(
                        "Name: ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(&alias.name),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Command: ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(&alias.command),
                ]),
            ];

            if let Some(note) = &alias.note {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Note: ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(note),
                ]));
            }

            if !alias.tags.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Tags: ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(alias.tags.join(", ")),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Line: ", Style::default().fg(Color::Gray)),
                Span::raw(alias.line_number.to_string()),
            ]));

            Text::from(lines)
        } else {
            Text::from("No alias selected")
        }
    } else {
        Text::from("No alias selected")
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Details ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_edit_screen(f: &mut Frame, area: Rect, app: &App, _title: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    let fields = [
        (&app.edit_name, "Name"),
        (&app.edit_command, "Command"),
        (&app.edit_note, "Note (optional)"),
        (&app.edit_tags, "Tags (comma-separated, optional)"),
    ];

    for (i, (content, label)) in fields.iter().enumerate() {
        let style = if app.current_edit_field == i {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let input = Paragraph::new(content.as_str()).block(
            Block::default()
                .title(format!(" {label} "))
                .borders(Borders::ALL)
                .border_style(style),
        );

        f.render_widget(input, chunks[i]);
    }
}

fn render_delete_confirm(f: &mut Frame, area: Rect, app: &App) {
    if let Some(idx) = app.delete_index {
        if idx < app.aliases.len() {
            let alias = &app.aliases[idx];

            let confirm_area = centered_rect(60, 30, area);
            f.render_widget(Clear, confirm_area);

            let text = format!(
                "Delete alias '{}'?\n\nCommand: {}\n\nPress 'y' to confirm, 'n' or ESC to cancel",
                alias.name, alias.command
            );

            let popup = Paragraph::new(text)
                .block(
                    Block::default()
                        .title(" Confirm Delete ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red)),
                )
                .wrap(Wrap { trim: true });

            f.render_widget(popup, confirm_area);
        }
    }
}

fn render_settings_screen(f: &mut Frame, area: Rect, _app: &App) {
    let paragraph = Paragraph::new("Settings screen - Coming soon!\n\nPress ESC to go back").block(
        Block::default()
            .title(" Settings ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(paragraph, area);
}

fn render_help_screen(f: &mut Frame, area: Rect, _app: &App) {
    let help_text = vec![
        "Shorty Interactive Mode Help",
        "",
        "Global:",
        "  q/Ctrl+q - Quit",
        "  ?/F1     - Toggle help",
        "  ESC      - Go back/Cancel",
        "",
        "Navigation:",
        "  ↑/k      - Move up",
        "  ↓/j      - Move down",
        "  Enter    - Select/Activate",
        "  Tab      - Next field (in edit mode)",
        "",
        "Alias Browser:",
        "  /        - Focus search (or Ctrl+f)",
        "  e        - Edit selected alias",
        "  a        - Add new alias (or Ctrl+n)",
        "  d        - Delete selected alias",
        "  r        - Reload aliases (or Ctrl+r)",
        "",
        "Search:",
        "  Type to search, ESC to clear",
        "  Field search: name:git, cmd:status, tag:dev",
        "",
        "Edit/Add Mode:",
        "  Tab/Shift+Tab - Navigate fields",
        "  Enter         - Save",
        "  ESC           - Cancel",
    ];

    let paragraph = Paragraph::new(help_text.join("\n"))
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let status_text = match &app.status_message {
        Some(msg) => msg.clone(),
        None => match app.screen {
            Screen::MainMenu => {
                "Use arrow keys to navigate, Enter to select, q to quit".to_string()
            }
            Screen::AliasBrowser => {
                if app.search_focused {
                    "Search: type to filter | Field search: name:term, cmd:term, tag:term | ESC to cancel".to_string()
                } else {
                    "/ search | e edit | a add | d delete | r reload | Ctrl+f/n/r | ESC menu"
                        .to_string()
                }
            }
            Screen::EditAlias => {
                "Edit alias - Tab: next field, Shift+Tab: previous field, Enter: save, ESC: cancel"
                    .to_string()
            }
            Screen::AddAlias => {
                "Add alias - Tab: next field, Shift+Tab: previous field, Enter: save, ESC: cancel"
                    .to_string()
            }
            Screen::ConfirmDelete => "Confirm delete - y: yes, n/ESC: no".to_string(),
            Screen::Settings => "Settings - ESC to go back".to_string(),
            Screen::Help => "Help screen - Press any key to close".to_string(),
        },
    };

    let status = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(status, area);
}

fn render_help_popup(f: &mut Frame, _app: &App) {
    let area = centered_rect(80, 80, f.area());
    f.render_widget(Clear, area);

    let help_text = "Quick Help\n\n? - Toggle this help\nq - Quit\n↑↓ - Navigate\nEnter - Select";
    let popup = Paragraph::new(help_text).block(
        Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(popup, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn parse_alias_line(line: &str, line_number: usize) -> Option<Alias> {
    if !line.starts_with("alias ") {
        return None;
    }

    let eq_pos = line.find('=')?;
    let name = line[6..eq_pos].trim().to_string();
    let rest = &line[eq_pos + 1..];

    let mut command = String::new();
    let mut note = None;
    let mut tags = Vec::new();

    let rest = rest.trim();
    if let Some(stripped) = rest.strip_prefix('\'') {
        if let Some(end_quote) = stripped.find('\'') {
            command = stripped[..end_quote].to_string();
            let remaining = &rest[end_quote + 2..];
            parse_comments_and_tags(remaining, &mut note, &mut tags);
        }
    } else if let Some(stripped) = rest.strip_prefix('"') {
        if let Some(end_quote) = stripped.find('"') {
            command = stripped[..end_quote].to_string();
            let remaining = &rest[end_quote + 2..];
            parse_comments_and_tags(remaining, &mut note, &mut tags);
        }
    } else {
        let mut end = rest.len();
        for (i, ch) in rest.char_indices() {
            if ch == ' ' || ch == '#' {
                end = i;
                break;
            }
        }
        command = rest[..end].to_string();
        if end < rest.len() {
            parse_comments_and_tags(&rest[end..], &mut note, &mut tags);
        }
    }

    Some(Alias {
        name,
        command,
        note,
        tags,
        line_number,
    })
}

fn parse_comments_and_tags(text: &str, note: &mut Option<String>, tags: &mut Vec<String>) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }

    if let Some(tags_pos) = text.find("#tags:") {
        let tags_part = &text[tags_pos + 6..];
        *tags = tags_part.split(',').map(|s| s.trim().to_string()).collect();

        let note_part = text[..tags_pos].trim();
        if let Some(stripped) = note_part.strip_prefix('#') {
            let note_text = stripped.trim();
            if !note_text.is_empty() {
                *note = Some(note_text.to_string());
            }
        }
    } else if let Some(stripped) = text.strip_prefix('#') {
        let note_text = stripped.trim();
        if !note_text.is_empty() {
            *note = Some(note_text.to_string());
        }
    }
}

fn parse_search_query(query: &str) -> Option<(&str, String)> {
    if let Some(colon_pos) = query.find(':') {
        let field = &query[..colon_pos];
        let term = query[colon_pos + 1..].to_string();
        if !term.is_empty() {
            return Some((field, term));
        }
    }
    None
}

fn highlight_text(text: &str, search: &str) -> Vec<Span<'static>> {
    if search.is_empty() {
        return vec![Span::raw(text.to_string())];
    }

    let search_lower = search.to_lowercase();
    let text_lower = text.to_lowercase();
    let mut spans = Vec::new();
    let mut last_end = 0;

    // Find all matches
    let mut start = 0;
    while let Some(pos) = text_lower[start..].find(&search_lower) {
        let match_start = start + pos;
        let match_end = match_start + search.len();

        // Add text before match
        if match_start > last_end {
            spans.push(Span::raw(text[last_end..match_start].to_string()));
        }

        // Add highlighted match
        spans.push(Span::styled(
            text[match_start..match_end].to_string(),
            Style::default().bg(Color::Yellow).fg(Color::Black),
        ));

        last_end = match_end;
        start = match_end;
    }

    // Add remaining text
    if last_end < text.len() {
        spans.push(Span::raw(text[last_end..].to_string()));
    }

    if spans.is_empty() {
        vec![Span::raw(text.to_string())]
    } else {
        spans
    }
}
