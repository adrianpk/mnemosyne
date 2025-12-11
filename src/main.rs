mod app;
mod document;

use std::env;
use std::io;
use std::path::Path;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let args: Vec<String> = env::args().collect();
    let mut app = if args.len() > 1 {
        let path = Path::new(&args[1]);
        App::from_file(path).unwrap_or_else(|e| {
            eprintln!("Error loading file: {}", e);
            std::process::exit(1);
        })
    } else {
        App::new()
    };

    while app.running {
        terminal.draw(|frame| {
            let chunks =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(frame.area());

            let left_panel = Paragraph::new("AI Assistant")
                .block(Block::default().title("Conversation").borders(Borders::ALL));

            let doc_content: String = app
                .document
                .paragraphs
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    if i == app.document.selected {
                        format!(">> [{}] {}", i + 1, p)
                    } else {
                        format!("   [{}] {}", i + 1, p)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n\n");

            let right_panel = Paragraph::new(doc_content)
                .block(Block::default().title("Document").borders(Borders::ALL));

            frame.render_widget(left_panel, chunks[0]);
            frame.render_widget(right_panel, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => app.quit(),
                KeyCode::Char('j') | KeyCode::Down => app.document.select_next(),
                KeyCode::Char('k') | KeyCode::Up => app.document.select_prev(),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
