# Configuration Guide

This guide covers all configuration options for the Multiagent Hive System, including environment variables, feature flags, and runtime settings.

## Configuration Overview

The system uses a layered configuration approach:

1. **Default Values**: Built-in defaults for all settings
2. **Environment Variables**: Override defaults via environment
3. **Configuration Files**: TOML files for complex settings
4. **Runtime Configuration**: Dynamic changes via API

## Backend Configuration

### Environment Variables

Create a `.env` file in the `backend/` directory:

```env
# Server Configuration
HIVE_PORT=3001
HIVE_HOST=0.0.0.0
CORS_ORIGINS=http://localhost:3000

# Logging & Monitoring
LOG_LEVEL=info
LOG_FORMAT=json
METRICS_COLLECTION_INTERVAL_MS=5000
ALERT_CHECK_INTERVAL_MS=30000

# Neural Processing
NEURAL_MODE=basic
MAX_AGENTS=1000
TASK_QUEUE_SIZE=10000
LEARNING_RATE=0.01

# WebSocket & Communication
WS_MAX_CONNECTIONS=100
WS_HEARTBEAT_INTERVAL=30
WS_TIMEOUT_SECS=300

# Resource Management
MEMORY_LIMIT_MB=1024
CPU_CORES=auto
CPU_WARNING_THRESHOLD=70.0
MEMORY_WARNING_THRESHOLD=80.0

# Persistence
DATABASE_URL=./data/hive_persistence.db
PERSISTENCE_CHECKPOINT_INTERVAL_MINUTES=5
BACKUP_ENABLED=true

# Security
RATE_LIMIT_REQUESTS_PER_MINUTE=1000
JWT_SECRET=your-secret-key-here
AUDIT_LOGGING_ENABLED=true

# Performance
CIRCUIT_BREAKER_FAILURE_THRESHOLD=5
CIRCUIT_BREAKER_RECOVERY_TIMEOUT_MS=30000
PERFORMANCE_OPTIMIZATION_ENABLED=true

# Monitoring
MONITORING_INTERVAL_SECS=5
METRICS_RETENTION_DAYS=7
ALERT_THRESHOLD=0.8
ENABLE_AGENT_DISCOVERY=true
ENABLE_HEALTH_MONITORING=true
ENABLE_PERFORMANCE_MONITORING=true
ENABLE_BEHAVIOR_ANALYSIS=true
ENABLE_DASHBOARDS=true
ENABLE_ALERTING=true
ENABLE_DIAGNOSTICS=true
ENABLE_REPORTING=true
ENABLE_AUTOMATION=false
ENABLE_EXTERNAL_INTEGRATION=false

# Development
DEBUG_MODE=false
EXAMPLE_DATA_ENABLED=true
```

### Configuration Files

#### settings/default.toml

```toml
# AI Orchestrator Hub - Default Configuration
# This file contains all default settings that can be overridden

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
alert_threshold = 0.8
metrics_endpoint = "http://localhost:8000/metrics"
health_endpoint = "http://localhost:8000/health"
enable_agent_discovery = true
enable_health_monitoring = true
enable_performance_monitoring = true
enable_behavior_analysis = true
enable_dashboards = true
enable_alerting = true
enable_diagnostics = true
enable_reporting = true
enable_automation = false
enable_external_integration = false

[monitoring.diagnostics]
default_health_score = 0.85

[monitoring.diagnostics.component_health_scores]
database = 0.95
cache = 0.88
network = 0.92

[monitoring.diagnostics.component_issues]
database = ["Slow query performance"]
cache = ["High cache miss rate"]
network = ["Intermittent connectivity"]

[monitoring.diagnostics.component_recommendations]
database = ["Add database indexes", "Optimize query patterns"]
cache = ["Increase cache size", "Implement cache warming"]
network = ["Implement retry logic", "Monitor network latency"]

[monitoring.diagnostics.network_components]
internal_api = "internal_api"
external_services = "external_services"
database = "database"

[monitoring.diagnostics.performance_bottlenecks]
bottleneck1 = "Database query optimization needed"
bottleneck2 = "Memory usage spikes during peak hours"
bottleneck3 = "Network latency affecting response times"

[monitoring.diagnostics.optimization_opportunities]
opt1 = "Implement database query caching"
opt2 = "Add memory pooling for frequent allocations"
opt3 = "Use CDN for static assets"
opt4 = "Implement horizontal scaling"
```

#### settings/development.toml

```toml
[server]
port = 3001

[logging]
level = "debug"

[neural]
max_agents = 100
task_queue_size = 1000

[development]
debug_mode = true
example_data_enabled = true
```

