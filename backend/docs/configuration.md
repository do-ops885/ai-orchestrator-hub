# Configuration Guide

This guide covers all configuration options for the AI Orchestrator Hub, including environment variables, configuration files, and runtime settings.

## Configuration Sources

The system supports multiple configuration sources with the following priority (highest to lowest):

1. **Environment Variables** (highest priority)
2. **Configuration Files** (TOML format)
3. **Command Line Flags**
4. **Default Values** (lowest priority)

## Environment Variables

### Server Configuration

```env
# ==========================================
# SERVER CONFIGURATION
# ==========================================

# Server host binding
HIVE_SERVER__HOST=0.0.0.0

# Server port
HIVE_SERVER__PORT=3001

# CORS allowed origins (comma-separated)
HIVE_SERVER__CORS_ORIGINS=http://localhost:3000,http://localhost:3001

# WebSocket timeout in seconds
HIVE_SERVER__WEBSOCKET_TIMEOUT_SECS=300

# Maximum concurrent connections
HIVE_SERVER__MAX_CONNECTIONS=1000

# Request timeout in seconds
HIVE_SERVER__REQUEST_TIMEOUT_SECS=30

# Shutdown timeout in seconds
HIVE_SERVER__SHUTDOWN_TIMEOUT_SECS=10
```

### Database Configuration

```env
# ==========================================
# DATABASE CONFIGURATION
# ==========================================

# Database connection URL
HIVE_DATABASE__URL=./data/hive_persistence.db

# For PostgreSQL:
# HIVE_DATABASE__URL=postgresql://user:password@localhost:5432/hive_db

# For MySQL:
# HIVE_DATABASE__URL=mysql://user:password@localhost:3306/hive_db

# Maximum database connections
HIVE_DATABASE__MAX_CONNECTIONS=10

# Connection timeout in seconds
HIVE_DATABASE__CONNECTION_TIMEOUT_SECS=30

# Enable database migrations on startup
HIVE_DATABASE__AUTO_MIGRATE=true

# Database backup settings
HIVE_DATABASE__BACKUP_ENABLED=true
HIVE_DATABASE__BACKUP_INTERVAL_HOURS=24
HIVE_DATABASE__BACKUP_RETENTION_DAYS=7
```

### Agent Configuration

```env
# ==========================================
# AGENT CONFIGURATION
# ==========================================

# Maximum number of agents
HIVE_AGENTS__MAX_AGENTS=100

# Default agent energy level
HIVE_AGENTS__DEFAULT_ENERGY=100.0

# Energy decay rate per second
HIVE_AGENTS__ENERGY_DECAY_RATE=0.1

# Learning rate for agent adaptation
HIVE_AGENTS__LEARNING_RATE=0.01

# Maximum memory size per agent
HIVE_AGENTS__MAX_MEMORY_SIZE=1000

# Agent heartbeat interval in seconds
HIVE_AGENTS__HEARTBEAT_INTERVAL_SECS=30

# Agent cleanup interval in seconds
HIVE_AGENTS__CLEANUP_INTERVAL_SECS=300

# Enable agent auto-scaling
HIVE_AGENTS__AUTO_SCALING_ENABLED=true

# Minimum agents to maintain
HIVE_AGENTS__MIN_AGENTS=5

# Maximum agents to scale to
HIVE_AGENTS__MAX_SCALE_AGENTS=500
```

### Task Configuration

```env
# ==========================================
# TASK CONFIGURATION
# ==========================================

# Maximum concurrent tasks
HIVE_TASKS__MAX_CONCURRENT_TASKS=50

# Task timeout in seconds
HIVE_TASKS__TASK_TIMEOUT_SECS=300

# Task retry attempts
HIVE_TASKS__RETRY_ATTEMPTS=3

# Number of priority levels
HIVE_TASKS__PRIORITY_LEVELS=4

# Task queue size
HIVE_TASKS__QUEUE_SIZE=10000

# Enable work stealing
HIVE_TASKS__WORK_STEALING_ENABLED=true

# Task execution pool size
HIVE_TASKS__EXECUTION_POOL_SIZE=10

# Task result retention time in hours
HIVE_TASKS__RESULT_RETENTION_HOURS=24
```

### Neural Processing Configuration

