# Troubleshooting Guide

This guide helps you diagnose and resolve common issues with the Multiagent Hive System.

## Quick Diagnosis

### System Health Check

```bash
# Check if backend is running
curl http://localhost:3001/api/hive/status

# Check if frontend is accessible
curl http://localhost:3000

# Check system resources
top
df -h
free -h
```

### Log Analysis

```bash
# Backend logs
tail -f backend/hive.log

# Frontend logs
cd frontend && npm run dev 2>&1 | tee frontend.log

# System logs
journalctl -u multiagent-hive -f
```

## Backend Issues

### Backend Won't Start

#### Port Already in Use

**Symptoms:**
- Error: "Address already in use"
- Backend fails to start

**Solution:**
```bash
# Find process using port 3001
lsof -i :3001

# Kill the process
kill -9 <PID>

# Or change port
export HIVE_PORT=3002
cargo run
```

#### Compilation Errors

**Symptoms:**
- Rust compilation fails
- Missing dependencies

**Solutions:**
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build

# Check for missing system dependencies
sudo apt install build-essential  # Ubuntu/Debian
# or
xcode-select --install            # macOS
```

#### Database Connection Issues

**Symptoms:**
- "Connection refused" errors
- Database migration failures

**Solutions:**
```bash
# Check database status
sudo systemctl status postgresql

# Test connection
psql -h localhost -U hive_user -d multiagent_hive

# Reset database
rm backend/hive.db
cargo run  # Will recreate database
```

### Runtime Errors

#### WebSocket Connection Issues

**Symptoms:**
- Frontend shows "Connection lost"
- Real-time updates not working

**Solutions:**
```bash
# Check WebSocket endpoint
curl -I http://localhost:3001/ws

# Verify CORS settings
curl -H "Origin: http://localhost:3000" \
     -H "Access-Control-Request-Method: GET" \
     -X OPTIONS http://localhost:3001/ws

# Check firewall
sudo ufw status
```

#### Neural Processing Errors

**Symptoms:**
- "FANN library not found" errors
- Advanced neural features not working

**Solutions:**
```bash
# Install FANN library
sudo apt install libfann-dev  # Ubuntu/Debian

# Rebuild with features
cargo clean
cargo build --features advanced-neural

# Check GPU support
nvidia-smi  # If using GPU
```

#### Memory Issues

**Symptoms:**
- Out of memory errors
- System becomes unresponsive

**Solutions:**
```bash
# Check memory usage
free -h

# Reduce agent count
export MAX_AGENTS=500

# Enable memory profiling
export RUSTFLAGS="-g"
cargo build --release
```

### Performance Issues

#### High CPU Usage

**Symptoms:**
- System running slow
- High CPU utilization

**Solutions:**
```bash
# Profile performance
cargo flamegraph --bin multiagent-hive

# Reduce neural complexity
export NEURAL_MODE=basic

# Limit concurrent tasks
export MAX_CONCURRENT_TASKS=25
```

#### Slow Task Processing

**Symptoms:**
- Tasks taking longer than expected
- Queue building up

**Solutions:**
```bash
# Check task queue status
curl http://localhost:3001/api/tasks?status=pending

# Monitor agent utilization
curl http://localhost:3001/api/hive/metrics

# Adjust task priorities
curl -X PUT http://localhost:3001/api/tasks/{id} \
     -H "Content-Type: application/json" \
     -d '{"priority": "High"}'
```

## Frontend Issues

### Build Errors

#### Node.js Version Issues

**Symptoms:**
- "Node version not supported" errors
- Build failures

**Solutions:**
```bash
# Check Node version
node --version  # Should be 18+

# Update Node.js
nvm use 20

# Clear cache and rebuild
cd frontend
rm -rf node_modules package-lock.json
npm install
npm run build
```

#### TypeScript Errors

**Symptoms:**
- Type compilation errors
- Build fails with TS errors

**Solutions:**
```bash
# Check TypeScript version
npx tsc --version

# Run type checking
npm run type-check

# Fix common issues
npm run lint:fix
```

### Runtime Errors

#### API Connection Issues

**Symptoms:**
- "Failed to fetch" errors
- API calls failing

**Solutions:**
```bash
# Test API connectivity
curl http://localhost:3001/api/hive/status

