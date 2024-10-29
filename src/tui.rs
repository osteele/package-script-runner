use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use scopeguard::defer;
use std::io::stdout;

use crate::project::Project;
use crate::script_type::{Script, ScriptType, SPECIAL_SCRIPTS};
use crate::{
    config::{Settings, Theme},
    project::create_project,
};

impl ScriptType {
    fn color(&self, theme: Theme) -> Color {
        match theme {
            Theme::NoColor => Color::Reset,
            Theme::Dark => match self {
                ScriptType::Build => Color::Rgb(255, 204, 0),
                ScriptType::Development => Color::Rgb(0, 255, 0),
                ScriptType::Test => Color::Rgb(0, 255, 255),
                ScriptType::Deployment => Color::Rgb(0, 191, 255),
                ScriptType::Format => Color::Rgb(191, 0, 255),
                ScriptType::Lint => Color::Rgb(255, 128, 0),
                ScriptType::Clean => Color::Rgb(192, 192, 192),
                ScriptType::Other => Color::White,
            },
            Theme::Light => match self {
                ScriptType::Build => Color::Rgb(204, 102, 0),
                ScriptType::Development => Color::Rgb(0, 153, 0),
                ScriptType::Test => Color::Rgb(0, 102, 204),
                ScriptType::Deployment => Color::Rgb(153, 0, 0),
                ScriptType::Format => Color::Rgb(102, 0, 204),
                ScriptType::Lint => Color::Rgb(204, 51, 0),
                ScriptType::Clean => Color::Rgb(64, 64, 64),
                ScriptType::Other => Color::Black,
            },
        }
    }
}

pub struct App<'a> {
    project: &'a Project,
    projects: &'a Vec<&'a Project>,
    theme: Theme,
    scripts: Vec<Script>,
    search_mode: bool,
    search_query: String,
    visible_script_indices: Vec<usize>,
    selected_project_state: ListState,
    selected_script_state: ListState,
}

impl<'a> App<'a> {
    pub fn new(
        project: &'a Project,
        projects: &'a Vec<&'a Project>,
        theme: Theme,
    ) -> anyhow::Result<Self> {
        let scripts = project.scripts()?;
        let filtered_indices: Vec<usize> = (0..scripts.len()).collect();

        let mut app = Self {
            project,
            projects: &projects,
            theme,
            scripts,
            search_mode: false,
            search_query: String::new(),
            selected_script_state: ListState::default(),
            visible_script_indices: filtered_indices,
            selected_project_state: ListState::default(),
        };

        app.selected_script_state.select(Some(0));
        if !app.projects.is_empty() {
            app.selected_project_state.select(Some(0));
        }
        Ok(app)
    }

    fn next(&mut self) {
        let i = match self.selected_script_state.selected() {
            Some(i) => (i + 1) % self.visible_script_indices.len(),
            None => 0,
        };
        self.selected_script_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.selected_script_state.selected() {
            Some(i) => (i + self.visible_script_indices.len() - 1) % self.visible_script_indices.len(),
            None => 0,
        };
        self.selected_script_state.select(Some(i));
    }

    #[allow(dead_code)]
    fn update_search(&mut self) {
        self.visible_script_indices = self
            .scripts
            .iter()
            .enumerate()
            .filter(|(_, script)| script.matches_search(&self.search_query))
            .map(|(i, _)| i)
            .collect();

        if !self.visible_script_indices.is_empty() {
            self.selected_script_state.select(Some(0));
        } else {
            self.selected_script_state.select(None);
        }
    }

    fn get_selected_script(&self) -> Option<&Script> {
        self.selected_script_state
            .selected()
            .and_then(|i| self.visible_script_indices.get(i))
            .map(|&i| &self.scripts[i])
    }

    fn next_project(&mut self) {
        let len = self.projects.len();
        if len == 0 {
            return;
        }
        let i = match self.selected_project_state.selected() {
            Some(i) => (i + 1) % len,
            None => 0,
        };
        self.set_project_by_index(i);
    }

    fn previous_project(&mut self) {
        let len = self.projects.len();
        if len == 0 {
            return;
        }
        let i = match self.selected_project_state.selected() {
            Some(i) => (i + len - 1) % len,
            None => 0,
        };
        self.set_project_by_index(i);
    }

