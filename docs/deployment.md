# Deployment Guide

This guide covers deploying the AI Orchestrator Hub to various environments, from development to production.

## Deployment Overview

### Architecture Options

1. **Single Server**: Backend and frontend on one machine
2. **Distributed**: Separate backend and frontend servers
3. **Containerized**: Docker containers with orchestration
4. **Cloud Native**: Kubernetes with microservices
5. **Serverless**: API Gateway + Lambda functions

## Local Development Deployment

### Quick Start

```bash
# Clone repository
git clone https://github.com/do-ops885/ai-orchestrator-hub.git
cd ai-orchestrator-hub
```

### Development with Docker

```dockerfile
# docker-compose.dev.yml
version: '3.8'

services:
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile.dev
    ports:
      - "3001:3001"
    volumes:
      - ./backend:/app
      - /app/target
    environment:
      - RUST_LOG=debug
      - DEBUG_MODE=true

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile.dev
    ports:
      - "3000:3000"
    volumes:
      - ./frontend:/app
      - /app/node_modules
      - /app/.next
    environment:
      - NODE_ENV=development
```

## Production Deployment

### Single Server Deployment

#### Backend Production Build

```bash
cd backend

# Build optimized release
cargo build --release

# Run with production config
./target/release/ai-orchestrator-hub
```

#### Frontend Production Build

```bash
cd frontend

# Build optimized bundle
npm run build

# Start production server
npm start
```

### Docker Deployment

#### Backend Dockerfile

```dockerfile
FROM rust:1.70-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build optimized binary
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ai-orchestrator-hub /usr/local/bin/

EXPOSE 3001
CMD ["ai-orchestrator-hub"]
```

#### Frontend Dockerfile

```dockerfile
FROM node:20-alpine as builder

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

FROM node:20-alpine as runner

WORKDIR /app
COPY --from=builder /app/package*.json ./
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/public ./public

EXPOSE 3000
ENV NODE_ENV=production
CMD ["npm", "start"]
```

#### Docker Compose Production

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    ports:
      - "3001:3001"
    environment:
      - HIVE_PORT=3001
      - LOG_LEVEL=warn
      - NEURAL_MODE=advanced
    volumes:
      - ./data:/app/data
    restart: unless-stopped

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      - NEXT_PUBLIC_API_URL=http://backend:3001
    depends_on:
      - backend
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/ssl/certs
    depends_on:
      - frontend
    restart: unless-stopped
```

#### Nginx Configuration

```nginx
# nginx.conf
events {
    worker_connections 1024;
}

