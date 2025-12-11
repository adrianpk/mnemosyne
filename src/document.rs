use std::fs;
use std::path::Path;

pub struct Document {
    pub paragraphs: Vec<String>,
    pub selected: usize,
    pub scroll: u16,
}

impl Document {
    pub fn new() -> Self {
        Document {
            paragraphs: vec![String::new()],
            selected: 0,
            scroll: 0,
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
            scroll: 0,
        })
    }

    pub fn select_next(&mut self) {
        if self.selected < self.paragraphs.len() - 1 {
            self.selected += 1;
            self.scroll = (self.selected as u16).saturating_sub(2);
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.scroll = (self.selected as u16).saturating_sub(2);
        }
    }
}

pub fn index_label(i: usize) -> String {
    if i < 9 {
        format!("{}", i + 1)
    } else if i < 9 + 26 {
        let c = (b'a' + (i - 9) as u8) as char;
        c.to_string()
    } else {
        let n = i - 9 - 26;
        let first = (b'a' + (n / 26) as u8) as char;
        let second = (b'a' + (n % 26) as u8) as char;
        format!("{}{}", first, second)
    }
}
