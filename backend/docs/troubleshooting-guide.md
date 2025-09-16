# Troubleshooting Guide

This comprehensive troubleshooting guide helps operators and developers diagnose and resolve issues in the AI Orchestrator Hub system. Follow the systematic approach outlined below for efficient problem resolution.

## Diagnostic Methodology

### 1. Problem Assessment
```bash
# Gather initial information
echo "Problem: $(date)"
echo "System: $(uname -a)"
echo "User: $(whoami)"
echo "Working Directory: $(pwd)"
echo "Process Status: $(ps aux | grep ai-orchestrator-hub | grep -v grep)"
```

### 2. Health Check Verification
```bash
# Basic health check
curl -s http://localhost:3001/health | jq .

# Detailed health check
curl -s "http://localhost:3001/health?detailed=true" | jq .

# System metrics
curl -s http://localhost:3001/metrics | jq '.data.current_metrics'
```

### 3. Log Analysis
```bash
# Recent application logs
sudo journalctl -u ai-orchestrator-hub -n 100 --no-pager

# Error patterns
sudo journalctl -u ai-orchestrator-hub -p err -n 50 --no-pager

# Time-based log analysis
sudo journalctl -u ai-orchestrator-hub --since "1 hour ago" --no-pager
```

## Common Issues and Solutions

### Issue 1: Service Won't Start

#### Symptoms
- Service fails to start after deployment
- Systemd reports activation failure
- Application exits immediately

#### Diagnostic Steps
```bash
# Check systemd status
sudo systemctl status ai-orchestrator-hub

# Check application logs
sudo journalctl -u ai-orchestrator-hub -n 50

# Test configuration
sudo -u ai-orchestrator /opt/ai-orchestrator-hub/ai-orchestrator-hub --config settings/production.toml --check

# Check dependencies
ldd /opt/ai-orchestrator-hub/ai-orchestrator-hub

# Verify file permissions
ls -la /opt/ai-orchestrator-hub/
ls -la /var/lib/ai-orchestrator-hub/
```

#### Common Causes and Solutions

**Configuration Error**
```bash
# Check configuration syntax
sudo -u ai-orchestrator /opt/ai-orchestrator-hub/ai-orchestrator-hub --config settings/production.toml --check

# Validate TOML syntax
python3 -c "import tomllib; tomllib.load(open('settings/production.toml', 'rb'))"

# Check environment variables
env | grep HIVE_
```

**Port Already in Use**
```bash
# Find process using port 3001
sudo netstat -tulpn | grep :3001
sudo lsof -i :3001

# Kill conflicting process
sudo kill -9 <PID>

# Or change port in configuration
echo 'port = 3002' >> settings/production.toml
```

**Database Connection Failure**
```bash
# Test database connectivity
psql -h localhost -U ai_orchestrator -d hive_db -c "SELECT 1;"

# Check database service
sudo systemctl status postgresql

# Verify connection string
grep DATABASE_URL settings/production.toml
```

**Permission Issues**
```bash
# Check file ownership
ls -la /opt/ai-orchestrator-hub/
ls -la /var/lib/ai-orchestrator-hub/

# Fix permissions
sudo chown -R ai-orchestrator:ai-orchestrator /opt/ai-orchestrator-hub/
sudo chown -R ai-orchestrator:ai-orchestrator /var/lib/ai-orchestrator-hub/
```

### Issue 2: High Memory Usage

#### Symptoms
- System memory usage >80%
- Application becomes unresponsive
- Out of memory errors in logs

#### Diagnostic Steps
```bash
# Check current memory usage
free -h
ps aux --sort=-%mem | head -10

# Application memory details
ps -p $(pgrep ai-orchestrator-hub) -o pid,ppid,cmd,%mem,%cpu

# System memory info
cat /proc/meminfo

# Application logs for memory issues
sudo journalctl -u ai-orchestrator-hub | grep -i memory
```

#### Solutions

