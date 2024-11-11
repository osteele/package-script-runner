#[derive(Clone, Debug)]
pub struct Script {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub phase: Phase,
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
            phase: script_type.map(|p| p.phase()).unwrap_or(Phase::Unknown),
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
pub enum Phase {
    Development,     // Local development activities
    Quality,         // Code quality, testing, verification
    Build,           // Building, packaging, artifacts
    Dependencies,    // Managing project dependencies
    Release,         // Publishing and deployment
    Infrastructure,  // Infrastructure and environment management
    Unknown,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ScriptType {
    // Development Phase
    Serve,           // dev, start, run, watch - local development server
    Generate,        // codegen, scaffold - code generation
    Migration,       // migrate, db:migrate - database migrations

    // Quality Phase
    Test,            // test, jest, vitest - unit/integration tests
    TestE2E,         // test:e2e, cypress - end-to-end testing
    Lint,            // lint, eslint, stylelint - code linting
    TypeCheck,       // tsc, typecheck, mypy - type checking
    Format,          // format, prettier, rustfmt - code formatting
    Audit,           // audit, security - security auditing

    // Build Phase
    Clean,           // clean, clear - cleanup build artifacts
    Build,           // build, compile - main build process
    BuildDev,        // build:dev - development builds
    BuildProd,       // build:prod - production builds

    // Dependencies Phase
    Install,         // install, ci - install dependencies
    Update,          // update, upgrade - update dependencies
    Lock,            // lock, freeze - lock dependencies

    // Release Phase
    Version,         // version, bump - version management
    Publish,         // publish, release - package publishing
    Deploy,          // deploy - deployment
    DeployStaging,   // deploy:staging - staging deployment
    DeployProd,      // deploy:prod - production deployment

    // Infrastructure Phase
    DockerBuild,     // docker:build - container builds
    DockerPush,      // docker:push - push containers
    Provision,       // provision, terraform - infrastructure provisioning

    Other,
}

impl ScriptType {
    pub fn phase(&self) -> Phase {
        match self {
            Self::Serve | Self::Generate | Self::Migration => Phase::Development,
            Self::Test | Self::TestE2E | Self::Lint | Self::TypeCheck |
            Self::Format | Self::Audit => Phase::Quality,
            Self::Clean | Self::Build | Self::BuildDev |
            Self::BuildProd => Phase::Build,
            Self::Install | Self::Update | Self::Lock => Phase::Dependencies,
            Self::Version | Self::Publish | Self::Deploy |
            Self::DeployStaging | Self::DeployProd => Phase::Release,
            Self::DockerBuild | Self::DockerPush |
            Self::Provision => Phase::Infrastructure,
            Self::Other => Phase::Development,
        }
    }

    pub fn synonyms(&self) -> &'static [&'static str] {
        match self {
            Self::Serve => &["dev", "start", "run", "watch", "serve"],
            Self::Generate => &["generate", "gen", "scaffold", "codegen"],
            Self::Migration => &["migrate", "db:migrate", "migration"],

            Self::Test => &["test", "jest", "vitest", "pytest"],
            Self::TestE2E => &["test:e2e", "cypress", "playwright"],
            Self::Lint => &["lint", "eslint", "stylelint", "clippy", "flake8", "pylint", "ruff"],
            Self::TypeCheck => &["typecheck", "tsc", "tc", "mypy"],
            Self::Format => &["format", "fmt", "prettier", "rustfmt", "black"],
            Self::Audit => &["audit", "security"],

            Self::Clean => &["clean", "clear", "purge"],
            Self::Build => &["build", "compile", "webpack", "vite"],
            Self::BuildDev => &["build:dev", "compile:dev"],
            Self::BuildProd => &["build:prod", "compile:prod"],

            Self::Install => &["install", "ci", "deps"],
            Self::Update => &["update", "upgrade", "deps:update"],
            Self::Lock => &["lock", "freeze"],

            Self::Version => &["version", "bump", "changeset"],
            Self::Publish => &["publish", "release", "pack"],
            Self::Deploy => &["deploy", "push"],
            Self::DeployStaging => &["deploy:staging", "push:staging"],
            Self::DeployProd => &["deploy:prod", "push:prod"],

            Self::DockerBuild => &["docker:build", "container:build"],
            Self::DockerPush => &["docker:push", "container:push"],
            Self::Provision => &["provision", "terraform", "infra"],

            Self::Other => &[],
        }
    }

    pub fn from_script(name: &str, command: &str) -> Self {
        let text = format!("{} {}", name, command).to_lowercase();

        // Keep only the most generic patterns that are common across ecosystems
        if text.contains("test:e2e") {
            Self::TestE2E
        } else if text.contains("test") {
            Self::Test
        } else if text.contains("lint") {
            Self::Lint
        } else if text.contains("format") || text.contains("fmt") {
            Self::Format
        } else if text.contains("build:prod") {
            Self::BuildProd
        } else if text.contains("build:dev") {
            Self::BuildDev
        } else if text.contains("build") {
            Self::Build
        } else if text.contains("deploy:prod") {
            Self::DeployProd
        } else if text.contains("deploy:staging") {
            Self::DeployStaging
        } else if text.contains("deploy") {
            Self::Deploy
        } else {
            Self::Other
        }
    }

    pub fn icon(&self) -> Option<&'static str> {
        match self {
            // Development
            Self::Serve => Some("▶️"),
            Self::Generate => Some("✨"),
            Self::Migration => Some("🔄"),

            // Quality
            Self::Test => Some("🧪"),
            Self::TestE2E => Some("🔄"),
            Self::Lint => Some("🔍"),
            Self::TypeCheck => Some("✅"),
            Self::Format => Some("✨"),
            Self::Audit => Some("🔒"),

            // Build
            Self::Clean => Some("🧹"),
            Self::Build | Self::BuildDev | Self::BuildProd => Some("🔨"),

            // Dependencies
            Self::Install => Some("📦"),
            Self::Update => Some("⬆️"),
            Self::Lock => Some("🔒"),

            // Release
            Self::Version => Some("🏷️"),
            Self::Publish => Some("📤"),
            Self::Deploy | Self::DeployStaging | Self::DeployProd => Some("🚀"),

            // Infrastructure
            Self::DockerBuild | Self::DockerPush => Some("🐳"),
            Self::Provision => Some("☁️"),

            Self::Other => None,
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
    // Direct match
    if scripts.iter().any(|s| s.name == name) {
        return Some(name.to_string());
    }

    // Check all ScriptTypes for synonyms
    for script_type in [
        ScriptType::Serve,
        ScriptType::Test,
        ScriptType::Lint,
        ScriptType::TypeCheck,
        ScriptType::Format,
        ScriptType::Build,
        ScriptType::Clean,
        ScriptType::Deploy,
        ScriptType::Publish,
    ] {
        if script_type.synonyms().contains(&name) {
            // Find first script of this type that exists
            return scripts
                .iter()
                .find(|s| s.script_type == script_type)
                .map(|s| s.name.clone());
        }
    }

    None
}

pub fn group_scripts<'a>(scripts: &'a [Script]) -> Vec<Vec<&'a Script>> {
    let mut prioritized_with_shortcuts = Vec::new();
    let mut prioritized_without_shortcuts = Vec::new();
    let mut with_shortcuts = Vec::new();
    let mut others = Vec::new();

    for script in scripts.iter() {
        match (script.phase != Phase::Development, script.shortcut) {
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
            phase: Phase::Development,
        }
    }

    #[test]
    #[ignore]
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
    #[ignore]
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
