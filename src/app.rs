use std::path::{Path, PathBuf};

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
        is_full_document: bool,
    },
    /// Editing a paragraph directly
    Edit {
        paragraph_index: usize,
        original: String,
    },
    /// Browsing repeated terms (Repeats/Echoes results)
    BrowseRepeats {
        terms: Vec<String>,
        highlighted: Option<String>,
    },
    /// More operations menu (F12)
    MoreMenu,
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
    /// Path to the source document file (if loaded from file)
    pub file_path: Option<PathBuf>,
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
            file_path: None,
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
            file_path: Some(path.to_path_buf()),
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
            // Save version before making changes
            if let Err(e) = self.document.save_version() {
                self.conversation.push(Message {
                    role: Role::Assistant,
                    content: format!("Warning: Could not save version: {}", e),
                });
            }

            // Apply the change
            self.document.paragraphs[*paragraph_index] = suggestion.replacement.clone();

            // Save to file
            if let Err(e) = self.document.save_to_file() {
                self.conversation.push(Message {
                    role: Role::Assistant,
                    content: format!("Warning: Could not save file: {}", e),
                });
            }

            self.conversation.push(Message {
                role: Role::Assistant,
                content: format!("Change accepted. {}", self.document.version_info()),
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

    pub fn enter_select_alternative(&mut self, alternatives: Vec<String>, paragraph_index: usize, is_full_document: bool) {
        self.mode = AppMode::SelectAlternative {
            alternatives,
            paragraph_index,
            is_full_document,
        };
    }

    pub fn apply_alternative(&mut self, choice: usize) -> bool {
        if let AppMode::SelectAlternative { alternatives, paragraph_index, is_full_document } = &self.mode {
            if choice > 0 && choice <= alternatives.len() {
                // Save version before making changes
                if let Err(e) = self.document.save_version() {
                    self.conversation.push(Message {
                        role: Role::Assistant,
                        content: format!("Warning: Could not save version: {}", e),
                    });
                }

                if *is_full_document {
                    // Full document: parse numbered paragraphs from selected alternative
                    let selected = &alternatives[choice - 1];
                    let parsed = parse_numbered_paragraphs(selected);

                    if parsed.is_empty() {
                        self.conversation.push(Message {
                            role: Role::Assistant,
                            content: String::from("Warning: Could not parse paragraphs from alternative. Not applied."),
                        });
                        self.mode = AppMode::Normal;
                        return false;
                    }

                    // Allow different paragraph count - the document structure can change
                    // User can always undo (Ctrl+Z) if they don't like the result
                    self.document.paragraphs = parsed;

                    // Reset selection if it's now out of bounds
                    if self.document.selected >= self.document.paragraphs.len() {
                        self.document.selected = self.document.paragraphs.len().saturating_sub(1);
                    }
                } else {
                    // Single paragraph: apply directly
                    self.document.paragraphs[*paragraph_index] = alternatives[choice - 1].clone();
                }

                // Save to file
                if let Err(e) = self.document.save_to_file() {
                    self.conversation.push(Message {
                        role: Role::Assistant,
                        content: format!("Warning: Could not save file: {}", e),
                    });
                }

                self.conversation.push(Message {
                    role: Role::Assistant,
                    content: format!("Applied alternative [{}]. {}", choice, self.document.version_info()),
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
                // Save version before making changes
                if let Err(e) = self.document.save_version() {
                    self.conversation.push(Message {
                        role: Role::Assistant,
                        content: format!("Warning: Could not save version: {}", e),
                    });
                }

                // Join lines back with spaces (sentences were split by newlines)
                let new_text = editor.lines().join(" ");
                self.document.paragraphs[*paragraph_index] = new_text;

                // Save to file
                if let Err(e) = self.document.save_to_file() {
                    self.conversation.push(Message {
                        role: Role::Assistant,
                        content: format!("Warning: Could not save file: {}", e),
                    });
                }

                self.conversation.push(Message {
                    role: Role::Assistant,
                    content: format!("Edit applied. {}", self.document.version_info()),
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

    pub fn enter_browse_repeats(&mut self, terms: Vec<String>) {
        self.mode = AppMode::BrowseRepeats {
            terms,
            highlighted: None,
        };
    }

    pub fn highlight_term(&mut self, choice: usize) -> bool {
        if let AppMode::BrowseRepeats { terms, highlighted } = &mut self.mode {
            if choice > 0 && choice <= terms.len() {
                let term = terms[choice - 1].clone();
                *highlighted = Some(term);
                return true;
            }
        }
        false
    }

    pub fn exit_browse_repeats(&mut self) {
        self.mode = AppMode::Normal;
    }

    /// Toggle More menu (F12)
    pub fn toggle_more_menu(&mut self) {
        self.mode = match self.mode {
            AppMode::MoreMenu => AppMode::Normal,
            _ => AppMode::MoreMenu,
        };
    }

    /// Check if More menu is open
    pub fn is_more_menu_open(&self) -> bool {
        matches!(self.mode, AppMode::MoreMenu)
    }

    /// Get the currently highlighted term for rendering
    pub fn highlighted_term(&self) -> Option<&str> {
        if let AppMode::BrowseRepeats { highlighted, .. } = &self.mode {
            highlighted.as_deref()
        } else {
            None
        }
    }

    /// Undo to previous document version
    pub fn undo(&mut self) {
        match self.document.undo() {
            Ok(true) => {
                self.conversation.push(Message {
                    role: Role::Assistant,
                    content: format!("Undo successful. {}", self.document.version_info()),
                });
            }
            Ok(false) => {
                self.conversation.push(Message {
                    role: Role::Assistant,
                    content: String::from("No more versions to undo."),
                });
            }
            Err(e) => {
                self.conversation.push(Message {
                    role: Role::Assistant,
                    content: format!("Undo failed: {}", e),
                });
            }
        }
    }

    /// Redo to next document version
    pub fn redo(&mut self) {
        match self.document.redo() {
            Ok(true) => {
                self.conversation.push(Message {
                    role: Role::Assistant,
                    content: format!("Redo successful. {}", self.document.version_info()),
                });
            }
            Ok(false) => {
                self.conversation.push(Message {
                    role: Role::Assistant,
                    content: String::from("No more versions to redo."),
                });
            }
            Err(e) => {
                self.conversation.push(Message {
                    role: Role::Assistant,
                    content: format!("Redo failed: {}", e),
                });
            }
        }
    }
}

fn default_system_prompt() -> String {
    prompts::system_prompt()
}

/// Parse numbered paragraphs from LLM output format.
/// Expected format: "[N] paragraph text...\n\n[N+1] paragraph text..." where N is the paragraph number.
/// Returns a Vec of paragraph strings without the numbering prefix.
fn parse_numbered_paragraphs(text: &str) -> Vec<String> {
    text.split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .map(|p| {
            let trimmed = p.trim();
            // Try to strip leading [N] prefix (single bracket)
            if let Some(stripped) = trimmed.strip_prefix('[') {
                if let Some((_, after_bracket)) = stripped.split_once(']') {
                    return after_bracket.trim().to_string();
                }
            }
            // If no [N] prefix found, return as-is
            trimmed.to_string()
        })
        .collect()
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