**Memory Leak Investigation**
```bash
# Enable memory profiling (development)
export RUSTFLAGS="-g"
cargo build --release --features memory-profiling

# Use valgrind for memory analysis
valgrind --tool=massif --massif-out-file=massif.out ./target/release/ai-orchestrator-hub
ms_print massif.out | head -50

# Check for memory leaks
valgrind --leak-check=full ./target/release/ai-orchestrator-hub --config settings/test.toml
```

**Configuration Adjustments**
```toml
# Reduce memory limits
[performance]
memory_limit_mb = 2048
memory_warning_threshold = 75.0

[neural]
max_agents = 500  # Reduce from default 1000

[database]
max_connections = 10  # Reduce connection pool
```

**Memory Optimization**
```bash
# Clear system cache
echo 3 | sudo tee /proc/sys/vm/drop_caches

# Adjust swappiness
echo 10 | sudo tee /proc/sys/vm/swappiness

# Enable memory overcommit
echo 1 | sudo tee /proc/sys/vm/overcommit_memory
```

### Issue 3: High CPU Usage

#### Symptoms
- CPU usage >80% sustained
- Slow response times
- System becomes unresponsive

#### Diagnostic Steps
```bash
# Check CPU usage
top -p $(pgrep ai-orchestrator-hub)
ps aux --sort=-%cpu | head -10

# System load average
uptime
cat /proc/loadavg

# CPU info
lscpu
cat /proc/cpuinfo | grep -c processor
```

#### Solutions

**Performance Profiling**
```bash
# CPU profiling with flamegraph
cargo flamegraph --bin ai-orchestrator-hub -- --config settings/production.toml

# perf profiling
sudo perf record -p $(pgrep ai-orchestrator-hub) -g -- sleep 30
sudo perf report

# strace for system calls
strace -p $(pgrep ai-orchestrator-hub) -c
```

**Configuration Optimization**
```toml
[server]
workers = 4  # Match CPU cores

[performance]
cpu_cores = 4
cpu_warning_threshold = 70.0

[neural]
gpu_enabled = true  # Offload to GPU if available
```

**Thread Pool Adjustment**
```rust
// Adjust async runtime configuration
#[tokio::main(worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Application code
}
```

### Issue 4: Database Connection Issues

#### Symptoms
- Database connection errors
- Slow query performance
- Connection pool exhausted

#### Diagnostic Steps
```bash
# Test database connectivity
psql -h localhost -U ai_orchestrator -d hive_db -c "SELECT 1;"

# Check connection pool status
curl http://localhost:3001/metrics | jq '.data.current_metrics.database'

# Database logs
sudo tail -f /var/log/postgresql/postgresql-*.log

# Connection count
psql -h localhost -U ai_orchestrator -d hive_db -c "SELECT count(*) FROM pg_stat_activity;"

# Slow queries
psql -h localhost -U ai_orchestrator -d hive_db -c "SELECT * FROM pg_stat_activity WHERE state = 'active' AND now() - query_start > interval '1 minute';"
```

#### Solutions

**Connection Pool Tuning**
```toml
[database]
max_connections = 20
min_connections = 5
connection_timeout_seconds = 30
idle_timeout_seconds = 300
```

**Database Optimization**
```sql
-- Analyze table statistics
ANALYZE;

-- Check for slow queries
SELECT
    query,
    calls,
    total_time,
    mean_time,
    rows
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 10;

-- Add indexes for common queries
CREATE INDEX CONCURRENTLY idx_tasks_status_created ON tasks(status, created_at);
CREATE INDEX CONCURRENTLY idx_agents_capabilities ON agents USING GIN(capabilities);
```

**Database Maintenance**
```bash
# Vacuum database
vacuumdb -h localhost -U ai_orchestrator --analyze hive_db

# Reindex database
reindexdb -h localhost -U ai_orchestrator hive_db

# Check database size
psql -h localhost -U ai_orchestrator -d hive_db -c "SELECT pg_size_pretty(pg_database_size('hive_db'));"
```

### Issue 5: Agent Coordination Problems

#### Symptoms
- Tasks not being assigned to agents
- Agents not responding
- Swarm coordination failures

