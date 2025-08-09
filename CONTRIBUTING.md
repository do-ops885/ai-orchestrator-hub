# Contributing to Multiagent Hive System

Thank you for your interest in contributing to the Multiagent Hive System! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contribution Guidelines](#contribution-guidelines)
- [Code Standards](#code-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please be respectful and professional in all interactions.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/d-oit/multiagent-hive.git
   cd multiagent-hive
   ```
3. **Set up the development environment** (see below)
4. **Create a feature branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Rust 1.70+ with Cargo
- Node.js 18+ with npm
- Git

### Backend Setup

```bash
cd backend
cargo build
cargo test
```

### Frontend Setup

```bash
cd frontend
npm install
npm run dev
```

### Running the Full System

```bash
# Terminal 1: Backend
cd backend
cargo run

# Terminal 2: Frontend
cd frontend
npm run dev
```

## Contribution Guidelines

### Types of Contributions

We welcome the following types of contributions:

- **Bug fixes**: Fix issues in existing functionality
- **Feature additions**: Add new capabilities to the system
- **Documentation improvements**: Enhance or clarify documentation
- **Performance optimizations**: Improve system efficiency
- **Test coverage**: Add or improve test cases
- **Code quality**: Refactoring and cleanup

### Before You Start

1. **Check existing issues** to see if your idea is already being worked on
2. **Open an issue** to discuss major changes before implementing
3. **Review the codebase** to understand the architecture and patterns
4. **Read the documentation** to understand the system design

## Code Standards

### Rust Code Standards

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
- Use `cargo fmt` for formatting
- Pass all `cargo clippy` lints (see `clippy.toml` for configuration)
- Write comprehensive documentation with `///` for public APIs
- Use `anyhow::Result` for error handling
- Avoid `unwrap()` and `panic!()` in production code

### TypeScript Code Standards

- Follow the ESLint configuration in `eslint.config.js`
- Use TypeScript strict mode
- Write JSDoc comments for complex functions
- Use consistent naming conventions
- Prefer functional components with hooks

### Documentation Standards

- Use clear, concise language
- Include code examples for complex features
- Update README.md for new features
- Write inline documentation for public APIs
- Include usage examples in docstrings

## Testing

### Backend Testing

```bash
# Run all tests
cargo test

# Run tests with advanced features
cargo test --features advanced-neural

# Run specific test
cargo test test_name

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Frontend Testing

```bash
# Run tests
npm test

# Run tests in watch mode
npm test -- --watch

# Run with coverage
npm test -- --coverage
```

### Test Requirements

- **Unit tests**: All new functions should have unit tests
- **Integration tests**: Test component interactions
- **Example tests**: Ensure examples work correctly
- **Performance tests**: For performance-critical changes

## Documentation

### Required Documentation

1. **API Documentation**: Document all public functions and types
2. **Usage Examples**: Provide clear examples for new features
3. **Architecture Documentation**: Explain design decisions
4. **Configuration Documentation**: Document new configuration options

### Documentation Format

```rust
/// Brief description of the function.
///
/// More detailed explanation if needed. Explain the purpose,
/// behavior, and any important considerations.
///
/// # Arguments
///
/// * `param1` - Description of the first parameter
/// * `param2` - Description of the second parameter
///
/// # Returns
///
/// Description of the return value and its meaning.
///
/// # Errors
///
/// Describe when and why this function might return an error.
///
/// # Examples
///
/// ```rust
/// use crate::example::function_name;
///
/// let result = function_name("example", 42)?;
/// assert_eq!(result, expected_value);
/// ```
pub fn function_name(param1: &str, param2: i32) -> Result<ReturnType> {
    // Implementation
}
```

## Pull Request Process

### Before Submitting

1. **Run the full test suite**:
   ```bash
   cargo test --all-features
   cd frontend && npm test
   ```

2. **Run linting**:
   ```bash
   cargo clippy --all-features
   cd frontend && npm run lint
   ```

3. **Update documentation** for any new features or changes

4. **Add tests** for new functionality

### Pull Request Template

When creating a pull request, please include:

- **Description**: Clear description of what the PR does
- **Motivation**: Why this change is needed
- **Testing**: How you tested the changes
- **Documentation**: What documentation was updated
- **Breaking Changes**: Any breaking changes and migration notes

### Review Process

1. **Automated checks**: All CI checks must pass
2. **Code review**: At least one maintainer must approve
3. **Testing**: Reviewers may test the changes locally
4. **Documentation review**: Ensure documentation is complete and accurate

## Issue Reporting

### Bug Reports

Please include:

- **Environment**: OS, Rust version, Node.js version
- **Steps to reproduce**: Clear steps to reproduce the issue
- **Expected behavior**: What you expected to happen
- **Actual behavior**: What actually happened
- **Logs**: Relevant error messages or logs
- **Configuration**: Any relevant configuration details

### Feature Requests

Please include:

- **Use case**: Why this feature would be useful
- **Proposed solution**: How you think it should work
- **Alternatives**: Other solutions you've considered
- **Implementation notes**: Any technical considerations

### Issue Labels

We use the following labels to categorize issues:

- `bug`: Something isn't working
- `enhancement`: New feature or request
- `documentation`: Improvements or additions to documentation
- `good first issue`: Good for newcomers
- `help wanted`: Extra attention is needed
- `performance`: Performance-related issues
- `security`: Security-related issues

## Development Tips

### Debugging

- Use `tracing::debug!()` for debug logging in Rust
- Use browser dev tools for frontend debugging
- Enable verbose logging with `RUST_LOG=debug`

### Performance Testing

- Use the built-in metrics system to monitor performance
- Test with various agent counts and task loads
- Profile with `cargo flamegraph` for performance bottlenecks

### Architecture Guidelines

- Follow the existing module structure
- Use dependency injection patterns
- Implement proper error handling
- Consider backward compatibility
- Design for extensibility

## Getting Help

If you need help with your contribution:

1. **Check the documentation** in the `docs/` directory
2. **Review existing code** for similar patterns
3. **Ask questions** in GitHub issues or discussions
4. **Join our community** for real-time help

## Recognition

Contributors will be recognized in:

- The project's README.md
- Release notes for significant contributions
- The project's contributors page

Thank you for contributing to the Multiagent Hive System!