http {
    upstream backend {
        server backend:3001;
    }

    upstream frontend {
        server frontend:3000;
    }

    server {
        listen 80;
        server_name your-domain.com;

        # Frontend
        location / {
            proxy_pass http://frontend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # Backend API
        location /api/ {
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # WebSocket
        location /ws/ {
            proxy_pass http://backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
```

## Cloud Deployment

### AWS Deployment

#### ECS Fargate

```yaml
# ecs-task-definition.json
{
  "family": "multiagent-hive",
  "taskRoleArn": "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
  "executionRoleArn": "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "1024",
  "memory": "2048",
  "containerDefinitions": [
    {
      "name": "backend",
      "image": "your-registry/multiagent-hive-backend:latest",
      "portMappings": [
        {
          "containerPort": 3001,
          "hostPort": 3001,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {"name": "HIVE_PORT", "value": "3001"},
        {"name": "DATABASE_URL", "value": "postgresql://..."}
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/multiagent-hive",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

#### API Gateway + Lambda

```yaml
# serverless.yml
service: multiagent-hive

provider:
  name: aws
  runtime: rust
  stage: prod
  region: us-east-1

functions:
  api:
    handler: target/lambda/bootstrap
    events:
      - http:
          path: /{proxy+}
          method: any
    environment:
      DATABASE_URL: ${env:DATABASE_URL}

resources:
  Resources:
    ApiGateway:
      Type: AWS::ApiGateway::RestApi
      Properties:
        Name: MultiagentHiveAPI
```

### Google Cloud Platform

#### Cloud Run

```yaml
# cloud-run.yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: multiagent-hive
spec:
  template:
    spec:
      containers:
      - image: gcr.io/your-project/multiagent-hive:latest
        ports:
        - containerPort: 3001
        env:
        - name: HIVE_PORT
          value: "3001"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: connection-string
        resources:
          limits:
            cpu: 1000m
            memory: 2Gi
```

#### Kubernetes Deployment

```yaml
# k8s-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: multiagent-hive-backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: multiagent-hive-backend
  template:
    metadata:
      labels:
        app: multiagent-hive-backend
    spec:
      containers:
      - name: backend
        image: your-registry/multiagent-hive-backend:latest
        ports:
        - containerPort: 3001
        env:
        - name: HIVE_PORT
          value: "3001"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: connection-string
        resources:
          requests:
            cpu: 500m
            memory: 1Gi
          limits:
            cpu: 1000m
            memory: 2Gi
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
```

### Azure Deployment

#### Azure Container Instances

```bash
# Deploy to ACI
az container create \
  --resource-group your-rg \
  --name multiagent-hive \
  --image your-registry/multiagent-hive:latest \
  --ports 3001 \
  --environment-variables HIVE_PORT=3001 \
  --cpu 1 \
  --memory 2
```

#### Azure App Service

```json
// arm-template.json
{
  "$schema": "https://schema.management.azure.com/schemas/2019-04-01/deploymentTemplate.json#",
  "contentVersion": "1.0.0.0",
  "parameters": {
    "siteName": {
      "type": "string",
      "defaultValue": "multiagent-hive"
    }
  },
  "resources": [
    {
      "type": "Microsoft.Web/sites",
      "apiVersion": "2020-06-01",
      "name": "[parameters('siteName')]",
      "location": "[resourceGroup().location]",
      "properties": {
        "serverFarmId": "[resourceId('Microsoft.Web/serverfarms', variables('hostingPlanName'))]",
        "siteConfig": {
          "linuxFxVersion": "DOCKER|your-registry/multiagent-hive:latest"
        }
      }
    }
  ]
}
```

## Database Deployment

### PostgreSQL Setup

```bash
# Create database
createdb multiagent_hive

# Create user
createuser hive_user
psql -c "ALTER USER hive_user PASSWORD 'secure-password';"

# Grant permissions
psql -c "GRANT ALL PRIVILEGES ON DATABASE multiagent_hive TO hive_user;"
```

### Connection String Examples

```env
# Local PostgreSQL
DATABASE_URL=postgresql://hive_user:password@localhost/multiagent_hive

# AWS RDS
DATABASE_URL=postgresql://hive_user:password@hive-db.cluster-xyz.us-east-1.rds.amazonaws.com/multiagent_hive

# Google Cloud SQL
DATABASE_URL=postgresql://hive_user:password@/multiagent_hive?host=/cloudsql/your-project:us-central1:hive-db
```

## Load Balancing

### Nginx Load Balancer

```nginx
# nginx-lb.conf
upstream backend_servers {
    server backend1:3001;
    server backend2:3001;
    server backend3:3001;
}

server {
    listen 80;
    server_name api.your-domain.com;

    location / {
        proxy_pass http://backend_servers;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket support
    location /ws/ {
        proxy_pass http://backend_servers;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### HAProxy Configuration

```haproxy
# haproxy.cfg
frontend http_front
    bind *:80
    default_backend backend_servers

backend backend_servers
    balance roundrobin
    server backend1 backend1:3001 check
    server backend2 backend2:3001 check
    server backend3 backend3:3001 check
```

## SSL/TLS Configuration

### Let's Encrypt SSL

```bash
# Install certbot
sudo apt install certbot

# Get certificate
sudo certbot certonly --standalone -d your-domain.com

# Configure Nginx
server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    # ... rest of config
}
```

### Self-Signed Certificate

```bash
# Generate certificate
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# Configure in application
SSL_CERT_PATH=/path/to/cert.pem
SSL_KEY_PATH=/path/to/key.pem
```

## Monitoring and Logging

### Application Monitoring

```yaml
# monitoring-stack.yml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana

  loki:
    image: grafana/loki
    ports:
      - "3100:3100"

  promtail:
    image: grafana/promtail
    volumes:
      - ./promtail-config.yml:/etc/promtail/config.yml
      - /var/log:/var/log
```

### Log Aggregation

```yaml
# promtail-config.yml
server:
  http_listen_port: 9080
  grpc_listen_port: 0

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: system
    static_configs:
      - targets:
          - localhost
        labels:
          job: varlogs
          __path__: /var/log/*log
```

## Backup and Recovery

### Database Backup

```bash
# PostgreSQL backup
pg_dump multiagent_hive > backup.sql

# Automated backup script
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
pg_dump multiagent_hive > backup_$DATE.sql
gzip backup_$DATE.sql
```

### Application Data Backup

```bash
# Backup configuration and data
tar -czf backup.tar.gz \
  backend/.env \
  backend/settings/ \
  data/ \
  frontend/.env.local
```

### Recovery Procedure

```bash
# Restore database
psql multiagent_hive < backup.sql

# Restore application data
tar -xzf backup.tar.gz

# Restart services
docker-compose restart
```

## Scaling Strategies

### Horizontal Scaling

```yaml
# k8s-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: multiagent-hive-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: multiagent-hive-backend
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
kubectl patch deployment multiagent-hive-backend \
  --type='json' \
  -p='[{"op": "replace", "path": "/spec/template/spec/containers/0/resources/limits/cpu", "value": "2000m"}]'
```

## Security Hardening

### Network Security

```bash
# Configure firewall
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw --force enable
```

### Container Security

```yaml
# security-context.yaml
apiVersion: v1
kind: Pod
metadata:
  name: secure-hive
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    fsGroup: 2000
  containers:
  - name: backend
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop:
        - ALL
```

## Performance Optimization

### Database Optimization

```sql
-- Create indexes
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_agents_capability ON agents USING GIN(capabilities);

-- Optimize queries
EXPLAIN ANALYZE SELECT * FROM tasks WHERE status = 'pending';
```

### Caching Strategy

```env
# Redis cache
REDIS_URL=redis://localhost:6379
CACHE_TTL=3600
CACHE_SIZE=1000
```

### CDN Configuration

```bash
# CloudFront distribution
aws cloudfront create-distribution \
  --origin-domain-name your-domain.com \
  --default-cache-behavior '{"TargetOriginId": "your-origin", "ViewerProtocolPolicy": "redirect-to-https"}'
```

## Troubleshooting Deployment

### Common Issues

#### Container Won't Start
```bash
# Check logs
docker logs container_name

# Debug container
docker run -it --entrypoint /bin/bash your-image
```

#### Database Connection Issues
```bash
# Test connection
psql "postgresql://user:password@host/database"

# Check network connectivity
telnet host 5432
```

#### Performance Problems
```bash
# Monitor resources
top
iotop
nethogs

# Check application metrics
curl http://localhost:3001/metrics
```

### Health Checks

```bash
# Application health
curl http://localhost:3001/health

# Database health
psql -c "SELECT 1" database

# System health
systemctl status multiagent-hive
```

## Maintenance Procedures

### Rolling Updates

```bash
# Zero-downtime deployment
kubectl set image deployment/multiagent-hive-backend \
  backend=your-registry/multiagent-hive:v2.0.0

# Check rollout status
kubectl rollout status deployment/multiagent-hive-backend
```

### Backup Verification

```bash
# Test backup integrity
pg_restore --list backup.sql

# Verify data consistency
psql -c "SELECT COUNT(*) FROM agents" multiagent_hive
```

### Log Rotation

```bash
# Configure logrotate
cat > /etc/logrotate.d/multiagent-hive << EOF
/var/log/multiagent-hive/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 644 hive hive
}
EOF
```

## Next Steps

- **Configuration**: See [docs/configuration.md](configuration.md)
- **Security**: See [docs/security-hardening.md](security-hardening.md)
- **Performance**: See [docs/performance.md](performance.md)
- **Monitoring**: See [docs/observability.md](observability.md)