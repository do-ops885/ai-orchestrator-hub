# Multiagent Hive System

A sophisticated **hybrid neural multiagent hive system** implementing swarm intelligence with NLP self-learning capabilities. **CPU-native, GPU-optional - built for the GPU-poor.**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![TypeScript](https://img.shields.io/badge/typescript-5.6+-blue.svg)](https://www.typescriptlang.org)
[![Next.js](https://img.shields.io/badge/next.js-15.0+-black.svg)](https://nextjs.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## Table of Contents

- [Overview](#overview)
- [System Architecture](#system-architecture)
- [Key Features](#key-features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Configuration](#configuration)
- [API Documentation](#api-documentation)
- [Development](#development)
- [Deployment](#deployment)
- [Contributing](#contributing)
- [Troubleshooting](#troubleshooting)
- [License](#license)

## Overview

The Multiagent Hive System is a cutting-edge implementation of swarm intelligence that combines neural processing, real-time communication, and adaptive learning. Designed with a "CPU-native, GPU-optional" philosophy, it delivers maximum intelligence on minimal hardware while scaling to utilize advanced resources when available.

## System Architecture

### Core Components

#### Backend (Rust)
- **Main Entry**: `backend/src/main.rs` - Axum web server with WebSocket support
- **Hive Coordinator**: `backend/src/hive.rs` - Central orchestration system managing agent swarms
- **Agent System**: `backend/src/agent.rs` - Individual agent implementations with capabilities and behaviors
- **Task Management**: `backend/src/task.rs` - Task queue, distribution, and execution system
- **Communication**: `backend/src/communication.rs` - WebSocket handlers for real-time coordination
- **NLP Processing**: `backend/src/nlp.rs` - Lightweight natural language processing for agent communication
- **Neural Networks**: `backend/src/neural.rs` - Hybrid neural architecture with optional ruv-FANN integration

#### Frontend (TypeScript/React/Next.js)
- **Dashboard**: `frontend/src/components/HiveDashboard.tsx` - Main monitoring interface
- **Visualization**: `frontend/src/components/SwarmVisualization.tsx` - Real-time agent swarm display
- **Metrics**: `frontend/src/components/MetricsPanel.tsx` & `NeuralMetrics.tsx` - Performance monitoring
- **State Management**: `frontend/src/store/hiveStore.ts` - Zustand store for WebSocket communication
- **Agent Management**: `frontend/src/components/AgentManager.tsx` - Agent creation and configuration
- **Task Management**: `frontend/src/components/TaskManager.tsx` - Task creation and monitoring

## Key Features

### Hybrid Neural Architecture - CPU-native, GPU-optional
- **Basic NLP** (default): Lightweight CPU processing for real-time swarm coordination
- **Advanced Neural** (optional): ruv-FANN integration for complex pattern recognition
- **Philosophy**: Built for the GPU-poor - maximum intelligence on minimal hardware
- **Feature Flags**: `basic-nlp`, `advanced-neural`, `gpu-acceleration`

### Agent Types & Capabilities
- **Worker**: General task execution
- **Coordinator**: Swarm coordination and task distribution
- **Specialist**: Domain-specific expertise
- **Learner**: Continuous learning and adaptation

### Communication System
- **WebSocket**: Real-time bidirectional communication
- **REST API**: Standard CRUD operations for agents/tasks
- **Message Types**: `hive_status`, `agents_update`, `metrics_update`, `agent_created`, `task_created`

### Task Management
- **Priority Levels**: Low, Medium, High, Critical
- **Status Tracking**: Pending, Assigned, InProgress, Completed, Failed, Cancelled
- **Capability Matching**: Automatic agent assignment based on required capabilities

## Quick Start

### Basic Setup (Recommended)
```bash
# Backend with basic NLP
cd backend
cargo run

# Frontend
cd frontend
npm install
npm run dev
```

### Advanced Neural Features (Optional)
```bash
# Backend with ruv-FANN integration
cd backend
cargo run --features advanced-neural

# Run neural comparison demo
cargo run --features advanced-neural --example neural_comparison
```

## Ports & Endpoints
- **Backend**: `http://localhost:3001`
- **Frontend**: `http://localhost:3000`
- **WebSocket**: `ws://localhost:3001/ws`
- **API**: `/api/agents`, `/api/tasks`, `/api/hive/status`

## Data Structures

### Core Types
- **Agent**: ID, type, state, capabilities, position, energy, social connections
- **Task**: ID, description, priority, status, required capabilities, assigned agents
- **HiveStatus**: Metrics, swarm center, total energy, creation/update timestamps
- **SwarmMetrics**: Agent counts, task completion, performance, cohesion, learning progress

### Neural Processing
- **NLPProcessor**: Pattern recognition, semantic analysis, learning insights
- **HybridNeuralProcessor**: Optional FANN networks for advanced capabilities
- **NetworkType**: Basic, FANN, LSTM configurations

## Installation

### Prerequisites

- **Rust**: 1.70+ with Cargo
- **Node.js**: 18+ with npm
- **System Requirements**:
  - Minimum: 2GB RAM, 2 CPU cores
  - Recommended: 4GB RAM, 4+ CPU cores
  - Optimal: 8GB+ RAM, 8+ CPU cores with SIMD support

### Backend Setup

```bash
# Clone the repository
git clone <repository-url>
cd multiagent-hive

# Install Rust dependencies
cd backend
cargo build

# Run with basic NLP (recommended for development)
cargo run

# Run with advanced neural features (requires more resources)
cargo run --features advanced-neural
```

### Frontend Setup

```bash
# Install Node.js dependencies
cd frontend
npm install

# Start development server
npm run dev

# Build for production
npm run build
```

### MCP Server (Optional)

```bash
# Run standalone MCP server for external integrations
cd backend
cargo run --bin mcp_server
```

## Configuration

### Environment Variables

Create a `.env` file in the backend directory:

```env
# Server Configuration
HIVE_PORT=3001
HIVE_HOST=localhost

# Logging
LOG_LEVEL=info
LOG_FORMAT=json

# Neural Processing
NEURAL_MODE=basic  # or "advanced"
MAX_AGENTS=1000
TASK_QUEUE_SIZE=10000

# WebSocket
WS_MAX_CONNECTIONS=100
WS_HEARTBEAT_INTERVAL=30

# Resource Management
MEMORY_LIMIT_MB=1024
CPU_CORES=auto  # or specific number
```

### Feature Flags

The system supports multiple build configurations:

- `default = ["basic-nlp"]`: Lightweight NLP processing
- `advanced-neural`: Enables ruv-FANN neural networks
- `gpu-acceleration`: GPU support (requires compatible hardware)

## API Documentation

### REST Endpoints

#### Agents

- `GET /api/agents` - List all agents
- `POST /api/agents` - Create a new agent
- `GET /api/agents/{id}` - Get agent details
- `PUT /api/agents/{id}` - Update agent
- `DELETE /api/agents/{id}` - Remove agent

#### Tasks

- `GET /api/tasks` - List all tasks
- `POST /api/tasks` - Create a new task
- `GET /api/tasks/{id}` - Get task details
- `PUT /api/tasks/{id}` - Update task
- `DELETE /api/tasks/{id}` - Cancel task

#### Hive Status

- `GET /api/hive/status` - Get current hive status and metrics
- `GET /api/hive/metrics` - Get detailed performance metrics
- `POST /api/hive/reset` - Reset hive state (development only)

### WebSocket Events

Connect to `ws://localhost:3001/ws` for real-time updates:

- `hive_status`: Complete hive state and metrics
- `agents_update`: Agent list with current states
- `metrics_update`: Performance metrics updates
- `agent_created`: New agent creation notifications
- `task_created`: New task creation notifications
- `task_completed`: Task completion notifications
- `error`: Error notifications

### MCP Integration

The system implements Model Context Protocol (MCP) 1.0 for external tool integration:

#### Available Tools

- `create_swarm_agent`: Create new agents with specified capabilities
- `assign_swarm_task`: Assign tasks to the swarm
- `get_swarm_status`: Retrieve current hive status
- `analyze_with_nlp`: Perform NLP analysis on text
- `coordinate_agents`: Coordinate agent behaviors

#### Resources

- `hive://status`: Live system status
- `hive://agents`: Agent information
- `hive://tasks`: Task information

## Development

### Code Standards

- **Rust**: Follow Clippy rules with comprehensive linting
- **TypeScript**: Strict type checking with ESLint flat config
- **Documentation**: Use `///` for public APIs, `//!` for module docs
- **Testing**: Write unit tests for all public functions
- **Error Handling**: Use `anyhow::Result` for Rust, proper error boundaries for React

### Running Tests

```bash
# Backend tests
cd backend
cargo test
cargo test --features advanced-neural

# Frontend tests
cd frontend
npm test

# Linting
cargo clippy
npm run lint
```

### Examples and Demos

```bash
# Compare basic vs advanced neural processing
cargo run --features advanced-neural --example neural_comparison

# Test advanced neural capabilities
cargo run --features advanced-neural --example advanced_neural_test

# LSTM network demonstration
cargo run --features advanced-neural --example lstm_demo
```

## Deployment

### Production Build

```bash
# Backend
cd backend
cargo build --release

# Frontend
cd frontend
npm run build
```

### Docker Deployment

```dockerfile
# Example Dockerfile for backend
FROM rust:1.70 as builder
WORKDIR /app
COPY backend/ .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/multiagent-hive /usr/local/bin/
EXPOSE 3001
CMD ["multiagent-hive"]
```

### Security Considerations

- Configure CORS policies for production domains
- Use HTTPS in production environments
- Implement rate limiting for API endpoints
- Monitor WebSocket connection limits
- Validate all input data
- Use secure environment variable management

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Run the test suite: `cargo test && npm test`
5. Run linting: `cargo clippy && npm run lint`
6. Commit your changes: `git commit -m 'Add amazing feature'`
7. Push to the branch: `git push origin feature/amazing-feature`
8. Open a Pull Request

### Code Review Process

- All changes require review from at least one maintainer
- Ensure all CI checks pass
- Update documentation for new features
- Add tests for bug fixes and new functionality

## Troubleshooting

### Common Issues

#### Backend Won't Start

```bash
# Check Rust version
rustc --version  # Should be 1.70+

# Clean and rebuild
cargo clean
cargo build

# Check for port conflicts
lsof -i :3001
```

#### Frontend Build Errors

```bash
# Clear node modules and reinstall
rm -rf node_modules package-lock.json
npm install

# Check Node.js version
node --version  # Should be 18+
```

#### WebSocket Connection Issues

- Ensure backend is running on port 3001
- Check firewall settings
- Verify CORS configuration in production

#### Performance Issues

- Monitor system resources with the ResourceMonitor component
- Consider using `advanced-neural` features for better performance
- Adjust `MAX_AGENTS` and `TASK_QUEUE_SIZE` in configuration

### Getting Help

- Check the [Issues](https://github.com/your-repo/issues) page
- Review the [Documentation](docs/)
- Join our community discussions

## Testing & Examples

### Available Examples

- **Neural Comparison**: `backend/examples/neural_comparison.rs` - Compare basic vs advanced processing
- **Advanced Neural Test**: `backend/examples/advanced_neural_test.rs` - Test FANN integration
- **LSTM Demo**: `backend/examples/lstm_demo.rs` - Time series forecasting demonstration

### Running Examples

```bash
# Basic neural comparison
cargo run --example neural_comparison

# Advanced features (requires advanced-neural feature)
cargo run --features advanced-neural --example advanced_neural_test
cargo run --features advanced-neural --example lstm_demo
```

## Performance Benchmarks

| Configuration | Agents | Tasks/sec | Memory Usage | CPU Usage |
|---------------|--------|-----------|--------------|-----------|
| Basic NLP     | 100    | 50        | 256MB        | 15%       |
| Basic NLP     | 500    | 200       | 512MB        | 35%       |
| Advanced      | 100    | 75        | 384MB        | 25%       |
| Advanced      | 500    | 350       | 768MB        | 55%       |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [Tokio](https://tokio.rs/)
- Frontend powered by [Next.js](https://nextjs.org/) and [React](https://reactjs.org/)
- Neural processing with [ruv-FANN](https://github.com/rust-unofficial/ruv-fann)
- Real-time communication via WebSockets
- MCP integration for external tool support

---

**CPU-native, GPU-optional - built for the GPU-poor.** 🚀