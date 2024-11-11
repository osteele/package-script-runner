use anyhow::Result;
use std::collections::HashMap;

use crate::cli::{Cli, Commands, ProjectsAction};
use crate::config::Settings;
use crate::execution::{run_script, run_script_with_env};
use crate::types::{find_synonym_script, Project, Script, SPECIAL_SCRIPTS};
use crate::themes::Theme;
use crate::tui::run_tui;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::Write;

impl Commands {
    pub fn execute(&self) -> Result<()> {
        match self {
            Commands::Projects { action } => action.execute(),
        }
    }
}

impl ProjectsAction {
    pub fn execute(&self) -> Result<()> {
        let mut settings = Settings::new()?;
        match self {
            ProjectsAction::Add { name, path } => {
                settings.add_project(name.clone(), path.clone())?;
                println!("Added project '{}' at '{}'", name, path.display());
            }
            ProjectsAction::Remove { name } => {
                settings.remove_project(name)?;
                println!("Removed project '{}'", name);
            }
            ProjectsAction::Rename { old_name, new_name } => {
                settings.rename_project(old_name, new_name.clone())?;
                println!("Renamed project '{}' to '{}'", old_name, new_name);
            }
            ProjectsAction::List => {
                println!("Saved projects:");
                for (name, path) in &settings.projects {
                    println!("  {} -> {}", name, path.display());
                }
            }
        }
        Ok(())
    }
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        if let Some(command) = self.command {
            return command.execute();
        }

        let settings = Settings::new()?;

        // Determine working directory
        let working_dir = if let Some(project) = &self.project {
            settings
                .get_project_path(project)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project))?
        } else {
            self.dir
                .clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap())
        };

        // Change to working directory
        std::env::set_current_dir(&working_dir)?;

        // Detect package manager
        let current_dir = std::env::current_dir()?;
        let project = Project::detect(&current_dir)
            .ok_or_else(|| anyhow::anyhow!("Could not detect package manager"))?;

        // Find scripts
        let scripts = project.scripts()?;

        if scripts.is_empty() {
            println!("No scripts found");
            return Ok(());
        }

        if self.list {
            return self.handle_list_flag(&scripts);
        }

        if self.script_command.is_some() {
            let exit_code = self.handle_direct_script_execution(&scripts, &project)?;
            std::process::exit(exit_code);
        }

        // Run interactive mode (TUI or CLI)
        self.run_interactive_mode(&project)
    }

    fn handle_list_flag(&self, scripts: &[Script]) -> Result<()> {
        println!("Available scripts:");
        for script in scripts {
            println!("  {} - {}", script.name, script.command);
            if let Some(desc) = &script.description {
                println!("    Description: {}", desc);
            }
            println!();
        }
        Ok(())
    }

    fn handle_direct_script_execution(&self, scripts: &[Script], project: &Project) -> Result<i32> {
        let command = self.script_command.as_ref().unwrap();
        let script_to_run = match command.as_str() {
            cmd if SPECIAL_SCRIPTS.contains(&cmd) => {
                if self.script.is_some() {
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
                if let Some(script_name) = &self.script {
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

        run_script_with_env(
            &project.package_manager,
            &script_to_run,
            &self.args,
            &env_vars,
        )
    }

    fn run_interactive_mode(&self, project: &Project) -> Result<()> {
        let mut mode = if self.tui { Mode::TUI } else { Mode::CLI };
        let settings = Settings::new()?;

        loop {
            match mode {
                Mode::TUI => {
                    run_tui(&project, &settings)?;
                    break;
                }
                Mode::CLI => {
                    let scripts = project.scripts()?;
                    if let Ok(Some(script)) =
                        run_cli_mode(&scripts, self.get_effective_theme(&settings))
                    {
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
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    CLI,
    TUI,
}

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
    println!("[t] Switch to TUI mode");

    print!("\nPress a key to select a command, or 'q' to quit> ");
    std::io::stdout().flush()?;

    // Read single keypress
    enable_raw_mode()?;
    if let Event::Key(key) = event::read()? {
        disable_raw_mode()?;
        match key.code {
            KeyCode::Char('t') => return Ok(Some("__TUI_MODE__".to_string())),
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