    fn set_project_by_index(&mut self, i: usize) {
        self.project = &self.projects[i];
        self.scripts = self.project.scripts().context("error getting scripts").unwrap();
        self.visible_script_indices = (0..self.scripts.len()).collect();
        self.selected_project_state.select(Some(i));
        self.selected_script_state.select(Some(0));
    }

    #[allow(dead_code)]
    fn set_project(&'a mut self, project: &'a Project) {
        let i = self.projects.iter().position(|p| p.path == project.path).unwrap();
        self.selected_project_state.select(Some(i));
        self.project = project;
        self.scripts = self.project.scripts().context("error getting scripts").unwrap();
        self.visible_script_indices = (0..self.scripts.len()).collect();
    }

    // Add this helper method
    fn is_current_dir_project(&self, name: &str) -> bool {
        name == "Current Directory"
    }
}

fn render_script_preview(script: &Script, theme: Theme) -> Vec<Line> {
    vec![
        Line::from(vec![
            Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&script.name),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:?}", script.script_type),
                Style::default().fg(script.script_type.color(theme)),
            ),
        ]),
        Line::from(vec![
            Span::styled("Command: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&script.command),
        ]),
        Line::from(vec![
            Span::styled(
                "Description: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(
                script
                    .description
                    .as_deref()
                    .unwrap_or("No description available"),
            ),
        ]),
    ]
}

pub enum AppAction {
    Quit,
    RunScript(String),
}

pub fn run_app(project: &Project, settings: &Settings) -> Result<()> {
    // filter out projects where create_project returns None
    let project_owners = &settings
        .projects
        .iter()
        .filter_map(|(name, path)| create_project(name, path))
        .collect::<Vec<Project>>();
    // make a vec of references to the project owners
    let mut project_owners_refs = project_owners.iter().map(|p| p).collect::<Vec<&Project>>();

    // add project to the beginning of the list if it's not in the list
    if !project_owners_refs
        .iter()
        .any(|p| p.path.as_path() == project.path.as_path())
    {
        project_owners_refs.insert(0, project);
    }

    let mut app = App::new(project, &project_owners_refs, settings.theme)?;

    let mut stdout = stdout();
    enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    defer! {
        let  _ = disable_raw_mode();
        let _ = std::io::stdout().execute(LeaveAlternateScreen);
    }
    loop {
        // Get user selection
        let selection = run_app_loop(&mut terminal, &mut app)?;

        match selection {
            AppAction::Quit => break,
            AppAction::RunScript(script_name) => {
                if let Some(script) = app.scripts.iter().find(|s| s.name == script_name) {
                    // Run the script
                    let _ = disable_raw_mode();
                    let _ = std::io::stdout().execute(LeaveAlternateScreen);
                    let status = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&script.command)
                        .status()?;
                    enable_raw_mode()?;
                    // Wait for user input to continue or quit
                    println!("Press 'q' to quit or any other key to continue...");
                    if let Event::Key(key) = event::read()? {
                        if key.code == KeyCode::Char('q') {
                            return Ok(());
                        }
                    }
                    std::io::stdout().execute(EnterAlternateScreen)?;

                    if !status.success() {
                        if let Some(code) = status.code() {
                            display_error_splash(&mut terminal, code)?;
                        }
                    }
                }
            }
        }
    }

    let _ = restore_terminal(terminal);
    Ok(())
}