```env
# ==========================================
# NEURAL PROCESSING CONFIGURATION
# ==========================================

# Enable advanced neural processing
HIVE_NEURAL__ENABLE_ADVANCED_NEURAL=false

# Neural network batch size
HIVE_NEURAL__BATCH_SIZE=32

# Neural network learning rate
HIVE_NEURAL__LEARNING_RATE=0.001

# Maximum neural network iterations
HIVE_NEURAL__MAX_ITERATIONS=1000

# Neural network hidden layer size
HIVE_NEURAL__HIDDEN_LAYER_SIZE=128

# Enable GPU acceleration (requires CUDA)
HIVE_NEURAL__GPU_ACCELERATION_ENABLED=false

# GPU memory limit in MB
HIVE_NEURAL__GPU_MEMORY_LIMIT_MB=1024

# Neural model cache size
HIVE_NEURAL__MODEL_CACHE_SIZE=10

# Enable neural model persistence
HIVE_NEURAL__MODEL_PERSISTENCE_ENABLED=true
```

### Security Configuration

```env
# ==========================================
# SECURITY CONFIGURATION
# ==========================================

# JWT secret key (generate a secure random key)
HIVE_SECURITY__JWT_SECRET=your-super-secret-key-change-this-in-production

# JWT expiration time in hours
HIVE_SECURITY__JWT_EXPIRATION_HOURS=24

# Rate limiting: requests per minute
HIVE_SECURITY__RATE_LIMIT_REQUESTS_PER_MINUTE=1000

# Enable audit logging
HIVE_SECURITY__AUDIT_LOGGING_ENABLED=true

# Security headers enabled
HIVE_SECURITY__SECURITY_HEADERS_ENABLED=true

# CORS max age in seconds
HIVE_SECURITY__CORS_MAX_AGE_SECS=86400

# Enable HTTPS redirect
HIVE_SECURITY__HTTPS_REDIRECT_ENABLED=false

# SSL certificate path (for HTTPS)
HIVE_SECURITY__SSL_CERT_PATH=/path/to/cert.pem

# SSL key path (for HTTPS)
HIVE_SECURITY__SSL_KEY_PATH=/path/to/key.pem
```

### Monitoring Configuration

```env
# ==========================================
# MONITORING CONFIGURATION
# ==========================================

# Monitoring interval in seconds
HIVE_MONITORING__MONITORING_INTERVAL_SECS=5

# Metrics retention period in days
HIVE_MONITORING__METRICS_RETENTION_DAYS=7

# Enable agent discovery
HIVE_MONITORING__ENABLE_AGENT_DISCOVERY=true

# Enable health monitoring
HIVE_MONITORING__ENABLE_HEALTH_MONITORING=true

# Enable performance monitoring
HIVE_MONITORING__ENABLE_PERFORMANCE_MONITORING=true

# Enable behavior analysis
HIVE_MONITORING__ENABLE_BEHAVIOR_ANALYSIS=true

# Enable dashboard
HIVE_MONITORING__ENABLE_DASHBOARDS=true

# Enable alerting
HIVE_MONITORING__ENABLE_ALERTING=true

# Enable diagnostics
HIVE_MONITORING__ENABLE_DIAGNOSTICS=true

# Enable reporting
HIVE_MONITORING__ENABLE_REPORTING=true

# Alert threshold for CPU usage (%)
HIVE_MONITORING__CPU_ALERT_THRESHOLD=80.0

# Alert threshold for memory usage (%)
HIVE_MONITORING__MEMORY_ALERT_THRESHOLD=85.0

# Alert threshold for disk usage (%)
HIVE_MONITORING__DISK_ALERT_THRESHOLD=90.0
```

### Performance Configuration

```env
# ==========================================
# PERFORMANCE CONFIGURATION
# ==========================================

# CPU warning threshold (%)
HIVE_PERFORMANCE__CPU_WARNING_THRESHOLD=70.0

# CPU critical threshold (%)
HIVE_PERFORMANCE__CPU_CRITICAL_THRESHOLD=90.0

# Memory warning threshold (%)
HIVE_PERFORMANCE__MEMORY_WARNING_THRESHOLD=80.0

# Memory critical threshold (%)
HIVE_PERFORMANCE__MEMORY_CRITICAL_THRESHOLD=95.0

# Metrics collection interval in milliseconds
HIVE_PERFORMANCE__METRICS_COLLECTION_INTERVAL_MS=5000

# Alert check interval in milliseconds
HIVE_PERFORMANCE__ALERT_CHECK_INTERVAL_MS=30000

# Circuit breaker failure threshold
HIVE_PERFORMANCE__CIRCUIT_BREAKER_FAILURE_THRESHOLD=5

# Circuit breaker recovery timeout in milliseconds
HIVE_PERFORMANCE__CIRCUIT_BREAKER_RECOVERY_TIMEOUT_MS=30000

# Enable performance optimization
HIVE_PERFORMANCE__PERFORMANCE_OPTIMIZATION_ENABLED=true

# Connection pool size
HIVE_PERFORMANCE__CONNECTION_POOL_SIZE=20

# Enable caching
HIVE_PERFORMANCE__CACHING_ENABLED=true

# Cache size in MB
HIVE_PERFORMANCE__CACHE_SIZE_MB=512
```

