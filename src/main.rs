mod config;
mod execution;
mod package_managers;
mod project;
mod script_type;
mod themes;
mod tui;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use std::{collections::HashMap, io::Write, path::PathBuf};

use crate::config::Settings;
use crate::execution::{run_script, run_script_with_env};
use crate::package_managers::PackageManager;
use crate::project::{detect_project, Project};
use crate::script_type::{find_synonym_script, Script, SPECIAL_SCRIPTS};
use crate::themes::Theme;

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
    println!("Working directory: {}", std::env::current_dir()?.display());
    println!("Available scripts (press key to select):");

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

    // Finally print commands to the CLI itself
    if !remaining_scripts.is_empty() {
        println!("---");
    }
    // println!("[q] Quit");
    println!("[t] Switch to TUI mode");

    print!("\nPress a key to select a command, or 'q' to quit> ");
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
                anyhow::bail!(
                    "Cannot specify script name with special command '{}'",
                    command
                );
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
        _ => anyhow::bail!(
            "Unknown command '{}'. Use 'run <script>' for custom scripts",
            command
        ),
    };

    let mut env_vars = std::env::vars().collect::<HashMap<String, String>>();
    if command == "dev" && (script_to_run == "start" || script_to_run == "run") {
        env_vars.insert("NODE_ENV".to_string(), "dev".to_string());
    }

    run_script_with_env(&package_manager, &script_to_run, &cli.args, &env_vars)
}

fn run_interactive_mode(cli: &Cli, project: &Project) -> Result<()> {
    let mut mode = if cli.tui { Mode::TUI } else { Mode::CLI };
    let settings = Settings::new()?;

    loop {
        match mode {
            Mode::TUI => {
                run_tui_mode(cli, &project, &settings)?;
                break;
            }
            Mode::CLI => {
                let scripts = project.scripts()?;
                if let Ok(Some(script)) = run_cli_mode(
                    &scripts,
                    cli.get_effective_theme(&Settings::new().expect("Failed to load config")),
                ) {
                    if script == "__TUI_MODE__" {
                        mode = Mode::TUI;
                        continue;
                    }
                    let exit_code = run_script(&project.package_manager, &script, &[])?;
                    std::process::exit(exit_code);
                }
                break;
            }
        }
    }
    Ok(())
}

fn run_tui_mode(_cli: &Cli, project: &Project, settings: &Settings) -> Result<()> {
    tui::run_app(project, settings)
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
        cli.dir
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
    };

    // Change to working directory
    std::env::set_current_dir(&working_dir)?;

    // Detect package manager
    let current_dir = std::env::current_dir()?;
    let project = detect_project(&current_dir).context("Could not detect package manager")?;

    // Find scripts
    let scripts = project.scripts()?;

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
        let exit_code = handle_direct_script_execution(&cli, &scripts, &project.package_manager)?;
        std::process::exit(exit_code);
    }

    // Run interactive mode (TUI or CLI)
    run_interactive_mode(&cli, &project)
}
