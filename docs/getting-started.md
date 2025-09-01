# Getting Started

This guide will walk you through setting up and running the Multiagent Hive System from scratch.

## Prerequisites

Before you begin, ensure you have the following installed:

### System Requirements
- **Operating System**: Linux, macOS, or Windows (WSL recommended)
- **Memory**: Minimum 2GB RAM, Recommended 4GB+ RAM
- **CPU**: 2+ cores, 4+ cores recommended
- **Storage**: 1GB free space for installation and data

### Required Software
- **Rust**: Version 1.70 or later
- **Node.js**: Version 18 or later (with npm)
- **Git**: For cloning the repository

### Optional Software
- **Docker**: For containerized deployment
- **Visual Studio Code**: Recommended IDE with Rust and TypeScript extensions

## Quick Setup

### 1. Clone the Repository

```bash
git clone https://github.com/do-ops885/ai-orchestrator-hub.git
cd ai-orchestrator-hub
```

### 2. Backend Setup

```bash
# Navigate to backend directory
cd backend

# Install Rust dependencies and build
cargo build

# Run with basic NLP (recommended for development)
cargo run

# Run with advanced neural features
cargo run --features advanced-neural

# Run MCP server for external integrations
cargo run --bin mcp_server
```

### Backend Configuration

The backend uses layered configuration. Copy the default settings:

```bash
# Copy default configuration
cp settings/default.toml settings/active.toml

# For development, you can also copy development settings
cp settings/development.toml settings/active.toml
```

### Environment Variables

Create a `.env` file in the backend directory:

```env
# Server Configuration
HIVE_PORT=3001
HIVE_HOST=0.0.0.0
LOG_LEVEL=info

# Database (optional - uses SQLite by default)
DATABASE_URL=./data/hive_persistence.db

# Neural Processing
NEURAL_MODE=basic  # or "advanced"
MAX_AGENTS=100

# Security
RATE_LIMIT_REQUESTS_PER_MINUTE=1000
JWT_SECRET=your-development-secret-key
```

The backend will start on `http://localhost:3001` with WebSocket support on `ws://localhost:3001/ws`.

### 3. Frontend Setup

Open a new terminal window:

```bash
# Navigate to frontend directory
cd frontend

# Install Node.js dependencies (includes AI SDK)
npm install

# Start the development server
npm run dev

# Build for production
npm run build

# Run linting and type checking
npm run lint
npm run type-check
```

### Frontend Configuration

Create environment files for the frontend:

```bash
# Development environment
cp .env.example .env.local
```

Edit `.env.local`:

```env
# Backend API
NEXT_PUBLIC_API_URL=http://localhost:3001
NEXT_PUBLIC_WS_URL=ws://localhost:3001/ws

# MCP Server (optional)
NEXT_PUBLIC_MCP_URL=http://localhost:3002

# Application
NEXT_PUBLIC_APP_NAME="AI Orchestrator Hub"
NEXT_PUBLIC_APP_ENV=development

# AI Integration (optional)
OPENAI_API_KEY=your-openai-key
ANTHROPIC_API_KEY=your-anthropic-key
```

The frontend will be available at `http://localhost:3000`.

### 4. Verify Installation

1. Open your browser and navigate to `http://localhost:3000`
2. You should see the Hive Dashboard
3. The backend should show connection logs
4. Try creating a test agent and task

## Detailed Setup

### Backend Configuration

Create a `.env` file in the `backend/` directory:

```env
# Server Configuration
HIVE_PORT=3001
HIVE_HOST=localhost

# Logging
LOG_LEVEL=info
LOG_FORMAT=json

# Neural Processing
NEURAL_MODE=basic
MAX_AGENTS=100
TASK_QUEUE_SIZE=1000

# WebSocket
WS_MAX_CONNECTIONS=100
WS_HEARTBEAT_INTERVAL=30

# Database (optional)
DATABASE_URL=hive.db
```

### Feature Selection

The system supports different feature sets:

