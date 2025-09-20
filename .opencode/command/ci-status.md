---
description: Check and monitor CI/CD pipeline status with actionable insights
agent: github-workflow-manager
---

# CI Status Command

Monitor and analyze CI/CD pipeline status across all workflows, providing actionable insights, performance metrics, and automated recommendations for pipeline optimization.

## CI/CD Monitoring Strategy

### 1. Environment Setup
Prepare CI monitoring environment:

```bash
# Authenticate with GitHub
gh auth login

# Set repository context
export GITHUB_REPO="your-org/ai-orchestrator-hub"
export GITHUB_BRANCH="main"

# Create monitoring directory
mkdir -p ci-status/$(date +%Y%m%d_%H%M%S)
```

### 2. Workflow Status Overview
Get comprehensive workflow status:

```bash
# Get all workflow runs
gh workflow list --json name,status,conclusion > ci-status/workflows.json

# Get recent workflow runs
gh run list --limit 20 --json status,conclusion,createdAt > ci-status/recent-runs.json

# Get workflow run details
gh run view latest --json jobs > ci-status/latest-run-details.json
```

### 3. Pipeline Health Analysis
Analyze pipeline health and performance:

```bash
# Pipeline success rate
npm run ci:analyze:success-rate -- --period 30d --output ci-status/success-rate.json

# Pipeline duration trends
npm run ci:analyze:duration -- --period 30d --output ci-status/duration-trends.json

# Failure pattern analysis
npm run ci:analyze:failures -- --period 30d --output ci-status/failure-patterns.json
```

### 4. Job Status Monitoring
Monitor individual job status:

```bash
# Get job status for latest run
gh run view latest --json jobs.status,jobs.conclusion,jobs.name > ci-status/job-status.json

# Monitor job progress
npm run ci:monitor:jobs -- --run-id latest --watch

# Job performance analysis
npm run ci:analyze:jobs -- --run-id latest --output ci-status/job-performance.json
```

### 5. Artifact and Deployment Status
Check build artifacts and deployments:

```bash
# List build artifacts
gh run view latest --json artifacts > ci-status/artifacts.json

# Check deployment status
npm run ci:deploy:status -- --environment production --output ci-status/deployment-status.json

# Artifact validation
npm run ci:artifact:validate -- --run-id latest --output ci-status/artifact-validation.json
```

## CI/CD Analysis Categories

### Performance Analysis
Analyze pipeline performance metrics:

```bash
# Pipeline execution time analysis
npm run ci:performance:execution -- --analyze --output ci-status/execution-analysis.json

# Resource utilization analysis
npm run ci:performance:resources -- --analyze --output ci-status/resource-utilization.json

# Bottleneck identification
npm run ci:performance:bottlenecks -- --identify --output ci-status/bottleneck-analysis.json
```

### Reliability Analysis
Assess pipeline reliability:

```bash
# Failure rate analysis
npm run ci:reliability:failures -- --analyze --output ci-status/failure-rate-analysis.json

# Flaky test detection
npm run ci:reliability:flaky-tests -- --detect --output ci-status/flaky-tests.json

# Recovery time analysis
npm run ci:reliability:recovery -- --analyze --output ci-status/recovery-analysis.json
```

### Cost Analysis
Analyze CI/CD costs and optimization opportunities:

```bash
# Cost analysis by workflow
npm run ci:cost:workflows -- --analyze --output ci-status/workflow-costs.json

# Cost optimization recommendations
npm run ci:cost:optimize -- --recommend --output ci-status/cost-optimization.json

# Resource efficiency analysis
npm run ci:cost:efficiency -- --analyze --output ci-status/resource-efficiency.json
```

## Real-time Monitoring

### Live Dashboard
Set up real-time CI monitoring:

```bash
# Start CI monitoring dashboard
npm run ci:dashboard -- --live --port 3006

# Real-time workflow monitoring
npm run ci:monitor:workflows -- --watch --alerts

# Job progress visualization
npm run ci:visualize:progress -- --run-id latest --output ci-status/progress-visualization.html
```

### Alert Configuration
Configure CI/CD alerts:

```bash
# Pipeline failure alerts
npm run ci:alerts:failure -- --configure --channels slack,email

# Performance degradation alerts
npm run ci:alerts:performance -- --configure --threshold 20%

# Cost overrun alerts
npm run ci:alerts:cost -- --configure --budget 1000
```

## Workflow Optimization

### Performance Optimization
Optimize workflow performance:

```bash
# Workflow parallelization analysis
npm run ci:optimize:parallel -- --analyze --output ci-status/parallelization-opportunities.json

# Cache optimization
npm run ci:optimize:cache -- --analyze --output ci-status/cache-optimization.json

# Build time reduction
npm run ci:optimize:build-time -- --recommend --output ci-status/build-time-optimization.json
```

