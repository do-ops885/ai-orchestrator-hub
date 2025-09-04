# Multiagent Hive System - Usage Guides

This document provides practical usage guides for common tasks and workflows in the Multiagent Hive System.

## Getting Started

### Basic Setup

1. **Clone and build the project:**
```bash
git clone https://github.com/your-org/multiagent-hive.git
cd multiagent-hive/backend
cargo build
```

2. **Start the server:**
```bash
cargo run
```

3. **Verify the server is running:**
```bash
curl http://localhost:3001/
# Should return: "🐝 Multiagent Hive System API v2.0 - CPU-native, GPU-optional"
```

### Health Check

Check system health:
```bash
curl http://localhost:3001/health
```

Expected response:
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "components": {
      "hive_coordinator": { "status": "healthy" },
      "resource_manager": { "status": "healthy" },
      "metrics_collector": { "status": "healthy" }
    }
  }
}
```

## Agent Management

### Creating Agents

#### Basic Worker Agent

```bash
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
  }'
```

#### Specialist Agent

```bash
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "DataAnalyst-1",
    "type": "specialist:data_analysis",
    "capabilities": [
      {
        "name": "data_analysis",
        "proficiency": 0.9,
        "learning_rate": 0.05
      },
      {
        "name": "report_writing",
        "proficiency": 0.8,
        "learning_rate": 0.08
      }
    ]
  }'
```

#### Coordinator Agent

```bash
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Coordinator-1",
    "type": "coordinator",
    "capabilities": [
      {
        "name": "coordination",
        "proficiency": 0.95,
        "learning_rate": 0.02
      },
      {
        "name": "quality_review",
        "proficiency": 0.9,
        "learning_rate": 0.03
      }
    ]
  }'
```

### Listing Agents

```bash
curl http://localhost:3001/api/agents
```

Response:
```json
{
  "success": true,
  "data": {
    "agents": [
      {
        "id": "uuid-1",
        "name": "Worker-1",
        "type": "worker",
        "state": "Active",
        "capabilities": [...],
        "performance_score": 0.85,
        "tasks_completed": 15
      }
    ]
  }
}
```

## Task Management

### Creating Tasks

#### Simple Task

```bash
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Process customer feedback data",
    "type": "data_processing",
    "priority": 1
  }'
```

#### Task with Requirements

```bash
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Generate sales analytics report",
    "type": "data_analysis",
    "priority": 2,
    "required_capabilities": [
      {
        "name": "data_analysis",
        "min_proficiency": 0.7
      }
    ]
  }'
```

#### High-Priority Task

```bash
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Fix critical system vulnerability",
    "type": "security_audit",
    "priority": 3,
    "required_capabilities": [
      {
        "name": "security_analysis",
        "min_proficiency": 0.9
      }
    ]
  }'
```

### Listing Tasks

```bash
curl http://localhost:3001/api/tasks
```

Response:
```json
{
  "success": true,
  "data": {
    "tasks": [
      {
        "id": "uuid-task-1",
        "description": "Process customer feedback data",
        "type": "data_processing",
        "priority": 1,
        "status": "completed",
        "assigned_agent": "uuid-agent-1",
        "execution_time_ms": 2500
      }
    ]
  }
}
```

## System Monitoring

### Getting System Metrics

```bash
curl http://localhost:3001/metrics
```

Response:
```json
{
  "success": true,
  "data": {
    "current_metrics": {
      "performance": {
        "average_response_time_ms": 25.5,
        "requests_per_second": 8.3
      },
      "agents": {
        "total_agents": 5,
        "active_agents": 3,
        "average_agent_performance": 0.82
      },
      "tasks": {
        "total_tasks_submitted": 150,
        "total_tasks_completed": 142,
        "task_success_rate": 94.7
      }
    }
  }
}
```

### Getting Hive Status

```bash
curl http://localhost:3001/api/hive/status
```

Response:
```json
{
  "success": true,
  "data": {
    "metrics": {
      "total_agents": 5,
      "active_agents": 3,
      "completed_tasks": 142,
      "average_performance": 0.82,
      "swarm_cohesion": 0.78
    }
  }
}
```

## Real-time Monitoring with WebSocket

### Connecting to WebSocket

```javascript
const ws = new WebSocket('ws://localhost:3001/ws');

ws.onopen = () => {
    console.log('Connected to Hive WebSocket');
};

ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    console.log('Received:', message);
};

