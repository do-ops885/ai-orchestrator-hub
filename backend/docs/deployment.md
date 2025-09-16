# Deployment Guide

This guide covers deploying the AI Orchestrator Hub to various environments, from development to production.

## Quick Deployment Options

### Docker Compose (Recommended for Development)

```yaml
# docker-compose.yml
version: '3.8'
services:
  backend:
    image: ai-orchestrator-hub-backend:latest
    ports:
      - "3001:3001"
    environment:
      - HIVE_SERVER__HOST=0.0.0.0
      - HIVE_DATABASE__URL=./data/hive.db
    volumes:
      - ./data:/app/data
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3001/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  frontend:
    image: ai-orchestrator-hub-frontend:latest
    ports:
      - "3000:3000"
    depends_on:
      - backend
    environment:
      - REACT_APP_API_URL=http://localhost:3001
```

```bash
# Deploy
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f
```

### Kubernetes

```yaml
# backend-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ai-orchestrator-backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ai-orchestrator-backend
  template:
    metadata:
      labels:
        app: ai-orchestrator-backend
    spec:
      containers:
      - name: backend
        image: ai-orchestrator-hub-backend:latest
        ports:
        - containerPort: 3001
        env:
        - name: HIVE_SERVER__HOST
          value: "0.0.0.0"
        - name: HIVE_DATABASE__URL
          value: "/data/hive.db"
        volumeMounts:
        - name: data
          mountPath: /data
        livenessProbe:
          httpGet:
            path: /health
            port: 3001
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3001
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: ai-orchestrator-data
```

```yaml
# backend-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: ai-orchestrator-backend
spec:
  selector:
    app: ai-orchestrator-backend
  ports:
  - port: 3001
    targetPort: 3001
  type: ClusterIP
```

```bash
# Deploy to Kubernetes
kubectl apply -f backend-deployment.yaml
kubectl apply -f backend-service.yaml

# Check deployment
kubectl get pods
kubectl get services

# View logs
kubectl logs -f deployment/ai-orchestrator-backend
```

## Environment-Specific Deployments

### Development Environment

```bash
# Local development
cargo run

# With custom config
cargo run -- --config config/dev.toml

# With environment variables
HIVE_LOGGING__LEVEL=debug HIVE_DATABASE__URL=./dev.db cargo run
```

### Staging Environment

```bash
# Docker deployment
docker run -d \
  --name ai-orchestrator-staging \
  -p 3001:3001 \
  -e HIVE_SERVER__HOST=0.0.0.0 \
  -e HIVE_DATABASE__URL=./data/staging.db \
  -e HIVE_LOGGING__LEVEL=info \
  ai-orchestrator-hub-backend:latest
```

### Production Environment

```bash
# Production Docker
docker run -d \
  --name ai-orchestrator-prod \
  --restart unless-stopped \
  -p 3001:3001 \
  -e HIVE_SERVER__HOST=0.0.0.0 \
  -e HIVE_DATABASE__URL=postgresql://user:pass@db:5432/hive \
  -e HIVE_LOGGING__LEVEL=warn \
  -e HIVE_SECURITY__JWT_SECRET=your-prod-secret \
  -v /opt/ai-orchestrator/data:/app/data \
  ai-orchestrator-hub-backend:latest
```

## Cloud Deployments

### AWS

#### ECS (Elastic Container Service)

```json
{
  "family": "ai-orchestrator-backend",
  "taskRoleArn": "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
  "executionRoleArn": "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "1024",
  "memory": "2048",
  "containerDefinitions": [
    {
      "name": "backend",
      "image": "123456789012.dkr.ecr.us-east-1.amazonaws.com/ai-orchestrator-hub-backend:latest",
      "essential": true,
      "portMappings": [
        {
          "containerPort": 3001,
          "hostPort": 3001
        }
      ],
      "environment": [
        {
          "name": "HIVE_SERVER__HOST",
          "value": "0.0.0.0"
        },
        {
          "name": "HIVE_DATABASE__URL",
          "value": "postgresql://user:pass@db.endpoint:5432/hive"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/ai-orchestrator-backend",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      },
      "healthCheck": {
        "command": ["CMD-SHELL", "curl -f http://localhost:3001/health || exit 1"],
        "interval": 30,
        "timeout": 5,
        "retries": 3
      }
    }
  ]
}
```

