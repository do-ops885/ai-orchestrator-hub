# Hands-on Exercise: Basic System Setup

**Duration:** 1 hour  
**Difficulty:** Beginner  
**Prerequisites:** Linux environment, basic command line knowledge  

## Learning Objectives

After completing this exercise, you will be able to:
- Install and configure the AI Orchestrator Hub
- Start the system and verify basic functionality
- Perform health checks and basic monitoring
- Understand system configuration basics

## Exercise Overview

This exercise guides you through the complete setup of the AI Orchestrator Hub system, from installation to basic operation. You'll deploy the system locally and verify that all components are working correctly.

## Step-by-Step Instructions

### Step 1: Environment Preparation (10 minutes)

#### 1.1 System Requirements Check
Verify your system meets the minimum requirements:

```bash
# Check operating system
uname -a
cat /etc/os-release

# Check available resources
free -h
df -h /
nproc

# Check for required tools
which curl
which git
which cargo
which rustc
```

**Expected Output:**
- Linux distribution information
- At least 4GB RAM available
- At least 10GB disk space available
- curl, git, cargo, and rustc installed

#### 1.2 Install Dependencies
Install any missing system dependencies:

```bash
# Update package list
sudo apt-get update

# Install required packages
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    sqlite3 \
    curl \
    jq

# Verify installations
curl --version
jq --version
sqlite3 --version
```

**Verification:**
- All packages install without errors
- Commands show version information

### Step 2: System Installation (15 minutes)

#### 2.1 Clone Repository
Download the AI Orchestrator Hub source code:

```bash
# Clone the repository
git clone https://github.com/do-ops885/ai-orchestrator-hub.git

# Navigate to backend directory
cd ai-orchestrator-hub/backend

# Verify repository contents
ls -la
cat Cargo.toml
```

**Expected Output:**
- Repository clones successfully
- Backend directory contains source files
- Cargo.toml shows project configuration

#### 2.2 Build the System
Compile the AI Orchestrator Hub:

```bash
# Build in debug mode first
cargo build

# Check for compilation errors
echo $?

# Build optimized release version
cargo build --release

# Verify binary creation
ls -la target/release/
file target/release/ai-orchestrator-hub
```

**Expected Output:**
- Debug build completes without errors
- Release build creates optimized binary
- Binary is executable and properly linked

#### 2.3 Test Basic Functionality
Perform initial functionality tests:

```bash
# Test help output
./target/release/ai-orchestrator-hub --help

# Test version information
./target/release/ai-orchestrator-hub --version

# Test configuration check
./target/release/ai-orchestrator-hub --config settings/default.toml --check
```

**Expected Output:**
- Help text displays available options
- Version information shows correctly
- Configuration validation passes

### Step 3: Configuration Setup (10 minutes)

#### 3.1 Review Default Configuration
Examine the default configuration file:

```bash
# View configuration file
cat settings/default.toml

# Check configuration structure
grep -n "^\[" settings/default.toml
```

**Key Configuration Sections:**
- `[server]`: Server settings (host, port)
- `[logging]`: Logging configuration
- `[database]`: Database connection settings
- `[neural]`: Neural processing options

#### 3.2 Create Development Configuration
Create a development-specific configuration:

```bash
# Copy default configuration
cp settings/default.toml settings/development.toml

# Edit development settings
cat >> settings/development.toml << EOF
[logging]
level = "debug"

[server]
host = "localhost"
port = 3001

[database]
url = "sqlite:data/hive_dev.db"
EOF

# Verify configuration
cat settings/development.toml
```

**Expected Output:**
- Development configuration file created
- Settings appropriate for local development

### Step 4: System Startup and Verification (15 minutes)

#### 4.1 Start the System
Launch the AI Orchestrator Hub:

```bash
# Start system in background
./target/release/ai-orchestrator-hub --config settings/development.toml &

# Capture process ID
HIVE_PID=$!
echo "AI Orchestrator Hub started with PID: $HIVE_PID"

# Wait for startup
sleep 5
```

**Expected Output:**
- System starts without errors
- Process ID captured for later use

#### 4.2 Basic Connectivity Test
Test basic system connectivity:

```bash
# Test root endpoint
curl -s http://localhost:3001/

# Test with verbose output
curl -v http://localhost:3001/ 2>&1 | head -10
```

**Expected Output:**
- HTTP 200 response
- Welcome message from the system

#### 4.3 Health Check Verification
Perform comprehensive health checks:

```bash
# Basic health check
curl -s http://localhost:3001/health | jq .

# Detailed health check
curl -s "http://localhost:3001/health?detailed=true" | jq '.data.modules'
```

**Expected Output:**
- All modules report healthy status
- Response time under 100ms
- System information displays correctly

