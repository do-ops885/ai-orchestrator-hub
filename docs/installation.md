# Installation Guide

This guide covers various installation methods for the AI Orchestrator Hub, from local development setup to production deployment.

## Table of Contents

- [System Requirements](#system-requirements)
- [Local Installation](#local-installation)
- [Docker Installation](#docker-installation)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Package Manager Installation](#package-manager-installation)
- [Cloud Deployment](#cloud-deployment)
- [Post-Installation](#post-installation)
- [Uninstallation](#uninstallation)

## System Requirements

### Minimum Requirements

| Component | Requirement | Notes |
|-----------|-------------|-------|
| **CPU** | 2 cores | 4+ cores recommended |
| **RAM** | 4GB | 8GB+ recommended |
| **Storage** | 5GB | 10GB+ for development |
| **OS** | Linux/macOS/Windows | Linux preferred for production |
| **Network** | Stable internet | Required for AI integrations |

### Recommended Requirements

| Component | Requirement | Notes |
|-----------|-------------|-------|
| **CPU** | 4+ cores with AVX2 | SIMD support for neural processing |
| **RAM** | 8GB+ | 16GB+ for advanced neural features |
| **Storage** | 20GB SSD | Fast storage for databases |
| **OS** | Ubuntu 20.04+ or macOS 12+ | Latest LTS versions |
| **Network** | 100Mbps+ | High-speed for real-time features |

### Software Dependencies

| Software | Version | Purpose |
|----------|---------|---------|
| **Rust** | 1.70+ | Backend compilation |
| **Node.js** | 18+ | Frontend build and runtime |
| **Docker** | 20.10+ | Container deployment (optional) |
| **PostgreSQL** | 13+ | Production database (optional) |
| **Redis** | 6+ | Caching and session storage (optional) |

## Local Installation

### Automated Setup

#### Using Setup Script (Recommended)

```bash
# Clone repository
git clone https://github.com/do-ops885/ai-orchestrator-hub.git
cd ai-orchestrator-hub

# Run automated setup
./scripts/setup-dev.sh

# Or manual setup
./scripts/install-dependencies.sh
./scripts/setup-dev.sh
```

#### Manual Setup

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. Install Node.js
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install 18
nvm use 18

# 3. Clone and build
git clone https://github.com/do-ops885/ai-orchestrator-hub.git
cd ai-orchestrator-hub

# 4. Build backend
cd backend
cargo build

# 5. Build frontend
cd ../frontend
npm install
npm run build
```

### Development Environment

#### Backend Development Setup

```bash
cd backend

# Install development tools
cargo install cargo-watch
cargo install cargo-expand
cargo install cargo-flamegraph

# Build with development features
cargo build --features advanced-neural

# Run with hot reload
cargo watch -x run
```

#### Frontend Development Setup

```bash
cd frontend

# Install development dependencies
npm install

# Start development server with hot reload
npm run dev

# Run tests in watch mode
npm run test:watch
```

### Production Build

```bash
# Backend production build
cd backend
cargo build --release

# Frontend production build
cd frontend
npm run build
npm run export  # For static hosting
```

## Docker Installation

### Using Pre-built Images

```bash
# Pull latest images
docker pull ghcr.io/do-ops885/ai-orchestrator-hub/backend:latest
docker pull ghcr.io/do-ops885/ai-orchestrator-hub/frontend:latest

# Run with docker-compose
docker-compose up -d
```

### Building from Source

#### Backend Docker Image

```dockerfile
# Dockerfile.backend
FROM rust:1.70-slim as builder

WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/multiagent-hive /usr/local/bin/

EXPOSE 3001
CMD ["multiagent-hive"]
```

#### Frontend Docker Image

```dockerfile
# Dockerfile.frontend
FROM node:18-alpine as builder

WORKDIR /app
COPY frontend/package*.json ./
RUN npm ci

COPY frontend/ .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/out /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### Docker Compose Setup

```yaml
# docker-compose.yml
version: '3.8'

services:
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    ports:
      - "3001:3001"
    environment:
      - HIVE_SERVER__HOST=0.0.0.0
      - HIVE_DATABASE__URL=./data/hive.db
    volumes:
      - ./data:/app/data
    depends_on:
      - postgres
      - redis

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    ports:
      - "3000:80"
    depends_on:
      - backend

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: ai_orchestrator_hub
      POSTGRES_USER: hive
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

volumes:
  postgres_data:
```

### Running with Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Scale services
docker-compose up -d --scale backend=3

# Stop services
docker-compose down
```

## Kubernetes Deployment

### Prerequisites

```bash
# Install kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
chmod +x kubectl
sudo mv kubectl /usr/local/bin/

# Install Helm
curl https://get.helm.sh/helm-v3.12.0-linux-amd64.tar.gz -o helm.tar.gz
tar -zxvf helm.tar.gz
sudo mv linux-amd64/helm /usr/local/bin/

# Configure kubectl
kubectl config set-cluster my-cluster --server=https://cluster.example.com
```

### Using Helm Chart

```bash
# Add repository
helm repo add ai-orchestrator-hub https://do-ops885.github.io/ai-orchestrator-hub
helm repo update

# Install with default values
helm install ai-orchestrator-hub ai-orchestrator-hub/ai-orchestrator-hub

# Install with custom values
helm install ai-orchestrator-hub ai-orchestrator-hub/ai-orchestrator-hub \
  --values custom-values.yaml
```

### Manual Kubernetes Deployment

#### Namespace

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: ai-orchestrator-hub
```

#### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: hive-config
  namespace: ai-orchestrator-hub
data:
  HIVE_SERVER__HOST: "0.0.0.0"
  HIVE_SERVER__PORT: "3001"
  HIVE_DATABASE__URL: "postgresql://hive:password@postgres:5432/hive"
  HIVE_LOGGING__LEVEL: "info"
```

#### Secrets

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: hive-secrets
  namespace: ai-orchestrator-hub
type: Opaque
data:
  jwt-secret: <base64-encoded-secret>
  db-password: <base64-encoded-password>
```

#### Backend Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hive-backend
  namespace: ai-orchestrator-hub
spec:
  replicas: 3
  selector:
    matchLabels:
      app: hive-backend
  template:
    metadata:
      labels:
        app: hive-backend
    spec:
      containers:
      - name: backend
        image: ghcr.io/do-ops885/ai-orchestrator-hub/backend:latest
        ports:
        - containerPort: 3001
        envFrom:
        - configMapRef:
            name: hive-config
        - secretRef:
            name: hive-secrets
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

#### Frontend Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hive-frontend
  namespace: ai-orchestrator-hub
spec:
  replicas: 2
  selector:
    matchLabels:
      app: hive-frontend
  template:
    metadata:
      labels:
        app: hive-frontend
    spec:
      containers:
      - name: frontend
        image: ghcr.io/do-ops885/ai-orchestrator-hub/frontend:latest
        ports:
        - containerPort: 80
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 200m
            memory: 256Mi
```

#### Services

```yaml
apiVersion: v1
kind: Service
metadata:
  name: hive-backend
  namespace: ai-orchestrator-hub
spec:
  selector:
    app: hive-backend
  ports:
  - port: 3001
    targetPort: 3001
  type: ClusterIP

---
apiVersion: v1
kind: Service
metadata:
  name: hive-frontend
  namespace: ai-orchestrator-hub
spec:
  selector:
    app: hive-frontend
  ports:
  - port: 80
    targetPort: 80
  type: LoadBalancer
```

#### Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: hive-ingress
  namespace: ai-orchestrator-hub
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
  - host: hive.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: hive-frontend
            port:
              number: 80
      - path: /api
        pathType: Prefix
        backend:
          service:
            name: hive-backend
            port:
              number: 3001
```

### Database Setup

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: ai-orchestrator-hub
spec:
  serviceName: postgres
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15
        ports:
        - containerPort: 5432
        env:
        - name: POSTGRES_DB
          value: hive
        - name: POSTGRES_USER
          value: hive
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
  volumeClaimTemplates:
  - metadata:
      name: postgres-storage
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 50Gi
```

## Package Manager Installation

### Using Cargo (Rust)

```bash
# Install from crates.io (when published)
cargo install ai-orchestrator-hub

# Install specific version
cargo install ai-orchestrator-hub --version 0.1.0

# Install from git
cargo install --git https://github.com/do-ops885/ai-orchestrator-hub.git
```

### Using npm (Frontend Only)

```bash
# Install frontend globally
npm install -g @ai-orchestrator-hub/frontend

# Run the dashboard
hive-dashboard
```

### Using Homebrew (macOS)

```bash
# Add tap
brew tap do-ops885/ai-orchestrator-hub

# Install backend
brew install ai-orchestrator-hub

# Install frontend
brew install ai-orchestrator-hub-frontend

# Start services
brew services start ai-orchestrator-hub
```

### Using apt (Ubuntu/Debian)

```bash
# Add repository
echo "deb [trusted=yes] https://do-ops885.github.io/ai-orchestrator-hub/deb ./" | sudo tee /etc/apt/sources.list.d/ai-orchestrator-hub.list

# Update package list
sudo apt update

# Install backend
sudo apt install ai-orchestrator-hub

# Install frontend
sudo apt install ai-orchestrator-hub-frontend
```

### Using yum/dnf (RHEL/CentOS)

```bash
# Add repository
sudo dnf config-manager --add-repo https://do-ops885.github.io/ai-orchestrator-hub/rpm/ai-orchestrator-hub.repo

# Install backend
sudo dnf install ai-orchestrator-hub

# Install frontend
sudo dnf install ai-orchestrator-hub-frontend
```

## Cloud Deployment

### AWS Deployment

#### Using Elastic Beanstalk

```bash
# Install EB CLI
pip install awsebcli

# Initialize application
eb init ai-orchestrator-hub

# Create environment
eb create production

# Deploy
eb deploy
```

#### Using ECS

```bash
# Create cluster
aws ecs create-cluster --cluster-name hive-cluster

# Register task definition
aws ecs register-task-definition --cli-input-json file://task-definition.json

# Create service
aws ecs create-service \
  --cluster hive-cluster \
  --service-name hive-service \
  --task-definition hive-task \
  --desired-count 3
```

### Google Cloud Platform

#### Using Cloud Run

```bash
# Build and deploy backend
gcloud run deploy hive-backend \
  --source ./backend \
  --platform managed \
  --port 3001 \
  --allow-unauthenticated

# Build and deploy frontend
gcloud run deploy hive-frontend \
  --source ./frontend \
  --platform managed \
  --port 3000 \
  --allow-unauthenticated
```

#### Using GKE

```bash
# Create cluster
gcloud container clusters create hive-cluster --num-nodes=3

# Get credentials
gcloud container clusters get-credentials hive-cluster

# Deploy using kubectl (see Kubernetes section above)
kubectl apply -f k8s/
```

### Microsoft Azure

#### Using Container Instances

```bash
# Create resource group
az group create --name hive-rg --location eastus

# Create container instance
az container create \
  --resource-group hive-rg \
  --name hive-backend \
  --image ghcr.io/do-ops885/ai-orchestrator-hub/backend:latest \
  --ports 3001 \
  --environment-variables HIVE_SERVER__HOST=0.0.0.0
```

#### Using AKS

```bash
# Create AKS cluster
az aks create --resource-group hive-rg --name hive-cluster --node-count 3

# Get credentials
az aks get-credentials --resource-group hive-rg --name hive-cluster

# Deploy using kubectl (see Kubernetes section above)
kubectl apply -f k8s/
```

## Post-Installation

### Initial Configuration

```bash
# Generate JWT secret
openssl rand -hex 32

# Configure environment
cp .env.example .env
nano .env

# Initialize database
./scripts/init-db.sh

# Create admin user
./scripts/create-admin.sh
```

### Service Management

#### Systemd Service

```ini
# /etc/systemd/system/ai-orchestrator-hub.service
[Unit]
Description=AI Orchestrator Hub
After=network.target

[Service]
Type=simple
User=hive
Group=hive
ExecStart=/usr/local/bin/multiagent-hive
Restart=always
RestartSec=5
Environment=HIVE_SERVER__PORT=3001
Environment=HIVE_DATABASE__URL=/var/lib/hive/hive.db

[Install]
WantedBy=multi-user.target
```

```bash
# Enable and start service
sudo systemctl enable ai-orchestrator-hub
sudo systemctl start ai-orchestrator-hub

# Check status
sudo systemctl status ai-orchestrator-hub

# View logs
sudo journalctl -u ai-orchestrator-hub -f
```

#### Docker Service Management

```bash
# Start services
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f backend

# Restart services
docker-compose restart

# Update services
docker-compose pull
docker-compose up -d
```

### Security Setup

```bash
# Generate SSL certificates
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365

# Configure firewall
sudo ufw allow 3001/tcp
sudo ufw allow 3000/tcp
sudo ufw allow 22/tcp

# Set up log rotation
sudo logrotate /etc/logrotate.d/ai-orchestrator-hub
```

### Monitoring Setup

```bash
# Install monitoring
./scripts/setup-monitoring.sh

# Configure alerts
./scripts/configure-alerts.sh

# Set up log aggregation
./scripts/setup-logging.sh
```

## Verification

### Health Checks

```bash
# Backend health check
curl http://localhost:3001/health

# Frontend health check
curl http://localhost:3000/api/health

# Database connectivity
./scripts/test-db-connection.sh

# WebSocket connectivity
./scripts/test-websocket.sh
```

### Functional Tests

```bash
# Run integration tests
cd backend
cargo test --test integration_tests

# Test API endpoints
./scripts/test-api.sh

# Performance testing
./scripts/benchmark.sh
```

## Uninstallation

### Local Uninstallation

```bash
# Stop services
pkill -f multiagent-hive
pkill -f "npm.*dev"

# Remove data
rm -rf ./data
rm -rf ./logs

# Remove dependencies
cargo uninstall ai-orchestrator-hub
npm uninstall -g @ai-orchestrator-hub/frontend
```

### Docker Uninstallation

```bash
# Stop and remove containers
docker-compose down

# Remove images
docker rmi ghcr.io/do-ops885/ai-orchestrator-hub/backend:latest
docker rmi ghcr.io/do-ops885/ai-orchestrator-hub/frontend:latest

# Remove volumes
docker volume rm $(docker volume ls -q | grep hive)
```

### Kubernetes Uninstallation

```bash
# Delete deployments
kubectl delete -f k8s/

# Delete namespace
kubectl delete namespace ai-orchestrator-hub

# Clean up persistent volumes
kubectl delete pvc --all -n ai-orchestrator-hub
```

### Systemd Uninstallation

```bash
# Stop and disable service
sudo systemctl stop ai-orchestrator-hub
sudo systemctl disable ai-orchestrator-hub

# Remove service file
sudo rm /etc/systemd/system/ai-orchestrator-hub.service

# Reload systemd
sudo systemctl daemon-reload

# Remove application files
sudo rm -rf /usr/local/bin/multiagent-hive
sudo rm -rf /var/lib/hive
sudo userdel hive
```

### Package Manager Uninstallation

```bash
# Using apt
sudo apt remove ai-orchestrator-hub ai-orchestrator-hub-frontend

# Using yum/dnf
sudo dnf remove ai-orchestrator-hub ai-orchestrator-hub-frontend

# Using Homebrew
brew uninstall ai-orchestrator-hub ai-orchestrator-hub-frontend
```

## Troubleshooting Installation

### Common Issues

#### Build Failures

```bash
# Clear build cache
cd backend
cargo clean
cargo build

# Update Rust
rustup update

# Check dependencies
cargo tree
```

#### Permission Issues

```bash
# Fix permissions
sudo chown -R $USER:$USER /path/to/ai-orchestrator-hub

# Add user to docker group
sudo usermod -aG docker $USER
```

#### Network Issues

```bash
# Check port availability
netstat -tlnp | grep :3001

# Test connectivity
curl -v http://localhost:3001/health

# Check firewall
sudo ufw status
```

#### Database Issues

```bash
# Test database connection
psql -h localhost -U hive -d ai_orchestrator_hub

# Reset database
./scripts/reset-db.sh

# Check database logs
tail -f /var/log/postgresql/postgresql-*.log
```

### Getting Help

- **Documentation**: Check [troubleshooting.md](troubleshooting.md)
- **GitHub Issues**: Search existing issues or create new ones
- **Community**: Join GitHub Discussions for community support
- **Logs**: Check application logs for detailed error information

---

This installation guide covers all major deployment scenarios. For specific environment requirements or advanced configurations, please refer to the [configuration documentation](configuration.md) or open an issue on GitHub.
