#[derive(Clone)]
pub struct Script {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub shortcut: Option<char>,
    pub script_type: ScriptType,
}

#[allow(dead_code)]
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
    SYNONYMS.iter()
        .find(|group| group.contains(&name))
        .and_then(|group| {
            // Look for the first script that exists from this group
            group.iter()
                .find(|&&synonym| scripts.iter().any(|s| s.name == synonym))
                .map(|&s| s.to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_script(name: &str) -> Script {
        Script {
            name: name.to_string(),
            command: "dummy".to_string(),
            description: None,
            shortcut: None,
            script_type: ScriptType::Other,
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
        let scripts = vec![
            make_script("dev"),
        ];

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
