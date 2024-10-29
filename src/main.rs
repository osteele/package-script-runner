mod package_managers;
mod script_type;
mod config;

use anyhow::{Context, Result};
use clap::Parser;
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
use std::{
    collections::HashMap,
    io::{stdout, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::package_managers::{detect_package_manager_in_dir, PackageManager};
use crate::script_type::{Script, ScriptType, SPECIAL_SCRIPTS, find_synonym_script};
use crate::config::{Settings, Theme};

fn search_upwards_for_package_manager(dir: &Path) -> Option<(Box<dyn PackageManager>, PathBuf)> {
    let mut current_dir = dir;
    let home_dir = dirs::home_dir()?;

    while current_dir >= home_dir.as_path() {
        if let Some(pm) = detect_package_manager_in_dir(current_dir) {
            return Some((pm, current_dir.to_path_buf()));
        }
        current_dir = current_dir.parent()?;
    }

    None
}

struct App {
    scripts: Vec<Script>,
    state: ListState,
    search_mode: bool,
    search_query: String,
    filtered_indices: Vec<usize>,
    theme: Theme,
}

impl App {
    fn new(scripts: Vec<Script>, theme: Theme) -> Self {
        let filtered_indices: Vec<usize> = (0..scripts.len()).collect();
        let mut app = Self {
            scripts,
            state: ListState::default(),
            search_mode: false,
            search_query: String::new(),
            filtered_indices,
            theme,
        };
        app.state.select(Some(0));
        app
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.filtered_indices.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + self.filtered_indices.len() - 1) % self.filtered_indices.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn update_search(&mut self) {
        self.filtered_indices = self
            .scripts
            .iter()
            .enumerate()
            .filter(|(_, script)| script.matches_search(&self.search_query))
            .map(|(i, _)| i)
            .collect();

        if !self.filtered_indices.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    fn get_selected_script(&self) -> Option<&Script> {
        self.state
            .selected()
            .and_then(|i| self.filtered_indices.get(i))
            .map(|&i| &self.scripts[i])
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

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<Option<String>> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(5)].as_ref())
                .split(f.size());

            // Search bar
            // let search_block = Block::default()
            //     .borders(Borders::NONE)
            //     .style(Style::default());

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
                Rect::new(chunks[0].x, chunks[0].y, chunks[0].width, 1),
            );

            // Scripts list
            let items: Vec<ListItem> = app
                .filtered_indices
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

            f.render_stateful_widget(list, chunks[0], &mut app.state);

            // Preview panel
            if let Some(script) = app.get_selected_script() {
                let preview = Paragraph::new(render_script_preview(script, app.theme))
                    .block(Block::default().title("Details").borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(preview, chunks[1]);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            match (app.search_mode, key.code) {
                (true, KeyCode::Esc) => {
                    app.search_mode = false;
                    app.search_query.clear();
                    app.update_search();
                }
                (true, KeyCode::Backspace) => {
                    app.search_query.pop();
                    app.update_search();
                }
                (true, KeyCode::Char(c)) => {
                    app.search_query.push(c);
                    app.update_search();
                }
                (false, KeyCode::Char('q')) => return Ok(None),
                (false, KeyCode::Char('/')) => {
                    app.search_mode = true;
                    app.search_query.clear();
                }
                (false, KeyCode::Char('j')) | (false, KeyCode::Down) => app.next(),
                (false, KeyCode::Char('k')) | (false, KeyCode::Up) => app.previous(),
                (false, KeyCode::Enter) => {
                    if let Some(script) = app.get_selected_script() {
                        return Ok(Some(script.name.clone()));
                    }
                }
                (false, KeyCode::Char(c)) => {
                    if let Some(script) = app.scripts.iter().find(|s| s.shortcut == Some(c)) {
                        return Ok(Some(script.name.clone()));
                    }
                }
                (false, KeyCode::Esc) => return Ok(None),
                _ => {}
            }
        }
    }
}

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

impl FromStr for Theme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dark" => Ok(Theme::Dark),
            "light" => Ok(Theme::Light),
            "nocolor" => Ok(Theme::NoColor),
            _ => Err(format!("Invalid theme: {}", s)),
        }
    }
}

#[derive(Parser)]
#[command(name = "psr")]
#[command(author = "Oliver Steele <steele@osteele.com>")]
#[command(version)]
#[command(about = "A fast TUI-based script runner for Node.js and Deno projects", long_about = None)]
struct Cli {
    /// Start in a specific directory instead of current directory
    #[arg(short, long)]
    dir: Option<PathBuf>,

    /// Use a saved project by name
    #[arg(short = 'p', long = "project")]
    project: Option<String>,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// List available scripts without launching the TUI
    #[arg(short, long)]
    list: bool,

    /// Set the color theme (dark or light)
    #[arg(long)]
    theme: Option<Theme>,

    /// Return to TUI after running a script instead of exiting
    #[arg(long)]
    r#loop: bool,

    /// Command to execute (run, dev, test, etc)
    script_command: Option<String>,