#### EKS (Elastic Kubernetes Service)

```bash
# Create EKS cluster
eksctl create cluster \
  --name ai-orchestrator \
  --region us-east-1 \
  --nodegroup-name workers \
  --node-type t3.medium \
  --nodes 3 \
  --nodes-min 1 \
  --nodes-max 5

# Deploy application
kubectl apply -f k8s/

# Set up ALB ingress
kubectl apply -f k8s/ingress.yaml
```

### Google Cloud Platform

#### GKE (Google Kubernetes Engine)

```bash
# Create GKE cluster
gcloud container clusters create ai-orchestrator \
  --num-nodes=3 \
  --machine-type=e2-medium \
  --region=us-central1

# Get credentials
gcloud container clusters get-credentials ai-orchestrator

# Deploy
kubectl apply -f k8s/

# Set up load balancer
kubectl apply -f k8s/gcp-load-balancer.yaml
```

#### Cloud Run

```bash
# Build and deploy
gcloud run deploy ai-orchestrator-backend \
  --source . \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --port 3001 \
  --memory 2Gi \
  --cpu 2 \
  --set-env-vars HIVE_SERVER__HOST=0.0.0.0 \
  --set-env-vars HIVE_DATABASE__URL=postgresql://user:pass@db.endpoint:5432/hive
```

### Microsoft Azure

#### AKS (Azure Kubernetes Service)

```bash
# Create AKS cluster
az aks create \
  --resource-group ai-orchestrator-rg \
  --name ai-orchestrator-cluster \
  --node-count 3 \
  --node-vm-size Standard_DS2_v2 \
  --enable-addons monitoring \
  --generate-ssh-keys

# Get credentials
az aks get-credentials \
  --resource-group ai-orchestrator-rg \
  --name ai-orchestrator-cluster

# Deploy
kubectl apply -f k8s/
```

#### Container Instances

```bash
# Create container instance
az container create \
  --resource-group ai-orchestrator-rg \
  --name ai-orchestrator-backend \
  --image ai-orchestrator-hub-backend:latest \
  --ports 3001 \
  --cpu 2 \
  --memory 4 \
  --environment-variables HIVE_SERVER__HOST=0.0.0.0 \
  --ip-address public
```

## Database Setup

### SQLite (Development)

```bash
# SQLite is used by default
# Data is stored in ./data/hive.db
mkdir -p data
cargo run  # Creates database automatically
```

### PostgreSQL (Production)

```bash
# Create database
createdb hive_db

# Create user
createuser hive_user
psql -c "ALTER USER hive_user PASSWORD 'secure_password';"

# Grant permissions
psql -c "GRANT ALL PRIVILEGES ON DATABASE hive_db TO hive_user;"

# Run migrations
psql -d hive_db -f migrations/001_initial.sql
```

### PostgreSQL with Docker

```yaml
# docker-compose.yml
version: '3.8'
services:
  db:
    image: postgres:15
    environment:
      POSTGRES_DB: hive
      POSTGRES_USER: hive_user
      POSTGRES_PASSWORD: secure_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d
    ports:
      - "5432:5432"

  backend:
    image: ai-orchestrator-hub-backend:latest
    depends_on:
      - db
    environment:
      HIVE_DATABASE__URL: postgresql://hive_user:secure_password@db:5432/hive
```

## Load Balancing

### Nginx

```nginx
# nginx.conf
upstream ai_orchestrator_backend {
    server backend1:3001;
    server backend2:3001;
    server backend3:3001;
}

server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://ai_orchestrator_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    # Health check endpoint
    location /health {
        proxy_pass http://ai_orchestrator_backend/health;
        access_log off;
    }
}
```

