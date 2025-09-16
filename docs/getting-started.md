# Getting Started Guide

This guide will walk you through setting up and running the AI Orchestrator Hub from scratch. By the end of this guide, you'll have a fully functional multiagent system running on your machine.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Detailed Setup](#detailed-setup)
- [First Agent](#first-agent)
- [First Task](#first-task)
- [Web Dashboard](#web-dashboard)
- [Advanced Configuration](#advanced-configuration)
- [Troubleshooting](#troubleshooting)
- [Next Steps](#next-steps)

## Prerequisites

### System Requirements

**Minimum Requirements:**
- **Operating System**: Linux, macOS, or Windows 10+
- **CPU**: 2-core processor (4+ cores recommended)
- **Memory**: 4GB RAM (8GB+ recommended)
- **Storage**: 5GB free disk space
- **Network**: Stable internet connection

**Recommended for Development:**
- **Operating System**: Linux or macOS
- **CPU**: 4+ core processor with AVX2 support
- **Memory**: 8GB+ RAM
- **Storage**: 10GB+ SSD storage
- **Network**: High-speed internet for AI integrations

### Software Dependencies

#### Required Software

1. **Rust**: Version 1.70 or later
   ```bash
   # Install Rust using rustup (recommended)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env

   # Verify installation
   rustc --version  # Should show 1.70+
   cargo --version  # Should show 1.70+
   ```

2. **Node.js**: Version 18 or later
   ```bash
   # Using Node Version Manager (recommended)
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
   source ~/.bashrc
   nvm install 18
   nvm use 18

   # Verify installation
   node --version  # Should show 18+
   npm --version   # Should show 8+
   ```

#### Optional Dependencies

1. **Docker**: For containerized deployment
   ```bash
   # Install Docker
   curl -fsSL https://get.docker.com -o get-docker.sh
   sudo sh get-docker.sh

   # Verify installation
   docker --version
   ```

2. **PostgreSQL**: For production database (SQLite used by default)
   ```bash
   # Install PostgreSQL
   sudo apt-get install postgresql postgresql-contrib  # Ubuntu/Debian
   # OR
   brew install postgresql  # macOS

   # Start PostgreSQL service
   sudo systemctl start postgresql  # Linux
   # OR
   brew services start postgresql   # macOS
   ```

## Quick Start

### One-Command Setup

If you have all prerequisites installed, you can get started with:

```bash
# Clone the repository
git clone https://github.com/do-ops885/ai-orchestrator-hub.git
cd ai-orchestrator-hub

# Quick setup script (if available)
./scripts/setup-dev.sh

# Or manual setup
cd backend && cargo build
cd ../frontend && npm install
```

### Basic Test Run

```bash
# Start the backend
cd backend
cargo run

# In another terminal, start the frontend
cd frontend
npm run dev

# Open your browser to http://localhost:3000
```

## Detailed Setup

### Step 1: Clone and Prepare

```bash
# Clone the repository
git clone https://github.com/do-ops885/ai-orchestrator-hub.git
cd ai-orchestrator-hub

# Verify the structure
ls -la
# Should show: backend/, frontend/, docs/, scripts/, etc.
```

### Step 2: Backend Setup

```bash
# Navigate to backend directory
cd backend

# Build the project (this may take a few minutes)
cargo build

# Optional: Build with release optimizations
cargo build --release

# Verify the build
ls -la target/debug/
# Should show the compiled binary
```

### Step 3: Frontend Setup

```bash
# Navigate to frontend directory
cd ../frontend

# Install dependencies
npm install

# Build the frontend
npm run build

# Optional: Start development server
npm run dev
```

### Step 4: Configuration

```bash
# Copy default configuration
cd ../backend
cp settings/default.toml settings/development.toml

# Edit configuration if needed
nano settings/development.toml
```

### Step 5: Database Setup

The system uses SQLite by default, which requires no additional setup. For production, you can configure PostgreSQL:

```bash
# Edit configuration for PostgreSQL
nano settings/development.toml

# Update database settings:
[database]
url = "postgresql://username:password@localhost/ai_orchestrator_hub"
max_connections = 10
```

## First Agent

### Creating Your First Agent

```bash
# Start the backend server
cd backend
cargo run
```

In another terminal:

```bash
# Create a simple worker agent
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "MyFirstAgent",
    "type": "worker",
    "capabilities": [
      {
        "name": "data_processing",
        "proficiency": 0.8,
        "learning_rate": 0.1
      }
    ]
  }'

# Expected response:
{
  "success": true,
  "data": {
    "agent_id": "uuid-generated-id",
    "message": "Agent created successfully"
  }
}
```

### Verifying Agent Creation

```bash
# List all agents
curl http://localhost:3001/api/agents

# Expected response:
{
  "success": true,
  "data": {
    "agents": [
      {
        "id": "uuid-generated-id",
        "name": "MyFirstAgent",
        "type": "worker",
        "state": "Active",
        "capabilities": [...],
        "performance_score": 0.8,
        "tasks_completed": 0
      }
    ]
  }
}
```

## First Task

### Creating Your First Task

```bash
# Create a simple data processing task
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Process sample data for analysis",
    "type": "data_processing",
    "priority": 1,
    "required_capabilities": [
      {
        "name": "data_processing",
        "min_proficiency": 0.7
      }
    ]
  }'

# Expected response:
{
  "success": true,
  "data": {
    "task_id": "uuid-task-id",
    "message": "Task created successfully"
  }
}
```

### Monitoring Task Execution

```bash
# Check task status
curl http://localhost:3001/api/tasks

# Expected response:
{
  "success": true,
  "data": {
    "tasks": [
      {
        "id": "uuid-task-id",
        "description": "Process sample data for analysis",
        "type": "data_processing",
        "priority": 1,
        "status": "completed",
        "assigned_agent": "uuid-generated-id",
        "execution_time_ms": 1500,
        "created_at": "2025-01-13T...",
        "completed_at": "2025-01-13T..."
      }
    ]
  }
}
```

## Web Dashboard

### Accessing the Dashboard

1. **Start the Frontend**:
   ```bash
   cd frontend
   npm run dev
   ```

2. **Open Browser**:
   - Navigate to `http://localhost:3000`
   - You should see the AI Orchestrator Hub dashboard

3. **Dashboard Features**:
   - **Agent Overview**: View all active agents and their status
   - **Task Monitor**: Track task execution in real-time
   - **System Metrics**: Monitor performance and resource usage
   - **Neural Processing**: View AI/ML processing status
   - **Real-time Updates**: WebSocket-powered live updates

### Dashboard Navigation

- **Home**: System overview and key metrics
- **Agents**: Agent management and monitoring
- **Tasks**: Task creation and monitoring
- **Metrics**: Detailed performance analytics
- **Settings**: Configuration management

## Advanced Configuration

### Environment Variables

Create a `.env` file in the backend directory:

```env
# Server Configuration
HIVE_SERVER__HOST=0.0.0.0
HIVE_SERVER__PORT=3001
HIVE_SERVER__CORS_ORIGINS='["http://localhost:3000"]'

# Database
HIVE_DATABASE__URL=./data/hive_persistence.db

# Logging
HIVE_LOGGING__LEVEL=info
HIVE_LOGGING__FORMAT=json

# Neural Processing
HIVE_NEURAL__ENABLE_ADVANCED_NEURAL=false

# Security
HIVE_SECURITY__JWT_SECRET=your-super-secret-key-change-this-in-production
HIVE_SECURITY__RATE_LIMIT_REQUESTS_PER_MINUTE=1000

# Performance
HIVE_PERFORMANCE__METRICS_COLLECTION_INTERVAL_MS=5000
HIVE_PERFORMANCE__CPU_WARNING_THRESHOLD=70.0
HIVE_PERFORMANCE__MEMORY_WARNING_THRESHOLD=80.0
```

### Feature Flags

#### Basic NLP (Default)

```bash
# Run with basic NLP features
cargo run
```

#### Advanced Neural Processing

```bash
# Enable advanced neural processing
cargo run --features advanced-neural

# Run neural comparison demo
cargo run --features advanced-neural --example neural_comparison
```

#### GPU Acceleration

```bash
# Enable GPU acceleration (requires CUDA)
cargo run --features advanced-neural,gpu-acceleration
```

### Database Configuration

#### SQLite (Default)

No additional configuration required. The database file will be created automatically at `./data/hive_persistence.db`.

#### PostgreSQL

```toml
[database]
url = "postgresql://username:password@localhost/ai_orchestrator_hub"
max_connections = 20
min_connections = 5
connection_timeout_seconds = 30
```

### Monitoring Configuration

```toml
[monitoring]
enabled = true
metrics_collection_interval_seconds = 5
alert_check_interval_seconds = 30
metrics_retention_days = 7

[monitoring.alerts]
cpu_threshold = 80.0
memory_threshold = 85.0
disk_threshold = 90.0
```

## Troubleshooting

### Common Issues

#### Backend Won't Start

**Problem**: Compilation errors or missing dependencies

**Solutions**:
```bash
# Clean and rebuild
cd backend
cargo clean
cargo build

# Update dependencies
cargo update

# Check Rust version
rustc --version
```

#### Frontend Build Fails

**Problem**: Node.js dependencies or build issues

**Solutions**:
```bash
# Clear node modules
cd frontend
rm -rf node_modules package-lock.json

# Reinstall dependencies
npm install

# Check Node.js version
node --version
```

#### WebSocket Connection Issues

**Problem**: Real-time updates not working

**Solutions**:
- Ensure backend is running on port 3001
- Check CORS configuration
- Verify firewall settings
- Check browser console for errors

#### Database Connection Issues

**Problem**: Unable to connect to database

**Solutions**:
```bash
# Check database file permissions
ls -la data/hive_persistence.db

# Reset database
rm data/hive_persistence.db
cargo run  # Will recreate database
```

#### Agent Creation Fails

**Problem**: Agents cannot be created

**Solutions**:
- Check API endpoint availability
- Verify JSON payload format
- Check server logs for errors
- Ensure sufficient system resources

### Getting Help

#### System Diagnostics

```bash
# Check system health
curl http://localhost:3001/health

# View system metrics
curl http://localhost:3001/metrics

# Check API availability
curl http://localhost:3001/
```

#### Log Analysis

```bash
# Enable debug logging
export RUST_LOG=debug
cargo run

# View structured logs
tail -f logs/ai-orchestrator-hub.log
```

#### Performance Monitoring

```bash
# Monitor system resources
top
df -h
free -h

# Check application performance
curl http://localhost:3001/api/resources
```

## Next Steps

### Learning More

1. **Explore Examples**:
   ```bash
   # Run neural processing examples
   cargo run --example neural_comparison
   cargo run --example agent_monitor_example

   # Run persistence examples
   cargo run --example persistence_demo
   ```

2. **Read Documentation**:
   - [API Documentation](api.md)
   - [Architecture Overview](ARCHITECTURE_OVERVIEW.md)
   - [Agent Lifecycle](agent-lifecycle.md)
   - [Troubleshooting Guide](troubleshooting.md)

3. **Join Community**:
   - GitHub Discussions
   - Issue tracking
   - Contributing guidelines

### Advanced Topics

1. **Custom Agent Development**:
   - Implement custom agent types
   - Add specialized capabilities
   - Integrate with external systems

2. **Neural Network Integration**:
   - Configure FANN networks
   - Implement custom neural models
   - GPU acceleration setup

3. **Production Deployment**:
   - Docker containerization
   - Kubernetes orchestration
   - Load balancing and scaling

4. **Security Hardening**:
   - Authentication and authorization
   - Input validation and sanitization
   - Audit logging and monitoring

### Development Workflow

```bash
# Development cycle
cargo build                    # Build changes
cargo test                     # Run tests
cargo clippy                   # Lint code
cargo fmt                      # Format code
cargo run                      # Test application

# Frontend development
npm run dev                    # Start dev server
npm test                       # Run tests
npm run lint                   # Lint code
npm run build                  # Build for production
```

### Contributing

Ready to contribute? See our [Contributing Guide](../CONTRIBUTING.md) for:
- Development setup
- Code standards
- Pull request process
- Issue reporting

---

Congratulations! You've successfully set up the AI Orchestrator Hub. The system is now running with your first agent and task. Explore the documentation and examples to learn more about the powerful capabilities of this multiagent orchestration platform.

For questions or issues, please check the [troubleshooting guide](troubleshooting.md) or open an issue on GitHub.
