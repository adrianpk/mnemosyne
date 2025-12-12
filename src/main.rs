mod agent;
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

use crate::agent::{Agent, MockAgent, Prompt};
use crate::app::{App, AppMode};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let agent = MockAgent::new();

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
            // Global keybindings (work in any mode)
            match key.code {
                KeyCode::F(11) => {
                    app.quit();
                    continue;
                }
                KeyCode::Char('q') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    app.quit();
                    continue;
                }
                _ => {}
            }

            // Mode-specific keybindings
            match &app.mode {
                AppMode::Normal => match key.code {
                    KeyCode::Char('j') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.document.select_next()
                    }
                    KeyCode::Char('k') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.document.select_prev()
                    }
                    KeyCode::Down => app.document.select_next(),
                    KeyCode::Up => app.document.select_prev(),
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            let user_msg = app.input.clone();
                            app.conversation.push(crate::app::Message {
                                role: crate::app::Role::User,
                                content: user_msg.clone(),
                            });

                            let selected_idx = app.document.selected;
                            let selected_paragraph = &app.document.paragraphs[selected_idx];
                            let prompt = Prompt::new(
                                &app.system_prompt,
                                &user_msg,
                                selected_paragraph,
                            );
                            let suggestion = agent.suggest(&prompt);

                            app.conversation.push(crate::app::Message {
                                role: crate::app::Role::Assistant,
                                content: suggestion.explanation.clone(),
                            });

                            app.enter_review(suggestion, selected_idx);
                            app.input.clear();
                        }
                    }
                    _ => {}
                },
                AppMode::Review { .. } => match key.code {
                    KeyCode::Enter => app.accept_suggestion(),
                    KeyCode::Esc => app.reject_suggestion(),
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
