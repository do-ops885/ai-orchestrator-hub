# AI Orchestrator Hub Backend

The backend component of the AI Orchestrator Hub, built with Rust for high performance and reliability.

## Overview

This Rust-based backend provides the core swarm intelligence engine with:

- **High-performance async processing** using Tokio
- **Real-time WebSocket communication** for agent coordination
- **Hybrid neural processing** (CPU-native with optional GPU acceleration)
- **RESTful API** for external integrations
- **Model Context Protocol (MCP)** support for tool integration
- **Extensible agent system** with capability-based matching and evolution
- **Comprehensive monitoring** and metrics collection with intelligent alerting
- **Simple verification system** for lightweight task validation
- **Auto-scaling and intelligent fallback** for resilience
- **Advanced testing infrastructure** with integration and chaos engineering tests
- **Intelligent caching** with multi-tier cache management
- **Security auditing** and structured logging
- **Circuit breaker pattern** for fault tolerance
- **Performance optimization** with adaptive learning

## Architecture

### Core Components

```
backend/
├── src/
│   ├── main.rs              # Application entry point with Axum server
│   ├── lib.rs               # Library exports and module declarations
│   ├── server.rs            # Server configuration and setup
│   ├── init.rs              # Application initialization
│   ├── agents/              # Agent system implementation
│   │   ├── adaptive_verification.rs
│   │   ├── agent_evolution.rs
│   │   ├── [11+ agent implementations]
│   │   └── mod.rs
│   ├── api/                 # API response types and validation
│   │   ├── responses.rs     # Standardized API responses
│   │   └── mod.rs
│   ├── communication/       # WebSocket and MCP handling
│   │   ├── communication.rs
│   │   ├── mcp_http.rs
│   │   └── [2+ communication modules]
│   ├── core/                # Core swarm intelligence logic
│   │   ├── hive/
│   │   │   ├── [7+ hive coordination modules]
│   │   ├── auto_scaling.rs  # Dynamic agent scaling
│   │   └── [5+ core modules]
│   ├── infrastructure/      # Infrastructure and utilities
│   │   ├── monitoring/
│   │   │   ├── [12+ monitoring modules]
│   │   ├── async_optimizer.rs
│   │   └── [18+ infrastructure modules]
│   ├── neural/              # Neural processing engine
│   │   ├── adaptive_learning.rs
│   │   ├── core.rs          # Hybrid neural processor
│   │   └── [7+ neural modules]
│   ├── tasks/               # Task management system
│   │   ├── mod.rs
│   │   ├── task.rs          # Task definitions and queue
│   │   └── [1+ task modules]
│   └── utils/               # Shared utilities
│       ├── auth.rs          # Authentication utilities
│       ├── config.rs        # Configuration management
│       └── [11+ utility modules]
├── examples/                # Example applications
├── tests/                   # Integration tests
├── benches/                 # Performance benchmarks
├── data/                    # Application data directory
├── settings/                # Configuration files
└── Cargo.toml              # Rust dependencies
```

### Key Features

- **Agent Types**: Worker, Coordinator, Specialist, Learner with evolution capabilities
- **Neural Modes**: Basic NLP (default), Advanced FANN networks, GPU acceleration
- **Communication**: WebSocket real-time updates, REST API, MCP protocol
- **Persistence**: SQLite with encryption, compression, and backup support
- **Security**: JWT authentication, input validation, rate limiting, security auditing, circuit breaker
- **Monitoring**: Comprehensive metrics, intelligent alerting, health checks, telemetry
- **Verification**: Simple verification system with configurable tiers and adaptive learning
- **Auto-scaling**: Dynamic agent scaling based on workload with intelligent fallback
- **Caching**: Multi-tier intelligent caching with invalidation strategies
- **Testing**: Comprehensive test suite with integration, chaos engineering, and performance tests
- **Recovery**: Agent recovery management and fault tolerance
- **Performance**: Adaptive learning system and performance optimization

## Quick Start

### Prerequisites

- Rust 1.70+
- Cargo package manager

### Installation

```bash
# Clone the repository
git clone https://github.com/do-ops885/ai-orchestrator-hub.git
cd ai-orchestrator-hub/backend

# Build the project
cargo build

# Run with basic features (default)
cargo run
```

