# Package Script Runner (psr)

<img src="docs/logo.svg" width="100" alt="Package Script Runner Logo">

A fast TUI-based script selector for Node.js (npm, bun, deno), Python (pip,
poetry, uv), and Rust projects. Quickly find and run package scripts with
keyboard shortcuts, search, and smart project type detection.

## Features

- 🚀 **Fast TUI Interface**: Navigate and run scripts quickly using keyboard shortcuts
- 🔍 **Smart Search**: Fuzzy find scripts by name, command, or description
- 🎨 **Color Coded**: Scripts are color-coded by type (build, test, dev, etc.)
- 📦 **Multi-Project Support**:
  - Node.js:
    - npm
    - yarn
    - pnpm
    - bun
    - deno
  - Python:
    - pip
    - poetry
    - uv
  - Rust:
    - Cargo
- 📁 **Smart Detection**:
  - Automatically detects the right project type and package manager
  - Finds `package.json`, `pyproject.toml`, `requirements.txt`, or `Cargo.toml` in parent directories
  - Stops at home directory
- ⌨️ **Keyboard Shortcuts**: Quick access to common scripts (`d` for dev, `t`
  for test, etc.)
- 📝 **Rich Preview**: See full script details including descriptions
- 🔄 **Live Filtering**: Results update as you type
- 🔄 **Script Synonyms**: Support for common script name alternatives

## Installation

### Building from Source

To build and install PSR from source:

