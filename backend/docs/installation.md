# Installation Guide

This guide covers all installation methods for the AI Orchestrator Hub, from local development to production deployment.

## System Requirements

### Minimum Requirements
- **CPU**: 2 cores (x86_64 or ARM64)
- **RAM**: 2GB
- **Storage**: 1GB free space
- **OS**: Linux, macOS, or Windows (with WSL2)

### Recommended Requirements
- **CPU**: 4+ cores with SIMD support
- **RAM**: 4GB
- **Storage**: 5GB free space
- **Network**: Stable internet connection

### High Performance Requirements
- **CPU**: 8+ cores with AVX2/AVX-512
- **RAM**: 8GB+
- **Storage**: 10GB+ SSD
- **GPU**: CUDA-compatible (optional, for neural acceleration)

## Local Installation

### Method 1: Direct Clone and Build

```bash
# Clone the repository
git clone https://github.com/do-ops885/ai-orchestrator-hub.git
cd ai-orchestrator-hub

# Backend installation
cd backend
cargo build --release

# Frontend installation (optional)
cd ../frontend
npm install
npm run build
```

### Method 2: Using Docker

```bash
# Clone the repository
git clone https://github.com/do-ops885/ai-orchestrator-hub.git
cd ai-orchestrator-hub

# Build and run with Docker Compose
docker-compose up --build

# Or build individual components
docker build -t ai-orchestrator-backend ./backend
docker build -t ai-orchestrator-frontend ./frontend
```