### Basic Usage

```rust
use ai_orchestrator_hub::HiveCoordinator;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a new hive coordinator
    let hive = HiveCoordinator::new().await?;

    // Create an agent
    let agent_config = json!({
        "name": "WorkerAgent-1",
        "type": "worker",
        "capabilities": [
            {
                "name": "data_processing",
                "proficiency": 0.8,
                "learning_rate": 0.1
            }
        ]
    });
    let agent_id = hive.create_agent(agent_config).await?;

    // Create a task
    let task_config = json!({
        "description": "Process customer data",
        "type": "data_processing",
        "priority": 1,
        "required_capabilities": [
            {
                "name": "data_processing",
                "min_proficiency": 0.7
            }
        ]
    });
    let task_id = hive.create_task(task_config).await?;

    // Monitor progress
    loop {
        let status = hive.get_task_status(task_id).await?;
        println!("Task status: {:?}", status);

        if status == serde_json::json!({"status": "completed"}) {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
```

### Basic Usage

```rust
use multiagent_hive::Hive;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new hive
    let mut hive = Hive::new();

    // Add an agent
    let agent_id = hive.add_agent("worker_agent", AgentType::Worker).await?;

    // Create a task
    let task_id = hive.create_task("Process data", Priority::Medium).await?;

    // Assign task to agent
    hive.assign_task(task_id, agent_id).await?;

    // Monitor progress
    loop {
        let status = hive.get_task_status(task_id).await?;
        println!("Task status: {:?}", status);

        if status == TaskStatus::Completed {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
```

## Configuration

### Configuration Files

The system uses TOML configuration files with environment variable overrides. Default configuration is in `settings/default.toml`:

```toml
[server]
host = "0.0.0.0"
port = 3001
cors_origins = ["http://localhost:3000"]
websocket_timeout_secs = 300
max_connections = 1000

[agents]
max_agents = 100
default_energy = 100.0
energy_decay_rate = 0.1
learning_rate = 0.01
max_memory_size = 1000

[tasks]
max_concurrent_tasks = 50
task_timeout_secs = 300
retry_attempts = 3
priority_levels = 4

[resources]
cpu_threshold = 80.0
memory_threshold = 85.0
auto_scaling_enabled = true
monitoring_interval_secs = 30

[neural]
enable_advanced_neural = false
batch_size = 32
learning_rate = 0.001
max_iterations = 1000

[logging]
level = "info"
format = "json"
max_file_size_mb = 100

[performance]
cpu_warning_threshold = 70.0
cpu_critical_threshold = 90.0
memory_warning_threshold = 80.0
memory_critical_threshold = 95.0
metrics_collection_interval_ms = 5000
alert_check_interval_ms = 30000
circuit_breaker_failure_threshold = 5
circuit_breaker_recovery_timeout_ms = 30000

[monitoring]
monitoring_interval_secs = 5
metrics_retention_days = 7
enable_agent_discovery = true
enable_health_monitoring = true
enable_performance_monitoring = true
enable_behavior_analysis = true
enable_dashboards = true
enable_alerting = true
enable_diagnostics = true
enable_reporting = true
```

### Environment Variables

Environment variables can override configuration file settings:

```bash
# Server Configuration
export HIVE_SERVER__HOST=0.0.0.0
export HIVE_SERVER__PORT=3001
export HIVE_SERVER__CORS_ORIGINS='["http://localhost:3000"]'

# Logging & Monitoring
export HIVE_LOGGING__LEVEL=info
export HIVE_LOGGING__FORMAT=json
export HIVE_PERFORMANCE__METRICS_COLLECTION_INTERVAL_MS=5000

# Neural Processing
export HIVE_NEURAL__ENABLE_ADVANCED_NEURAL=false
export HIVE_AGENTS__MAX_AGENTS=100

# Security
export HIVE_SECURITY__JWT_SECRET=your-secret-key-here
export HIVE_SECURITY__AUDIT_LOGGING_ENABLED=true
```

### Feature Flags

