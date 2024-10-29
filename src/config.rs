use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub theme: Theme,
}

#[derive(Debug, Clone, Copy, Deserialize)]
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

// impl FromStr for Theme {
//     type Err = String;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.to_lowercase().as_str() {
//             "dark" => Ok(Theme::Dark),
//             "light" => Ok(Theme::Light),
//             "nocolor" => Ok(Theme::NoColor),
//             _ => Err(format!("Invalid theme: {}", s)),
//         }
//     }
// }

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
}