# Check CORS headers
curl -H "Origin: http://localhost:3000" \
     http://localhost:3001/api/hive/status

# Verify environment variables
cat frontend/.env.local
```

#### WebSocket Connection Issues

**Symptoms:**
- Real-time updates not working
- Dashboard not updating

**Solutions:**
```bash
# Test WebSocket connection
wscat -c ws://localhost:3001/ws

# Check browser console for errors
# Open DevTools → Console

# Verify WebSocket URL
console.log(process.env.NEXT_PUBLIC_WS_URL)
```

#### Component Rendering Issues

**Symptoms:**
- Blank pages
- Components not loading
- Console errors

**Solutions:**
```bash
# Check React DevTools
# Install React DevTools extension

# Enable error boundaries
# Check browser console for component errors

# Clear browser cache
# Hard refresh: Ctrl+Shift+R
```

### Performance Issues

#### Slow Page Loads

**Symptoms:**
- Pages taking long to load
- High memory usage in browser

**Solutions:**
```bash
# Check bundle size
npm run build
ls -lh .next/static/chunks/

# Enable code splitting
# Check dynamic imports in components

# Optimize images
# Use next/image component
```

#### Memory Leaks

**Symptoms:**
- Browser memory usage increasing
- Page becomes unresponsive

**Solutions:**
```bash
# Use React DevTools Profiler
# Record performance profile

# Check for memory leaks
# Look for unmounted component updates

# Implement proper cleanup
# Remove event listeners in useEffect cleanup
```

## Database Issues

### Connection Problems

**Symptoms:**
- "Connection timeout" errors
- Database queries failing

**Solutions:**
```bash
# Check database status
sudo systemctl status postgresql

# Test connection
psql -h localhost -p 5432 -U hive_user -d multiagent_hive

# Check connection pool
export DATABASE_POOL_SIZE=5

# Verify connection string
echo $DATABASE_URL
```

### Migration Issues

**Symptoms:**
- Schema errors
- Migration failures

**Solutions:**
```bash
# Check migration status
cargo run -- --show-migrations

# Reset database
rm backend/hive.db
cargo run  # Recreates schema

# Manual migration
psql -d multiagent_hive -f migrations/001_initial.sql
```

### Data Corruption

**Symptoms:**
- Inconsistent data
- Query errors

**Solutions:**
```bash
# Backup current data
cp backend/hive.db backend/hive.db.backup

# Repair database
sqlite3 backend/hive.db ".recover" > recovered.sql
sqlite3 backend/hive_new.db < recovered.sql

# Verify data integrity
sqlite3 backend/hive.db "PRAGMA integrity_check;"
```

## Network Issues

### Firewall Problems

**Symptoms:**
- Connection refused
- Timeout errors

**Solutions:**
```bash
# Check firewall status
sudo ufw status

# Allow required ports
sudo ufw allow 3000/tcp  # Frontend
sudo ufw allow 3001/tcp  # Backend

# Check SELinux/AppArmor
sudo getenforce
```

### DNS Issues

**Symptoms:**
- "Name resolution failure"
- Unable to reach services

**Solutions:**
```bash
# Check DNS resolution
nslookup localhost

# Test connectivity
ping localhost

# Check /etc/hosts
cat /etc/hosts
```

### SSL/TLS Issues

**Symptoms:**
- Certificate errors
- HTTPS connection failures

**Solutions:**
```bash
# Check certificate
openssl s_client -connect localhost:443 -servername localhost

# Renew certificates
sudo certbot renew

# Check certificate validity
openssl x509 -in cert.pem -text -noout
```

## Docker Issues

### Container Won't Start

**Symptoms:**
- Container exits immediately
- Health check failures

**Solutions:**
```bash
# Check container logs
docker logs <container_id>

# Run container interactively
docker run -it --entrypoint /bin/bash your-image

# Check Docker resources
docker system df
```

### Networking Issues

**Symptoms:**
- Containers can't communicate
- Port mapping not working

**Solutions:**
```bash
# Check Docker networks
docker network ls

# Inspect container
docker inspect <container_id>

# Test inter-container communication
docker exec -it backend ping frontend
```

### Volume Issues

**Symptoms:**
- Data not persisting
- Permission errors

**Solutions:**
```bash
# Check volume permissions
ls -la /path/to/volume

# Fix permissions
sudo chown -R 1000:1000 /path/to/volume

