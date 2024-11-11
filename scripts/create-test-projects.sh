#!/bin/bash

# Node.js projects
mkdir -p testdata/projects/npm-project
cat > testdata/projects/npm-project/package.json << EOF
{
  "name": "npm-test",
  "scripts": {
    "start": "node index.js",
    "test": "jest",
    "build": "tsc",
    "lint": "eslint ."
  }
}
EOF
touch testdata/projects/npm-project/package-lock.json

mkdir -p testdata/projects/yarn-project
cat > testdata/projects/yarn-project/package.json << EOF
{
  "name": "yarn-test",
  "scripts": {
    "start": "node index.js",
    "test": "jest"
  }
}
EOF
touch testdata/projects/yarn-project/yarn.lock

mkdir -p testdata/projects/pnpm-project
cat > testdata/projects/pnpm-project/package.json << EOF
{
  "name": "pnpm-test",
  "scripts": {
    "dev": "vite",
    "build": "vite build"
  }
}
EOF
touch testdata/projects/pnpm-project/pnpm-lock.yaml

mkdir -p testdata/projects/bun-project
cat > testdata/projects/bun-project/package.json << EOF
{
  "name": "bun-test",
  "scripts": {
    "dev": "bun run index.ts",
    "test": "bun test"
  }
}
EOF
touch testdata/projects/bun-project/bun.lockb

mkdir -p testdata/projects/deno-project
cat > testdata/projects/deno-project/deno.json << EOF
{
  "tasks": {
    "start": "deno run main.ts",
    "test": "deno test"
  }
}
EOF
touch testdata/projects/deno-project/deno.lock

# Rust project
mkdir -p testdata/projects/rust-project
cat > testdata/projects/rust-project/Cargo.toml << EOF
[package]
name = "rust-test"
version = "0.1.0"

[package.metadata.scripts]
dev = "cargo watch -x run"
docs = "cargo doc --open"

[[bin]]
name = "cli"
path = "src/main.rs"
EOF

# Python projects
mkdir -p testdata/projects/pip-project
cat > testdata/projects/pip-project/requirements.txt << EOF
pytest==7.4.0
ruff==0.1.0
requests==2.31.0
EOF

mkdir -p testdata/projects/poetry-project
cat > testdata/projects/poetry-project/pyproject.toml << EOF
[tool.poetry]
name = "poetry-test"
version = "0.1.0"
description = "Test poetry project"

[tool.poetry.dependencies]
python = "^3.9"
requests = "^2.31.0"

[tool.poetry.dev-dependencies]
pytest = "^7.4.0"
ruff = "^0.1.0"
EOF
touch testdata/projects/poetry-project/poetry.lock

mkdir -p testdata/projects/uv-project
cat > testdata/projects/uv-project/pyproject.toml << EOF
[project]
name = "uv-test"
version = "0.1.0"

[build-system]
requires = ["uv"]
EOF
cat > testdata/projects/uv-project/uv.toml << EOF
[dependencies]
requests = "2.31.0"
ruff = "0.1.0"
EOF
# Go project
mkdir -p testdata/projects/go-project
cat > testdata/projects/go-project/go.mod << EOF
module example.com/test

go 1.21
EOF

cat > testdata/projects/go-project/main.go << EOF
package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
}
EOF

cat > testdata/projects/go-project/Makefile << EOF
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

docker:
	docker build -t test-app .
EOF
