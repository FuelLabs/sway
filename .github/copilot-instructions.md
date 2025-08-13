# Sway Programming Language Toolchain

Sway is a domain-specific language for the Fuel blockchain platform, heavily inspired by Rust. This repository contains the complete Sway toolchain including the compiler (forc), language server, formatter, standard library, and various plugins.

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

## Working Effectively

### Bootstrap and Build the Repository

Build the entire Sway workspace:
```bash
cargo build --locked --workspace
```
**NEVER CANCEL: Initial build takes 15-20 minutes. Set timeout to 30+ minutes.**

### Essential Commands

Install the Sway toolchain locally:
```bash
# Install forc compiler
cargo install --locked --path ./forc

# Install essential plugins
cargo install --locked --path ./forc-plugins/forc-fmt
cargo install --locked --path ./forc-plugins/forc-lsp
cargo install --locked --path ./forc-plugins/forc-client

# Verify installation
forc --version
```

Test that forc works correctly:
```bash
# Quick test without installation
cargo run --bin forc -- --help

# Build the Sway standard library (takes ~14 seconds)
cargo run --bin forc -- build --path sway-lib-std

# Run standard library unit tests (takes ~15 seconds)
cargo run --bin forc -- test --path sway-lib-std
```

### Create and Build Projects

Create a new Sway project:
```bash
cargo run --bin forc -- new my-project
cd my-project

# Update Forc.toml to use local std lib during development
echo 'std = { path = "../sway-lib-std/" }' >> Forc.toml

# Build the project (takes ~14 seconds)
cargo run --bin forc -- build

# Run project tests
cargo run --bin forc -- test
```

### Development and Testing

Run Rust unit tests for individual packages:
```bash
# Test a small package (~20-25 seconds)
cargo test --locked -p sway-types

# Test the core compiler (~80 seconds)  
cargo test --locked -p sway-core
```
**NEVER CANCEL: Package tests can take 15-90 seconds. Set timeout to 5+ minutes.**

Run workspace tests (excluding integration tests):
```bash
cargo test --locked --release --workspace --exclude forc-debug --exclude sway-lsp --exclude forc-client --exclude forc-mcp --exclude forc --exclude forc-node
```
**NEVER CANCEL: Workspace tests take 10-15 minutes. Set timeout to 30+ minutes.**

### Code Quality and Linting

Run formatting check (takes ~3 seconds):
```bash
cargo fmt --all -- --check
```

Run clippy linter:
```bash
# Quick check on single package (~13 seconds)
cargo clippy -p sway-types

# Full workspace clippy check (~10-15 minutes)
cargo clippy --all-features --all-targets -- -D warnings
```
**NEVER CANCEL: Full clippy takes 10-15 minutes. Set timeout to 30+ minutes.**

Check Sway code formatting:
```bash
cargo run --locked -p forc-fmt -- --check --path ./sway-lib-std
cargo run --locked -p forc-fmt -- --check --path ./examples
```

### Advanced Testing

Run end-to-end tests (requires fuel-core):
```bash
# Note: These tests require a running fuel-core node
# Install fuel-core first (see fuel-core setup section)
fuel-core run --db-type in-memory --debug &
sleep 5

# Run E2E tests (~10-15 minutes)
cargo run --locked --release --bin test -- --locked
```
**NEVER CANCEL: E2E tests take 10-15 minutes. Set timeout to 30+ minutes.**

Run forc unit tests:
```bash
# Standard library unit tests in debug and release (~2-3 minutes each)
forc test --path sway-lib-std
forc test --release --path sway-lib-std

# In-language unit tests
forc test --path test/src/in_language_tests
forc test --release --path test/src/in_language_tests
```

## Validation

### Build Validation
- ALWAYS run `cargo build --locked --workspace` after making changes to core components
- Build and test any Sway code changes with `forc build` and `forc test`
- Validate examples still work: `cargo run --locked -p forc -- build --locked --path ./examples/Forc.toml`

### Code Quality Validation
- ALWAYS run `cargo fmt --all -- --check` before committing
- ALWAYS run `cargo clippy --all-features --all-targets -- -D warnings` before submitting
- For Sway code changes, run `cargo run -p forc-fmt -- --check --path <changed-path>`

### End-to-End Validation Scenarios
After making significant changes:
1. Create a new project and verify it builds and runs tests
2. Build and test the standard library
3. Build all examples to ensure compatibility
4. Run clippy and formatting checks
5. If touching core compiler, run workspace tests

### Manual Testing Scenarios
- **New Project Creation**: `forc new test-proj && cd test-proj && forc build && forc test`
- **Standard Library**: Build and test sway-lib-std
- **Examples**: Build examples workspace to verify compatibility
- **Plugins**: Test formatter and LSP on sample code

## Common Tasks and File Locations

### Key Directories
- `forc/` - Main Sway compiler and CLI
- `sway-core/` - Core compiler implementation
- `sway-lib-std/` - Sway standard library
- `sway-lsp/` - Language Server Protocol implementation
- `forc-plugins/` - Forc plugins (fmt, lsp, client, etc.)
- `examples/` - Example Sway projects
- `test/` - Test infrastructure and test cases
- `docs/` - Documentation source

### Important Files
- `Cargo.toml` - Workspace configuration
- `ci_checks.sh` - Script that runs most CI checks locally
- `.github/workflows/ci.yml` - Complete CI pipeline
- `sway-lib-std/Forc.toml` - Standard library configuration

### Network Limitations
- **Network timeouts**: This environment experiences frequent network timeouts when accessing crates.io
- `cargo install` commands may fail due to network connectivity issues
- Use `cargo run --bin <tool>` instead of installing when possible
- Always use `--locked` flag to use existing Cargo.lock

### Timing Expectations
- **Initial build**: 15-20 minutes (with network delays)
- **Incremental builds**: 1-5 minutes depending on changes
- **Individual package tests**: 15-90 seconds
- **Workspace tests**: 10-15 minutes  
- **Standard library build**: ~14 seconds
- **Standard library tests**: ~15 seconds
- **Formatting check**: ~3 seconds
- **Single package clippy**: ~13 seconds
- **Full workspace clippy**: 10-15 minutes

### Experimental Features
The Sway compiler supports experimental features:
```bash
# Build with experimental features
forc build --experimental const_generics,new_hashing

# Test with experimental features  
forc test --experimental const_generics,new_hashing
```

## Dependencies and External Tools

### Required Dependencies
- **Rust toolchain**: Use `rustup default stable`
- **cargo**: Included with Rust toolchain

### Optional Dependencies (for full testing)
- **fuel-core**: Required for E2E tests (install from GitHub releases)
- **cargo-generate**: For integration test workflows

### Fuel-Core Setup
For running end-to-end tests, install fuel-core:
```bash
# Get the version from Cargo.toml
FUEL_CORE_VERSION=$(grep -E 'fuel-core-client.*version' Cargo.toml | cut -d'"' -f4)

# Download and install (Linux x64)
curl -sSLf "https://github.com/FuelLabs/fuel-core/releases/download/v${FUEL_CORE_VERSION}/fuel-core-${FUEL_CORE_VERSION}-x86_64-unknown-linux-gnu.tar.gz" -L -o fuel-core.tar.gz
tar -xzf fuel-core.tar.gz
chmod +x fuel-core-*/fuel-core
sudo mv fuel-core-*/fuel-core /usr/local/bin/

# Run fuel-core for testing
fuel-core run --db-type in-memory --debug
```

This covers the essential workflow for working effectively with the Sway codebase. Always build and test your changes thoroughly before submitting.