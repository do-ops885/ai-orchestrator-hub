---
description: Comprehensive quality assurance checks across the entire codebase
agent: quality-assurance
---

# Quality Check Command

Perform comprehensive quality assurance checks across the entire AI Orchestrator Hub codebase, including code quality, testing coverage, performance metrics, and compliance validation.

## Quality Assurance Strategy

### 1. Environment Setup
Prepare quality assessment environment:

```bash
# Ensure clean working directory
git status --porcelain

# Create quality assessment directory
mkdir -p quality-check/$(date +%Y%m%d_%H%M%S)

# Set quality thresholds
export QUALITY_THRESHOLD_A=90
export QUALITY_THRESHOLD_B=80
export QUALITY_THRESHOLD_C=70
```

### 2. Code Quality Analysis
Analyze code quality across all components:

```bash
# Rust code quality analysis
cargo clippy --all-targets --all-features -- -D warnings > quality-check/clippy-report.txt

# Code complexity analysis
npm run quality:complexity -- --language rust --output quality-check/complexity-rust.json

# Code duplication detection
npm run quality:duplication -- --scan . --output quality-check/duplication-report.json
```

### 3. Testing Coverage Analysis
Comprehensive testing coverage assessment:

```bash
# Backend test coverage
cargo tarpaulin --all-features --out Json > quality-check/coverage-backend.json

# Frontend test coverage
npm run test:coverage -- --json > quality-check/coverage-frontend.json

# Integration test coverage
npm run test:integration:coverage -- --output quality-check/coverage-integration.json

# End-to-end test coverage
npm run test:e2e:coverage -- --output quality-check/coverage-e2e.json
```

### 4. Performance Quality Metrics
Assess performance quality indicators:

```bash
# Code performance analysis
npm run quality:performance:code -- --analyze --output quality-check/performance-code.json

# Bundle size analysis
npm run quality:performance:bundle -- --analyze --output quality-check/bundle-analysis.json

# Memory usage analysis
npm run quality:performance:memory -- --profile --output quality-check/memory-profile.json
```

### 5. Documentation Quality
Evaluate documentation completeness and quality:

```bash
# Documentation coverage
npm run quality:docs:coverage -- --scan . --output quality-check/docs-coverage.json

# API documentation validation
npm run quality:docs:api -- --validate --output quality-check/api-docs-validation.json

# Code documentation analysis
npm run quality:docs:code -- --analyze --output quality-check/code-docs-analysis.json
```

## Quality Metrics Categories

### Code Quality Metrics
Comprehensive code quality assessment:

```bash
# Maintainability index
npm run quality:metrics:maintainability -- --calculate --output quality-check/maintainability-index.json

# Technical debt analysis
npm run quality:metrics:debt -- --analyze --output quality-check/technical-debt.json

# Code smell detection
npm run quality:metrics:smells -- --detect --output quality-check/code-smells.json
```

### Testing Quality Metrics
Evaluate testing effectiveness:

```bash
# Test quality assessment
npm run quality:test:effectiveness -- --analyze --output quality-check/test-effectiveness.json

# Test case coverage analysis
npm run quality:test:coverage-depth -- --analyze --output quality-check/test-coverage-depth.json

# Test reliability metrics
npm run quality:test:reliability -- --measure --output quality-check/test-reliability.json
```

### Performance Quality Metrics
Performance quality indicators:

```bash
# Performance benchmark analysis
npm run quality:performance:benchmarks -- --analyze --output quality-check/performance-benchmarks.json

# Resource efficiency metrics
npm run quality:performance:efficiency -- --measure --output quality-check/resource-efficiency.json

# Scalability assessment
npm run quality:performance:scalability -- --test --output quality-check/scalability-assessment.json
```

## Automated Quality Gates

### Quality Gate Configuration
Set up automated quality gates:

```bash
# Configure quality thresholds
npm run quality:gates:config -- --thresholds quality-thresholds.json

# Code quality gates
npm run quality:gates:code -- --enforce --block-on-failure

# Test coverage gates
npm run quality:gates:coverage -- --minimum 80% --enforce

# Performance gates
npm run quality:gates:performance -- --baseline --enforce
```

### Quality Gate Validation
Validate against quality standards:

```bash
# Validate all quality gates
npm run quality:gates:validate -- --all

# Generate quality gate report
npm run quality:gates:report -- --output quality-check/quality-gates-report.json

# Quality gate status dashboard
npm run quality:gates:dashboard -- --serve --port 3004
```

## Code Review Automation

### Automated Code Review
Perform automated code review analysis:

```bash
# Code review analysis
npm run quality:review:auto -- --analyze --output quality-check/auto-review-report.json

# Best practices validation
npm run quality:review:best-practices -- --validate --output quality-check/best-practices-validation.json

# Security code review
npm run quality:review:security -- --analyze --output quality-check/security-review.json
```

### Code Review Metrics
Measure code review effectiveness:

```bash
# Review coverage analysis
npm run quality:review:coverage -- --analyze --output quality-check/review-coverage.json

# Review quality assessment
npm run quality:review:quality -- --measure --output quality-check/review-quality-metrics.json

# Review feedback analysis
npm run quality:review:feedback -- --analyze --output quality-check/review-feedback-analysis.json
```

## Quality Reporting

### Comprehensive Quality Report
Generate detailed quality assessment reports:

```bash
# Executive quality summary
npm run quality:report:executive -- --generate --output quality-check/executive-summary.pdf

# Technical quality report
npm run quality:report:technical -- --detailed --output quality-check/technical-report.pdf

# Quality trends analysis
npm run quality:report:trends -- --analyze --output quality-check/quality-trends.pdf
```

### Quality Dashboard
Interactive quality visualization:

```bash
# Quality metrics dashboard
npm run quality:dashboard -- --serve --port 3005

# Quality trend visualization
npm run quality:dashboard:trends -- --generate --output quality-check/quality-trends-dashboard.html

# Quality comparison reports
npm run quality:dashboard:compare -- --baseline --output quality-check/quality-comparison.html
```

## Quality Improvement Recommendations

### Automated Recommendations
Generate quality improvement suggestions:

```bash
# Code quality recommendations
npm run quality:recommend:code -- --generate --output quality-check/code-improvements.md

# Testing recommendations
npm run quality:recommend:testing -- --generate --output quality-check/testing-improvements.md

# Performance recommendations
npm run quality:recommend:performance -- --generate --output quality-check/performance-improvements.md
```

### Quality Action Plan
Create actionable improvement plans:

```bash
# Quality improvement roadmap
npm run quality:plan:roadmap -- --generate --output quality-check/quality-roadmap.md

# Prioritized action items
npm run quality:plan:actions -- --prioritize --output quality-check/quality-action-items.md

# Quality improvement timeline
npm run quality:plan:timeline -- --create --output quality-check/quality-timeline.md
```

## Integration with Development Workflow

### CI/CD Quality Integration
Integrate quality checks into CI/CD pipeline:

```bash
# Pre-commit quality hooks
npm run quality:hooks:pre-commit -- --install

# CI quality gates
npm run quality:ci:gates -- --configure --pipeline github-actions

# Quality check automation
npm run quality:ci:automate -- --setup
```

### Quality as Code
Define quality standards as code:

```bash
# Quality standards definition
npm run quality:standards:define -- --output quality-check/quality-standards.yaml

# Quality rules configuration
npm run quality:rules:config -- --generate --output quality-check/quality-rules.json

# Quality policy enforcement
npm run quality:policy:enforce -- --automate
```

## Quality Training and Awareness

### Quality Training Recommendations
Automated quality training suggestions:

```bash
# Quality training needs assessment
npm run quality:training:needs -- --assess --output quality-check/training-needs.md

# Quality best practices guide
npm run quality:training:guide -- --generate --output quality-check/quality-guide.md

# Quality improvement workshops
npm run quality:training:workshops -- --plan --output quality-check/training-workshops.md
```

## Best Practices

1. **Continuous Quality**: Quality checks should be continuous, not just at release time
2. **Automated Gates**: Use automated quality gates to prevent quality regressions
3. **Measurable Metrics**: Define clear, measurable quality metrics
4. **Progressive Improvement**: Focus on continuous quality improvement
5. **Team Accountability**: Make quality everyone's responsibility
6. **Documentation**: Document quality standards and procedures
7. **Tool Integration**: Integrate quality tools into development workflow

## Common Quality Issues

- **Code Complexity**: High cyclomatic complexity, large functions
- **Test Coverage Gaps**: Missing test coverage for critical paths
- **Technical Debt**: Accumulated code quality issues
- **Performance Regressions**: Performance degradation over time
- **Documentation Gaps**: Missing or outdated documentation
- **Security Vulnerabilities**: Code quality issues leading to security problems
- **Maintainability Issues**: Code that is hard to understand and modify
- **Scalability Concerns**: Code that doesn't scale well

## Quality Frameworks

### Code Quality Standards
Adhere to industry code quality standards:

- **Clean Code**: Readable, maintainable, and well-structured code
- **SOLID Principles**: Single responsibility, open-closed, etc.
- **DRY Principle**: Don't repeat yourself
- **KISS Principle**: Keep it simple and straightforward
- **YAGNI Principle**: You aren't gonna need it

### Testing Standards
Comprehensive testing standards:

- **Unit Testing**: Test individual components in isolation
- **Integration Testing**: Test component interactions
- **System Testing**: Test the entire system
- **Acceptance Testing**: Test from user perspective
- **Performance Testing**: Test system performance under load
- **Security Testing**: Test system security

### Documentation Standards
Documentation quality standards:

- **API Documentation**: Complete and accurate API documentation
- **Code Documentation**: Inline code documentation
- **User Documentation**: User guides and tutorials
- **Architecture Documentation**: System architecture documentation
- **Deployment Documentation**: Deployment and operations guides