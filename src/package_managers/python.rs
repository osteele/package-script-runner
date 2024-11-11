use anyhow::Result;
use toml::Value;

use std::{fs, path::Path, process::Command};

use super::PackageManager;
use crate::types::{Script, ScriptType};

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

    fn find_scripts(&self, path: &Path) -> Result<Vec<Script>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::project_dir_mocks::*;

    #[test]
    fn test_parse_pip_scripts() {
        let uv = PythonPackageManager::Uv;
        let temp_dir = create_pip_project(&std::env::temp_dir().join("pip-project")).unwrap();
        let scripts = uv.parse_pip_scripts(&temp_dir.dir).unwrap();
        println!("{:?}", scripts);

        assert!(scripts.iter().any(|s| s.name == "lint" && s.script_type == ScriptType::Lint));
    }

    #[test]
    #[ignore]
    fn test_parse_poetry_scripts() {
        let poetry = PythonPackageManager::Poetry;
        let temp_dir = create_poetry_project(&std::env::temp_dir().join("poetry-project")).unwrap();
        let scripts = poetry.parse_poetry_scripts(&temp_dir.dir).unwrap();

        assert!(scripts.iter().any(|s| s.name == "lint" && s.script_type == ScriptType::Lint));
    }
}