ws.onclose = () => {
    console.log('WebSocket connection closed');
};
```

### Handling Different Message Types

```javascript
ws.onmessage = (event) => {
    const message = JSON.parse(event.data);

    switch (message.type) {
        case 'agent_created':
            console.log('New agent created:', message.data.agent.name);
            break;

        case 'task_completed':
            console.log('Task completed:', message.data.task_id);
            console.log('Execution time:', message.data.execution_time_ms, 'ms');
            break;

        case 'task_failed':
            console.error('Task failed:', message.data.task_id);
            console.error('Error:', message.data.error);
            break;

        case 'system_alert':
            if (message.data.level === 'critical') {
                console.error('CRITICAL ALERT:', message.data.title);
            } else if (message.data.level === 'warning') {
                console.warn('WARNING:', message.data.title);
            }
            break;

        default:
            console.log('Unknown message type:', message.type);
    }
};
```

## Working with Examples

### Running the Simple Verification Demo

```bash
cargo run --example simple_verification_demo
```

This demo shows:
- Creating agents with different capabilities
- Creating tasks with various priorities
- Executing tasks with simple verification
- Comparing verification tiers
- Configuring custom verification rules

### Running the Neural Comparison Demo

```bash
cargo run --example neural_comparison
```

This demo demonstrates:
- Neural processing capabilities
- CPU vs GPU performance comparison
- Different neural network configurations

### Running the Agent Monitor Example

```bash
cargo run --example agent_monitor_example
```

This example shows:
- Real-time agent monitoring
- Performance tracking
- Health status monitoring

## Advanced Usage Patterns

### Creating a Complete Workflow

1. **Create specialized agents:**
```bash
# Data processing agent
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "DataProcessor",
    "type": "specialist:data_processing",
    "capabilities": [{"name": "data_processing", "proficiency": 0.9}]
  }'

# Analysis agent
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "DataAnalyst",
    "type": "specialist:data_analysis",
    "capabilities": [{"name": "data_analysis", "proficiency": 0.85}]
  }'
```

2. **Create a multi-step task pipeline:**
```bash
# Step 1: Data processing
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Clean and preprocess raw data",
    "type": "data_processing",
    "priority": 2
  }'

# Step 2: Data analysis
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Analyze processed data for insights",
    "type": "data_analysis",
    "priority": 2
  }'
```

3. **Monitor progress:**
```bash
# Check task status
curl http://localhost:3001/api/tasks

# Monitor system health
curl http://localhost:3001/health

# Get detailed metrics
curl http://localhost:3001/metrics
```

### Handling Errors and Rate Limits

```javascript
async function createAgentWithRetry(agentData, maxRetries = 3) {
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
        try {
            const response = await fetch('http://localhost:3001/api/agents', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(agentData)
            });

            const result = await response.json();

            if (response.ok) {
                return result;
            }

            if (response.status === 429) {
                // Rate limited - wait and retry
                const retryAfter = result.error.details?.retry_after_seconds || 60;
                console.log(`Rate limited. Retrying in ${retryAfter} seconds...`);
                await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));
                continue;
            }

            if (response.status === 400) {
                // Validation error
                console.error('Validation error:', result.error.field_errors);
                return result;
            }

            throw new Error(`HTTP ${response.status}: ${result.error.message}`);

        } catch (error) {
            console.error(`Attempt ${attempt} failed:`, error.message);

            if (attempt === maxRetries) {
                throw error;
            }

            // Wait before retrying
            await new Promise(resolve => setTimeout(resolve, 1000 * attempt));
        }
    }
}
```

### Batch Operations

```javascript
async function createMultipleAgents(agentsData) {
    const results = [];

    for (const agentData of agentsData) {
        try {
            const result = await createAgentWithRetry(agentData);
            results.push({ success: true, data: result.data });
        } catch (error) {
            results.push({ success: false, error: error.message });
        }
    }

    return results;
}

// Usage
const agentsToCreate = [
    {
        name: "Worker-1",
        type: "worker",
        capabilities: [{ name: "general_tasks", proficiency: 0.8 }]
    },
    {
        name: "Worker-2",
        type: "worker",
        capabilities: [{ name: "general_tasks", proficiency: 0.75 }]
    }
];

const results = await createMultipleAgents(agentsToCreate);
console.log('Batch creation results:', results);
```

## Configuration

### Environment Variables

Create a `.env` file or set environment variables:

```bash
# Server configuration
export HIVE_SERVER__HOST=localhost
export HIVE_SERVER__PORT=3001

# Logging
export HIVE_LOGGING__LEVEL=info

# Performance
export HIVE_PERFORMANCE__CPU_WARNING_THRESHOLD=70.0
export HIVE_PERFORMANCE__MEMORY_WARNING_THRESHOLD=80.0

# Neural processing
export HIVE_NEURAL__LEARNING_RATE=0.01
```

### Configuration File

The system uses `settings/default.toml` for configuration:

```toml
[server]
host = "localhost"
port = 3001

[logging]
level = "info"

[performance]
cpu_warning_threshold = 70.0
memory_warning_threshold = 80.0
metrics_collection_interval_ms = 30000

