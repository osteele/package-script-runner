#[derive(Clone)]
pub struct Script {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub category: ScriptCategory,
    pub script_type: ScriptType,
    pub shortcut: Option<char>,
}

impl Script {
    pub fn new(
        name: &str,
        command: &str,
        description: Option<String>,
        script_type: Option<ScriptType>,
        shortcut: Option<char>,
    ) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            description,
            category: ScriptCategory::from_script(name, command),
            script_type: script_type.unwrap_or(ScriptType::from_script(name, command)),
            shortcut,
        }
    }

    pub fn icon(&self) -> Option<&'static str> {
        self.script_type.icon()
    }

    #[allow(dead_code)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ScriptCategory {
    Development,
    Deployment,
    Build,
    Run,
    Other,
}

impl ScriptCategory {
    pub fn from_script(name: &str, command: &str) -> Self {
        ScriptType::from_script(name, command).category()
    }

    #[allow(dead_code)]
    pub fn shortcut(name: &str) -> Option<char> {
        match name {
            "clean" => Some('x'),
            "deployment" => Some('p'),
            s => {
                if SPECIAL_SCRIPTS.contains(&s) {
                    Some(s.chars().next().unwrap())
                } else {
                    None
                }
            }
        }
    }

    pub fn icon(&self) -> Option<&'static str> {
        match self {
            ScriptCategory::Development => Some("🔨"),
            ScriptCategory::Deployment => Some("📦"),
            ScriptCategory::Build => Some("🔨"),
            ScriptCategory::Run => Some("▶️"),
            ScriptCategory::Other => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ScriptType {
    Build,
    Check,
    Clean,
    DevRun,
    Deploy,
    Fix,
    Format,
    Lint,
    Publish,
    Run,
    Test,
    Other,
}

impl ScriptType {
    pub fn from_script(name: &str, command: &str) -> Self {
        let text = format!("{} {}", name, command).to_lowercase();
        if text.contains("build") || text.contains("webpack") || text.contains("compile") {
            Self::Build
        } else if text.contains("check") {
            Self::Check
        } else if text.contains("dev")
            || text.contains("start")
            || text.contains("watch")
            || text.contains("run")
        {
            Self::Run
        } else if text.contains("test") || text.contains("jest") || text.contains("vitest") {
            Self::Test
        } else if text.contains("deploy") {
            Self::Deploy
        } else if text.contains("publish") {
            Self::Publish
        } else if text.contains("format") || text.contains("prettier") {
            Self::Format
        } else if text.contains("lint")
            || text.contains("eslint")
            || text.contains("stylelint")
            || text.contains("clippy")
            || text.contains("flake8")
            || text.contains("pylint")
            || text.contains("ruff")
        {
            Self::Lint
        } else if text.contains("clean") || text.contains("clear") {
            Self::Clean
        } else {
            Self::Other
        }
    }

    pub fn category(&self) -> ScriptCategory {
        match self {
            Self::Build => ScriptCategory::Build,
            Self::Check => ScriptCategory::Development,
            Self::Clean => ScriptCategory::Build,
            Self::Deploy => ScriptCategory::Deployment,
            Self::Fix => ScriptCategory::Development,
            Self::Format => ScriptCategory::Development,
            Self::Lint => ScriptCategory::Development,
            Self::Publish => ScriptCategory::Deployment,
            Self::Run => ScriptCategory::Run,
            Self::DevRun => ScriptCategory::Development,
            Self::Test => ScriptCategory::Development,
            Self::Other => ScriptCategory::Other,
        }
    }

    pub fn icon(&self) -> Option<&'static str> {
        match self {
            ScriptType::Build => Some("🔨"),
            ScriptType::Check => Some("✅"),
            ScriptType::Clean => Some("🧹"),
            ScriptType::Deploy => Some("🚀"),
            ScriptType::Fix => Some("✨"),
            ScriptType::Format => Some("🔧"),
            ScriptType::Lint => Some("🔍"),
            ScriptType::Publish => Some("📦"),
            ScriptType::Run => Some("▶️"),
            ScriptType::DevRun => Some("▶️"),
            ScriptType::Test => Some("🧪"),
            ScriptType::Other => self.category().icon(),
        }
    }
}

pub const SPECIAL_SCRIPTS: &[&str] = &[
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

/**
 * Finds a synonym script from a list of scripts based on the given name.
 *
 * This function first checks if the exact script name exists. If not, it looks
 * for synonymous commands (e.g., "dev" is synonymous with "start" and "run").
 *
 * @param scripts - A slice of Script objects to search through
 * @param name - The name of the script to find a synonym for
 * @returns The name of the found script (either exact match or synonym), or
 * None if not found
 */
pub fn find_synonym_script(scripts: &[Script], name: &str) -> Option<String> {
    // First check if the script exists directly
    if scripts.iter().any(|s| s.name == name) {
        return Some(name.to_string());
    }

    // Define groups of synonymous commands
    const SYNONYMS: &[&[&str]] = &[
        &["dev", "start", "run"],
        &["test", "check"],
        &["typecheck", "tc"],
        &["lint", "check"],
        &["format", "fmt"],
    ];

    // Find the synonym group that contains our script name
    SYNONYMS
        .iter()
        .find(|group| group.contains(&name))
        .and_then(|group| {
            // Look for the first script that exists from this group
            group
                .iter()
                .find(|&&synonym| scripts.iter().any(|s| s.name == synonym))
                .map(|&s| s.to_string())
        })
}

pub fn group_scripts<'a>(scripts: &'a [Script]) -> Vec<Vec<&'a Script>> {
    let mut prioritized_with_shortcuts = Vec::new();
    let mut prioritized_without_shortcuts = Vec::new();
    let mut with_shortcuts = Vec::new();
    let mut others = Vec::new();