    /// Script name (when using 'run' command)
    script: Option<String>,

    /// Additional arguments to pass to the script
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,

    /// Use TUI mode instead of command-line interface
    #[arg(long)]
    tui: bool,

    /// Subcommands for project management etc
    #[command(subcommand)]
    subcommand: Option<Commands>,
}

#[derive(Parser)]
enum Commands {
    /// Manage saved projects
    Projects {
        #[command(subcommand)]
        action: ProjectsAction,
    },
}

#[derive(Parser)]
enum ProjectsAction {
    /// Add a new project
    Add {
        /// Name of the project
        name: String,
        /// Path to the project directory
        path: PathBuf,
    },
    /// Remove a project
    Remove {
        /// Name of the project to remove
        name: String,
    },
    /// Rename a project
    Rename {
        /// Current name of the project
        old_name: String,
        /// New name for the project
        new_name: String,
    },
    /// List all saved projects
    List,
}

impl Cli {
    fn get_effective_theme(&self, settings: &Settings) -> Theme {
        settings.get_effective_theme(self.theme)
    }
}

// Add this new function
fn run_cli_mode(scripts: &[Script], _theme: Theme) -> Result<Option<String>> {
    println!("Available scripts (press key to select):");
    println!("[t] Switch to TUI mode");

    let mut numbered_scripts = Vec::new();

    // Print scripts with shortcuts first
    scripts
        .iter()
        .filter(|s| s.shortcut.is_some())
        .for_each(|script| {
            println!(
                "[{}] {} ({})",
                script.shortcut.unwrap(),
                script.name,
                script.command
            );
        });

    // Collect scripts without shortcuts
    let remaining_scripts: Vec<_> = scripts.iter().filter(|s| s.shortcut.is_none()).collect();

    // Print divider if we have numeric options
    if !remaining_scripts.is_empty() {
        println!("---");
    }

    // Print numbered options for remaining scripts (up to 9)
    remaining_scripts
        .iter()
        .take(9)
        .enumerate()
        .for_each(|(i, script)| {
            println!("[{}] {} ({})", i + 1, script.name, script.command);
            numbered_scripts.push(script);
        });

    if remaining_scripts.len() > 9 {
        println!("\nAdditional scripts (requires TUI mode):");
        remaining_scripts.iter().skip(9).for_each(|script| {
            println!("    {} ({})", script.name, script.command);
        });
    }

    print!("\nPress a key to select a script, or 'q' to quit> ");
    std::io::stdout().flush()?;

    // Read single keypress
    enable_raw_mode()?;
    if let Event::Key(key) = event::read()? {
        disable_raw_mode()?;
        match key.code {
            KeyCode::Char('t') => return Ok(Some("__TUI_MODE__".to_string())), // Special sentinel value
            KeyCode::Char('q') => return Ok(None),
            KeyCode::Char(c) => {
                // Check for letter shortcuts
                if let Some(script) = scripts.iter().find(|s| s.shortcut == Some(c)) {
                    return Ok(Some(script.name.clone()));
                }
                // Check for number shortcuts
                if let Some(digit) = c.to_digit(10) {
                    if digit > 0 && (digit as usize) <= numbered_scripts.len() {
                        return Ok(Some(numbered_scripts[digit as usize - 1].name.clone()));
                    }
                }
            }
            KeyCode::Esc => return Ok(None),
            _ => {}
        }
    }
    disable_raw_mode()?;

    Ok(None)
}

// Add near the top with other types
#[derive(Debug, Clone, Copy)]
enum Mode {
    CLI,
    TUI,
}

fn handle_list_flag(scripts: &[Script]) {
    println!("Available scripts:");
    for script in scripts {
        println!("  {} - {}", script.name, script.command);
        if let Some(desc) = &script.description {
            println!("    Description: {}", desc);
        }
        println!();
    }
}

fn handle_direct_script_execution(
    cli: &Cli,
    scripts: &[Script],
    package_manager: &Box<dyn PackageManager>,
) -> Result<i32> {
    let command = cli.script_command.as_ref().unwrap();
    let script_to_run = match command.as_str() {
        cmd if SPECIAL_SCRIPTS.contains(&cmd) => {
            if cli.script.is_some() {
                anyhow::bail!("Cannot specify script name with special command '{}'", command);
            }
            if let Some(script) = scripts.iter().find(|s| &s.name == command) {
                script.name.clone()
            } else if let Some(synonym) = find_synonym_script(&scripts, command) {
                synonym
            } else {
                anyhow::bail!("Script '{}' not found", command);
            }
        }
        "run" => {
            if let Some(script_name) = &cli.script {
                if let Some(script) = scripts.iter().find(|s| &s.name == script_name) {
                    script.name.clone()
                } else {
                    anyhow::bail!("Script '{}' not found", script_name);
                }
            } else {
                if let Some(script) = scripts.iter().find(|s| s.name == "run") {
                    script.name.clone()
                } else if let Some(synonym) = find_synonym_script(&scripts, "run") {
                    synonym
                } else {
                    anyhow::bail!("No script name provided and no 'run' script found");
                }
            }
        }
        _ => anyhow::bail!("Unknown command '{}'. Use 'run <script>' for custom scripts", command),
    };

    let mut env_vars = std::env::vars().collect::<HashMap<String, String>>();
    if command == "dev" && (script_to_run == "start" || script_to_run == "run") {
        env_vars.insert("NODE_ENV".to_string(), "dev".to_string());
    }

    run_script_with_env(&package_manager, &script_to_run, &cli.args, &env_vars)
}

