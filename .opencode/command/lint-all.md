---
description: Comprehensive linting across Rust, TypeScript, and configuration files
agent: formatting-agent
---

# Lint All Command

Perform comprehensive linting across all languages and file types in the AI Orchestrator Hub project, including Rust, TypeScript, configuration files, and documentation.

## Comprehensive Linting Strategy

### 1. Environment Setup
Prepare linting environment for all languages:

```bash
# Ensure all linting tools are available
npm run lint:check-tools -- --all

# Create linting reports directory
mkdir -p lint-reports/$(date +%Y%m%d_%H%M%S)

# Set linting configuration
export LINT_STRICT=true
export LINT_FIX=false
export LINT_REPORT_FORMAT=json
```

### 2. Rust Linting
Comprehensive Rust code linting:

```bash
# Run Clippy with all targets and features
cargo clippy --all-targets --all-features -- -D warnings > lint-reports/rust-clippy.txt

# Check for unwrap usage (security critical)
cargo clippy --all-targets --all-features -- -W clippy::unwrap_used -W clippy::expect_used > lint-reports/rust-unwrap-check.txt

# Format check
cargo fmt --all --check > lint-reports/rust-format-check.txt

# Additional Rust linting
cargo clippy --all-targets -- -W clippy::pedantic -W clippy::nursery > lint-reports/rust-pedantic.txt
```

### 3. TypeScript/JavaScript Linting
Frontend code quality analysis:

```bash
# Run ESLint with all rules
npm run lint -- --format json > lint-reports/frontend-eslint.json

# TypeScript strict checking
npm run type-check -- --strict --noEmit > lint-reports/typescript-strict.json

# Prettier format checking
npm run format:check -- --write false > lint-reports/prettier-check.json

# Additional frontend linting
npm run lint:css -- --format json > lint-reports/css-lint.json
```

### 4. Configuration File Linting
Validate configuration files:

```bash
# YAML configuration validation
npm run lint:yaml -- --files "**/*.yml,**/*.yaml" > lint-reports/yaml-validation.json

# JSON configuration validation
npm run lint:json -- --files "**/*.json" --exclude "node_modules/**" > lint-reports/json-validation.json

# TOML configuration validation
npm run lint:toml -- --files "**/*.toml" > lint-reports/toml-validation.json

# Docker file linting
npm run lint:docker -- --files "**/Dockerfile*" > lint-reports/docker-lint.json
```

### 5. Documentation Linting
Documentation quality checks:

```bash
# Markdown linting
npm run lint:markdown -- --files "**/*.md" --exclude "node_modules/**" > lint-reports/markdown-lint.json

# Documentation link checking
npm run lint:docs:links -- --files "**/*.md" > lint-reports/docs-links.json

# Documentation consistency
npm run lint:docs:consistency -- --files "**/*.md" > lint-reports/docs-consistency.json
```

### 6. Security Linting
Security-focused code analysis:

```bash
# Security linting for Rust
cargo clippy --all-targets -- -W clippy::suspicious -W clippy::complexity > lint-reports/rust-security-lint.txt

# Security linting for TypeScript
npm run lint:security -- --format json > lint-reports/frontend-security-lint.json

# Secrets detection
npm run lint:secrets -- --scan . --exclude "node_modules/**,.git/**" > lint-reports/secrets-detection.json
```

## Linting Categories

### Code Quality Linting
Ensure code quality standards:

```bash
# Complexity analysis
npm run lint:complexity -- --language rust --files "**/*.rs" > lint-reports/rust-complexity.json
npm run lint:complexity -- --language typescript --files "**/*.ts,**/*.tsx" > lint-reports/ts-complexity.json

# Code duplication detection
npm run lint:duplication -- --language rust --files "**/*.rs" > lint-reports/rust-duplication.json
npm run lint:duplication -- --language typescript --files "**/*.ts,**/*.tsx" > lint-reports/ts-duplication.json

# Dead code detection
npm run lint:dead-code -- --language rust --files "**/*.rs" > lint-reports/rust-dead-code.json
npm run lint:dead-code -- --language typescript --files "**/*.ts,**/*.tsx" > lint-reports/ts-dead-code.json
```

### Performance Linting
Performance-related code analysis:

```bash
# Performance anti-patterns
npm run lint:performance -- --language rust --files "**/*.rs" > lint-reports/rust-performance-lint.json
npm run lint:performance -- --language typescript --files "**/*.ts,**/*.tsx" > lint-reports/ts-performance-lint.json

# Memory usage analysis
npm run lint:memory -- --language rust --files "**/*.rs" > lint-reports/rust-memory-lint.json

# Async code analysis
npm run lint:async -- --language rust --files "**/*.rs" > lint-reports/rust-async-lint.json
```

### Maintainability Linting
Code maintainability assessment:

```bash
# Code maintainability index
npm run lint:maintainability -- --language rust --files "**/*.rs" > lint-reports/rust-maintainability.json
npm run lint:maintainability -- --language typescript --files "**/*.ts,**/*.tsx" > lint-reports/ts-maintainability.json

# Code readability analysis
npm run lint:readability -- --language rust --files "**/*.rs" > lint-reports/rust-readability.json
npm run lint:readability -- --language typescript --files "**/*.ts,**/*.tsx" > lint-reports/ts-readability.json
```

## Automated Linting Fixes

### Auto-fix Capabilities
Automatically fix common linting issues:

