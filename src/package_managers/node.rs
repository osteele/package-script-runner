use anyhow::Result;
use serde::Deserialize;

use std::{collections::HashMap, fs, path::Path, process::Command};

use super::PackageManager;
use crate::types::{Phase, Script, ScriptType};

pub enum NodePackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Deno,
}

#[derive(Deserialize)]
struct PackageJson {
    scripts: Option<HashMap<String, String>>,
    #[serde(default)]
    descriptions: HashMap<String, String>, // Optional script descriptions
}

impl NodePackageManager {
    fn detect_script_type(&self, name: &str, command: &str) -> ScriptType {
        let text = format!("{} {}", name, command).to_lowercase();

        const PATTERNS: &[(&[&str], ScriptType)] = &[
            // Development
            (&["dev", "start", "serve", "watch", "run"], ScriptType::Serve),
            (&["generate", "gen:", "scaffold"], ScriptType::Generate),
            (&["migrate", "db:migrate", "migration"], ScriptType::Migration),

            // Quality
            (&["test:e2e", "cypress", "playwright", "e2e"], ScriptType::TestE2E),
            (&["test", "jest", "vitest", "mocha"], ScriptType::Test),
            (&["tsc", "typecheck", "type-check", "types"], ScriptType::TypeCheck),
            (&["lint", "eslint", "tslint", "xo"], ScriptType::Lint),
            (&["format", "fmt", "prettier", "beautify"], ScriptType::Format),
            (&["audit", "security"], ScriptType::Audit),

            // Build
            (&["build:prod", "build --prod", "build:production"], ScriptType::BuildProd),
            (&["build:dev", "build:development"], ScriptType::BuildDev),
            (&["build", "compile", "webpack", "vite build"], ScriptType::Build),
            (&["clean", "cleanup", "clear"], ScriptType::Clean),

            // Dependencies
            (&["install", "ci", "deps"], ScriptType::Install),
            (&["update", "upgrade", "deps:update"], ScriptType::Update),
            (&["lock", "shrinkwrap", "freeze"], ScriptType::Lock),

            // Release
            (&["version", "bump", "changeset"], ScriptType::Version),
            (&["publish", "release", "pack"], ScriptType::Publish),
            (&["deploy:prod", "deploy:production"], ScriptType::DeployProd),
            (&["deploy:staging", "deploy:stage"], ScriptType::DeployStaging),
            (&["deploy", "push"], ScriptType::Deploy),

            // Infrastructure
            (&["docker:push", "container:push"], ScriptType::DockerPush),
            (&["docker", "container"], ScriptType::DockerBuild),
            (&["terraform", "provision", "infra"], ScriptType::Provision),
        ];

        PATTERNS
            .iter()
            .find(|(patterns, _)| {
                patterns.iter().any(|&pattern| {
                    text.contains(pattern) || name == pattern || command.contains(pattern)
                })
            })
            .map(|(_, script_type)| *script_type)
            .unwrap_or(ScriptType::Other)
    }
}

impl PackageManager for NodePackageManager {
    fn detect(dir: &Path) -> Option<Self> {
        if !dir.join("package.json").exists() {
            return None;
        }
        // Check lock files first
        if dir.join("bun.lockb").exists() {
            return Some(Self::Bun);
        } else if dir.join("pnpm-lock.yaml").exists() {
            return Some(Self::Pnpm);
        } else if dir.join("yarn.lock").exists() {
            return Some(Self::Yarn);
        } else if dir.join("package-lock.json").exists() {
            return Some(Self::Npm);
        } else if dir.join("deno.lock").exists() {
            return Some(Self::Deno);
        }

        // Check config files as fallback
        if dir.join(".npmrc").exists() {
            return Some(Self::Npm);
        } else if dir.join(".yarnrc").exists() || dir.join(".yarnrc.yml").exists() {
            return Some(Self::Yarn);
        } else if dir.join(".npmrc").exists()
            && std::fs::read_to_string(dir.join(".npmrc"))
                .map_or(false, |content| content.contains("pnpm"))
        {
            return Some(Self::Pnpm);
        }

        None
    }

    fn run_command(&self, script: &str) -> Command {
        let mut cmd = match self {
            Self::Npm => {
                let mut c = Command::new("npm");
                c.arg("run");
                c
            }
            Self::Yarn => {
                let mut c = Command::new("yarn");
                c.arg("run");
                c
            }
            Self::Pnpm => {
                let mut c = Command::new("pnpm");
                c.arg("run");
                c
            }
            Self::Bun => {
                let mut c = Command::new("bun");
                c.arg("run");
                c
            }
            Self::Deno => {
                let mut c = Command::new("deno");
                c.arg("task");
                c
            }
        };
        cmd.arg(script);
        cmd
    }

    fn find_scripts(&self, path: &Path) -> Result<Vec<Script>> {
        let package_json_path = path.join("package.json");
        if !package_json_path.exists() {
            return Err(anyhow::anyhow!("package.json not found"));
        }
        let content = fs::read_to_string(package_json_path)?;
        let package: PackageJson = serde_json::from_str(&content)?;

        let mut scripts = Vec::new();
        if let Some(script_map) = package.scripts {
            // First collect all scripts
            let mut all_scripts: Vec<_> = script_map
                .into_iter()
                .map(|(name, command)| {
                    Script::new(
                        &name,
                        &command,
                        package.descriptions.get(&name).cloned(),
                        Some(self.detect_script_type(&name, &command)),
                        None,
                    )
                })
                .collect();

            // Sort scripts: non-Other types first (alphabetically), then Other types (alphabetically)
            all_scripts.sort_by(|a, b| match (a.phase, b.phase) {
                (Phase::Unknown, Phase::Unknown) => a.name.cmp(&b.name),
                (Phase::Unknown, _) => std::cmp::Ordering::Greater,
                (_, Phase::Unknown) => std::cmp::Ordering::Less,
                _ => a.name.cmp(&b.name),
            });

            scripts.extend(all_scripts);
        }
        Ok(scripts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_script_type() {
        let npm = NodePackageManager::Npm;

        assert_eq!(npm.detect_script_type("start", "node index.js"), ScriptType::Serve);
        assert_eq!(npm.detect_script_type("dev", "vite"), ScriptType::Serve);
        assert_eq!(npm.detect_script_type("test", "jest"), ScriptType::Test);
        assert_eq!(npm.detect_script_type("format", "prettier --write ."), ScriptType::Format);
        assert_eq!(npm.detect_script_type("typecheck", "tsc"), ScriptType::TypeCheck);
    }
}