### Logging Configuration

```env
# ==========================================
# LOGGING CONFIGURATION
# ==========================================

# Log level (trace, debug, info, warn, error)
HIVE_LOGGING__LEVEL=info

# Log format (json, pretty, compact)
HIVE_LOGGING__FORMAT=json

# Maximum log file size in MB
HIVE_LOGGING__MAX_FILE_SIZE_MB=100

# Enable console logging
HIVE_LOGGING__ENABLE_CONSOLE_LOGGING=true

# Enable file logging
HIVE_LOGGING__ENABLE_FILE_LOGGING=true

# Enable structured logging
HIVE_LOGGING__ENABLE_STRUCTURED_LOGGING=true

# Log file path
HIVE_LOGGING__LOG_FILE_PATH=./logs/hive.log

# Log rotation (daily, hourly, size)
HIVE_LOGGING__LOG_ROTATION=daily

# Maximum number of log files to keep
HIVE_LOGGING__MAX_LOG_FILES=30

# Enable request logging
HIVE_LOGGING__ENABLE_REQUEST_LOGGING=true

# Enable error logging
HIVE_LOGGING__ENABLE_ERROR_LOGGING=true
```

### Resource Management Configuration

```env
# ==========================================
# RESOURCE MANAGEMENT CONFIGURATION
# ==========================================

# CPU threshold for scaling (%)
HIVE_RESOURCES__CPU_THRESHOLD=80.0

# Memory threshold for scaling (%)
HIVE_RESOURCES__MEMORY_THRESHOLD=85.0

# Enable auto-scaling
HIVE_RESOURCES__AUTO_SCALING_ENABLED=true

# Monitoring interval in seconds
HIVE_RESOURCES__MONITORING_INTERVAL_SECS=30

# Resource allocation strategy (fair, priority, capacity)
HIVE_RESOURCES__ALLOCATION_STRATEGY=fair

# Enable resource quotas
HIVE_RESOURCES__QUOTAS_ENABLED=true

# Default CPU quota per agent (cores)
HIVE_RESOURCES__DEFAULT_CPU_QUOTA=0.5

# Default memory quota per agent (MB)
HIVE_RESOURCES__DEFAULT_MEMORY_QUOTA=256

# Enable resource overcommit
HIVE_RESOURCES__OVERCOMMIT_ENABLED=false

# Overcommit ratio
HIVE_RESOURCES__OVERCOMMIT_RATIO=1.2
```

## Configuration Files

### TOML Configuration File

Create a `config.toml` file in the backend directory:

```toml
[server]
host = "0.0.0.0"
port = 3001
cors_origins = ["http://localhost:3000"]
websocket_timeout_secs = 300
max_connections = 1000

[database]
url = "./data/hive_persistence.db"
max_connections = 10
connection_timeout_secs = 30
auto_migrate = true

[agents]
max_agents = 100
default_energy = 100.0
energy_decay_rate = 0.1
learning_rate = 0.01
max_memory_size = 1000
heartbeat_interval_secs = 30
cleanup_interval_secs = 300
auto_scaling_enabled = true
min_agents = 5
max_scale_agents = 500

[tasks]
max_concurrent_tasks = 50
task_timeout_secs = 300
retry_attempts = 3
priority_levels = 4
queue_size = 10000
work_stealing_enabled = true
execution_pool_size = 10
result_retention_hours = 24

[neural]
enable_advanced_neural = false
batch_size = 32
learning_rate = 0.001
max_iterations = 1000
hidden_layer_size = 128
gpu_acceleration_enabled = false
gpu_memory_limit_mb = 1024
model_cache_size = 10
model_persistence_enabled = true

[security]
jwt_secret = "your-super-secret-key-change-this-in-production"
jwt_expiration_hours = 24
rate_limit_requests_per_minute = 1000
audit_logging_enabled = true
security_headers_enabled = true
cors_max_age_secs = 86400
https_redirect_enabled = false

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
cpu_alert_threshold = 80.0
memory_alert_threshold = 85.0
disk_alert_threshold = 90.0

[performance]
cpu_warning_threshold = 70.0
cpu_critical_threshold = 90.0
memory_warning_threshold = 80.0
memory_critical_threshold = 95.0
metrics_collection_interval_ms = 5000
alert_check_interval_ms = 30000
circuit_breaker_failure_threshold = 5
circuit_breaker_recovery_timeout_ms = 30000
performance_optimization_enabled = true
connection_pool_size = 20
caching_enabled = true
cache_size_mb = 512

[logging]
level = "info"
format = "json"
max_file_size_mb = 100
enable_console_logging = true
enable_file_logging = true
enable_structured_logging = true
log_file_path = "./logs/hive.log"
log_rotation = "daily"
max_log_files = 30
enable_request_logging = true
enable_error_logging = true

[resources]
cpu_threshold = 80.0
memory_threshold = 85.0
auto_scaling_enabled = true
monitoring_interval_secs = 30
allocation_strategy = "fair"
quotas_enabled = true
default_cpu_quota = 0.5
default_memory_quota = 256.0
overcommit_enabled = false
overcommit_ratio = 1.2
```