    for script in scripts.iter() {
        match (script.category != ScriptCategory::Other, script.shortcut) {
            (true, Some(_)) => prioritized_with_shortcuts.push(script),
            (true, None) => prioritized_without_shortcuts.push(script),
            (false, Some(_)) => with_shortcuts.push(script),
            _ => others.push(script),
        }
    }

    vec![
        prioritized_with_shortcuts,
        prioritized_without_shortcuts,
        with_shortcuts,
        others,
    ]
    .into_iter()
    .filter(|group| !group.is_empty())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_script(name: &str) -> Script {
        Script {
            name: name.to_string(),
            command: "dummy".to_string(),
            description: None,
            script_type: ScriptType::Other,
            shortcut: None,
            category: ScriptCategory::Other,
        }
    }

    #[test]
    fn test_find_synonym_script() {
        let scripts = vec![
            make_script("dev"),
            make_script("test"),
            make_script("tc"),
            make_script("fmt"),
        ];

        // Direct matches should return the same name
        assert_eq!(
            find_synonym_script(&scripts, "dev"),
            Some("dev".to_string())
        );

        // Synonyms should find the first matching script
        assert_eq!(
            find_synonym_script(&scripts, "start"),
            Some("dev".to_string())
        );
        assert_eq!(
            find_synonym_script(&scripts, "run"),
            Some("dev".to_string())
        );

        // Typecheck should find tc
        assert_eq!(
            find_synonym_script(&scripts, "typecheck"),
            Some("tc".to_string())
        );

        // Format should find fmt
        assert_eq!(
            find_synonym_script(&scripts, "format"),
            Some("fmt".to_string())
        );

        // Non-existent scripts should return None
        assert_eq!(find_synonym_script(&scripts, "nonexistent"), None);

        // When no scripts from a synonym group exist, should return None
        assert_eq!(find_synonym_script(&scripts, "lint"), None);
    }

    #[test]
    fn test_find_synonym_script_empty() {
        let empty_scripts: Vec<Script> = vec![];
        assert_eq!(find_synonym_script(&empty_scripts, "dev"), None);
        assert_eq!(find_synonym_script(&empty_scripts, "start"), None);
    }

    #[test]
    fn test_find_synonym_script_order() {
        // Test that we get the first script from the SYNONYMS list that exists
        let scripts = vec![make_script("dev")];

        // When looking for any of these, should get "dev" because it's first in SYNONYMS
        assert_eq!(
            find_synonym_script(&scripts, "dev"),
            Some("dev".to_string())
        );
        assert_eq!(
            find_synonym_script(&scripts, "start"),
            Some("dev".to_string())
        );
        assert_eq!(
            find_synonym_script(&scripts, "run"),
            Some("dev".to_string())
        );
    }
}
