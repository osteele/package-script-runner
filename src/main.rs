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
use serde::Deserialize;
use std::{
    collections::HashMap,
    env, fs,
    io::stdout,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

#[derive(Debug, Clone, Copy)]
enum ScriptType {
    Build,
    Development,
    Test,
    Deployment,
    Format,
    Clean,
    Other,
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
                ScriptType::Clean => Color::Rgb(192, 192, 192),
                ScriptType::Other => Color::White,
            },
            Theme::Light => match self {
                ScriptType::Build => Color::Rgb(204, 102, 0),
                ScriptType::Development => Color::Rgb(0, 153, 0),
                ScriptType::Test => Color::Rgb(0, 102, 204),
                ScriptType::Deployment => Color::Rgb(153, 0, 0),
                ScriptType::Format => Color::Rgb(102, 0, 204),
                ScriptType::Clean => Color::Rgb(64, 64, 64),
                ScriptType::Other => Color::Black,
            },
        }
    }

    fn from_script(name: &str, command: &str) -> Self {
        let text = format!("{} {}", name, command).to_lowercase();
        if text.contains("build") || text.contains("webpack") || text.contains("compile") {
            Self::Build
        } else if text.contains("dev") || text.contains("start") || text.contains("watch") {
            Self::Development
        } else if text.contains("test") || text.contains("jest") || text.contains("vitest") {
            Self::Test
        } else if text.contains("deploy") || text.contains("publish") {
            Self::Deployment
        } else if text.contains("format") || text.contains("lint") || text.contains("prettier") {
            Self::Format
        } else if text.contains("clean") || text.contains("clear") {
            Self::Clean
        } else {
            Self::Other
        }
    }
}

const PRIORITY_SCRIPTS: &[&str] = &[
    "dev",
    "start",
    "run",
    "build",
    "deploy",
    "clean",
    "watch",
    "test",
    "format",
    "typecheck",
];

#[derive(Deserialize)]
struct PackageJson {
    scripts: Option<HashMap<String, String>>,
    #[serde(default)]
    descriptions: HashMap<String, String>, // Optional script descriptions
}

#[derive(Clone)]
struct Script {
    name: String,
    command: String,
    description: Option<String>,
    shortcut: Option<char>,
    script_type: ScriptType,
}

impl Script {
    fn matches_search(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.name.to_lowercase().contains(&query)
            || self.command.to_lowercase().contains(&query)
            || self
                .description
                .as_ref()
                .map_or(false, |d| d.to_lowercase().contains(&query))
    }
}

enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Deno,
}

impl PackageManager {
    fn detect(dir: &Path) -> Option<Self> {
        // Check lock files first
        if dir.join("bun.lockb").exists() {
            return Some(Self::Bun);
        } else if dir.join("pnpm-lock.yaml").exists() {
            return Some(Self::Pnpm);
        } else if dir.join("yarn.lock").exists() {
            return Some(Self::Yarn);
        } else if dir.join("package-lock.json").exists() {
            return Some(Self::Npm);
        } else if dir.join("deno.lock").exists() {
            return Some(Self::Deno);
        }

        // Check config files as fallback
        if dir.join(".npmrc").exists() {
            return Some(Self::Npm);
        } else if dir.join(".yarnrc").exists() || dir.join(".yarnrc.yml").exists() {
            return Some(Self::Yarn);
        } else if dir.join(".npmrc").exists()
            && std::fs::read_to_string(dir.join(".npmrc"))
                .map_or(false, |content| content.contains("pnpm"))
        {
            return Some(Self::Pnpm);
        }

        None
    }

    fn run_command(&self, script: &str) -> Command {
        let mut cmd = match self {
            Self::Npm => {
                let mut c = Command::new("npm");
                c.arg("run");
                c
            }
            Self::Yarn => {
                let mut c = Command::new("yarn");
                c.arg("run");
                c
            }
            Self::Pnpm => {
                let mut c = Command::new("pnpm");
                c.arg("run");
                c
            }
            Self::Bun => {
                let mut c = Command::new("bun");
                c.arg("run");
                c
            }
            Self::Deno => {
                let mut c = Command::new("deno");
                c.arg("task");
                c
            }
        };
        cmd.arg(script);
        cmd
    }
}

fn find_package_json() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    let home_dir = dirs::home_dir()?;

    let mut current = current_dir.as_path();
    while current >= home_dir.as_path() {
        let package_json = current.join("package.json");
        if package_json.exists() {
            return Some(package_json);
        }
        current = current.parent()?;
    }
    None
}

