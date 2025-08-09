# Quick Start Guide - Multiagent Hive System

## System Overview

Your hybrid neural multiagent hive system is ready to run! **CPU-native, GPU-optional - built for the GPU-poor.** Here's how to get started:

## Prerequisites

### Required Tools
```bash
# Rust (for backend)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Node.js (for frontend)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Verify installations
cargo --version
node --version
npm --version
```

## Launch Sequence

### Step 1: Start Backend (Basic Mode)
```bash
cd backend
cargo run
```

**Expected Output:**
```
INFO Starting Multiagent Hive System
INFO Server running on http://0.0.0.0:3001
INFO Created hive coordinator with ID: [uuid]
```

### Step 2: Start Frontend
```bash
# In a new terminal
cd frontend
npm install
npm run dev
```

**Expected Output:**
```
ready - started server on 0.0.0.0:3000, url: http://localhost:3000
```

### Step 3: Access Dashboard
Open your browser to: **http://localhost:3000**

You should see:
- 🐝 Multiagent Hive System header
- Connection status (should show "Connected")
- Three tabs: Dashboard, Agents, Tasks

## 🧪 Testing Basic Neural Mode

### Create Your First Agent
1. Click **"Agents"** tab
2. Click **"Create Agent"** button
3. Fill in:
   ```
   Name: TestAgent-001
   Type: Worker
   Capabilities:
   - Name: communication, Proficiency: 0.8
   - Name: analysis, Proficiency: 0.6
   ```
4. Click **"Create Agent"**

### Create Your First Task
1. Click **"Tasks"** tab
2. Click **"Create Task"** button
3. Fill in:
   ```
   Description: Analyze incoming data stream
   Type: data_processing
   Priority: 5
   Required Capabilities:
   - Name: analysis, Min Proficiency: 0.5
   ```
4. Click **"Create Task"**

### Monitor the Hive
1. Go back to **"Dashboard"** tab
2. Watch the **Swarm Visualization** - you should see your agent as a colored dot
3. Check **Hive Metrics** - should show 1 total agent
4. Monitor **Neural Metrics** - should show "Basic NLP" mode

## 🚀 Testing Advanced Neural Mode

### Enable Advanced Features
```bash
# Stop the backend (Ctrl+C) and restart with advanced features
cd backend
cargo run --features advanced-neural
```

### Create Advanced Neural Agent
1. In the frontend, create a new agent:
   ```
   Name: NeuralAgent-001
   Type: specialist:pattern_recognition
   Capabilities:
   - Name: pattern_recognition, Proficiency: 0.9
   - Name: deep_learning, Proficiency: 0.8
   ```
2. **Note**: In a real implementation, you'd add a checkbox for "Use Advanced Neural" in the UI

### Run Neural Comparison Demo
```bash
# In backend directory
cargo run --features advanced-neural --example neural_comparison
```

**Expected Output:**
```
🧠 Neural Processing Comparison Demo
=====================================
✅ Created basic agent: [uuid]
🚀 Created advanced agent: [uuid]

🧪 Testing Scenarios:
---------------------
📋 Scenario: Analyze customer sentiment in reviews
  📊 Basic Agent:
    - Sentiment: 0.23
    - Keywords: ["analyze", "customer", "sentiment"]
  🚀 Advanced Agent:
    - Sentiment: 0.31
    - Keywords: ["analyze", "customer", "sentiment"]
  📈 Performance Predictions:
    - Basic: 67.2%
    - Advanced: 73.8%
```

## 🔍 What to Look For

### Basic Mode Indicators
- ✅ Fast startup (~50ms)
- ✅ Low memory usage
- ✅ Real-time agent coordination
- ✅ Basic sentiment analysis working
- ✅ Pattern learning from interactions

### Advanced Mode Indicators
- 🚀 Enhanced accuracy in predictions
- 🚀 Better pattern recognition
- 🚀 More sophisticated learning
- ⚠️ Slightly higher memory usage
- ⚠️ Longer startup time

### Dashboard Features to Test
1. **Swarm Visualization**: Agents moving and coordinating
2. **Real-time Metrics**: Updates every 5 seconds
3. **Agent Management**: Create/view agents with capabilities
4. **Task Management**: Create tasks and watch assignment
5. **Neural Metrics**: Performance comparison between modes

## 🐛 Troubleshooting

### Backend Won't Start
```bash
# Check Rust installation
rustc --version

# Update dependencies
cargo update

# Clean and rebuild
cargo clean && cargo build
```

### Frontend Won't Start
```bash
# Clear npm cache
npm cache clean --force

# Delete node_modules and reinstall
rm -rf node_modules package-lock.json
npm install
```

### WebSocket Connection Issues
- Check that backend is running on port 3001
- Verify no firewall blocking connections
- Look for CORS errors in browser console

### No Agents Visible
- Check browser console for JavaScript errors
- Verify WebSocket connection status
- Try refreshing the page

## 📊 Performance Testing

### Load Testing
```bash
# Create multiple agents quickly
for i in {1..10}; do
  curl -X POST http://localhost:3001/api/agents \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"Agent-$i\",\"type\":\"Worker\"}"
done
```

### Memory Monitoring
```bash
# Monitor backend memory usage
ps aux | grep multiagent-hive

# Monitor with htop
htop -p $(pgrep multiagent-hive)
```

### Network Monitoring
```bash
# Monitor WebSocket connections
netstat -an | grep 3001

# Monitor HTTP requests
curl -s http://localhost:3001/api/hive/status | jq
```

## 🎯 Success Criteria

### ✅ Basic System Working
- [ ] Backend starts without errors
- [ ] Frontend connects successfully
- [ ] Can create agents and tasks
- [ ] Swarm visualization shows agents
- [ ] Real-time updates working

### 🚀 Advanced Features Working
- [ ] Advanced neural mode starts
- [ ] Neural comparison demo runs
- [ ] Performance improvements visible
- [ ] Enhanced accuracy in processing
- [ ] Neural metrics showing hybrid mode

## 🔄 Next Steps

Once the system is running successfully:

1. **🧪 Experiment**: Create different agent types and task complexities
2. **📊 Monitor**: Watch performance metrics and learning progress
3. **🎨 Customize**: Modify agent capabilities and behaviors
4. **⚡ Optimize**: Enable GPU acceleration for large swarms
5. **🌐 Deploy**: Set up production environment

## 🆘 Need Help?

### Common Issues
- **Port conflicts**: Change ports in configuration files
- **Permission errors**: Run with appropriate user permissions
- **Dependency issues**: Update Rust/Node.js to latest versions

### Debug Mode
```bash
# Run backend with debug logging
RUST_LOG=debug cargo run

# Run frontend in development mode
npm run dev
```

### Log Files
- Backend logs: Check terminal output
- Frontend logs: Check browser developer console
- System logs: Check system journal if needed