#### 4.4 Metrics Verification
Check system metrics:

```bash
# Get current metrics
curl -s http://localhost:3001/metrics | jq '.data.current_metrics'

# Check system resource usage
curl -s http://localhost:3001/metrics | jq '.data.current_metrics.system'
```

**Expected Output:**
- Metrics data structure is valid
- System resources show reasonable values
- No error conditions reported

### Step 5: Basic Operations Testing (5 minutes)

#### 5.1 Test Agent API
Test agent management functionality:

```bash
# List agents (should be empty)
curl -s http://localhost:3001/api/agents | jq '.data.agents | length'

# Create a test agent
curl -s -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "TestAgent",
    "type": "worker",
    "capabilities": [
      {
        "name": "test_tasks",
        "proficiency": 0.8,
        "learning_rate": 0.1
      }
    ]
  }' | jq .

# Verify agent creation
curl -s http://localhost:3001/api/agents | jq '.data.agents[0].name'
```

**Expected Output:**
- Agent creation succeeds
- Agent appears in agent list
- Agent has correct properties

#### 5.2 Test Task API
Test task management functionality:

```bash
# List tasks (should be empty initially)
curl -s http://localhost:3001/api/tasks | jq '.data.tasks | length'

# Create a test task
curl -s -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Test task for setup verification",
    "type": "test",
    "priority": 1
  }' | jq .

# Monitor task processing
sleep 2
curl -s http://localhost:3001/api/tasks | jq '.data.tasks[0].status'
```

**Expected Output:**
- Task creation succeeds
- Task gets processed by the agent
- Task status shows as completed

### Step 6: System Shutdown and Cleanup (5 minutes)

#### 6.1 Graceful Shutdown
Stop the system properly:

```bash
# Send termination signal
kill -TERM $HIVE_PID

# Wait for graceful shutdown
sleep 3

# Check if process is still running
ps -p $HIVE_PID || echo "Process terminated successfully"
```

**Expected Output:**
- Process terminates gracefully
- No error messages during shutdown

#### 6.2 Cleanup
Clean up exercise artifacts:

```bash
# Remove test database
rm -f data/hive_dev.db

# Remove development config
rm -f settings/development.toml

# Check remaining files
ls -la data/
ls -la settings/
```

**Expected Output:**
- Test artifacts cleaned up
- Original configuration preserved

## Verification Checklist

Use this checklist to verify your setup is complete:

- [ ] System requirements met (RAM, disk, dependencies)
- [ ] Repository cloned successfully
- [ ] System builds without errors
- [ ] Configuration files created and valid
- [ ] System starts and responds to HTTP requests
- [ ] Health checks pass for all modules
- [ ] Basic API operations work (agents, tasks)
- [ ] System shuts down gracefully
- [ ] Cleanup completed successfully

## Common Issues and Solutions

### Build Failures
**Issue:** Compilation errors during `cargo build`  
**Solution:**
```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update

# Check Rust version
rustc --version  # Should be 1.70+
```

### Port Conflicts
**Issue:** Port 3001 already in use  
**Solution:**
```bash
# Find conflicting process
sudo netstat -tulpn | grep :3001

# Kill conflicting process or change port
sed -i 's/port = 3001/port = 3002/' settings/development.toml
```

### Database Issues
**Issue:** SQLite database creation fails  
**Solution:**
```bash
# Check permissions
ls -la data/
mkdir -p data
chmod 755 data

# Verify SQLite installation
sqlite3 --version
```

### Health Check Failures
**Issue:** Health endpoint returns errors  
**Solution:**
```bash
# Check system logs
sudo journalctl -u ai-orchestrator-hub -n 20

# Verify all modules started
curl -s http://localhost:3001/health | jq '.data.modules'

# Restart system
kill -TERM $HIVE_PID
sleep 2
./target/release/ai-orchestrator-hub --config settings/development.toml &
```

## Next Steps

Now that you have successfully set up the AI Orchestrator Hub, you can:

1. **Explore Advanced Features**: Try creating different agent types and complex tasks
2. **Monitor System Performance**: Set up monitoring dashboards and alerts
3. **Scale the System**: Add more agents and test concurrent task processing
4. **Integrate External Systems**: Connect the API to your applications

## Additional Resources

- [System Architecture Overview](../system-architecture.md)
- [Operational Procedures](../operational-procedures.md)
- [API Reference](../api.md)
- [Troubleshooting Guide](../troubleshooting-guide.md)

## Exercise Completion

Congratulations! You have successfully completed the basic system setup exercise. Your AI Orchestrator Hub is now ready for development and testing.

**Exercise Score:** Record the time taken and any issues encountered for future reference.