fn parse_scripts(path: &Path) -> Result<Vec<Script>> {
    let content = fs::read_to_string(path)?;
    let package: PackageJson = serde_json::from_str(&content)?;

    let mut scripts = Vec::new();
    if let Some(script_map) = package.scripts {
        // Process priority scripts first
        for &priority in PRIORITY_SCRIPTS {
            if let Some(command) = script_map.get(priority) {
                let script_type = ScriptType::from_script(priority, command);
                scripts.push(Script {
                    name: priority.to_string(),
                    command: command.clone(),
                    description: package.descriptions.get(priority).cloned(),
                    shortcut: Some(priority.chars().next().unwrap()),
                    script_type,
                });
            }
        }

        // Process remaining scripts alphabetically
        let mut other_scripts: Vec<_> = script_map
            .iter()
            .filter(|(name, _)| !PRIORITY_SCRIPTS.contains(&name.as_str()))
            .collect();
        other_scripts.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (name, command) in other_scripts {
            let script_type = ScriptType::from_script(name, command);
            scripts.push(Script {
                name: name.clone(),
                command: command.clone(),
                description: package.descriptions.get(name).cloned(),
                shortcut: None,
                script_type,
            });
        }
    }
    Ok(scripts)
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
                    let is_priority = PRIORITY_SCRIPTS.contains(&script.name.as_str());
                    let shortcut = script
                        .shortcut
                        .map(|c| format!("[{}] ", c))
                        .unwrap_or_default();

                    let content = if i > 0
                        && is_priority
                            != PRIORITY_SCRIPTS.contains(&app.scripts[i - 1].name.as_str())
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

#[derive(Debug, Clone, Copy)]
enum Theme {
    Dark,
    Light,
    NoColor,
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

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// List available scripts without launching the TUI
    #[arg(short, long)]
    list: bool,

    /// Set the color theme (dark or light)
    #[arg(long, env = "PSR_THEME", default_value = "dark")]
    theme: Theme,

    /// Return to TUI after running a script instead of exiting
    #[arg(long)]
    r#loop: bool,

    /// Name of the script to run directly
    script: Option<String>,
}

impl Cli {
    fn get_effective_theme(&self) -> Theme {
        if env::var_os("NO_COLOR").is_some() {
            Theme::NoColor
        } else {
            self.theme
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let effective_theme = cli.get_effective_theme();

    // Change directory if specified
    if let Some(dir) = cli.dir {
        std::env::set_current_dir(dir)?;
    }

    // Find package.json
    let package_json = find_package_json().context("Could not find package.json")?;
    let scripts = parse_scripts(&package_json)?;

    if scripts.is_empty() {
        println!("No scripts found in package.json");
        return Ok(());
    }

    // If --list flag is provided, print scripts and exit
    if cli.list {
        println!("Available scripts:");
        for script in &scripts {
            println!("  {} - {}", script.name, script.command);
            if let Some(desc) = &script.description {
                println!("    Description: {}", desc);
            }
            println!();
        }
        return Ok(());
    }

    // Detect package manager
    let package_manager = PackageManager::detect(package_json.parent().unwrap())
        .context("Could not detect package manager")?;

    // If a script name is provided, run it directly
    if let Some(script_name) = cli.script {
        if let Some(script) = scripts.iter().find(|s| s.name == script_name) {
            return run_script(&package_manager, &script.name);
        } else {
            anyhow::bail!("Script '{}' not found", script_name);
        }
    }

    // Setup terminal
    stdout().execute(EnterAlternateScreen)?;

    // Run TUI
    loop {
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let mut app = App::new(scripts.clone(), effective_theme);
        let result = run_app(&mut terminal, &mut app);

        // Cleanup terminal
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;

        // Run selected script
        if let Ok(Some(script)) = result {
            run_script(&package_manager, &script)?;

            if !cli.r#loop {
                break;
            }

            // Re-setup terminal for next iteration
            stdout().execute(EnterAlternateScreen)?;
            enable_raw_mode()?;
        } else {
            break;
        }
    }

    Ok(())
}

fn run_script(package_manager: &PackageManager, script: &str) -> Result<()> {
    let status = package_manager
        .run_command(script)
        .status()
        .context("Failed to run script")?;

    if !status.success() {
        anyhow::bail!("Script failed with exit code: {}", status);
    }
    Ok(())
}
