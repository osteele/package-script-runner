use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::types::Script;
use crate::themes::Theme;

pub fn render_script_preview(script: &Script, theme: Theme, show_emoji: bool) -> Vec<Line> {
    vec![
        Line::from(vec![
            Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(
                "{} {}",
                if show_emoji {
                    script.icon().unwrap_or("")
                } else {
                    ""
                },
                script.name
            )),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:?}", script.phase),
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