```bash
# Basic NLP only (default - recommended for most use cases)
cargo run

# Advanced neural processing with FANN networks
cargo run --features advanced-neural

# GPU acceleration (requires advanced-neural)
cargo run --features advanced-neural,gpu-acceleration

# All features
cargo run --all-features
```

## API Reference

### REST Endpoints

#### Core Endpoints

```http
GET    /                     # Server status message
GET    /health               # Comprehensive health check
GET    /metrics              # System metrics and trends
GET    /debug/system         # Debug system information
```

#### Agents

```http
GET    /api/agents           # List all agents with status
POST   /api/agents           # Create new agent with validation
```

#### Tasks

```http
GET    /api/tasks            # List all tasks with status
POST   /api/tasks            # Create new task with validation
```

#### Hive Management

```http
GET    /api/hive/status      # Get current hive status and metrics
GET    /api/resources        # Get system resource information
```

#### MCP Integration

```http
GET    /api/mcp/health       # MCP server health check
POST   /api/mcp/tools        # Execute MCP tools
GET    /api/mcp/resources    # List MCP resources
```

### WebSocket Events

Connect to `ws://localhost:3001/ws` for real-time updates with standardized message format:

```javascript
const ws = new WebSocket('ws://localhost:3001/ws');

ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    console.log('Message type:', message.message_type);
    console.log('Data:', message.data);
    console.log('Timestamp:', message.timestamp);
};
```

#### Supported Message Types

- **`hive_status`**: Complete hive status and metrics
- **`agents_update`**: Agent list with current states and capabilities
- **`metrics_update`**: Real-time performance metrics and swarm data
- **`agent_created`**: New agent creation confirmation
- **`task_created`**: New task creation confirmation
- **`error`**: Error messages with correlation IDs

#### Client Message Format

Send messages to the server:

```javascript
const message = {
    action: "create_agent",
    payload: {
        name: "Worker-1",
        type: "worker",
        capabilities: [...]
    },
    correlation_id: "optional-id",
    timeout_ms: 30000
};

ws.send(JSON.stringify(message));
```

### API Response Format

All API responses follow a standardized format:

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "timestamp": "2024-01-01T00:00:00Z",
  "request_id": "uuid-v4"
}
```

### MCP Integration

The backend implements MCP (Model Context Protocol) 1.0 for external tool integration with standardized JSON-RPC communication.

#### MCP Server

The MCP server provides tools for external AI models to interact with the hive system:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "create_swarm_agent",
    "arguments": {
      "name": "AssistantAgent",
      "type": "specialist",
      "capabilities": [
        {
          "name": "data_analysis",
          "proficiency": 0.9,
          "learning_rate": 0.1
        }
      ]
    }
  }
}
```

#### Available MCP Tools

- **`create_swarm_agent`**: Create agents with custom capabilities and behaviors
- **`assign_swarm_task`**: Distribute tasks with intelligent agent matching
- **`get_swarm_status`**: Retrieve comprehensive hive status and metrics
- **`analyze_with_nlp`**: Perform advanced NLP analysis using neural networks
- **`coordinate_agents`**: Trigger swarm coordination and formation optimization
- **`get_performance_metrics`**: Access detailed performance analytics
- **`manage_resources`**: Monitor and optimize system resource usage

#### MCP Resources

- **`hive://status`**: Live system status with real-time updates
- **`hive://agents`**: Agent information with capability profiles
- **`hive://tasks`**: Task queue with execution status and history
- **`hive://metrics`**: Performance metrics and alerting data
- **`hive://resources`**: System resource utilization and optimization

#### Standalone MCP Server

Run the dedicated MCP server for external integrations:

```bash
cd backend
cargo run --bin mcp_server
```

Access at `http://localhost:3002` with WebSocket support at `ws://localhost:3002/ws`

## Development

### Project Structure

