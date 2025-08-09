# API Documentation

This document provides comprehensive documentation for the Multiagent Hive System APIs.

## Table of Contents

- [REST API](#rest-api)
- [WebSocket API](#websocket-api)
- [MCP Integration](#mcp-integration)
- [Data Models](#data-models)
- [Error Handling](#error-handling)
- [Authentication](#authentication)
- [Rate Limiting](#rate-limiting)

## REST API

Base URL: `http://localhost:3001/api`

### Agents API

#### List All Agents

```http
GET /api/agents
```

**Response:**
```json
{
  "agents": [
    {
      "id": "uuid",
      "name": "Agent Name",
      "type": "Worker",
      "state": "Idle",
      "capabilities": [
        {
          "name": "data_processing",
          "proficiency": 0.85,
          "learning_rate": 0.1
        }
      ],
      "position": [10.5, 20.3],
      "energy": 95.5,
      "created_at": "2024-01-01T00:00:00Z",
      "last_active": "2024-01-01T12:00:00Z"
    }
  ]
}
```

#### Create Agent

```http
POST /api/agents
Content-Type: application/json

{
  "name": "New Agent",
  "agent_type": "Worker",
  "capabilities": [
    {
      "name": "data_processing",
      "proficiency": 0.7,
      "learning_rate": 0.15
    }
  ]
}
```

**Response:**
```json
{
  "id": "uuid",
  "name": "New Agent",
  "type": "Worker",
  "state": "Idle",
  "capabilities": [...],
  "position": [0.0, 0.0],
  "energy": 100.0,
  "created_at": "2024-01-01T00:00:00Z",
  "last_active": "2024-01-01T00:00:00Z"
}
```

#### Get Agent Details

```http
GET /api/agents/{agent_id}
```

#### Update Agent

```http
PUT /api/agents/{agent_id}
Content-Type: application/json

{
  "name": "Updated Name",
  "capabilities": [...]
}
```

#### Delete Agent

```http
DELETE /api/agents/{agent_id}
```

### Tasks API

#### List All Tasks

```http
GET /api/tasks
```

**Query Parameters:**
- `status` (optional): Filter by task status (`pending`, `assigned`, `in_progress`, `completed`, `failed`, `cancelled`)
- `priority` (optional): Filter by priority (`low`, `medium`, `high`, `critical`)
- `limit` (optional): Maximum number of tasks to return (default: 100)
- `offset` (optional): Number of tasks to skip (default: 0)

**Response:**
```json
{
  "tasks": [
    {
      "id": "uuid",
      "description": "Process data batch",
      "priority": "High",
      "status": "InProgress",
      "required_capabilities": [
        {
          "name": "data_processing",
          "min_proficiency": 0.8
        }
      ],
      "assigned_agents": ["agent_uuid"],
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T12:00:00Z",
      "estimated_duration": 300,
      "progress": 0.65
    }
  ],
  "total": 42,
  "has_more": true
}
```

#### Create Task

```http
POST /api/tasks
Content-Type: application/json

{
  "description": "New task description",
  "priority": "Medium",
  "required_capabilities": [
    {
      "name": "data_processing",
      "min_proficiency": 0.7
    }
  ],
  "estimated_duration": 600
}
```

#### Get Task Details

```http
GET /api/tasks/{task_id}
```

#### Update Task

```http
PUT /api/tasks/{task_id}
Content-Type: application/json

{
  "description": "Updated description",
  "priority": "High"
}
```

#### Cancel Task

```http
DELETE /api/tasks/{task_id}
```

### Hive Status API

#### Get Hive Status

```http
GET /api/hive/status
```

**Response:**
```json
{
  "id": "uuid",
  "created_at": "2024-01-01T00:00:00Z",
  "last_update": "2024-01-01T12:00:00Z",
  "metrics": {
    "total_agents": 25,
    "active_agents": 18,
    "completed_tasks": 142,
    "failed_tasks": 3,
    "average_performance": 0.87,
    "swarm_cohesion": 0.92,
    "learning_progress": 0.75
  },
  "swarm_center": [15.2, 23.8],
  "total_energy": 2340.5
}
```

#### Get Detailed Metrics

```http
GET /api/hive/metrics
```

**Response:**
```json
{
  "performance_metrics": {
    "tasks_per_second": 12.5,
    "average_task_duration": 245.3,
    "success_rate": 0.94,
    "error_rate": 0.06
  },
  "resource_metrics": {
    "memory_usage": 512.3,
    "cpu_usage": 0.35,
    "network_throughput": 1024.5
  },
  "neural_metrics": {
    "nlp_processing_time": 15.2,
    "learning_iterations": 1250,
    "pattern_recognition_accuracy": 0.89
  }
}
```

#### Reset Hive (Development Only)

```http
POST /api/hive/reset
```

## WebSocket API

Connect to: `ws://localhost:3001/ws`

### Connection

```javascript
const ws = new WebSocket('ws://localhost:3001/ws');

ws.onopen = () => {
  console.log('Connected to hive');
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  handleMessage(data);
};
```

### Message Types

#### Hive Status Updates

```json
{
  "type": "hive_status",
  "data": {
    "id": "uuid",
    "metrics": {...},
    "swarm_center": [x, y],
    "total_energy": 2340.5,
    "last_update": "2024-01-01T12:00:00Z"
  }
}
```

#### Agent Updates

```json
{
  "type": "agents_update",
  "data": {
    "agents": [...],
    "total_count": 25,
    "active_count": 18
  }
}
```

#### Task Updates

```json
{
  "type": "task_update",
  "data": {
    "task_id": "uuid",
    "status": "completed",
    "progress": 1.0,
    "result": "Task completed successfully"
  }
}
```

#### Metrics Updates

```json
{
  "type": "metrics_update",
  "data": {
    "timestamp": "2024-01-01T12:00:00Z",
    "performance": {...},
    "resources": {...},
    "neural": {...}
  }
}
```

#### Agent Created

```json
{
  "type": "agent_created",
  "data": {
    "agent": {...},
    "message": "New agent created successfully"
  }
}
```

#### Task Created

```json
{
  "type": "task_created",
  "data": {
    "task": {...},
    "message": "New task added to queue"
  }
}
```

#### Error Notifications

```json
{
  "type": "error",
  "data": {
    "error_code": "AGENT_CREATION_FAILED",
    "message": "Failed to create agent: insufficient resources",
    "details": {...}
  }
}
```

## MCP Integration

The system implements Model Context Protocol (MCP) 1.0 for external tool integration.

### Available Tools

#### create_swarm_agent

Create a new agent with specified capabilities.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "name": {"type": "string"},
    "agent_type": {"type": "string", "enum": ["Worker", "Coordinator", "Specialist", "Learner"]},
    "specialization": {"type": "string"},
    "capabilities": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": {"type": "string"},
          "proficiency": {"type": "number", "minimum": 0, "maximum": 1},
          "learning_rate": {"type": "number", "minimum": 0, "maximum": 1}
        }
      }
    }
  },
  "required": ["name", "agent_type"]
}
```

#### assign_swarm_task

Assign a task to the swarm.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "description": {"type": "string"},
    "priority": {"type": "string", "enum": ["Low", "Medium", "High", "Critical"]},
    "required_capabilities": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": {"type": "string"},
          "min_proficiency": {"type": "number", "minimum": 0, "maximum": 1}
        }
      }
    }
  },
  "required": ["description"]
}
```

#### get_swarm_status

Retrieve current hive status and metrics.

#### analyze_with_nlp

Perform NLP analysis on provided text.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "text": {"type": "string"},
    "analysis_type": {"type": "string", "enum": ["sentiment", "keywords", "patterns", "all"]}
  },
  "required": ["text"]
}
```

#### coordinate_agents

Coordinate agent behaviors and interactions.

### Resources

#### hive://status

Live system status and metrics.

#### hive://agents

Current agent information and states.

#### hive://tasks

Task queue and execution status.

## Data Models

### Agent

```typescript
interface Agent {
  id: string;
  name: string;
  type: 'Worker' | 'Coordinator' | 'Specialist' | 'Learner';
  state: 'Idle' | 'Working' | 'Learning' | 'Communicating' | 'Failed';
  capabilities: AgentCapability[];
  position: [number, number];
  energy: number;
  created_at: string;
  last_active: string;
  experience_count: number;
  social_connections: number;
}
```

### Task

```typescript
interface Task {
  id: string;
  description: string;
  priority: 'Low' | 'Medium' | 'High' | 'Critical';
  status: 'Pending' | 'Assigned' | 'InProgress' | 'Completed' | 'Failed' | 'Cancelled';
  required_capabilities: TaskRequiredCapability[];
  assigned_agents: string[];
  created_at: string;
  updated_at: string;
  estimated_duration?: number;
  progress: number;
  result?: string;
}
```

### HiveMetrics

```typescript
interface HiveMetrics {
  total_agents: number;
  active_agents: number;
  completed_tasks: number;
  failed_tasks: number;
  average_performance: number;
  swarm_cohesion: number;
  learning_progress: number;
}
```

## Error Handling

### HTTP Status Codes

- `200 OK`: Request successful
- `201 Created`: Resource created successfully
- `400 Bad Request`: Invalid request data
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource conflict (e.g., duplicate name)
- `422 Unprocessable Entity`: Validation errors
- `500 Internal Server Error`: Server error

### Error Response Format

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid agent configuration",
    "details": {
      "field": "capabilities",
      "reason": "Proficiency must be between 0 and 1"
    },
    "timestamp": "2024-01-01T12:00:00Z"
  }
}
```

