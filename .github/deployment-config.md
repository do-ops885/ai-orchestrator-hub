# Production Deployment Configuration Guide

This guide provides platform-specific configuration for the blue-green deployment workflow.

## Supported Platforms

### 1. AWS ECS (Elastic Container Service)

#### Prerequisites
- AWS CLI configured with appropriate permissions
- ECS cluster created
- ECR repositories for backend and frontend images
- Application Load Balancer (ALB) configured
- Target Groups for blue and green environments

#### Environment Variables
```bash
AWS_REGION=us-east-1
ECS_CLUSTER=ai-orchestrator-prod
ECS_SERVICE_BLUE=ai-orchestrator-service-blue
ECS_SERVICE_GREEN=ai-orchestrator-service-green
ALB_LISTENER_ARN=arn:aws:elasticloadbalancing:region:account:listener/app/alb-name/123456789/listener-rule
BLUE_TG_ARN=arn:aws:elasticloadbalancing:region:account:targetgroup/blue-tg/123456789
GREEN_TG_ARN=arn:aws:elasticloadbalancing:region:account:targetgroup/green-tg/123456789
```

#### Deployment Commands (Replace placeholders in workflow)
```yaml
- name: Deploy to inactive environment
  run: |
    TARGET_ENV="${{ steps.target.outputs.target_environment }}"
    aws ecs update-service \
      --cluster ${{ env.ECS_CLUSTER }} \
      --service ai-orchestrator-service-$TARGET_ENV \
      --force-new-deployment \
      --task-definition ${{ env.BACKEND_IMAGE }}:latest

- name: Gradual traffic shifting
  run: |
    # Shift 25% traffic to new environment
    aws elbv2 modify-listener \
      --listener-arn ${{ env.ALB_LISTENER_ARN }} \
      --default-actions '[
        {
          "Type": "forward",
          "ForwardConfig": {
            "TargetGroups": [
              {"TargetGroupArn": "${{ env.BLUE_TG_ARN }}", "Weight": 75},
              {"TargetGroupArn": "${{ env.GREEN_TG_ARN }}", "Weight": 25}
            ]
          }
        }
      ]'
```

### 2. Kubernetes

#### Prerequisites
- kubectl configured with cluster access
- Kubernetes cluster with blue/green namespaces or deployments
- Ingress controller configured
- Service mesh (Istio) for traffic management (recommended)

#### Deployment Commands
```yaml
- name: Deploy to inactive environment
  run: |
    TARGET_ENV="${{ steps.target.outputs.target_environment }}"
    kubectl set image deployment/ai-orchestrator-backend-$TARGET_ENV \
      ai-orchestrator-backend=${{ env.REGISTRY }}/${{ env.BACKEND_IMAGE }}:latest
    kubectl set image deployment/ai-orchestrator-frontend-$TARGET_ENV \
      ai-orchestrator-frontend=${{ env.REGISTRY }}/${{ env.FRONTEND_IMAGE }}:latest
    kubectl rollout status deployment/ai-orchestrator-backend-$TARGET_ENV
    kubectl rollout status deployment/ai-orchestrator-frontend-$TARGET_ENV

- name: Gradual traffic shifting (with Istio)
  run: |
    TARGET_ENV="${{ steps.target.outputs.target_environment }}"
    # Update VirtualService to shift traffic
    kubectl apply -f - <<EOF
    apiVersion: networking.istio.io/v1beta1
    kind: VirtualService
    metadata:
      name: ai-orchestrator
    spec:
      http:
      - route:
        - destination:
            host: ai-orchestrator-backend-$TARGET_ENV
          weight: 25
        - destination:
            host: ai-orchestrator-backend-${{ steps.target.outputs.current_active }}
          weight: 75
    EOF
```

### 3. Docker Compose (for simpler deployments)

#### Prerequisites
- Docker and Docker Compose installed
- docker-compose.prod.yml file configured

#### Deployment Commands
```yaml
- name: Deploy to inactive environment
  run: |
    TARGET_ENV="${{ steps.target.outputs.target_environment }}"
    export COMPOSE_PROJECT_NAME=ai-orchestrator-$TARGET_ENV
    docker-compose -f docker-compose.prod.yml pull
    docker-compose -f docker-compose.prod.yml up -d

- name: Health check
  run: |
    TARGET_ENV="${{ steps.target.outputs.target_environment }}"
    # Wait for health check to pass
    for i in {1..30}; do
      if docker-compose -f docker-compose.prod.yml exec -T backend curl -f http://localhost:8080/health; then
        break
      fi
      sleep 10
    done
```

## Health Check Configuration

### Backend Health Endpoint
The backend should expose a `/health` endpoint that returns:
- HTTP 200 for healthy
- HTTP 5xx for unhealthy

Example health check implementation:
```rust
#[get("/health")]
async fn health_check() -> HttpResponse {
    // Check database connectivity
    // Check external service dependencies
    // Return appropriate status
    HttpResponse::Ok().json(json!({"status": "healthy"}))
}
```

### Frontend Health Check
For Next.js applications, you can use:
```javascript
// pages/api/health.js
export default function handler(req, res) {
  res.status(200).json({ status: 'healthy' });
}
```

## Monitoring Integration

### Application Performance Monitoring (APM)
- **DataDog**: Use `datadog-ci` for deployment markers
- **New Relic**: Use `newrelic` CLI for deployment notifications
- **Custom**: Send webhooks to your monitoring system

### Alerting
Configure alerts for:
- Deployment failures
- Health check failures
- Performance degradation
- Error rate spikes

## Secrets Management

Required GitHub Secrets:
```
AWS_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY
AWS_REGION
GITHUB_TOKEN (automatically provided)
DOCKER_HUB_TOKEN (if using Docker Hub)
```

## Rollback Procedures

### Automatic Rollback Triggers
- Health check failures during deployment
- Error rate > 5% after deployment
- Response time degradation > 50%

### Manual Rollback
Use the `rollback.yml` workflow with appropriate parameters:
- `immediate`: Instant switch to previous version
- `gradual`: 10-minute traffic shift back
- `blue-green-swap`: Switch to previous environment

## Cost Optimization

### GitHub Actions Costs
- Use larger runners for faster builds (but monitor usage)
- Cache dependencies effectively
- Schedule heavy workflows during off-peak hours

### Infrastructure Costs
- Scale down inactive environment during normal operation
- Use spot instances for non-production workloads
- Implement auto-scaling based on traffic patterns

## Security Considerations

### Deployment Security
- Use private registries for container images
- Scan images for vulnerabilities before deployment
- Implement secrets management for sensitive configuration
- Use least-privilege IAM roles

### Runtime Security
- Enable security headers
- Implement rate limiting
- Use HTTPS everywhere
- Regular security audits

## Troubleshooting

### Common Issues
1. **Health checks failing**: Check application logs, database connectivity
2. **Traffic not shifting**: Verify load balancer configuration
3. **Image pull failures**: Check registry credentials and network connectivity
4. **Rollback failures**: Ensure previous version artifacts are available

### Debug Commands
```bash
# Check deployment status
kubectl get pods -n production
kubectl logs deployment/ai-orchestrator-backend

# Check load balancer
aws elbv2 describe-target-health --target-group-arn $TG_ARN

# Check container health
docker ps
docker logs ai-orchestrator-backend
```

## Performance Benchmarks

Expected deployment times:
- Build: 10-15 minutes
- Health checks: 2-5 minutes
- Traffic shifting: 5-10 minutes
- Total deployment: 20-30 minutes

Monitor and adjust timeouts based on your infrastructure performance.