```
src/
├── agents/              # Agent implementations
│   ├── adaptive_verification.rs
│   ├── agent_evolution.rs
│   ├── [11+ agent implementations]
│   └── mod.rs
├── api/                 # API response types
│   ├── responses.rs     # Standardized API responses
│   └── mod.rs
├── communication/       # Communication protocols
│   ├── communication.rs # Communication utilities
│   ├── mcp_http.rs      # MCP HTTP handling
│   └── [2+ communication modules]
├── core/               # Core system logic
│   ├── hive/
│   │   ├── [7+ hive coordination modules]
│   ├── auto_scaling.rs # Dynamic agent scaling
│   └── [5+ core modules]
├── infrastructure/     # Infrastructure components
│   ├── monitoring/
│   │   ├── [12+ monitoring modules]
│   ├── async_optimizer.rs
│   └── [18+ infrastructure modules]
├── neural/            # Neural processing
│   ├── adaptive_learning.rs
│   ├── core.rs         # Hybrid neural processor
│   └── [7+ neural modules]
├── tasks/             # Task management
│   ├── task.rs         # Task definitions and queue
│   └── mod.rs
└── utils/             # Utilities
    ├── auth.rs         # Authentication utilities
    ├── config.rs       # Configuration management
    └── [11+ utility modules]
    ├── error.rs       # Error handling types
    ├── validation.rs  # Input validation
    └── structured_logging.rs
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Build with specific features
cargo build --features advanced-neural

# Check compilation without building
cargo check
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test agents

# Run integration tests
cargo test --test api_integration_tests
cargo test --test chaos_engineering_tests
cargo test --test performance_regression_tests

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html

# Run benchmarks
cargo bench
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code (with clippy configuration)
cargo clippy

# Check documentation
cargo doc --open

# Generate docs with private items
cargo doc --document-private-items
```

## Examples

### Basic Agent Creation

```rust
use ai_orchestrator_hub::HiveCoordinator;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hive = HiveCoordinator::new().await?;

    // Create agent with capabilities
    let agent_config = json!({
        "name": "DataProcessor-1",
        "type": "specialist:data_processing",
        "capabilities": [
            {
                "name": "data_processing",
                "proficiency": 0.8,
                "learning_rate": 0.1
            },
            {
                "name": "analysis",
                "proficiency": 0.7,
                "learning_rate": 0.15
            }
        ]
    });

    let agent_id = hive.create_agent(agent_config).await?;
    println!("Created agent: {}", agent_id);
    Ok(())
}
```

### Task Processing with Simple Verification

```rust
use ai_orchestrator_hub::HiveCoordinator;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hive = HiveCoordinator::new().await?;

    // Create task with requirements
    let task_config = json!({
        "description": "Analyze customer satisfaction data",
        "type": "data_analysis",
        "priority": 2,
        "required_capabilities": [
            {
                "name": "data_processing",
                "min_proficiency": 0.7
            }
        ]
    });

    let task_id = hive.create_task(task_config).await?;

    // Execute with simple verification
    let (execution_result, verification_result) = hive
        .execute_task_with_simple_verification(task_id, Some("Provide actionable insights"))
        .await?;

    println!("Execution success: {}", execution_result.success);
    println!("Verification status: {:?}", verification_result.verification_status);
    println!("Overall score: {:.2}", verification_result.overall_score);

    Ok(())
}
```

### Neural Processing

```rust
use ai_orchestrator_hub::neural::HybridNeuralProcessor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize neural processor (basic mode by default)
    let processor = HybridNeuralProcessor::new().await?;

    // Process text with NLP
    let analysis = processor.analyze_text(
        "The customer data shows interesting patterns in user behavior."
    ).await?;

    println!("NLP Analysis: {:?}", analysis);

    // Pattern recognition
    let patterns = processor.find_patterns(vec![
        "user clicked button",
        "user viewed page",
        "user purchased item"
    ]).await?;

    println!("Detected patterns: {:?}", patterns);

    Ok(())
}
```

### Running Examples

```bash
# Simple verification demo
cargo run --example simple_verification_demo

# Neural processing examples
cargo run --example neural_comparison

# Advanced persistence demo
cargo run --example advanced_persistence_demo

# Agent monitoring example
cargo run --example agent_monitor_example
```

## Performance Tuning

### Memory Optimization

```rust
// Use object pooling for agents
use object_pool::Pool;

let agent_pool: Pool<Agent> = Pool::new(1000, || Agent::default());

// Reuse objects instead of allocating
let mut agent = agent_pool.pull(|| Agent::new());
```

