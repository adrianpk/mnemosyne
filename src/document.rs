use std::fs;
use std::path::Path;

pub struct Document {
    pub paragraphs: Vec<String>,
    pub selected: usize,
}

impl Document {
    pub fn new() -> Self {
        Document {
            paragraphs: vec![String::new()],
            selected: 0,
        }
    }

    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let paragraphs: Vec<String> = content
            .split("\n\n")
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Document {
            paragraphs: if paragraphs.is_empty() {
                vec![String::new()]
            } else {
                paragraphs
            },
            selected: 0,
        })
    }

    pub fn select_next(&mut self) {
        if self.selected < self.paragraphs.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
}
