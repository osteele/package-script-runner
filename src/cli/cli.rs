use crate::config::Settings;
use crate::themes::Theme;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "psr")]
#[command(author = "Oliver Steele <steele@osteele.com>")]
#[command(version)]
#[command(about = "A fast TUI-based script runner for Node.js and Deno projects", long_about = None)]
pub struct Cli {
    /// Start in a specific directory instead of current directory
    #[arg(short, long)]
    pub dir: Option<PathBuf>,

    /// Use a saved project by name
    #[arg(short = 'p', long = "project")]
    pub project: Option<String>,

    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// List available scripts without launching the TUI
    #[arg(short, long)]
    pub list: bool,

    /// Set the color theme (dark or light)
    #[arg(long)]
    pub theme: Option<Theme>,

    /// Return to TUI after running a script instead of exiting
    #[arg(long)]
    pub r#loop: bool,

    /// Command to execute (run, dev, test, etc)
    pub script_command: Option<String>,

    /// Script name (when using 'run' command)
    pub script: Option<String>,

    /// Additional arguments to pass to the script
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,

    /// Use TUI mode instead of command-line interface
    #[arg(long)]
    pub tui: bool,

    /// Subcommands for project management etc
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Parser)]
pub enum Commands {
    /// Manage saved projects
    Projects {
        #[command(subcommand)]
        action: ProjectsAction,
    },
}

#[derive(Parser)]
pub enum ProjectsAction {
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
    pub fn get_effective_theme(&self, settings: &Settings) -> Theme {
        settings.get_effective_theme(self.theme)
    }
}
