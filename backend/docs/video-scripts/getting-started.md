# Video Tutorial: Getting Started with AI Orchestrator Hub

**Duration:** ~30 minutes  
**Target Audience:** Beginners, System Administrators  
**Prerequisites:** Basic Linux command line knowledge  

---

## Video Script Outline

### Introduction (0:00 - 2:00)
**Visual:** Fade in with AI Orchestrator Hub logo, system architecture diagram  
**Narrator:**  
"Welcome to the AI Orchestrator Hub! In this tutorial, we'll get you up and running with your first multi-agent system. Whether you're a developer looking to build intelligent applications or an operator managing AI workflows, this video will guide you through the initial setup and basic operations."

**On Screen:**
- Title: "Getting Started with AI Orchestrator Hub"
- Subtitle: "Deploy, Configure, and Run Your First Multi-Agent System"
- Learning Objectives:
  - Install and configure the system
  - Create your first agent
  - Submit and monitor tasks
  - Understand basic monitoring

---

### Section 1: System Requirements and Installation (2:00 - 8:00)

#### Prerequisites Check (2:00 - 3:00)
**Visual:** Terminal window, system checking commands  
**Narrator:**  
"Before we begin, let's ensure your system meets the requirements. The AI Orchestrator Hub runs on Linux and requires Rust 1.70+, along with some system dependencies."

**Terminal Commands:**
```bash
# Check system information
uname -a
lsb_release -a

# Check available memory and CPU
free -h
nproc

# Verify Rust installation
rustc --version
cargo --version
```

#### Installation Process (3:00 - 6:00)
**Visual:** Step-by-step terminal commands with explanations  
**Narrator:**  
"Now let's install the system. We'll clone the repository and build the application. This process typically takes 5-10 minutes depending on your system."

**Terminal Commands:**
```bash
# Clone the repository
git clone https://github.com/do-ops885/ai-orchestrator-hub.git
cd ai-orchestrator-hub/backend

# Build the project
cargo build --release

# Verify the build
ls -la target/release/
./target/release/ai-orchestrator-hub --help
```

**Visual:** Progress bars, compilation output  
**Narrator:**  
"Great! The build completed successfully. You can see the binary was created in target/release/. Let's test that it runs correctly."

#### Basic Configuration (6:00 - 8:00)
**Visual:** Configuration file editing, explanation of key settings  
**Narrator:**  
"Before starting the system, let's review the basic configuration. The system uses TOML files for configuration, with sensible defaults for development."

**File: settings/default.toml**
```toml
[server]
host = "localhost"
port = 3001

[logging]
level = "info"

[database]
url = "sqlite:data/hive_persistence.db"
```

**Narrator:**  
"For our first run, we'll use the default SQLite database which requires no additional setup. In production, you'd typically use PostgreSQL."

---

### Section 2: Starting the System (8:00 - 12:00)

#### First System Start (8:00 - 10:00)
**Visual:** Terminal starting the service, initial logs  
**Narrator:**  
"Now let's start the AI Orchestrator Hub for the first time. We'll run it in the foreground so we can see the startup process and initial logs."

**Terminal Commands:**
```bash
# Start the system
./target/release/ai-orchestrator-hub

# In another terminal, test connectivity
curl http://localhost:3001/
```

**Visual:** Server startup logs, health check response  
**Narrator:**  
"Excellent! The system is running. You can see it started successfully and is listening on port 3001. The welcome message confirms everything is working."

#### Health Check and Basic Monitoring (10:00 - 12:00)
**Visual:** Health endpoint responses, explanation of metrics  
**Narrator:**  
"Let's check the system's health and see what information it provides. The health endpoint gives us a comprehensive view of system status."

**Terminal Commands:**
```bash
# Comprehensive health check
curl http://localhost:3001/health | jq .

# System metrics
curl http://localhost:3001/metrics | jq '.data.current_metrics'
```

**Visual:** JSON responses formatted and explained  
**Narrator:**  
"The health endpoint shows all modules are healthy, and the metrics give us insight into system performance. Notice we have 0 agents and 0 tasks since we haven't created any yet."

---

### Section 3: Creating Your First Agent (12:00 - 18:00)

#### Understanding Agent Types (12:00 - 14:00)
**Visual:** Agent type diagrams, capability explanations  
**Narrator:**  
"Now let's create our first agent. Agents in the AI Orchestrator Hub are specialized entities with specific capabilities. There are several types: Worker, Specialist, Coordinator, and Learner."

**On Screen:**
- Agent Types:
  - **Worker**: General-purpose task execution
  - **Specialist**: Domain-specific expertise (e.g., data analysis, content writing)
  - **Coordinator**: Task management and coordination
  - **Learner**: Continuous learning and adaptation

#### Creating a Worker Agent (14:00 - 16:00)
**Visual:** API call construction, JSON payload explanation  
**Narrator:**  
"Let's create a simple worker agent that can handle general tasks. We'll use the REST API to create the agent with basic capabilities."

**Terminal Commands:**
```bash
# Create a worker agent
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Worker-1",
    "type": "worker",
    "capabilities": [
      {
        "name": "general_tasks",
        "proficiency": 0.8,
        "learning_rate": 0.1
      }
    ]
  }' | jq .
```

**Visual:** Success response, agent creation confirmation  
**Narrator:**  
"Perfect! The agent was created successfully. Notice the response includes the agent ID, which we'll need for task assignment."

