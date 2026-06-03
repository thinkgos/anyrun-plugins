# Agents Instructions for anyrun-plugins

These instructions apply to all AI-assisted contributions to `thinkgos/anyrun-plugins`, it describes the architecture, command, and development workflow of the project.

## Project Overview

**anyrun-plugins** is a plugin collection for [anyrun](https://github.com/anyrun-org/anyrun), providing various search plugins to extend anyrun's functionality.

## Architecture Overview

### Project Structure

```
anyrun-plugins/
├── Cargo.toml           # Workspace manifest
└── ssh-pattern/                 # SSH search plugin
```

This is a Cargo workspace containing multiple independent plugins. Each plugin is a separate crate that implements the anyrun plugin interface using `anyrun-plugin` and `abi_stable`.

### Technical Stack

- **Language**: Rust (Edition 2024, MSRV 1.96)
- **Build System**: Cargo
- **Plugin Framework**: `anyrun-plugin` with `abi_stable` for ABI stability
- **Package**: Workspace with multiple plugins

### Dependencies

Core dependencies (detail see Cargo.toml):

- `anyrun-plugin`: Plugin interface for anyrun
- `abi_stable`: ABI-stable library for plugin safety

**Build dependencies:**

## Development Guidelines

### Plugin Structure

Each plugin follows the anyrun plugin interface:
- Exports `plugin` function that returns an `anyrun_plugin::Entry`
- Uses `abi_stable` for stable ABI across Rust versions
- Implements search/query functionality for external services

### Build & Run

```sh
# Build all plugins
cargo build

# Build specific plugin
cargo build -p ssh-pattern

# Release build (optimized)
cargo build --release

# Run tests
cargo test
cargo test -p ssh-pattern
```

### Testing

```sh
# Run all tests
cargo test

# Run all tests all features
cargo test --all-features

# Run specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture

# Run tests in specific module
cargo test <module_name>::
```

### Linting & Quality

```sh
# Check without building
cargo check

# Format code
cargo fmt -- --check

# Run clippy lints
cargo clippy

# Check all targets and features
cargo clippy --all-targets --all-features -- -D warnings
```

### Build Verification (Mandatory)

### Build Optimizations

Release profile (Cargo.toml:79-84):

- `opt-level = z`: Maximum optimization
- `lto = true`: Link-time optimization
- `codegen-units = 1`: Single codegen for better optimization
- `strip = true`: Remove debug symbols
- `panic = "abort"`: Smaller binary size

**CRITICAL**: After ANY Rust file edits, ALWAYS run the full quality check pipeline before committing:

```sh
cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo check && cargo test
```

**Rules**:

- Never commit code that hasn't passed all 4 checks
- Fix ALL clippy warnings before moving on (zero tolerance)
- If build fails, fix it immediately before continuing to next task

## Commit Message Convention

This project follows [Conventional Commits](https://www.conventionalcommits.org/).

### Format

```
head: <type>(<scope>): <subject>
<body>
<footer>
```

**head**:

- type: feat, fix, doc, perf, style, refactor, test, chore, security, revert
- scope: can be empty (eg. if the change is a global or difficult to assign to a single component)
- subject: start with verb (such as 'change'), 50-character line

**body**: 72-character wrapped, This should answer:

- Why was this change necessary?
- How does it address the problem?
- Are there any side effects?

**footer**:

- Include a link to the ticket, if any.(`Refs: #123` or `Closes: #123`)
- BREAKING CHANGE(`BREAKING CHANGE: description`)

### Examples

```
feat(registry): add support for private mirrors

Add support for custom Go mirrors via GOUP_GO_REGISTRY environment
variable. This allows organizations to use internal mirrors for downloads.

The implementation adds a new Registry trait implementation and
updates the installer to honor the environment variable.

Refs: #42
```

## Error Handling

Each plugin follows Rust best practices for error handling:

**Rules**:

- **anyhow::Result** for plugin code
- **ALWAYS** use `?` operator, "SUGGEST" `.context("description")` with `?` operator
- **NO unwrap()** in production code (tests only - use expect("explanation") if needed)

Examples:

```rust
use anyhow::{Context, Result};

pub fn filter_git_log(input: &str) -> Result<String> {
    let lines: Vec<_> = input
        .lines()
        .filter(|line| !line.is_empty())
        .collect();

    // ✅ RIGHT: Context on error
    let hash = extract_hash(lines[0])
        .context("Failed to extract commit hash from git log")?;

    // ✅ RIGHT: Convert on error
    let hash = extract_hash(lines[0])?;

    // ❌ WRONG: Panic in production
    let hash = extract_hash(lines[0]).unwrap();

    Ok(format!("Commit: {}", hash))
}
```

## Boundaries

**Don't panic on failure** (breaks user workflow) Always use `?` operator, Log to stdout if need.