```bash
# Basic NLP only (lightweight, recommended for development)
cargo run

# Advanced neural processing with FANN networks
cargo run --features advanced-neural

# GPU acceleration (requires CUDA-compatible hardware)
cargo run --features advanced-neural,gpu-acceleration

# All features enabled
cargo run --all-features
```

### Available Features

- `basic-nlp`: Lightweight natural language processing (default)
- `advanced-neural`: FANN neural network integration
- `gpu-acceleration`: GPU support via CUDA/OpenCL
- All features can be combined as needed

### Frontend Configuration

The frontend is configured via environment variables in `frontend/.env.local`:

```env
# Backend API URL
NEXT_PUBLIC_API_URL=http://localhost:3001

# WebSocket URL
NEXT_PUBLIC_WS_URL=ws://localhost:3001/ws

# Application settings
NEXT_PUBLIC_APP_NAME="Multiagent Hive"
NEXT_PUBLIC_APP_VERSION="0.1.0"
```

## First Steps

### 1. Create Your First Agent

Using the web interface:
1. Navigate to the Agent Manager
2. Click "Create Agent"
3. Fill in:
   - **Name**: "Test Agent"
   - **Type**: "Worker"
   - **Capabilities**: Add "data_processing" with proficiency 0.8

### 2. Create Your First Task

1. Go to the Task Manager
2. Click "Create Task"
3. Configure:
   - **Description**: "Process sample data"
   - **Priority**: "Medium"
   - **Required Capabilities**: "data_processing" (min proficiency 0.7)

### 3. Monitor the Swarm

- Watch the Swarm Visualization for real-time agent positions
- Check the Metrics Panel for performance statistics
- Observe task progress in the Task Manager

## Testing the System

### Run Backend Tests

```bash
cd backend

# Basic test suite
cargo test

# Test with advanced features
cargo test --features advanced-neural

# Test all feature combinations
cargo test --all-features

# Run integration tests
cargo test --test integration_tests

# Run with verbose output
cargo test -- --nocapture
```

### Run Frontend Tests

```bash
cd frontend

# Unit tests
npm test

# Run tests in watch mode
npm test -- --watch

# Run with coverage
npm test -- --coverage
```

### Run Examples and Demos

```bash
cd backend

# Neural processing examples
cargo run --features advanced-neural --example neural_comparison
cargo run --features advanced-neural --example advanced_neural_test
cargo run --features advanced-neural --example lstm_demo

# Agent system examples
cargo run --example adaptive_verification_demo
cargo run --example agent_monitor_example
cargo run --example pair_programming_demo

# Persistence examples
cargo run --example persistence_demo
cargo run --example advanced_persistence_demo

# Swarm intelligence
cargo run --example swarm_coordination_demo
```

## Troubleshooting

### Common Issues

#### Backend Won't Start
- **Check Rust version**: `rustc --version` (should be 1.70+)
- **Clean and rebuild**: `cargo clean && cargo build`
- **Check port availability**: `lsof -i :3001` or `netstat -tlnp | grep 3001`
- **Check configuration**: Verify `settings/active.toml` exists and is valid
- **Check environment**: Ensure `.env` file is properly configured
- **Check dependencies**: `cargo check` for compilation errors

#### Frontend Build Errors
- **Clear node modules**: `rm -rf node_modules package-lock.json && npm install`
- **Check Node.js version**: `node --version` (should be 18+)
- **Update dependencies**: `npm update`
- **Check TypeScript**: `npm run type-check` for type errors
- **Check environment**: Verify `.env.local` is properly configured

#### WebSocket Connection Issues
- **Verify backend is running**: Check `http://localhost:3001/health`
- **Check CORS**: Ensure backend allows frontend origin
- **Check firewall**: Ensure ports 3001 (backend) and 3000 (frontend) are accessible
- **Browser console**: Look for WebSocket connection errors
- **Network tools**: Use browser dev tools Network tab to inspect WebSocket traffic

