# Troubleshooting Guide for AI Orchestrator Hub

This comprehensive troubleshooting guide covers the top 10 most common issues encountered in the AI Orchestrator Hub project, which consists of a Rust backend, TypeScript/React frontend, and a multi-agent system. Each section provides detailed problem descriptions, symptoms, root causes, step-by-step solutions, prevention tips, and escalation guidelines.

For general project information, see [README.md](../README.md). For agent-specific issues, refer to [AGENTS.md](../AGENTS.md).

## 1. Build and Compilation Issues

### Problem: Rust Compilation Errors
Compilation failures in the backend preventing successful builds.

### Symptoms
- `cargo build` fails with error messages
- Dependency resolution issues
- Linker errors or missing libraries

### Root Causes
- Outdated dependencies
- Missing system libraries (e.g., OpenSSL, SQLite)
- Rust version incompatibility
- Conflicting crate versions

### Step-by-Step Solutions
1. Update Rust toolchain:
   ```bash
   rustup update
   ```

2. Check Rust version compatibility:
   ```bash
   rustc --version
   # Ensure version matches backend/Cargo.toml requirements
   ```

3. Clean and rebuild:
   ```bash
   cd backend
   cargo clean
   cargo build
   ```

4. Update dependencies:
   ```bash
   cargo update
   ```