#### settings/production.toml

```toml
[server]
port = 3001
host = "0.0.0.0"
workers = 8

[logging]
level = "warn"

[neural]
max_agents = 10000
task_queue_size = 100000

[security]
api_key_required = true

[resources]
memory_limit_mb = 4096
cpu_cores = 8
max_concurrent_tasks = 200

[monitoring]
enabled = true
```

## Frontend Configuration

### Environment Variables

Create `.env.local` in the `frontend/` directory:

```env
# API Configuration
NEXT_PUBLIC_API_URL=http://localhost:3001
NEXT_PUBLIC_WS_URL=ws://localhost:3001/ws

# Application Settings
NEXT_PUBLIC_APP_NAME="Multiagent Hive"
NEXT_PUBLIC_APP_VERSION="0.1.0"
NEXT_PUBLIC_APP_ENV=development

# Authentication
NEXT_PUBLIC_AUTH_ENABLED=false
NEXT_PUBLIC_AUTH_PROVIDER=github

# UI Configuration
NEXT_PUBLIC_THEME=default
NEXT_PUBLIC_LOCALE=en
NEXT_PUBLIC_TIMEZONE=UTC

# Feature Flags
NEXT_PUBLIC_ADVANCED_METRICS=true
NEXT_PUBLIC_DEBUG_PANEL=false
NEXT_PUBLIC_EXPERIMENTAL_FEATURES=false

# Performance
NEXT_PUBLIC_POLLING_INTERVAL=5000
NEXT_PUBLIC_MAX_RETRIES=3
NEXT_PUBLIC_REQUEST_TIMEOUT=10000

# Analytics (optional)
NEXT_PUBLIC_ANALYTICS_ID=
NEXT_PUBLIC_ANALYTICS_ENABLED=false
```

### Runtime Configuration

The frontend supports dynamic configuration via the backend API:

```typescript
// Fetch configuration from backend
const config = await fetch('/api/config').then(r => r.json());

// Apply configuration
if (config.theme) {
  document.documentElement.setAttribute('data-theme', config.theme);
}
```

## Feature Flags

### Backend Features

Enable features using Cargo feature flags:

```bash
# Basic NLP only
cargo run

# Advanced neural processing
cargo run --features advanced-neural

# GPU acceleration
cargo run --features advanced-neural,gpu-acceleration

# All features
cargo run --all-features
```

Available features:
- `basic-nlp`: Lightweight natural language processing (default)
- `advanced-neural`: FANN neural network integration
- `gpu-acceleration`: GPU support via CUDA/OpenCL
- `persistence`: Database persistence layer
- `metrics`: Performance metrics collection
- `security`: Enhanced security features

### Frontend Features

Control features via environment variables:

```env
# Enable advanced metrics
NEXT_PUBLIC_ADVANCED_METRICS=true

# Enable debug panel
NEXT_PUBLIC_DEBUG_PANEL=true

# Enable experimental features
NEXT_PUBLIC_EXPERIMENTAL_FEATURES=true
```

## Neural Processing Configuration

### Basic Mode

```env
NEURAL_MODE=basic
ENABLE_ADVANCED_NEURAL=false
LEARNING_RATE=0.01
MAX_AGENTS=1000
BATCH_SIZE=32
```

### Advanced Mode

```env
NEURAL_MODE=advanced
ENABLE_ADVANCED_NEURAL=true
LEARNING_RATE=0.001
MAX_ITERATIONS=1000
BATCH_SIZE=32
```

### GPU Configuration

```env
# Enable GPU acceleration (requires --features gpu-acceleration)
GPU_ACCELERATION=true
GPU_DEVICE=0
GPU_MEMORY_LIMIT=2048
CUDA_VISIBLE_DEVICES=0
```

### Adaptive Learning Configuration

```env
# Adaptive learning system
ADAPTIVE_LEARNING_ENABLED=true
LEARNING_RATE=0.01
MOMENTUM=0.9
DECAY_FACTOR=0.95
MIN_CONFIDENCE_THRESHOLD=0.7
PATTERN_RETENTION_DAYS=30
MAX_PATTERNS=10000
```

## Database Configuration

### SQLite (Default)

```env
DATABASE_URL=./data/hive_persistence.db
PERSISTENCE_CHECKPOINT_INTERVAL_MINUTES=5
MAX_SNAPSHOTS=20
COMPRESSION_ENABLED=true
ENCRYPTION_ENABLED=false
BACKUP_ENABLED=true
STORAGE_PATH=./data
COMPRESSION_LEVEL=6
BACKUP_RETENTION_DAYS=7
BACKUP_LOCATION=./data/backups
INCREMENTAL_BACKUP=true
```