#### Diagnostic Steps
```bash
# Check agent status
curl http://localhost:3001/api/agents | jq '.data.agents[] | {id, name, state, capabilities}'

# Check task queue
curl http://localhost:3001/api/tasks | jq '.data.tasks[] | {id, status, assigned_agent}'

# Agent metrics
curl http://localhost:3001/metrics | jq '.data.current_metrics.agent_management'

# Swarm coordination status
curl http://localhost:3001/api/hive/status | jq '.data.metrics'
```

#### Solutions

**Agent Health Checks**
```bash
# Test agent responsiveness
curl http://localhost:3001/api/agents | jq '.data.agents | length'

# Check agent capabilities matching
curl http://localhost:3001/api/tasks/pending | jq '.data.tasks[].required_capabilities'

# Restart agent management
curl -X POST http://localhost:3001/api/modules/agent_management/restart
```

**Task Distribution Issues**
```bash
# Check work-stealing queue
curl http://localhost:3001/metrics | jq '.data.current_metrics.task_management'

# Reset task distributor
curl -X POST http://localhost:3001/api/modules/task_management/reset

# Manual task assignment (debugging)
curl -X POST http://localhost:3001/api/tasks/assign \
  -H "Content-Type: application/json" \
  -d '{"task_id": "task-uuid", "agent_id": "agent-uuid"}'
```

### Issue 6: WebSocket Connection Issues

#### Symptoms
- Real-time updates not working
- WebSocket connection failures
- Client disconnection errors

#### Diagnostic Steps
```bash
# Test WebSocket connection
websocat ws://localhost:3001/ws

# Check WebSocket metrics
curl http://localhost:3001/metrics | jq '.data.current_metrics.websocket'

# Network connectivity
netstat -tulpn | grep :3001

# Firewall rules
sudo ufw status | grep 3001
```

#### Solutions

**WebSocket Configuration**
```toml
[websocket]
max_connections = 1000
heartbeat_interval = 30
timeout_seconds = 300
max_message_size_mb = 10
```

**Network Troubleshooting**
```bash
# Check network interfaces
ip addr show

# Test connectivity
telnet localhost 3001

# Check reverse proxy (if applicable)
nginx -t
sudo systemctl reload nginx
```

### Issue 7: Neural Processing Failures

#### Symptoms
- Neural processing errors
- GPU acceleration not working
- NLP analysis failures

#### Diagnostic Steps
```bash
# Check neural processing status
curl http://localhost:3001/api/modules/neural/status

# GPU availability
nvidia-smi

# Neural processing logs
sudo journalctl -u ai-orchestrator-hub | grep -i neural

# Test neural functionality
curl -X POST http://localhost:3001/api/neural/analyze \
  -H "Content-Type: application/json" \
  -d '{"text": "test input"}'
```

#### Solutions

**GPU Configuration**
```toml
[neural]
gpu_enabled = true
gpu_memory_limit_mb = 4096
cuda_version = "11.8"
```

**Neural Processing Reset**
```bash
# Restart neural processor
curl -X POST http://localhost:3001/api/modules/neural/restart

# Clear neural cache
curl -X POST http://localhost:3001/api/modules/neural/clear-cache

# Switch to CPU mode (fallback)
sed -i 's/gpu_enabled = true/gpu_enabled = false/' settings/production.toml
sudo systemctl restart ai-orchestrator-hub
```

### Issue 8: Security and Authentication Problems

#### Symptoms
- Authentication failures
- Unauthorized access errors
- JWT token issues

#### Diagnostic Steps
```bash
# Check authentication logs
sudo journalctl -u ai-orchestrator-hub | grep -i auth

# Test authentication endpoint
curl -X POST http://localhost:3001/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "test", "password": "test"}'

# Check JWT configuration
grep JWT_SECRET settings/production.toml

# Security audit logs
sudo journalctl -u ai-orchestrator-hub | grep -i security
```

#### Solutions

**JWT Configuration**
```toml
[security]
jwt_secret = "your-256-bit-secret-here-change-this-in-production"
jwt_expiration_hours = 24
refresh_token_expiration_days = 7
```