### Loading Configuration Files

```rust
use config::{Config, File};

// Load configuration
let settings = Config::builder()
    .add_source(File::with_name("config.toml"))
    .add_source(config::Environment::with_prefix("HIVE"))
    .build()
    .unwrap();

// Access configuration values
let host = settings.get_string("server.host").unwrap_or("0.0.0.0".to_string());
let port = settings.get_int("server.port").unwrap_or(3001);
```

## Command Line Flags

The application supports various command line flags:

```bash
# Basic usage
cargo run -- --host 0.0.0.0 --port 3001

# With configuration file
cargo run -- --config config.toml

# Enable features
cargo run -- --features advanced-neural

# Debug mode
cargo run -- --log-level debug

# Help
cargo run -- --help
```

Available command line options:

- `--host <HOST>`: Server host binding [default: 0.0.0.0]
- `--port <PORT>`: Server port [default: 3001]
- `--config <FILE>`: Configuration file path
- `--log-level <LEVEL>`: Logging level [default: info]
- `--features <FEATURES>`: Enable specific features
- `--help`: Print help information
- `--version`: Print version information

## Runtime Configuration

### Dynamic Configuration Updates

The system supports dynamic configuration updates without restart:

```bash
# Update configuration via API
curl -X POST http://localhost:3001/api/config \
  -H "Content-Type: application/json" \
  -d '{
    "logging.level": "debug",
    "monitoring.monitoring_interval_secs": 10
  }'
```

### Configuration Validation

The system validates configuration on startup:

```bash
# Validate configuration
cargo run -- --validate-config

# Check for deprecated settings
cargo run -- --check-deprecated
```

## Environment-Specific Configuration

### Development Configuration

```env
# Development settings
HIVE_LOGGING__LEVEL=debug
HIVE_MONITORING__MONITORING_INTERVAL_SECS=1
HIVE_DATABASE__URL=./data/hive_dev.db
HIVE_SERVER__CORS_ORIGINS=http://localhost:3000,http://localhost:3001
```

### Production Configuration

```env
# Production settings
HIVE_LOGGING__LEVEL=warn
HIVE_MONITORING__MONITORING_INTERVAL_SECS=30
HIVE_DATABASE__URL=postgresql://user:password@prod-db:5432/hive_db
HIVE_SECURITY__JWT_SECRET=your-production-secret-key
HIVE_SECURITY__HTTPS_REDIRECT_ENABLED=true
HIVE_PERFORMANCE__PERFORMANCE_OPTIMIZATION_ENABLED=true
```

### Testing Configuration

```env
# Testing settings
HIVE_LOGGING__LEVEL=debug
HIVE_DATABASE__URL=:memory:
HIVE_AGENTS__MAX_AGENTS=10
HIVE_TASKS__MAX_CONCURRENT_TASKS=5
HIVE_MONITORING__ENABLE_ALERTING=false
```

## Configuration Best Practices

### Security

1. **Never commit secrets** to version control
2. **Use environment variables** for sensitive data
3. **Rotate secrets regularly**
4. **Use strong, random keys** for JWT and encryption
5. **Enable audit logging** in production

### Performance

