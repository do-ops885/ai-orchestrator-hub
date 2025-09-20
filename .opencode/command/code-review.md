---
description: Comprehensive code review using multiple specialized agents
agent: ai-code-analysis-swarm
---

# Code Review Command

Perform comprehensive code review using multiple specialized agents to analyze code quality, security, performance, and best practices across the AI Orchestrator Hub codebase.

## Multi-Agent Review Strategy

### 1. Review Preparation
Set up comprehensive code review environment:

```bash
# Identify files to review
git diff --name-only HEAD~1 > review-files.txt

# Create review directory
mkdir -p code-review/$(date +%Y%m%d_%H%M%S)

# Initialize review context
npm run review:init -- --files review-files.txt --output code-review/context.json
```

### 2. Automated Code Analysis
Leverage multiple agents for comprehensive analysis:

```bash
# AI-powered code analysis
npm run review:ai:analyze -- --files review-files.txt --output code-review/ai-analysis.json

# Technical reviewer analysis
npm run review:technical:analyze -- --files review-files.txt --output code-review/technical-review.json

# Security-focused review
npm run review:security:analyze -- --files review-files.txt --output code-review/security-review.json
```

### 3. Specialized Agent Reviews
Utilize specialized agents for domain-specific analysis:

```bash
# Rust-specific analysis (if applicable)
npm run review:rust:analyze -- --files review-files.txt --output code-review/rust-analysis.json

# React-specific analysis (if applicable)
npm run review:react:analyze -- --files review-files.txt --output code-review/react-analysis.json

# Performance analysis
npm run review:performance:analyze -- --files review-files.txt --output code-review/performance-analysis.json
```

### 4. Quality Assurance Review
Comprehensive quality checks:

```bash
# Code quality metrics
npm run review:quality:metrics -- --files review-files.txt --output code-review/quality-metrics.json

# Best practices validation
npm run review:quality:best-practices -- --files review-files.txt --output code-review/best-practices.json

# Maintainability assessment
npm run review:quality:maintainability -- --files review-files.txt --output code-review/maintainability.json
```

### 5. Documentation Review
Review documentation completeness:

```bash
# Code documentation analysis
npm run review:docs:code -- --files review-files.txt --output code-review/code-docs.json

# API documentation validation
npm run review:docs:api -- --files review-files.txt --output code-review/api-docs.json

# README and guides review
npm run review:docs:guides -- --files review-files.txt --output code-review/docs-guides.json
```

## Review Categories

### Code Quality Review
Analyze code quality aspects:

```bash
# Code complexity analysis
npm run review:complexity:analyze -- --files review-files.txt --output code-review/complexity-analysis.json

# Code duplication detection
npm run review:duplication:detect -- --files review-files.txt --output code-review/duplication-report.json

# Code smell identification
npm run review:smells:identify -- --files review-files.txt --output code-review/code-smells.json
```

### Security Review
Comprehensive security analysis:

```bash
# Vulnerability assessment
npm run review:security:vulnerabilities -- --files review-files.txt --output code-review/vulnerability-assessment.json

# Authentication and authorization review
npm run review:security:auth -- --files review-files.txt --output code-review/auth-review.json

# Data protection analysis
npm run review:security:data -- --files review-files.txt --output code-review/data-protection.json
```

### Performance Review
Performance impact analysis:

```bash
# Performance bottleneck identification
npm run review:performance:bottlenecks -- --files review-files.txt --output code-review/performance-bottlenecks.json

# Memory usage analysis
npm run review:performance:memory -- --files review-files.txt --output code-review/memory-analysis.json

# Scalability assessment
npm run review:performance:scalability -- --files review-files.txt --output code-review/scalability-assessment.json
```

## Automated Review Insights

### AI-Powered Insights
Generate intelligent review insights:

```bash
# Pattern recognition
npm run review:ai:patterns -- --analyze --output code-review/pattern-insights.json

# Anomaly detection
npm run review:ai:anomalies -- --detect --output code-review/anomaly-detection.json

# Predictive analysis
npm run review:ai:predictive -- --forecast --output code-review/predictive-insights.json
```

### Review Recommendations
Generate actionable recommendations:

```bash
# Code improvement suggestions
npm run review:recommend:code -- --generate --output code-review/code-improvements.md

# Security recommendations
npm run review:recommend:security -- --generate --output code-review/security-recommendations.md

# Performance recommendations
npm run review:recommend:performance -- --generate --output code-review/performance-recommendations.md
```

## Review Reporting

### Comprehensive Review Report
Generate detailed review reports:

```bash
# Executive summary
npm run review:report:executive -- --generate --output code-review/executive-summary.pdf

# Technical review report
npm run review:report:technical -- --generate --output code-review/technical-report.pdf

# Security review report
npm run review:report:security -- --generate --output code-review/security-report.pdf
```

### Review Dashboard
Interactive review visualization:

```bash
# Review dashboard
npm run review:dashboard -- --serve --port 3008

# Review trends visualization
npm run review:dashboard:trends -- --generate --output code-review/review-trends.html

# Review comparison reports
npm run review:dashboard:compare -- --baseline --output code-review/review-comparison.html
```

## Review Workflow Integration

### Pre-commit Review
Integrate review into development workflow:

```bash
# Pre-commit review hooks
npm run review:hooks:pre-commit -- --install

# Local review validation
npm run review:validate:local -- --run

# Review requirement checks
npm run review:requirements:check -- --validate
```

### Pull Request Integration
Automate PR review process:

```bash
# PR review automation
npm run review:pr:auto -- --pr-number 123 --output code-review/pr-review-123.json

# Review comment generation
npm run review:pr:comments -- --generate --output code-review/pr-comments.md

# Review approval workflow
npm run review:pr:approve -- --conditions met --output code-review/pr-approval.json
```

## Review Quality Assurance

### Review Accuracy Validation
Validate review quality:

```bash
# Review accuracy assessment
npm run review:validate:accuracy -- --measure --output code-review/review-accuracy.json

# False positive analysis
npm run review:validate:false-positives -- --analyze --output code-review/false-positive-analysis.json

# Review completeness check
npm run review:validate:completeness -- --assess --output code-review/review-completeness.json
```

### Review Improvement
Continuous review improvement:

```bash
# Review feedback analysis
npm run review:improve:feedback -- --analyze --output code-review/review-feedback.json

# Review model training
npm run review:improve:training -- --update --output code-review/model-training.json

# Review accuracy improvement
npm run review:improve:accuracy -- --enhance --output code-review/accuracy-improvement.json
```

## Specialized Reviews

### Architecture Review
Review system architecture changes:

```bash
# Architecture impact analysis
npm run review:architecture:impact -- --analyze --output code-review/architecture-impact.json

# Design pattern validation
npm run review:architecture:patterns -- --validate --output code-review/design-patterns.json

# System integration review
npm run review:architecture:integration -- --assess --output code-review/system-integration.json
```

### Testing Review
Review test quality and coverage:

```bash
# Test coverage analysis
npm run review:testing:coverage -- --analyze --output code-review/test-coverage.json

# Test quality assessment
npm run review:testing:quality -- --evaluate --output code-review/test-quality.json

# Test effectiveness measurement
npm run review:testing:effectiveness -- --measure --output code-review/test-effectiveness.json
```

## Review Best Practices

### Review Guidelines
Follow established review guidelines:

```bash
# Review checklist validation
npm run review:guidelines:checklist -- --validate --output code-review/guidelines-checklist.json

# Review standards compliance
npm run review:guidelines:standards -- --check --output code-review/standards-compliance.json

# Review consistency analysis
npm run review:guidelines:consistency -- --analyze --output code-review/review-consistency.json
```

### Review Training
Continuous reviewer training:

```bash
# Review training recommendations
npm run review:training:recommend -- --generate --output code-review/training-recommendations.md

# Review skill assessment
npm run review:training:skills -- --assess --output code-review/skill-assessment.json

# Review improvement plan
npm run review:training:plan -- --create --output code-review/training-improvement-plan.md
```

## Common Review Issues

- **Review Fatigue**: Too many review comments overwhelming developers
- **Inconsistent Reviews**: Different reviewers applying different standards
- **Missing Context**: Reviews lacking understanding of broader system context
- **False Positives**: Incorrect identification of issues
- **Review Bottlenecks**: Slow review processes delaying development
- **Quality Inconsistency**: Variable review quality across team members
- **Automation Over-reliance**: Over-dependence on automated tools
- **Feedback Loop Gaps**: Lack of feedback on review effectiveness

## Review Metrics

### Quality Metrics
Track review quality indicators:

- **Review Accuracy**: Percentage of correct findings
- **False Positive Rate**: Percentage of incorrect findings
- **Review Completeness**: Coverage of all relevant aspects
- **Review Timeliness**: Time from submission to completion
- **Developer Satisfaction**: Developer feedback on review quality
- **Bug Detection Rate**: Percentage of bugs caught in review
- **Review Coverage**: Percentage of code reviewed

### Process Metrics
Track review process efficiency:

- **Review Cycle Time**: Average time for review completion
- **Review Participation**: Percentage of team members participating
- **Review Volume**: Number of reviews per period
- **Review Distribution**: Evenness of review workload
- **Review Automation**: Percentage of review automated
- **Review Feedback Loop**: Speed of review improvement
- **Review Scalability**: Ability to handle increased review load

### Impact Metrics
Track review impact on development:

- **Defect Detection**: Number of defects found per review
- **Code Quality Improvement**: Improvement in code quality metrics
- **Development Velocity**: Impact on development speed
- **Team Learning**: Knowledge sharing through reviews
- **Code Consistency**: Improvement in coding standards adherence
- **Security Posture**: Enhancement of security practices
- **Maintainability**: Improvement in code maintainability