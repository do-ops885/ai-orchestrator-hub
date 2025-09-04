# Multiagent Hive API Documentation

This document provides comprehensive documentation for the Multiagent Hive System REST API and WebSocket endpoints.

## Overview

The API provides programmatic access to manage agents, tasks, and monitor the hive system. All endpoints return standardized JSON responses with consistent error handling.

## Base URL

```
http://localhost:3001
```

## Authentication

Currently, the API does not require authentication. Rate limiting is applied to prevent abuse.

## Response Format

All API responses follow this standardized format:

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "timestamp": "2024-01-01T00:00:00Z",
  "request_id": "uuid-v4-optional"
}
```

### Error Response Format

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": { ... },
    "field_errors": [
      {
        "field": "field_name",
        "message": "Field-specific error",
        "value": "invalid_value"
      }
    ]
  },
  "timestamp": "2024-01-01T00:00:00Z",
  "request_id": "uuid-v4-optional"
}
```

## Core Endpoints

### GET /

Returns a simple status message indicating the server is running.

**Response:**
```json
"🐝 Multiagent Hive System API v2.0 - CPU-native, GPU-optional"
```

### GET /health

Provides comprehensive health check information about all system components.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "timestamp": "2024-01-01T00:00:00Z",
    "response_time_ms": 15,
    "version": "2.0.0",
    "components": {
      "hive_coordinator": {
        "status": "healthy",
        "total_agents": 5,
        "active_agents": 3,
        "completed_tasks": 42,
        "average_performance": 0.85
      },
      "resource_manager": {
        "status": "healthy",
        "memory_usage_percent": 65.2,
        "cpu_usage_percent": 45.8,
        "available_memory_mb": 2048,
        "cpu_cores": 8
      },
      "metrics_collector": {
        "status": "healthy",
        "response_time_ms": 5,
        "requests_per_second": 12.5,
        "error_rate": 0.02
      },
      "intelligent_alerting": {
        "status": "healthy",
        "active_rules": "monitoring",
        "system_operational": true
      }
    },
    "system_info": {
      "cpu_native": true,
      "gpu_optional": true,
      "phase_2_active": true,
      "swarm_cohesion": 0.78,
      "learning_progress": 0.65
    }
  }
}
```

### GET /metrics

Returns current system metrics and performance trends.

**Response:**
```json
{
  "success": true,
  "data": {
    "current_metrics": {
      "performance": {
        "average_response_time_ms": 25.5,
        "requests_per_second": 8.3,
        "error_rate_per_minute": 0.01
      },
      "system": {
        "cpu_usage_percent": 45.2,
        "memory_usage_percent": 62.8,
        "disk_usage_percent": 34.1
      },
      "agents": {
        "total_agents": 5,
        "active_agents": 3,
        "idle_agents": 2,
        "failed_agents": 0,
        "average_agent_performance": 0.82
      },
      "tasks": {
        "total_tasks_submitted": 150,
        "total_tasks_completed": 142,
        "total_tasks_failed": 3,
        "tasks_in_queue": 5,
        "average_task_duration_ms": 1250,
        "task_success_rate": 94.7
      }
    },
    "trends": {
      "cpu_trend": "stable",
      "memory_trend": "increasing",
      "task_completion_trend": "stable"
    },
    "collection_timestamp": "2024-01-01T00:00:00Z"
  }
}
```

## Agent Management

### GET /api/agents

Retrieves information about all agents in the hive.

**Response:**
```json
{
  "success": true,
  "data": {
    "agents": [
      {
        "id": "uuid-1",
        "name": "DataProcessor-1",
        "type": "specialist",
        "state": "Active",
        "capabilities": [
          {
            "name": "data_processing",
            "proficiency": 0.85,
            "learning_rate": 0.1
          }
        ],
        "performance_score": 0.82,
        "tasks_completed": 25,
        "created_at": "2024-01-01T00:00:00Z"
      }
    ]
  }
}
```

### POST /api/agents

Creates a new agent in the hive.

**Request Body:**
```json
{
  "name": "ContentWriter-1",
  "type": "specialist:content_writing",
  "capabilities": [
    {
      "name": "content_writing",
      "proficiency": 0.8,
      "learning_rate": 0.12
    },
    {
      "name": "editing",
      "proficiency": 0.75,
      "learning_rate": 0.1
    }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "agent_id": "uuid-new-agent",
    "message": "Agent created successfully"
  }
}
```

**Error Codes:**
- `VALIDATION_ERROR`: Invalid agent configuration
- `AGENT_CREATION_FAILED`: Failed to create agent due to system constraints

## Task Management

### GET /api/tasks

Retrieves information about all tasks in the system.

**Response:**
```json
{
  "success": true,
  "data": {
    "tasks": [
      {
        "id": "uuid-task-1",
        "description": "Analyze customer satisfaction data",
        "type": "data_analysis",
        "priority": 2,
        "status": "completed",
        "assigned_agent": "uuid-agent-1",
        "created_at": "2024-01-01T00:00:00Z",
        "completed_at": "2024-01-01T00:05:00Z",
        "execution_time_ms": 5000
      }
    ]
  }
}
```

### POST /api/tasks

Creates a new task for execution by agents.

**Request Body:**
```json
{
  "description": "Generate monthly sales report",
  "type": "reporting",
  "priority": 1,
  "required_capabilities": [
    {
      "name": "data_analysis",
      "min_proficiency": 0.7
    }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "task_id": "uuid-new-task",
    "message": "Task created successfully"
  }
}
```

**Error Codes:**
- `VALIDATION_ERROR`: Invalid task configuration
- `TASK_CREATION_FAILED`: Failed to create task

## Hive Management

### GET /api/hive/status

Retrieves comprehensive status information about the entire hive system.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "hive-uuid",
    "created_at": "2024-01-01T00:00:00Z",
    "last_update": "2024-01-01T00:30:00Z",
    "metrics": {
      "total_agents": 5,
      "active_agents": 3,
      "completed_tasks": 142,
      "failed_tasks": 3,
      "average_performance": 0.82,
      "swarm_cohesion": 0.78,
      "learning_progress": 0.65
    },
    "swarm_center": [0.5, 0.5],
    "total_energy": 15.2
  }
}
```

