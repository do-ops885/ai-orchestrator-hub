# Agent Development Guide

## Build/Lint/Test Commands

### Backend (Rust)
- **Build**: `cargo build` or `cargo build --release`
- **Lint**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Format**: `cargo fmt --all -- --check`
- **Test All**: `cargo test --all-features`
- **Single Test**: `cargo test test_name` or `cargo test -- --test-threads=1 test_name`
- **Run**: `cargo run` (basic) or `cargo run --features advanced-neural`

### Frontend (TypeScript)
- **Build**: `cd frontend && npm run build`
- **Lint**: `cd frontend && npm run lint:check`
- **Format**: `cd frontend && npm run lint:fix`
- **Type Check**: `cd frontend && npx tsc --noEmit`
- **Dev Server**: `cd frontend && npm run dev`

## Code Style Guidelines

### Rust
- **Line Width**: 100 characters max
- **Indentation**: 4 spaces, no tabs
- **Line Endings**: Unix (LF)
- **Imports**: Reorder with `cargo fmt`
- **Functions**: `fn_params_layout = "Tall"`
- **Error Handling**: Use `anyhow::Result<T>`
- **Naming**: snake_case for variables/functions, PascalCase for types
- **Avoid**: `unwrap()`, `panic!`, `clone_on_ref_ptr`

### TypeScript/React
- **Quotes**: Single quotes (`'`)
- **Semicolons**: Never use
- **Indentation**: 2 spaces
- **Line Endings**: Unix (LF)
- **Types**: Use `interface` over `type`
- **Components**: PascalCase, functional with hooks
- **Imports**: Group by external/internal, alphabetize
- **Error Handling**: Try-catch with proper error boundaries
- **Naming**: camelCase for variables/functions, PascalCase for components/types

### General
- **Comments**: No unnecessary comments, focus on clarity
- **Async**: Use `async/await` throughout
- **Security**: Never expose secrets, validate all inputs
- **Performance**: Follow clippy performance lints, avoid memory leaks