# Verify volume mounting
docker volume ls
```

## Performance Troubleshooting

### Profiling Tools

```bash
# Backend profiling
cargo flamegraph --bin multiagent-hive

# Frontend profiling
# Chrome DevTools → Performance tab

# System profiling
perf record -g ./target/release/multiagent-hive
perf report
```

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Load testing
ab -n 1000 -c 10 http://localhost:3001/api/hive/status

# Memory profiling
valgrind --tool=massif ./target/release/multiagent-hive
```

### Optimization Tips

```bash
# Enable release optimizations
cargo build --release

# Use performance lints
cargo clippy -- -W perf

# Profile-guided optimization
RUSTFLAGS="-C profile-generate" cargo build --release
./target/release/multiagent-hive  # Run with data
RUSTFLAGS="-C profile-use" cargo build --release
```

## Monitoring and Alerting

### Setting Up Monitoring

```bash
# Enable metrics
export METRICS_ENABLED=true
export METRICS_PORT=9090

# Check metrics endpoint
curl http://localhost:9090/metrics
```

### Log Analysis

```bash
# Search for errors
grep "ERROR" backend/hive.log

# Monitor log rate
tail -f backend/hive.log | grep --line-buffered "ERROR" | wc -l

# Structured logging analysis
jq '.level == "error"' backend/hive.log
```

### Alert Configuration

```yaml
# alert-rules.yml
groups:
  - name: multiagent-hive
    rules:
      - alert: HighErrorRate
        expr: rate(errors_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
```

## Common Error Messages

### Backend Errors

#### "Failed to bind to address"
```
Error: Failed to bind to address 0.0.0.0:3001: Address already in use
```
**Solution:** Change port or kill existing process

#### "Database migration failed"
```
Error: Database migration failed: relation already exists
```
**Solution:** Reset database or check migration files

#### "Neural network initialization failed"
```
Error: Failed to initialize FANN network: Library not found
```
**Solution:** Install FANN library or disable advanced features

### Frontend Errors

#### "Module not found"
```
Error: Cannot resolve module 'react'
```
**Solution:** Reinstall dependencies

#### "TypeScript error"
```
Error: Property 'X' does not exist on type 'Y'
```
**Solution:** Update type definitions or fix type annotations

#### "WebSocket connection failed"
```
Error: WebSocket connection to 'ws://localhost:3001/ws' failed
```
**Solution:** Check backend WebSocket server and CORS settings

## Getting Help

### Community Support

- **GitHub Issues**: [Report bugs](../../issues)
- **Discussions**: Join community discussions
- **Documentation**: Check [docs/](../) directory

### Debug Information

When reporting issues, include:

```bash
# System information
uname -a
rustc --version
node --version
docker --version

# Application logs
tail -n 50 backend/hive.log
tail -n 50 frontend.log

# Configuration
cat backend/.env
cat frontend/.env.local

# Process information
ps aux | grep multiagent
```

### Emergency Procedures

#### System Recovery

```bash
# Stop all services
docker-compose down

# Backup data
cp backend/hive.db backend/hive.db.backup

# Reset system
rm backend/hive.db
docker-compose up --build
```

#### Data Recovery

```bash
# Restore from backup
cp backend/hive.db.backup backend/hive.db

# Verify data integrity
sqlite3 backend/hive.db "PRAGMA integrity_check;"

# Rebuild indexes if needed
sqlite3 backend/hive.db "REINDEX;"
```

## Prevention

### Best Practices

- **Regular Backups**: Automate database backups
- **Monitor Resources**: Set up alerting for high usage
- **Update Dependencies**: Keep Rust and Node.js updated
- **Test Deployments**: Use staging environment for testing
- **Log Rotation**: Configure log rotation to prevent disk filling

### Health Checks

```bash
# Add to crontab
*/5 * * * * curl -f http://localhost:3001/health || systemctl restart multiagent-hive
```

### Automated Testing

```bash
# Run tests before deployment
cargo test --all-features
cd frontend && npm test

# Integration tests
cargo run --example integration_test
```

## Next Steps

- **Configuration**: See [docs/configuration.md](configuration.md)
- **Performance**: See [docs/performance.md](performance.md)
- **Security**: See [docs/security-hardening.md](security-hardening.md)
- **Monitoring**: See [docs/observability.md](observability.md)