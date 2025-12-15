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

        let mut textarea = TextArea::default();
        textarea.insert_str(&original);
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Edit (Ctrl+Enter to apply)")
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
                let new_text = editor.lines().join("\n");
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
