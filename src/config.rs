use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub api_key: Option<String>,
}

impl Config {
    /// Get config file path: ~/.config/mnemosyne/config.toml
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("mnemosyne").join("config.toml"))
    }

    /// Load config from file, or return default if file doesn't exist
    pub fn load() -> Self {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Config::default(),
        };

        if !path.exists() {
            return Config::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Config::default(),
        }
    }

    /// Save config to file, creating directory if needed
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Config directory not found")
        })?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize config
        let content = toml::to_string_pretty(self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;

        // Write to file
        fs::write(&path, content)?;

        // Set restrictive permissions (600 - owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    /// Get API key with priority: config file > OPENAI_MNEMOSYNE_API_KEY > OPENAI_API_KEY
    pub fn get_api_key() -> Option<String> {
        // Priority 1: Config file
        let config = Self::load();
        if let Some(key) = config.api_key {
            if !key.is_empty() {
                return Some(key);
            }
        }

        // Priority 2: OPENAI_MNEMOSYNE_API_KEY
        if let Ok(key) = env::var("OPENAI_MNEMOSYNE_API_KEY") {
            if !key.is_empty() {
                return Some(key);
            }
        }

        // Priority 3: OPENAI_API_KEY
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            if !key.is_empty() {
                return Some(key);
            }
        }

        None
    }
}
