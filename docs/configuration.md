# Configuration Guide

This guide covers all configuration options for the AI Orchestrator Hub, including environment variables, configuration files, and runtime settings.

## Table of Contents

- [Configuration Overview](#configuration-overview)
- [Configuration Files](#configuration-files)
- [Environment Variables](#environment-variables)
- [Server Configuration](#server-configuration)
- [Database Configuration](#database-configuration)
- [Agent Configuration](#agent-configuration)
- [Task Configuration](#task-configuration)
- [Neural Processing Configuration](#neural-processing-configuration)
- [Security Configuration](#security-configuration)
- [Monitoring Configuration](#monitoring-configuration)
- [Performance Configuration](#performance-configuration)
- [Logging Configuration](#logging-configuration)
- [Feature Flags](#feature-flags)
- [Runtime Configuration](#runtime-configuration)
- [Configuration Validation](#configuration-validation)
- [Troubleshooting](#troubleshooting)

## Configuration Overview

The AI Orchestrator Hub uses a hierarchical configuration system:

1. **Default Values**: Built-in defaults for all settings
2. **Configuration Files**: TOML files for structured configuration
3. **Environment Variables**: Override specific settings
4. **Runtime Configuration**: Dynamic updates for some settings

### Configuration Priority

```
Environment Variables > Configuration Files > Default Values
```

### Configuration Files Location

```
backend/
├── settings/
│   ├── default.toml      # Default configuration
│   ├── development.toml  # Development overrides
│   └── production.toml   # Production overrides
└── .env                  # Environment variables
```

## Configuration Files

### Default Configuration

The `settings/default.toml` file contains all default settings:

```toml
# Server Configuration
[server]
host = "0.0.0.0"
port = 3001
cors_origins = ["http://localhost:3000"]
websocket_timeout_secs = 300
max_connections = 1000

# Agent Configuration
[agents]
max_agents = 100
default_energy = 100.0
energy_decay_rate = 0.1
learning_rate = 0.01
max_memory_size = 1000

# Task Configuration
[tasks]
max_concurrent_tasks = 50
task_timeout_secs = 300
retry_attempts = 3
priority_levels = 4

# Resource Management
[resources]
cpu_threshold = 80.0
memory_threshold = 85.0
auto_scaling_enabled = true
monitoring_interval_secs = 30

# Neural Processing
[neural]
enable_advanced_neural = false
batch_size = 32
learning_rate = 0.001
max_iterations = 1000

# Logging
[logging]
level = "info"
format = "json"
max_file_size_mb = 100

# Performance
[performance]
cpu_warning_threshold = 70.0
cpu_critical_threshold = 90.0
memory_warning_threshold = 80.0
memory_critical_threshold = 95.0
metrics_collection_interval_ms = 5000
alert_check_interval_ms = 30000
circuit_breaker_failure_threshold = 5
circuit_breaker_recovery_timeout_ms = 30000

# Monitoring
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

### Environment-Specific Configuration

#### Development Configuration (`settings/development.toml`)

```toml
# Development overrides
[server]
port = 3001

[logging]
level = "debug"
format = "pretty"

[monitoring]
enable_diagnostics = true
enable_reporting = true

[neural]
enable_advanced_neural = false
```

#### Production Configuration (`settings/production.toml`)

```toml
# Production overrides
[server]
host = "0.0.0.0"
port = 3001
max_connections = 10000

[logging]
level = "warn"
format = "json"

[performance]
cpu_warning_threshold = 75.0
memory_warning_threshold = 85.0

[monitoring]
metrics_retention_days = 30
enable_diagnostics = false
```

## Environment Variables

### Server Configuration

```env
# Server settings
HIVE_SERVER__HOST=0.0.0.0
HIVE_SERVER__PORT=3001
HIVE_SERVER__CORS_ORIGINS=["http://localhost:3000","https://myapp.com"]
HIVE_SERVER__WEBSOCKET_TIMEOUT_SECS=300
HIVE_SERVER__MAX_CONNECTIONS=1000
```

### Database Configuration

```env
# SQLite (default)
HIVE_DATABASE__URL=./data/hive_persistence.db

# PostgreSQL
HIVE_DATABASE__URL=postgresql://user:password@localhost/hive_db
HIVE_DATABASE__MAX_CONNECTIONS=20
HIVE_DATABASE__MIN_CONNECTIONS=5
HIVE_DATABASE__CONNECTION_TIMEOUT_SECS=30
```

### Agent Configuration

```env
# Agent settings
HIVE_AGENTS__MAX_AGENTS=100
HIVE_AGENTS__DEFAULT_ENERGY=100.0
HIVE_AGENTS__ENERGY_DECAY_RATE=0.1
HIVE_AGENTS__LEARNING_RATE=0.01
HIVE_AGENTS__MAX_MEMORY_SIZE=1000
```

### Task Configuration

```env
# Task settings
HIVE_TASKS__MAX_CONCURRENT_TASKS=50
HIVE_TASKS__TASK_TIMEOUT_SECS=300
HIVE_TASKS__RETRY_ATTEMPTS=3
HIVE_TASKS__PRIORITY_LEVELS=4
```

### Neural Processing

```env
# Neural settings
HIVE_NEURAL__ENABLE_ADVANCED_NEURAL=false
HIVE_NEURAL__BATCH_SIZE=32
HIVE_NEURAL__LEARNING_RATE=0.001
HIVE_NEURAL__MAX_ITERATIONS=1000
HIVE_NEURAL__GPU_ENABLED=false
```

### Security Configuration

```env
# Security settings
HIVE_SECURITY__JWT_SECRET=your-256-bit-secret-key-here
HIVE_SECURITY__JWT_EXPIRATION_HOURS=24
HIVE_SECURITY__RATE_LIMIT_REQUESTS_PER_MINUTE=1000
HIVE_SECURITY__AUDIT_LOGGING_ENABLED=true
HIVE_SECURITY__ENCRYPTION_ENABLED=true
HIVE_SECURITY__CORS_MAX_AGE=86400
```

### Monitoring Configuration

```env
# Monitoring settings
HIVE_MONITORING__MONITORING_INTERVAL_SECS=5
HIVE_MONITORING__METRICS_RETENTION_DAYS=7
HIVE_MONITORING__ENABLE_AGENT_DISCOVERY=true
HIVE_MONITORING__ENABLE_HEALTH_MONITORING=true
HIVE_MONITORING__ENABLE_PERFORMANCE_MONITORING=true
HIVE_MONITORING__ENABLE_BEHAVIOR_ANALYSIS=true
HIVE_MONITORING__ENABLE_DASHBOARDS=true
HIVE_MONITORING__ENABLE_ALERTING=true
HIVE_MONITORING__ENABLE_DIAGNOSTICS=true
HIVE_MONITORING__ENABLE_REPORTING=true
HIVE_MONITORING__ENABLE_AUTOMATION=false
HIVE_MONITORING__ENABLE_EXTERNAL_INTEGRATION=false
```

### Performance Configuration

```env
# Performance settings
HIVE_PERFORMANCE__CPU_WARNING_THRESHOLD=70.0
HIVE_PERFORMANCE__CPU_CRITICAL_THRESHOLD=90.0
HIVE_PERFORMANCE__MEMORY_WARNING_THRESHOLD=80.0
HIVE_PERFORMANCE__MEMORY_CRITICAL_THRESHOLD=95.0
HIVE_PERFORMANCE__METRICS_COLLECTION_INTERVAL_MS=5000
HIVE_PERFORMANCE__ALERT_CHECK_INTERVAL_MS=30000
HIVE_PERFORMANCE__CIRCUIT_BREAKER_FAILURE_THRESHOLD=5
HIVE_PERFORMANCE__CIRCUIT_BREAKER_RECOVERY_TIMEOUT_MS=30000
HIVE_PERFORMANCE__PERFORMANCE_OPTIMIZATION_ENABLED=true
```

### Logging Configuration

```env
# Logging settings
HIVE_LOGGING__LEVEL=info
HIVE_LOGGING__FORMAT=json
HIVE_LOGGING__MAX_FILE_SIZE_MB=100
HIVE_LOGGING__FILE_PATH=/var/log/ai-orchestrator-hub.log
HIVE_LOGGING__ENABLE_CONSOLE_LOGGING=true
HIVE_LOGGING__ENABLE_FILE_LOGGING=true
HIVE_LOGGING__ENABLE_STRUCTURED_LOGGING=true
```

## Server Configuration

### Host and Port

```toml
[server]
host = "0.0.0.0"          # Listen on all interfaces
port = 3001               # Server port
```

```env
HIVE_SERVER__HOST=127.0.0.1
HIVE_SERVER__PORT=8080
```

### CORS Configuration

```toml
[server]
cors_origins = [
    "http://localhost:3000",
    "https://myapp.com"
]
```

```env
HIVE_SERVER__CORS_ORIGINS='["http://localhost:3000","https://myapp.com"]'
```

### Connection Limits

```toml
[server]
max_connections = 1000
websocket_timeout_secs = 300
```

## Database Configuration

### SQLite Configuration

```toml
[database]
url = "./data/hive_persistence.db"
max_connections = 10
```

### PostgreSQL Configuration

```toml
[database]
url = "postgresql://user:password@localhost/hive_db"
max_connections = 20
min_connections = 5
connection_timeout_seconds = 30
ssl_mode = "require"
```

### Database Migration

```bash
# Run migrations
cd backend
cargo run --bin migrate

# Check migration status
cargo run --bin migrate -- status

# Rollback migration
cargo run --bin migrate -- rollback
```

## Agent Configuration

### Basic Agent Settings

```toml
[agents]
max_agents = 100
default_energy = 100.0
energy_decay_rate = 0.1
learning_rate = 0.01
max_memory_size = 1000
```

### Agent Types Configuration

```toml
[agents.types]
worker = { max_instances = 50, default_capabilities = ["general_tasks"] }
coordinator = { max_instances = 10, default_capabilities = ["coordination"] }
specialist = { max_instances = 30, default_capabilities = ["analysis"] }
learner = { max_instances = 10, default_capabilities = ["learning"] }
```

### Agent Capabilities

```toml
[agents.capabilities]
data_processing = { max_proficiency = 1.0, learning_curve = "linear" }
analysis = { max_proficiency = 1.0, learning_curve = "exponential" }
coordination = { max_proficiency = 0.9, learning_curve = "logarithmic" }
learning = { max_proficiency = 0.8, learning_curve = "sigmoid" }
```

## Task Configuration

### Task Processing

```toml
[tasks]
max_concurrent_tasks = 50
task_timeout_secs = 300
retry_attempts = 3
priority_levels = 4
work_stealing_enabled = true
```

### Task Queues

```toml
[tasks.queues]
high_priority = { capacity = 100, timeout_multiplier = 1.0 }
medium_priority = { capacity = 500, timeout_multiplier = 1.5 }
low_priority = { capacity = 1000, timeout_multiplier = 2.0 }
```

### Task Scheduling

```toml
[tasks.scheduling]
fair_scheduling = true
load_balancing = "round_robin"
preemption_enabled = false
deadline_scheduling = true
```

## Neural Processing Configuration

### Basic NLP Configuration

```toml
[neural]
enable_advanced_neural = false
nlp_batch_size = 32
nlp_learning_rate = 0.001
nlp_max_tokens = 512
nlp_model_path = "./models/nlp"
```

### Advanced Neural Configuration

```toml
[neural]
enable_advanced_neural = true
fann_networks_enabled = true
gpu_acceleration = false
neural_batch_size = 64
neural_learning_rate = 0.0001
neural_epochs = 1000
neural_hidden_layers = [128, 64, 32]
```

### GPU Configuration

```toml
[neural.gpu]
enabled = true
device_id = 0
memory_limit_mb = 4096
compute_capability = "7.0"
cudnn_enabled = true
```

## Security Configuration

### Authentication

```toml
[security]
jwt_secret = "your-256-bit-secret-key-here"
jwt_expiration_hours = 24
authentication_required = false
allow_anonymous_read = true
```

### Authorization

```toml
[security.authorization]
enable_rbac = true
default_role = "user"
admin_role = "admin"
roles = ["user", "admin", "operator"]
```

### Rate Limiting

```toml
[security.rate_limiting]
enabled = true
requests_per_minute = 1000
burst_limit = 100
window_seconds = 60
```

### Encryption

```toml
[security.encryption]
enabled = true
algorithm = "AES-256-GCM"
key_rotation_days = 30
data_encryption = true
communication_encryption = true
```

## Monitoring Configuration

### Metrics Collection

```toml
[monitoring]
enabled = true
collection_interval_seconds = 5
retention_days = 7
export_format = "prometheus"
```

### Alerting

```toml
[monitoring.alerts]
enabled = true
email_notifications = false
webhook_notifications = true
slack_notifications = false
alert_thresholds = {
    cpu_usage = 80.0,
    memory_usage = 85.0,
    disk_usage = 90.0
}
```

### Dashboards

```toml
[monitoring.dashboards]
enabled = true
auto_refresh_seconds = 30
default_time_range = "1h"
custom_dashboards_enabled = true
```

## Performance Configuration

### Resource Management

```toml
[performance]
cpu_warning_threshold = 70.0
cpu_critical_threshold = 90.0
memory_warning_threshold = 80.0
memory_critical_threshold = 95.0
disk_warning_threshold = 85.0
disk_critical_threshold = 95.0
```

### Circuit Breaker

```toml
[performance.circuit_breaker]
enabled = true
failure_threshold = 5
recovery_timeout_ms = 30000
monitoring_window_ms = 60000
```

### Caching

```toml
[performance.caching]
enabled = true
default_ttl_seconds = 300
max_cache_size_mb = 512
cache_strategy = "lru"
redis_enabled = false
```

## Logging Configuration

### Basic Logging

```toml
[logging]
level = "info"
format = "json"
enable_console = true
enable_file = true
max_file_size_mb = 100
max_files = 5
```

### Structured Logging

```toml
[logging.structured]
enabled = true
include_timestamps = true
include_request_id = true
include_user_id = false
include_correlation_id = true
custom_fields = ["component", "operation"]
```

### Log Levels

```toml
[logging.levels]
default = "info"
hive = "debug"
agents = "info"
tasks = "info"
neural = "warn"
security = "warn"
```

## Feature Flags

### Build-time Features

```bash
# Basic NLP only (default)
cargo build

# Advanced neural processing
cargo build --features advanced-neural

# GPU acceleration
cargo build --features advanced-neural,gpu-acceleration

# All features
cargo build --all-features
```

### Runtime Features

```toml
[features]
websocket_enabled = true
mcp_enabled = true
persistence_enabled = true
monitoring_enabled = true
security_enabled = true
neural_enabled = false
gpu_enabled = false
```

## Runtime Configuration

### Dynamic Configuration Updates

```bash
# Update configuration via API
curl -X POST http://localhost:3001/api/config \
  -H "Content-Type: application/json" \
  -d '{
    "logging.level": "debug",
    "monitoring.collection_interval_seconds": 10
  }'
```

### Configuration Hot Reload

```toml
[config]
hot_reload_enabled = true
reload_interval_seconds = 30
validate_on_reload = true
backup_on_reload = true
```

## Configuration Validation

### Validation Rules

```toml
[validation]
strict_mode = true
validate_on_startup = true
validate_on_reload = true
fail_on_invalid_config = true
```

### Configuration Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "server": {
      "type": "object",
      "properties": {
        "host": { "type": "string" },
        "port": { "type": "integer", "minimum": 1, "maximum": 65535 }
      },
      "required": ["host", "port"]
    }
  }
}
```

### Configuration Testing

```bash
# Validate configuration
cd backend
cargo run --bin config-validator -- settings/default.toml

# Test configuration loading
cargo test config_tests

# Check configuration conflicts
cargo run --bin config-check
```

## Sample .env File

Create a `.env.example` file in the backend directory:

```env
# AI Orchestrator Hub Configuration
# Copy this file to .env and update values as needed

# ==========================================
# SERVER CONFIGURATION
# ==========================================
HIVE_SERVER__HOST=0.0.0.0
HIVE_SERVER__PORT=3001
HIVE_SERVER__CORS_ORIGINS=["http://localhost:3000"]
HIVE_SERVER__WEBSOCKET_TIMEOUT_SECS=300
HIVE_SERVER__MAX_CONNECTIONS=1000

# ==========================================
# DATABASE CONFIGURATION
# ==========================================
HIVE_DATABASE__URL=./data/hive_persistence.db
# For PostgreSQL:
# HIVE_DATABASE__URL=postgresql://user:password@localhost/hive_db
HIVE_DATABASE__MAX_CONNECTIONS=10

# ==========================================
# AGENT CONFIGURATION
# ==========================================
HIVE_AGENTS__MAX_AGENTS=100
HIVE_AGENTS__DEFAULT_ENERGY=100.0
HIVE_AGENTS__ENERGY_DECAY_RATE=0.1
HIVE_AGENTS__LEARNING_RATE=0.01
HIVE_AGENTS__MAX_MEMORY_SIZE=1000

# ==========================================
# TASK CONFIGURATION
# ==========================================
HIVE_TASKS__MAX_CONCURRENT_TASKS=50
HIVE_TASKS__TASK_TIMEOUT_SECS=300
HIVE_TASKS__RETRY_ATTEMPTS=3
HIVE_TASKS__PRIORITY_LEVELS=4

# ==========================================
# NEURAL PROCESSING
# ==========================================
HIVE_NEURAL__ENABLE_ADVANCED_NEURAL=false
HIVE_NEURAL__BATCH_SIZE=32
HIVE_NEURAL__LEARNING_RATE=0.001
HIVE_NEURAL__MAX_ITERATIONS=1000

# ==========================================
# SECURITY CONFIGURATION
# ==========================================
HIVE_SECURITY__JWT_SECRET=your-super-secret-key-change-this-in-production
HIVE_SECURITY__JWT_EXPIRATION_HOURS=24
HIVE_SECURITY__RATE_LIMIT_REQUESTS_PER_MINUTE=1000
HIVE_SECURITY__AUDIT_LOGGING_ENABLED=true

# ==========================================
# MONITORING CONFIGURATION
# ==========================================
HIVE_MONITORING__MONITORING_INTERVAL_SECS=5
HIVE_MONITORING__METRICS_RETENTION_DAYS=7
HIVE_MONITORING__ENABLE_AGENT_DISCOVERY=true
HIVE_MONITORING__ENABLE_HEALTH_MONITORING=true
HIVE_MONITORING__ENABLE_PERFORMANCE_MONITORING=true
HIVE_MONITORING__ENABLE_BEHAVIOR_ANALYSIS=true
HIVE_MONITORING__ENABLE_DASHBOARDS=true
HIVE_MONITORING__ENABLE_ALERTING=true
HIVE_MONITORING__ENABLE_DIAGNOSTICS=true
HIVE_MONITORING__ENABLE_REPORTING=true

# ==========================================
# PERFORMANCE CONFIGURATION
# ==========================================
HIVE_PERFORMANCE__CPU_WARNING_THRESHOLD=70.0
HIVE_PERFORMANCE__CPU_CRITICAL_THRESHOLD=90.0
HIVE_PERFORMANCE__MEMORY_WARNING_THRESHOLD=80.0
HIVE_PERFORMANCE__MEMORY_CRITICAL_THRESHOLD=95.0
HIVE_PERFORMANCE__METRICS_COLLECTION_INTERVAL_MS=5000
HIVE_PERFORMANCE__ALERT_CHECK_INTERVAL_MS=30000
HIVE_PERFORMANCE__CIRCUIT_BREAKER_FAILURE_THRESHOLD=5
HIVE_PERFORMANCE__CIRCUIT_BREAKER_RECOVERY_TIMEOUT_MS=30000

# ==========================================
# LOGGING CONFIGURATION
# ==========================================
HIVE_LOGGING__LEVEL=info
HIVE_LOGGING__FORMAT=json
HIVE_LOGGING__MAX_FILE_SIZE_MB=100
HIVE_LOGGING__ENABLE_CONSOLE_LOGGING=true
HIVE_LOGGING__ENABLE_FILE_LOGGING=true
HIVE_LOGGING__ENABLE_STRUCTURED_LOGGING=true

# ==========================================
# RESOURCE MANAGEMENT
# ==========================================
HIVE_RESOURCES__CPU_THRESHOLD=80.0
HIVE_RESOURCES__MEMORY_THRESHOLD=85.0
HIVE_RESOURCES__AUTO_SCALING_ENABLED=true
HIVE_RESOURCES__MONITORING_INTERVAL_SECS=30

# ==========================================
# EXTERNAL INTEGRATIONS
# ==========================================
# OpenAI API Key (optional)
OPENAI_API_KEY=your-openai-api-key-here

# Anthropic API Key (optional)
ANTHROPIC_API_KEY=your-anthropic-api-key-here

# Redis URL (optional)
REDIS_URL=redis://localhost:6379

# Prometheus metrics endpoint (optional)
PROMETHEUS_PUSHGATEWAY_URL=http://localhost:9091
```

## Troubleshooting

### Configuration Loading Issues

```bash
# Check configuration file syntax
cd backend
cargo run --bin config-validator -- settings/default.toml

# Debug configuration loading
RUST_LOG=config=debug cargo run

# Validate environment variables
env | grep HIVE_
```

### Common Configuration Problems

#### Invalid TOML Syntax

```toml
# Incorrect
[server]
host = "localhost"
port = "3001"  # String instead of integer

# Correct
[server]
host = "localhost"
port = 3001
```

#### Environment Variable Format

```bash
# Incorrect
HIVE_SERVER__CORS_ORIGINS=http://localhost:3000

# Correct
HIVE_SERVER__CORS_ORIGINS='["http://localhost:3000"]'
```

#### Permission Issues

```bash
# Fix configuration file permissions
chmod 644 settings/*.toml

# Fix data directory permissions
chmod 755 data/
```

### Configuration Debugging

```bash
# Print loaded configuration
curl http://localhost:3001/debug/config

# Check configuration validation
curl http://localhost:3001/health

# View configuration in logs
RUST_LOG=config=trace cargo run
```

### Performance Tuning

```toml
# High-performance configuration
[performance]
metrics_collection_interval_ms = 1000
alert_check_interval_ms = 5000
circuit_breaker_failure_threshold = 10

[server]
max_connections = 10000

[agents]
max_agents = 1000

[tasks]
max_concurrent_tasks = 500
```

---

This configuration guide covers all aspects of configuring the AI Orchestrator Hub. For specific use cases or advanced configurations, please refer to the [troubleshooting guide](troubleshooting.md) or open an issue on GitHub.