### HAProxy

```haproxy
# haproxy.cfg
frontend http_front
    bind *:80
    default_backend ai_orchestrator_backend

backend ai_orchestrator_backend
    balance roundrobin
    option httpchk GET /health
    http-check expect status 200

    server backend1 backend1:3001 check
    server backend2 backend2:3001 check
    server backend3 backend3:3001 check
```

### AWS ALB

```yaml
# alb-ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: ai-orchestrator-ingress
  annotations:
    kubernetes.io/ingress.class: alb
    alb.ingress.kubernetes.io/scheme: internet-facing
    alb.ingress.kubernetes.io/target-type: ip
    alb.ingress.kubernetes.io/healthcheck-path: /health
spec:
  rules:
  - host: your-domain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: ai-orchestrator-backend
            port:
              number: 3001
```

## SSL/TLS Configuration

### Let's Encrypt (Automatic)

```bash
# Install certbot
sudo apt install certbot

# Get certificate
sudo certbot certonly --standalone -d your-domain.com

# Configure Nginx with SSL
server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    location / {
        proxy_pass http://ai_orchestrator_backend;
        # ... proxy settings
    }
}
```

### Self-Signed Certificate

```bash
# Generate self-signed certificate
openssl req -x509 -newkey rsa:4096 \
  -keyout key.pem -out cert.pem -days 365 -nodes \
  -subj "/C=US/ST=State/L=City/O=Organization/CN=localhost"

# Use in Docker
docker run -d \
  -p 3001:3001 \
  -v $(pwd)/cert.pem:/app/cert.pem \
  -v $(pwd)/key.pem:/app/key.pem \
  -e HIVE_SECURITY__SSL_CERT_PATH=/app/cert.pem \
  -e HIVE_SECURITY__SSL_KEY_PATH=/app/key.pem \
  ai-orchestrator-hub-backend:latest
```

## Monitoring and Observability

### Prometheus

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'ai-orchestrator'
    static_configs:
      - targets: ['localhost:3001']
    metrics_path: '/metrics'
    scrape_interval: 5s
