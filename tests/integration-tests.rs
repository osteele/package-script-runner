// tests/integration_tests.rs
mod helpers;

use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

use helpers::*; // Import all helper functions

#[test]
fn test_package_manager_detection() -> Result<()> {
    let temp_dir = setup_test_projects()?;

    // Test npm detection
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.current_dir(temp_dir.path().join("npm-project"))
        .arg("--list")
        .assert()
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("lint"));

    // Test cargo detection
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.current_dir(temp_dir.path().join("rust-project"))
        .arg("--list")
        .assert()
        .success()
        .stdout(predicate::str::contains("dev"))
        .stdout(predicate::str::contains("docs"));

    Ok(())
}

#[test]
fn test_npm_comprehensive() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project = create_npm_project(&temp_dir.path().join("npm-test"))?;

    // Test script listing
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.current_dir(&project.dir)
        .arg("--list")
        .assert()
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("lint"));

    // Test successful script execution
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.current_dir(&project.dir)
        .arg("test") // Using the 'test' script that runs 'true'
        .assert()
        .success();

    // Test failing script
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.current_dir(&project.dir)
        .arg("lint") // Using the 'lint' script that runs 'false'
        .assert()
        .failure()
        .code(1);

    Ok(())
}

#[test]
fn test_project_management() -> Result<()> {
    let temp_dir = setup_test_projects()?;
    let project_path = temp_dir.path().join("npm-project");

    // Test adding project
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.arg("projects")
        .arg("add")
        .arg("test-proj")
        .arg(&project_path)
        .assert()
        .success();

    // Test listing projects
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.arg("projects")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-proj"));

    // Test removing project
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.arg("projects")
        .arg("remove")
        .arg("test-proj")
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_error_handling() -> Result<()> {
    let temp_dir = setup_test_projects()?;

    // Create an empty project directory with just a README
    let invalid_dir = temp_dir.path().join("invalid-project");
    fs::create_dir(&invalid_dir)?;
    fs::write(
        invalid_dir.join("README.md"),
        "This is not a package manager project",
    )?;

    // Test invalid project reference
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.arg("-p")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Project 'nonexistent' not found"));

    // Test nonexistent script using 'run'
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.current_dir(temp_dir.path().join("npm-project"))
        .arg("run")
        .arg("nonexistent-script")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Cannot specify script name with special command 'run'",
        ));

    // Test nonexistent command
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.current_dir(temp_dir.path().join("npm-project"))
        .arg("nonexistent-script")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Unknown command 'nonexistent-script'",
        ))
        .stderr(predicate::str::contains(
            "Use 'run <script>' for custom scripts",
        ));

    // Test directory without package manager
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.current_dir(&invalid_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Could not detect package manager"));

    Ok(())
}

#[test]
fn test_cli_options() -> Result<()> {
    let temp_dir = setup_test_projects()?;
    let project_dir = temp_dir.path().join("npm-project");

    // Test --list flag
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.current_dir(&project_dir)
        .arg("--list")
        .assert()
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("test"));

    // Test --version flag
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));

    // Test --help flag - match the actual help text format
    let mut cmd = Command::cargo_bin("psr")?;
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("A fast TUI-based script runner"))
        .stdout(predicate::str::contains("\nUsage: psr"))
        .stdout(predicate::str::contains("\nOptions:"))
        .stdout(predicate::str::contains("\nCommands:"))
        .stdout(predicate::str::contains("\nArguments:"));

    Ok(())
}