fn run_app_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<AppAction> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3), // Projects list
                        Constraint::Min(3),    // Scripts list
                        Constraint::Length(5), // Details
                        Constraint::Length(3), // Help
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // Projects list (if not empty)
            if !app.projects.is_empty() {
                let projects: Vec<ListItem> = app
                    .projects
                    .iter()
                    .map(|project| {
                        let style =
                            if app.is_current_dir_project(project.name.as_deref().unwrap_or("")) {
                                Style::default()
                                    .add_modifier(Modifier::BOLD)
                                    .add_modifier(Modifier::ITALIC)
                            } else {
                                Style::default().add_modifier(Modifier::BOLD)
                            };

                        ListItem::new(Line::from(vec![
                            Span::styled(project.name.as_deref().unwrap_or("").to_string(), style),
                            Span::raw(": "),
                            Span::raw(project.path.display().to_string()),
                        ]))
                    })
                    .collect();

                let projects_list = List::new(projects)
                    .block(
                        Block::default()
                            .title("Projects (Tab to switch)")
                            .borders(Borders::ALL),
                    )
                    .highlight_style(
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    );

                f.render_stateful_widget(projects_list, chunks[0], &mut app.selected_project_state);
            }

            // Search bar
            let search_text = if app.search_mode {
                format!("Search: {}", app.search_query)
            } else {
                "Press '/' to search".to_string()
            };

            let search_style = if app.search_mode {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            };

            f.render_widget(
                Paragraph::new(search_text).style(search_style),
                Rect::new(chunks[1].x, chunks[1].y, chunks[1].width, 1),
            );

            // Scripts list
            let items: Vec<ListItem> = app
                .visible_script_indices
                .iter()
                .map(|&i| {
                    let script = &app.scripts[i];
                    let is_priority = SPECIAL_SCRIPTS.contains(&script.name.as_str());
                    let shortcut = script
                        .shortcut
                        .map(|c| format!("[{}] ", c))
                        .unwrap_or_default();

                    let content = if i > 0
                        && is_priority
                            != SPECIAL_SCRIPTS.contains(&app.scripts[i - 1].name.as_str())
                    {
                        vec![
                            Line::from("───────────────────"),
                            Line::from(vec![
                                Span::styled(
                                    format!("{}{}", shortcut, script.name),
                                    Style::default()
                                        .fg(script.script_type.color(app.theme))
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(": "),
                                Span::raw(&script.command),
                            ]),
                        ]
                    } else {
                        vec![Line::from(vec![
                            Span::styled(
                                format!("{}{}", shortcut, script.name),
                                Style::default()
                                    .fg(script.script_type.color(app.theme))
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(": "),
                            Span::raw(&script.command),
                        ])]
                    };
                    ListItem::new(content)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("Scripts").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::DarkGray));

            f.render_stateful_widget(list, chunks[1], &mut app.selected_script_state);

            // Preview panel
            if let Some(script) = app.get_selected_script() {
                let preview = Paragraph::new(render_script_preview(script, app.theme))
                    .block(Block::default().title("Details").borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(preview, chunks[2]);
            }

            // Help footer
            let help_text = vec![Line::from(vec![
                Span::styled(
                    "Navigation: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw("↑/↓ Scripts, ←/→ Projects, "),
                Span::styled("Search: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("/, "),
                Span::styled("Select: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("Enter, "),
                Span::styled("Quit: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("q/Esc"),
            ])];
            let help = Paragraph::new(help_text)
                .block(Block::default().title("Help").borders(Borders::ALL))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(help, chunks[3]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                // Script navigation
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::Down | KeyCode::Char('j') => app.next(),

                // Project navigation
                KeyCode::Left => app.previous_project(),
                KeyCode::Right => {
                    app.next_project();
                    // if let Some((_, dirname)) = app.get_selected_project() {
                    //     println!("Switching to project");
                    //     return Ok(AppAction::SwitchProject(dirname.to_str().unwrap().to_string()));
                    // }
                }

                // Script selection
                KeyCode::Enter => {
                    if let Some(script) = app.get_selected_script() {
                        return Ok(AppAction::RunScript(script.name.clone()));
                    }
                }

                // Project switching
                // KeyCode::Char('\t') => {
                //     if let Some((name, dirname)) = app.get_selected_project() {
                //         if !app.is_current_dir_project(name) {
                //             return Ok(AppAction::SwitchProject(dirname.to_str().unwrap().to_string()));
                //         }
                //     }
                // }

                // Search and quit
                KeyCode::Char('/') => {
                    app.search_mode = true;
                    app.search_query.clear();
                }
                KeyCode::Char('q') | KeyCode::Esc => return Ok(AppAction::Quit),

                // Shortcut keys
                KeyCode::Char(c) => {
                    if let Some(script) = app.scripts.iter().find(|s| s.shortcut == Some(c)) {
                        return Ok(AppAction::RunScript(script.name.clone()));
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn display_error_splash(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    exit_code: i32,
) -> Result<()> {
    terminal.clear()?;

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default().title("Script Error").borders(Borders::ALL);
        let area = centered_rect(60, 20, size);
        f.render_widget(block, area);

        let text = vec![
            Line::from(vec![
                Span::raw("The script exited with code: "),
                Span::styled(
                    exit_code.to_string(),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from("Press any key to continue..."),
        ];

        let paragraph = Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    })?;

    // Wait for a key press
    loop {
        if let Event::Key(_) = event::read()? {
            break;
        }
    }

    Ok(())
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

#[allow(dead_code)]
fn restore_terminal(mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    terminal.show_cursor()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    Ok(())
}
