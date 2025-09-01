# API Documentation

This document provides comprehensive documentation for the AI Orchestrator Hub APIs, including REST endpoints, WebSocket real-time communication, and MCP (Model Context Protocol) integration.

## Table of Contents

- [REST API](#rest-api)
- [WebSocket API](#websocket-api)
- [MCP Integration](#mcp-integration)
- [Data Models](#data-models)
- [Error Handling](#error-handling)
- [Authentication](#authentication)
- [Rate Limiting](#rate-limiting)

## REST API

Base URL: `http://localhost:3001`

### System Endpoints

#### Health Check

```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00Z",
  "response_time_ms": 15,
  "version": "2.0.0",
  "components": {
    "hive_coordinator": {
      "status": "healthy",
      "total_agents": 25,
      "active_agents": 18,
      "completed_tasks": 142
    },
    "resource_manager": {
      "status": "healthy",
      "memory_usage_percent": 65.2,
      "cpu_usage_percent": 45.8
    },
    "metrics_collector": {
      "status": "healthy",
      "response_time_ms": 12.5
    }
  }
}
```

#### Metrics Endpoint

```http
GET /metrics
```

**Response:**
```json
{
  "current_metrics": {
    "performance": {
      "average_response_time_ms": 45.2,
      "requests_per_second": 12.5,
      "error_rate_per_minute": 0.02
    },
    "resources": {
      "memory_usage_mb": 512.3,
      "cpu_usage_percent": 35.8,
      "disk_usage_percent": 45.2
    }
  },
  "trends": {
    "cpu_trend": "stable",
    "memory_trend": "increasing",
    "task_completion_trend": "improving"
  },
  "collection_timestamp": "2024-01-01T12:00:00Z"
}
```

#### Root Endpoint

```http
GET /
```

**Response:**
```json
{
  "message": "🐝 AI Orchestrator Hub API v2.0 - CPU-native, GPU-optional",
  "version": "2.0.0",
  "endpoints": {
    "health": "/health",
    "metrics": "/metrics",
    "api": {
      "agents": "/api/agents",
      "tasks": "/api/tasks",
      "hive": "/api/hive/status",
      "resources": "/api/resources"
    },
    "websocket": "ws://localhost:3001/ws",
    "mcp": "http://localhost:3002"
  }
}
```

### API Endpoints

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

#### Get Resource Information

```http
GET /api/resources
```

**Response:**
```json
{
  "system_resources": {
    "cpu_usage": 45.8,
    "memory_usage": 65.2,
    "available_memory_mb": 2048.5,
    "cpu_cores": 8,
    "disk_usage": 45.2
  },
  "hive_resources": {
    "active_agents": 18,
    "idle_agents": 7,
    "total_memory_allocated": 256.3,
    "average_agent_memory": 14.2
  },
  "performance_indicators": {
    "memory_efficiency": 0.85,
    "cpu_efficiency": 0.78,
    "resource_utilization_score": 0.82
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

#### Agent Failed

```json
{
  "type": "agent_failed",
  "data": {
    "agent_id": "uuid",
    "reason": "resource_exhaustion",
    "recovery_attempted": true,
    "timestamp": "2024-01-01T12:00:00Z"
  }
}
```

#### Task Failed

```json
{
  "type": "task_failed",
  "data": {
    "task_id": "uuid",
    "reason": "capability_mismatch",
    "retry_count": 2,
    "max_retries": 3,
    "timestamp": "2024-01-01T12:00:00Z"
  }
}
```

#### Alert Triggered

```json
{
  "type": "alert_triggered",
  "data": {
    "alert_id": "uuid",
    "level": "warning",
    "title": "High CPU Usage",
    "message": "CPU usage exceeded 80% threshold",
    "confidence": 0.95,
    "predicted": true,
    "timestamp": "2024-01-01T12:00:00Z"
  }
}
```

#### Resource Update

```json
{
  "type": "resource_update",
  "data": {
    "cpu_usage": 75.2,
    "memory_usage": 68.5,
    "active_connections": 45,
    "timestamp": "2024-01-01T12:00:00Z"
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
    "details": {
      "available_memory": 512,
      "required_memory": 1024
    },
    "timestamp": "2024-01-01T12:00:00Z"
  }
}
```

## MCP Integration

The system implements Model Context Protocol (MCP) 1.0 for external tool integration.

### Available Tools

#### create_swarm_agent

Create a new agent with specified capabilities and configuration.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "name": {"type": "string"},
    "agent_type": {"type": "string", "enum": ["Worker", "Coordinator", "Specialist", "Learner"]},
    "specialization": {"type": "string"},
    "initial_energy": {"type": "number", "minimum": 0, "maximum": 100, "default": 100},
    "capabilities": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": {"type": "string"},
          "proficiency": {"type": "number", "minimum": 0, "maximum": 1},
          "learning_rate": {"type": "number", "minimum": 0, "maximum": 1, "default": 0.1}
        },
        "required": ["name", "proficiency"]
      }
    },
    "position": {
      "type": "array",
      "items": {"type": "number"},
      "minItems": 2,
      "maxItems": 2
    }
  },
  "required": ["name", "agent_type"]
}
```

**Output Schema:**
```json
{
  "type": "object",
  "properties": {
    "agent_id": {"type": "string"},
    "status": {"type": "string", "enum": ["created", "queued", "failed"]},
    "message": {"type": "string"}
  }
}
```

#### assign_swarm_task

Assign a task to the swarm with intelligent agent matching.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "description": {"type": "string"},
    "priority": {"type": "string", "enum": ["Low", "Medium", "High", "Critical"]},
    "estimated_duration": {"type": "number", "minimum": 1},
    "required_capabilities": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": {"type": "string"},
          "min_proficiency": {"type": "number", "minimum": 0, "maximum": 1}
        },
        "required": ["name"]
      }
    },
    "metadata": {"type": "object"}
  },
  "required": ["description"]
}
```

**Output Schema:**
```json
{
  "type": "object",
  "properties": {
    "task_id": {"type": "string"},
    "assigned_agents": {"type": "array", "items": {"type": "string"}},
    "estimated_completion": {"type": "string"},
    "status": {"type": "string"}
  }
}
```

#### get_swarm_status

Retrieve comprehensive hive status and metrics.

**Output Schema:**
```json
{
  "type": "object",
  "properties": {
    "hive_status": {"$ref": "#/definitions/HiveStatus"},
    "metrics": {"$ref": "#/definitions/HiveMetrics"},
    "active_alerts": {"type": "array", "items": {"$ref": "#/definitions/Alert"}},
    "timestamp": {"type": "string", "format": "date-time"}
  }
}
```

#### analyze_with_nlp

Perform advanced NLP analysis using neural networks.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "text": {"type": "string"},
    "analysis_type": {"type": "string", "enum": ["sentiment", "keywords", "patterns", "entities", "all"]},
    "model": {"type": "string", "enum": ["basic", "advanced"], "default": "basic"}
  },
  "required": ["text"]
}
```

**Output Schema:**
```json
{
  "type": "object",
  "properties": {
    "sentiment": {"type": "number", "minimum": -1, "maximum": 1},
    "keywords": {"type": "array", "items": {"type": "string"}},
    "patterns": {"type": "array", "items": {"type": "object"}},
    "entities": {"type": "array", "items": {"type": "object"}},
    "confidence": {"type": "number", "minimum": 0, "maximum": 1}
  }
}
```

#### get_performance_metrics

Access detailed performance analytics and trends.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "time_range": {"type": "string", "enum": ["1h", "24h", "7d", "30d"], "default": "24h"},
    "metrics": {"type": "array", "items": {"type": "string"}}
  }
}
```

#### manage_resources

Monitor and optimize system resource usage.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "action": {"type": "string", "enum": ["monitor", "optimize", "scale"]},
    "resource_type": {"type": "string", "enum": ["cpu", "memory", "disk", "network"]},
    "target_value": {"type": "number"}
  },
  "required": ["action"]
}
```

