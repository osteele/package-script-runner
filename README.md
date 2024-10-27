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

```bash
cargo install package-script-runner
```

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
  -v, --verbose      Show verbose output
  -h, --help         Print help information
  -V, --version      Print version information
```

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

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Copyright 2024 Oliver Steele.

MIT License - see [LICENSE](LICENSE) for details
