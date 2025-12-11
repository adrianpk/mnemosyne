mod app;
mod document;

use std::env;
use std::io;
use std::path::Path;

use crate::document::index_label;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
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

            let doc_lines: Vec<Line> = app
                .document
                .paragraphs
                .iter()
                .enumerate()
                .flat_map(|(i, p)| {
                    let style = if i == app.document.selected {
                        Style::default().add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default()
                    };
                    let index_style = Style::default().add_modifier(Modifier::DIM);
                    let line = Line::from(vec![
                        Span::styled(format!("[{}] ", index_label(i)), index_style),
                        Span::styled(p.clone(), style),
                    ]);
                    vec![line, Line::from("")]
                })
                .collect();

            let right_panel = Paragraph::new(doc_lines)
                .block(Block::default().title("Document").borders(Borders::ALL))
                .wrap(Wrap { trim: false })
                .scroll((app.document.scroll, 0));

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
