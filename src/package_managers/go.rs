use anyhow::Result;

use std::{fs, path::Path, process::Command};

use super::PackageManager;
use crate::types::{Script, ScriptType};

pub struct GoPackageManager;

impl PackageManager for GoPackageManager {
    fn detect(dir: &Path) -> Option<Self> {
        if dir.join("go.mod").exists() {
            Some(GoPackageManager)
        } else {
            None
        }
    }

    fn run_command(&self, script: &str) -> Command {
        let mut cmd = Command::new("go");
        cmd.arg(script);
        cmd
    }

    fn find_scripts(&self, path: &Path) -> Result<Vec<Script>> {
        let mut scripts = Vec::new();

        // Add standard Go commands
        scripts.extend(vec![
            Script::new(
                "build",
                "go build",
                Some("Compile the package".to_string()),
                Some(ScriptType::Build),
                Some('b'),
            ),
            Script::new(
                "run",
                "go run .",
                Some("Run the main package".to_string()),
                Some(ScriptType::Serve),
                Some('r'),
            ),
            Script::new(
                "test",
                "go test ./...",
                Some("Run package tests".to_string()),
                Some(ScriptType::Test),
                Some('t'),
            ),
            Script::new(
                "lint",
                "golangci-lint run",
                Some("Run linters".to_string()),
                Some(ScriptType::Lint),
                Some('l'),
            ),
            Script::new(
                "fmt",
                "go fmt ./...",
                Some("Format code".to_string()),
                Some(ScriptType::Format),
                Some('f'),
            ),
            Script::new(
                "mod tidy",
                "go mod tidy",
                Some("Clean up dependencies".to_string()),
                Some(ScriptType::Update),
                None,
            ),
            Script::new(
                "get",
                "go get",
                Some("Download and install packages and dependencies".to_string()),
                Some(ScriptType::Serve),
                None,
            ),
        ]);

        // Try to parse Makefile targets if present
        if path.join("Makefile").exists() {
            if let Ok(content) = fs::read_to_string(path.join("Makefile")) {
                for line in content.lines() {
                    if let Some(target) = line.trim().strip_suffix(':') {
                        if !target.starts_with('.') && !target.contains(' ') {
                            scripts.push(Script::new(
                                &format!("make:{}", target),
                                &format!("make {}", target),
                                Some(format!("Run make target: {}", target)),
                                Some(ScriptType::Serve),
                                None,
                            ));
                        }
                    }
                }
            }
        }

        Ok(scripts)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::project_dir_mocks::*;
    use super::*;

    #[test]
    fn test_find_scripts() {
        let go = GoPackageManager;
        let temp_dir = create_go_project(&std::env::temp_dir().join("go-project")).unwrap();
        let scripts = go.find_scripts(&temp_dir.dir).unwrap();

        assert!(scripts.iter().any(|s| s.name == "run" && s.script_type == ScriptType::Serve));
        assert!(scripts.iter().any(|s| s.name == "test" && s.script_type == ScriptType::Test));
        assert!(scripts.iter().any(|s| s.name == "lint" && s.script_type == ScriptType::Lint));
        assert!(scripts.iter().any(|s| s.name == "fmt" && s.script_type == ScriptType::Format));
    }
}