### CPU Optimization

```rust
// Parallel task processing
use rayon::prelude::*;

let results: Vec<_> = tasks
    .par_iter()
    .map(|task| process_task(task))
    .collect();
```

### Database Optimization

```sql
-- Optimized indexes
CREATE INDEX CONCURRENTLY idx_agents_capabilities ON agents USING GIN(capabilities);
CREATE INDEX CONCURRENTLY idx_tasks_status_priority ON tasks(status, priority DESC);

-- Connection pooling
let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(&database_url)
    .await?;
```

## Monitoring

### Metrics

```rust
use metrics::{counter, histogram, gauge};

// Application metrics
counter!("tasks_created_total", 1);
counter!("agents_created_total", 1);
histogram!("task_processing_duration", duration.as_millis() as f64);
gauge!("active_agents", active_agents.len() as f64);
```

### Logging

```rust
use tracing::{info, error, warn};

// Structured logging
info!(
    agent_id = %agent.id,
    task_id = %task.id,
    "Agent assigned to task"
);

error!(
    error = %e,
    "Failed to process task"
);
```

### Health Checks

```rust
use warp::Filter;

// Health check endpoint
let health = warp::path("health")
    .map(|| {
        // Check database connectivity
        // Check agent responsiveness
        // Check system resources
        warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now(),
            "version": env!("CARGO_PKG_VERSION")
        }))
    });
```

## Security

### Authentication

```rust
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};

// JWT token generation
fn generate_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        iat: chrono::Utc::now().timestamp() as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}
```

### Input Validation

```rust
use validator::Validate;

#[derive(Validate)]
struct CreateAgentRequest {
    #[validate(length(min = 1, max = 100))]
    name: String,

    #[validate(range(min = 0.0, max = 1.0))]
    proficiency: f64,

    #[validate(length(min = 1))]
    capabilities: Vec<String>,
}
```

### Rate Limiting

```rust
use governor::{Quota, RateLimiter};

// Rate limiter for API endpoints
let limiter = RateLimiter::direct(Quota::per_second(nonzero!(10u32)));

if let Err(_) = limiter.check_n(1) {
    return Err("Rate limit exceeded".into());
}
```

## Deployment

### Docker

```dockerfile
FROM rust:1.70-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/multiagent-hive /usr/local/bin/

EXPOSE 3001
CMD ["multiagent-hive"]
```

### Systemd Service

```ini
[Unit]
Description=Multiagent Hive Backend
After=network.target

[Service]
Type=simple
User=hive
Group=hive
ExecStart=/usr/local/bin/multiagent-hive
Restart=always
RestartSec=5
Environment=HIVE_PORT=3001
Environment=DATABASE_URL=/var/lib/hive/hive.db

[Install]
WantedBy=multi-target
```

### Kubernetes

```yaml
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
```

## Contributing

### Development Setup

```bash
# Clone repository
git clone https://github.com/your-org/multiagent-hive.git
cd multiagent-hive/backend

# Install dependencies
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Code Standards

- Follow Rust idioms and best practices
- Use `cargo fmt` for consistent formatting
- Pass all `cargo clippy` lints
- Write comprehensive documentation
- Include unit tests for all public functions
- Use meaningful variable and function names

### Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Update documentation
7. Submit a pull request

## Troubleshooting

### Common Issues

#### Compilation Errors

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update

# Check Rust version
rustc --version
```

#### Runtime Errors

```bash
# Enable debug logging
export RUST_LOG=debug
cargo run

# Check system resources
top
df -h
free -h
```

#### Database Issues

```bash
# Check database file permissions
ls -la hive.db

# Reset database
rm hive.db
cargo run  # Recreates database
```

### Performance Issues

```bash
# Profile CPU usage
cargo flamegraph --bin multiagent-hive

# Profile memory usage
valgrind --tool=massif ./target/release/multiagent-hive

# Check system limits
ulimit -a
```

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## Support

- **Documentation**: [docs/](docs/) directory
- **Issues**: [GitHub Issues](../../issues)
- **Discussions**: [GitHub Discussions](../../discussions)
- **Email**: support@multiagent-hive.dev
