use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend, Terminal
};
use std::io::stdout;

use crate::types::Project;
use crate::config::Settings;

use crate::tui::actions::AppAction;
use crate::tui::app::App;
use crate::tui::script_execution::{display_error_splash, run_script};
use crate::tui::utils::{prepare_terminal, restore_terminal};

pub fn run_tui(project: &Project, settings: &Settings) -> Result<()> {
    let project_owners = &settings
        .projects
        .iter()
        .filter_map(|(name, path)| Project::create(name, path))
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
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    prepare_terminal()?;
    loop {
        let selection = super::run_event_loop(&mut terminal, &mut app)?;

        match selection {
            AppAction::Quit => break,
            AppAction::RunScript(script_name) => {
                if let Some(script) = app.scripts.iter().find(|s| s.name == script_name) {
                    let status_code = run_script(script)?;
                    terminal.draw(|_| {})?;
                    if let Some(code) = status_code {
                        display_error_splash(&mut terminal, code)?;
                    }
                }
            }
        }
    }

    restore_terminal()?;
    Ok(())
}
