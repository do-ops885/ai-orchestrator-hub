# Getting Started with AI Orchestrator Hub

This guide will help you get up and running with the AI Orchestrator Hub quickly. We'll cover installation, basic setup, creating your first agent and task, and troubleshooting common issues.

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust**: Version 1.70+ with Cargo
- **Node.js**: Version 18+ with npm (for frontend development)
- **System Requirements**:
  - Minimum: 2GB RAM, 2 CPU cores
  - Recommended: 4GB RAM, 4+ CPU cores
  - Optimal: 8GB+ RAM, 8+ CPU cores with SIMD support

### Installing Rust

```bash
# Install Rust using rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Installing Node.js

```bash
# Using Node Version Manager (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install 18
nvm use 18

# Verify installation
node --version
npm --version
```

## Quick Start

### 1. Clone and Setup

```bash
# Clone the repository
git clone https://github.com/do-ops885/ai-orchestrator-hub.git
cd ai-orchestrator-hub

# Backend setup
cd backend
cargo build

# Frontend setup (optional)
cd ../frontend
npm install
```

### 2. Run the System

```bash
# Start the backend server
cd backend
cargo run

# In another terminal, start the frontend (optional)
cd ../frontend
npm run dev
```

The system will be available at:
- **Backend API**: http://localhost:3001
- **Frontend Dashboard**: http://localhost:3000 (if running)
- **WebSocket**: ws://localhost:3001/ws
- **Health Check**: http://localhost:3001/health

### 3. Verify Installation

```bash
# Check if the server is running
curl http://localhost:3001/health

# Expected response:
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "2.0.0",
    "modules": {
      "coordinator": "healthy",
      "agent_management": "healthy",
      "task_management": "healthy"
    }
  }
}
```

## Your First Agent and Task

### Creating Your First Agent

Let's create a simple worker agent that can process data:

```bash
# Create an agent using the REST API
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "DataProcessor-1",
    "type": "worker",
    "capabilities": [
      {
        "name": "data_processing",
        "proficiency": 0.8,
        "learning_rate": 0.1
      }
    ]
  }'
```

**Expected Response:**
```json
{
  "success": true,
  "data": {
    "agent_id": "uuid-generated-id",
    "message": "Agent created successfully"
  }
}
```

### Creating Your First Task

Now let's create a task for the agent to execute:

```bash
# Create a task
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Process customer data for analytics",
    "type": "data_analysis",
    "priority": 2,
    "required_capabilities": [
      {
        "name": "data_processing",
        "min_proficiency": 0.7
      }
    ]
  }'
```

**Expected Response:**
```json
{
  "success": true,
  "data": {
    "task_id": "uuid-task-id",
    "message": "Task created successfully"
  }
}
```

### Monitoring Progress

Check the status of your agents and tasks:

```bash
# List all agents
curl http://localhost:3001/api/agents

# List all tasks
curl http://localhost:3001/api/tasks

# Get system status
curl http://localhost:3001/api/hive/status
```

## Configuration

### Basic Configuration

Create a `.env` file in the backend directory:

```env
# Server Configuration
HIVE_SERVER__HOST=0.0.0.0
HIVE_SERVER__PORT=3001
HIVE_SERVER__CORS_ORIGINS=http://localhost:3000

# Database
HIVE_DATABASE__URL=./data/hive_persistence.db

# Agent Configuration
HIVE_AGENTS__MAX_AGENTS=100
HIVE_AGENTS__DEFAULT_ENERGY=100.0

# Task Configuration
HIVE_TASKS__MAX_CONCURRENT_TASKS=50
HIVE_TASKS__TASK_TIMEOUT_SECS=300

