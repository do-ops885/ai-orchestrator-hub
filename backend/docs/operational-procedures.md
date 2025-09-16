# Operational Procedures

This guide covers the standard operating procedures for deploying, maintaining, and managing the AI Orchestrator Hub system in production environments.

## Pre-Deployment Preparation

### System Requirements Check

#### Hardware Requirements
```bash
# Minimum requirements
CPU: 4 cores
RAM: 8 GB
Storage: 50 GB SSD
Network: 1 Gbps

# Recommended for production
CPU: 8+ cores
RAM: 16+ GB
GPU: NVIDIA GPU (optional, for neural acceleration)
Storage: 100+ GB SSD
Network: 10 Gbps
```

#### Software Dependencies
```bash
# Required packages
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    sqlite3 \
    postgresql-client \
    curl \
    jq

# Optional: GPU support
sudo apt-get install -y nvidia-cuda-toolkit
```

#### Network Configuration
```bash
# Firewall configuration
sudo ufw allow 3001/tcp  # Main API port
sudo ufw allow 5432/tcp  # PostgreSQL (if external)
sudo ufw allow 6379/tcp  # Redis (if used for caching)

# SELinux/AppArmor configuration (if applicable)
sudo setsebool -P httpd_can_network_connect 1
```

### Environment Setup

#### Directory Structure
```bash
# Create application directory
sudo mkdir -p /opt/ai-orchestrator-hub
sudo chown -R ai-orchestrator:ai-orchestrator /opt/ai-orchestrator-hub

# Create data directories
sudo mkdir -p /var/lib/ai-orchestrator-hub/data
sudo mkdir -p /var/log/ai-orchestrator-hub
sudo chown -R ai-orchestrator:ai-orchestrator /var/lib/ai-orchestrator-hub
sudo chown -R ai-orchestrator:ai-orchestrator /var/log/ai-orchestrator-hub
```

#### User and Permissions
```bash
# Create dedicated user
sudo useradd -r -s /bin/false ai-orchestrator

# Set proper permissions
sudo chown -R ai-orchestrator:ai-orchestrator /opt/ai-orchestrator-hub
sudo chmod 755 /opt/ai-orchestrator-hub
```

## Deployment Procedures

### Method 1: Binary Deployment

#### Download and Install
```bash
# Download latest release
wget https://github.com/do-ops885/ai-orchestrator-hub/releases/latest/download/ai-orchestrator-hub-linux-x64.tar.gz

# Extract and install
tar -xzf ai-orchestrator-hub-linux-x64.tar.gz
sudo mv ai-orchestrator-hub /opt/ai-orchestrator-hub/
sudo chown ai-orchestrator:ai-orchestrator /opt/ai-orchestrator-hub/ai-orchestrator-hub
sudo chmod +x /opt/ai-orchestrator-hub/ai-orchestrator-hub
```

#### Configuration Setup
```bash
# Copy configuration template
sudo cp /opt/ai-orchestrator-hub/settings/default.toml /opt/ai-orchestrator-hub/settings/production.toml

# Edit production configuration
sudo vi /opt/ai-orchestrator-hub/settings/production.toml
```

#### Systemd Service Setup
```ini
# /etc/systemd/system/ai-orchestrator-hub.service
[Unit]
Description=AI Orchestrator Hub
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=ai-orchestrator
Group=ai-orchestrator
WorkingDirectory=/opt/ai-orchestrator-hub
ExecStart=/opt/ai-orchestrator-hub/ai-orchestrator-hub --config settings/production.toml
Restart=always
RestartSec=5
Environment=HIVE_ENV=production
Environment=RUST_LOG=info
StandardOutput=journal
StandardError=journal
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

#### Service Management
```bash
# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable ai-orchestrator-hub
sudo systemctl start ai-orchestrator-hub

# Check status
sudo systemctl status ai-orchestrator-hub
sudo journalctl -u ai-orchestrator-hub -f
```

### Method 2: Docker Deployment

#### Dockerfile
```dockerfile
FROM rust:1.70-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ai-orchestrator-hub /usr/local/bin/

EXPOSE 3001
USER ai-orchestrator
CMD ["ai-orchestrator-hub"]
```

#### Docker Compose Setup
```yaml
# docker-compose.yml
version: '3.8'