### PostgreSQL

```env
DATABASE_URL=postgresql://user:password@localhost/hive
DATABASE_POOL_SIZE=20
DATABASE_TIMEOUT=60
PERSISTENCE_CHECKPOINT_INTERVAL_MINUTES=5
MAX_SNAPSHOTS=50
COMPRESSION_ENABLED=true
ENCRYPTION_ENABLED=true
BACKUP_ENABLED=true
```

### MySQL

```env
DATABASE_URL=mysql://user:password@localhost/hive
DATABASE_POOL_SIZE=20
DATABASE_TIMEOUT=60
PERSISTENCE_CHECKPOINT_INTERVAL_MINUTES=5
MAX_SNAPSHOTS=50
COMPRESSION_ENABLED=true
ENCRYPTION_ENABLED=true
BACKUP_ENABLED=true
```

### Storage Backend Configuration

```toml
[persistence]
storage_backend = "SQLite"  # or "PostgreSQL", "MySQL"
checkpoint_interval_minutes = 5
max_snapshots = 20
compression_enabled = true
encryption_enabled = false
backup_enabled = true
storage_path = "./data"
encryption_key = "optional-encryption-key"
compression_level = 6
backup_retention_days = 7
backup_location = "./data/backups"
incremental_backup = true
```

## Security Configuration

### Authentication & Authorization

```env
# JWT Configuration
JWT_SECRET=your-256-bit-secret-key-here
JWT_EXPIRATION_HOURS=24

# Rate Limiting
RATE_LIMIT_REQUESTS_PER_MINUTE=1000
RATE_LIMIT_BURST_SIZE=100
RATE_LIMIT_WINDOW_SECONDS=60

# Security Auditing
AUDIT_LOGGING_ENABLED=true
AUDIT_RETENTION_DAYS=90
SECURITY_HEADERS_ENABLED=true

# Input Validation
INPUT_VALIDATION_ENABLED=true
SANITIZATION_ENABLED=true
```

### CORS Configuration

```env
CORS_ORIGINS=http://localhost:3000,https://yourdomain.com
CORS_METHODS=GET,POST,PUT,DELETE,OPTIONS
CORS_HEADERS=Content-Type,Authorization,X-API-Key,X-Requested-With
CORS_CREDENTIALS=true
CORS_MAX_AGE=86400
```

### Security Monitoring

```env
# Security event logging
SECURITY_AUDIT_ENABLED=true
FAILED_LOGIN_ATTEMPTS_THRESHOLD=5
SUSPICIOUS_ACTIVITY_MONITORING=true

# Data protection
DATA_ENCRYPTION_ENABLED=true
ENCRYPTION_KEY_ROTATION_DAYS=30
SENSITIVE_DATA_MASKING=true
```

## Monitoring Configuration

### Metrics & Monitoring

```env
# Core monitoring
METRICS_COLLECTION_INTERVAL_MS=5000
ALERT_CHECK_INTERVAL_MS=30000
MONITORING_INTERVAL_SECS=5
METRICS_RETENTION_DAYS=7

# Alerting
ALERT_THRESHOLD=0.8
INTELLIGENT_ALERTING_ENABLED=true
ALERT_CORRELATION_ENABLED=true

# Component monitoring
ENABLE_AGENT_DISCOVERY=true
ENABLE_HEALTH_MONITORING=true
ENABLE_PERFORMANCE_MONITORING=true
ENABLE_BEHAVIOR_ANALYSIS=true
ENABLE_DASHBOARDS=true
ENABLE_ALERTING=true
ENABLE_DIAGNOSTICS=true
ENABLE_REPORTING=true
ENABLE_AUTOMATION=false
ENABLE_EXTERNAL_INTEGRATION=false
```

### Logging Configuration

```env
LOG_LEVEL=info
LOG_FORMAT=json
LOG_MAX_SIZE_MB=100
LOG_ROTATION=hourly
LOG_COMPRESSION=true
```

### Health Checks

```env
HEALTH_CHECK_ENABLED=true
HEALTH_CHECK_PATH=/health
HEALTH_CHECK_INTERVAL=30
HEALTH_CHECK_TIMEOUT=5
COMPREHENSIVE_HEALTH_CHECKS=true
DEPENDENCY_HEALTH_CHECKS=true
```

## Performance Tuning

### Resource Limits