```bash
# Auto-fix Rust formatting
cargo fmt --all

# Auto-fix TypeScript/ESLint issues
npm run lint:fix

# Auto-fix Prettier formatting
npm run format:fix

# Auto-fix common configuration issues
npm run lint:config:fix -- --files "**/*.json,**/*.yml,**/*.yaml"
```

### Safe Auto-fixing
Apply fixes with safety checks:

```bash
# Safe auto-fix with backup
npm run lint:safe-fix -- --backup --files "**/*.rs,**/*.ts,**/*.tsx"

# Gradual auto-fixing
npm run lint:gradual-fix -- --phase 1 --files "**/*.rs"
npm run lint:gradual-fix -- --phase 2 --files "**/*.ts,**/*.tsx"

# Review auto-fixes before applying
npm run lint:review-fixes -- --interactive
```

## Linting Reports and Analytics

### Comprehensive Reports
Generate detailed linting reports:

```bash
# Executive summary
npm run lint:report:summary -- --input lint-reports/ --output lint-reports/executive-summary.pdf

# Technical report
npm run lint:report:technical -- --input lint-reports/ --output lint-reports/technical-report.pdf

# Trend analysis
npm run lint:report:trends -- --history 30d --output lint-reports/linting-trends.pdf
```

### Linting Dashboard
Interactive linting visualization:

```bash
# Linting dashboard
npm run lint:dashboard -- --serve --port 3010

# Category breakdown
npm run lint:dashboard:categories -- --generate --output lint-reports/category-breakdown.html

# Severity analysis
npm run lint:dashboard:severity -- --generate --output lint-reports/severity-analysis.html
```

## Linting Rules Configuration

### Custom Rules
Configure project-specific linting rules:

```bash
# Rust custom rules
cat > .cargo/custom-lints.toml << EOF
[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
missing_docs = "warn"
pedantic = "warn"
EOF

# TypeScript custom rules
cat > .eslintrc.custom.js << EOF
module.exports = {
  rules: {
    'no-console': 'warn',
    'prefer-const': 'error',
    'no-unused-vars': 'error'
  }
}
EOF
```

### Rules Management
Manage linting rules effectively:

```bash
# Validate rule configuration
npm run lint:rules:validate -- --config .eslintrc.js

# Update rules to latest standards
npm run lint:rules:update -- --latest

# Audit rule effectiveness
npm run lint:rules:audit -- --effectiveness
```

## CI/CD Integration

### Pre-commit Hooks
Integrate linting into development workflow:

```bash
# Install pre-commit hooks
npm run lint:hooks:install -- --pre-commit

# Pre-commit linting validation
npm run lint:pre-commit -- --validate

# Selective pre-commit linting
npm run lint:pre-commit -- --files "src/**/*.rs"
```

### CI Pipeline Integration
Integrate linting into CI/CD:

```bash
# CI linting with quality gates
npm run lint:ci -- --quality-gate --block-on-errors

# Parallel linting in CI
npm run lint:ci:parallel -- --workers 4

# Incremental linting
npm run lint:ci:incremental -- --baseline main
```

## Linting Best Practices

### Rule Prioritization
Prioritize linting rules effectively:

```bash
# Critical rules (block CI)
npm run lint:rules:prioritize -- --level critical --action block

# Warning rules (allow but track)
npm run lint:rules:prioritize -- --level warning --action track

# Info rules (optional)
npm run lint:rules:prioritize -- --level info --action optional
```

### Team Standards
Establish team linting standards:

```bash
# Generate team standards document
npm run lint:standards:generate -- --output lint-reports/team-standards.md

# Validate adherence to standards
npm run lint:standards:validate -- --team-standards

# Update standards based on feedback
npm run lint:standards:update -- --feedback lint-reports/standards-feedback.json
```

## Common Linting Issues

### Rust Linting Issues
- **Unwrap Usage**: Replace with proper error handling
- **Missing Documentation**: Add comprehensive documentation
- **Complex Functions**: Break down into smaller functions
- **Performance Issues**: Optimize for better performance

### TypeScript Linting Issues
- **Type Safety**: Add proper type annotations
- **Unused Variables**: Remove or properly use variables
- **Console Usage**: Replace with proper logging
- **Async Handling**: Proper async/await usage

### Configuration Issues
- **Invalid YAML/JSON**: Fix syntax errors
- **Inconsistent Formatting**: Apply consistent formatting
- **Deprecated Settings**: Update to current standards
- **Security Misconfigurations**: Fix security settings

## Linting Metrics

### Quality Metrics
Track linting effectiveness:

- **Linting Coverage**: Percentage of code covered by linting
- **Error Rate**: Number of linting errors per 1000 lines
- **Fix Rate**: Percentage of auto-fixable issues
- **Compliance Rate**: Adherence to linting standards

### Process Metrics
Track linting process efficiency:

- **Execution Time**: Average linting execution time
- **False Positives**: Percentage of incorrect linting warnings
- **Developer Satisfaction**: Developer feedback on linting
- **CI Impact**: Impact on CI/CD pipeline performance

### Improvement Metrics
Track linting improvements over time:

- **Error Reduction**: Reduction in linting errors over time
- **Standards Compliance**: Improvement in standards adherence
- **Team Adoption**: Increase in team linting adoption
- **Automation Rate**: Percentage of linting tasks automated

This comprehensive linting approach ensures consistent code quality across all languages and file types, with automated fixing capabilities and detailed reporting for continuous improvement.