#### Verifying Agent Creation (16:00 - 18:00)
**Visual:** Agent listing, detailed agent information  
**Narrator:**  
"Let's verify our agent was created and check its status. We'll list all agents and examine the details."

**Terminal Commands:**
```bash
# List all agents
curl http://localhost:3001/api/agents | jq '.data.agents[]'

# Check updated metrics
curl http://localhost:3001/metrics | jq '.data.current_metrics.agent_management'
```

**Visual:** Agent information display, metrics update  
**Narrator:**  
"There it is! Our Worker-1 agent is now active in the system. The metrics show we have 1 total agent and 1 active agent."

---

### Section 4: Submitting and Monitoring Tasks (18:00 - 25:00)

#### Creating Your First Task (18:00 - 20:00)
**Visual:** Task creation API call, task structure explanation  
**Narrator:**  
"Now let's create our first task. Tasks in the system have descriptions, types, priorities, and optional capability requirements."

**Terminal Commands:**
```bash
# Create a simple task
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Process sample data file",
    "type": "data_processing",
    "priority": 1
  }' | jq .
```

**Visual:** Task creation response, task ID assignment  
**Narrator:**  
"Great! The task was created and assigned an ID. Since we have an agent available, the system should automatically assign and process this task."

#### Monitoring Task Execution (20:00 - 23:00)
**Visual:** Task status checking, real-time updates  
**Narrator:**  
"Let's monitor the task execution. We'll check the task status and see how the system processes it."

**Terminal Commands:**
```bash
# Check task status
curl http://localhost:3001/api/tasks | jq '.data.tasks[]'

# Monitor system activity (run multiple times)
curl http://localhost:3001/metrics | jq '.data.current_metrics.task_management'
```

**Visual:** Task status progression, metrics updates  
**Narrator:**  
"You can see the task status changed from 'pending' to 'completed'. The system automatically assigned it to our Worker-1 agent and processed it successfully."

#### Understanding Task Results (23:00 - 25:00)
**Visual:** Task details with execution information  
**Narrator:**  
"Let's examine the completed task to see the execution details and results."

**Terminal Commands:**
```bash
# Get detailed task information
curl http://localhost:3001/api/tasks | jq '.data.tasks[] | select(.status == "completed")'

# Check agent performance
curl http://localhost:3001/api/agents | jq '.data.agents[]'
```

**Visual:** Task execution details, agent performance metrics  
**Narrator:**  
"The task shows execution time and success status. Our agent now has updated performance metrics showing it completed one task successfully."

---

### Section 5: Real-time Monitoring with WebSocket (25:00 - 28:00)

#### Connecting to WebSocket (25:00 - 26:00)
**Visual:** WebSocket connection demonstration  
**Narrator:**  
"The AI Orchestrator Hub provides real-time updates via WebSocket. Let's connect and see live system events."

**Terminal Commands:**
```bash
# Connect to WebSocket (using websocat or similar tool)
websocat ws://localhost:3001/ws
```

**Visual:** WebSocket connection, initial messages  
**Narrator:**  
"We're now connected to the WebSocket stream. We'll see real-time events as they happen in the system."

#### Observing Live Events (26:00 - 28:00)
**Visual:** Real-time event stream, system activity  
**Narrator:**  
"Let's create another task while watching the WebSocket to see the live events."

**Terminal Commands:**
```bash
# In another terminal, create a new task
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Analyze system performance",
    "type": "analysis",
    "priority": 2
  }'
```

**Visual:** WebSocket events appearing in real-time  
**Narrator:**  
"You can see the live events streaming through the WebSocket connection - task creation, agent assignment, and completion events."

---

### Conclusion and Next Steps (28:00 - 30:00)

#### Summary (28:00 - 29:00)
**Visual:** Recap of what was accomplished, system status  
**Narrator:**  
"In this tutorial, we successfully installed and configured the AI Orchestrator Hub, created our first agent, submitted and monitored tasks, and explored real-time monitoring capabilities."

**On Screen:**
- ✅ System installation and configuration
- ✅ Agent creation and management
- ✅ Task submission and monitoring
- ✅ Real-time WebSocket monitoring
- ✅ Basic health checks and metrics

#### What's Next (29:00 - 30:00)
**Visual:** Teaser for next videos, resource links  
**Narrator:**  
"This is just the beginning! In the next tutorial, we'll dive deeper into agent types and capabilities. Check the documentation for advanced features and join our community for support."

**On Screen:**
- Next: "Understanding Agents and Tasks"
- Resources:
  - Documentation: docs/
  - API Reference: docs/api.md
  - Community: GitHub Discussions

---

## Production Notes

### Video Production Checklist
- [ ] Record at 1080p 30fps
- [ ] Use clear, large terminal text
- [ ] Include system diagrams and animations
- [ ] Add captions for accessibility
- [ ] Test all commands before recording
- [ ] Include error handling demonstrations

### Technical Requirements
- [ ] Clean Linux environment
- [ ] Pre-built binary for faster demo
- [ ] websocat or similar WebSocket client
- [ ] jq for JSON formatting
- [ ] Multiple terminal windows

### Accessibility Features
- [ ] Closed captions
- [ ] Clear audio narration
- [ ] Visual indicators for key actions
- [ ] Text overlays for commands
- [ ] High contrast terminal themes