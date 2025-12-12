use std::path::Path;

use crate::agent::Suggestion;
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
}

pub struct App {
    pub document: Document,
    pub running: bool,
    pub input: String,
    pub conversation: Vec<Message>,
    pub system_prompt: String,
    pub mode: AppMode,
}

impl App {
    pub fn new() -> Self {
        App {
            document: Document::new(),
            running: true,
            input: String::new(),
            conversation: Vec::new(),
            system_prompt: default_system_prompt(),
            mode: AppMode::Normal,
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
}

fn default_system_prompt() -> String {
    String::from(
        "You are an expert proofreader and writing assistant. \
        Your role is to help improve text clarity, grammar, style, and flow. \
        Be concise and specific in your suggestions."
    )
}
