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

use crate::themes::Theme;
use crate::{config::Settings, project::create_project, script_type::group_scripts};
use crate::{project::Project, script_type::Script};

struct App<'a> {
    project: &'a Project,
    projects: &'a Vec<&'a Project>,
    theme: Theme,
    scripts: Vec<Script>,
    visible_script_indices: Vec<usize>,
    selected_project_state: ListState,
    selected_script_state: ListState,
    show_emoji: bool,
    visual_to_script_index: Vec<Option<usize>>,
}

impl<'a> App<'a> {
    pub fn new(
        project: &'a Project,
        projects: &'a Vec<&'a Project>,
        theme: Theme,
        settings: &Settings,
    ) -> anyhow::Result<Self> {
        let scripts = project.scripts()?;
        let filtered_indices: Vec<usize> = (0..scripts.len()).collect();

        let mut app = Self {
            project,
            projects,
            theme,
            scripts,
            selected_script_state: ListState::default(),
            visible_script_indices: filtered_indices,
            selected_project_state: ListState::default(),
            show_emoji: settings.show_emoji,
            visual_to_script_index: Vec::new(),
        };

        app.selected_script_state.select(Some(0));
        if !app.projects.is_empty() {
            app.selected_project_state.select(Some(0));
        }
        Ok(app)
    }

    fn next(&mut self) {
        let i = match self.selected_script_state.selected() {
            Some(i) => {
                let mut next = (i + 1) % self.visual_to_script_index.len();
                // Skip dividers
                while next < self.visual_to_script_index.len()
                    && self.visual_to_script_index[next].is_none()
                {
                    next = (next + 1) % self.visual_to_script_index.len();
                }
                next
            }
            None => 0,
        };
        self.selected_script_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.selected_script_state.selected() {
            Some(i) => {
                let mut prev = if i == 0 {
                    self.visual_to_script_index.len() - 1
                } else {
                    i - 1
                };
                // Skip dividers
                while prev < self.visual_to_script_index.len()
                    && self.visual_to_script_index[prev].is_none()
                {
                    prev = if prev == 0 {
                        self.visual_to_script_index.len() - 1
                    } else {
                        prev - 1
                    };
                }
                prev
            }
            None => 0,
        };
        self.selected_script_state.select(Some(i));
    }

    fn get_selected_script(&self) -> Option<&Script> {
        self.selected_script_state
            .selected()
            .and_then(|i| self.visual_to_script_index.get(i))
            .and_then(|opt| opt.as_ref())
            .map(|&script_idx| &self.scripts[script_idx])
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
        self.scripts = self
            .project
            .scripts()
            .context("error getting scripts")
            .unwrap();
        self.visible_script_indices = (0..self.scripts.len()).collect();
        self.selected_project_state.select(Some(i));
        self.selected_script_state.select(Some(0));
    }

    #[allow(dead_code)]
    fn set_project(&'a mut self, project: &'a Project) {
        let i = self
            .projects
            .iter()
            .position(|p| p.path == project.path)
            .unwrap();
        self.selected_project_state.select(Some(i));
        self.project = project;
        self.scripts = self
            .project
            .scripts()
            .context("error getting scripts")
            .unwrap();
        self.visible_script_indices = (0..self.scripts.len()).collect();
    }

    fn group_scripts(&self) -> Vec<Vec<&Script>> {
        group_scripts(&self.scripts)
    }

    fn is_current_dir_project(&self, name: &str) -> bool {
        name == "Current Directory"
    }
}

fn render_script_preview(script: &Script, theme: Theme, show_emoji: bool) -> Vec<Line> {
    vec![
        Line::from(vec![
            Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(
                "{} {}",
                if show_emoji {
                    script.category.icon().unwrap_or("")
                } else {
                    ""
                },
                script.name
            )),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:?}", script.category),
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
    let project_owners = &settings
        .projects
        .iter()
        .filter_map(|(name, path)| create_project(name, path))
        .collect::<Vec<Project>>();
    let mut project_owners_refs = project_owners.iter().map(|p| p).collect::<Vec<&Project>>();

    // add project to the beginning of the list if it's not already in the list
    if !project_owners_refs
        .iter()
        .any(|p| p.path.as_path() == project.path.as_path())
    {
        project_owners_refs.insert(0, project);
    }

    let mut app = App::new(project, &project_owners_refs, settings.theme, settings)?;

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
                            .title("Projects (←/→ to switch)")
                            .borders(Borders::ALL),
                    )
                    .highlight_style(
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    );

                f.render_stateful_widget(projects_list, chunks[0], &mut app.selected_project_state);
            }

            // Scripts list - collect items before rendering
            let grouped_scripts = app
                .group_scripts()
                .into_iter()
                .map(|group| {
                    group
                        .into_iter()
                        .map(|script| script.clone())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            let mut visual_mapping = Vec::new();
            let items: Vec<ListItem> = {
                let mut items = Vec::new();

                for (group_idx, group) in grouped_scripts.iter().enumerate() {
                    if group_idx > 0 {
                        items.push(ListItem::new(Line::from("───────────────────")));
                        visual_mapping.push(None);
                    }

                    for (script_idx, script) in group.iter().enumerate() {
                        let shortcut = script
                            .shortcut
                            .map(|c| format!("[{}] ", c))
                            .unwrap_or_default();

                        let icon = if app.show_emoji { script.icon() } else { None };

                        items.push(ListItem::new(Line::from(vec![
                            Span::styled(
                                format!(
                                    "{}{}",
                                    icon.map(|s| format!("{} ", s)).unwrap_or_default(),
                                    shortcut
                                ),
                                Style::default()
                                    .fg(script.script_type.color(app.theme))
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(": "),
                            Span::raw(&script.command),
                        ])));
                        visual_mapping.push(Some(script_idx));
                    }
                }
                items
            };

            app.visual_to_script_index = visual_mapping;

            let list = List::new(items)
                .block(
                    Block::default()
                        .title("Scripts (↑/↓ to navigate)")
                        .borders(Borders::ALL),
                )
                .highlight_style(Style::default().bg(Color::DarkGray));

            f.render_stateful_widget(list, chunks[1], &mut app.selected_script_state);

            // Preview panel
            if let Some(script) = app.get_selected_script() {
                let preview =
                    Paragraph::new(render_script_preview(script, app.theme, app.show_emoji))
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
                KeyCode::Right => app.next_project(),

                // Script selection
                KeyCode::Enter => {
                    if let Some(script) = app.get_selected_script() {
                        return Ok(AppAction::RunScript(script.name.clone()));
                    }
                }

                KeyCode::Char('q') | KeyCode::Esc => return Ok(AppAction::Quit),

                // Script shortcut keys
                KeyCode::Char(c) => {
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(event::KeyModifiers::CONTROL)
                    {
                        return Ok(AppAction::Quit);
                    }
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
