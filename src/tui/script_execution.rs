use anyhow::Result;
use crossterm::{event::{self, Event, KeyCode}, terminal::enable_raw_mode};
use ratatui::{
  backend::CrosstermBackend, style::{Color, Modifier, Style}, text::{Span, Line}, widgets::{Block, Borders, Paragraph, Wrap}, Terminal
};

use crate::types::Script;
use super::utils::{restore_terminal, prepare_terminal, centered_rect};

pub fn run_script(script: &Script) -> Result<Option<i32>> {
    restore_terminal()?;
    let _guard = scopeguard::guard((), |_| {
        let _ = prepare_terminal();
    });

    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&script.command)
        .status()?;

    println!("Press 'q' to quit or any other key to continue...");
    enable_raw_mode()?;
    if let Event::Key(key) = event::read()? {
        if key.code == KeyCode::Char('q') {
            return Ok(None);
        }
    }
    if !status.success() {
        return Ok(status.code());
    }
    Ok(None)
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