### GET /api/resources

Retrieves information about system resource usage and availability.

**Response:**
```json
{
  "success": true,
  "data": {
    "system_resources": {
      "cpu_usage": 45.8,
      "memory_usage": 62.4,
      "available_memory": 2048,
      "cpu_cores": 8,
      "disk_usage": 34.1
    },
    "hive_resources": {
      "allocated_memory": 1024,
      "active_connections": 12,
      "queue_depth": 5
    }
  }
}
```

## WebSocket API

### Connection

Connect to the WebSocket endpoint for real-time updates:

```
ws://localhost:3001/ws
```

### Message Format

All WebSocket messages follow this format:

```json
{
  "type": "message_type",
  "data": { ... },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### Supported Message Types

#### Agent Events

```json
{
  "type": "agent_created",
  "data": {
    "agent": {
      "id": "uuid-agent",
      "name": "NewAgent-1",
      "type": "worker",
      "capabilities": [...]
    }
  }
}
```

```json
{
  "type": "agent_status_changed",
  "data": {
    "agent_id": "uuid-agent",
    "old_status": "idle",
    "new_status": "active",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

#### Task Events

```json
{
  "type": "task_created",
  "data": {
    "task": {
      "id": "uuid-task",
      "description": "Process data",
      "priority": 1,
      "status": "pending"
    }
  }
}
```

```json
{
  "type": "task_completed",
  "data": {
    "task_id": "uuid-task",
    "agent_id": "uuid-agent",
    "execution_time_ms": 2500,
    "result": { ... }
  }
}
```

```json
{
  "type": "task_failed",
  "data": {
    "task_id": "uuid-task",
    "agent_id": "uuid-agent",
    "error": "Task execution failed",
    "retry_count": 2
  }
}
```

#### System Events

```json
{
  "type": "hive_status_update",
  "data": {
    "metrics": { ... },
    "alerts": [...]
  }
}
```

```json
{
  "type": "system_alert",
  "data": {
    "level": "warning",
    "title": "High CPU Usage",
    "description": "CPU usage has exceeded 80%",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

## Rate Limiting

The API implements rate limiting to prevent abuse:

- **Agent creation**: 10 requests per minute
- **Task creation**: 20 requests per minute
- **Status endpoints**: 60 requests per minute

When rate limited, you'll receive:

```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded",
    "details": {
      "retry_after_seconds": 60
    }
  }
}
```

## Error Codes

### Common Error Codes

- `VALIDATION_ERROR`: Input validation failed
- `RATE_LIMIT_EXCEEDED`: Too many requests
- `INTERNAL_ERROR`: Unexpected server error
- `SERVICE_UNAVAILABLE`: Service temporarily unavailable

### Agent-Specific Errors

- `AGENT_NOT_FOUND`: Specified agent doesn't exist
- `AGENT_CREATION_FAILED`: Failed to create agent
- `AGENT_LIMIT_EXCEEDED`: Maximum number of agents reached

### Task-Specific Errors

- `TASK_NOT_FOUND`: Specified task doesn't exist
- `TASK_CREATION_FAILED`: Failed to create task
- `TASK_ASSIGNMENT_FAILED`: Could not assign task to agent

## Client Examples

### JavaScript/Node.js

```javascript
const axios = require('axios');

