use anyhow::Result;
use toml::Value;

use std::{fs, path::Path, process::Command};

use super::PackageManager;
use crate::types::{Script, ScriptType};

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
