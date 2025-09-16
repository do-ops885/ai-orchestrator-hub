# Contributing to AI Orchestrator Hub Backend

Thank you for your interest in contributing to the AI Orchestrator Hub Backend! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- **Rust**: Version 1.70+ (check with `rustc --version`)
- **Cargo**: Package manager (comes with Rust)
- **Git**: Version control system

### Local Development Environment

1. **Clone the repository:**
   ```bash
   git clone https://github.com/do-ops885/ai-orchestrator-hub.git
   cd ai-orchestrator-hub/backend
   ```

2. **Install dependencies:**
   ```bash
   cargo build
   ```

3. **Run tests to verify setup:**
   ```bash
   cargo test
   ```

4. **Start development server:**
   ```bash
   cargo run
   ```

### IDE Setup

#### VS Code (Recommended)
- Install Rust Analyzer extension
- Install CodeLLDB extension for debugging
- Configure settings for Rust formatting

#### Other Editors
- Vim/Neovim: Install rust.vim plugin
- Emacs: Install rust-mode
- IntelliJ IDEA: Install Rust plugin

## Code Standards

### Rust Code Style

We follow the official Rust style guidelines with some project-specific conventions:

#### Formatting
- Use `rustfmt` for consistent formatting (automatically enforced)
- Maximum line length: 100 characters
- 4 spaces for indentation

#### Naming Conventions
- **Functions/Methods**: `snake_case`
- **Types/Structs**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

#### Code Organization
```rust
// Group imports by type
use std::collections::HashMap;
use tokio::sync::RwLock;

// External crates
use serde::{Deserialize, Serialize};
use axum::Router;

// Local modules
use crate::agents::Agent;
use crate::tasks::Task;
```

### Documentation Standards

#### Code Documentation
- Use `///` for public API documentation
- Include examples for complex functions
- Document error conditions and panics

```rust
/// Creates a new agent with the specified configuration.
///
/// # Arguments
/// * `config` - Agent configuration including type and capabilities
///
/// # Returns
/// Returns the agent ID on success, or an error if creation fails
///
/// # Examples
/// ```
/// let config = AgentConfig::new("worker", vec!["data_processing"]);
/// let agent_id = create_agent(config).await?;
/// ```
pub async fn create_agent(config: AgentConfig) -> Result<String, HiveError> {
    // Implementation
}
```

#### Commit Messages

Follow conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test additions/modifications
- `chore`: Maintenance tasks

Examples:
```
feat(agents): add capability-based task matching

fix(api): handle malformed JSON in agent creation

docs(api): update WebSocket endpoint documentation

test(verification): add integration tests for simple verification
```

### Testing Requirements

#### Unit Tests
- All public functions must have unit tests
- Test both success and error cases
- Use descriptive test names: `test_function_name_condition`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_agent_success() {
        // Test successful agent creation
    }

    #[tokio::test]
    async fn test_create_agent_invalid_config() {
        // Test error handling for invalid config
    }
}
```

#### Integration Tests
- Place in `tests/` directory
- Test complete workflows
- Use realistic test data

#### Performance Tests
- Use `criterion` for benchmarking
- Place benchmarks in `benches/` directory
- Test critical performance paths

### Code Quality Tools

#### Automatic Checks
```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check compilation
cargo check

# Run tests
cargo test

# Generate documentation
cargo doc --open
```

#### Pre-commit Hooks (Recommended)
Consider setting up pre-commit hooks to run quality checks automatically:

```bash
#!/bin/sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Development Workflow

### 1. Choose an Issue
- Check [GitHub Issues](../../issues) for open tasks
- Look for issues labeled `good first issue` or `help wanted`
- Comment on the issue to indicate you're working on it

### 2. Create a Branch
```bash
# Create and switch to feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/issue-number-description
```

### 3. Make Changes
- Write tests first (TDD approach recommended)
- Implement the feature/fix
- Ensure all tests pass
- Update documentation if needed

### 4. Commit Changes
```bash
# Stage your changes
git add .

# Commit with conventional format
git commit -m "feat(agents): add capability-based task matching

- Implement capability matching algorithm
- Add unit tests for matching logic
- Update agent documentation"
```

### 5. Push and Create Pull Request
```bash
# Push your branch
git push origin feature/your-feature-name

# Create pull request on GitHub
# - Use descriptive title
# - Fill out PR template
# - Reference related issues
```

## Pull Request Process

### PR Requirements
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt --check`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated if needed
- [ ] PR description follows template
- [ ] Related issues referenced

### PR Review Process
1. **Automated Checks**: CI runs tests, linting, and formatting checks
2. **Peer Review**: At least one maintainer reviews the code
3. **Discussion**: Address review comments and make requested changes
4. **Approval**: PR approved and merged by maintainer

### PR Template
```markdown
## Description
Brief description of the changes made.

## Type of Change
- [ ] Bug fix (non-breaking change)
- [ ] New feature (non-breaking change)
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Documentation updated
- [ ] Tests pass locally
- [ ] No breaking changes
```

## Architecture Guidelines

### Modular Design
The codebase follows a modular architecture. When adding new features:

1. **Identify the appropriate module** (agents, tasks, neural, etc.)
2. **Keep modules focused** on single responsibilities
3. **Use clear interfaces** between modules
4. **Document module boundaries** and dependencies

### Async Programming
- Use `tokio` for async runtime
- Prefer `async fn` for async functions
- Use appropriate synchronization primitives:
  - `Arc<Mutex<T>>` for shared mutable state
  - `RwLock` for read-heavy concurrent access
  - Channels for message passing

### Error Handling
- Use `anyhow::Result<T>` for operations that may fail
- Implement `std::error::Error` for custom error types
- Provide meaningful error messages
- Use `thiserror` for error type definitions

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HiveError {
    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: String },

    #[error("Task creation failed: {reason}")]
    TaskCreationFailed { reason: String },
}
```

## Performance Considerations

### Memory Management
- Prefer `&str` over `String` when possible
- Use object pooling for frequently created objects
- Implement proper cleanup in async operations

### CPU Optimization
- Avoid unnecessary allocations in hot paths
- Use parallel processing when appropriate
- Profile performance-critical code

### Database Optimization
- Use prepared statements
- Implement connection pooling
- Add appropriate indexes
- Batch operations when possible

## Security Guidelines

### Input Validation
- Validate all user inputs
- Use type-safe parsing
- Implement rate limiting
- Sanitize data before processing

### Authentication & Authorization
- Use secure token generation
- Implement proper session management
- Validate permissions for operations
- Log security-relevant events

### Dependency Management
- Keep dependencies updated
- Review security advisories regularly
- Use `cargo audit` for vulnerability checking
- Pin dependency versions for production

## Getting Help

### Communication Channels
- **GitHub Issues**: For bug reports and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Documentation**: Check docs/ directory for detailed guides

### Asking for Help
When asking for help:
1. Provide context about what you're trying to accomplish
2. Include error messages and stack traces
3. Share relevant code snippets
4. Describe your development environment

### Finding Your Way Around
- **README.md**: Project overview and quick start
- **docs/**: Detailed documentation for all components
- **src/**: Source code organized by modules
- **tests/**: Test files and test utilities

## Recognition

Contributors are recognized in several ways:
- GitHub contributor statistics
- Mention in release notes for significant contributions
- Attribution in documentation for major features

Thank you for contributing to the AI Orchestrator Hub Backend! Your contributions help make this project better for everyone.