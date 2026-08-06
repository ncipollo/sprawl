# Sprawl

## Architecture

The code is broken down into two layers (see `docs/arch.md`):

- `ui` — The view layer, built with gpui. `main` calls the top-level entry point in this module; each view gets its own file and implements `Render`. No domain logic lives in this layer.
- `feature` — Where all domain logic lives. The ui layer calls through to feature.

## After Each Change
Run the following commands after every code change and fix any issues before considering the change complete:

1. `cargo fmt` - Format all code
2. `cargo test` - Run all tests
3. `cargo clippy` - Run linter; fix all warnings and errors before completing the change

### Fixing Clippy Complexity Warnings
When clippy reports `cognitive_complexity`, `too_many_lines`, or `too_many_arguments` warnings, fix them by refactoring — never suppress with `#[allow]`:
- Extract logical sub-steps into well-named helper functions.
- When a file accumulates many functions, reorganize into helper files and structs (following the module conventions below).

## Dependencies
Always use exact versions for dependencies in `Cargo.toml` (e.g., `"4.5.60"` not `"4"`). Check `Cargo.lock` for the resolved version when pinning.

## Module Conventions
Never use `mod.rs`. Always use the modern Rust style: create a top-level file (e.g., `foo.rs`) as the module root, and a matching folder (`foo/`) for any submodules.

## Imports
Always use `use` imports rather than full crate paths at call sites. For example, prefer `use crate::feature::greeting;` + `greeting::message()` over `crate::feature::greeting::message()`.
