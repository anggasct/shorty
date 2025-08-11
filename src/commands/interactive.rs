use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
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
    Search,
    Settings,
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
    status_message: Option<String>,
    should_quit: bool,
    show_help: bool,
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
            status_message: None,
            should_quit: false,
            show_help: false,
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

        self.status_message = Some(format!("Loaded {} aliases", self.aliases.len()));
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
                let new_index = if current > 0 { current - 1 } else { 3 };
                self.main_menu_state.select(Some(new_index));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let current = self.main_menu_state.selected().unwrap_or(0);
                let new_index = if current < 3 { current + 1 } else { 0 };
                self.main_menu_state.select(Some(new_index));
            }
            KeyCode::Enter | KeyCode::Char(' ') => match self.main_menu_state.selected() {
                Some(0) => {
                    self.screen = Screen::AliasBrowser;
                    if !self.filtered_aliases.is_empty() {
                        self.alias_list_state.select(Some(0));
                    }
                }
                Some(1) => {
                    self.screen = Screen::Search;
                    self.search_input.clear();
                }
                Some(2) => {
                    self.screen = Screen::Settings;
                }
                Some(3) => {
                    self.should_quit = true;
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_alias_browser_input(&mut self, key: KeyCode) -> anyhow::Result<()> {
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
            KeyCode::Char('/') => {
                self.screen = Screen::Search;
                self.search_input.clear();
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
        Ok(())
    }

    fn handle_search_input(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Enter => {
                self.apply_search_filter();
                self.screen = Screen::AliasBrowser;
                self.status_message =
                    Some(format!("Found {} matches", self.filtered_aliases.len()));
            }
            KeyCode::Esc => {
                self.search_input.clear();
                self.reset_filter();
                self.screen = Screen::AliasBrowser;
            }
            KeyCode::Backspace => {
                self.search_input.pop();
                self.apply_search_filter();
            }
            KeyCode::Char(c) => {
                self.search_input.push(c);
                self.apply_search_filter();
            }
            _ => {}
        }
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
                        Screen::Search => app.handle_search_input(key.code)?,
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
        Screen::Search => render_search_screen(f, chunks[0], app),
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
        ListItem::new("Search Aliases"),
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
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let items: Vec<ListItem> = app
        .filtered_aliases
        .iter()
        .map(|&i| {
            let alias = &app.aliases[i];
            let display = format!(
                "{} → {}",
                alias.name,
                if alias.command.len() > 30 {
                    format!("{}...", &alias.command[..27])
                } else {
                    alias.command.clone()
                }
            );
            ListItem::new(display)
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

fn render_search_screen(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let search_input = Paragraph::new(app.search_input.as_str()).block(
        Block::default()
            .title(" Search Aliases ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(search_input, chunks[0]);

    let items: Vec<ListItem> = app
        .filtered_aliases
        .iter()
        .map(|&i| {
            let alias = &app.aliases[i];
            ListItem::new(format!("{} → {}", alias.name, alias.command))
        })
        .collect();

    let results = List::new(items).block(
        Block::default()
            .title(format!(" Results ({}) ", app.filtered_aliases.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );

    f.render_widget(results, chunks[1]);
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
        "  q        - Quit (from main menu)",
        "  ?/F1     - Toggle help",
        "  ESC      - Go back/Cancel",
        "",
        "Navigation:",
        "  ↑/k      - Move up",
        "  ↓/j      - Move down",
        "  Enter    - Select/Activate",
        "",
        "Alias Browser:",
        "  /        - Start search",
        "  r        - Reload aliases",
        "",
        "Search:",
        "  Type     - Filter aliases",
        "  Enter    - Apply search",
        "  ESC      - Clear search",
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
                "Browse aliases - Press / to search, r to reload, ESC for menu".to_string()
            }
            Screen::Search => "Type to search, Enter to apply, ESC to cancel".to_string(),
            Screen::Settings => "Settings - ESC to go back".to_string(),
            Screen::Help => "Help screen - Press any key to close".to_string(),
        },
    };

    let status = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        )
        .style(Style::default().fg(Color::Gray));

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
    if rest.starts_with('\'') {
        if let Some(end_quote) = rest[1..].find('\'') {
            command = rest[1..end_quote + 1].to_string();
            let remaining = &rest[end_quote + 2..];
            parse_comments_and_tags(remaining, &mut note, &mut tags);
        }
    } else if rest.starts_with('"') {
        if let Some(end_quote) = rest[1..].find('"') {
            command = rest[1..end_quote + 1].to_string();
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
        if note_part.starts_with('#') {
            let note_text = note_part[1..].trim();
            if !note_text.is_empty() {
                *note = Some(note_text.to_string());
            }
        }
    } else if text.starts_with('#') {
        let note_text = text[1..].trim();
        if !note_text.is_empty() {
            *note = Some(note_text.to_string());
        }
    }
}
