use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, Paragraph, Wrap}, Frame, Terminal
};
use crossterm::event::{self, Event, KeyCode};

use crate::tui::widgets::render_script_preview;
use crate::tui::actions::AppAction;

use super::App;

fn draw_projects_list(
    f: &mut Frame,
    app: &mut App,
    area: Rect
) {
    if app.projects.is_empty() {
        return;
    }

    let projects: Vec<ListItem> = app
        .projects
        .iter()
        .map(|project| {
            let style = if app.is_project_in_current_dir(project.name.as_deref().unwrap_or("")) {
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
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(projects_list, area, &mut app.selected_project_state);
}

fn draw_scripts_list(
    f: &mut Frame,
    app: &mut App,
    area: Rect
) {
    let grouped_scripts = app
        .group_scripts()
        .into_iter()
        .map(|group| group.into_iter().map(|script| script.clone()).collect::<Vec<_>>())
        .collect::<Vec<_>>();

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

    let list = List::new(items)
        .block(
            Block::default()
                .title("Scripts (↑/↓ to navigate)")
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(list, area, &mut app.selected_script_state);
}

fn draw_script_preview(
    f: &mut Frame,
    app: &App,
    area: Rect
) {
    if let Some(script) = app.get_selected_script() {
        let preview = Paragraph::new(render_script_preview(script, app.theme, app.show_emoji))
            .block(Block::default().title("Details").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        f.render_widget(preview, area);
    }
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = vec![Line::from(vec![
        Span::styled("Navigation: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("↑/↓ Scripts, ←/→ Projects, "),
        Span::styled("Select: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Enter, "),
        Span::styled("Quit: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("q/Esc"),
    ])];
    let help = Paragraph::new(help_text)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(help, area);
}

fn draw_ui<W: std::io::Write>(
    terminal: &mut Terminal<CrosstermBackend<W>>,
    app: &mut App,
) -> Result<()> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(5),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.size());

        draw_projects_list(f, app, chunks[0]);
        draw_scripts_list(f, app, chunks[1]);
        draw_script_preview(f, app, chunks[2]);
        draw_help(f, chunks[3]);
    })?;

    Ok(())
}

pub fn run_event_loop<T: std::io::Write>(
    terminal: &mut Terminal<CrosstermBackend<T>>,
    app: &mut App,
) -> Result<AppAction> {
    loop {
        draw_ui(terminal, app)?;

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Up | KeyCode::Char('k') => app.previous_script(),
                KeyCode::Down | KeyCode::Char('j') => app.next_script(),
                KeyCode::Left => app.previous_project(),
                KeyCode::Right => app.next_project(),
                KeyCode::Enter => {
                    if let Some(script) = app.get_selected_script() {
                        return Ok(AppAction::RunScript(script.name.clone()));
                    }
                }
                KeyCode::Char('q') | KeyCode::Esc => return Ok(AppAction::Quit),
                KeyCode::Char(c) => {
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                        return Ok(AppAction::Quit);
                    }
                    if let Some(script) = app.scripts.iter().find(|s| s.shortcut == Some(c)) {
                        return Ok(AppAction::RunScript(script.name.clone()));
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
