use anyhow::Result;
use serde::Deserialize;

use std::{collections::HashMap, fs, path::Path, process::Command};

use super::PackageManager;
use crate::script_type::{Script, ScriptCategory};

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

    fn parse_scripts(&self, path: &Path) -> Result<Vec<Script>> {
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
                        None,
                        None,
                    )
                })
                .collect();

            // Sort scripts: non-Other types first (alphabetically), then Other types (alphabetically)
            all_scripts.sort_by(|a, b| match (a.category, b.category) {
                (ScriptCategory::Other, ScriptCategory::Other) => a.name.cmp(&b.name),
                (ScriptCategory::Other, _) => std::cmp::Ordering::Greater,
                (_, ScriptCategory::Other) => std::cmp::Ordering::Less,
                _ => a.name.cmp(&b.name),
            });

            scripts.extend(all_scripts);
        }
        Ok(scripts)
    }
}
