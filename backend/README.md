# Multiagent Hive Backend

The backend component of the Multiagent Hive System, built with Rust for high performance and reliability.

## Overview

This Rust-based backend provides the core swarm intelligence engine with:

- **High-performance async processing** using Tokio
- **Real-time WebSocket communication** for agent coordination
- **Hybrid neural processing** (CPU-native with optional GPU acceleration)
- **RESTful API** for external integrations
- **Model Context Protocol (MCP)** support for tool integration
- **Extensible agent system** with capability-based matching
- **Comprehensive monitoring** and metrics collection

## Architecture

### Core Components

```
backend/
├── src/
│   ├── main.rs              # Application entry point
│   ├── agents/              # Agent system implementation
│   ├── communication/       # WebSocket and MCP handling
│   ├── core/                # Core swarm intelligence logic
│   ├── infrastructure/      # Infrastructure and utilities
│   ├── neural/              # Neural processing engine
│   ├── tasks/               # Task management system
│   └── utils/               # Shared utilities
├── examples/                # Example applications
├── tests/                   # Integration tests
└── Cargo.toml              # Rust dependencies
```

### Key Features

- **Agent Types**: Worker, Coordinator, Specialist, Learner
- **Neural Modes**: Basic NLP, Advanced FANN networks, GPU acceleration
- **Communication**: WebSocket real-time updates, REST API, MCP protocol
- **Persistence**: SQLite/PostgreSQL support with migration system
- **Security**: JWT authentication, input validation, rate limiting
- **Monitoring**: Prometheus metrics, structured logging, health checks

## Quick Start

### Prerequisites

- Rust 1.70+
- Cargo package manager

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/multiagent-hive.git
cd multiagent-hive/backend

# Build the project
cargo build

# Run with basic features
cargo run
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

### Environment Variables

```env
# Server configuration
HIVE_PORT=3001
HIVE_HOST=localhost
HIVE_WORKERS=4

# Neural processing
NEURAL_MODE=basic
MAX_AGENTS=1000
LEARNING_RATE=0.1

# Database
DATABASE_URL=hive.db
DATABASE_POOL_SIZE=10

# Security
JWT_SECRET=your-secret-key
API_KEY_REQUIRED=false

# Logging
LOG_LEVEL=info
LOG_FORMAT=json
```

### Feature Flags

```bash
# Basic NLP only (recommended)
cargo run

# Advanced neural processing
cargo run --features advanced-neural

# GPU acceleration
cargo run --features advanced-neural,gpu-acceleration

# All features
cargo run --all-features
```

## API Reference

### REST Endpoints

#### Agents

```http
GET    /api/agents           # List all agents
POST   /api/agents           # Create new agent
GET    /api/agents/{id}      # Get agent details
PUT    /api/agents/{id}      # Update agent
DELETE /api/agents/{id}      # Remove agent
```

#### Tasks

```http
GET    /api/tasks            # List all tasks
POST   /api/tasks            # Create new task
GET    /api/tasks/{id}       # Get task details
PUT    /api/tasks/{id}       # Update task
DELETE /api/tasks/{id}       # Cancel task
```

#### Hive Status

```http
GET    /api/hive/status      # Get current status
GET    /api/hive/metrics     # Get performance metrics
POST   /api/hive/reset       # Reset hive state
```

### WebSocket Events

Connect to `ws://localhost:3001/ws` for real-time updates:

```javascript
const ws = new WebSocket('ws://localhost:3001/ws');

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);

    switch (data.type) {
        case 'hive_status':
            console.log('Hive status:', data.data);
            break;
        case 'agent_created':
            console.log('New agent:', data.data.agent);
            break;
        case 'task_completed':
            console.log('Task completed:', data.data.task);
            break;
    }
};
```

### MCP Integration

The backend implements MCP 1.0 for external tool integration:

```json
{
  "mcp": {
    "version": "1.0",
    "tools": [
      {
        "name": "create_swarm_agent",
        "description": "Create a new agent with specified capabilities",
        "input_schema": {
          "type": "object",
          "properties": {
            "name": {"type": "string"},
            "agent_type": {"type": "string"},
            "capabilities": {"type": "array"}
          }
        }
      }
    ]
  }
}
```

## Development

### Project Structure

```
src/
├── agents/              # Agent implementations
│   ├── mod.rs
│   ├── agent.rs         # Core agent logic
│   ├── adaptive_verification.rs
│   ├── agent_evolution.rs
│   └── ...
├── communication/       # Communication protocols
│   ├── mod.rs
│   ├── websocket.rs     # WebSocket handling
│   └── mcp.rs          # MCP protocol
├── core/               # Core system logic
│   ├── mod.rs
│   ├── hive.rs         # Main hive coordinator
│   └── swarm_intelligence.rs
├── infrastructure/     # Infrastructure components
│   ├── mod.rs
│   ├── metrics.rs      # Metrics collection
│   ├── cache.rs        # Caching layer
│   └── security.rs     # Security middleware
├── neural/            # Neural processing
│   ├── mod.rs
│   ├── neural.rs      # Neural network logic
│   └── nlp.rs         # Natural language processing
├── tasks/             # Task management
│   ├── mod.rs
│   ├── task.rs        # Task definitions
│   └── work_stealing_queue.rs
└── utils/             # Utilities
    ├── mod.rs
    ├── config.rs      # Configuration management
    ├── error.rs       # Error handling
    └── validation.rs  # Input validation
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

# Run specific test
cargo test test_agent_creation

# Run with coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check documentation
cargo doc --open
```

## Examples

### Basic Agent Creation

```rust
use multiagent_hive::{Hive, AgentType, Capability};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut hive = Hive::new();

    // Create agent with capabilities
    let agent_id = hive.create_agent(
        "data_processor",
        AgentType::Worker,
        vec![
            Capability::new("data_processing", 0.8, 0.1),
            Capability::new("analysis", 0.7, 0.15),
        ]
    ).await?;

    println!("Created agent: {}", agent_id);
    Ok(())
}
```

### Task Processing

```rust
use multiagent_hive::{Hive, Priority, TaskRequirements};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut hive = Hive::new();

    // Create task with requirements
    let task_id = hive.create_task(
        "Analyze customer data",
        Priority::High,
        TaskRequirements {
            capabilities: vec!["data_processing".to_string()],
            min_proficiency: 0.7,
            estimated_duration: Some(300), // 5 minutes
        }
    ).await?;

    // Monitor task progress
    while let Some(status) = hive.get_task_status(task_id).await? {
        match status {
            TaskStatus::Completed => {
                println!("Task completed successfully!");
                break;
            }
            TaskStatus::Failed => {
                println!("Task failed");
                break;
            }
            _ => {
                println!("Task status: {:?}", status);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    Ok(())
}
```

### Neural Processing

```rust
use multiagent_hive::neural::{NeuralProcessor, ProcessingMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize neural processor
    let processor = NeuralProcessor::new(ProcessingMode::Advanced)?;

    // Process text
    let result = processor.analyze_text(
        "The customer data shows interesting patterns in user behavior.",
        AnalysisType::Sentiment
    ).await?;

    println!("Sentiment analysis: {:?}", result);

    // Pattern recognition
    let patterns = processor.find_patterns(vec![
        "user clicked button",
        "user viewed page",
        "user purchased item",
        "user clicked button",
        "user viewed page"
    ]).await?;

    println!("Detected patterns: {:?}", patterns);

    Ok(())
}
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

- **Documentation**: [docs/](../docs/) directory
- **Issues**: [GitHub Issues](../../issues)
- **Discussions**: [GitHub Discussions](../../discussions)
- **Email**: support@multiagent-hive.dev