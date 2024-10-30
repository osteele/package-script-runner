use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path, process::Command};
use toml::Value;

use crate::script_type::{Script, ScriptCategory, ScriptType};
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
            Script::new(
                "build",
                "cargo build",
                Some("Compile the current package".to_string()),
                Some(ScriptType::Build),
                Some('b'),
            ),
            Script::new(
                "run",
                "cargo run",
                Some("Run the main binary of the current package".to_string()),
                Some(ScriptType::DevRun),
                Some('r'),
            ),
            Script::new(
                "test",
                "cargo test",
                Some("Run the tests".to_string()),
                Some(ScriptType::Test),
                Some('t'),
            ),
            Script::new(
                "check",
                "cargo check",
                Some(
                    "Analyze the current package and report errors, but don't build object files"
                        .to_string(),
                ),
                Some(ScriptType::Lint),
                Some('c'),
            ),
            Script::new(
                "lint",
                "cargo clippy",
                Some("Run the Rust linter (clippy)".to_string()),
                Some(ScriptType::Lint),
                Some('l'),
            ),
            Script::new(
                "fix",
                "cargo clippy --fix",
                Some("Automatically fix linting issues".to_string()),
                Some(ScriptType::Fix),
                None,
            ),
            Script::new(
                "install",
                "cargo install --path .",
                Some("Install the current package".to_string()),
                Some(ScriptType::Deploy),
                None,
            ),
            Script::new(
                "publish",
                "cargo publish",
                Some("Publish the current package".to_string()),
                Some(ScriptType::Publish),
                None,
            ),
        ]);

        // Parse custom scripts from [package.metadata.scripts]
        if let Some(package) = cargo_toml.get("package") {
            if let Some(metadata) = package.get("metadata") {
                if let Some(custom_scripts) = metadata.get("scripts") {
                    if let Some(script_table) = custom_scripts.as_table() {
                        for (name, value) in script_table {
                            if let Some(command) = value.as_str() {
                                scripts.push(Script::new(
                                    &name,
                                    &command,
                                    None,
                                    Some(ScriptType::Run),
                                    None,
                                ));
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
                        scripts.push(Script::new(
                            &format!("run:{}", name),
                            &format!("cargo run --bin {}", name),
                            Some(format!("Run the {} binary", name)),
                            None,
                            None,
                        ));
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

        // Check for linting tools in requirements.txt
        let has_ruff = content.lines().any(|l| l.starts_with("ruff"));
        let has_flake8 = content.lines().any(|l| l.starts_with("flake8"));
        let has_pylint = content.lines().any(|l| l.starts_with("pylint"));

        if has_ruff {
            scripts.push(Script::new(
                "lint",
                "ruff check .",
                Some("Run Ruff linter".to_string()),
                Some(ScriptType::Lint),
                Some('l'),
            ));
        } else if has_flake8 {
            scripts.push(Script::new(
                "lint",
                "flake8",
                Some("Run Flake8 linter".to_string()),
                Some(ScriptType::Lint),
                Some('l'),
            ));
        } else if has_pylint {
            scripts.push(Script::new(
                "lint",
                "pylint **/*.py",
                Some("Run Pylint linter".to_string()),
                Some(ScriptType::Lint),
                Some('l'),
            ));
        }

        for line in content.lines() {
            if let Some(package) = line.split_whitespace().next() {
                scripts.push(Script::new(
                    &package.to_string(),
                    &format!("pip install {}", package),
                    None,
                    None,
                    None,
                ));
            }
        }

        Ok(scripts)
    }

    fn parse_poetry_scripts(&self, path: &Path) -> Result<Vec<Script>> {
        let pyproject_path = path.join("pyproject.toml");
        let content = fs::read_to_string(pyproject_path)?;
        let pyproject: toml::Value = toml::from_str(&content)?;

        let mut scripts = Vec::new();

        // Add common Python linting commands if the tools are in dependencies
        if let Some(tool) = pyproject.get("tool") {
            if let Some(poetry) = tool.get("poetry") {
                if let Some(deps) = poetry
                    .get("dependencies")
                    .or_else(|| poetry.get("dev-dependencies"))
                {
                    let has_ruff = deps.as_table().map_or(false, |t| t.contains_key("ruff"));
                    let has_flake8 = deps.as_table().map_or(false, |t| t.contains_key("flake8"));
                    let has_pylint = deps.as_table().map_or(false, |t| t.contains_key("pylint"));

                    if has_ruff {
                        scripts.push(Script::new(
                            "lint",
                            "poetry run ruff check .",
                            Some("Run Ruff linter".to_string()),
                            Some(ScriptType::Lint),
                            Some('l'),
                        ));
                    } else if has_flake8 {
                        scripts.push(Script::new(
                            "lint",
                            "poetry run flake8",
                            Some("Run Flake8 linter".to_string()),
                            Some(ScriptType::Lint),
                            Some('l'),
                        ));
                    } else if has_pylint {
                        scripts.push(Script::new(
                            "lint",
                            "poetry run pylint **/*.py",
                            Some("Run Pylint linter".to_string()),
                            Some(ScriptType::Lint),
                            Some('l'),
                        ));
                    }
                }
            }
        }

        if let Some(tool) = pyproject.get("tool") {
            if let Some(poetry) = tool.get("poetry") {
                if let Some(dependencies) = poetry.get("dependencies") {
                    for (name, value) in dependencies.as_table().unwrap() {
                        let command = if value.is_str() {
                            format!("poetry add {}", name)
                        } else {
                            format!("poetry add {}@{}", name, value.as_str().unwrap_or("latest"))
                        };
                        scripts.push(Script::new(&name.to_string(), &command, None, None, None));
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
                        scripts.push(Script::new(
                            &format!("dev:{}", name),
                            &command,
                            None,
                            None,
                            None,
                        ));
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
            // Create a new empty map that lives long enough
            let empty_map = toml::map::Map::new();
            // Use a reference to either the dependencies table or the empty map
            let deps = dependencies.as_table().unwrap_or(&empty_map);

            let has_ruff = deps.contains_key("ruff");
            let has_flake8 = deps.contains_key("flake8");
            let has_pylint = deps.contains_key("pylint");

            if has_ruff {
                scripts.push(Script::new(
                    "lint",
                    "uv run ruff check .",
                    Some("Run Ruff linter".to_string()),
                    Some(ScriptType::Lint),
                    Some('l'),
                ));
            } else if has_flake8 {
                scripts.push(Script::new(
                    "lint",
                    "uv run flake8",
                    Some("Run Flake8 linter".to_string()),
                    Some(ScriptType::Lint),
                    Some('l'),
                ));
            } else if has_pylint {
                scripts.push(Script::new(
                    "lint",
                    "uv run pylint **/*.py",
                    Some("Run Pylint linter".to_string()),
                    Some(ScriptType::Lint),
                    Some('l'),
                ));
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