services:
  ai-orchestrator-hub:
    build: .
    ports:
      - "3001:3001"
    environment:
      - HIVE_ENV=production
      - DATABASE_URL=postgresql://user:password@postgres:5432/hive
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs
    restart: unless-stopped

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: hive
      POSTGRES_USER: ai_orchestrator
      POSTGRES_PASSWORD: secure_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
```

#### Docker Deployment Commands
```bash
# Build and deploy
docker-compose build
docker-compose up -d

# Check logs
docker-compose logs -f ai-orchestrator-hub

# Scale services
docker-compose up -d --scale ai-orchestrator-hub=3
```

### Method 3: Kubernetes Deployment

#### Kubernetes Manifests
```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ai-orchestrator-hub
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ai-orchestrator-hub
  template:
    metadata:
      labels:
        app: ai-orchestrator-hub
    spec:
      containers:
      - name: ai-orchestrator-hub
        image: your-registry/ai-orchestrator-hub:latest
        ports:
        - containerPort: 3001
        env:
        - name: HIVE_ENV
          value: "production"
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

#### Kubernetes Deployment
```bash
# Apply manifests
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f configmap.yaml

# Check deployment
kubectl get pods -l app=ai-orchestrator-hub
kubectl logs -l app=ai-orchestrator-hub

# Scale deployment
kubectl scale deployment ai-orchestrator-hub --replicas=5
```

## Configuration Management

### Environment Variables
```bash
# Server configuration
export HIVE_SERVER__HOST=0.0.0.0
export HIVE_SERVER__PORT=3001
export HIVE_SERVER__WORKERS=4

# Database configuration
export DATABASE_URL=postgresql://user:password@localhost:5432/hive_db
export DATABASE_MAX_CONNECTIONS=20

# Neural processing
export HIVE_NEURAL__MODE=advanced
export HIVE_NEURAL__GPU_ENABLED=true
export HIVE_NEURAL__LEARNING_RATE=0.01

# Security
export JWT_SECRET=your-256-bit-secret-here
export API_KEY=your-api-key-here

# Monitoring
export METRICS_ENABLED=true
export LOG_LEVEL=info
export SENTRY_DSN=https://your-sentry-dsn
```

### Configuration File (TOML)
```toml
# settings/production.toml
[server]
host = "0.0.0.0"
port = 3001
workers = 4
cors_origins = ["https://yourdomain.com"]

[database]
url = "postgresql://user:password@localhost:5432/hive_db"
max_connections = 20
connection_timeout_seconds = 30

[neural]
mode = "advanced"
gpu_enabled = true
learning_rate = 0.01
max_agents = 1000

[security]
jwt_secret = "your-256-bit-secret-here"
rate_limit_requests_per_minute = 1000
audit_logging_enabled = true

[monitoring]
metrics_collection_interval_ms = 30000
alert_check_interval_ms = 60000
log_level = "info"

[performance]
memory_limit_mb = 4096
cpu_cores = "auto"
circuit_breaker_failure_threshold = 5
performance_optimization_enabled = true
```

## Startup and Shutdown Procedures

### Normal Startup
```bash
# Systemd service
sudo systemctl start ai-orchestrator-hub

# Docker
docker-compose up -d

# Kubernetes
kubectl scale deployment ai-orchestrator-hub --replicas=3

# Manual startup
cd /opt/ai-orchestrator-hub
./ai-orchestrator-hub --config settings/production.toml
```

### Graceful Shutdown
```bash
# Systemd service
sudo systemctl stop ai-orchestrator-hub

# Docker
docker-compose down

# Kubernetes
kubectl scale deployment ai-orchestrator-hub --replicas=0

# Manual shutdown (send SIGTERM)
kill -TERM $(pgrep ai-orchestrator-hub)
```

### Emergency Shutdown
```bash
# Force immediate shutdown
sudo systemctl kill ai-orchestrator-hub

# Docker force stop
docker-compose down --timeout 0

# Manual force kill
kill -KILL $(pgrep ai-orchestrator-hub)
```

## Monitoring and Alerting

### Health Checks
```bash
# Basic health check
curl -f http://localhost:3001/health

# Detailed health with metrics
curl http://localhost:3001/health?detailed=true

# System metrics
curl http://localhost:3001/metrics

# Resource usage
curl http://localhost:3001/api/resources
```

