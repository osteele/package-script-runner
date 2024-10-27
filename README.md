# Package Script Runner (psr)

A fast TUI-based script runner for Node.js npm, bun, and deno projects. Quickly
find and run package scripts with keyboard shortcuts, search, and smart package
manager detection.

![TUI Screenshot Placeholder]

## Features

- 🚀 **Fast TUI Interface**: Navigate and run scripts quickly using keyboard shortcuts
- 🔍 **Smart Search**: Fuzzy find scripts by name, command, or description
- 🎨 **Color Coded**: Scripts are color-coded by type (build, test, dev, etc.)
- 📦 **Multi-Package Manager Support**:
  - npm
  - yarn
  - pnpm
  - bun
  - deno
- 📁 **Smart Detection**:
  - Automatically detects the right package manager
  - Finds `package.json` in parent directories
  - Stops at home directory
- ⌨️ **Keyboard Shortcuts**: Quick access to common scripts (`d` for dev, `t`
  for test, etc.)
- 📝 **Rich Preview**: See full script details including descriptions
- 🔄 **Live Filtering**: Results update as you type

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

Navigate to any directory containing a `package.json` (or parent directory) and
run:

```bash
psr
```

### Command Line Options

```
Usage: psr [OPTIONS]

Options:
  -d, --dir <PATH>   Start in a specific directory instead of current directory
  -l, --list         List available scripts without launching TUI
      --theme <THEME> Set the color theme (dark or light) [env: PSR_THEME=] [default: dark]
  -v, --verbose      Show verbose output
  -h, --help         Print help information
  -V, --version      Print version information
```

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

## Examples

List scripts in a specific project:
```bash
psr --dir ~/projects/my-app --list
```

Run with a light theme:
```bash
psr --theme light
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

## Configuration

Package Script Runner automatically detects your package manager through:
1. Lock files:
   - `package-lock.json` (npm)
   - `yarn.lock` (yarn)
   - `pnpm-lock.yaml` (pnpm)
   - `bun.lockb` (bun)
   - `deno.lock` (deno)
2. Config files (fallback):
   - `.npmrc`
   - `.yarnrc`/`.yarnrc.yml`

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
