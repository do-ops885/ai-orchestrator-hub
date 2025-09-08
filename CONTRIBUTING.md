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
    git clone https://github.com/your-username/ai-orchestrator-hub.git
    cd ai-orchestrator-hub
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
cargo build                    # Build with default features
cargo test                     # Run basic tests
cargo test --all-features      # Test all feature combinations
cargo clippy --all-features    # Comprehensive linting
cargo fmt --all                # Format code
```

### Frontend Setup

```bash
cd frontend
npm install                    # Install dependencies
npm run dev                    # Start development server
npm test                       # Run tests
npm run lint                   # ESLint checks
npm run build                  # Production build
```

### Running the Full System

```bash
# Terminal 1: Backend (basic features)
cd backend
cargo run

# Terminal 2: Backend (advanced features)
cd backend
cargo run --features advanced-neural

# Terminal 3: MCP Server (optional)
cd backend
cargo run --bin mcp_server

# Terminal 4: Frontend
cd frontend
npm run dev
```

### Feature Development Setup

```bash
# Enable all features for comprehensive testing
cd backend
cargo run --all-features

# Run specific feature combinations
cargo run --features advanced-neural,gpu-acceleration
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
- Use `cargo fmt` for formatting (configured in `rustfmt.toml`)
- Pass all `cargo clippy` lints (strict configuration in `clippy.toml`)
- Write comprehensive documentation with `///` for public APIs
- Use `anyhow::Result` for error handling with proper error context
- Avoid `unwrap()`, `expect()`, and `panic!()` in production code
- Use structured logging with the `tracing` crate
- Implement proper async error handling with `ResultExt`
- Follow the module structure in `backend/src/`
- Use dependency injection patterns for testability

### TypeScript Code Standards

- Follow the ESLint flat configuration in `eslint.config.js`
- Use TypeScript strict mode with no implicit any
- Write JSDoc comments for complex functions and components
- Use consistent naming: camelCase for variables/functions, PascalCase for components/types
- Prefer functional components with hooks over class components
- Use proper TypeScript generics and utility types
- Implement proper error boundaries for React components
- Follow the component structure in `frontend/src/components/`
- Use Zustand for state management with proper typing

### Documentation Standards

- Use clear, concise language
- Include code examples for complex features
- Update README.md for new features
- Write inline documentation for public APIs
- Include usage examples in docstrings

## Testing

### Backend Testing

```bash
# Basic test suite
cargo test

# Test with advanced neural features
cargo test --features advanced-neural

# Test all feature combinations
cargo test --all-features

# Run specific test
cargo test test_name

# Run integration tests
cargo test --test integration_tests

# Run examples as tests
cargo test --examples

# Generate coverage report (requires cargo-tarpaulin)
cargo tarpaulin --out Html --all-features
```

### Frontend Testing

```bash
# Run unit tests
npm test

# Run tests in watch mode
npm test -- --watch

# Run with coverage
npm test -- --coverage

# Run e2e tests (if configured)
npm run test:e2e
```

### Comprehensive Testing Workflow

```bash
# Backend comprehensive testing
cd backend
cargo test --all-features          # All unit tests
cargo test --test integration_tests # Integration tests
cargo clippy --all-features        # Linting
cargo fmt --all --check            # Format check

# Frontend comprehensive testing
cd frontend
npm test                           # Unit tests
npm run lint:check                 # Linting
npm run build                      # Build verification

# End-to-end testing
# Start backend and frontend, then run e2e tests
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

1. **Run the comprehensive test suite**:
    ```bash
    # Backend tests
    cd backend
    cargo test --all-features
    cargo test --test integration_tests
    cargo clippy --all-features
    cargo fmt --all --check

    # Frontend tests
    cd frontend
    npm test
    npm run lint:check
    npm run build
    ```

2. **Update documentation** for any new features or changes:
    - Update relevant docs in `docs/` directory
    - Update API documentation if endpoints changed
    - Update configuration docs if settings changed
    - Update README.md for new features

3. **Add comprehensive tests** for new functionality:
    - Unit tests for all new functions
    - Integration tests for new features
    - Update existing tests if behavior changed

4. **Security review**:
    - Ensure no secrets or sensitive data in code
    - Validate input sanitization for new endpoints
    - Check for proper authentication/authorization

5. **Performance validation**:
    - Run performance benchmarks if applicable
    - Ensure no performance regressions
    - Test with various configurations

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

- **Backend**: Use structured logging with `tracing::debug!()` and `RUST_LOG=debug`
- **Frontend**: Use browser dev tools and React Developer Tools
- **WebSocket**: Monitor connections with browser Network tab
- **Database**: Check SQLite files in `./data/` directory
- **Health Checks**: Use `/health` endpoint for system diagnostics
- **Metrics**: Access `/metrics` endpoint for performance data

### Performance Testing

- Use the built-in metrics system: `GET /api/hive/metrics`
- Test with various agent counts and task loads
- Profile with `cargo flamegraph` for performance bottlenecks
- Monitor resource usage with the ResourceMonitor component
- Use the performance optimizer for automatic tuning
- Check circuit breaker status for resilience testing

### Feature Development

- **Neural Features**: Test with `--features advanced-neural`
- **MCP Integration**: Run standalone MCP server for testing
- **Persistence**: Check `./data/` directory for database files
- **Security**: Test rate limiting and authentication
- **Monitoring**: Use intelligent alerting for anomaly detection

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
3. **Ask questions** in GitHub discussions
4. **Join our community** for real-time help

## Recognition

Contributors will be recognized in:

- The project's README.md
- Release notes for significant contributions
- The project's contributors page

Thank you for contributing to the Multiagent Hive System!