#### coordinate_agents

Trigger swarm coordination and formation optimization.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "coordination_type": {"type": "string", "enum": ["formation", "task_distribution", "resource_sharing"]},
    "target_agents": {"type": "array", "items": {"type": "string"}},
    "parameters": {"type": "object"}
  },
  "required": ["coordination_type"]
}
```

### Resources

#### hive://status

Live system status and metrics with real-time updates.

**Schema:**
```json
{
  "type": "object",
  "properties": {
    "status": {"$ref": "#/definitions/HiveStatus"},
    "metrics": {"$ref": "#/definitions/HiveMetrics"},
    "alerts": {"type": "array", "items": {"$ref": "#/definitions/Alert"}},
    "last_updated": {"type": "string", "format": "date-time"}
  }
}
```

#### hive://agents

Current agent information and states with capabilities.

**Schema:**
```json
{
  "type": "object",
  "properties": {
    "agents": {
      "type": "array",
      "items": {"$ref": "#/definitions/Agent"}
    },
    "total_count": {"type": "integer"},
    "active_count": {"type": "integer"},
    "by_type": {"type": "object"}
  }
}
```

#### hive://tasks

Task queue and execution status with progress tracking.

**Schema:**
```json
{
  "type": "object",
  "properties": {
    "tasks": {
      "type": "array",
      "items": {"$ref": "#/definitions/Task"}
    },
    "queue_length": {"type": "integer"},
    "processing_count": {"type": "integer"},
    "completed_today": {"type": "integer"}
  }
}
```

#### hive://metrics

Performance metrics and alerting data.

**Schema:**
```json
{
  "type": "object",
  "properties": {
    "performance": {"type": "object"},
    "resources": {"type": "object"},
    "neural": {"type": "object"},
    "alerts": {"type": "array"},
    "trends": {"type": "object"}
  }
}
```

#### hive://resources

System resource utilization and optimization data.

**Schema:**
```json
{
  "type": "object",
  "properties": {
    "system": {"type": "object"},
    "hive": {"type": "object"},
    "optimization": {"type": "object"},
    "recommendations": {"type": "array"}
  }
}
```

## Data Models

### Agent

```typescript
interface Agent {
  id: string;
  name: string;
  type: 'Worker' | 'Coordinator' | 'Specialist' | 'Learner';
  state: 'Idle' | 'Working' | 'Learning' | 'Communicating' | 'Failed' | 'Recovering';
  capabilities: AgentCapability[];
  position: [number, number];
  energy: number;
  max_energy: number;
  created_at: string;
  last_active: string;
  experience_count: number;
  social_connections: number;
  learning_progress: number;
  failure_count: number;
  recovery_attempts: number;
  specialization?: string;
  metadata?: Record<string, any>;
}
```

### AgentCapability

```typescript
interface AgentCapability {
  name: string;
  proficiency: number; // 0.0 to 1.0
  learning_rate: number; // 0.0 to 1.0
  experience_points: number;
  last_used?: string;
  success_rate: number;
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
  started_at?: string;
  completed_at?: string;
  estimated_duration?: number;
  actual_duration?: number;
  progress: number; // 0.0 to 1.0
  result?: string;
  error_message?: string;
  retry_count: number;
  max_retries: number;
  metadata?: Record<string, any>;
}
```

### TaskRequiredCapability

```typescript
interface TaskRequiredCapability {
  name: string;
  min_proficiency: number; // 0.0 to 1.0
  preferred_proficiency?: number;
  mandatory: boolean;
}
```

### Alert

```typescript
interface Alert {
  id: string;
  level: 'Info' | 'Warning' | 'Critical';
  title: string;
  message: string;
  source: string;
  timestamp: string;
  acknowledged: boolean;
  resolved: boolean;
  confidence?: number; // For intelligent alerts
  predicted?: boolean; // For predictive alerts
}
```

### HiveMetrics

```typescript
interface HiveMetrics {
  total_agents: number;
  active_agents: number;
  idle_agents: number;
  failed_agents: number;
  completed_tasks: number;
  failed_tasks: number;
  pending_tasks: number;
  average_performance: number;
  swarm_cohesion: number;
  learning_progress: number;
  average_response_time: number;
  tasks_per_second: number;
  success_rate: number;
  resource_utilization: number;
}
```

### HiveStatus

```typescript
interface HiveStatus {
  id: string;
  created_at: string;
  last_update: string;
  version: string;
  status: 'Initializing' | 'Running' | 'Degraded' | 'Failed';
  metrics: HiveMetrics;
  swarm_center: [number, number];
  total_energy: number;
  active_alerts: number;
  system_health: 'Healthy' | 'Warning' | 'Critical';
}
```

### SystemResources

```typescript
interface SystemResources {
  cpu_usage_percent: number;
  memory_usage_percent: number;
  available_memory_mb: number;
  cpu_cores: number;
  disk_usage_percent: number;
  network_throughput: number;
  active_connections: number;
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