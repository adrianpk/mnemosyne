use std::path::Path;

use tui_textarea::TextArea;

use crate::agent::{prompts, Suggestion};
use crate::document::Document;

pub struct Message {
    pub role: Role,
    pub content: String,
}

pub enum Role {
    User,
    Assistant,
}

pub enum AppMode {
    Normal,
    Review {
        suggestion: Suggestion,
        paragraph_index: usize,
    },
    /// Waiting for user to select an alternative (1, 2, 3...)
    SelectAlternative {
        alternatives: Vec<String>,
        paragraph_index: usize,
    },
    /// Editing a paragraph directly
    Edit {
        paragraph_index: usize,
        original: String,
    },
}

pub struct App<'a> {
    pub document: Document,
    pub running: bool,
    pub input: String,
    pub conversation: Vec<Message>,
    pub system_prompt: String,
    pub mode: AppMode,
    pub editor: Option<TextArea<'a>>,
    /// When true, all paragraphs are highlighted (full-document operations)
    pub highlight_all: bool,
    /// Scroll offset for conversation panel (0 = auto-scroll to bottom)
    pub conversation_scroll: usize,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        App {
            document: Document::new(),
            running: true,
            input: String::new(),
            conversation: Vec::new(),
            system_prompt: default_system_prompt(),
            mode: AppMode::Normal,
            editor: None,
            highlight_all: false,
            conversation_scroll: 0,
        }
    }

    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        Ok(App {
            document: Document::from_file(path)?,
            running: true,
            input: String::new(),
            conversation: Vec::new(),
            system_prompt: default_system_prompt(),
            mode: AppMode::Normal,
            editor: None,
            highlight_all: false,
            conversation_scroll: 0,
        })
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn enter_review(&mut self, suggestion: Suggestion, paragraph_index: usize) {
        self.mode = AppMode::Review {
            suggestion,
            paragraph_index,
        };
    }

    pub fn accept_suggestion(&mut self) {
        if let AppMode::Review { suggestion, paragraph_index } = &self.mode {
            self.document.paragraphs[*paragraph_index] = suggestion.replacement.clone();
            self.conversation.push(Message {
                role: Role::Assistant,
                content: String::from("Change accepted."),
            });
        }
        self.mode = AppMode::Normal;
    }

    pub fn reject_suggestion(&mut self) {
        if let AppMode::Review { .. } = &self.mode {
            self.conversation.push(Message {
                role: Role::Assistant,
                content: String::from("Change discarded."),
            });
        }
        self.mode = AppMode::Normal;
    }

    pub fn enter_select_alternative(&mut self, alternatives: Vec<String>, paragraph_index: usize) {
        self.mode = AppMode::SelectAlternative {
            alternatives,
            paragraph_index,
        };
    }

    pub fn apply_alternative(&mut self, choice: usize) -> bool {
        if let AppMode::SelectAlternative { alternatives, paragraph_index } = &self.mode {
            if choice > 0 && choice <= alternatives.len() {
                self.document.paragraphs[*paragraph_index] = alternatives[choice - 1].clone();
                self.conversation.push(Message {
                    role: Role::Assistant,
                    content: format!("Applied alternative [{}].", choice),
                });
                self.mode = AppMode::Normal;
                return true;
            }
        }
        false
    }

    pub fn cancel_select_alternative(&mut self) {
        if let AppMode::SelectAlternative { .. } = &self.mode {
            self.conversation.push(Message {
                role: Role::Assistant,
                content: String::from("Selection cancelled."),
            });
        }
        self.mode = AppMode::Normal;
    }

    pub fn enter_edit(&mut self) {
        use ratatui::style::{Color, Style};
        use ratatui::widgets::{Block, Borders};

        let paragraph_index = self.document.selected;
        let original = self.document.paragraphs[paragraph_index].clone();

        // Split text into sentences for easier editing
        let sentences = split_into_sentences(&original);
        let formatted = sentences.join("\n");

        let mut textarea = TextArea::default();
        textarea.insert_str(&formatted);
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Edit (Ctrl+S to apply)")
        );
        textarea.set_cursor_line_style(Style::default());
        textarea.set_style(Style::default().fg(Color::Rgb(187, 154, 247))); // purple like suggestions

        self.editor = Some(textarea);
        self.mode = AppMode::Edit {
            paragraph_index,
            original,
        };
    }

    pub fn accept_edit(&mut self) {
        if let AppMode::Edit { paragraph_index, .. } = &self.mode {
            if let Some(editor) = &self.editor {
                // Join lines back with spaces (sentences were split by newlines)
                let new_text = editor.lines().join(" ");
                self.document.paragraphs[*paragraph_index] = new_text;
                self.conversation.push(Message {
                    role: Role::Assistant,
                    content: String::from("Edit applied."),
                });
            }
        }
        self.editor = None;
        self.mode = AppMode::Normal;
    }

    pub fn cancel_edit(&mut self) {
        if let AppMode::Edit { .. } = &self.mode {
            self.conversation.push(Message {
                role: Role::Assistant,
                content: String::from("Edit cancelled."),
            });
        }
        self.editor = None;
        self.mode = AppMode::Normal;
    }
}

fn default_system_prompt() -> String {
    prompts::system_prompt()
}

/// Split text into sentences for line-by-line editing.
/// Uses simple heuristics: split on ". ", "? ", "! " followed by uppercase or end of string.
fn split_into_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    let mut i = 0;
    while i < len {
        current.push(chars[i]);

        // Check for sentence endings
        if (chars[i] == '.' || chars[i] == '?' || chars[i] == '!') && i + 1 < len {
            // Look ahead for space followed by uppercase (or end)
            if chars[i + 1] == ' ' {
                if i + 2 < len && chars[i + 2].is_uppercase() {
                    // End of sentence - include the period but not the space
                    sentences.push(current.trim().to_string());
                    current = String::new();
                    i += 2; // Skip the space, next iteration starts at uppercase
                    continue;
                }
            }
        }
        i += 1;
    }

    // Don't forget the last sentence
    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }

    sentences
}