async function createAgent() {
    try {
        const response = await axios.post('http://localhost:3001/api/agents', {
            name: 'Worker-1',
            type: 'worker',
            capabilities: [
                {
                    name: 'data_processing',
                    proficiency: 0.8,
                    learning_rate: 0.1
                }
            ]
        });

        console.log('Agent created:', response.data);
    } catch (error) {
        console.error('Error:', error.response.data);
    }
}

async function getHiveStatus() {
    try {
        const response = await axios.get('http://localhost:3001/api/hive/status');
        console.log('Hive status:', response.data);
    } catch (error) {
        console.error('Error:', error.response.data);
    }
}
```

### Python

```python
import requests
import json

def create_task():
    task_data = {
        "description": "Analyze sales data",
        "type": "data_analysis",
        "priority": 2,
        "required_capabilities": [
            {
                "name": "data_analysis",
                "min_proficiency": 0.7
            }
        ]
    }

    try:
        response = requests.post(
            'http://localhost:3001/api/tasks',
            json=task_data,
            headers={'Content-Type': 'application/json'}
        )

        if response.status_code == 201:
            print('Task created:', response.json())
        else:
            print('Error:', response.json())

    except requests.exceptions.RequestException as e:
        print('Request failed:', e)

def websocket_client():
    import websocket

    def on_message(ws, message):
        data = json.loads(message)
        print(f'Received: {data}')

    ws = websocket.WebSocketApp(
        'ws://localhost:3001/ws',
        on_message=on_message
    )

    ws.run_forever()
```

### WebSocket JavaScript Client

```javascript
class HiveWebSocketClient {
    constructor(url = 'ws://localhost:3001/ws') {
        this.url = url;
        this.ws = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
    }

    connect() {
        try {
            this.ws = new WebSocket(this.url);

            this.ws.onopen = () => {
                console.log('Connected to Hive WebSocket');
                this.reconnectAttempts = 0;
            };

            this.ws.onmessage = (event) => {
                const message = JSON.parse(event.data);
                this.handleMessage(message);
            };

            this.ws.onclose = () => {
                console.log('WebSocket connection closed');
                this.attemptReconnect();
            };

            this.ws.onerror = (error) => {
                console.error('WebSocket error:', error);
            };

        } catch (error) {
            console.error('Failed to connect:', error);
        }
    }

    handleMessage(message) {
        switch (message.type) {
            case 'agent_created':
                console.log('New agent:', message.data.agent);
                break;
            case 'task_completed':
                console.log('Task completed:', message.data);
                break;
            case 'system_alert':
                console.warn('Alert:', message.data);
                break;
            default:
                console.log('Unknown message type:', message.type);
        }
    }

    attemptReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);

            setTimeout(() => {
                console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
                this.connect();
            }, delay);
        }
    }

    disconnect() {
        if (this.ws) {
            this.ws.close();
        }
    }
}

// Usage
const client = new HiveWebSocketClient();
client.connect();
```

## Versioning

The API follows semantic versioning:

- **v1.0**: Initial release with basic agent and task management
- **v2.0**: Enhanced with neural processing, verification systems, and advanced monitoring

## Support

For API support or questions:

- Check the health endpoint: `GET /health`
- Review system logs for detailed error information
- Monitor metrics: `GET /metrics`
- WebSocket events provide real-time system updates
