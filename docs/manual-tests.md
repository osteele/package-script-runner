# Manual Testing Guide for Package Script Runner

## Setup Test Projects

```bash
./scripts/create-test-projects.sh
```

This creates test projects for:
- npm project with basic scripts
- yarn project with test scripts
- pnpm project with vite scripts
- bun project with dev scripts
- deno project with start/test scripts
- Rust project with cargo scripts
- Python projects (pip, poetry, uv)
- Go project with Makefile

## CLI Mode Navigation Tests

Test CLI mode navigation:

```bash
bash
psr
```

Expected:
- Shows working directory
- Lists scripts with shortcuts
- Can use letter shortcuts (t for test, etc)
- Can use number shortcuts (1-9)
- 't' switches to TUI mode
- 'q' quits

## TUI Mode Tests

```bash
psr --tui
```

Expected:
- Full terminal interface
- Script list with icons
- Preview panel shows details
- Can navigate with arrows/hjkl
- Enter runs script
- Esc/q quits

## Advanced Script Execution Tests

1. Test script with arguments:

```bash
psr run --watch
```

2. Test script synonyms:

```bash
cd testdata/projects/npm-project
psr dev # Should run start with NODE_ENV=dev
```

## Theme Tests

```bash
psr --tui # test default theme
psr --tui --theme dark # test dark theme
psr --tui --theme light # test light theme
NO_COLOR=1 psr --tui # test with NO_COLOR
```

## Additional Package Manager Tests

Test each package manager not covered by automation:

```bash
cd testdata/projects/yarn-project
psr test
```

```bash
cd testdata/projects/pnpm-project
psr dev
```

```bash
cd testdata/projects/bun-project
psr dev
```

```bash
cd testdata/projects/deno-project
psr test
```

```bash
cd testdata/projects/python-project
psr test
```

```bash
cd testdata/projects/poetry-project
psr test
```

```bash
cd testdata/projects/uv-project
psr test
```
