#[derive(Clone)]
pub struct Script {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub shortcut: Option<char>,
    pub script_type: ScriptType,
}

impl Script {
    pub fn matches_search(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.name.to_lowercase().contains(&query)
            || self.command.to_lowercase().contains(&query)
            || self
                .description
                .as_ref()
                .map_or(false, |d| d.to_lowercase().contains(&query))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ScriptType {
    Build,
    Development,
    Test,
    Deployment,
    Format,
    Lint,
    Clean,
    Other,
}

impl ScriptType {
    pub fn from_script(name: &str, command: &str) -> Self {
        let text = format!("{} {}", name, command).to_lowercase();
        if text.contains("build") || text.contains("webpack") || text.contains("compile") {
            Self::Build
        } else if text.contains("dev") || text.contains("start") || text.contains("watch") {
            Self::Development
        } else if text.contains("test") || text.contains("jest") || text.contains("vitest") {
            Self::Test
        } else if text.contains("deploy") || text.contains("publish") {
            Self::Deployment
        } else if text.contains("format") || text.contains("prettier") {
            Self::Format
        } else if text.contains("lint")
            || text.contains("eslint")
            || text.contains("stylelint")
            || text.contains("clippy")
            || text.contains("flake8")
            || text.contains("pylint")
            || text.contains("ruff") {
            Self::Lint
        } else if text.contains("clean") || text.contains("clear") {
            Self::Clean
        } else {
            Self::Other
        }
    }
}

pub const PRIORITY_SCRIPTS: &[&str] = &[
    "dev",
    "start",
    "run",
    "build",
    "deploy",
    "clean",
    "watch",
    "test",
    "format",
    "lint",
    "typecheck",
];
