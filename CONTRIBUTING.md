# Contributing to AI Orchestrator Hub

Thank you for your interest in contributing to the AI Orchestrator Hub! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contribution Guidelines](#contribution-guidelines)
- [Code Standards](#code-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Security Report Organization](#security-report-organization)
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
cargo test --test api_integration_tests  # Run API integration tests
cargo test --test chaos_engineering_tests  # Run chaos engineering tests
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

### Running Tests

```bash
# Backend tests
cd backend
cargo test --all-features          # All unit tests
cargo test --test api_integration_tests # API integration tests
cargo test --test chaos_engineering_tests # Chaos engineering tests
cargo test --test performance_regression_tests # Performance tests

# Frontend tests
cd frontend
npm test                           # Unit tests
npm run test:e2e                   # End-to-end tests
```

### Feature Development Setup

```bash
# Enable all features for comprehensive testing
cd backend
cargo run --all-features

# Run specific feature combinations
cargo run --features advanced-neural,gpu-acceleration

# Test with different configurations
cargo test --features advanced-neural
cargo test --test performance_regression_tests
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
cargo test --test api_integration_tests
cargo test --test chaos_engineering_tests
cargo test --test performance_regression_tests

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
cargo test --test api_integration_tests # API integration tests
cargo test --test chaos_engineering_tests # Chaos engineering tests
cargo test --test performance_regression_tests # Performance tests
cargo clippy --all-features        # Linting
cargo fmt --all --check            # Format check

# Frontend comprehensive testing
cd frontend
npm test                           # Unit tests
npm run test:e2e                   # End-to-end tests
npm run lint:check                 # Linting
npm run build                      # Build verification

# Full system testing
cd backend && cargo test --all-features
cd ../frontend && npm test && npm run build
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
    cargo test --test api_integration_tests
    cargo test --test chaos_engineering_tests
    cargo test --test performance_regression_tests
    cargo clippy --all-features
    cargo fmt --all --check

    # Frontend tests
    cd frontend
    npm test
    npm run test:e2e
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
- **API Testing**: Use the API integration tests for endpoint validation
- **Performance**: Run performance regression tests for optimization

### Architecture Guidelines

- Follow the existing module structure
- Use dependency injection patterns
- Implement proper error handling
- Consider backward compatibility
- Design for extensibility

## Security Report Organization

### Security Reports Folder Structure

All security-related reports, scans, and audit outputs **must** be placed in the `security-reports/` folder at the project root. This is the **standard and only accepted location** for security reports to ensure:

- **Consistency**: All security reports are in one predictable location
- **Automation**: CI/CD pipelines and scripts know where to find and process reports
- **Security**: Prevents accidental exposure of sensitive security information
- **Maintenance**: Easier to manage, backup, and clean up security reports

### Required Security Report Locations

| Report Type | Required Location | File Naming Convention |
|-------------|-------------------|----------------------|
| Cargo Audit | `security-reports/cargo-audit-*.json` | `cargo-audit-YYYYMMDD-HHMMSS.json` |
| NPM Audit | `security-reports/npm-audit-*.json` | `npm-audit-YYYYMMDD-HHMMSS.json` |
| Secrets Scan | `security-reports/secrets-scan-*.txt` | `secrets-scan-YYYYMMDD-HHMMSS.txt` |
| CodeQL Results | `security-reports/codeql-*.sarif` | `codeql-YYYYMMDD-HHMMSS.sarif` |
| Container Scan | `security-reports/container-scan-*.sarif` | `container-scan-YYYYMMDD-HHMMSS.sarif` |
| Dependency Review | `security-reports/dependency-review-*.json` | `dependency-review-YYYYMMDD-HHMMSS.json` |
| Security Metrics | `security-reports/security-metrics-*.json` | `security-metrics-YYYYMMDD-HHMMSS.json` |

### Security Report Generation Guidelines

When creating scripts or tools that generate security reports:

1. **Always use the `security-reports/` folder** as the output directory
2. **Use timestamped filenames** to prevent overwrites and maintain history
3. **Include file extensions** appropriate to the report format (.json, .txt, .sarif, etc.)
4. **Document output locations** in script comments and documentation
5. **Use absolute paths** when possible to avoid path resolution issues

### Example Script Template

```bash
#!/bin/bash
# Security Report Generation Template

# Define security reports directory
SECURITY_REPORTS_DIR="security-reports"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Create directory if it doesn't exist
mkdir -p "$SECURITY_REPORTS_DIR"

# Generate security report with proper naming
REPORT_FILE="$SECURITY_REPORTS_DIR/cargo-audit-$TIMESTAMP.json"

# Run security scan
cargo audit --format json --output "$REPORT_FILE"

echo "Security report generated: $REPORT_FILE"
```

### Validation and Enforcement

- **Pre-commit hooks** will check for misplaced security files
- **CI/CD pipelines** validate that security reports are in the correct location
- **Automated scripts** ensure proper directory structure
- **Pull request checks** will fail if security reports are in wrong locations

### Migration of Existing Reports

If you find security reports in incorrect locations:

1. Move them to `security-reports/` folder
2. Rename with proper timestamp format if needed
3. Update any scripts that reference the old locations
4. Test that automated processes still work correctly

### Security Report Retention

- **Local retention**: Keep reports for active development cycles
- **CI/CD retention**: Configure artifact retention policies appropriately
- **Cleanup**: Use automated cleanup scripts for old reports
- **Archival**: Move important reports to long-term storage when needed

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

Thank you for contributing to the AI Orchestrator Hub!