### Method 3: Development Setup

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Node.js (using nvm recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install 18
nvm use 18

# Clone and setup
git clone https://github.com/do-ops885/ai-orchestrator-hub.git
cd ai-orchestrator-hub

# Backend development setup
cd backend
cargo build

# Frontend development setup
cd ../frontend
npm install
```

## Package Manager Installation

### Cargo (Rust Package Manager)

```bash
# Install from crates.io (when published)
cargo install ai-orchestrator-hub

# Or build from source
cargo install --path .
```

### NPM (Node.js Package Manager)

```bash
# Install frontend dependencies
npm install ai-orchestrator-hub-frontend

# For development
npm install
npm run dev
```

## Docker Installation

### Using Docker Compose (Recommended)

```yaml
# docker-compose.yml
version: '3.8'
services:
  backend:
    build: ./backend
    ports:
      - "3001:3001"
    environment:
      - HIVE_SERVER__HOST=0.0.0.0
      - HIVE_DATABASE__URL=./data/hive_persistence.db
    volumes:
      - ./data:/app/data
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3001/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  frontend:
    build: ./frontend
    ports:
      - "3000:3000"
    depends_on:
      - backend
    environment:
      - REACT_APP_API_URL=http://localhost:3001
```

```bash
# Start the system
docker-compose up -d

# View logs
docker-compose logs -f

# Stop the system
docker-compose down
```

### Using Individual Docker Images

```bash
# Backend only
docker run -p 3001:3001 \
  -e HIVE_SERVER__HOST=0.0.0.0 \
  -v $(pwd)/data:/app/data \
  ai-orchestrator-hub-backend

# Frontend only
docker run -p 3000:3000 \
  -e REACT_APP_API_URL=http://localhost:3001 \
  ai-orchestrator-hub-frontend
```

### Docker Image Build

```dockerfile
# Backend Dockerfile
FROM rust:1.70-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/multiagent-hive /usr/local/bin/
EXPOSE 3001
CMD ["multiagent-hive"]
```

## Kubernetes Installation

### Using Helm Chart

```bash
# Add the repository
helm repo add ai-orchestrator https://charts.ai-orchestrator-hub.dev
helm repo update

# Install the chart
helm install ai-orchestrator ai-orchestrator/ai-orchestrator-hub \
  --set backend.replicas=3 \
  --set frontend.replicas=2
```

### Manual Kubernetes Deployment

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
          value: "/data/hive_persistence.db"
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
# Apply the configurations
kubectl apply -f backend-deployment.yaml
kubectl apply -f backend-service.yaml
```

## Cloud Installation

### AWS

#### Using EC2

```bash
# Launch EC2 instance
aws ec2 run-instances \
  --image-id ami-12345678 \
  --instance-type t3.medium \
  --key-name your-key-pair \
  --security-groups ai-orchestrator-sg

# Install dependencies on the instance
sudo yum update -y
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Deploy the application
git clone https://github.com/do-ops885/ai-orchestrator-hub.git
cd ai-orchestrator-hub/backend
cargo build --release
./target/release/multiagent-hive
```

#### Using ECS

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
      "image": "ai-orchestrator-hub-backend:latest",
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
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/ai-orchestrator-backend",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

### Google Cloud Platform

#### Using GKE

```bash
# Create GKE cluster
gcloud container clusters create ai-orchestrator \
  --num-nodes=3 \
  --machine-type=e2-medium

# Get credentials
gcloud container clusters get-credentials ai-orchestrator

# Deploy using kubectl
kubectl apply -f k8s/
```

#### Using Cloud Run

```bash
# Build and deploy to Cloud Run
gcloud run deploy ai-orchestrator-backend \
  --source . \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --port 3001 \
  --memory 2Gi \
  --cpu 2
```

### Microsoft Azure

#### Using AKS

```bash
# Create AKS cluster
az aks create \
  --resource-group ai-orchestrator-rg \
  --name ai-orchestrator-cluster \
  --node-count 3 \
  --node-vm-size Standard_DS2_v2

# Get credentials
az aks get-credentials \
  --resource-group ai-orchestrator-rg \
  --name ai-orchestrator-cluster

# Deploy the application
kubectl apply -f k8s/
```

#### Using Container Instances

```bash
# Create container instance
az container create \
  --resource-group ai-orchestrator-rg \
  --name ai-orchestrator-backend \
  --image ai-orchestrator-hub-backend:latest \
  --ports 3001 \
  --cpu 2 \
  --memory 4 \
  --environment-variables HIVE_SERVER__HOST=0.0.0.0
```

## Package Installation

### Debian/Ubuntu

```bash
# Add the repository
echo "deb [trusted=yes] https://packages.ai-orchestrator-hub.dev/debian/ stable main" | sudo tee /etc/apt/sources.list.d/ai-orchestrator.list

# Install the package
sudo apt update
sudo apt install ai-orchestrator-hub

# Start the service
sudo systemctl start ai-orchestrator-hub
sudo systemctl enable ai-orchestrator-hub
```

### Red Hat/CentOS

```bash
# Add the repository
sudo tee /etc/yum.repos.d/ai-orchestrator.repo << EOF
[ai-orchestrator]
name=AI Orchestrator Hub
baseurl=https://packages.ai-orchestrator-hub.dev/rpm/
enabled=1
gpgcheck=0
EOF

# Install the package
sudo yum install ai-orchestrator-hub

# Start the service
sudo systemctl start ai-orchestrator-hub
sudo systemctl enable ai-orchestrator-hub
```

### macOS

```bash
# Using Homebrew
brew tap ai-orchestrator-hub/tap
brew install ai-orchestrator-hub

# Or using MacPorts
sudo port install ai-orchestrator-hub
```

### Windows

```powershell
# Using Chocolatey
choco install ai-orchestrator-hub

# Using Scoop
scoop bucket add ai-orchestrator https://github.com/ai-orchestrator-hub/scoop-bucket
scoop install ai-orchestrator-hub
```

## Post-Installation Configuration

### Environment Setup

```bash
# Create environment file
cp .env.example .env

# Edit configuration
nano .env
```

### Database Setup

```bash
# Initialize SQLite database
cd backend
cargo run --bin init-db

# Or for PostgreSQL
createdb hive_db
psql hive_db < schema.sql
```

### Service Configuration

```bash
# Configure systemd service
sudo tee /etc/systemd/system/ai-orchestrator-hub.service << EOF
[Unit]
Description=AI Orchestrator Hub
After=network.target

[Service]
Type=simple
User=ai-orchestrator
Group=ai-orchestrator
WorkingDirectory=/opt/ai-orchestrator-hub
ExecStart=/opt/ai-orchestrator-hub/multiagent-hive
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# Enable and start the service
sudo systemctl daemon-reload
sudo systemctl enable ai-orchestrator-hub
sudo systemctl start ai-orchestrator-hub
```

### Firewall Configuration

```bash
# UFW (Ubuntu/Debian)
sudo ufw allow 3001/tcp
sudo ufw allow 3000/tcp  # Frontend
sudo ufw allow 22/tcp    # SSH

# firewalld (Red Hat/CentOS)
sudo firewall-cmd --permanent --add-port=3001/tcp
sudo firewall-cmd --permanent --add-port=3000/tcp
sudo firewall-cmd --reload
```

## Verification

### Health Check

```bash
# Check backend health
curl http://localhost:3001/health

# Check frontend (if running)
curl http://localhost:3000
```

### Service Status

```bash
# Systemd service status
sudo systemctl status ai-orchestrator-hub

# Docker container status
docker ps

# Kubernetes pod status
kubectl get pods
```

### Log Verification

```bash
# View application logs
sudo journalctl -u ai-orchestrator-hub -f

# Docker logs
docker logs ai-orchestrator-backend

# Kubernetes logs
kubectl logs -f deployment/ai-orchestrator-backend
```

## Troubleshooting Installation

### Common Issues

#### Build Failures

```bash
# Clean build artifacts
cargo clean
rm -rf target/

# Update Rust toolchain
rustup update

# Rebuild
cargo build
```

#### Permission Issues

```bash
# Fix permission issues
sudo chown -R $USER:$USER /opt/ai-orchestrator-hub
chmod +x /opt/ai-orchestrator-hub/multiagent-hive
```

#### Port Conflicts

```bash
# Check port usage
sudo netstat -tulpn | grep :3001

# Kill conflicting process
sudo kill -9 <PID>
```

#### Memory Issues

```bash
# Check available memory
free -h

# Adjust system limits
echo "vm.max_map_count=262144" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

## Next Steps

After successful installation:

1. **Configure the System**: Edit `.env` file with your settings
2. **Start the Services**: Use the appropriate method for your installation
3. **Verify Operation**: Check health endpoints and logs
4. **Create Your First Agent**: Follow the getting started guide
5. **Monitor Performance**: Set up monitoring and alerting

For detailed configuration options, see the [Configuration Guide](configuration.md).