```env
MEMORY_LIMIT_MB=1024
CPU_CORES=auto
CPU_WARNING_THRESHOLD=70.0
CPU_CRITICAL_THRESHOLD=90.0
MEMORY_WARNING_THRESHOLD=80.0
MEMORY_CRITICAL_THRESHOLD=95.0
```

### Agent Configuration

```env
MAX_AGENTS=1000
DEFAULT_ENERGY=100.0
ENERGY_DECAY_RATE=0.1
LEARNING_RATE=0.01
MAX_MEMORY_SIZE=1000
```

### Task Configuration

```env
MAX_CONCURRENT_TASKS=50
TASK_TIMEOUT_SECS=300
RETRY_ATTEMPTS=3
PRIORITY_LEVELS=4
TASK_QUEUE_SIZE=10000
```

### Circuit Breaker & Resilience

```env
CIRCUIT_BREAKER_FAILURE_THRESHOLD=5
CIRCUIT_BREAKER_RECOVERY_TIMEOUT_MS=30000
AUTO_SCALING_ENABLED=true
PERFORMANCE_OPTIMIZATION_ENABLED=true
```

## WebSocket Configuration

### Connection Settings

```env
WS_MAX_CONNECTIONS=100
WS_HEARTBEAT_INTERVAL=30
WS_TIMEOUT_SECS=300
WS_BUFFER_SIZE=65536
```

### Message Handling

```env
WS_MAX_MESSAGE_SIZE=1048576
WS_COMPRESSION_ENABLED=true
WS_COMPRESSION_LEVEL=6
WS_RATE_LIMIT_ENABLED=true
```

### MCP Configuration

```env
MCP_ENABLED=true
MCP_PORT=3002
MCP_MAX_CONNECTIONS=50
MCP_MESSAGE_TIMEOUT=30
MCP_PROTOCOL_VERSION=1.0
```

## Development Configuration

### Debug Settings

```env
DEBUG_MODE=true
LOG_LEVEL=debug
EXAMPLE_DATA_ENABLED=true
CORS_ORIGINS=*
```

### Testing Configuration

```env
TEST_DATABASE_URL=:memory:
TEST_MAX_AGENTS=10
TEST_TASK_TIMEOUT=10
```

## Configuration Validation

The system validates configuration on startup:

```bash
# Validate configuration
cargo run -- --validate-config

# Show current configuration
cargo run -- --show-config
```

### Validation Rules

- **Port ranges**: 1024-65535
- **Memory limits**: 128MB - system RAM
- **Agent counts**: 1 - 10000
- **Task timeouts**: 1 - 3600 seconds
- **Learning rates**: 0.001 - 1.0

## Configuration Management

### Environment-Specific Configs

```bash
# Development
cp settings/development.toml settings/active.toml

# Production
cp settings/production.toml settings/active.toml
```

### Configuration Hot Reload

```bash
# Trigger config reload (requires restart)
curl -X POST http://localhost:3001/api/admin/reload-config
```

### Backup Configuration

```bash
# Backup current config
cp .env .env.backup
cp settings/active.toml settings/active.toml.backup
```

## Troubleshooting Configuration

### Common Issues

#### Configuration Not Loading
```bash
# Check file permissions
ls -la .env settings/

# Validate TOML syntax
python -c "import toml; toml.load('settings/default.toml')"
```

#### Environment Variables Not Applied
```bash
# Check variable precedence
env | grep HIVE_

# Use --env-file flag
cargo run -- --env-file .env
```

#### Invalid Configuration Values
```bash
# Run validation
cargo run -- --validate-config

# Check logs for errors
tail -f hive.log
```

### Configuration Examples

#### Minimal Development Setup
```env
HIVE_PORT=3001
LOG_LEVEL=debug
NEURAL_MODE=basic
MAX_AGENTS=100
```

#### Production Setup
```env
HIVE_PORT=3001
HIVE_HOST=0.0.0.0
LOG_LEVEL=warn
NEURAL_MODE=advanced
MAX_AGENTS=10000
DATABASE_URL=postgresql://prod-db:5432/hive
JWT_SECRET=production-secret-key
METRICS_ENABLED=true
```

#### High-Performance Setup
```env
HIVE_WORKERS=16
MEMORY_LIMIT_MB=8192
CPU_CORES=16
MAX_AGENTS=50000
TASK_QUEUE_SIZE=100000
WS_MAX_CONNECTIONS=1000
GPU_ENABLED=true
```

## Next Steps

- **API Configuration**: See [docs/api/API.md](../api/API.md)
- **Deployment**: See [docs/deployment.md](deployment.md)
- **Security**: See [docs/security-hardening.md](security-hardening.md)
- **Performance**: See [docs/performance.md](performance.md)