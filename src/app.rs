use std::path::Path;

use crate::document::Document;

pub struct App {
    pub document: Document,
    pub running: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            document: Document::new(),
            running: true,
        }
    }

    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        Ok(App {
            document: Document::from_file(path)?,
            running: true,
        })
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