# Logging
HIVE_LOGGING__LEVEL=info
```

### Advanced Features

To enable advanced neural processing:

```env
# Enable advanced neural features
HIVE_NEURAL__ENABLE_ADVANCED_NEURAL=true
HIVE_NEURAL__BATCH_SIZE=32
```

## Using the Frontend Dashboard

If you started the frontend server:

1. Open http://localhost:3000 in your browser
2. You'll see the main dashboard with:
   - **Agent Monitor**: Real-time agent status
   - **Task Manager**: Task creation and monitoring
   - **Hive Visualization**: 3D swarm representation
   - **Metrics Panel**: Performance metrics
   - **Resource Monitor**: System resource usage

### Frontend Development

```bash
cd frontend
npm install
npm run dev          # Development server
npm run build        # Production build
npm test            # Run tests
npm run lint        # Code linting
```

## Examples and Demos

The project includes several examples to help you understand the system:

### Running Examples

```bash
cd backend

# Simple verification demo
cargo run --example simple_verification_demo

# Neural processing comparison
cargo run --features advanced-neural --example neural_comparison

# Agent monitoring example
cargo run --example agent_monitor_example

# Persistence demo
cargo run --example persistence_demo
```

### Example Output

```bash
# Simple verification demo output
🤖 Starting Simple Verification Demo
📊 Creating verification agent...
✅ Agent created: uuid-1234-5678
📋 Creating verification task...
✅ Task created: uuid-abcd-efgh
⏳ Executing task...
✅ Task completed successfully
📈 Results: accuracy=0.95, processing_time=1250ms
```

## Troubleshooting

### Common Issues

#### Backend Won't Start

**Symptoms:** Server fails to start or crashes immediately

**Solutions:**
```bash
# Check Rust version
rustc --version  # Should be 1.70+

# Clean and rebuild
cd backend
cargo clean
cargo build

# Check for port conflicts
lsof -i :3001
```

#### Frontend Build Errors

**Symptoms:** npm install or npm run dev fails

**Solutions:**
```bash
# Clear node modules and reinstall
cd frontend
rm -rf node_modules package-lock.json
npm install

# Check Node.js version
node --version  # Should be 18+
```

#### WebSocket Connection Issues

**Symptoms:** Real-time updates not working

**Solutions:**
- Ensure backend is running on port 3001
- Check firewall settings for WebSocket connections
- Verify CORS configuration in production
- Check browser console for connection errors

#### Agent/Task Creation Fails

**Symptoms:** API returns validation errors

**Solutions:**
```bash
# Check API health
curl http://localhost:3001/health

# Verify JSON format
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{"name": "TestAgent", "type": "worker"}'

# Check server logs for detailed error messages
```

### Getting Help

- **Health Check Endpoint**: `GET /health` for system diagnostics
- **System Logs**: Enable debug logging with `RUST_LOG=debug`
- **API Documentation**: See `docs/api.md` for detailed endpoint information
- **GitHub Issues**: Report bugs at https://github.com/do-ops885/ai-orchestrator-hub/issues

### Performance Tuning

For better performance:

```env
# Increase resource limits
HIVE_AGENTS__MAX_AGENTS=500
HIVE_TASKS__MAX_CONCURRENT_TASKS=100

# Enable performance optimizations
HIVE_PERFORMANCE__CIRCUIT_BREAKER_ENABLED=true
HIVE_PERFORMANCE__AUTO_SCALING_ENABLED=true
```

## Next Steps

Now that you have the basics running, you can:

1. **Explore Advanced Features**:
   - Enable neural processing with `--features advanced-neural`
   - Set up monitoring dashboards
   - Configure persistence and recovery

2. **Learn More**:
   - Read the [Architecture Overview](system-architecture.md)
   - Explore the [API Documentation](api.md)
   - Check out [Configuration Guide](configuration.md)

3. **Development**:
   - Follow the [Contributing Guide](../CONTRIBUTING.md)
   - Run the test suite with `cargo test`
   - Set up your development environment

4. **Production Deployment**:
   - See [Deployment Guide](deployment.md)
   - Configure security and monitoring
   - Set up high availability

Happy orchestrating! 🚀