1. Ensure you have Rust and Cargo installed. If not, install them from
   [https://rustup.rs/](https://rustup.rs/).

2. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/package-script-runner.git
   cd package-script-runner
   ```

3. Build the project:
   ```bash
   cargo build --release
   ```

4. The compiled binary will be in `target/release/psr`. You can run it directly
   or install it to your system:
   ```bash
   cargo install --path .
   ```

This will install the `psr` binary to your Cargo bin directory (usually
`~/.cargo/bin/`).

### Verifying Installation

After installation, you can verify that PSR is installed correctly by running:

```bash
psr --version
```

This should display the version number of PSR.

## Usage

Navigate to any directory containing a `package.json`, `pyproject.toml`, `requirements.txt`, or `Cargo.toml` (or parent directory) and run:

```bash
psr [COMMAND] [SCRIPT_NAME] [-- SCRIPT_ARGS...]
```

By default, PSR runs in a simple CLI mode that shows available scripts and their shortcuts. Press the corresponding key to run a script, 't' to switch to TUI mode, or 'q' to quit.

If a script name is provided as a positional argument, PSR will run that script directly without launching any interface. You can pass additional arguments to the script by adding `--` followed by the arguments.

### Command Line Options

```text
Usage: psr [OPTIONS] [COMMAND] [SCRIPT_NAME] [-- SCRIPT_ARGS...]

Arguments:
  [COMMAND]         Command to execute (run, dev, test, etc)
  [SCRIPT_NAME]     Name of the script to run directly
  [SCRIPT_ARGS]...  Additional arguments to pass to the script

Options:
  -d, --dir <PATH>   Start in a specific directory instead of current directory
  -l, --list         List available scripts without launching interface
      --tui          Start in TUI mode instead of CLI mode
      --theme <THEME> Set the color theme (dark or light) [env: PSR_THEME=] [default: dark]
  -v, --verbose      Show verbose output
  -h, --help         Print help information
  -V, --version      Print version information
```

### Interface Modes

PSR offers two interface modes:

1. **CLI Mode** (default):
   - Simple command-line interface
   - Shows available scripts with their shortcuts
   - Use letter shortcuts for common scripts
   - Numbers 1-9 for additional scripts
   - Press 't' to switch to TUI mode
   - Press 'q' to quit

2. **TUI Mode** (access with `--tui` or by pressing 't' in CLI mode):
   - Full terminal user interface
   - Rich preview and navigation
   - Advanced search capabilities
   - More extensive script list

### Special Commands

The following commands can be run directly without using `run`:
- `dev`
- `start`
- `run`
- `build`
- `deploy`
- `clean`
- `watch`
- `test`
- `format`
- `lint`
- `typecheck`

### Environment Variables

- `PSR_THEME`: Set the color theme (dark or light). Overridden by the `--theme`
  CLI option if provided.
- `NO_COLOR`: When set (to any value), disables all color output. This adheres
  to the [NO_COLOR standard](https://no-color.org/).

### Keyboard Controls

- `/`: Enter search mode
- `↑`/`↓` or `j`/`k`: Navigate scripts
- `Enter`: Run selected script
- `Esc`: Exit search or quit
- `q`: Quit

### Priority Script Shortcuts

Quick access to common scripts:
- `d`: dev
- `s`: start
- `b`: build
- `t`: test
- `w`: watch
- `f`: format
- `c`: clean

### Search

Press `/` to enter search mode and type to filter scripts by:
- Script name
- Command content
- Description (if available)

### Script Synonyms

PSR supports some common script name alternatives:

- `dev`: If no `dev` script exists, PSR will look for `start` or `run` scripts. When using this synonym, PSR sets the `NODE_ENV` environment variable to `dev`.
- `typecheck` and `tc`: These are treated as synonyms. If one doesn't exist but the other does, PSR will run the existing script.

## Examples
Launch the script selector:
```bash
psr
```

Run a special commands directly:
```bash
psr test
psr dev
psr build
```

Run arbitrary scripts using `run`:
```bash
psr run custom-task
```

Run scripts with additional arguments:
```bash
psr test -- --watch
psr run custom-task -- --flag value
```

Use a script synonym:
```bash
psr dev  # Runs 'start' or 'run' if 'dev' doesn't exist, with NODE_ENV=dev
psr typecheck  # Runs 'tc' if 'typecheck' doesn't exist
```

Run the default 'run' script:
```bash
psr run # Tries to find a script named 'run', 'dev', or their synonyms
```

List scripts:
```bash
psr --list
```

Run a specific script directly:
```bash
psr build
```

Run the TUI and return to it after running a script:
```bash
psr --loop
```

Run with a light theme:
```bash
psr --theme light
```

or:

```bash
PSR_THEME=light psr
```

Run with a dark theme (default):
```bash
psr --theme dark
```

Run with colors disabled:
```bash
NO_COLOR=1 psr
```

Show version:
```bash
psr --version
```

## Project Type Detection

Package Script Runner automatically detects your project type through:
1. Lock files:
   - `package-lock.json` (npm)
   - `yarn.lock` (yarn)
   - `pnpm-lock.yaml` (pnpm)
   - `bun.lockb` (bun)
   - `deno.lock` (deno)
   - `poetry.lock` (poetry)
   - `Cargo.toml` (Rust)
2. Config files (fallback):
   - `.npmrc`
   - `.yarnrc`/`.yarnrc.yml`
   - `pyproject.toml`
   - `requirements.txt`
   - `uv.toml`

## Configuration

### Config File

PSR looks for a configuration file in the following locations (in order):
1. `.pkr.toml` in the current directory
2. `.pkr.toml` in your home directory

The configuration file uses TOML format. Currently supported options:

- `theme`: Set the color theme (dark or light)

```toml
# Theme can be "dark", "light", or "nocolor"
theme = "dark" # Optional, defaults to "dark"
```

Configuration priority (highest to lowest):
1. Command line arguments (e.g., `--theme light`)
2. Environment variables (`NO_COLOR`, then `PSR_THEME`)
3. Project config file (`./.pkr.toml`)
4. User config file (`~/.pkr.toml`)
5. Built-in defaults

### Theming

PSR supports two color themes: dark (default) and light. You can set the theme
using the `--theme` CLI option or the `PSR_THEME` environment variable. The CLI
option takes precedence over the environment variable.

To set the theme using the environment variable:

```bash
export PSR_THEME=light
psr
```

To disable colors entirely, you can use the `NO_COLOR` environment variable, which adheres to the [NO_COLOR standard](https://no-color.org/):

```bash
export NO_COLOR=1
psr
```

This will run PSR without any color output, regardless of the theme setting.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Copyright 2024 Oliver Steele.

MIT License - see [LICENSE](LICENSE) for details