1. **Tune pool sizes** based on your workload
2. **Enable caching** for frequently accessed data
3. **Configure appropriate timeouts**
4. **Monitor resource usage** and adjust thresholds
5. **Use connection pooling** for databases

### Monitoring

1. **Enable comprehensive monitoring** in production
2. **Set appropriate alert thresholds**
3. **Configure log rotation** to prevent disk space issues
4. **Use structured logging** for better analysis
5. **Enable metrics collection** for performance tracking

### Scalability

1. **Configure auto-scaling** based on your needs
2. **Set appropriate resource limits**
3. **Use external databases** for production
4. **Configure load balancing** for high availability
5. **Enable circuit breakers** for fault tolerance

## Configuration Examples

### Minimal Configuration

```env
HIVE_SERVER__PORT=3001
HIVE_DATABASE__URL=./data/hive.db
HIVE_LOGGING__LEVEL=info
```

### Full Production Configuration

```env
# Server
HIVE_SERVER__HOST=0.0.0.0
HIVE_SERVER__PORT=3001
HIVE_SERVER__CORS_ORIGINS=https://yourdomain.com
HIVE_SERVER__MAX_CONNECTIONS=10000

# Database
HIVE_DATABASE__URL=postgresql://hive_user:secure_password@db.yourdomain.com:5432/hive_prod
HIVE_DATABASE__MAX_CONNECTIONS=50
HIVE_DATABASE__AUTO_MIGRATE=true

# Security
HIVE_SECURITY__JWT_SECRET=your-256-bit-secret-key-here
HIVE_SECURITY__RATE_LIMIT_REQUESTS_PER_MINUTE=5000
HIVE_SECURITY__AUDIT_LOGGING_ENABLED=true
HIVE_SECURITY__HTTPS_REDIRECT_ENABLED=true

# Performance
HIVE_PERFORMANCE__CPU_WARNING_THRESHOLD=75.0
HIVE_PERFORMANCE__MEMORY_WARNING_THRESHOLD=80.0
HIVE_PERFORMANCE__CIRCUIT_BREAKER_ENABLED=true
HIVE_PERFORMANCE__CONNECTION_POOL_SIZE=100
HIVE_PERFORMANCE__CACHING_ENABLED=true
HIVE_PERFORMANCE__CACHE_SIZE_MB=2048

# Monitoring
HIVE_MONITORING__MONITORING_INTERVAL_SECS=10
HIVE_MONITORING__METRICS_RETENTION_DAYS=30
HIVE_MONITORING__ENABLE_AGENT_DISCOVERY=true
HIVE_MONITORING__ENABLE_HEALTH_MONITORING=true
HIVE_MONITORING__ENABLE_PERFORMANCE_MONITORING=true
HIVE_MONITORING__ENABLE_ALERTING=true

# Agents
HIVE_AGENTS__MAX_AGENTS=1000
HIVE_AGENTS__AUTO_SCALING_ENABLED=true
HIVE_AGENTS__MIN_AGENTS=10
HIVE_AGENTS__MAX_SCALE_AGENTS=2000

# Tasks
HIVE_TASKS__MAX_CONCURRENT_TASKS=500
HIVE_TASKS__WORK_STEALING_ENABLED=true
HIVE_TASKS__EXECUTION_POOL_SIZE=50

# Logging
HIVE_LOGGING__LEVEL=warn
HIVE_LOGGING__FORMAT=json
HIVE_LOGGING__ENABLE_FILE_LOGGING=true
HIVE_LOGGING__LOG_FILE_PATH=/var/log/hive/hive.log
HIVE_LOGGING__LOG_ROTATION=daily
HIVE_LOGGING__MAX_LOG_FILES=30
```

## Troubleshooting Configuration

### Common Configuration Issues

#### Configuration Not Loading

```bash
# Check file permissions
ls -la config.toml

# Validate TOML syntax
cargo run -- --validate-config config.toml

# Check environment variables
env | grep HIVE_
```

#### Invalid Configuration Values

```bash
# Check configuration validation
cargo run -- --validate-config

# View current configuration
curl http://localhost:3001/api/config
```

#### Performance Issues

```bash
# Check current metrics
curl http://localhost:3001/metrics

# Adjust performance settings
curl -X POST http://localhost:3001/api/config \
  -H "Content-Type: application/json" \
  -d '{"performance.connection_pool_size": 50}'
```

For more information, see the [Installation Guide](installation.md) and [Deployment Guide](deployment.md).