```

### Grafana

```json
// grafana-dashboard.json
{
  "dashboard": {
    "title": "AI Orchestrator Hub",
    "panels": [
      {
        "title": "Active Agents",
        "type": "graph",
        "targets": [
          {
            "expr": "ai_orchestrator_agents_active",
            "legendFormat": "Active Agents"
          }
        ]
      },
      {
        "title": "Task Success Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "ai_orchestrator_task_success_rate",
            "legendFormat": "Success Rate"
          }
        ]
      }
    ]
  }
}
```

### ELK Stack

```yaml
# filebeat.yml
filebeat.inputs:
- type: log
  paths:
    - /var/log/ai-orchestrator/*.log
  fields:
    service: ai-orchestrator

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
```

## Backup and Recovery

### Database Backup

```bash
# SQLite backup
sqlite3 data/hive.db ".backup hive_backup_$(date +%Y%m%d).db"

# PostgreSQL backup
pg_dump hive_db > hive_backup_$(date +%Y%m%d).sql

# Automated backup script
#!/bin/bash
BACKUP_DIR="/opt/backups"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup
pg_dump hive_db > $BACKUP_DIR/hive_$DATE.sql

# Compress
gzip $BACKUP_DIR/hive_$DATE.sql

# Clean old backups (keep last 7 days)
find $BACKUP_DIR -name "hive_*.sql.gz" -mtime +7 -delete
```

### Configuration Backup

```bash
# Backup configuration
cp .env .env.backup
cp config.toml config.toml.backup

# Backup with timestamp
tar -czf config_backup_$(date +%Y%m%d).tar.gz .env config.toml
```

### Recovery Procedures

```bash
# Stop service
docker-compose down

# Restore database
psql hive_db < hive_backup_20240101.sql

# Restore configuration
cp .env.backup .env

# Start service
docker-compose up -d

# Verify recovery
curl http://localhost:3001/health
```

## Scaling Strategies

### Horizontal Scaling

```yaml
# Kubernetes HPA
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: ai-orchestrator-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: ai-orchestrator-backend
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

### Vertical Scaling

```bash
# Increase resources
kubectl patch deployment ai-orchestrator-backend \
  --type='json' \
  -p='[{"op": "replace", "path": "/spec/template/spec/containers/0/resources/requests/cpu", "value": "1000m"}]'

# Scale manually
kubectl scale deployment ai-orchestrator-backend --replicas=5
```

### Auto-scaling Configuration

```env
# Enable auto-scaling
HIVE_AGENTS__AUTO_SCALING_ENABLED=true
HIVE_AGENTS__MIN_AGENTS=5
HIVE_AGENTS__MAX_SCALE_AGENTS=50

# Scaling thresholds
HIVE_PERFORMANCE__CPU_WARNING_THRESHOLD=70.0
HIVE_PERFORMANCE__MEMORY_WARNING_THRESHOLD=80.0
```

## Security Hardening

### Network Security

```bash
# Configure firewall
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 80
sudo ufw allow 443
sudo ufw --force enable
```

### Container Security

```yaml
# security-context.yaml
apiVersion: v1
kind: Pod
metadata:
  name: ai-orchestrator-secure
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    fsGroup: 2000
  containers:
  - name: backend
    image: ai-orchestrator-hub-backend:latest
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      runAsNonRoot: true
      runAsUser: 1000
      capabilities:
        drop:
        - ALL
```

### Secrets Management

```yaml
# secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: ai-orchestrator-secrets
type: Opaque
data:
  jwt-secret: <base64-encoded-secret>
  db-password: <base64-encoded-password>
```

## Performance Optimization

### Application Optimization

```env
# Performance settings
HIVE_PERFORMANCE__CONNECTION_POOL_SIZE=50
HIVE_PERFORMANCE__CACHE_SIZE_MB=1024
HIVE_PERFORMANCE__CIRCUIT_BREAKER_ENABLED=true

# Database optimization
HIVE_DATABASE__MAX_CONNECTIONS=20
HIVE_DATABASE__CONNECTION_TIMEOUT_SECS=10
```

### Infrastructure Optimization

```bash
# Kernel optimization
echo "net.core.somaxconn = 65536" | sudo tee -a /etc/sysctl.conf
echo "net.ipv4.tcp_max_syn_backlog = 65536" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p

# File descriptor limits
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf
```

## Troubleshooting Deployment

### Common Issues

#### Container Won't Start

```bash
# Check logs
docker logs ai-orchestrator-backend

# Check resource usage
docker stats ai-orchestrator-backend

# Check configuration
docker exec ai-orchestrator-backend env
```

#### Database Connection Issues

```bash
# Test database connection
psql -h db-host -U username -d database

# Check connection pool
curl http://localhost:3001/metrics | grep database
```

#### High Resource Usage

```bash
# Monitor resources
top -p $(pgrep multiagent-hive)

# Check memory leaks
valgrind --tool=memcheck ./target/release/multiagent-hive

# Profile performance
perf record -g ./target/release/multiagent-hive
perf report
```

#### Network Issues

```bash
# Test connectivity
telnet localhost 3001

# Check network configuration
netstat -tulpn | grep 3001

# Test load balancer
curl -H "Host: your-domain.com" http://load-balancer/health
```

### Health Checks

```bash
# Application health
curl http://localhost:3001/health

# Database health
psql -c "SELECT 1;"

# Load balancer health
curl http://load-balancer/health

# External monitoring
curl https://healthcheck.your-domain.com
```

### Log Analysis

```bash
# View recent logs
tail -f /var/log/ai-orchestrator/app.log

# Search for errors
grep "ERROR" /var/log/ai-orchestrator/app.log

# Analyze log patterns
awk '{print $1}' /var/log/ai-orchestrator/app.log | sort | uniq -c | sort -nr
```

This deployment guide provides comprehensive instructions for deploying the AI Orchestrator Hub in various environments with production-ready configurations.