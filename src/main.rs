mod agent;
mod app;
mod document;
mod ui;

use std::env;
use std::io;
use std::path::Path;
use std::time::Duration;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::agent::{
    Agent, LLMAgent, MockAgent, OpenAIProvider, Prompt, ResponseMode,
    prompts::{self, operations},
};
use crate::app::{App, AppMode, Message, Role};

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // Try to create LLM agent, fall back to mock
    let llm_agent = OpenAIProvider::new().ok().map(LLMAgent::new);
    let mock_agent = MockAgent::new();

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

    // NOTE: Show which agent is active
    if llm_agent.is_some() {
        app.conversation.push(Message {
            role: Role::Assistant,
            content: String::from("LLM agent ready (OpenAI)."),
        });
    } else {
        app.conversation.push(Message {
            role: Role::Assistant,
            content: String::from("No API key found. Using mock agent."),
        });
    }

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
                    KeyCode::Enter | KeyCode::F(3) | KeyCode::F(5) | KeyCode::F(6) | KeyCode::F(7) => {
                        let (user_input, operation_prompt, is_critique) = if key.code == KeyCode::F(3) {
                            (String::from("/critique"), operations::CRITIQUE, true)
                        } else if key.code == KeyCode::F(5) {
                            (String::from("/style"), operations::STYLE, false)
                        } else if key.code == KeyCode::F(6) {
                            (String::from("/grammar"), operations::GRAMMAR, false)
                        } else if key.code == KeyCode::F(7) {
                            (String::from("/rephrase"), operations::REPHRASE, false)
                        } else if app.input.starts_with("/critique") {
                            (app.input.clone(), operations::CRITIQUE, true)
                        } else if app.input.starts_with("/style") {
                            (app.input.clone(), operations::STYLE, false)
                        } else if app.input.starts_with("/grammar") {
                            (app.input.clone(), operations::GRAMMAR, false)
                        } else if app.input.starts_with("/rephrase") {
                            (app.input.clone(), operations::REPHRASE, false)
                        } else if !app.input.is_empty() {
                            (app.input.clone(), operations::GRAMMAR, false)
                        } else {
                            continue;
                        };
                        app.input.clear();

                        app.conversation.push(Message {
                            role: Role::User,
                            content: user_input.clone(),
                        });

                        let selected_idx = app.document.selected;
                        let selected_paragraph = &app.document.paragraphs[selected_idx];

                        if let Some(ref agent) = llm_agent {
                            let prompt = Prompt::new(
                                &prompts::system_prompt(),
                                operation_prompt,
                                selected_paragraph,
                            );

                            // Animated "Thinking" indicator
                            let dots = ["Thinking", "Thinking.", "Thinking..", "Thinking..."];
                            let mut dot_idx = 0;
                            app.conversation.push(Message {
                                role: Role::Assistant,
                                content: String::from(dots[dot_idx]),
                            });
                            terminal.draw(|frame| ui::draw(frame, &app))?;

                            let llm_future = agent.send(&prompt);
                            tokio::pin!(llm_future);

                            let mut ticker = tokio::time::interval(Duration::from_millis(300));
                            ticker.tick().await; // First tick is immediate

                            let response = loop {
                                tokio::select! {
                                    result = &mut llm_future => break result,
                                    _ = ticker.tick() => {
                                        dot_idx = (dot_idx + 1) % dots.len();
                                        if let Some(msg) = app.conversation.last_mut() {
                                            msg.content = String::from(dots[dot_idx]);
                                        }
                                        terminal.draw(|frame| ui::draw(frame, &app))?;
                                    }
                                }
                            };

                            // Remove "Thinking..."
                            app.conversation.pop();

                            match response {
                                Ok(response) => {
                                    // Show comments
                                    for comment in &response.comments {
                                        app.conversation.push(Message {
                                            role: Role::Assistant,
                                            content: comment.clone(),
                                        });
                                    }

                                    // Show alternatives (for Rephrase and similar)
                                    for (i, alt) in response.alternatives.iter().enumerate() {
                                        app.conversation.push(Message {
                                            role: Role::Assistant,
                                            content: format!("[{}] {}", i + 1, alt),
                                        });
                                    }

                                    if response.result.mode != ResponseMode::Critique
                                        && !response.result.text.is_empty()
                                    {
                                        let suggestion = response.to_suggestion(selected_paragraph);
                                        app.enter_review(suggestion, selected_idx);
                                    }
                                }
                                Err(e) => {
                                    app.conversation.push(Message {
                                        role: Role::Assistant,
                                        content: format!("Error: {}", e),
                                    });
                                }
                            }
                        } else {
                            // Use mock agent
                            let prompt = Prompt::new(
                                &app.system_prompt,
                                &user_input,
                                selected_paragraph,
                            );
                            let suggestion = mock_agent.suggest(&prompt);

                            app.conversation.push(Message {
                                role: Role::Assistant,
                                content: suggestion.explanation.clone(),
                            });

                            if !is_critique {
                                app.enter_review(suggestion, selected_idx);
                            }
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