#### Database Issues
- **Check permissions**: Ensure write access to `./data/` directory
- **Check SQLite**: Verify SQLite is available (`sqlite3 --version`)
- **Reset database**: Delete `./data/hive_persistence.db` to start fresh
- **Check configuration**: Verify `DATABASE_URL` in `.env`

#### Performance Issues
- **Monitor resources**: Use `/health` endpoint to check system status
- **Check metrics**: Access `/metrics` for performance data
- **Adjust configuration**: Modify `MAX_AGENTS` and memory settings
- **Enable optimization**: Check `PERFORMANCE_OPTIMIZATION_ENABLED` setting

### Getting Help

- **Documentation**: Check the [docs/](../docs/) directory
- **Issues**: Search existing [GitHub Issues](../../issues)
- **Discussions**: Join community discussions
- **Logs**: Enable debug logging with `RUST_LOG=debug`

## Next Steps

Now that you have the system running, you can:

1. **Explore Features**: Try different agent types and capabilities
2. **Scale Up**: Add more agents and observe swarm behavior
3. **Customize**: Modify configuration for your use case
4. **Develop**: Start building custom agents or integrations
5. **Deploy**: Set up production deployment (see [deployment guide](deployment.md))

## Development Workflow

For ongoing development:

```bash
# Backend development with auto-restart
cd backend
cargo watch -x run  # Auto-restart on changes

# Backend with advanced features
cargo watch -x 'run --features advanced-neural'

# Frontend development with hot reload
cd frontend
npm run dev  # Hot reload enabled

# Run tests on changes
cd backend && cargo test -- --nocapture
cd frontend && npm run test:watch

# Linting and formatting
cd backend && cargo clippy && cargo fmt
cd frontend && npm run lint:fix
```

### Development Tools

```bash
# Health monitoring
curl http://localhost:3001/health

# Metrics monitoring
curl http://localhost:3001/metrics

# API testing
curl http://localhost:3001/api/hive/status

# WebSocket testing (requires wscat or similar)
wscat -c ws://localhost:3001/ws
```

### Debugging

```bash
# Enable debug logging
cd backend
RUST_LOG=debug cargo run

# Frontend debugging
cd frontend
npm run dev  # Then use browser dev tools

# Database inspection
sqlite3 ./data/hive_persistence.db ".tables"
sqlite3 ./data/hive_persistence.db ".schema"
```

## Performance Tuning

For optimal performance:

### Configuration Tuning

```env
# Basic development setup
MAX_AGENTS=100
TASK_QUEUE_SIZE=1000
MEMORY_LIMIT_MB=1024
CPU_CORES=auto

# Production setup
MAX_AGENTS=1000
TASK_QUEUE_SIZE=10000
MEMORY_LIMIT_MB=4096
CPU_CORES=8

# High-performance setup
MAX_AGENTS=5000
TASK_QUEUE_SIZE=50000
MEMORY_LIMIT_MB=8192
CPU_CORES=16
```

### Feature Selection for Performance

- **Basic Mode**: Use for development and small deployments (< 500 agents)
- **Advanced Neural**: Enable for complex tasks and learning (500+ agents)
- **GPU**: Add for high-throughput neural processing (1000+ agents)
- **Persistence**: Enable for state recovery and long-running deployments

### Monitoring Performance

```bash
# Check system health
curl http://localhost:3001/health

# Monitor metrics
curl http://localhost:3001/metrics

# Check resource usage
curl http://localhost:3001/api/resources
```

### Optimization Tips

- **Memory**: Monitor and adjust `MEMORY_LIMIT_MB` based on usage
- **CPU**: Use `CPU_CORES=auto` for automatic detection or set explicitly
- **Database**: Enable connection pooling for high-throughput scenarios
- **WebSocket**: Monitor connection limits and adjust `WS_MAX_CONNECTIONS`
- **Rate Limiting**: Configure `RATE_LIMIT_REQUESTS_PER_MINUTE` appropriately

See the [performance guide](performance.md) for detailed benchmarks and tuning tips.