---
description: Deploy the application to various environments with automated checks
agent: github-workflow-manager
---

# Deploy Command

Deploy the AI Orchestrator Hub to staging or production environments with comprehensive pre-deployment checks, automated testing, and rollback capabilities.

## Deployment Strategy

### 1. Pre-Deployment Validation
Ensure deployment readiness:

```bash
# Run comprehensive quality checks
npm run quality-check

# Validate security compliance
npm run security-audit

# Check build integrity
npm run build:validate

# Verify environment configuration
npm run env:validate -- --env production
```

### 2. Build Optimization
Prepare optimized build artifacts:

```bash
# Create production build
npm run build:production

# Optimize bundle size
npm run build:optimize

# Generate deployment manifest
npm run build:manifest -- --version $(git describe --tags)
```

### 3. Deployment Execution
Execute deployment with monitoring:

```bash
# Deploy to staging
npm run deploy:staging -- --wait

# Run smoke tests
npm run test:smoke -- --env staging

# Deploy to production (if staging successful)
npm run deploy:production -- --canary 10%

# Monitor deployment health
npm run deploy:monitor -- --duration 30m
```

### 4. Post-Deployment Validation
Verify successful deployment:

```bash
# Health checks
npm run health:check -- --env production

# Performance validation
npm run perf:validate -- --baseline

# User acceptance tests
npm run test:uat -- --automated
```

## Deployment Environments

### Staging Deployment
Safe environment for testing:

```bash
# Deploy to staging with full validation
npm run deploy:staging -- --full-validation

# Run integration tests
npm run test:integration -- --env staging

# Performance testing
npm run test:performance -- --env staging
```

### Production Deployment
Production deployment with safety measures:

```bash
# Blue-green deployment
npm run deploy:production -- --blue-green

# Feature flag validation
npm run deploy:feature-flags -- --validate

# Gradual rollout
npm run deploy:rollout -- --percentage 25% --monitor
```

### Rollback Procedures
Automated rollback capabilities:

```bash
# Automatic rollback on failure
npm run deploy:rollback -- --auto

# Manual rollback with confirmation
npm run deploy:rollback -- --manual --reason "Performance degradation"

# Rollback validation
npm run rollback:validate
```

## Deployment Types

### Standard Deployment
Regular application updates:

```bash
# Full application deployment
npm run deploy:standard -- --env production

# Database migrations
npm run deploy:migrations -- --safe

# Cache invalidation
npm run deploy:cache -- --invalidate
```

### Blue-Green Deployment
Zero-downtime deployments:

```bash
# Prepare green environment
npm run deploy:prepare-green

# Deploy to green
npm run deploy:green

# Switch traffic
npm run deploy:switch-traffic -- --gradual 5m

# Cleanup blue environment
npm run deploy:cleanup-blue
```

### Canary Deployment
Gradual feature rollout:

```bash
# Deploy to 10% of users
npm run deploy:canary -- --percentage 10

# Monitor metrics
npm run deploy:monitor -- --canary

# Scale up on success
npm run deploy:scale -- --percentage 50

# Full rollout
npm run deploy:complete
```

## Monitoring and Alerting

### Deployment Monitoring
Real-time deployment tracking:

```bash
# Deployment dashboard
npm run deploy:dashboard -- --live

# Metrics monitoring
npm run deploy:metrics -- --alerts

# Log aggregation
npm run deploy:logs -- --tail
```

### Alert Configuration
Automated alerting for deployment issues:

```bash
# Configure deployment alerts
npm run deploy:alerts:config -- --rules deployment-alerts.json

# Error rate monitoring
npm run deploy:alerts:error-rate -- --threshold 5%

# Performance degradation alerts
npm run deploy:alerts:performance -- --baseline
```

## Security and Compliance

### Security Validation
Pre-deployment security checks:

```bash
# Security scanning
npm run security:scan -- --pre-deploy

# Vulnerability assessment
npm run security:vulnerabilities -- --block-critical

# Compliance checks
npm run compliance:check -- --regulatory-requirements
```

### Access Control
Deployment authorization:

```bash
# Approval workflow
npm run deploy:approve -- --request-id 123

# Access validation
npm run deploy:access -- --user $USER --env production

# Audit logging
npm run deploy:audit -- --log-all-actions
```

## Configuration Management

### Environment Configuration
Environment-specific settings:

```bash
# Validate configuration
npm run config:validate -- --env production

# Secrets management
npm run config:secrets -- --rotate

# Feature flags
npm run config:features -- --env production
```

### Infrastructure as Code
Infrastructure deployment:

```bash
# Infrastructure deployment
npm run infra:deploy -- --terraform

# Configuration drift detection
npm run infra:drift -- --check

# Resource optimization
npm run infra:optimize -- --cost
```

## Rollback and Recovery

### Automated Rollback
Intelligent rollback mechanisms:

```bash
# Health-based rollback
npm run rollback:health -- --auto

# Performance-based rollback
npm run rollback:performance -- --threshold 20%

# Manual rollback
npm run rollback:manual -- --confirm
```

### Recovery Procedures
Post-rollback recovery:

```bash
# Data consistency check
npm run recovery:data -- --validate

# Service restoration
npm run recovery:services -- --parallel

# User notification
npm run recovery:notify -- --stakeholders
```

## Best Practices

1. **Zero-Downtime**: Use blue-green or canary deployments
2. **Automated Testing**: Comprehensive pre and post-deployment testing
3. **Monitoring**: Real-time monitoring with automated alerts
4. **Rollback Ready**: Always have rollback procedures ready
5. **Security First**: Security validation at every step
6. **Documentation**: Document all deployment procedures
7. **Audit Trail**: Maintain complete audit logs

## Common Issues

- **Deployment Failures**: Network timeouts, resource constraints
- **Configuration Errors**: Environment-specific misconfigurations
- **Database Issues**: Migration failures, connection problems
- **Performance Degradation**: Resource contention, memory leaks
- **Security Vulnerabilities**: Unpatched dependencies, misconfigurations
- **Monitoring Gaps**: Missing alerts, incomplete metrics
- **Rollback Complexity**: Complex state management during rollbacks

## Integration

This command integrates with:
- **GitHub Actions**: Automated deployment workflows
- **Docker**: Container deployment and orchestration
- **Kubernetes**: Orchestration and scaling
- **Monitoring**: Prometheus, Grafana, ELK stack
- **Security**: Vulnerability scanners, compliance tools
- **CDN**: Content distribution and caching