use ratatui::style::Color;
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;

use crate::types::{Phase, ScriptType};

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

impl Phase {
    pub fn color(&self, theme: Theme) -> Color {
        match theme {
            Theme::NoColor => Color::Reset,
            Theme::Dark => match self {
                Phase::Development => Color::Rgb(0, 255, 0),      // Bright green
                Phase::Quality => Color::Rgb(255, 215, 0),        // Gold
                Phase::Build => Color::Rgb(255, 165, 0),          // Orange
                Phase::Dependencies => Color::Rgb(147, 112, 219),  // Medium purple
                Phase::Release => Color::Rgb(0, 191, 255),        // Deep sky blue
                Phase::Infrastructure => Color::Rgb(255, 99, 71),  // Tomato
                Phase::Unknown => Color::White,
            },
            Theme::Light => match self {
                Phase::Development => Color::Rgb(0, 128, 0),      // Dark green
                Phase::Quality => Color::Rgb(184, 134, 11),       // Dark goldenrod
                Phase::Build => Color::Rgb(205, 102, 0),          // Dark orange
                Phase::Dependencies => Color::Rgb(75, 0, 130),    // Indigo
                Phase::Release => Color::Rgb(0, 102, 204),        // Dark blue
                Phase::Infrastructure => Color::Rgb(178, 34, 34), // Firebrick
                Phase::Unknown => Color::Black,
            },
        }
    }
}

impl ScriptType {
    pub fn color(&self, theme: Theme) -> Color {
        match theme {
            Theme::NoColor => Color::Reset,
            Theme::Dark => match self {
                // Development
                Self::Serve => Color::Rgb(0, 255, 0),        // Bright green
                Self::Generate => Color::Rgb(50, 205, 50),   // Lime green
                Self::Migration => Color::Rgb(144, 238, 144), // Light green

                // Quality
                Self::Test => Color::Rgb(255, 215, 0),       // Gold
                Self::TestE2E => Color::Rgb(218, 165, 32),   // Goldenrod
                Self::Lint => Color::Rgb(255, 165, 0),       // Orange
                Self::TypeCheck => Color::Rgb(255, 140, 0),  // Dark orange
                Self::Format => Color::Rgb(255, 127, 80),    // Coral
                Self::Audit => Color::Rgb(255, 99, 71),      // Tomato

                // Build
                Self::Clean => Color::Rgb(169, 169, 169),    // Dark gray
                Self::Build | Self::BuildDev | Self::BuildProd => Color::Rgb(255, 165, 0), // Orange

                // Dependencies
                Self::Install | Self::Update | Self::Lock => Color::Rgb(147, 112, 219), // Medium purple

                // Release
                Self::Version => Color::Rgb(135, 206, 235),  // Sky blue
                Self::Publish => Color::Rgb(0, 191, 255),    // Deep sky blue
                Self::Deploy | Self::DeployStaging | Self::DeployProd => Color::Rgb(30, 144, 255), // Dodger blue

                // Infrastructure
                Self::DockerBuild | Self::DockerPush => Color::Rgb(255, 99, 71), // Tomato
                Self::Provision => Color::Rgb(233, 150, 122), // Dark salmon

                Self::Other => Color::White,
            },
            Theme::Light => self.phase().color(theme), // Use phase colors for light theme
        }
    }
}
