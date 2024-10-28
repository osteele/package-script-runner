use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path, process::Command};
use toml::Value;

use crate::script_type::{Script, ScriptType, PRIORITY_SCRIPTS};
use anyhow::Result;

pub trait PackageManager {
    fn detect(dir: &Path) -> Option<Self>
    where
        Self: Sized;
    fn run_command(&self, script: &str) -> Command;
    fn parse_scripts(&self, path: &Path) -> Result<Vec<Script>>;
}

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
            // Process priority scripts first
            for &priority in PRIORITY_SCRIPTS {
                if let Some(command) = script_map.get(priority) {
                    let script_type = ScriptType::from_script(priority, command);
                    scripts.push(Script {
                        name: priority.to_string(),
                        command: command.clone(),
                        description: package.descriptions.get(priority).cloned(),
                        shortcut: Some(priority.chars().next().unwrap()),
                        script_type,
                    });
                }
            }

            // Process remaining scripts alphabetically
            let mut other_scripts: Vec<_> = script_map
                .iter()
                .filter(|(name, _)| !PRIORITY_SCRIPTS.contains(&name.as_str()))
                .collect();
            other_scripts.sort_by(|(a, _), (b, _)| a.cmp(b));

            for (name, command) in other_scripts {
                let script_type = ScriptType::from_script(name, command);
                scripts.push(Script {
                    name: name.clone(),
                    command: command.clone(),
                    description: package.descriptions.get(name).cloned(),
                    shortcut: None,
                    script_type,
                });
            }
        }
        Ok(scripts)
    }
}

pub struct RustPackageManager;

impl PackageManager for RustPackageManager {
    fn detect(dir: &Path) -> Option<Self> {
        if dir.join("Cargo.toml").exists() {
            Some(RustPackageManager)
        } else {
            None
        }
    }

    fn run_command(&self, script: &str) -> Command {
        let mut cmd = Command::new("cargo");
        cmd.arg(script);
        cmd
    }

