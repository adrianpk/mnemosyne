mod app;
mod document;
mod ui;

use std::env;
use std::io;
use std::path::Path;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

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
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::F(11) => app.quit(),
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
