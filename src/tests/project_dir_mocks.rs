use anyhow::Result;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

pub struct TestProject {
    pub dir: PathBuf,
}

impl TestProject {
    pub fn create_file(&self, path: &str, content: &str) -> Result<()> {
        let full_path = self.dir.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full_path, content)?;
        Ok(())
    }
}

pub fn create_npm_project(dir: &PathBuf) -> Result<TestProject> {
    let project = TestProject { dir: dir.clone() };

    project.create_file(
        "package.json",
        &json!({
            "name": "npm-test",
            "scripts": {
                "start": "true",
                "test": "true",
                "build": "true",
                "lint": "false",
                "error": "nonexistent-command"
            }
        })
        .to_string(),
    )?;

    project.create_file("package-lock.json", "{}")?;

    Ok(project)
}

pub fn create_yarn_project(dir: &PathBuf) -> Result<TestProject> {
    let project = TestProject { dir: dir.clone() };

    project.create_file(
        "package.json",
        &json!({
            "name": "yarn-test",
            "scripts": {
                "start": "node index.js",
                "test": "jest"
            }
        })
        .to_string(),
    )?;

    project.create_file("yarn.lock", "")?;

    Ok(project)
}

pub fn create_pnpm_project(dir: &PathBuf) -> Result<TestProject> {
    let project = TestProject { dir: dir.clone() };

    project.create_file(
        "package.json",
        &json!({
            "name": "pnpm-test",
            "scripts": {
                "dev": "vite",
                "build": "vite build"
            }
        })
        .to_string(),
    )?;

    project.create_file("pnpm-lock.yaml", "")?;

    Ok(project)
}

pub fn create_cargo_project(dir: &PathBuf) -> Result<TestProject> {
    let project = TestProject { dir: dir.clone() };

    project.create_file(
        "Cargo.toml",
        r#"
[package]
name = "rust-test"
version = "0.1.0"

[package.metadata.scripts]
dev = "cargo watch -x run"
docs = "cargo doc --open"
"#,
    )?;

    Ok(project)
}

pub fn create_poetry_project(dir: &PathBuf) -> Result<TestProject> {
    let project = TestProject { dir: dir.clone() };

    project.create_file(
        "pyproject.toml",
        r#"
[tool.poetry]
name = "poetry-test"
version = "0.1.0"

[tool.poetry.dependencies]
python = "^3.9"
requests = "^2.31.0"

[tool.poetry.dev-dependencies]
pytest = "^7.4.0"
ruff = "^0.1.0"
"#,
    )?;

    project.create_file("poetry.lock", "")?;

    Ok(project)
}

pub fn create_pip_project(dir: &PathBuf) -> Result<TestProject> {
    let project = TestProject { dir: dir.clone() };

    project.create_file(
        "requirements.txt",
        r#"
pytest==7.4.0
ruff==0.1.0
requests==2.31.0
"#,
    )?;

    Ok(project)
}

pub fn create_go_project(dir: &PathBuf) -> Result<TestProject> {
    let project = TestProject { dir: dir.clone() };

    project.create_file(
        "go.mod",
        r#"
module example.com/test
go 1.21
"#,
    )?;

    project.create_file(
        "Makefile",
        r#"
.PHONY: all build test

all: build

build:
    go build -o bin/app

test:
    go test ./...

run:
    go run .

dev:
    go run . -dev
"#,
    )?;

    Ok(project)
}

pub fn setup_test_projects() -> Result<tempfile::TempDir> {
    let temp_dir = tempfile::TempDir::new()?;

    // Create test projects for each package manager
    create_npm_project(&temp_dir.path().join("npm-project"))?;
    create_yarn_project(&temp_dir.path().join("yarn-project"))?;
    create_pnpm_project(&temp_dir.path().join("pnpm-project"))?;
    create_cargo_project(&temp_dir.path().join("rust-project"))?;
    create_poetry_project(&temp_dir.path().join("poetry-project"))?;
    create_pip_project(&temp_dir.path().join("pip-project"))?;
    create_go_project(&temp_dir.path().join("go-project"))?;

    Ok(temp_dir)
}
