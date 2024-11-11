use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use std::io::stdout;

use crate::types::Project;
use crate::config::Settings;

use crate::tui::actions::AppAction;
use crate::tui::app::App;
use crate::tui::script_execution::{display_error_splash, run_script};
use crate::tui::utils::{prepare_terminal, restore_terminal};
use crate::tui::widgets::render_script_preview;

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
        let selection = run_tui_event_loop(&mut terminal, &mut app)?;

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

fn run_tui_event_loop(
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
                            if app.is_project_in_current_dir(project.name.as_deref().unwrap_or("")) {
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

            let visual_mapping = Vec::new();
            let items: Vec<ListItem> = grouped_scripts
                .iter()
                .map(|group| {
                    group
                        .iter()
                        .map(|script| {
                            let shortcut = script
                                .shortcut
                                .map(|c| format!("[{}] ", c))
                                .unwrap_or_default();

                            let icon = if app.show_emoji { script.icon() } else { None };

                            ListItem::new(Line::from(vec![
                                Span::styled(
                                    format!(
                                        "{}{} {}",
                                        icon.map(|s| format!("{} ", s)).unwrap_or_default(),
                                        shortcut,
                                        script.name
                                    ),
                                    Style::default()
                                        .fg(script.script_type.color(app.theme))
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(": "),
                                Span::raw(&script.command),
                            ]))
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect();

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
                KeyCode::Up | KeyCode::Char('k') => app.previous_script(),
                KeyCode::Down | KeyCode::Char('j') => app.next_script(),

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