### Common Error Codes

- `VALIDATION_ERROR`: Input validation failed
- `RESOURCE_NOT_FOUND`: Requested resource doesn't exist
- `INSUFFICIENT_RESOURCES`: System resources exhausted
- `AGENT_CREATION_FAILED`: Failed to create agent
- `TASK_ASSIGNMENT_FAILED`: Failed to assign task
- `NEURAL_PROCESSING_ERROR`: Neural processing failure

## Authentication

Currently, the system operates without authentication for development purposes. In production environments, implement:

- API key authentication for REST endpoints
- Token-based authentication for WebSocket connections
- Role-based access control (RBAC)
- Rate limiting per user/API key

## Rate Limiting

Default rate limits (configurable):

- REST API: 100 requests per minute per IP
- WebSocket: 10 connections per IP
- Agent creation: 10 agents per minute
- Task creation: 50 tasks per minute

Rate limit headers:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
```

## SDK Examples

### JavaScript/TypeScript

```typescript
import { HiveClient } from '@multiagent-hive/client';

const client = new HiveClient('http://localhost:3001');

// Create an agent
const agent = await client.agents.create({
  name: 'Data Processor',
  agent_type: 'Worker',
  capabilities: [
    { name: 'data_processing', proficiency: 0.8, learning_rate: 0.1 }
  ]
});

// Create a task
const task = await client.tasks.create({
  description: 'Process customer data',
  priority: 'High',
  required_capabilities: [
    { name: 'data_processing', min_proficiency: 0.7 }
  ]
});

// Monitor hive status
client.onStatusUpdate((status) => {
  console.log('Hive status:', status);
});
```

### Python

```python
from multiagent_hive import HiveClient

client = HiveClient('http://localhost:3001')

# Create an agent
agent = client.agents.create(
    name='Data Processor',
    agent_type='Worker',
    capabilities=[
        {'name': 'data_processing', 'proficiency': 0.8, 'learning_rate': 0.1}
    ]
)

# Create a task
task = client.tasks.create(
    description='Process customer data',
    priority='High',
    required_capabilities=[
        {'name': 'data_processing', 'min_proficiency': 0.7}
    ]
)
```

For more examples and detailed usage, see the [examples](../examples/) directory.