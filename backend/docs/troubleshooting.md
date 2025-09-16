# Troubleshooting Guide

This guide helps you diagnose and resolve common issues with the AI Orchestrator Hub.

## Quick Diagnosis

### Health Check

Always start with the health check endpoint:

```bash
curl http://localhost:3001/health
```

Expected response:
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "2.0.0",
    "response_time_ms": 15
  }
}
```

### System Information

Get comprehensive system information:

```bash
curl http://localhost:3001/api/hive/status
```

### Log Levels

Enable debug logging for detailed information:

```bash
export HIVE_LOGGING__LEVEL=debug
# Restart the service
```

## Common Issues

### Backend Won't Start

**Symptoms:**
- Server fails to start
- Port binding errors
- Database connection failures

**Solutions:**

1. **Check Port Availability:**
   ```bash
   # Check if port 3001 is in use
   sudo netstat -tulpn | grep :3001
   sudo lsof -i :3001
   ```

2. **Database Issues:**
   ```bash
   # Check database file permissions
   ls -la data/hive_persistence.db

   # For PostgreSQL
   psql -h localhost -U hive_user -d hive_db -c "SELECT 1;"
   ```

3. **Build Issues:**
   ```bash
   # Clean and rebuild
   cd backend
   cargo clean
   cargo build --release
   ```

4. **Missing Dependencies:**
   ```bash
   # Check Rust version
   rustc --version  # Should be 1.70+

   # Update Rust
   rustup update
   ```

### Frontend Build Errors

**Symptoms:**
- npm install fails
- Build process errors
- Missing dependencies

**Solutions:**

1. **Clear Cache:**
   ```bash
   cd frontend
   rm -rf node_modules package-lock.json
   npm cache clean --force
   npm install
   ```

2. **Check Node.js Version:**
   ```bash
   node --version  # Should be 18+
   npm --version   # Should be 8+
   ```

3. **Update Dependencies:**
   ```bash
   npm audit fix
   npm update
   ```

### WebSocket Connection Issues

**Symptoms:**
- Real-time updates not working
- Connection timeouts
- WebSocket errors in browser console

**Solutions:**

1. **Check Backend Status:**
   ```bash
   curl http://localhost:3001/health
   ```

2. **Verify WebSocket Endpoint:**
   ```bash
   # Test WebSocket connection
   websocat ws://localhost:3001/ws
   ```

3. **Check Firewall:**
   ```bash
   # Allow WebSocket connections
   sudo ufw allow 3001
   ```

4. **CORS Configuration:**
   ```env
   HIVE_SERVER__CORS_ORIGINS=http://localhost:3000
   ```

### Agent Creation Fails

**Symptoms:**
- Agent creation returns errors
- Agents don't appear in the system
- Validation errors

**Solutions:**

1. **Check Request Format:**
   ```bash
   curl -X POST http://localhost:3001/api/agents \
     -H "Content-Type: application/json" \
     -d '{
       "name": "TestAgent",
       "type": "worker",
       "capabilities": [
         {
           "name": "test",
           "proficiency": 0.8,
           "learning_rate": 0.1
         }
       ]
     }'
   ```

2. **Validate Capabilities:**
   - Ensure proficiency is between 0.0 and 1.0
   - Check capability names are valid
   - Verify agent type is supported

3. **Check System Limits:**
   ```bash
   curl http://localhost:3001/api/hive/status
   # Check max_agents limit
   ```

### Task Execution Issues

**Symptoms:**
- Tasks remain pending
- Tasks fail immediately
- No agent assignment

**Solutions:**

1. **Check Agent Availability:**
   ```bash
   curl http://localhost:3001/api/agents
   # Ensure agents are in "idle" or "active" state
   ```

2. **Validate Task Requirements:**
   ```bash
   # Check task capabilities match agent capabilities
   curl http://localhost:3001/api/tasks
   ```

3. **Review Task Queue:**
   ```bash
   curl http://localhost:3001/api/hive/status
   # Check queue_size and active tasks
   ```

4. **Enable Debug Logging:**
   ```env
   HIVE_LOGGING__LEVEL=debug
   HIVE_LOGGING__ENABLE_REQUEST_LOGGING=true
   ```

### Performance Issues

**Symptoms:**
- Slow response times
- High CPU/memory usage
- System becomes unresponsive

**Solutions:**

1. **Monitor Resources:**
   ```bash
   curl http://localhost:3001/api/resources
   ```

2. **Check System Metrics:**
   ```bash
   curl http://localhost:3001/metrics
   ```

3. **Optimize Configuration:**
   ```env
   # Reduce concurrent tasks
   HIVE_TASKS__MAX_CONCURRENT_TASKS=25

   # Enable caching
   HIVE_PERFORMANCE__CACHING_ENABLED=true

   # Adjust pool sizes
   HIVE_PERFORMANCE__CONNECTION_POOL_SIZE=20
   ```

4. **Database Optimization:**
   ```bash
   # For SQLite
   sqlite3 data/hive_persistence.db "VACUUM;"

   # For PostgreSQL
   psql -d hive_db -c "VACUUM ANALYZE;"
   ```

### Memory Issues

**Symptoms:**
- Out of memory errors
- System crashes
- Slow performance

**Solutions:**

1. **Check Memory Usage:**
   ```bash
   # System memory
   free -h

   # Process memory
   ps aux | grep multiagent-hive
   ```

2. **Adjust Memory Settings:**
   ```env
   # Reduce agent count
   HIVE_AGENTS__MAX_AGENTS=50

   # Enable memory optimization
   HIVE_PERFORMANCE__PERFORMANCE_OPTIMIZATION_ENABLED=true

   # Set memory limits
   HIVE_PERFORMANCE__MEMORY_WARNING_THRESHOLD=80.0
   ```

3. **Database Memory:**
   ```env
   # Reduce connection pool
   HIVE_DATABASE__MAX_CONNECTIONS=5

   # Enable connection pooling optimizations
   ```

### Database Connection Issues

**Symptoms:**
- Database connection errors
- Slow queries
- Connection pool exhausted

**Solutions:**

1. **Test Database Connection:**
   ```bash
   # For SQLite
   sqlite3 data/hive_persistence.db "SELECT 1;"

   # For PostgreSQL
   psql -h localhost -U hive_user -d hive_db -c "SELECT 1;"
   ```

2. **Check Connection Limits:**
   ```env
   HIVE_DATABASE__MAX_CONNECTIONS=10
   HIVE_DATABASE__CONNECTION_TIMEOUT_SECS=30
   ```

3. **Database Configuration:**
   ```sql
   -- PostgreSQL optimization
   ALTER SYSTEM SET shared_buffers = '256MB';
   ALTER SYSTEM SET effective_cache_size = '1GB';
   ALTER SYSTEM SET work_mem = '4MB';
   ```

### Network Issues

**Symptoms:**
- Connection timeouts
- Intermittent failures
- High latency

**Solutions:**

1. **Check Network Connectivity:**
   ```bash
   ping localhost
   telnet localhost 3001
   ```

2. **Firewall Configuration:**
   ```bash
   # UFW
   sudo ufw status
   sudo ufw allow 3001

   # iptables
   sudo iptables -A INPUT -p tcp --dport 3001 -j ACCEPT
   ```

3. **Load Balancer Issues:**
   ```bash
   # Check load balancer configuration
   # Verify health check endpoints
   curl http://load-balancer/health
   ```

### Authentication Issues

**Symptoms:**
- Authentication failures
- Invalid token errors
- Permission denied

**Solutions:**

1. **Check JWT Configuration:**
   ```env
   HIVE_SECURITY__JWT_SECRET=your-secret-key
   HIVE_SECURITY__JWT_EXPIRATION_HOURS=24
   ```

2. **Validate Tokens:**
   ```bash
   # Check token format
   jwt decode your-token-here
   ```

3. **Review Security Settings:**
   ```env
   HIVE_SECURITY__AUDIT_LOGGING_ENABLED=true
   # Check audit logs for failed authentication attempts
   ```

### Scaling Issues

**Symptoms:**
- System can't handle load
- Auto-scaling not working
- Resource exhaustion

**Solutions:**

1. **Check Current Load:**
   ```bash
   curl http://localhost:3001/metrics
   ```

2. **Enable Auto-scaling:**
   ```env
   HIVE_AGENTS__AUTO_SCALING_ENABLED=true
   HIVE_AGENTS__MIN_AGENTS=5
   HIVE_AGENTS__MAX_SCALE_AGENTS=100
   ```

3. **Resource Optimization:**
   ```env
   HIVE_PERFORMANCE__CPU_WARNING_THRESHOLD=70.0
   HIVE_PERFORMANCE__MEMORY_WARNING_THRESHOLD=80.0
   ```

4. **Horizontal Scaling:**
   ```yaml
   # Kubernetes HPA
   apiVersion: autoscaling/v2
   kind: HorizontalPodAutoscaler
   metadata:
     name: ai-orchestrator-hpa
   spec:
     scaleTargetRef:
       apiVersion: apps/v1
       kind: Deployment
       name: ai-orchestrator-backend
     minReplicas: 3
     maxReplicas: 10
     metrics:
     - type: Resource
       resource:
         name: cpu
         target:
           type: Utilization
           averageUtilization: 70
   ```

## Advanced Troubleshooting

### Log Analysis

1. **Enable Comprehensive Logging:**
   ```env
   HIVE_LOGGING__LEVEL=debug
   HIVE_LOGGING__FORMAT=json
   HIVE_LOGGING__ENABLE_FILE_LOGGING=true
   HIVE_LOGGING__LOG_FILE_PATH=./logs/hive.log
   ```

2. **Log Analysis Commands:**
   ```bash
   # Search for errors
   grep "ERROR" logs/hive.log

   # Find specific events
   grep "agent_created" logs/hive.log

   # Monitor in real-time
   tail -f logs/hive.log
   ```

### Performance Profiling

1. **Enable Performance Monitoring:**
   ```env
   HIVE_MONITORING__ENABLE_PERFORMANCE_MONITORING=true
   HIVE_MONITORING__MONITORING_INTERVAL_SECS=5
   ```

2. **Use Profiling Tools:**
   ```bash
   # Rust profiling
   cargo build --release
   perf record ./target/release/multiagent-hive
   perf report
   ```

### Memory Leak Detection

1. **Monitor Memory Usage:**
   ```bash
   # System memory
   vmstat 1

   # Process memory
   ps aux --sort=-%mem | head
   ```

2. **Enable Memory Profiling:**
   ```env
   HIVE_PERFORMANCE__MEMORY_WARNING_THRESHOLD=75.0
   HIVE_MONITORING__ENABLE_DIAGNOSTICS=true
   ```

### Database Performance

1. **Query Performance:**
   ```sql
   -- PostgreSQL slow query log
   SET log_min_duration_statement = '1000';

   -- SQLite analysis
   .timer on
   ANALYZE;
   .schema
   ```

2. **Index Optimization:**
   ```sql
   -- Create indexes on frequently queried columns
   CREATE INDEX idx_tasks_status ON tasks(status);
   CREATE INDEX idx_agents_type ON agents(type);
   ```

## Diagnostic Tools

### Built-in Diagnostics

1. **Health Check:**
   ```bash
   curl http://localhost:3001/health
   ```

2. **System Diagnostics:**
   ```bash
   curl http://localhost:3001/api/hive/status
   ```

3. **Module Diagnostics:**
   ```bash
   curl http://localhost:3001/api/modules/status
   ```

### External Tools

1. **Network Diagnostics:**
   ```bash
   # Network connectivity
   mtr localhost

   # Packet capture
   tcpdump -i any port 3001
   ```

2. **System Monitoring:**
   ```bash
   # System monitoring
   htop
   iotop

   # Network monitoring
   nload
   ```

3. **Database Monitoring:**
   ```bash
   # PostgreSQL monitoring
   pg_stat_activity

   # SQLite monitoring
   sqlite3 data/hive_persistence.db ".stats on" "SELECT 1;"
   ```

## Recovery Procedures

### Data Recovery

1. **Database Backup:**
   ```bash
   # SQLite backup
   sqlite3 data/hive_persistence.db ".backup hive_backup.db"

   # PostgreSQL backup
   pg_dump hive_db > hive_backup.sql
   ```

2. **Restore from Backup:**
   ```bash
   # SQLite restore
   sqlite3 data/hive_persistence.db ".restore hive_backup.db"

   # PostgreSQL restore
   psql hive_db < hive_backup.sql
   ```

### System Recovery

1. **Graceful Shutdown:**
   ```bash
   # Send shutdown signal
   kill -TERM $(pgrep multiagent-hive)

   # Wait for graceful shutdown
   sleep 30
   ```

2. **Force Restart:**
   ```bash
   # Kill and restart
   pkill -9 multiagent-hive
   ./target/release/multiagent-hive &
   ```

### Emergency Procedures

1. **Disable Problematic Features:**
   ```env
   # Disable neural processing
   HIVE_NEURAL__ENABLE_ADVANCED_NEURAL=false

   # Reduce agent count
   HIVE_AGENTS__MAX_AGENTS=10

   # Disable auto-scaling
   HIVE_AGENTS__AUTO_SCALING_ENABLED=false
   ```

2. **Isolate Components:**
   ```bash
   # Run with minimal features
   cargo run -- --config minimal-config.toml
   ```

## Getting Help

### Support Resources

1. **Documentation:**
   - [Installation Guide](installation.md)
   - [Configuration Guide](configuration.md)
   - [API Documentation](api.md)

2. **Community Support:**
   - GitHub Issues
   - Documentation comments
   - Health check outputs

3. **Diagnostic Information:**
   ```bash
   # System information
   uname -a
   rustc --version
   cargo --version

   # Configuration dump
   curl http://localhost:3001/api/config
   ```

### When to Contact Support

- **Critical Issues:** System completely down
- **Data Loss:** Unable to recover data
- **Security Incidents:** Suspected breaches
- **Performance Degradation:** Sustained performance issues

### Support Checklist

Before contacting support, please provide:

- [ ] System information (`uname -a`)
- [ ] Rust version (`rustc --version`)
- [ ] Configuration files
- [ ] Log files (last 1000 lines)
- [ ] Steps to reproduce the issue
- [ ] Health check output
- [ ] System metrics

This comprehensive troubleshooting guide should help resolve most issues. For persistent problems, please create a GitHub issue with the diagnostic information above.