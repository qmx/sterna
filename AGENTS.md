# Sterna Agent Guide

## Build, Lint, and Test Commands

### Build Commands
```bash
# Build the project
cargo build

# Build in release mode
cargo build --release

# Check compilation without building
cargo check

# Run all tests
cargo test

# Run a specific test file
cargo test --lib

# Run a specific test function
cargo test <test_function_name>

# Run tests with more verbose output
cargo test -- --nocapture

# Run only integration tests (if any)
cargo test --test integration
```

### Linting and Formatting
```bash
# Format code using rustfmt
cargo fmt

# Check formatting without making changes
cargo fmt --check

# Lint with clippy
cargo clippy

# Run clippy with warnings as errors
cargo clippy -- -D warnings
```

## Code Style Guidelines

### Imports and Module Structure
- All modules are organized in `src/commands/` directory with corresponding `mod.rs`
- Use explicit imports at the top of each file for clarity
- Follow Rust's standard module structure conventions
- All modules must be declared in `src/commands/mod.rs`

### Formatting and Naming Conventions
- Use snake_case for function and variable names
- Use PascalCase for struct and enum names
- Use SCREAMING_SNAKE_CASE for constants
- Use descriptive names that clearly indicate purpose
- Follow Rust's standard formatting conventions (2 spaces for indentation)

### Types and Error Handling
- All functions should return `Result<T, Error>` where appropriate
- Custom errors are defined in `src/error.rs` with clear error messages
- Handle Git errors using the `From<git2::Error>` trait
- Handle IO errors using the `From<std::io::Error>` trait
- Use proper error propagation with `?` operator
- All custom errors should implement `Display` and `Error` traits

### Documentation and Comments
- Document all public functions with Rust doc comments (`///`)
- Use `//!` for module-level documentation
- Prefer clear, descriptive names over excessive comments
- Include examples in documentation where helpful

### Code Structure
- Commands are implemented as separate files in `src/commands/`
- Core functionality is organized in `src/types.rs`, `src/storage.rs`, `src/index.rs`
- Each command file should follow the same pattern:
  - Public `run()` function that handles the command logic
  - Error handling with appropriate return types
  - Proper validation of inputs

### Git Integration
- All storage operations use git2 crate for Git integration
- Blob storage uses `git2::Repository::blob()` method
- File structure follows Git conventions (sterna/index/issues, sterna/index/edges)
- All git operations should be wrapped in proper error handling

## Testing Guidelines

### Unit Tests
- Place unit tests in the same file as the code being tested
- Use `#[cfg(test)]` module for test functions
- Test both success and failure cases
- Mock external dependencies where possible

### Integration Tests
- Integration tests should be placed in a separate `tests/` directory if it exists
- Test end-to-end functionality including Git operations
- Ensure tests clean up after themselves

## Project Structure Notes

This is a Rust project using Cargo. Key files and directories:
- `src/main.rs`: Main entry point with CLI argument parsing
- `src/types.rs`: Core data structures (Issue, Edge, etc.)
- `src/storage.rs`: Git blob storage operations
- `src/index.rs`: Issue and edge index management
- `src/id.rs`: ID generation logic
- `src/dag.rs`: Dependency graph and cycle detection
- `src/commands/`: Individual command implementations

## Special Considerations

### Error Handling Patterns
All functions that can fail should return `Result<T, Error>` where:
- `Error` is a custom enum defined in `src/error.rs`
- Git operations use `git2::Error` wrapped in `Error::Git`
- IO operations use `std::io::Error` wrapped in `Error::Io`
- JSON serialization uses `serde_json::Error` wrapped in `Error::Json`

### Idempotency
- Commands should be idempotent where possible (e.g., calling `st init` multiple times should not break things)
- Use LWW (Last Write Wins) merge strategy for concurrent updates

### Git Operations
- All Git operations are atomic and use proper error handling
- Repository discovery uses `git2::Repository::discover(".")`
- All git blob operations go through the storage module

## Cursor/Copilot Instructions

This project follows standard Rust conventions. No special Cursor rules were found in .cursor/rules/ or .cursorrules.