fn run_interactive_mode(
    cli: &Cli,
    scripts: Vec<Script>,
    package_manager: &Box<dyn PackageManager>,
) -> Result<()> {
    let mut mode = if cli.tui { Mode::TUI } else { Mode::CLI };

    loop {
        match mode {
            Mode::TUI => {
                if let Some(exit_code) = run_tui_mode(cli, &scripts, package_manager)? {
                    std::process::exit(exit_code);
                }
                break;
            }
            Mode::CLI => {
                if let Ok(Some(script)) = run_cli_mode(&scripts, cli.get_effective_theme(&Settings::new().expect("Failed to load config"))) {
                    if script == "__TUI_MODE__" {
                        mode = Mode::TUI;
                        continue;
                    }
                    let exit_code = run_script(&package_manager, &script, &[])?;
                    std::process::exit(exit_code);
                }
                break;
            }
        }
    }
    Ok(())
}

fn run_tui_mode(
    cli: &Cli,
    scripts: &[Script],
    package_manager: &Box<dyn PackageManager>,
) -> Result<Option<i32>> {
    stdout().execute(EnterAlternateScreen)?;
    loop {
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let theme = cli.get_effective_theme(&Settings::new().expect("Failed to load config"));
        let mut app = App::new(scripts.to_vec(), theme);
        let result = run_app(&mut terminal, &mut app);

        // Cleanup terminal
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;

        // Run selected script
        if let Ok(Some(script)) = result {
            let exit_code = run_script(&package_manager, &script, &[])?;

            if cli.r#loop {
                if exit_code != 0 {
                    display_error_splash(&mut terminal, exit_code)?;
                }
                stdout().execute(EnterAlternateScreen)?;
                enable_raw_mode()?;
            } else {
                return Ok(Some(exit_code));
            }
        } else {
            break;
        }
    }
    Ok(None)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut settings = Settings::new()?;

    // Handle projects subcommand
    if let Some(Commands::Projects { action }) = &cli.subcommand {
        match action {
            ProjectsAction::Add { name, path } => {
                settings.add_project(name.clone(), path.clone())?;
                println!("Added project '{}' at '{}'", name, path.display());
            }
            ProjectsAction::Remove { name } => {
                settings.remove_project(name)?;
                println!("Removed project '{}'", name);
            }
            ProjectsAction::Rename { old_name, new_name } => {
                settings.rename_project(&old_name, new_name.clone())?;
                println!("Renamed project '{}' to '{}'", old_name, new_name);
            }
            ProjectsAction::List => {
                println!("Saved projects:");
                for (name, path) in &settings.projects {
                    println!("  {} -> {}", name, path.display());
                }
            }
        }
        return Ok(());
    }

    // Determine working directory
    let working_dir = if let Some(project) = &cli.project {
        settings
            .get_project_path(project)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project))?
    } else {
        cli.dir.clone().unwrap_or_else(|| std::env::current_dir().unwrap())
    };

    // Change to working directory
    std::env::set_current_dir(&working_dir)?;

    // Detect package manager
    let current_dir = std::env::current_dir()?;
    let (package_manager, project_dir) = search_upwards_for_package_manager(&current_dir)
        .context("Could not detect package manager")?;

    // Find scripts
    let scripts = package_manager.parse_scripts(&project_dir)?;

    if scripts.is_empty() {
        println!("No scripts found");
        return Ok(());
    }

    // Handle --list flag
    if cli.list {
        handle_list_flag(&scripts);
        return Ok(());
    }

    // Handle direct script execution
    if cli.script_command.is_some() {
        let exit_code = handle_direct_script_execution(&cli, &scripts, &package_manager)?;
        std::process::exit(exit_code);
    }

    // Run interactive mode (TUI or CLI)
    run_interactive_mode(&cli, scripts, &package_manager)
}

fn run_script(
    package_manager: &Box<dyn PackageManager>,
    script: &str,
    args: &[String],
) -> Result<i32> {
    let mut command = package_manager.run_command(script);
    command.args(args);

    let status = command.status().context("Failed to run script")?;

    Ok(status.code().unwrap_or(-1))
}

fn display_error_splash(
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

fn run_script_with_env(
    package_manager: &Box<dyn PackageManager>,
    script: &str,
    args: &[String],
    env_vars: &HashMap<String, String>,
) -> Result<i32> {
    let mut command = package_manager.run_command(script);
    command.args(args);
    command.envs(env_vars);

    let status = command.status().context("Failed to run script")?;

    Ok(status.code().unwrap_or(-1))
}
