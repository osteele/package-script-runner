PSR (Package Script Runner) - Developer Notes
=============================================

Tech Stack Overview
------------------
PSR is built using the following technologies:

- Rust programming language
- Ratatui library for terminal UI
- Crossterm library for terminal manipulation
- Clap library for command-line argument parsing
- Serde for JSON serialization/deserialization
- Anyhow for error handling
- Dirs for cross-platform directory handling
- Walkdir for directory traversal
- Which for executable path resolution
- Toml for configuration file parsing
- Scopeguard for RAII-style resource management

Dev Setup Instructions
---------------------
To set up the development environment for PSR:

1. Install Rust by following the instructions at https://www.rust-lang.org/tools/install
2. Clone the PSR repository:
   ```
   git clone https://github.com/osteele/package-script-runner.git
   cd package-script-runner
   ```
3. Install `just` command runner:
   ```
   cargo install just
   ```
4. Install development dependencies:
   ```
   cargo install cargo-husky
   ```
5. Run tests to verify setup:
   ```
   just test
   ```

You're now ready to start developing PSR!

Core Functionality
----------------
PSR is a TUI-based script runner that supports multiple package managers. The main features are:

1. Package Manager Support
   - NPM/Node.js projects (package.json)
   - Rust/Cargo projects (Cargo.toml)

   Detection is automatic based on project files.

2. Project Management
   - Add projects with aliases
   - List registered projects
   - Remove projects
   - Reference projects via -p flag

3. Script Execution
   - Direct script execution: psr <script-name>
   - Special 'run' command for custom scripts
   - Supports both successful and failing script scenarios

Error Handling
-------------
The system handles several error cases:
- Invalid project references
- Nonexistent scripts
- Unknown commands
- Directories without package managers
- Invalid script execution

CLI Interface
------------
Command structure:
psr [OPTIONS] [COMMAND] [ARGS]

Main commands:
- projects add <alias> <path>
- projects list
- projects remove <alias>
- --list : Show available scripts
- --version : Show version
- --help : Show help

Architecture Diagram
------------------
```mermaid
graph TD
PSR[PSR Core] --> PM[Project Manager]
PSR --> SR[Script Runner]
PM --> PS[Project Store]
SR --> PMD[Package Manager Detection]
PMD --> NPM[NPM]
PMD --> Cargo[Cargo]
PMD --> Future[Other Package Managers]
```

Testing Strategy
--------------
Integration tests cover:
1. Package manager detection
2. Comprehensive NPM functionality
3. Project management operations
4. Error handling scenarios
5. CLI options and commands

Each test category ensures isolated testing using temporary directories and cleanup.

Development Guidelines
-------------------
1. Always implement proper error handling with descriptive messages
2. Support both success and failure scenarios for script execution
3. Maintain clear separation between project management and script execution
4. Ensure proper cleanup of temporary resources
5. Follow semantic versioning for releases
6. Use `just` for running development tasks
7. Use `cargo fmt` for consistent code formatting
8. Run `cargo clippy` to catch common issues and improve code quality
9. Ensure `cargo test` passes before committing changes

Future Considerations
------------------
1. Support for additional package managers
2. Enhanced TUI features
3. Script execution history
4. Project favorites or tagging
5. Custom script aliases

Note: This is a living document and should be updated as the project evolves.
