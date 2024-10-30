use ratatui::style::Color;
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;

use crate::script_type::{ScriptCategory, ScriptType};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
    NoColor,
}

impl FromStr for Theme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dark" => Ok(Theme::Dark),
            "light" => Ok(Theme::Light),
            "nocolor" => Ok(Theme::NoColor),
            _ => Err(format!("Invalid theme: {}", s)),
        }
    }
}

impl ScriptCategory {
    pub fn color(&self, theme: Theme) -> Color {
        match theme {
            Theme::NoColor => Color::Reset,
            Theme::Dark => match self {
                ScriptCategory::Build => Color::Rgb(255, 204, 0),
                ScriptCategory::Run => Color::Rgb(0, 255, 0),
                ScriptCategory::Development => Color::Rgb(0, 255, 0),
                ScriptCategory::Deployment => Color::Rgb(0, 191, 255),
                ScriptCategory::Other => Color::White,
            },
            Theme::Light => match self {
                ScriptCategory::Build => Color::Rgb(204, 102, 0), // Dark orange
                ScriptCategory::Run => Color::Rgb(0, 128, 0),     // Dark green
                ScriptCategory::Development => Color::Rgb(0, 153, 0), // Medium green
                ScriptCategory::Deployment => Color::Rgb(0, 102, 204), // Dark blue
                ScriptCategory::Other => Color::Black,
            },
        }
    }
}

impl ScriptType {
    pub fn color(&self, theme: Theme) -> Color {
        match theme {
            Theme::NoColor => Color::Reset,
            Theme::Dark => match self {
                ScriptType::Build => Color::Rgb(255, 204, 0),
                ScriptType::Format => Color::Rgb(191, 0, 255),
                ScriptType::Lint => Color::Rgb(255, 128, 0),
                ScriptType::Clean => Color::Rgb(192, 192, 192),
                ScriptType::Test => Color::Rgb(0, 255, 255),
                _ => self.category().color(theme),
            },
            Theme::Light => match self {
                ScriptType::Build => Color::Rgb(204, 102, 0), // Dark orange
                ScriptType::Format => Color::Rgb(102, 0, 204), // Dark purple
                ScriptType::Lint => Color::Rgb(204, 51, 0),   // Dark red-orange
                ScriptType::Test => Color::Rgb(0, 102, 204),  // Dark blue
                ScriptType::Clean => Color::Rgb(64, 64, 64),  // Dark gray
                _ => self.category().color(theme),
            },
        }
    }
}