5. Install missing system dependencies:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install libssl-dev sqlite3 libsqlite3-dev

   # macOS
   brew install openssl sqlite
   ```

### Prevention Tips
- Regularly run `cargo update` to keep dependencies current
- Use `cargo clippy` for linting before builds
- Pin dependency versions in Cargo.toml for stability

### When to Escalate
If compilation fails after trying all solutions, check for upstream crate issues or create an issue in the project repository.

---

## 2. Runtime Errors

### Problem: Application Panics or Crashes
Unexpected runtime failures in the Rust backend or JavaScript errors in the frontend.

### Symptoms
- Backend: Panic messages in logs, application termination
- Frontend: Console errors, white screen, unresponsive UI
- Error messages like "thread panicked" or "TypeError: Cannot read property"

### Root Causes
- Null pointer dereferences in Rust
- Unhandled exceptions in TypeScript
- Memory corruption or leaks
- Invalid configuration values

### Step-by-Step Solutions
1. Check application logs:
   ```bash
   # Backend logs
   cd backend
   cargo run -- --log-level debug

   # Frontend logs
   cd frontend
   npm run dev
   # Open browser dev tools
   ```

2. Enable detailed error reporting:
   ```rust
   // In backend/src/main.rs
   std::env::set_var("RUST_BACKTRACE", "1");
   ```

3. Add error boundaries in React:
   ```tsx
   // In frontend/src/components/ErrorBoundary.tsx
   class ErrorBoundary extends React.Component {
     componentDidCatch(error, errorInfo) {
       console.error('Error caught by boundary:', error, errorInfo);
     }
   }
   ```

4. Validate configuration:
   ```bash
   # Check backend/settings/*.toml files
   cat backend/settings/default.toml
   ```

### Prevention Tips
- Use `Result<T, E>` and `Option<T>` in Rust for error handling
- Implement error boundaries in React components
- Add comprehensive logging with correlation IDs

### When to Escalate
If panics persist, analyze stack traces and consider memory profiling with tools like `valgrind`.

---

## 3. Database Connection Problems

### Problem: Unable to Connect to Database
Failures in establishing or maintaining database connections.

### Symptoms
- "Connection refused" errors
- Timeout errors during database operations
- Data not persisting or retrieving correctly

### Root Causes
- Incorrect connection string
- Database server not running
- Network connectivity issues
- Authentication failures

### Step-by-Step Solutions
1. Verify database configuration:
   ```bash
   # Check backend/settings/*.toml
   grep -r "database" backend/settings/
   ```

2. Test database connectivity:
   ```bash
   # For SQLite (default)
   sqlite3 backend/data/hive_persistence.db "SELECT 1;"

   # For PostgreSQL (if configured)
   psql -h localhost -U username -d database_name
   ```

3. Check database service status:
   ```bash
   # SQLite: No service needed
   # PostgreSQL
   sudo systemctl status postgresql
   ```

4. Update connection parameters:
   ```toml
   # backend/settings/default.toml
   [database]
   url = "sqlite://backend/data/hive_persistence.db"
   max_connections = 10
   ```

### Prevention Tips
- Use connection pooling to manage database connections
- Implement retry logic for transient failures
- Monitor connection health with health checks

### When to Escalate
If database corruption is suspected, backup data and consider database migration tools.

---

## 4. WebSocket Connection Issues

### Problem: Real-time Communication Failures
WebSocket connections failing between frontend and backend or between agents.

### Symptoms
- Real-time updates not working
- "WebSocket connection failed" errors
- Agent communication timeouts

### Root Causes
- Firewall blocking WebSocket ports
- Incorrect WebSocket URL configuration
- Server-side WebSocket handler issues
- Network proxy interference

### Step-by-Step Solutions
1. Verify WebSocket configuration:
   ```bash
   # Check backend configuration
   grep -r "websocket\|ws" backend/src/
   ```

2. Test WebSocket connectivity:
   ```javascript
   // In browser console
   const ws = new WebSocket('ws://localhost:3001/ws');
   ws.onopen = () => console.log('Connected');
   ws.onerror = (error) => console.error('Error:', error);
   ```

3. Check firewall settings:
   ```bash
   # Linux
   sudo ufw status
   # Allow WebSocket port (e.g., 3001)
   sudo ufw allow 3001
   ```

4. Enable WebSocket debugging:
   ```bash
   # Backend: Add logging
   cd backend
   RUST_LOG=debug cargo run
   ```

### Prevention Tips
- Use secure WebSocket (WSS) in production
- Implement connection retry logic with exponential backoff
- Monitor WebSocket connection health

### When to Escalate
If network infrastructure issues are suspected, involve network administrators.

---

## 5. Agent Communication Problems

### Problem: Multi-Agent System Communication Failures
Agents unable to communicate or coordinate properly.

### Symptoms
- Tasks not being assigned to agents
- Agent responses not received
- Swarm coordination failures

### Root Causes
- MCP (Message Communication Protocol) configuration issues
- Agent registration failures
- Message queue problems
- Synchronization issues

### Step-by-Step Solutions
1. Check agent status:
   ```bash
   # Use swarm-coordinator agent
   opencode swarm-coordinator "Check the current status of the multiagent hive"
   ```

2. Verify MCP configuration:
   ```bash
   # Check .rovodev/mcp.json
   cat .rovodev/mcp.json
   ```

3. Restart agent services:
   ```bash
   # Backend
   cd backend
   cargo run --bin mcp_server

   # Or use service script
   ./scripts/run-mcp-service.sh
   ```

4. Debug agent communication:
   ```bash
   # Enable debug logging
   cd backend
   RUST_LOG=trace cargo run
   ```

### Prevention Tips
- Implement heartbeat mechanisms for agent health monitoring
- Use message acknowledgments and retries
- Monitor agent metrics with the monitoring system

### When to Escalate
If agent coordination fails consistently, review swarm intelligence algorithms in the codebase.

---

## 6. Performance Issues

### Problem: Slow Response Times or High Resource Usage
Application performance degradation affecting user experience.

### Symptoms
- Slow page loads or API responses
- High CPU/memory usage
- Database query timeouts

### Root Causes
- Memory leaks in Rust or JavaScript
- Inefficient database queries
- Unoptimized React re-renders
- Resource contention in multi-agent system

### Step-by-Step Solutions
1. Profile application performance:
   ```bash
   # Backend: Use cargo flamegraph
   cd backend
   cargo install flamegraph
   cargo flamegraph --bin main

   # Frontend: Use React DevTools Profiler
   ```

2. Check resource usage:
   ```bash
   # Monitor system resources
   top
   htop
   ```

3. Optimize database queries:
   ```rust
   // Add indexes and optimize queries
   // See backend/src/infrastructure/cache.rs
   ```

4. Implement caching:
   ```bash
   # Enable Redis caching if configured
   # Check backend/src/infrastructure/cache.rs
   ```

### Prevention Tips
- Use performance benchmarks regularly (see backend/benches/)
- Implement lazy loading in React components
- Monitor performance metrics with the monitoring system

### When to Escalate
If performance issues persist after optimization, consider hardware upgrades or architecture changes.

---

## 7. Deployment Problems

### Problem: Application Deployment Failures
Issues deploying the application to production or staging environments.

### Symptoms
- Build failures in CI/CD pipelines
- Container startup failures
- Environment configuration issues

### Root Causes
- Missing environment variables
- Incorrect Docker configuration
- CI/CD pipeline misconfigurations
- Dependency conflicts in deployment

### Step-by-Step Solutions
1. Check deployment logs:
   ```bash
   # GitHub Actions
   # View .github/workflows/*.yml
   # Check build logs in GitHub

   # Docker
   docker logs <container_id>
   ```

2. Verify environment variables:
   ```bash
   # Check .env files
   cat .env
   cat frontend/.env.local
   ```

3. Test Docker build:
   ```bash
   # If using Docker
   docker build -t ai-orchestrator-hub .
   docker run -p 3000:3000 ai-orchestrator-hub
   ```

4. Validate CI/CD configuration:
   ```bash
   # Check .github/workflows/
   cat .github/workflows/build.yml
   ```

### Prevention Tips
- Use environment-specific configuration files
- Implement health checks in deployment scripts
- Test deployments in staging environments first

### When to Escalate
If deployment issues are due to infrastructure problems, involve DevOps team.

---

## 8. Testing Issues

### Problem: Test Failures or Inconsistencies
Unit, integration, or end-to-end tests failing unexpectedly.

### Symptoms
- Test suite failures
- Flaky tests
- Coverage reports showing low coverage

### Root Causes
- Test environment setup issues
- Race conditions in async tests
- Mock configuration problems
- Code changes breaking existing tests

### Step-by-Step Solutions
1. Run tests with verbose output:
   ```bash
   # Backend
   cd backend
   cargo test -- --nocapture

   # Frontend
   cd frontend
   npm test -- --verbose
   ```

2. Check test configuration:
   ```bash
   # Backend: Check Cargo.toml [dev-dependencies]
   # Frontend: Check vitest.config.ts
   cat frontend/vitest.config.ts
   ```

3. Debug failing tests:
   ```bash
   # Run specific test
   cd backend
   cargo test test_function_name

   # Frontend
   cd frontend
   npm test -- testName
   ```

4. Update test snapshots (if applicable):
   ```bash
   cd frontend
   npm run test:update
   ```

### Prevention Tips
- Write comprehensive unit tests for new features
- Use integration tests for critical paths
- Run tests in CI/CD pipelines before merging

### When to Escalate
If tests fail due to fundamental architecture issues, review test strategy with the team.

---

## 9. Security-Related Issues

### Problem: Security Vulnerabilities or Access Control Failures
Security breaches, unauthorized access, or vulnerability alerts.

### Symptoms
- Security audit failures
- Unauthorized access attempts
- Data exposure warnings

### Root Causes
- Outdated dependencies with known vulnerabilities
- Improper authentication/authorization
- Insecure configuration settings
- Missing input validation

### Step-by-Step Solutions
1. Run security audits:
   ```bash
   # Backend
   cd backend
   cargo audit

   # Frontend
   cd frontend
   npm audit
   ```

2. Check authentication configuration:
   ```bash
   # Review backend/src/utils/auth.rs
   cat backend/src/utils/auth.rs
   ```

3. Update dependencies:
   ```bash
   # Backend
   cd backend
   cargo update

   # Frontend
   cd frontend
   npm update
   ```

4. Review security settings:
   ```bash
   # Check .gitleaks.toml and security workflows
   cat .github/workflows/security.yml
   ```

### Prevention Tips
- Regularly update dependencies and run security audits
- Implement proper input validation and sanitization
- Use HTTPS and secure headers in production

### When to Escalate
For any suspected security breach, immediately notify security team and follow incident response procedures in [SECURITY.md](.github/SECURITY.md).

---

## 10. General Debugging Tips

### Problem: General Debugging Strategies
Approaches for diagnosing and resolving various issues not covered above.

### Symptoms
- Unclear error messages
- Intermittent failures
- Performance anomalies

### Root Causes
- Insufficient logging
- Complex interactions between components
- Environmental differences

### Step-by-Step Solutions
1. Enable comprehensive logging:
   ```bash
   # Backend
   cd backend
   RUST_LOG=debug cargo run

   # Frontend
   cd frontend
   DEBUG=* npm run dev
   ```

2. Use debugging tools:
   ```bash
   # Rust: gdb or lldb
   rust-gdb target/debug/main

   # JavaScript: Chrome DevTools
   # Open browser dev tools and set breakpoints
   ```

3. Monitor system resources:
   ```bash
   # Use monitoring scripts
   ./scripts/load_test.js
   ```

4. Review application metrics:
   ```bash
   # Check monitoring dashboard
   # See monitoring/ directory
   ```

### Prevention Tips
- Implement structured logging with correlation IDs
- Use feature flags for gradual rollouts
- Maintain comprehensive documentation

### When to Escalate
If issues require deep architectural changes, involve the core development team.

---

## Additional Resources

- [Project README](../README.md) - General project information
- [AGENTS.md](../AGENTS.md) - Agent-specific documentation
- [TESTING.md](backend/TESTING.md) - Testing guidelines
- [SECURITY.md](.github/SECURITY.md) - Security procedures
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines

For issues not covered here, please check the project repository's issue tracker or create a new issue with detailed information about your problem.
