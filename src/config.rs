use config::{Config, ConfigError, File};
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
use std::str::FromStr;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub projects: HashMap<String, PathBuf>,
    #[serde(default = "default_show_emoji")]
    pub show_emoji: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
    NoColor,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let config_path = Self::get_config_path();

        let s = Config::builder()
            // Start with default values
            .set_default("theme", "dark")?
            // Add config file if it exists
            .add_source(File::from(config_path).required(false))
            .build()?;

        s.try_deserialize()
    }

    fn get_config_path() -> PathBuf {
        // First check current directory
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        let local_config = current_dir.join(".pkr.toml");

        if local_config.exists() {
            return local_config;
        }

        // Fall back to home directory
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".pkr.toml")
    }

    pub fn get_effective_theme(&self, cli_theme: Option<Theme>) -> Theme {
        // Priority order:
        // 1. CLI argument (if present)
        // 2. Environment variable NO_COLOR (if present)
        // 3. Environment variable PSR_THEME (if present)
        // 4. Config file setting
        // 5. Default (Dark)

        if let Some(theme) = cli_theme {
            return theme;
        }

        if std::env::var_os("NO_COLOR").is_some() {
            return Theme::NoColor;
        }

        if let Ok(env_theme) = std::env::var("PSR_THEME") {
            if let Ok(theme) = Theme::from_str(&env_theme) {
                return theme;
            }
        }

        self.theme
    }

    pub fn add_project(&mut self, name: String, path: PathBuf) -> Result<(), ConfigError> {
        if self.projects.contains_key(&name) {
            return Err(ConfigError::Message(format!("Project '{}' already exists", name)));
        }
        self.projects.insert(name, path);
        self.save()
    }

    pub fn rename_project(&mut self, old_name: &str, new_name: String) -> Result<(), ConfigError> {
        if !self.projects.contains_key(old_name) {
            return Err(ConfigError::Message(format!("Project '{}' not found", old_name)));
        }
        if self.projects.contains_key(&new_name) {
            return Err(ConfigError::Message(format!("Project '{}' already exists", new_name)));
        }

        if let Some(path) = self.projects.remove(old_name) {
            self.projects.insert(new_name.clone(), path);
            self.save()?;
            Ok(())
        } else {
            Err(ConfigError::Message(format!("Failed to rename project '{}'", old_name)))
        }
    }

    pub fn remove_project(&mut self, name: &str) -> Result<(), ConfigError> {
        if !self.projects.contains_key(name) {
            return Err(ConfigError::Message(format!("Project '{}' not found", name)));
        }
        self.projects.remove(name);
        self.save()
    }

    pub fn get_project_path(&self, name: &str) -> Option<&PathBuf> {
        self.projects.get(name)
    }

    fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::get_config_path();
        let toml = toml::to_string(&self)
            .map_err(|e| ConfigError::Message(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(&config_path, toml)
            .map_err(|e| ConfigError::Message(format!("Failed to write config: {}", e)))?;
        Ok(())
    }
}

fn default_show_emoji() -> bool {
    true
}
