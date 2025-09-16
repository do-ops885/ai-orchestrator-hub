# AI Orchestrator Hub - Production Deployment Guide

This document provides comprehensive instructions for deploying the AI Orchestrator Hub backend to production environments.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Docker Deployment](#docker-deployment)
4. [Kubernetes Deployment](#kubernetes-deployment)
5. [Helm Deployment](#helm-deployment)
6. [CI/CD Pipeline](#ci-cd-pipeline)
7. [Monitoring & Observability](#monitoring--observability)
8. [Security Configuration](#security-configuration)
9. [Scaling & Performance](#scaling--performance)
10. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- **Kubernetes**: v1.24+
- **Helm**: v3.8+
- **Docker**: v20.10+
- **kubectl**: v1.24+
- **Rust**: 1.75+ (for local development)

### Infrastructure Requirements

- **PostgreSQL**: 15+ (recommended) or SQLite (development)
- **Redis**: 7+ (for caching and session management)
- **Ingress Controller**: NGINX Ingress Controller
- **Cert Manager**: For TLS certificate management
- **Monitoring Stack**: Prometheus + Grafana (optional but recommended)

### Network Requirements

- **Ports**:
  - 8080: HTTP API
  - 8081: WebSocket
  - 5432: PostgreSQL
  - 6379: Redis
  - 9090: Prometheus
  - 3000: Grafana

## Quick Start

### Local Development

```bash
# Clone the repository
git clone https://github.com/your-org/ai-orchestrator-hub.git
cd ai-orchestrator-hub/backend

# Copy environment configuration
cp .env.example .env

# Start with Docker Compose
docker-compose up -d

# Check health
curl http://localhost:8080/health
```

### Production Deployment

```bash
# Using deployment script
./scripts/deploy.sh production v1.0.0

# Or using Helm
helm install ai-orchestrator ./helm/ai-orchestrator \
  --namespace production \
  --create-namespace \
  --set image.tag=v1.0.0
```

## Docker Deployment

### Single Container

```bash
# Build the image
docker build -t ai-orchestrator-backend:latest .

# Run the container
docker run -d \
  --name ai-orchestrator \
  -p 8080:8080 \
  -e DATABASE_URL=sqlite:///app/data/hive_persistence.db \
  -e JWT_SECRET=your-secret-key \
  -v ./data:/app/data \
  ai-orchestrator-backend:latest
```

### Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f ai-orchestrator-backend

# Scale services
docker-compose up -d --scale ai-orchestrator-backend=3
```

## Kubernetes Deployment

### Manual Deployment

```bash
# Create namespace
kubectl create namespace ai-orchestrator

# Apply configurations
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml
kubectl apply -f k8s/pvc.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/ingress.yaml
kubectl apply -f k8s/hpa.yaml
kubectl apply -f k8s/network-policy.yaml

# Check deployment status
kubectl get pods -n ai-orchestrator
kubectl get svc -n ai-orchestrator
```

### Using Kustomize

```bash
# Apply with Kustomize
kubectl apply -k k8s/

# Check status
kubectl get all -k k8s/
```

## Helm Deployment

### Install Chart

```bash
# Add Helm repository (if using remote chart)
helm repo add ai-orchestrator https://your-helm-repo.com
helm repo update

# Install with custom values
helm install ai-orchestrator ./helm/ai-orchestrator \
  --namespace ai-orchestrator \
  --create-namespace \
  --values values-production.yaml
```

### Upgrade Deployment

```bash
# Upgrade with new version
helm upgrade ai-orchestrator ./helm/ai-orchestrator \
  --set image.tag=v1.1.0 \
  --set replicaCount=5

# Rollback if needed
helm rollback ai-orchestrator 1
```

### Uninstall

```bash
# Uninstall release
helm uninstall ai-orchestrator -n ai-orchestrator

# Clean up PVCs (if needed)
kubectl delete pvc -l app=ai-orchestrator-backend -n ai-orchestrator
```

## CI/CD Pipeline

### GitHub Actions

The CI/CD pipeline includes:

1. **Testing**: Unit tests, integration tests, security scans
2. **Building**: Docker image build and push
3. **Security**: Vulnerability scanning with Trivy
4. **Deployment**: Automatic deployment to staging/production
5. **Performance**: Load testing with k6

### Pipeline Stages

```yaml
# Key stages in .github/workflows/ci-cd.yml
- test: Run tests and linting
- build: Build Docker image
- docker: Push to registry
- security-scan: Vulnerability scanning
- deploy-staging: Deploy to staging
- deploy-production: Deploy to production
- performance-test: Load testing
```

### Manual Deployment

```bash
# Trigger deployment manually
gh workflow run ci-cd.yml \
  -f environment=production \
  -f version=v1.0.0
```

## Monitoring & Observability

### Prometheus Metrics

The application exposes metrics at `/metrics`:

- **HTTP Metrics**: Request count, duration, error rates
- **Agent Metrics**: Active agents, performance, task completion
- **System Metrics**: CPU, memory, disk usage
- **Business Metrics**: Task success rate, queue size

### Grafana Dashboards

Pre-configured dashboards include:

- **System Overview**: Health status, resource usage
- **Agent Performance**: Agent metrics, task completion rates
- **API Performance**: Response times, error rates
- **Infrastructure**: Kubernetes pod metrics, node status

### Alerting

Configured alerts for:

- Service downtime
- High error rates
- Resource exhaustion
- Performance degradation
- Security incidents

## Security Configuration

### Secrets Management

```yaml
# Kubernetes secrets
apiVersion: v1
kind: Secret
metadata:
  name: ai-orchestrator-secrets
type: Opaque
data:
  database_url: <base64-encoded>
  jwt_secret: <base64-encoded>
  encryption_key: <base64-encoded>
```

### Network Security

- **Network Policies**: Restrict pod-to-pod communication
- **TLS Encryption**: End-to-end encryption with cert-manager
- **RBAC**: Role-based access control for Kubernetes resources

### Container Security

- **Non-root user**: Containers run as non-root user
- **Read-only filesystem**: Prevent unauthorized modifications
- **Security contexts**: Pod security standards compliance

## Scaling & Performance

### Horizontal Pod Autoscaler

```yaml
# HPA configuration
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: ai-orchestrator-hpa
spec:
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

### Resource Limits

```yaml
resources:
  requests:
    memory: "256Mi"
    cpu: "250m"
  limits:
    memory: "1Gi"
    cpu: "1000m"
```

### Performance Optimization

- **Connection pooling**: Database and Redis connection pooling
- **Caching**: Multi-level caching strategy
- **Async processing**: Non-blocking I/O operations
- **Resource monitoring**: Automatic scaling based on metrics

## Troubleshooting

### Common Issues

#### Pod Startup Issues

```bash
# Check pod status
kubectl describe pod <pod-name> -n ai-orchestrator

# Check logs
kubectl logs <pod-name> -n ai-orchestrator

# Check events
kubectl get events -n ai-orchestrator
```

#### Database Connection Issues

```bash
# Test database connectivity
kubectl exec -it <pod-name> -n ai-orchestrator -- nc -zv <db-host> 5432

# Check database credentials
kubectl get secret ai-orchestrator-secrets -o yaml
```

#### Performance Issues

```bash
# Check resource usage
kubectl top pods -n ai-orchestrator

# Check HPA status
kubectl get hpa -n ai-orchestrator

# Check metrics
kubectl exec -it <pod-name> -n ai-orchestrator -- curl http://localhost:8080/metrics
```

### Health Checks

```bash
# Application health
curl http://your-domain.com/health

# Kubernetes health
kubectl get componentstatuses

# Database health
kubectl exec -it <db-pod> -- pg_isready
```

### Logs and Debugging

```bash
# Application logs
kubectl logs -f deployment/ai-orchestrator-backend -n ai-orchestrator

# System logs
kubectl logs -f -l app=ai-orchestrator-backend -n ai-orchestrator

# Debug container
kubectl debug <pod-name> -n ai-orchestrator --image=busybox -- sleep 3600
```

## Configuration Reference

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_HOST` | Server bind address | `0.0.0.0` |
| `SERVER_PORT` | Server port | `8080` |
| `DATABASE_URL` | Database connection URL | SQLite path |
| `JWT_SECRET` | JWT signing secret | Required |
| `REDIS_URL` | Redis connection URL | `redis://localhost:6379` |
| `LOG_LEVEL` | Logging level | `info` |

### Helm Values

```yaml
# Complete values reference
image:
  repository: ai-orchestrator-backend
  tag: "latest"

replicaCount: 3

resources:
  requests:
    memory: "256Mi"
    cpu: "250m"
  limits:
    memory: "1Gi"
    cpu: "1000m"

ingress:
  enabled: true
  hosts:
    - host: api.your-domain.com

monitoring:
  prometheus: true
  grafana: true
```

## Backup and Recovery

### Database Backup

```bash
# PostgreSQL backup
kubectl exec -it <postgres-pod> -- pg_dump -U ai_user ai_orchestrator > backup.sql

# SQLite backup (if using SQLite)
kubectl cp <pod>:/app/data/hive_persistence.db ./backup.db
```

### Configuration Backup

```bash
# Backup Helm release
helm get values ai-orchestrator > values-backup.yaml

# Backup secrets (be careful with sensitive data)
kubectl get secret ai-orchestrator-secrets -o yaml > secrets-backup.yaml
```

## Support and Maintenance

### Regular Maintenance Tasks

1. **Update Dependencies**: Regularly update Rust dependencies
2. **Security Patches**: Apply security patches promptly
3. **Performance Monitoring**: Monitor and optimize performance
4. **Log Rotation**: Configure log rotation and retention
5. **Backup Verification**: Regularly test backup restoration

### Support Contacts

- **Development Team**: dev@your-domain.com
- **Infrastructure Team**: infra@your-domain.com
- **Security Team**: security@your-domain.com

### Documentation Updates

Keep this deployment guide updated with:
- New configuration options
- Security updates
- Performance optimizations
- Troubleshooting procedures

---

For additional support or questions, please contact the development team or create an issue in the project repository.