[neural]
learning_rate = 0.01
momentum = 0.9
```

## Troubleshooting

### Common Issues

#### Server Won't Start

**Problem**: Port already in use or configuration error

**Solution**:
```bash
# Check if port is in use
lsof -i :3001

# Try different port
export HIVE_SERVER__PORT=3002
cargo run
```

#### Agent Creation Fails

**Problem**: Invalid agent configuration

**Solution**: Check the error response for validation details:
```bash
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{"invalid": "config"}'
```

#### Tasks Not Being Processed

**Problem**: No suitable agents available

**Solution**:
```bash
# Check agent status
curl http://localhost:3001/api/agents

# Check task queue
curl http://localhost:3001/api/tasks

# Verify agent capabilities match task requirements
```

#### High Memory Usage

**Problem**: System consuming too much memory

**Solution**:
```bash
# Check memory usage
curl http://localhost:3001/api/resources

# Monitor metrics
curl http://localhost:3001/metrics

# Consider reducing agent count or adjusting configuration
```

### Performance Optimization

#### Monitoring Performance

```bash
# Get performance metrics
curl http://localhost:3001/metrics

# Monitor response times
curl http://localhost:3001/health
```

#### Scaling the System

1. **Add more agents:**
```bash
# Create additional worker agents
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Worker-Extra",
    "type": "worker",
    "capabilities": [{"name": "general_tasks", "proficiency": 0.7}]
  }'
```

2. **Monitor system resources:**
```bash
curl http://localhost:3001/api/resources
```

3. **Adjust configuration for better performance:**
```toml
[performance]
cpu_warning_threshold = 80.0
memory_warning_threshold = 85.0
metrics_collection_interval_ms = 60000
```

### Logging and Debugging

#### Enable Debug Logging

```bash
export HIVE_LOGGING__LEVEL=debug
cargo run
```

#### Check System Logs

The system uses structured logging. Logs include:
- Request IDs for correlation
- Performance metrics
- Error details
- Security events

#### WebSocket Debugging

```javascript
const ws = new WebSocket('ws://localhost:3001/ws');

ws.onmessage = (event) => {
    console.log('Raw message:', event.data);
    try {
        const parsed = JSON.parse(event.data);
        console.log('Parsed message:', parsed);
    } catch (e) {
        console.error('Failed to parse message:', e);
    }
};
```

## Best Practices

### Agent Management

1. **Use descriptive names** for agents
2. **Set appropriate proficiency levels** based on capabilities
3. **Monitor agent performance** regularly
4. **Balance agent types** (workers, specialists, coordinators)

### Task Management

1. **Set realistic priorities** (0-3 scale)
2. **Provide clear descriptions** for tasks
3. **Specify capability requirements** when needed
4. **Monitor task completion rates**

### System Monitoring

1. **Regular health checks** using `/health` endpoint
2. **Monitor key metrics** via `/metrics`
3. **Set up alerts** for critical thresholds
4. **Review performance trends**

### Error Handling

1. **Check response status codes**
2. **Parse error responses** for details
3. **Implement retry logic** for transient failures
4. **Handle rate limiting** appropriately

### Security

1. **Validate all inputs** before sending
2. **Use HTTPS in production**
3. **Monitor for unusual activity**
4. **Keep dependencies updated**

## Integration Examples

### Integrating with External Systems

#### Webhook Notifications

```javascript
class HiveWebhookHandler {
    constructor(webhookUrl) {
        this.webhookUrl = webhookUrl;
        this.ws = new WebSocket('ws://localhost:3001/ws');

        this.ws.onmessage = this.handleMessage.bind(this);
    }

    async handleMessage(event) {
        const message = JSON.parse(event.data);

        // Forward important events to external system
        if (['task_completed', 'task_failed', 'system_alert'].includes(message.type)) {
            await this.sendWebhook(message);
        }
    }

    async sendWebhook(message) {
        try {
            await fetch(this.webhookUrl, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    source: 'multiagent-hive',
                    event: message.type,
                    data: message.data,
                    timestamp: message.timestamp
                })
            });
        } catch (error) {
            console.error('Webhook failed:', error);
        }
    }
}
```

#### Database Integration

```javascript
class HiveDatabaseLogger {
    constructor(dbConnection) {
        this.db = dbConnection;
        this.ws = new WebSocket('ws://localhost:3001/ws');

        this.ws.onmessage = this.logToDatabase.bind(this);
    }

    async logToDatabase(event) {
        const message = JSON.parse(event.data);

        try {
            await this.db.collection('hive_events').insertOne({
                type: message.type,
                data: message.data,
                timestamp: new Date(message.timestamp),
                logged_at: new Date()
            });
        } catch (error) {
            console.error('Database logging failed:', error);
        }
    }
}
```

This comprehensive usage guide covers the most common operations and patterns for working with the Multiagent Hive System. For more advanced features, refer to the API documentation and module documentation.