### Reliability Optimization
Improve workflow reliability:

```bash
# Flaky test mitigation
npm run ci:optimize:flaky -- --recommend --output ci-status/flaky-test-mitigation.json

# Error handling improvement
npm run ci:optimize:errors -- --analyze --output ci-status/error-handling-improvements.json

# Retry strategy optimization
npm run ci:optimize:retry -- --recommend --output ci-status/retry-strategy-optimization.json
```

## CI/CD Reporting

### Status Reports
Generate comprehensive CI/CD reports:

```bash
# Daily CI status report
npm run ci:report:daily -- --generate --output ci-status/daily-report.pdf

# Weekly CI performance report
npm run ci:report:weekly -- --generate --output ci-status/weekly-report.pdf

# Monthly CI trends report
npm run ci:report:monthly -- --generate --output ci-status/monthly-report.pdf
```

### Executive Reports
High-level CI/CD insights:

```bash
# Executive CI summary
npm run ci:report:executive -- --generate --output ci-status/executive-summary.pdf

# CI ROI analysis
npm run ci:report:roi -- --calculate --output ci-status/ci-roi-analysis.pdf

# CI improvement roadmap
npm run ci:report:roadmap -- --generate --output ci-status/ci-improvement-roadmap.pdf
```

## Troubleshooting and Debugging

### Pipeline Debugging
Debug failing pipelines:

```bash
# Failed job analysis
npm run ci:debug:failed-jobs -- --run-id latest --output ci-status/failed-job-analysis.json

# Log analysis
npm run ci:debug:logs -- --run-id latest --search "error" --output ci-status/log-analysis.json

# Dependency analysis
npm run ci:debug:dependencies -- --run-id latest --output ci-status/dependency-analysis.json
```

### Performance Debugging
Debug performance issues:

```bash
# Slow job analysis
npm run ci:debug:performance -- --run-id latest --output ci-status/performance-debug.json

# Resource bottleneck analysis
npm run ci:debug:resources -- --run-id latest --output ci-status/resource-bottleneck.json

# Network issue detection
npm run ci:debug:network -- --run-id latest --output ci-status/network-issue-analysis.json
```

## Integration with Development Workflow

### Pre-commit CI Checks
Integrate CI checks into development workflow:

```bash
# Pre-commit CI validation
npm run ci:pre-commit -- --setup

# Local CI simulation
npm run ci:simulate -- --run-local

# CI requirement validation
npm run ci:validate:requirements -- --check
```

### Branch-specific CI
Manage CI for different branches:

```bash
# Branch-specific workflow status
npm run ci:branch:status -- --branch feature/new-feature --output ci-status/branch-status.json

# Branch comparison
npm run ci:branch:compare -- --source feature/new-feature --target main --output ci-status/branch-comparison.json

# Branch-specific optimizations
npm run ci:branch:optimize -- --branch feature/new-feature --output ci-status/branch-optimization.json
```

## Best Practices

1. **Fast Feedback**: Ensure quick CI feedback for developers
2. **Reliable Pipelines**: Maintain high pipeline reliability
3. **Cost Optimization**: Optimize CI costs without sacrificing quality
4. **Security Integration**: Integrate security checks into CI
5. **Monitoring**: Continuous monitoring of CI performance
6. **Documentation**: Document CI processes and troubleshooting
7. **Automation**: Automate as much as possible

## Common CI/CD Issues

- **Slow Pipelines**: Long execution times affecting developer productivity
- **Flaky Tests**: Intermittent test failures causing false negatives
- **Resource Contention**: Insufficient resources causing timeouts
- **Dependency Issues**: Dependency conflicts or network issues
- **Configuration Drift**: Inconsistent CI configuration across branches
- **Security Vulnerabilities**: Outdated CI tools or configurations
- **Cost Overruns**: Excessive CI resource usage
- **Integration Problems**: Issues with external service integrations

## CI/CD Metrics

### Key Performance Indicators
Track important CI/CD metrics:

- **Pipeline Success Rate**: Percentage of successful pipeline runs
- **Mean Time to Feedback**: Average time from commit to CI results
- **Pipeline Duration**: Average time to complete CI pipeline
- **Failure Recovery Time**: Time to fix and redeploy after failures
- **Cost per Build**: Average cost of CI pipeline execution
- **Test Coverage**: Code coverage from CI tests
- **Deployment Frequency**: How often deployments occur

### Quality Metrics
CI/CD quality indicators:

- **Test Reliability**: Consistency of test results
- **Build Stability**: Consistency of build success
- **Deployment Success Rate**: Percentage of successful deployments
- **Rollback Frequency**: How often rollbacks are needed
- **Security Scan Results**: Security vulnerabilities found
- **Performance Benchmarks**: Performance regression detection
- **Code Quality Gates**: Quality standards compliance