    fn parse_scripts(&self, path: &Path) -> Result<Vec<Script>> {
        let cargo_toml_path = path.join("Cargo.toml");
        let content = fs::read_to_string(cargo_toml_path)?;
        let cargo_toml: Value = toml::from_str(&content)?;

        let mut scripts = Vec::new();

        // Add default Cargo commands
        scripts.extend(vec![
            Script {
                name: "build".to_string(),
                command: "cargo build".to_string(),
                description: Some("Compile the current package".to_string()),
                shortcut: Some('b'),
                script_type: ScriptType::Build,
            },
            Script {
                name: "run".to_string(),
                command: "cargo run".to_string(),
                description: Some("Run the main binary of the current package".to_string()),
                shortcut: Some('r'),
                script_type: ScriptType::Development,
            },
            Script {
                name: "test".to_string(),
                command: "cargo test".to_string(),
                description: Some("Run the tests".to_string()),
                shortcut: Some('t'),
                script_type: ScriptType::Test,
            },
            Script {
                name: "check".to_string(),
                command: "cargo check".to_string(),
                description: Some(
                    "Analyze the current package and report errors, but don't build object files"
                        .to_string(),
                ),
                shortcut: Some('c'),
                script_type: ScriptType::Other,
            },
        ]);

        // Parse custom scripts from [package.metadata.scripts]
        if let Some(package) = cargo_toml.get("package") {
            if let Some(metadata) = package.get("metadata") {
                if let Some(custom_scripts) = metadata.get("scripts") {
                    if let Some(script_table) = custom_scripts.as_table() {
                        for (name, value) in script_table {
                            if let Some(command) = value.as_str() {
                                scripts.push(Script {
                                    name: name.clone(),
                                    command: command.to_string(),
                                    description: None, // You could add descriptions in Cargo.toml if desired
                                    shortcut: None,
                                    script_type: ScriptType::Other,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Parse binary targets
        if let Some(bin) = cargo_toml.get("bin") {
            if let Some(binaries) = bin.as_array() {
                for binary in binaries {
                    if let Some(name) = binary.get("name").and_then(|n| n.as_str()) {
                        scripts.push(Script {
                            name: format!("run:{}", name),
                            command: format!("cargo run --bin {}", name),
                            description: Some(format!("Run the {} binary", name)),
                            shortcut: None,
                            script_type: ScriptType::Development,
                        });
                    }
                }
            }
        }

        Ok(scripts)
    }
}

pub enum PythonPackageManager {
    Pip,
    Poetry,
    Uv,
}

impl PackageManager for PythonPackageManager {
    fn detect(dir: &Path) -> Option<Self> {
        if dir.join("pyproject.toml").exists() {
            if let Ok(content) = std::fs::read_to_string(dir.join("pyproject.toml")) {
                if let Ok(pyproject) = content.parse::<Value>() {
                    if pyproject
                        .get("tool")
                        .and_then(|t| t.get("poetry"))
                        .is_some()
                    {
                        return Some(Self::Poetry);
                    } else if pyproject.get("tool").and_then(|t| t.get("uv")).is_some()
                        || pyproject
                            .get("build-system")
                            .and_then(|bs| bs.as_table())
                            .and_then(|bs| bs.get("requires"))
                            .and_then(|r| r.as_array())
                            .map(|r| {
                                r.iter()
                                    .any(|v| v.as_str().map(|s| s.contains("uv")).unwrap_or(false))
                            })
                            .unwrap_or(false)
                    {
                        return Some(Self::Uv);
                    }
                }
            }
        }
        if dir.join("poetry.lock").exists() {
            Some(Self::Poetry)
        } else if dir.join(".uv").exists() || dir.join("uv.toml").exists() {
            Some(Self::Uv)
        } else if dir.join("requirements.txt").exists() {
            Some(Self::Pip)
        } else {
            None
        }
    }

    fn run_command(&self, script: &str) -> Command {
        match self {
            Self::Pip => {
                let mut cmd = Command::new("pip");
                cmd.arg("run");
                cmd.arg(script);
                cmd
            }
            Self::Poetry => {
                let mut cmd = Command::new("poetry");
                cmd.arg("run");
                cmd.arg(script);
                cmd
            }
            Self::Uv => {
                let mut cmd = Command::new("uv");
                cmd.arg("run");
                cmd.arg(script);
                cmd
            }
        }
    }

    fn parse_scripts(&self, path: &Path) -> Result<Vec<Script>> {
        match self {
            Self::Pip => self.parse_pip_scripts(path),
            Self::Poetry => self.parse_poetry_scripts(path),
            Self::Uv => self.parse_uv_scripts(path),
        }
    }
}

impl PythonPackageManager {
    fn parse_pip_scripts(&self, path: &Path) -> Result<Vec<Script>> {
        let requirements_path = path.join("requirements.txt");
        let content = fs::read_to_string(requirements_path)?;

        let mut scripts = Vec::new();
        for line in content.lines() {
            if let Some(package) = line.split_whitespace().next() {
                scripts.push(Script {
                    name: package.to_string(),
                    command: format!("pip install {}", package),
                    description: None,
                    shortcut: None,
                    script_type: ScriptType::Other,
                });
            }
        }

        Ok(scripts)
    }

    fn parse_poetry_scripts(&self, path: &Path) -> Result<Vec<Script>> {
        let pyproject_path = path.join("pyproject.toml");
        let content = fs::read_to_string(pyproject_path)?;
        let pyproject: toml::Value = toml::from_str(&content)?;

        let mut scripts = Vec::new();

        if let Some(tool) = pyproject.get("tool") {
            if let Some(poetry) = tool.get("poetry") {
                if let Some(dependencies) = poetry.get("dependencies") {
                    for (name, value) in dependencies.as_table().unwrap() {
                        let command = if value.is_str() {
                            format!("poetry add {}", name)
                        } else {
                            format!("poetry add {}@{}", name, value.as_str().unwrap_or("latest"))
                        };
                        scripts.push(Script {
                            name: name.to_string(),
                            command,
                            description: None,
                            shortcut: None,
                            script_type: ScriptType::Other,
                        });
                    }
                }
                if let Some(dev_dependencies) = poetry.get("dev-dependencies") {
                    for (name, value) in dev_dependencies.as_table().unwrap() {
                        let command = if value.is_str() {
                            format!("poetry add --dev {}", name)
                        } else {
                            format!(
                                "poetry add --dev {}@{}",
                                name,
                                value.as_str().unwrap_or("latest")
                            )
                        };
                        scripts.push(Script {
                            name: format!("dev:{}", name),
                            command,
                            description: None,
                            shortcut: None,
                            script_type: ScriptType::Development,
                        });
                    }
                }
            }
        }

        Ok(scripts)
    }

    fn parse_uv_scripts(&self, path: &Path) -> Result<Vec<Script>> {
        let uv_toml_path = path.join("uv.toml");
        let content = fs::read_to_string(uv_toml_path)?;
        let uv_config: toml::Value = toml::from_str(&content)?;

        let mut scripts = Vec::new();

        if let Some(dependencies) = uv_config.get("dependencies") {
            for (name, value) in dependencies.as_table().unwrap() {
                let command = if value.is_str() {
                    format!("uv pip install {}", name)
                } else {
                    format!(
                        "uv pip install {}=={}",
                        name,
                        value.as_str().unwrap_or("latest")
                    )
                };
                scripts.push(Script {
                    name: name.to_string(),
                    command,
                    description: None,
                    shortcut: None,
                    script_type: ScriptType::Other,
                });
            }
        }

        Ok(scripts)
    }
}

pub fn detect_package_manager_in_dir(dir: &Path) -> Option<Box<dyn PackageManager>> {
    if let Some(npm) = NodePackageManager::detect(dir) {
        Some(Box::new(npm))
    } else if let Some(rust) = RustPackageManager::detect(dir) {
        Some(Box::new(rust))
    } else if let Some(python) = PythonPackageManager::detect(dir) {
        Some(Box::new(python))
    } else {
        None
    }
}