### Log Monitoring
```bash
# Systemd logs
sudo journalctl -u ai-orchestrator-hub -f

# Docker logs
docker-compose logs -f ai-orchestrator-hub

# Kubernetes logs
kubectl logs -f deployment/ai-orchestrator-hub

# Application logs
tail -f /var/log/ai-orchestrator-hub/app.log
```

### Alert Configuration
```bash
# CPU usage alert (>80%)
# Memory usage alert (>85%)
# Disk usage alert (>90%)
# Agent failure rate (>5%)
# Task failure rate (>10%)
# Response time degradation (>500ms)
```

### Monitoring Dashboard Setup
```bash
# Prometheus configuration
# Grafana dashboard setup
# Alert manager configuration
# Custom metrics exporters
```

## Backup and Recovery

### Database Backup
```bash
# PostgreSQL backup
pg_dump -h localhost -U ai_orchestrator hive_db > backup_$(date +%Y%m%d_%H%M%S).sql

# SQLite backup (if used)
sqlite3 /var/lib/ai-orchestrator-hub/data/hive.db ".backup backup.db"

# Automated backup script
#!/bin/bash
BACKUP_DIR="/var/backups/ai-orchestrator-hub"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR
pg_dump -h localhost -U ai_orchestrator hive_db > $BACKUP_DIR/backup_$DATE.sql
find $BACKUP_DIR -name "backup_*.sql" -mtime +30 -delete
```

### Configuration Backup
```bash
# Configuration files
tar -czf config_backup_$(date +%Y%m%d).tar.gz /opt/ai-orchestrator-hub/settings/

# Environment variables
env | grep ^HIVE_ > environment_backup_$(date +%Y%m%d).env
```

### Full System Backup
```bash
# Complete backup script
#!/bin/bash
BACKUP_ROOT="/var/backups/ai-orchestrator-hub"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="$BACKUP_ROOT/$DATE"

# Create backup directory
mkdir -p $BACKUP_DIR

# Stop service for consistent backup
sudo systemctl stop ai-orchestrator-hub

# Database backup
pg_dump -h localhost -U ai_orchestrator hive_db > $BACKUP_DIR/database.sql

# Data directory backup
tar -czf $BACKUP_DIR/data.tar.gz /var/lib/ai-orchestrator-hub/data/

# Configuration backup
tar -czf $BACKUP_DIR/config.tar.gz /opt/ai-orchestrator-hub/settings/

# Log backup (optional)
tar -czf $BACKUP_DIR/logs.tar.gz /var/log/ai-orchestrator-hub/

# Restart service
sudo systemctl start ai-orchestrator-hub

# Create backup manifest
echo "Backup completed: $DATE" > $BACKUP_DIR/manifest.txt
echo "Database: $(stat -c%s $BACKUP_DIR/database.sql) bytes" >> $BACKUP_DIR/manifest.txt
echo "Data: $(stat -c%s $BACKUP_DIR/data.tar.gz) bytes" >> $BACKUP_DIR/manifest.txt

# Cleanup old backups (keep last 7 days)
find $BACKUP_ROOT -maxdepth 1 -type d -mtime +7 -exec rm -rf {} \;
```

### Recovery Procedures
```bash
# Database recovery
sudo systemctl stop ai-orchestrator-hub
psql -h localhost -U ai_orchestrator hive_db < backup.sql
sudo systemctl start ai-orchestrator-hub

# Full system recovery
sudo systemctl stop ai-orchestrator-hub
tar -xzf config_backup.tar.gz -C /
tar -xzf data_backup.tar.gz -C /
sudo systemctl start ai-orchestrator-hub
```

## Maintenance Procedures

### Regular Maintenance Tasks

#### Daily Tasks
```bash
# Health check verification
curl -f http://localhost:3001/health

# Log rotation check
logrotate -f /etc/logrotate.d/ai-orchestrator-hub

# Disk space monitoring
df -h /var/lib/ai-orchestrator-hub

# Process monitoring
ps aux | grep ai-orchestrator-hub
```

