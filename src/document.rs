use std::fs;
use std::path::{Path, PathBuf};
use chrono::Local;

pub struct Document {
    pub paragraphs: Vec<String>,
    pub selected: usize,
    /// Path to the source file (for versioning)
    source_path: Option<PathBuf>,
    /// Directory where versions are stored
    version_dir: Option<PathBuf>,
    /// Current version index (0 = original, 1+ = versions)
    current_version: usize,
    /// Total number of versions available (including original)
    total_versions: usize,
}

impl Document {
    pub fn new() -> Self {
        Document {
            paragraphs: vec![String::new()],
            selected: 0,
            source_path: None,
            version_dir: None,
            current_version: 0,
            total_versions: 1,
        }
    }

    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let paragraphs: Vec<String> = content
            .split("\n\n")
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let paragraphs = if paragraphs.is_empty() {
            vec![String::new()]
        } else {
            paragraphs
        };

        // Setup version directory
        let version_dir = Self::get_version_dir(path);

        // Count existing versions
        let total_versions = Self::count_versions(&version_dir).unwrap_or(0) + 1; // +1 for current

        Ok(Document {
            paragraphs,
            selected: 0,
            source_path: Some(path.to_path_buf()),
            version_dir: Some(version_dir),
            current_version: total_versions - 1, // Start at latest version
            total_versions,
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

    /// Get the version directory path for a source file
    fn get_version_dir(source_path: &Path) -> PathBuf {
        let parent = source_path.parent().unwrap_or(Path::new("."));
        let filename = source_path.file_name().unwrap().to_string_lossy();
        parent.join(".mnemosyne").join("versions").join(filename.as_ref())
    }

    /// Count existing version files in the version directory
    fn count_versions(version_dir: &Path) -> std::io::Result<usize> {
        if !version_dir.exists() {
            return Ok(0);
        }
        let count = fs::read_dir(version_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .count();
        Ok(count)
    }

    /// Purge future versions (called when committing to a past version)
    fn purge_future_versions(&mut self) -> std::io::Result<()> {
        if self.current_version >= self.total_versions - 1 {
            return Ok(()); // Already at latest version, nothing to purge
        }

        let version_dir = match &self.version_dir {
            Some(d) => d,
            None => return Ok(()),
        };

        // If we're not at the latest version, discard all future versions (linear history)
        // Example: if current_version=2 (v3 displayed), total_versions=7 (v1-v7)
        // We want to keep v1-v3 and delete v4-v7
        // Saved files are: 000-*.txt (snapshot before v2), 001-*.txt (snapshot before v3), etc.
        // current_version=2 means we're AT v3, so we keep files 000 and 001, delete 002+

        // Delete version files from current_version onwards (these are snapshots BEFORE future versions)
        let versions_to_delete_from = self.current_version; // Keep files < current_version

        if let Ok(entries) = fs::read_dir(version_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // Extract version number from filename (format: "NNN-timestamp.txt")
                    if let Some(version_str) = filename.split('-').next() {
                        if let Ok(version_num) = version_str.parse::<usize>() {
                            // Delete this file if it's >= versions_to_delete_from
                            // These are snapshots for versions we're discarding
                            if version_num >= versions_to_delete_from {
                                let _ = fs::remove_file(&path);
                            }
                        }
                    }
                }
            }
        }

        // Reset total_versions to current position
        self.total_versions = self.current_version + 1;
        Ok(())
    }

    /// Save current document state as a version before making changes
    pub fn save_version(&mut self) -> std::io::Result<()> {
        if self.source_path.is_none() {
            return Ok(()); // No file loaded, skip versioning
        }

        // Purge future versions if we're making a change from the past
        self.purge_future_versions()?;

        let version_dir = self.version_dir.as_ref().unwrap().clone();

        // Create version directory if it doesn't exist
        fs::create_dir_all(&version_dir)?;

        // Generate version filename with timestamp
        let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let version_number = format!("{:03}", self.total_versions - 1); // 0-indexed saved versions
        let filename = format!("{}-{}.txt", version_number, timestamp);
        let version_path = version_dir.join(filename);

        // Write current paragraphs to version file
        let content = self.paragraphs.join("\n\n");
        fs::write(&version_path, content)?;

        // Update version tracking
        self.total_versions += 1;
        self.current_version = self.total_versions - 1;

        Ok(())
    }

    /// Write current state to source file
    pub fn save_to_file(&self) -> std::io::Result<()> {
        if let Some(source_path) = &self.source_path {
            let content = self.paragraphs.join("\n\n");
            fs::write(source_path, content)?;
        }
        Ok(())
    }

    /// Commit current state: save to file and purge future versions
    /// Use this when user manually saves (Ctrl+S) without making changes
    pub fn commit_current_state(&mut self) -> std::io::Result<()> {
        // Purge future versions if we're in the past
        self.purge_future_versions()?;
        // Save current state to file
        self.save_to_file()?;
        Ok(())
    }

    /// Undo to previous version
    pub fn undo(&mut self) -> std::io::Result<bool> {
        if !self.can_undo() {
            return Ok(false);
        }

        // Save current state before undoing (if at latest version)
        if self.current_version == self.total_versions - 1 {
            self.save_to_file()?;
        }

        self.current_version -= 1;
        self.load_version(self.current_version)?;

        // Save the loaded version to the source file so it persists
        self.save_to_file()?;
        Ok(true)
    }

    /// Redo to next version
    pub fn redo(&mut self) -> std::io::Result<bool> {
        if !self.can_redo() {
            return Ok(false);
        }

        self.current_version += 1;
        self.load_version(self.current_version)?;

        // Save the loaded version to the source file so it persists
        self.save_to_file()?;
        Ok(true)
    }

    /// Load a specific version from disk
    fn load_version(&mut self, version: usize) -> std::io::Result<()> {
        let content = if version == 0 {
            // Version 0 is the original file (no modifications yet)
            // Load the first saved version, or empty if none exists
            let version_dir = self.version_dir.as_ref().unwrap();
            if let Some(first_version) = self.get_version_path(0) {
                fs::read_to_string(first_version)?
            } else {
                // No versions saved yet, this shouldn't happen
                return Ok(());
            }
        } else if version == self.total_versions - 1 {
            // Latest version is the current file
            let source_path = self.source_path.as_ref().unwrap();
            fs::read_to_string(source_path)?
        } else {
            // Load from version file
            let version_path = self.get_version_path(version - 1).unwrap();
            fs::read_to_string(version_path)?
        };

        self.paragraphs = content
            .split("\n\n")
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if self.paragraphs.is_empty() {
            self.paragraphs = vec![String::new()];
        }

        // Adjust selected paragraph if out of bounds
        if self.selected >= self.paragraphs.len() {
            self.selected = self.paragraphs.len().saturating_sub(1);
        }

        Ok(())
    }

    /// Get path to a specific version file
    fn get_version_path(&self, version_index: usize) -> Option<PathBuf> {
        let version_dir = self.version_dir.as_ref()?;
        if !version_dir.exists() {
            return None;
        }

        let mut versions: Vec<_> = fs::read_dir(version_dir)
            .ok()?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect();

        versions.sort_by_key(|e| e.file_name());

        versions.get(version_index).map(|e| e.path())
    }

    /// Check if undo is possible
    pub fn can_undo(&self) -> bool {
        self.current_version > 0
    }

    /// Check if redo is possible
    pub fn can_redo(&self) -> bool {
        self.current_version < self.total_versions - 1
    }

    /// Get version info for display
    pub fn version_info(&self) -> String {
        format!("v{}/{}", self.current_version + 1, self.total_versions)
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

/// Convert a label character to its index (1-9 → 0-8, a-z → 9-34)
pub fn label_to_index(c: char) -> Option<usize> {
    if c.is_ascii_digit() && c != '0' {
        Some((c as usize) - ('1' as usize))
    } else if c.is_ascii_lowercase() {
        Some(9 + (c as usize) - ('a' as usize))
    } else {
        None
    }
}