**Security Hardening**
```bash
# Generate secure JWT secret
openssl rand -hex 32

# Update configuration
sed -i 's/jwt_secret = ".*"/jwt_secret = "'"$(openssl rand -hex 32)"'"/' settings/production.toml

# Restart service
sudo systemctl restart ai-orchestrator-hub
```

## Advanced Troubleshooting Tools

### Log Analysis Tools
```bash
# Structured log parsing
sudo journalctl -u ai-orchestrator-hub -o json | jq 'select(.level == "error")'

# Log aggregation
sudo journalctl -u ai-orchestrator-hub --since "1 day ago" > logs_$(date +%Y%m%d).log

# Error pattern analysis
grep "ERROR" logs_$(date +%Y%m%d).log | sort | uniq -c | sort -nr
```

### Performance Monitoring
```bash
# System performance snapshot
#!/bin/bash
echo "=== System Performance Snapshot ==="
echo "Timestamp: $(date)"
echo "Load Average: $(uptime | awk -F'load average:' '{print $2}')"
echo "Memory Usage:"
free -h
echo "Disk Usage:"
df -h
echo "Process Status:"
ps aux --sort=-%cpu | head -5
echo "Network Connections:"
netstat -tulpn | grep :3001
```

### Database Query Analysis
```sql
-- Query performance analysis
SELECT
    schemaname,
    tablename,
    attname,
    n_distinct,
    correlation
FROM pg_stats
WHERE schemaname = 'public'
ORDER BY n_distinct DESC;

-- Index usage analysis
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;

-- Table bloat analysis
SELECT
    schemaname,
    tablename,
    n_tup_ins,
    n_tup_upd,
    n_tup_del,
    n_live_tup,
    n_dead_tup
FROM pg_stat_user_tables
ORDER BY n_dead_tup DESC;
```

### Network Diagnostics
```bash
# Network connectivity test
ping -c 4 localhost

# Port availability
nc -zv localhost 3001

# Connection analysis
ss -tulpn | grep :3001

# Bandwidth test
iperf3 -c localhost -p 3001
```

## Emergency Response Procedures

### Critical System Failure
```bash
# Immediate actions
1. Stop accepting new requests (load balancer)
2. Assess system status
3. Check monitoring alerts
4. Isolate failing components
5. Initiate backup recovery if needed
6. Notify stakeholders
7. Begin root cause analysis
```

### Data Recovery
```bash
# Database recovery
sudo systemctl stop ai-orchestrator-hub
pg_restore -h localhost -U ai_orchestrator -d hive_db backup.sql
sudo systemctl start ai-orchestrator-hub

# File system recovery
tar -xzf data_backup.tar.gz -C /
tar -xzf config_backup.tar.gz -C /
```

### Service Restoration
```bash
# Gradual service restoration
sudo systemctl start ai-orchestrator-hub
curl -f http://localhost:3001/health
# If healthy, enable load balancer
# Monitor for 15 minutes
# Gradually increase traffic
```

## Prevention and Maintenance

### Proactive Monitoring
```bash
# Set up alerts for key metrics
# CPU > 80%
# Memory > 85%
# Disk > 90%
# Error rate > 5%
# Response time > 500ms

# Regular health checks
# Daily: curl -f http://localhost:3001/health
# Weekly: Full system diagnostics
# Monthly: Performance benchmarks
```

### Regular Maintenance
```bash
# Database maintenance
vacuumdb --analyze --verbose hive_db

# Log rotation
logrotate -f /etc/logrotate.d/ai-orchestrator-hub

# Security updates
apt-get update && apt-get upgrade

# Backup verification
# Test restore procedures monthly
```

### Performance Baselines
```bash
# Establish normal operating parameters
# Average response time: <200ms
# CPU usage: <60%
# Memory usage: <70%
# Error rate: <1%
# Active agents: 10-50
# Tasks per second: 50-200
```

This troubleshooting guide provides systematic approaches to diagnosing and resolving the most common issues in the AI Orchestrator Hub system. Regular application of these procedures will help maintain system reliability and performance.