#### Weekly Tasks
```bash
# Database maintenance
vacuumdb -h localhost -U ai_orchestrator --analyze hive_db

# Backup verification
# Test restore from backup
# Review system metrics trends
# Check for security updates
```

#### Monthly Tasks
```bash
# Full system backup
# Performance benchmark comparison
# Dependency updates review
# Configuration optimization
# Documentation updates
```

### Performance Optimization

#### Memory Management
```bash
# Monitor memory usage
curl http://localhost:3001/api/resources | jq '.data.system_resources.memory_usage'

# Adjust memory limits
# Implement memory pooling
# Configure garbage collection
```

#### CPU Optimization
```bash
# Monitor CPU usage
curl http://localhost:3001/metrics | jq '.data.current_metrics.system.cpu_usage_percent'

# Adjust worker threads
# Optimize async operations
# Profile performance bottlenecks
```

#### Database Optimization
```bash
# Analyze query performance
EXPLAIN ANALYZE SELECT * FROM tasks WHERE status = 'pending';

# Add indexes as needed
CREATE INDEX CONCURRENTLY idx_tasks_status_created ON tasks(status, created_at);

# Vacuum and reindex
vacuumdb -h localhost -U ai_orchestrator --full --analyze hive_db
reindexdb -h localhost -U ai_orchestrator hive_db
```

## Security Procedures

### Access Control
```bash
# User management
sudo useradd -m operator
sudo usermod -aG ai-orchestrator operator

# SSH key setup
ssh-keygen -t ed25519
ssh-copy-id operator@server

# Sudo configuration
echo "operator ALL=(ALL) NOPASSWD: /usr/bin/systemctl * ai-orchestrator-hub" >> /etc/sudoers.d/operator
```

### Security Updates
```bash
# System updates
sudo apt-get update && sudo apt-get upgrade

# Rust security advisories
cargo audit

# Dependency updates
cargo update

# Security scanning
# Run vulnerability scans
# Review security logs
```

### Incident Response
```bash
# Security incident checklist
1. Isolate affected systems
2. Preserve evidence (logs, memory dumps)
3. Assess impact and scope
4. Contain the incident
5. Eradicate the threat
6. Recover systems
7. Post-incident analysis
8. Update security measures
```

## Troubleshooting Procedures

### Common Issues and Solutions

#### Service Won't Start
```bash
# Check configuration
sudo -u ai-orchestrator /opt/ai-orchestrator-hub/ai-orchestrator-hub --config settings/production.toml --check

# Check dependencies
ldd /opt/ai-orchestrator-hub/ai-orchestrator-hub

# Check logs
sudo journalctl -u ai-orchestrator-hub -n 50
```

#### High Memory Usage
```bash
# Check memory usage
ps aux --sort=-%mem | head

# Analyze memory leaks
valgrind --tool=massif ./ai-orchestrator-hub

# Adjust configuration
# memory_limit_mb = 2048
```

#### Database Connection Issues
```bash
# Test database connection
psql -h localhost -U ai_orchestrator -d hive_db -c "SELECT 1;"

# Check connection pool
curl http://localhost:3001/metrics | jq '.data.current_metrics.database'

# Restart database service
sudo systemctl restart postgresql
```

#### Performance Degradation
```bash
# Profile application
cargo flamegraph --bin ai-orchestrator-hub

# Check system resources
top -p $(pgrep ai-orchestrator-hub)

# Analyze slow queries
# Check network latency
# Review recent changes
```

## Emergency Procedures

### System Outage Response
```bash
# Immediate actions
1. Assess system status
2. Check monitoring alerts
3. Review recent changes
4. Attempt service restart
5. Check dependencies (database, network)
6. Contact on-call engineer if needed

# Communication
- Update status page
- Notify stakeholders
- Provide ETA if known
- Document incident details
```

### Data Loss Recovery
```bash
# Assess data loss scope
# Restore from backup
# Verify data integrity
# Update affected systems
# Communicate with users
# Implement preventive measures
```

### Security Breach Response
```bash
# Isolate systems
# Preserve evidence
# Notify security team
# Assess damage
# Contain breach
# Recover systems
# Update security policies
```

This operational procedures guide provides comprehensive instructions for managing the AI Orchestrator Hub in production environments. Regular review and updates to these procedures are essential for maintaining system reliability and security.