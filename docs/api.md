# AI Orchestrator Hub API Documentation

This document provides comprehensive API documentation for the AI Orchestrator Hub backend, including REST endpoints, WebSocket events, and MCP integration.

## Table of Contents

- [Overview](#overview)
- [Authentication](#authentication)
- [REST API Endpoints](#rest-api-endpoints)
- [WebSocket Events](#websocket-events)
- [MCP Integration](#mcp-integration)
- [Error Handling](#error-handling)
- [Rate Limiting](#rate-limiting)
- [Examples](#examples)

## Overview

The AI Orchestrator Hub provides a comprehensive API for managing multi-agent systems, including:

- **Agent Management**: Create and list agents in the multi-agent system
- **Task Management**: Create and list tasks with intelligent assignment
- **Hive Monitoring**: Real-time system status and metrics
- **Resource Management**: System resource monitoring and optimization
- **MCP Integration**: Model Context Protocol for external AI tool integration

### Base URL
```
http://localhost:3001
```

### Content Type
All requests should use `application/json` content type.

## Authentication

### API Key Authentication

Include the API key in the request header:
```
X-API-Key: your-api-key-here
```

**Note**: JWT authentication is not currently implemented in the REST API.

## REST API Endpoints

### Core Endpoints

#### Root Endpoint
```http
GET /
```

Returns basic API information and version details.

**Parameters:** None

**Authentication:** Not required

**Response (Success - 200):**
```json
{
  "message": "🐝 Multiagent Hive System API v2.0 - CPU-native, GPU-optional",
  "version": "2.0.0",
  "status": "operational"
}
```

**Example:**
```bash
curl -X GET http://localhost:3001/ \
  -H "Accept: application/json"
```

#### Health Check
```http
GET /health
```

Performs a comprehensive health check of the AI Orchestrator Hub system, including all critical components and services. This endpoint is crucial for monitoring system health and can be used for load balancer health checks.

**Parameters:** None

**Authentication:** Optional (API key recommended for production)

**Rate Limiting:** Not rate limited (health checks should be frequent)

**Response (Success - 200):**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T00:00:00Z",
  "response_time_ms": 15,
  "version": "2.0.0",
  "request_id": "req-health-12345",
  "components": {
    "hive_coordinator": {
      "status": "healthy",
      "total_agents": 15,
      "active_agents": 12,
      "completed_tasks": 45,
      "average_performance": 0.87,
      "last_check": "2024-01-01T00:00:00Z"
    },
    "resource_manager": {
      "status": "healthy",
      "memory_usage_percent": 45.2,
      "cpu_usage_percent": 32.1,
      "available_memory_mb": 8192,
      "cpu_cores": 8,
      "disk_usage_percent": 25.0
    },
    "metrics_collector": {
      "status": "healthy",
      "response_time_ms": 2.3,
      "requests_per_second": 15.7,
      "error_rate": 0.001,
      "uptime_seconds": 3600
    },
    "intelligent_alerting": {
      "status": "healthy",
      "active_rules": 8,
      "system_operational": true,
      "last_alert_check": "2024-01-01T00:00:00Z"
    }
  },
  "system_info": {
    "cpu_native": true,
    "gpu_optional": true,
    "phase_2_active": true,
    "swarm_cohesion": 0.85,
    "learning_progress": 0.72,
    "architecture": "x86_64",
    "os": "linux"
  }
}
```

**Response (Service Unavailable - 503):**
```json
{
  "status": "unhealthy",
  "timestamp": "2024-01-01T00:00:00Z",
  "response_time_ms": 15,
  "version": "2.0.0",
  "request_id": "req-health-12345",
  "components": {
    "hive_coordinator": {
      "status": "unhealthy",
      "total_agents": 0,
      "active_agents": 0,
      "completed_tasks": 0,
      "average_performance": 0.0,
      "error": "No agents available"
    }
  },
  "error": "System health check failed",
  "recommendation": "Check system logs and restart services if necessary"
}
```

**Response Fields:**
- `status`: Overall system health ("healthy" or "unhealthy")
- `timestamp`: ISO 8601 timestamp of the health check
- `response_time_ms`: Time taken to perform the health check
- `version`: API version
- `request_id`: Unique identifier for this health check request
- `components`: Detailed status of each system component
- `system_info`: System configuration and capabilities

**Common Issues:**
- **503 Unhealthy**: Usually indicates critical system components are down
- **Slow Response**: May indicate performance issues or high system load
- **Missing Components**: Some components may be temporarily unavailable

**Examples:**
```bash
# Basic health check
curl -X GET http://localhost:3001/health \
  -H "Accept: application/json"

# Health check with API key (recommended for production)
curl -X GET http://localhost:3001/health \
  -H "Accept: application/json" \
  -H "X-API-Key: your-api-key-here"

# Health check with verbose output
curl -X GET http://localhost:3001/health \
  -H "Accept: application/json" \
  -v
```

#### System Metrics
```http
GET /metrics
```

Returns comprehensive real-time system metrics including performance indicators, resource usage, agent statistics, and operational trends. This endpoint provides detailed insights into system health and performance.

**Parameters:** None

**Authentication:** API key required

**Rate Limiting:** 100 requests per minute per API key

**Response (Success - 200):**
```json
{
  "current_metrics": {
    "performance": {
      "average_response_time_ms": 245.0,
      "requests_per_second": 15.7,
      "error_rate_per_minute": 0.001,
      "throughput_mb_per_sec": 45.2,
      "concurrent_connections": 23
    },
    "error_metrics": {
      "error_rate_per_minute": 0.001,
      "total_errors": 5,
      "error_types": {
        "validation_errors": 2,
        "timeout_errors": 1,
        "internal_errors": 2
      },
      "last_error_timestamp": "2024-01-01T00:05:00Z"
    },
    "agent_metrics": {
      "total_agents": 15,
      "active_agents": 12,
      "idle_agents": 2,
      "failed_agents": 1,
      "average_agent_performance": 0.87,
      "agent_utilization_percent": 78.5,
      "average_agent_energy": 85.0,
      "agent_types_distribution": {
        "Worker": 8,
        "Coordinator": 3,
        "Specialist": 2,
        "Learner": 2
      },
      "individual_agent_metrics": {
        "agent-123": {
          "performance_score": 0.92,
          "tasks_completed": 45,
          "energy_level": 88.0,
          "last_activity": "2024-01-01T00:05:00Z"
        }
      }
    },
    "task_metrics": {
      "total_tasks_submitted": 45,
      "total_tasks_completed": 32,
      "total_tasks_failed": 5,
      "tasks_in_queue": 8,
      "average_task_duration_ms": 2450.0,
      "task_success_rate": 87.0,
      "average_queue_wait_time_ms": 125.0,
      "tasks_by_priority": {
        "low": 10,
        "medium": 20,
        "high": 12,
        "critical": 3
      },
      "tasks_by_status": {
        "pending": 8,
        "running": 5,
        "completed": 32,
        "failed": 5
      }
    },
    "resource_metrics": {
      "cpu_usage_percent": 45.2,
      "memory_usage_percent": 60.8,
      "disk_usage_percent": 25.0,
      "network_bandwidth_mbps": 1000,
      "gpu_usage_percent": 35.8,
      "available_memory_mb": 4096,
      "swap_usage_percent": 10.5
    }
  },
  "trends": {
    "cpu_trend": "stable",
    "memory_trend": "increasing",
    "task_completion_trend": "improving",
    "error_rate_trend": "decreasing",
    "performance_trend": "stable",
    "trend_period_hours": 24
  },
  "anomalies": [
    {
      "type": "high_memory_usage",
      "severity": "warning",
      "description": "Memory usage above 80% for 15 minutes",
      "timestamp": "2024-01-01T00:10:00Z",
      "current_value": 85.2,
      "threshold": 80.0
    }
  ],
  "collection_timestamp": "2024-01-01T00:15:00Z",
  "collection_duration_ms": 45,
  "next_collection_in_ms": 15000
}
```

**Response Fields:**
- `current_metrics`: Real-time system metrics
  - `performance`: API performance indicators
  - `error_metrics`: Error rates and types
  - `agent_metrics`: Agent system statistics
  - `task_metrics`: Task processing statistics
  - `resource_metrics`: System resource usage
- `trends`: Performance trends over time
- `anomalies`: Detected system anomalies
- `collection_timestamp`: When metrics were collected
- `next_collection_in_ms`: Time until next collection

**Rate Limit Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 99
X-RateLimit-Reset: 1640995200
X-RateLimit-Reset-After: 45
```

**Examples:**
```bash
# Get current system metrics
curl -X GET http://localhost:3001/metrics \
  -H "Accept: application/json" \
  -H "X-API-Key: your-api-key-here"

# Get metrics with detailed output
curl -X GET http://localhost:3001/metrics \
  -H "Accept: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -v

# Monitor metrics in a loop (with rate limiting consideration)
while true; do
  curl -s -X GET http://localhost:3001/metrics \
    -H "X-API-Key: your-api-key-here" | jq '.current_metrics.performance'
  sleep 60
done
```

**Use Cases:**
- System monitoring and alerting
- Performance analysis and optimization
- Capacity planning
- Troubleshooting system issues
- Real-time dashboard updates

### Agent Management

#### List Agents
```http
GET /api/agents
```

Retrieves comprehensive information about all agents in the multi-agent system, including their status, capabilities, performance metrics, and operational statistics.

**Parameters:** None

**Authentication:** API key required

**Rate Limiting:** 100 requests per minute per API key

**Response (Success - 200):**
```json
{
  "agents": [
    {
      "id": "agent-123",
      "name": "WorkerAgent-1",
      "type": "worker",
      "status": "Active",
      "capabilities": [
        {
          "name": "data_processing",
          "proficiency": 0.85,
          "learning_rate": 0.12,
          "experience_points": 1250,
          "last_used": "2024-01-01T00:05:00Z"
        }
      ],
      "position": [10.5, 20.3],
      "energy": 85.0,
      "performance_score": 0.92,
      "tasks_completed": 45,
      "tasks_failed": 2,
      "average_task_duration_ms": 2450,
      "created_at": "2024-01-01T00:00:00Z",
      "last_active": "2024-01-01T00:05:00Z",
      "uptime_seconds": 18000,
      "memory_usage_mb": 256,
      "cpu_usage_percent": 15.2
    }
  ],
  "summary": {
    "total_count": 15,
    "active_count": 12,
    "idle_count": 2,
    "failed_count": 1,
    "by_type": {
      "worker": 8,
      "coordinator": 3,
      "specialist": 2,
      "learner": 2
    },
    "by_status": {
      "active": 12,
      "idle": 2,
      "failed": 1
    },
    "average_performance": 0.87,
    "total_energy": 1250.5,
    "average_energy": 83.4
  },
  "collection_timestamp": "2024-01-01T00:10:00Z",
  "request_id": "req-agents-12345"
}
```

**Response Fields:**
- `agents`: Array of agent objects with detailed information
  - `id`: Unique agent identifier
  - `name`: Human-readable agent name
  - `type`: Agent type (worker, coordinator, specialist, learner)
  - `status`: Current operational status
  - `capabilities`: Array of agent skills and proficiencies
  - `position`: 2D coordinates in swarm space
  - `energy`: Current energy level (0.0-100.0)
  - `performance_score`: Overall performance rating
  - `tasks_completed/failed`: Task execution statistics
  - `created_at/last_active`: Timestamps
- `summary`: Aggregated statistics across all agents

**Examples:**
```bash
# List all agents
curl -X GET http://localhost:3001/api/agents \
  -H "Accept: application/json" \
  -H "X-API-Key: your-api-key-here"

# List agents with detailed output
curl -X GET http://localhost:3001/api/agents \
  -H "Accept: application/json" \
  -H "X-API-Key: your-api-key-here" | jq '.'

# Monitor agent count
curl -s -X GET http://localhost:3001/api/agents \
  -H "X-API-Key: your-api-key-here" | jq '.summary.total_count'
```

**Use Cases:**
- System monitoring and agent health checks
- Capacity planning and resource allocation
- Performance analysis and optimization
- Agent lifecycle management
- Real-time dashboard updates

#### Create Agent
```http
POST /api/agents
```

Creates a new agent in the multi-agent system with specified capabilities, configuration, and initial parameters. The agent will be automatically integrated into the swarm and begin participating in task execution.

**Request Body:**
```json
{
  "name": "DataProcessor-1",
  "agent_type": "worker",
  "capabilities": [
    {
      "name": "data_processing",
      "proficiency": 0.8,
      "learning_rate": 0.1
    }
  ],
  "initial_energy": 100.0,
  "position": [0.0, 0.0]
}
```

**Request Body Parameters:**

**Required Parameters:**
- `name` (string, required): Agent name
  - Must be 1-100 characters long
  - Alphanumeric characters, spaces, hyphens, and underscores only
  - Cannot be reserved words: "admin", "root", "system", "null", "undefined", "test"
  - Must be unique within the system
  - Example: "DataProcessor-1", "ML-Specialist", "TaskCoordinator"

- `agent_type` (string, required): Type of agent to create
  - Valid values: `"worker"`, `"coordinator"`, `"learner"`, `"specialist:<specialization>"`
  - `"worker"`: General-purpose agent for task execution
  - `"coordinator"`: Agent specialized in coordinating other agents
  - `"learner"`: Agent focused on learning and adaptation
  - `"specialist:<type>"`: Specialized agent (e.g., "specialist:machine_learning", "specialist:data_analysis")

**Optional Parameters:**
- `capabilities` (array, optional): Array of capability objects defining agent skills
  - `name` (string, required): Capability name
    - Examples: "data_processing", "machine_learning", "sentiment_analysis", "task_coordination"
  - `proficiency` (number, optional): Initial proficiency level (0.0-1.0)
    - Default: 0.5
    - 0.0 = novice, 1.0 = expert
  - `learning_rate` (number, optional): Learning rate for skill improvement (0.0-1.0)
    - Default: 0.1
    - Higher values = faster learning but potentially less stable

- `initial_energy` (number, optional): Starting energy level (0.0-100.0)
  - Default: 100.0
  - Affects agent performance and task assignment priority

- `position` (array, optional): Initial 2D position in swarm space [x, y]
  - Default: [0.0, 0.0]
  - Used for swarm coordination and spatial organization

**Authentication:** API key required

**Rate Limiting:** 10 requests per minute per API key (agent creation is resource-intensive)

**Content-Type:** `application/json`

**Response (Success - 201):**
```json
{
  "success": true,
  "agent_id": "agent-456",
  "message": "Agent created successfully",
  "agent_details": {
    "id": "agent-456",
    "name": "DataProcessor-1",
    "type": "worker",
    "status": "Initializing",
    "capabilities": [
      {
        "name": "data_processing",
        "proficiency": 0.8,
        "learning_rate": 0.1,
        "experience_points": 0
      }
    ],
    "position": [0.0, 0.0],
    "energy": 100.0,
    "created_at": "2024-01-01T00:10:00Z"
  },
  "request_id": "req-12345",
  "processing_time_ms": 45,
  "estimated_activation_time_ms": 5000
}
```

**Response (Bad Request - 400):**
```json
{
  "success": false,
  "error": "VALIDATION_ERROR",
  "message": "Input validation failed",
  "details": {
    "field": "name",
    "reason": "Agent name cannot be empty"
  },
  "field_errors": [
    {
      "field": "name",
      "message": "Agent name cannot be empty",
      "value": ""
    }
  ],
  "request_id": "req-12345",
  "timestamp": "2024-01-01T00:10:00Z"
}
```

**Response (Conflict - 409):**
```json
{
  "success": false,
  "error": "AGENT_ALREADY_EXISTS",
  "message": "Agent with this name already exists",
  "details": {
    "existing_agent_id": "agent-123",
    "name": "DataProcessor-1"
  },
  "request_id": "req-12345",
  "timestamp": "2024-01-01T00:10:00Z"
}
```

**Response (Rate Limited - 429):**
```json
{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Rate limit exceeded",
  "details": {
    "limit": 10,
    "remaining": 0,
    "reset_after_seconds": 45,
    "retry_after": "2024-01-01T00:10:45Z"
  },
  "request_id": "req-12345",
  "timestamp": "2024-01-01T00:10:00Z"
}
```

**Rate Limit Headers:**
```
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 9
X-RateLimit-Reset: 1640995245
X-RateLimit-Reset-After: 45
```

**Examples:**

```bash
# Create a basic worker agent
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -d '{
    "name": "WorkerAgent-1",
    "agent_type": "worker",
    "capabilities": [
      {
        "name": "data_processing",
        "proficiency": 0.7
      }
    ]
  }'

# Create a specialist agent with advanced configuration
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -d '{
    "name": "ML-Specialist",
    "agent_type": "specialist:machine_learning",
    "capabilities": [
      {
        "name": "machine_learning",
        "proficiency": 0.9,
        "learning_rate": 0.2
      },
      {
        "name": "data_analysis",
        "proficiency": 0.8,
        "learning_rate": 0.15
      }
    ],
    "initial_energy": 95.0,
    "position": [10.5, 20.3]
  }'

# Create a coordinator agent
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -d '{
    "name": "TaskCoordinator",
    "agent_type": "coordinator",
    "capabilities": [
      {
        "name": "task_coordination",
        "proficiency": 0.95,
        "learning_rate": 0.05
      }
    ]
  }'

# Error example - invalid agent name
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -d '{
    "name": "",
    "agent_type": "worker"
  }'
```

**Best Practices:**
1. **Choose descriptive names**: Use names that clearly indicate the agent's purpose
2. **Set appropriate proficiencies**: Start with realistic proficiency levels based on agent capabilities
3. **Configure learning rates**: Higher learning rates for dynamic environments, lower for stable ones
4. **Monitor creation**: Check the response for the agent_id and monitor activation status
5. **Handle rate limits**: Implement exponential backoff for rate limit errors

**Notes:**
- Agent creation is asynchronous - the agent may take a few seconds to fully activate
- Individual agent operations (get by ID, update, delete) are not currently implemented in the REST API
- Use the list agents endpoint to monitor agent status after creation
- Agent management is primarily handled through the MCP interface for advanced operations

### Task Management

#### List Tasks
```http
GET /api/tasks
```

Retrieves comprehensive information about all tasks in the system, including their status, progress, assigned agents, and execution details.

**Parameters:** None

**Authentication:** API key required

**Rate Limiting:** 100 requests per minute per API key

**Response (Success - 200):**
```json
{
  "tasks": [
    {
      "id": "task-123",
      "description": "Process customer feedback data and generate insights",
      "type": "data_processing",
      "priority": "high",
      "status": "InProgress",
      "required_capabilities": [
        {
          "name": "data_processing",
          "minimum_proficiency": 0.8
        }
      ],
      "assigned_agent": "agent-456",
      "assigned_agent_name": "DataProcessor-1",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:05:00Z",
      "started_at": "2024-01-01T00:01:00Z",
      "progress": 0.65,
      "estimated_completion": "2024-01-01T00:12:00Z",
      "actual_completion": null,
      "execution_time_ms": 240000,
      "result": null,
      "error_message": null,
      "retry_count": 0,
      "max_retries": 3,
      "metadata": {
        "input_size": 1024,
        "output_format": "json",
        "data_source": "customer_feedback_db",
        "processing_steps": ["validate", "clean", "analyze", "generate_report"]
      },
      "performance_metrics": {
        "cpu_usage_percent": 75.5,
        "memory_usage_mb": 512,
        "data_processed_mb": 45.8
      }
    }
  ],
  "summary": {
    "total_count": 45,
    "pending_count": 8,
    "in_progress_count": 5,
    "completed_count": 32,
    "failed_count": 0,
    "cancelled_count": 0,
    "by_priority": {
      "low": 10,
      "medium": 20,
      "high": 12,
      "critical": 3
    },
    "by_status": {
      "pending": 8,
      "in_progress": 5,
      "completed": 32,
      "failed": 0,
      "cancelled": 0
    },
    "by_type": {
      "data_processing": 15,
      "analysis": 12,
      "coordination": 8,
      "learning": 5,
      "maintenance": 5
    },
    "average_completion_time_ms": 245000,
    "success_rate_percent": 100.0,
    "total_tasks_processed": 45
  },
  "collection_timestamp": "2024-01-01T00:10:00Z",
  "request_id": "req-tasks-12345"
}
```

**Response Fields:**
- `tasks`: Array of task objects with detailed information
  - `id`: Unique task identifier
  - `description`: Human-readable task description
  - `type`: Task category/type
  - `priority`: Task priority level
  - `status`: Current execution status
  - `required_capabilities`: Skills needed to execute the task
  - `assigned_agent/agent_name`: Agent assigned to the task
  - `progress`: Completion percentage (0.0-1.0)
  - `execution_time_ms`: Time spent executing
  - `result`: Task execution result (null if not completed)
  - `metadata`: Additional task-specific data
  - `performance_metrics`: Resource usage during execution
- `summary`: Aggregated statistics across all tasks

**Task Status Values:**
- `pending`: Task is queued waiting for agent assignment
- `in_progress`: Task is currently being executed
- `completed`: Task finished successfully
- `failed`: Task execution failed
- `cancelled`: Task was cancelled before completion

**Priority Levels:**
- `low`: Non-urgent tasks
- `medium`: Standard priority tasks
- `high`: Important tasks requiring prompt attention
- `critical`: Urgent tasks requiring immediate execution

**Examples:**
```bash
# List all tasks
curl -X GET http://localhost:3001/api/tasks \
  -H "Accept: application/json" \
  -H "X-API-Key: your-api-key-here"

# List tasks with detailed output
curl -X GET http://localhost:3001/api/tasks \
  -H "Accept: application/json" \
  -H "X-API-Key: your-api-key-here" | jq '.'

# Monitor task completion
curl -s -X GET http://localhost:3001/api/tasks \
  -H "X-API-Key: your-api-key-here" | jq '.summary.completed_count'

# Get tasks by status
curl -s -X GET http://localhost:3001/api/tasks \
  -H "X-API-Key: your-api-key-here" | jq '.tasks[] | select(.status == "completed")'
```

**Use Cases:**
- Task monitoring and progress tracking
- System performance analysis
- Agent workload management
- Troubleshooting failed tasks
- Real-time dashboard updates

#### Create Task
```http
POST /api/tasks
```

Creates a new task in the system and automatically assigns it to the most suitable agent based on capabilities, availability, current workload, and performance history. The system uses intelligent matching algorithms to ensure optimal task-agent pairing.

**Request Body:**
```json
{
  "description": "Analyze quarterly sales data and generate performance report",
  "priority": "high",
  "required_capabilities": [
    {
      "name": "data_processing",
      "minimum_proficiency": 0.8
    },
    {
      "name": "statistical_analysis",
      "minimum_proficiency": 0.7
    }
  ],
  "metadata": {
    "input_data_path": "/data/sales_q3.json",
    "output_format": "pdf",
    "data_source": "sales_database",
    "time_range": "Q3_2024"
  },
  "estimated_duration_ms": 300000,
  "max_retries": 3,
  "timeout_ms": 600000
}
```

**Request Body Parameters:**

**Required Parameters:**
- `description` (string, required): Detailed task description
  - Must be 1-1000 characters long
  - Should clearly describe what the task needs to accomplish
  - Examples: "Process customer feedback data and generate insights", "Analyze sales trends for Q3"

**Optional Parameters:**
- `priority` (string, optional): Task priority level
  - Valid values: `"low"`, `"medium"`, `"high"`, `"critical"`
  - Default: `"medium"`
  - Affects queue position and agent assignment priority

- `required_capabilities` (array, optional): Array of capability requirements
  - `name` (string, required): Capability name (e.g., "data_processing", "machine_learning")
  - `minimum_proficiency` (number, optional): Minimum required proficiency (0.0-1.0)
    - Default: 0.5
    - Higher values require more skilled agents

- `metadata` (object, optional): Additional task-specific configuration
  - Can contain any key-value pairs relevant to task execution
  - Examples: file paths, data sources, output formats, processing parameters

- `estimated_duration_ms` (number, optional): Expected execution time in milliseconds
  - Used for scheduling and resource planning
  - Default: 300000 (5 minutes)
  - Helps optimize agent workload distribution

- `max_retries` (number, optional): Maximum number of retry attempts on failure
  - Default: 3
  - Valid range: 0-10
  - Higher values for critical tasks

- `timeout_ms` (number, optional): Maximum allowed execution time
  - Default: 600000 (10 minutes)
  - Task will be cancelled if exceeded
  - Should be longer than estimated_duration_ms

**Authentication:** API key required

**Rate Limiting:** 50 requests per minute per API key (task creation is moderately resource-intensive)

**Content-Type:** `application/json`

**Response (Success - 201):**
```json
{
  "success": true,
  "task_id": "task-789",
  "message": "Task created and queued for execution",
  "task_details": {
    "id": "task-789",
    "description": "Analyze quarterly sales data and generate performance report",
    "priority": "high",
    "status": "pending",
    "queue_position": 3,
    "estimated_assignment_time": "2024-01-01T00:10:30Z",
    "estimated_completion_time": "2024-01-01T00:15:30Z",
    "required_capabilities": [
      {
        "name": "data_processing",
        "minimum_proficiency": 0.8
      }
    ],
    "created_at": "2024-01-01T00:10:00Z"
  },
  "assignment_info": {
    "matching_agents_count": 5,
    "best_match_agent": "agent-456",
    "best_match_score": 0.92,
    "assignment_criteria": ["capability_match", "availability", "performance_history"]
  },
  "request_id": "req-67890",
  "processing_time_ms": 23
}
```

**Response (Bad Request - 400):**
```json
{
  "success": false,
  "error": "VALIDATION_ERROR",
  "message": "Input validation failed",
  "details": {
    "field": "description",
    "reason": "Task description cannot be empty"
  },
  "field_errors": [
    {
      "field": "description",
      "message": "Task description cannot be empty",
      "value": ""
    }
  ],
  "request_id": "req-67890",
  "timestamp": "2024-01-01T00:10:00Z"
}
```

**Response (Service Unavailable - 503):**
```json
{
  "success": false,
  "error": "NO_SUITABLE_AGENTS",
  "message": "No agents available with required capabilities",
  "details": {
    "required_capabilities": ["data_processing"],
    "available_agents_count": 0,
    "suggestion": "Create agents with required capabilities or reduce proficiency requirements"
  },
  "request_id": "req-67890",
  "timestamp": "2024-01-01T00:10:00Z"
}
```

**Response (Rate Limited - 429):**
```json
{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Rate limit exceeded",
  "details": {
    "limit": 50,
    "remaining": 0,
    "reset_after_seconds": 45,
    "retry_after": "2024-01-01T00:10:45Z"
  },
  "request_id": "req-67890",
  "timestamp": "2024-01-01T00:10:00Z"
}
```

**Rate Limit Headers:**
```
X-RateLimit-Limit: 50
X-RateLimit-Remaining: 49
X-RateLimit-Reset: 1640995245
X-RateLimit-Reset-After: 45
```

**Examples:**

```bash
# Create a simple data processing task
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -d '{
    "description": "Process customer feedback data",
    "priority": "medium",
    "required_capabilities": [
      {
        "name": "data_processing",
        "minimum_proficiency": 0.7
      }
    ]
  }'

# Create a complex analysis task with metadata
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -d '{
    "description": "Generate comprehensive sales analytics report",
    "priority": "high",
    "required_capabilities": [
      {
        "name": "data_analysis",
        "minimum_proficiency": 0.9
      },
      {
        "name": "statistical_analysis",
        "minimum_proficiency": 0.8
      }
    ],
    "metadata": {
      "data_source": "sales_db",
      "time_range": "last_quarter",
      "output_format": "pdf_report",
      "include_charts": true
    },
    "estimated_duration_ms": 600000,
    "max_retries": 2
  }'

# Create a critical system maintenance task
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -d '{
    "description": "Perform emergency database backup and validation",
    "priority": "critical",
    "required_capabilities": [
      {
        "name": "database_administration",
        "minimum_proficiency": 0.95
      }
    ],
    "metadata": {
      "backup_type": "full",
      "compression": true,
      "encryption": true
    },
    "estimated_duration_ms": 1800000,
    "timeout_ms": 3600000,
    "max_retries": 1
  }'

# Error example - missing required field
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -d '{
    "priority": "high",
    "required_capabilities": [
      {
        "name": "data_processing"
      }
    ]
  }'
```

**Best Practices:**
1. **Write clear descriptions**: Be specific about what the task should accomplish
2. **Set realistic requirements**: Balance capability needs with agent availability
3. **Use appropriate priorities**: Reserve "critical" for truly urgent tasks
4. **Provide metadata**: Include all necessary configuration data
5. **Monitor task progress**: Use the list tasks endpoint to track execution
6. **Handle assignment failures**: Be prepared for cases where no suitable agents are available

**Notes:**
- Tasks are automatically assigned using intelligent matching algorithms
- Assignment considers agent availability, capability match, current workload, and performance history
- Individual task operations (get by ID, update, delete) are not currently implemented in the REST API
- Use the list tasks endpoint to monitor task status and progress
- Advanced task management is available through the MCP interface

### Hive Management

#### Get Hive Status
```http
GET /api/hive/status
```

Retrieves comprehensive operational status of the entire multi-agent hive system, including swarm intelligence metrics, agent coordination data, and system health indicators.

**Parameters:** None

**Authentication:** API key required

**Rate Limiting:** 100 requests per minute per API key

**Response (Success - 200):**
```json
{
  "success": true,
  "data": {
    "state": "Active",
    "total_agents": 15,
    "active_agents": 12,
    "idle_agents": 2,
    "failed_agents": 1,
    "total_tasks": 45,
    "pending_tasks": 8,
    "in_progress_tasks": 5,
    "completed_tasks": 32,
    "failed_tasks": 5,
    "cancelled_tasks": 0,
    "swarm_center": [45.2, 67.8],
    "swarm_radius": 25.5,
    "swarm_density": 0.75,
    "total_energy": 1250.5,
    "average_energy": 83.4,
    "energy_distribution": {
      "high": 5,
      "medium": 7,
      "low": 3
    },
    "cohesion": 0.85,
    "learning_progress": 0.72,
    "adaptation_rate": 0.15,
    "communication_efficiency": 0.91,
    "task_success_rate": 0.87,
    "average_task_completion_time_ms": 245000,
    "agent_utilization_percent": 78.5,
    "system_health_score": 0.89,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:10:00Z",
    "uptime_seconds": 36000,
    "phase": "Phase 2 - CPU-native, GPU-optional",
    "architecture": "swarm_intelligence_v2",
    "coordination_mode": "adaptive"
  },
  "performance_metrics": {
    "response_time_ms": 15,
    "cpu_usage_percent": 45.2,
    "memory_usage_percent": 60.8,
    "network_bandwidth_mbps": 150
  },
  "alerts": [
    {
      "id": "alert-123",
      "type": "warning",
      "message": "Agent utilization below optimal threshold",
      "severity": "medium",
      "timestamp": "2024-01-01T00:08:00Z"
    }
  ],
  "timestamp": "2024-01-01T00:10:00Z",
  "request_id": "req-hive-status-12345"
}
```

**Response Fields:**

**Core Status:**
- `state`: Current hive operational state
  - `"Active"`: System fully operational
  - `"Initializing"`: System starting up
  - `"Degraded"`: System operational but with issues
  - `"Maintenance"`: System in maintenance mode
- `total_agents/active_agents/idle_agents/failed_agents`: Agent counts by status
- `total_tasks/pending_tasks/completed_tasks/failed_tasks`: Task statistics

**Swarm Intelligence Metrics:**
- `swarm_center`: 2D coordinates of swarm center of mass
- `swarm_radius`: Spatial distribution radius
- `swarm_density`: Agent concentration metric (0.0-1.0)
- `cohesion`: Swarm unity metric (0.0-1.0, higher is better)
- `communication_efficiency`: Inter-agent communication effectiveness (0.0-1.0)

**Performance Metrics:**
- `learning_progress`: Overall system learning advancement (0.0-1.0)
- `adaptation_rate`: System adaptation speed (0.0-1.0)
- `task_success_rate`: Percentage of successful task completions
- `average_task_completion_time_ms`: Mean task execution time
- `agent_utilization_percent`: Percentage of agent capacity in use

**System Health:**
- `total_energy/average_energy`: Energy levels across the swarm
- `energy_distribution`: Energy levels categorized by ranges
- `system_health_score`: Overall system health (0.0-1.0)
- `uptime_seconds`: System uptime in seconds

**Examples:**
```bash
# Get current hive status
curl -X GET http://localhost:3001/api/hive/status \
  -H "Accept: application/json" \
  -H "X-API-Key: your-api-key-here"

# Monitor swarm cohesion
curl -s -X GET http://localhost:3001/api/hive/status \
  -H "X-API-Key: your-api-key-here" | jq '.data.cohesion'

# Check system health
curl -s -X GET http://localhost:3001/api/hive/status \
  -H "X-API-Key: your-api-key-here" | jq '.data.system_health_score'

# Get agent utilization
curl -s -X GET http://localhost:3001/api/hive/status \
  -H "X-API-Key: your-api-key-here" | jq '.data.agent_utilization_percent'
```

**Use Cases:**
- System monitoring and health checks
- Performance analysis and optimization
- Capacity planning and scaling decisions
- Troubleshooting system issues
- Real-time dashboard updates
- Swarm intelligence research and analysis

**Common Monitoring Patterns:**
```bash
# Continuous health monitoring
while true; do
  STATUS=$(curl -s -X GET http://localhost:3001/api/hive/status \
    -H "X-API-Key: your-api-key-here" | jq -r '.data.state')
  echo "$(date): Hive status: $STATUS"
  sleep 60
done

# Alert on low cohesion
COHESION=$(curl -s -X GET http://localhost:3001/api/hive/status \
  -H "X-API-Key: your-api-key-here" | jq -r '.data.cohesion')

if (( $(echo "$COHESION < 0.7" | bc -l) )); then
  echo "WARNING: Swarm cohesion is low: $COHESION"
fi
```

#### Get Hive Metrics
```http
GET /api/hive/metrics
```

Returns detailed performance metrics, trends, and analytics for the hive system over time.

**Query Parameters:**
- `time_range` (optional): Time range for metrics. Valid values: `1h`, `6h`, `24h`, `7d`, `30d`. Default: `1h`
- `include_trends` (optional): Include trend analysis. Default: `true`
- `metrics` (optional): Comma-separated list of specific metrics to include. Valid values: `performance`, `efficiency`, `adaptation`, `communication`, `resource_usage`. Default: all

**Response:**
```json
{
  "success": true,
  "data": {
    "time_range": "1h",
    "current_metrics": {
      "performance": {
        "average_task_completion_time_ms": 2450,
        "task_success_rate": 0.87,
        "agent_utilization_percent": 78.5
      },
      "efficiency": {
        "energy_efficiency": 0.82,
        "resource_utilization": 0.75,
        "communication_overhead": 0.12
      },
      "adaptation": {
        "learning_rate": 0.15,
        "adaptation_speed": 0.68,
        "skill_diversity": 0.91
      }
    },
    "trends": {
      "performance_trend": "improving",
      "efficiency_trend": "stable",
      "adaptation_trend": "accelerating",
      "data_points": [
        {
          "timestamp": "2024-01-01T09:00:00Z",
          "task_completion_time_ms": 2800,
          "success_rate": 0.82
        },
        {
          "timestamp": "2024-01-01T10:00:00Z",
          "task_completion_time_ms": 2450,
          "success_rate": 0.87
        }
      ]
    },
    "anomalies": [
      {
        "type": "performance_drop",
        "severity": "medium",
        "description": "Task completion time increased by 15%",
        "timestamp": "2024-01-01T09:30:00Z"
      }
    ]
  },
  "timestamp": "2024-01-01T10:00:00Z"
}
```

**Examples:**
```bash
# Get current metrics with trends
curl -X GET "http://localhost:3001/api/hive/metrics" \
  -H "Accept: application/json"

# Get 24-hour performance metrics only
curl -X GET "http://localhost:3001/api/hive/metrics?time_range=24h&metrics=performance" \
  -H "Accept: application/json"
```

#### Get Resource Information
```http
GET /api/resources
```

Retrieves detailed system resource utilization information including CPU, memory, disk, network, and GPU statistics. This endpoint provides comprehensive monitoring data for system performance analysis and capacity planning.

**Parameters:** None

**Authentication:** API key required

**Rate Limiting:** 100 requests per minute per API key

**Response (Success - 200):**
```json
{
  "success": true,
  "data": {
    "cpu": {
      "usage_percent": 45.2,
      "cores": 8,
      "physical_cores": 4,
      "logical_cores": 8,
      "frequency_mhz": 3200,
      "frequency_min_mhz": 800,
      "frequency_max_mhz": 4200,
      "temperature_celsius": 65.5,
      "load_average": [1.25, 1.15, 1.08],
      "core_usage": [42.1, 38.7, 51.2, 46.8, 44.3, 49.1, 41.5, 43.9],
      "context_switches_per_sec": 12500,
      "interrupts_per_sec": 8500
    },
    "memory": {
      "total_mb": 16384,
      "used_mb": 8192,
      "available_mb": 8192,
      "usage_percent": 50.0,
      "free_mb": 4096,
      "buffers_mb": 512,
      "cached_mb": 3584,
      "swap_total_mb": 4096,
      "swap_used_mb": 1024,
      "swap_free_mb": 3072,
      "swap_usage_percent": 25.0
    },
    "disk": {
      "total_gb": 512,
      "used_gb": 128,
      "available_gb": 384,
      "usage_percent": 25.0,
      "free_gb": 384,
      "read_bytes_per_sec": 1048576,
      "write_bytes_per_sec": 2097152,
      "read_operations_per_sec": 150,
      "write_operations_per_sec": 200,
      "average_queue_length": 0.8,
      "average_wait_time_ms": 5.2
    },
    "network": {
      "bytes_sent_per_sec": 1048576,
      "bytes_received_per_sec": 2097152,
      "packets_sent_per_sec": 12500,
      "packets_received_per_sec": 18750,
      "errors_sent": 0,
      "errors_received": 2,
      "dropped_sent": 0,
      "dropped_received": 1,
      "connections_active": 25,
      "connections_total": 150,
      "bandwidth_mbps": 1000,
      "bandwidth_used_percent": 15.5
    },
    "gpu": {
      "available": true,
      "count": 1,
      "usage_percent": 35.8,
      "memory_used_mb": 2048,
      "memory_total_mb": 8192,
      "memory_free_mb": 6144,
      "memory_usage_percent": 25.0,
      "temperature_celsius": 72.0,
      "power_draw_watts": 185,
      "power_limit_watts": 250,
      "fan_speed_percent": 65,
      "clocks": {
        "graphics_mhz": 1800,
        "memory_mhz": 3500,
        "video_mhz": 1350
      }
    },
    "system": {
      "uptime_seconds": 36000,
      "load_average": [1.25, 1.15, 1.08],
      "process_count": 245,
      "thread_count": 1250,
      "context_switches_total": 45000000,
      "interrupts_total": 28000000
    }
  },
  "trends": {
    "cpu_trend": "stable",
    "memory_trend": "increasing",
    "disk_trend": "stable",
    "network_trend": "moderate",
    "period_minutes": 15
  },
  "thresholds": {
    "cpu_warning_percent": 80,
    "cpu_critical_percent": 95,
    "memory_warning_percent": 85,
    "memory_critical_percent": 95,
    "disk_warning_percent": 90,
    "disk_critical_percent": 95
  },
  "alerts": [
    {
      "resource": "memory",
      "type": "warning",
      "message": "Memory usage approaching warning threshold",
      "current_value": 82.5,
      "threshold": 85.0,
      "timestamp": "2024-01-01T00:08:00Z"
    }
  ],
  "timestamp": "2024-01-01T00:10:00Z",
  "collection_duration_ms": 45,
  "request_id": "req-resources-12345"
}
```

**Response Fields:**

**CPU Metrics:**
- `usage_percent`: Overall CPU utilization
- `cores/physical_cores/logical_cores`: CPU core information
- `frequency_mhz`: Current CPU frequency
- `temperature_celsius`: CPU temperature
- `load_average`: System load averages (1, 5, 15 minutes)
- `core_usage`: Per-core utilization percentages

**Memory Metrics:**
- `total_mb/used_mb/available_mb`: Memory amounts in MB
- `usage_percent`: Memory utilization percentage
- `buffers_mb/cached_mb`: Memory buffer/cache usage
- `swap_*`: Swap space utilization

**Disk Metrics:**
- `total_gb/used_gb/available_gb`: Disk space in GB
- `usage_percent`: Disk utilization percentage
- `read/write_bytes_per_sec`: I/O throughput
- `read/write_operations_per_sec`: I/O operations per second

**Network Metrics:**
- `bytes/packets_*_per_sec`: Network throughput
- `errors/dropped_*`: Network error counters
- `connections_*`: Active connection counts
- `bandwidth_*`: Network bandwidth information

**GPU Metrics (if available):**
- `usage_percent`: GPU utilization
- `memory_*`: GPU memory information
- `temperature_celsius`: GPU temperature
- `power_draw/limit_watts`: Power consumption
- `clocks`: GPU clock frequencies

**System Metrics:**
- `uptime_seconds`: System uptime
- `process/thread_count`: Running processes/threads
- `context_switches/interrupts_total`: System activity counters

**Examples:**
```bash
# Get current resource usage
curl -X GET http://localhost:3001/api/resources \
  -H "Accept: application/json" \
  -H "X-API-Key: your-api-key-here"

# Monitor CPU usage
curl -s -X GET http://localhost:3001/api/resources \
  -H "X-API-Key: your-api-key-here" | jq '.data.cpu.usage_percent'

# Check memory usage
curl -s -X GET http://localhost:3001/api/resources \
  -H "X-API-Key: your-api-key-here" | jq '.data.memory.usage_percent'

# Get GPU information
curl -s -X GET http://localhost:3001/api/resources \
  -H "X-API-Key: your-api-key-here" | jq '.data.gpu'

# Monitor resource trends
curl -s -X GET http://localhost:3001/api/resources \
  -H "X-API-Key: your-api-key-here" | jq '.trends'
```

**Use Cases:**
- System performance monitoring
- Resource capacity planning
- Troubleshooting performance issues
- Auto-scaling decisions
- Infrastructure cost optimization
- Real-time dashboard updates

**Common Monitoring Patterns:**
```bash
# Continuous resource monitoring
while true; do
  CPU=$(curl -s -X GET http://localhost:3001/api/resources \
    -H "X-API-Key: your-api-key-here" | jq -r '.data.cpu.usage_percent')
  MEM=$(curl -s -X GET http://localhost:3001/api/resources \
    -H "X-API-Key: your-api-key-here" | jq -r '.data.memory.usage_percent')

  echo "$(date): CPU: ${CPU}%, Memory: ${MEM}%"
  sleep 30
done

# Alert on high resource usage
THRESHOLDS=$(curl -s -X GET http://localhost:3001/api/resources \
  -H "X-API-Key: your-api-key-here" | jq '.thresholds')

CPU_USAGE=$(curl -s -X GET http://localhost:3001/api/resources \
  -H "X-API-Key: your-api-key-here" | jq -r '.data.cpu.usage_percent')

if (( $(echo "$CPU_USAGE > 80" | bc -l) )); then
  echo "WARNING: High CPU usage: ${CPU_USAGE}%"
fi
```

### Debug System Information
```http
GET /debug/system
```

**⚠️ Development Only Endpoint**: This endpoint is intended for development and debugging purposes only. It should not be exposed in production environments.

Retrieves comprehensive system debugging information including hive status, agent details, task queue, resource metrics, memory statistics, configuration snapshots, and internal system state. This endpoint aggregates data from multiple internal systems for advanced troubleshooting and development analysis.

**Parameters:** None

**Authentication:** API key required

**Rate Limiting:** 30 requests per minute per API key (debug endpoints have lower limits for safety)

**Response (Success - 200):**
```json
{
  "success": true,
  "data": {
    "hive_status": {
      "state": "Active",
      "total_agents": 15,
      "active_agents": 12,
      "idle_agents": 2,
      "failed_agents": 1,
      "total_tasks": 45,
      "pending_tasks": 8,
      "in_progress_tasks": 5,
      "completed_tasks": 32,
      "failed_tasks": 5,
      "swarm_center": [45.2, 67.8],
      "swarm_radius": 25.5,
      "cohesion": 0.85,
      "learning_progress": 0.72,
      "communication_efficiency": 0.91,
      "phase": "Phase 2 - CPU-native, GPU-optional"
    },
    "agents_info": {
      "total_count": 15,
      "active_count": 12,
      "idle_count": 2,
      "failed_count": 1,
      "by_type": {
        "Worker": 8,
        "Coordinator": 3,
        "Specialist": 2,
        "Learner": 2
      },
      "by_status": {
        "Active": 12,
        "Idle": 2,
        "Initializing": 1,
        "Failed": 1
      },
      "average_performance": 0.87,
      "average_energy": 83.4,
      "total_capabilities": 45,
      "capability_distribution": {
        "data_processing": 8,
        "machine_learning": 3,
        "task_coordination": 5,
        "analysis": 6
      }
    },
    "tasks_info": {
      "total_count": 45,
      "pending_count": 8,
      "in_progress_count": 5,
      "completed_count": 32,
      "failed_count": 0,
      "cancelled_count": 0,
      "by_priority": {
        "Low": 10,
        "Medium": 20,
        "High": 12,
        "Critical": 3
      },
      "by_status": {
        "pending": 8,
        "in_progress": 5,
        "completed": 32,
        "failed": 0,
        "cancelled": 0
      },
      "by_type": {
        "data_processing": 15,
        "analysis": 12,
        "coordination": 8,
        "learning": 5,
        "maintenance": 5
      },
      "average_queue_wait_time_ms": 125000,
      "average_completion_time_ms": 245000,
      "success_rate_percent": 100.0
    },
    "resource_info": {
      "cpu_usage_percent": 45.2,
      "memory_usage_percent": 60.8,
      "disk_usage_percent": 25.0,
      "network_connections": 25,
      "gpu_available": true,
      "gpu_usage_percent": 35.8,
      "system_load": [1.25, 1.15, 1.08]
    },
    "memory_stats": {
      "total_allocated_mb": 256,
      "active_allocations": 1247,
      "largest_allocation_mb": 64,
      "memory_pressure": "low",
      "gc_cycles": 45,
      "heap_size_mb": 512,
      "heap_used_mb": 256,
      "fragmentation_percent": 12.5
    },
    "queue_systems": {
      "work_stealing_enabled": true,
      "work_stealing_efficiency": 0.89,
      "queue_depth": 8,
      "processing_threads": 4,
      "legacy_queue_info": {
        "pending_tasks": 8,
        "completed_tasks": 32,
        "failed_tasks": 0,
        "average_wait_time_ms": 125000
      }
    },
    "system_uptime_seconds": 36000,
    "last_health_check": "2024-01-01T00:10:00Z",
    "configuration_snapshot": {
      "server_host": "localhost",
      "server_port": 3001,
      "log_level": "info",
      "metrics_collection_interval_ms": 5000,
      "alert_check_interval_ms": 30000,
      "max_concurrent_tasks": 50,
      "rate_limit_requests_per_minute": 1000
    },
    "performance_metrics": {
      "api_response_time_ms": 15,
      "database_query_time_ms": 5,
      "cache_hit_rate": 0.85,
      "error_rate_per_minute": 0.001
    }
  },
  "debug_info": {
    "request_tracing": {
      "request_id": "debug-req-12345",
      "trace_id": "trace-abc-123",
      "span_id": "span-xyz-789"
    },
    "system_info": {
      "hostname": "hive-server-01",
      "platform": "linux",
      "architecture": "x86_64",
      "cpu_count": 8,
      "total_memory_mb": 16384,
      "rust_version": "1.70.0",
      "build_date": "2024-01-01T00:00:00Z"
    },
    "environment_variables": {
      "RUST_LOG": "info",
      "HIVE_CONFIG_PATH": "/etc/hive/config.toml",
      "DATABASE_URL": "postgresql://localhost/hive"
    }
  },
  "timestamp": "2024-01-01T00:10:00Z",
  "request_id": "debug-req-12345",
  "processing_time_ms": 45
}
```

**Response Fields:**

**Core System Data:**
- `hive_status`: Complete hive operational state
- `agents_info`: Detailed agent statistics and distributions
- `tasks_info`: Comprehensive task queue information
- `resource_info`: Current system resource utilization
- `memory_stats`: Detailed memory usage and allocation info
- `queue_systems`: Work queue and processing system status

**Configuration & Environment:**
- `configuration_snapshot`: Current system configuration
- `system_info`: Hardware and software environment details
- `environment_variables`: Relevant environment variables (sanitized)

**Performance Data:**
- `performance_metrics`: System performance indicators
- `debug_info`: Request tracing and debugging information

**Examples:**
```bash
# Get comprehensive debug information
curl -X GET http://localhost:3001/debug/system \
  -H "Accept: application/json" \
  -H "X-API-Key: your-api-key-here"

# Debug agent-related issues
curl -s -X GET http://localhost:3001/debug/system \
  -H "X-API-Key: your-api-key-here" | jq '.data.agents_info'

# Check task queue status
curl -s -X GET http://localhost:3001/debug/system \
  -H "X-API-Key: your-api-key-here" | jq '.data.tasks_info'

# Monitor memory usage
curl -s -X GET http://localhost:3001/debug/system \
  -H "X-API-Key: your-api-key-here" | jq '.data.memory_stats'

# Get configuration snapshot
curl -s -X GET http://localhost:3001/debug/system \
  -H "X-API-Key: your-api-key-here" | jq '.data.configuration_snapshot'
```

**Common Use Cases:**
- **Agent Communication Issues**: Check agent status, connectivity, and message queues
- **Task Assignment Problems**: Analyze task queue, agent capabilities, and assignment logic
- **Performance Bottlenecks**: Monitor resource usage, queue depths, and processing times
- **Memory Leaks**: Track memory allocation patterns and garbage collection
- **Configuration Issues**: Verify system configuration and environment settings
- **Integration Testing**: Comprehensive system state validation

**Debugging Workflows:**
```bash
# 1. Check overall system health
curl -s http://localhost:3001/debug/system | jq '.data.hive_status.state'

# 2. Investigate agent issues
curl -s http://localhost:3001/debug/system | jq '.data.agents_info'

# 3. Analyze task processing
curl -s http://localhost:3001/debug/system | jq '.data.tasks_info'

# 4. Check resource constraints
curl -s http://localhost:3001/debug/system | jq '.data.resource_info'

# 5. Monitor memory usage
curl -s http://localhost:3001/debug/system | jq '.data.memory_stats'
```

**Security Considerations:**
- **Access Control**: Restrict to development and admin users only
- **Data Sanitization**: Sensitive information is automatically redacted
- **Rate Limiting**: Lower limits to prevent abuse
- **Logging**: All debug requests are logged for audit purposes
- **Network Security**: Use HTTPS and proper authentication in development

**Best Practices:**
1. Use this endpoint sparingly in production-like environments
2. Implement proper access controls and monitoring
3. Combine with other debugging tools for comprehensive analysis
4. Document any findings for future reference
5. Clean up debug logs regularly to manage storage

## WebSocket Events

The AI Orchestrator Hub provides real-time updates via WebSocket connections. Connect to receive live notifications about agent activities, task progress, system status, and alerts.

### Connection Setup

Connect to the WebSocket endpoint for real-time updates:

```javascript
const ws = new WebSocket('ws://localhost:3001/ws');

// Handle connection open
ws.onopen = (event) => {
  console.log('Connected to AI Orchestrator Hub');
};

// Handle incoming messages
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  handleWebSocketMessage(message);
};

// Handle connection close
ws.onclose = (event) => {
  console.log('Disconnected from AI Orchestrator Hub');
};

// Handle errors
ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};
```

**Connection Parameters:**
- **URL**: `ws://localhost:3001/ws` (use `wss://` for secure connections)
- **Protocol**: Standard WebSocket protocol
- **Authentication**: Include API key in query parameter: `?api_key=your-api-key`

### Connection Events

#### Connection Established
Sent immediately after successful WebSocket connection.

```json
{
  "type": "connection_established",
  "data": {
    "client_id": "client-123",
    "server_version": "2.0.0",
    "connection_id": "conn-456",
    "timestamp": "2024-01-01T00:00:00Z",
    "features": ["agents", "tasks", "metrics", "alerts"]
  }
}
```

#### Heartbeat
Sent every 30 seconds to maintain connection and verify liveness.

```json
{
  "type": "heartbeat",
  "data": {
    "timestamp": "2024-01-01T00:00:30Z",
    "server_uptime_seconds": 3600,
    "active_connections": 15
  }
}
```

### Agent Events

#### Agent Created
Sent when a new agent is successfully created in the system.

```json
{
  "type": "agent_created",
  "data": {
    "agent": {
      "id": "agent-789",
      "name": "DataProcessor-1",
      "type": "Worker",
      "status": "Initializing",
      "capabilities": [
        {
          "name": "data_processing",
          "proficiency": 0.8,
          "learning_rate": 0.1
        }
      ],
      "position": [0.0, 0.0],
      "energy": 100.0,
      "created_at": "2024-01-01T00:01:00Z"
    },
    "hive_impact": {
      "total_agents": 16,
      "active_agents": 12
    },
    "timestamp": "2024-01-01T00:01:00Z"
  }
}
```

#### Agent Updated
Sent when an agent's status, capabilities, or properties are modified.

```json
{
  "type": "agent_updated",
  "data": {
    "agent_id": "agent-123",
    "changes": {
      "status": "Active",
      "energy": 90.0,
      "capabilities": [
        {
          "name": "data_processing",
          "proficiency": 0.95
        }
      ]
    },
    "previous_values": {
      "status": "Initializing",
      "energy": 100.0
    },
    "reason": "capability_upgrade",
    "timestamp": "2024-01-01T00:02:00Z"
  }
}
```

#### Agent Failed
Sent when an agent encounters an error or becomes unresponsive.

```json
{
  "type": "agent_failed",
  "data": {
    "agent_id": "agent-123",
    "error": "Connection timeout after 30 seconds",
    "error_code": "CONNECTION_TIMEOUT",
    "recovery_status": "Retrying",
    "retry_count": 1,
    "last_heartbeat": "2024-01-01T00:01:30Z",
    "impact_assessment": {
      "affected_tasks": 2,
      "severity": "medium"
    },
    "timestamp": "2024-01-01T00:02:00Z"
  }
}
```

#### Agent Recovered
Sent when a failed agent successfully recovers.

```json
{
  "type": "agent_recovered",
  "data": {
    "agent_id": "agent-123",
    "recovery_time_ms": 5000,
    "new_status": "Active",
    "energy_restored": 85.0,
    "timestamp": "2024-01-01T00:03:00Z"
  }
}
```

### Task Events

#### Task Created
Sent when a new task is created and queued for assignment.

```json
{
  "type": "task_created",
  "data": {
    "task": {
      "id": "task-456",
      "description": "Process customer feedback data",
      "type": "data_processing",
      "priority": "High",
      "status": "Pending",
      "required_capabilities": [
        {
          "name": "data_processing",
          "min_proficiency": 0.7
        }
      ],
      "created_at": "2024-01-01T00:04:00Z",
      "estimated_duration_ms": 30000
    },
    "queue_position": 3,
    "estimated_assignment_time": "2024-01-01T00:04:15Z",
    "timestamp": "2024-01-01T00:04:00Z"
  }
}
```

#### Task Assigned
Sent when a task is assigned to an agent.

```json
{
  "type": "task_assigned",
  "data": {
    "task_id": "task-123",
    "agent_id": "agent-456",
    "assignment_reason": "capability_match",
    "capability_match_score": 0.92,
    "estimated_completion": "2024-01-01T00:06:00Z",
    "agent_workload_before": 2,
    "agent_workload_after": 3,
    "timestamp": "2024-01-01T00:05:00Z"
  }
}
```

#### Task Progress Update
Sent periodically during task execution to report progress.

```json
{
  "type": "task_progress",
  "data": {
    "task_id": "task-123",
    "progress": 0.65,
    "stage": "processing_data",
    "stage_description": "Analyzing customer feedback patterns",
    "current_step": 13,
    "total_steps": 20,
    "elapsed_time_ms": 8500,
    "estimated_remaining_ms": 4500,
    "performance_metrics": {
      "cpu_usage": 75.5,
      "memory_usage": 60.2,
      "data_processed_mb": 45.8
    },
    "timestamp": "2024-01-01T00:05:30Z"
  }
}
```

#### Task Completed
Sent when a task finishes successfully.

```json
{
  "type": "task_completed",
  "data": {
    "task_id": "task-123",
    "result": {
      "summary": "Successfully processed 1000 customer feedback entries",
      "insights": [
        "85% positive sentiment",
        "Top issues: response time, product quality"
      ],
      "output_files": ["/results/task-123/analysis.json"],
      "metrics": {
        "records_processed": 1000,
        "accuracy_score": 0.94
      }
    },
    "execution_time_ms": 15000,
    "agent_performance_score": 0.96,
    "energy_consumed": 15.5,
    "timestamp": "2024-01-01T00:06:00Z"
  }
}
```

#### Task Failed
Sent when a task fails to complete.

```json
{
  "type": "task_failed",
  "data": {
    "task_id": "task-123",
    "error": "Data validation failed: invalid format in record 456",
    "error_code": "VALIDATION_ERROR",
    "error_details": {
      "field": "customer_email",
      "value": "invalid-email",
      "reason": "Invalid email format"
    },
    "retry_count": 2,
    "max_retries": 3,
    "can_retry": true,
    "partial_results": {
      "records_processed": 455,
      "valid_records": 450
    },
    "timestamp": "2024-01-01T00:07:00Z"
  }
}
```

### System Events

#### Hive Status Update
Sent when the overall hive status changes significantly.

```json
{
  "type": "hive_status",
  "data": {
    "status": {
      "state": "Active",
      "total_agents": 15,
      "active_agents": 13,
      "total_tasks": 45,
      "pending_tasks": 5,
      "completed_tasks": 35,
      "swarm_center": [45.2, 67.8],
      "cohesion": 0.87,
      "learning_progress": 0.74
    },
    "changes": {
      "active_agents": "+1",
      "completed_tasks": "+2"
    },
    "timestamp": "2024-01-01T00:08:00Z"
  }
}
```

#### Metrics Update
Sent with updated system performance metrics.

```json
{
  "type": "metrics_update",
  "data": {
    "metrics": {
      "performance": {
        "average_response_time_ms": 245,
        "requests_per_second": 15.7,
        "error_rate": 0.001
      },
      "resources": {
        "cpu_usage_percent": 45.2,
        "memory_usage_percent": 60.8,
        "disk_usage_percent": 25.0
      },
      "hive": {
        "agent_utilization": 78.5,
        "task_success_rate": 0.94,
        "communication_efficiency": 0.91
      }
    },
    "trends": {
      "cpu_trend": "stable",
      "memory_trend": "increasing",
      "performance_trend": "improving"
    },
    "timestamp": "2024-01-01T00:09:00Z"
  }
}
```

#### Alert Triggered
Sent when system alerts are activated.

```json
{
  "type": "alert_triggered",
  "data": {
    "alert_id": "alert-789",
    "alert_type": "high_cpu_usage",
    "severity": "warning",
    "title": "High CPU Usage Detected",
    "message": "CPU usage has exceeded 80% for 5 minutes",
    "threshold": 80.0,
    "current_value": 85.2,
    "duration_minutes": 5,
    "affected_components": ["cpu", "task_scheduler"],
    "recommended_actions": [
      "Check for runaway processes",
      "Consider scaling up resources",
      "Review task priorities"
    ],
    "auto_resolution": false,
    "timestamp": "2024-01-01T00:10:00Z"
  }
}
```

#### Alert Resolved
Sent when system alerts are resolved.

```json
{
  "type": "alert_resolved",
  "data": {
    "alert_id": "alert-789",
    "resolution_time_ms": 300000,
    "final_value": 65.8,
    "resolution_method": "automatic",
    "timestamp": "2024-01-01T00:15:00Z"
  }
}
```

### JavaScript Example

```javascript
function handleWebSocketMessage(message) {
  switch (message.type) {
    case 'connection_established':
      console.log('Connected:', message.data.client_id);
      break;

    case 'agent_created':
      console.log('New agent:', message.data.agent.name);
      updateAgentList(message.data.agent);
      break;

    case 'task_progress':
      updateTaskProgress(message.data.task_id, message.data.progress);
      break;

    case 'task_completed':
      console.log('Task completed:', message.data.task_id);
      showTaskResult(message.data.result);
      break;

    case 'alert_triggered':
      showAlert(message.data);
      break;

    case 'heartbeat':
      // Update connection status
      break;

    default:
      console.log('Unknown message type:', message.type);
  }
}

// Reconnection logic
function connectWebSocket() {
  const ws = new WebSocket('ws://localhost:3001/ws');

  ws.onopen = () => console.log('WebSocket connected');
  ws.onmessage = (event) => handleWebSocketMessage(JSON.parse(event.data));
  ws.onclose = () => {
    console.log('WebSocket disconnected, reconnecting...');
    setTimeout(connectWebSocket, 5000);
  };
  ws.onerror = (error) => console.error('WebSocket error:', error);

  return ws;
}
```

## MCP Integration

The AI Orchestrator Hub implements the Model Context Protocol (MCP) for seamless integration with external AI tools and services. MCP enables standardized communication between AI models and tools.

### MCP Endpoints

#### Health Check
```http
GET /api/mcp/health
```

Checks the health and connectivity of the MCP service.

**Parameters:** None

**Response:**
```json
{
  "service": "mcp-http",
  "status": "healthy",
  "hive_connected": true,
  "total_agents": 15,
  "active_agents": 12,
  "timestamp": "2024-01-01T00:00:00Z",
  "version": "1.0.0"
}
```

**Example:**
```bash
curl -X GET http://localhost:3001/api/mcp/health \
  -H "Accept: application/json"
```

#### Execute MCP Request
```http
POST /api/mcp/
```

Executes MCP tools and methods using JSON-RPC 2.0 protocol.

**Request Body:** JSON-RPC 2.0 request

**Request Body:** JSON-RPC 2.0 request
```json
{
  "jsonrpc": "2.0",
  "id": "req-12345",
  "method": "tools/call",
  "params": {
    "name": "create_swarm_agent",
    "arguments": {
      "name": "DataProcessor",
      "agent_type": "worker",
      "capabilities": [
        {
          "name": "data_processing",
          "proficiency": 0.8
        }
      ]
    }
  }
}
```

**Available Tools:**
- `create_swarm_agent`: Create a new agent with specified capabilities
- `assign_swarm_task`: Assign a task to the most suitable agent
- `get_swarm_status`: Retrieve comprehensive hive status and metrics
- `analyze_with_nlp`: Perform natural language processing analysis
- `coordinate_agents`: Coordinate multiple agents for complex tasks
- `list_agents`: List agents with filtering
- `list_tasks`: List tasks with filtering

**Response:** JSON-RPC 2.0 response
```json
{
  "jsonrpc": "2.0",
  "id": "req-12345",
  "result": {
    "success": true,
    "data": {
      "id": "agent-456",
      "name": "DataProcessor",
      "type": "worker",
      "status": "Initializing",
      "capabilities": [...],
      "created_at": "2024-01-01T00:10:00Z"
    },
    "message": "Agent created successfully",
    "processing_time_ms": 45
  }
}
```

**Examples:**
```bash
# Create a new agent via MCP
curl -X POST http://localhost:3001/api/mcp/ \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "req-1",
    "method": "tools/call",
    "params": {
      "name": "create_swarm_agent",
      "arguments": {
        "name": "AnalysisAgent",
        "agent_type": "worker",
        "capabilities": [
          {
            "name": "data_analysis",
            "proficiency": 0.9
          }
        ]
      }
    }
  }'

# Assign a task via MCP
curl -X POST http://localhost:3001/api/mcp/ \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "req-2",
    "method": "tools/call",
    "params": {
      "name": "assign_swarm_task",
      "arguments": {
        "task_description": "Analyze customer feedback data",
        "priority": "high",
        "required_capabilities": [
          {
            "name": "sentiment_analysis",
            "minimum_proficiency": 0.8
          }
        ]
      }
    }
  }'

# Get swarm status via MCP
curl -X POST http://localhost:3001/api/mcp/ \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "req-3",
    "method": "tools/call",
    "params": {
      "name": "get_swarm_status",
      "arguments": {
        "include_agents": true,
        "include_tasks": true,
        "include_metrics": true
      }
    }
  }'
```

## Error Handling

The AI Orchestrator Hub uses a comprehensive error handling system with standardized error responses, detailed error codes, and actionable error messages.

### Error Response Format

All API errors follow a consistent JSON structure:

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Input validation failed",
    "details": {
      "field": "name",
      "reason": "Agent name cannot be empty"
    },
    "field_errors": [
      {
        "field": "name",
        "message": "Agent name cannot be empty",
        "value": ""
      }
    ]
  },
  "timestamp": "2024-01-01T00:00:00Z",
  "request_id": "req-12345"
}
```

### Error Response Fields

- `success`: Always `false` for error responses
- `error.code`: Machine-readable error code for programmatic handling
- `error.message`: Human-readable error description
- `error.details`: Additional error context (optional)
- `error.field_errors`: Array of field-specific validation errors (optional)
- `timestamp`: When the error occurred
- `request_id`: Unique request identifier for tracing
- `processing_time_ms`: How long the server took to process the request

### HTTP Status Codes

| Status Code | Meaning | Typical Usage |
|-------------|---------|---------------|
| 400 | Bad Request | Validation errors, malformed requests |
| 401 | Unauthorized | Missing or invalid authentication |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource doesn't exist |
| 409 | Conflict | Resource state conflict (e.g., agent already exists) |
| 422 | Unprocessable Entity | Semantic validation errors |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Unexpected server errors |
| 502 | Bad Gateway | Upstream service errors |
| 503 | Service Unavailable | Service temporarily down |
| 504 | Gateway Timeout | Request timeout |

### Error Codes

#### Validation Errors (400)

- `VALIDATION_ERROR`: Input validation failed
  - **Common causes**: Invalid field values, missing required fields, incorrect data types
  - **Resolution**: Check field_errors array for specific issues

- `AGENT_CREATION_FAILED`: Agent creation validation failed
  - **Common causes**: Invalid agent name, unsupported agent type, invalid capabilities
  - **Resolution**: Review agent creation requirements

- `TASK_CREATION_FAILED`: Task creation validation failed
  - **Common causes**: Empty description, invalid priority, malformed capabilities
  - **Resolution**: Check task creation parameters

#### Resource Errors (404)

- `AGENT_NOT_FOUND`: Specified agent does not exist
  - **Common causes**: Invalid agent ID, agent was deleted
  - **Resolution**: Verify agent ID and check agent list

- `TASK_NOT_FOUND`: Specified task does not exist
  - **Common causes**: Invalid task ID, task was deleted
  - **Resolution**: Verify task ID and check task list

#### Server Errors (500)

- `INTERNAL_ERROR`: Unexpected server error
  - **Common causes**: Bugs, database issues, system failures
  - **Resolution**: Retry with exponential backoff, contact support if persistent

#### Rate Limiting (429)

- `RATE_LIMIT_EXCEEDED`: Too many requests
  - **Common causes**: Exceeded API rate limits
  - **Resolution**: Implement exponential backoff, reduce request frequency

### Handling Errors in Client Code

#### JavaScript/TypeScript Example

```javascript
async function apiRequest(endpoint, options = {}) {
  try {
    const response = await fetch(`/api/${endpoint}`, {
      headers: {
        'Content-Type': 'application/json',
        'X-API-Key': 'your-api-key'
      },
      ...options
    });

    const data = await response.json();

    if (!response.ok) {
      throw new ApiError(data.error, response.status);
    }

    return data;
  } catch (error) {
    if (error instanceof ApiError) {
      handleApiError(error);
    } else {
      // Network or other errors
      console.error('Network error:', error);
    }
    throw error;
  }
}

class ApiError extends Error {
  constructor(errorData, statusCode) {
    super(errorData.message);
    this.code = errorData.code;
    this.details = errorData.details;
    this.fieldErrors = errorData.field_errors || [];
    this.statusCode = statusCode;
    this.requestId = errorData.request_id;
  }
}

function handleApiError(error) {
  switch (error.code) {
    case 'VALIDATION_ERROR':
      showValidationErrors(error.fieldErrors);
      break;

    case 'RATE_LIMIT_EXCEEDED':
      showRateLimitMessage(error);
      break;

    case 'AGENT_NOT_FOUND':
      showNotFoundMessage('Agent', error.details?.id);
      break;

    case 'UNAUTHORIZED':
      redirectToLogin();
      break;

    default:
      showGenericError(error);
  }
}
```

#### Python Example

```python
import requests
from typing import Dict, Any

class ApiClient:
    def __init__(self, base_url: str, api_key: str):
        self.base_url = base_url
        self.session = requests.Session()
        self.session.headers.update({
            'X-API-Key': api_key,
            'Content-Type': 'application/json'
        })

    def request(self, method: str, endpoint: str, **kwargs) -> Dict[str, Any]:
        try:
            response = self.session.request(method, f"{self.base_url}/api/{endpoint}", **kwargs)
            data = response.json()

            if not response.ok:
                raise ApiError(data['error'], response.status_code)

            return data
        except requests.RequestException as e:
            raise NetworkError(f"Network error: {e}")

class ApiError(Exception):
    def __init__(self, error_data: Dict[str, Any], status_code: int):
        self.code = error_data['code']
        self.message = error_data['message']
        self.details = error_data.get('details')
        self.field_errors = error_data.get('field_errors', [])
        self.status_code = status_code
        self.request_id = error_data.get('request_id')
        super().__init__(self.message)

def handle_error(error: ApiError):
    if error.code == 'VALIDATION_ERROR':
        for field_error in error.field_errors:
            print(f"Validation error in {field_error['field']}: {field_error['message']}")
    elif error.code == 'RATE_LIMIT_EXCEEDED':
        print("Rate limit exceeded. Please wait before retrying.")
    elif error.code == 'AGENT_NOT_FOUND':
        print(f"Agent not found: {error.details.get('id', 'unknown')}")
    else:
        print(f"API Error: {error.message}")

# Usage
client = ApiClient("http://localhost:3001", "your-api-key")

try:
    agents = client.request('GET', 'agents')
    print("Agents:", agents)
except ApiError as e:
    handle_error(e)
```

### Error Prevention Best Practices

1. **Validate Input Client-Side**: Implement client-side validation to catch common errors before API calls
2. **Use Appropriate Timeouts**: Set reasonable timeouts for API calls
3. **Implement Retry Logic**: Use exponential backoff for transient errors
4. **Monitor Error Rates**: Track error patterns to identify issues early
5. **Handle Rate Limits**: Implement proper rate limit handling with backoff
6. **Log Errors**: Include request IDs in logs for better debugging

## Rate Limiting

The AI Orchestrator Hub implements comprehensive rate limiting to ensure fair usage and system stability. Rate limits are applied per API key and reset every minute.

### Rate Limit Categories

| Operation | Limit | Scope | Reset Period |
|-----------|-------|-------|--------------|
| Agent creation | Rate limited | Per API key | 1 minute |
| Task creation | Rate limited | Per API key | 1 minute |
| General API calls | Not explicitly limited | Per API key | N/A |

### Rate Limit Headers

Rate limit information is included in all API responses:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
X-RateLimit-Reset-After: 45
```

**Header Descriptions:**
- `X-RateLimit-Limit`: Maximum requests allowed in the current window
- `X-RateLimit-Remaining`: Number of requests remaining in the current window
- `X-RateLimit-Reset`: Unix timestamp when the rate limit resets
- `X-RateLimit-Reset-After`: Seconds until the rate limit resets

### Rate Limit Exceeded Response

When rate limits are exceeded, the API returns:

```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded",
    "details": {
      "limit": 100,
      "remaining": 0,
      "reset_after_seconds": 45,
      "retry_after": "2024-01-01T00:01:00Z"
    }
  },
  "timestamp": "2024-01-01T00:00:15Z",
  "request_id": "req-12345"
}
```

### Rate Limiting Strategy

#### Sliding Window
- Uses a sliding window algorithm for more accurate rate limiting
- Allows burst traffic while preventing sustained high usage
- Resets continuously rather than at fixed intervals

#### Per-API Key Limits
- Limits are enforced per API key, not per IP
- Allows multiple applications to share rate limit pools
- Prevents abuse while allowing legitimate high usage

#### Graduated Limits
- Different limits for different operation types
- Expensive operations (agent/task creation) have lower limits
- Read operations (health, metrics) have higher limits

### Handling Rate Limits

#### Client Implementation

```javascript
class RateLimitedApiClient {
  constructor(baseUrl, apiKey) {
    this.baseUrl = baseUrl;
    this.apiKey = apiKey;
    this.retryQueue = [];
    this.isProcessingQueue = false;
  }

  async request(endpoint, options = {}) {
    const response = await fetch(`${this.baseUrl}/api/${endpoint}`, {
      headers: {
        'X-API-Key': this.apiKey,
        'Content-Type': 'application/json',
        ...options.headers
      },
      ...options
    });

    // Check for rate limiting
    if (response.status === 429) {
      const resetAfter = response.headers.get('X-RateLimit-Reset-After');
      const retryAfter = parseInt(response.headers.get('Retry-After')) || 60;

      // Queue the request for retry
      this.retryQueue.push({
        endpoint,
        options,
        retryAfter: Date.now() + (retryAfter * 1000)
      });

      // Start processing retry queue if not already running
      if (!this.isProcessingQueue) {
        this.processRetryQueue();
      }

      throw new RateLimitError(retryAfter);
    }

    return response;
  }

  async processRetryQueue() {
    if (this.isProcessingQueue || this.retryQueue.length === 0) return;

    this.isProcessingQueue = true;

    while (this.retryQueue.length > 0) {
      const item = this.retryQueue[0];
      const now = Date.now();

      if (now < item.retryAfter) {
        // Wait until retry time
        await new Promise(resolve => setTimeout(resolve, item.retryAfter - now));
      }

      try {
        await this.request(item.endpoint, item.options);
        this.retryQueue.shift(); // Remove successful request
      } catch (error) {
        if (error instanceof RateLimitError) {
          // Still rate limited, wait longer
          item.retryAfter = Date.now() + (error.retryAfter * 1000);
          break; // Stop processing queue
        } else {
          // Other error, remove from queue
          this.retryQueue.shift();
        }
      }
    }

    this.isProcessingQueue = false;
  }
}

class RateLimitError extends Error {
  constructor(retryAfter) {
    super('Rate limit exceeded');
    this.retryAfter = retryAfter;
  }
}
```

#### Python Implementation

```python
import time
import requests
from typing import Dict, Any, Optional

class RateLimitedClient:
    def __init__(self, base_url: str, api_key: str):
        self.base_url = base_url
        self.api_key = api_key
        self.session = requests.Session()
        self.session.headers.update({
            'X-API-Key': api_key,
            'Content-Type': 'application/json'
        })
        self.retry_queue = []
        self.last_request_time = 0
        self.min_request_interval = 0.1  # 100ms between requests

    def request(self, method: str, endpoint: str, **kwargs) -> Dict[str, Any]:
        # Enforce minimum request interval
        elapsed = time.time() - self.last_request_time
        if elapsed < self.min_request_interval:
            time.sleep(self.min_request_interval - elapsed)

        response = self.session.request(method, f"{self.base_url}/api/{endpoint}", **kwargs)
        self.last_request_time = time.time()

        if response.status_code == 429:
            retry_after = int(response.headers.get('Retry-After', 60))
            raise RateLimitError(retry_after)

        response.raise_for_status()
        return response.json()

class RateLimitError(Exception):
    def __init__(self, retry_after: int):
        self.retry_after = retry_after
        super().__init__(f"Rate limit exceeded. Retry after {retry_after} seconds")

# Usage with automatic retry
def make_request_with_retry(client: RateLimitedClient, method: str, endpoint: str, max_retries: int = 3, **kwargs):
    for attempt in range(max_retries):
        try:
            return client.request(method, endpoint, **kwargs)
        except RateLimitError as e:
            if attempt == max_retries - 1:
                raise e
            print(f"Rate limited, waiting {e.retry_after} seconds...")
            time.sleep(e.retry_after)
```

### Best Practices

1. **Monitor Headers**: Always check rate limit headers in responses
2. **Implement Backoff**: Use exponential backoff for retries
3. **Batch Operations**: Combine multiple operations when possible
4. **Cache Results**: Cache frequently accessed data to reduce API calls
5. **Asynchronous Processing**: Use WebSocket for real-time updates instead of polling
6. **Load Balancing**: Distribute requests across multiple API keys if available

### Rate Limit Monitoring

Monitor your API usage through the metrics endpoint:

```bash
# Check current rate limit status
curl -H "X-API-Key: your-api-key" http://localhost:3001/api/metrics | jq '.rate_limits'
```

Expected response:
```json
{
  "rate_limits": {
    "general_api": {
      "limit": 1000,
      "remaining": 850,
      "reset_at": "2024-01-01T00:01:00Z"
    },
    "agent_operations": {
      "limit": 100,
      "remaining": 95,
      "reset_at": "2024-01-01T00:01:00Z"
    }
  }
}
```

## Examples

This section provides comprehensive examples for common API usage patterns, from basic operations to advanced multi-agent coordination scenarios.

### Authentication Setup

All examples assume you have a valid API key. Set it as an environment variable:

```bash
export API_KEY="your-api-key-here"
export BASE_URL="http://localhost:3001"
```

### 1. System Health and Monitoring

#### Basic Health Check
```bash
# Simple health check
curl -H "X-API-Key: $API_KEY" $BASE_URL/health

# Detailed health with all components
curl -H "X-API-Key: $API_KEY" $BASE_URL/health | jq '.'
```

#### System Metrics Monitoring
```bash
# Get current metrics
curl -H "X-API-Key: $API_KEY" $BASE_URL/metrics

# Get resource usage
curl -H "X-API-Key: $API_KEY" $BASE_URL/api/resources

# Get hive status
curl -H "X-API-Key: $API_KEY" $BASE_URL/api/hive/status
```

### 2. Agent Lifecycle Management

#### Creating Agents with Different Specializations

```bash
# Create a basic worker agent
curl -X POST $BASE_URL/api/agents \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "DataProcessor-1",
    "type": "Worker",
    "capabilities": [
      {
        "name": "data_processing",
        "proficiency": 0.8,
        "learning_rate": 0.1
      }
    ]
  }'

# Create a specialist agent
curl -X POST $BASE_URL/api/agents \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "ML-Specialist",
    "type": "Specialist",
    "specialization": "machine_learning",
    "capabilities": [
      {
        "name": "machine_learning",
        "proficiency": 0.9,
        "learning_rate": 0.2
      },
      {
        "name": "data_analysis",
        "proficiency": 0.8,
        "learning_rate": 0.15
      }
    ]
  }'

# Create a coordinator agent
curl -X POST $BASE_URL/api/agents \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "TaskCoordinator",
    "type": "Coordinator",
    "capabilities": [
      {
        "name": "task_coordination",
        "proficiency": 0.95,
        "learning_rate": 0.05
      }
    ]
  }'
```

#### Agent Management Operations

```bash
# List all agents
curl -H "X-API-Key: $API_KEY" "$BASE_URL/api/agents?limit=20&sort_by=created_at&sort_order=desc"

# List only active agents
curl -H "X-API-Key: $API_KEY" "$BASE_URL/api/agents?status=Active&type=Worker"

# Get specific agent details
AGENT_ID="agent-123"
curl -H "X-API-Key: $API_KEY" $BASE_URL/api/agents/$AGENT_ID

# Update agent capabilities
curl -X PUT $BASE_URL/api/agents/$AGENT_ID \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "capabilities": [
      {
        "name": "data_processing",
        "proficiency": 0.95
      },
      {
        "name": "new_capability",
        "proficiency": 0.7,
        "learning_rate": 0.12
      }
    ]
  }'

# Delete an agent (only if no active tasks)
curl -X DELETE $BASE_URL/api/agents/$AGENT_ID \
  -H "X-API-Key: $API_KEY"
```

### 3. Task Management and Execution

#### Creating Tasks with Different Priorities

```bash
# Create a high-priority data processing task
curl -X POST $BASE_URL/api/tasks \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Process quarterly sales data and generate performance report",
    "type": "data_analysis",
    "priority": "High",
    "required_capabilities": [
      {
        "name": "data_processing",
        "min_proficiency": 0.8
      },
      {
        "name": "statistical_analysis",
        "min_proficiency": 0.7
      }
    ],
    "metadata": {
      "data_source": "sales_db",
      "time_range": "Q3_2024",
      "output_format": "pdf_report"
    },
    "estimated_duration_ms": 300000
  }'

# Create a low-priority background task
curl -X POST $BASE_URL/api/tasks \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Clean up old log files older than 30 days",
    "type": "maintenance",
    "priority": "Low",
    "required_capabilities": [
      {
        "name": "file_management",
        "min_proficiency": 0.6
      }
    ],
    "metadata": {
      "retention_days": 30,
      "log_directory": "/var/log/hive"
    }
  }'
```

#### Task Monitoring and Management

```bash
# List all tasks
curl -H "X-API-Key: $API_KEY" "$BASE_URL/api/tasks?limit=50&sort_by=created_at&sort_order=desc"

# List pending tasks by priority
curl -H "X-API-Key: $API_KEY" "$BASE_URL/api/tasks?status=Pending&sort_by=priority&sort_order=desc"

# Get specific task details
TASK_ID="task-456"
curl -H "X-API-Key: $API_KEY" $BASE_URL/api/tasks/$TASK_ID

# Monitor task progress (can be called repeatedly)
while true; do
  TASK_STATUS=$(curl -s -H "X-API-Key: $API_KEY" $BASE_URL/api/tasks/$TASK_ID | jq -r '.data.status')
  PROGRESS=$(curl -s -H "X-API-Key: $API_KEY" $BASE_URL/api/tasks/$TASK_ID | jq -r '.data.progress')

  echo "Task $TASK_ID: $TASK_STATUS (Progress: $PROGRESS)"

  if [ "$TASK_STATUS" = "Completed" ] || [ "$TASK_STATUS" = "Failed" ]; then
    break
  fi

  sleep 5
done

# Cancel a running task
curl -X PUT $BASE_URL/api/tasks/$TASK_ID \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"status": "Cancelled"}'

# Reassign task to different agent
curl -X PUT $BASE_URL/api/tasks/$TASK_ID \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"assigned_agent": "agent-789"}'
```

### 4. Real-time Monitoring with WebSocket

#### JavaScript WebSocket Client

```javascript
class HiveWebSocketClient {
  constructor(apiKey) {
    this.apiKey = apiKey;
    this.ws = null;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
    this.eventHandlers = {};
  }

  connect() {
    const wsUrl = `ws://localhost:3001/ws?api_key=${this.apiKey}`;
    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = (event) => {
      console.log('Connected to AI Orchestrator Hub');
      this.reconnectAttempts = 0;
      this.emit('connected', { timestamp: new Date() });
    };

    this.ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        this.handleMessage(message);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    this.ws.onclose = (event) => {
      console.log('WebSocket disconnected:', event.code, event.reason);
      this.emit('disconnected', { code: event.code, reason: event.reason });
      this.attemptReconnect();
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      this.emit('error', error);
    };
  }

  handleMessage(message) {
    console.log('Received:', message.type, message.data);

    switch (message.type) {
      case 'connection_established':
        this.emit('ready', message.data);
        break;

      case 'agent_created':
        this.emit('agentCreated', message.data);
        break;

      case 'agent_updated':
        this.emit('agentUpdated', message.data);
        break;

      case 'task_created':
        this.emit('taskCreated', message.data);
        break;

      case 'task_assigned':
        this.emit('taskAssigned', message.data);
        break;

      case 'task_progress':
        this.emit('taskProgress', message.data);
        break;

      case 'task_completed':
        this.emit('taskCompleted', message.data);
        break;

      case 'task_failed':
        this.emit('taskFailed', message.data);
        break;

      case 'hive_status':
        this.emit('hiveStatus', message.data);
        break;

      case 'alert_triggered':
        this.emit('alert', message.data);
        break;

      case 'heartbeat':
        // Heartbeat - no action needed
        break;

      default:
        console.log('Unknown message type:', message.type);
    }
  }

  on(event, handler) {
    if (!this.eventHandlers[event]) {
      this.eventHandlers[event] = [];
    }
    this.eventHandlers[event].push(handler);
  }

  emit(event, data) {
    const handlers = this.eventHandlers[event] || [];
    handlers.forEach(handler => {
      try {
        handler(data);
      } catch (error) {
        console.error(`Error in ${event} handler:`, error);
      }
    });
  }

  attemptReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max reconnection attempts reached');
      return;
    }

    this.reconnectAttempts++;
    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);

    console.log(`Attempting reconnection ${this.reconnectAttempts}/${this.maxReconnectAttempts} in ${delay}ms`);

    setTimeout(() => {
      this.connect();
    }, delay);
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
    }
  }
}

// Usage example
const client = new HiveWebSocketClient('your-api-key');

client.on('connected', () => {
  console.log('WebSocket connected successfully');
});

client.on('taskCompleted', (data) => {
  console.log('Task completed:', data.task_id);
  updateTaskStatus(data.task_id, 'completed', data.result);
});

client.on('alert', (data) => {
  if (data.severity === 'critical') {
    showCriticalAlert(data);
  }
});

client.connect();
```

#### Python WebSocket Client

```python
import asyncio
import websockets
import json
import logging
from typing import Dict, Any, Callable

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class HiveWebSocketClient:
    def __init__(self, api_key: str, url: str = "ws://localhost:3001/ws"):
        self.api_key = api_key
        self.url = f"{url}?api_key={api_key}"
        self.websocket = None
        self.event_handlers: Dict[str, list[Callable]] = {}
        self.reconnect_attempts = 0
        self.max_reconnect_attempts = 5

    def on(self, event: str, handler: Callable):
        """Register an event handler"""
        if event not in self.event_handlers:
            self.event_handlers[event] = []
        self.event_handlers[event].append(handler)

    def emit(self, event: str, data: Any):
        """Emit an event to all registered handlers"""
        handlers = self.event_handlers.get(event, [])
        for handler in handlers:
            try:
                handler(data)
            except Exception as e:
                logger.error(f"Error in {event} handler: {e}")

    async def connect(self):
        """Connect to the WebSocket server"""
        try:
            async with websockets.connect(self.url) as websocket:
                self.websocket = websocket
                logger.info("Connected to AI Orchestrator Hub")
                self.reconnect_attempts = 0
                self.emit('connected', {'timestamp': asyncio.get_event_loop().time()})

                async for message in websocket:
                    try:
                        data = json.loads(message)
                        await self.handle_message(data)
                    except json.JSONDecodeError as e:
                        logger.error(f"Failed to parse message: {e}")

        except websockets.exceptions.ConnectionClosed:
            logger.warning("WebSocket connection closed")
            await self.attempt_reconnect()
        except Exception as e:
            logger.error(f"WebSocket error: {e}")
            await self.attempt_reconnect()

    async def handle_message(self, message: Dict[str, Any]):
        """Handle incoming WebSocket messages"""
        msg_type = message.get('type')
        data = message.get('data', {})

        logger.debug(f"Received message: {msg_type}")

        if msg_type == 'connection_established':
            self.emit('ready', data)
        elif msg_type == 'agent_created':
            self.emit('agentCreated', data)
        elif msg_type == 'task_progress':
            self.emit('taskProgress', data)
        elif msg_type == 'task_completed':
            self.emit('taskCompleted', data)
        elif msg_type == 'alert_triggered':
            self.emit('alert', data)
        elif msg_type == 'heartbeat':
            pass  # Ignore heartbeats
        else:
            logger.info(f"Unhandled message type: {msg_type}")

    async def attempt_reconnect(self):
        """Attempt to reconnect with exponential backoff"""
        if self.reconnect_attempts >= self.max_reconnect_attempts:
            logger.error("Max reconnection attempts reached")
            return

        self.reconnect_attempts += 1
        delay = min(2 ** self.reconnect_attempts, 30)  # Exponential backoff, max 30s

        logger.info(f"Reconnecting in {delay} seconds (attempt {self.reconnect_attempts})")
        await asyncio.sleep(delay)
        await self.connect()

# Usage example
async def main():
    client = HiveWebSocketClient("your-api-key")

    @client.on('connected')
    def on_connected(data):
        print("WebSocket connected!")

    @client.on('taskCompleted')
    def on_task_completed(data):
        print(f"Task {data['task_id']} completed with result: {data['result']['summary']}")

    @client.on('alert')
    def on_alert(data):
        if data['severity'] == 'critical':
            print(f"CRITICAL ALERT: {data['message']}")

    await client.connect()

if __name__ == "__main__":
    asyncio.run(main())
```

### 5. MCP Integration Examples

#### Using MCP Tools Programmatically

```bash
# List all available MCP tools
curl -H "X-API-Key: $API_KEY" $BASE_URL/api/mcp/tools

# Create agent via MCP
curl -X POST $BASE_URL/api/mcp/tools/create_swarm_agent/execute \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "parameters": {
      "name": "MCP-Agent",
      "agent_type": "Worker",
      "capabilities": [
        {
          "name": "mcp_integration",
          "proficiency": 0.9
        }
      ]
    }
  }'

# Assign task via MCP
curl -X POST $BASE_URL/api/mcp/tools/assign_swarm_task/execute \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "parameters": {
      "task_description": "Integrate with external MCP-compatible AI assistant",
      "priority": "High",
      "required_capabilities": [
        {
          "name": "mcp_integration",
          "min_proficiency": 0.8
        }
      ]
    }
  }'

# Get swarm status via MCP
curl -X POST $BASE_URL/api/mcp/tools/get_swarm_status/execute \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "parameters": {
      "include_agents": true,
      "include_tasks": true,
      "include_metrics": true
    }
  }'
```

### 6. Advanced Multi-Agent Coordination

#### Creating a Complete Workflow

```bash
#!/bin/bash
# Complete workflow: Create agents -> Assign coordinated tasks -> Monitor progress

API_KEY="your-api-key"
BASE_URL="http://localhost:3001"

echo "=== AI Orchestrator Hub - Multi-Agent Workflow Demo ==="

# 1. Create specialized agents
echo "Creating specialized agents..."

# Data collector agent
COLLECTOR_ID=$(curl -s -X POST $BASE_URL/api/agents \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "DataCollector",
    "type": "Worker",
    "capabilities": [
      {
        "name": "data_collection",
        "proficiency": 0.9
      }
    ]
  }' | jq -r '.data.id')

echo "Created DataCollector: $COLLECTOR_ID"

# Data processor agent
PROCESSOR_ID=$(curl -s -X POST $BASE_URL/api/agents \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "DataProcessor",
    "type": "Worker",
    "capabilities": [
      {
        "name": "data_processing",
        "proficiency": 0.85
      }
    ]
  }' | jq -r '.data.id')

echo "Created DataProcessor: $PROCESSOR_ID"

# Analysis agent
ANALYZER_ID=$(curl -s -X POST $BASE_URL/api/agents \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "DataAnalyzer",
    "type": "Specialist",
    "specialization": "data_analysis",
    "capabilities": [
      {
        "name": "data_analysis",
        "proficiency": 0.95
      }
    ]
  }' | jq -r '.data.id')

echo "Created DataAnalyzer: $ANALYZER_ID"

# 2. Create coordinated tasks
echo "Creating coordinated tasks..."

# Data collection task
COLLECTION_TASK_ID=$(curl -s -X POST $BASE_URL/api/tasks \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Collect customer feedback data from multiple sources",
    "type": "data_collection",
    "priority": "High",
    "required_capabilities": [
      {
        "name": "data_collection",
        "min_proficiency": 0.8
      }
    ],
    "metadata": {
      "sources": ["survey_db", "support_tickets", "social_media"],
      "time_range": "last_30_days"
    }
  }' | jq -r '.data.id')

echo "Created collection task: $COLLECTION_TASK_ID"

# Data processing task (depends on collection)
PROCESSING_TASK_ID=$(curl -s -X POST $BASE_URL/api/tasks \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Clean and normalize collected customer feedback data",
    "type": "data_processing",
    "priority": "High",
    "required_capabilities": [
      {
        "name": "data_processing",
        "min_proficiency": 0.7
      }
    ],
    "metadata": {
      "input_task": "'$COLLECTION_TASK_ID'",
      "normalization_rules": ["remove_duplicates", "standardize_formats", "handle_missing_values"]
    }
  }' | jq -r '.data.id')

echo "Created processing task: $PROCESSING_TASK_ID"

# Analysis task (depends on processing)
ANALYSIS_TASK_ID=$(curl -s -X POST $BASE_URL/api/tasks \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Analyze processed customer feedback for insights and trends",
    "type": "data_analysis",
    "priority": "High",
    "required_capabilities": [
      {
        "name": "data_analysis",
        "min_proficiency": 0.8
      }
    ],
    "metadata": {
      "input_task": "'$PROCESSING_TASK_ID'",
      "analysis_types": ["sentiment_analysis", "topic_modeling", "trend_analysis"],
      "output_format": "comprehensive_report"
    }
  }' | jq -r '.data.id')

echo "Created analysis task: $ANALYSIS_TASK_ID"

# 3. Monitor workflow progress
echo "Monitoring workflow progress..."

while true; do
  echo "=== Workflow Status ==="

  # Check task statuses
  for task_id in $COLLECTION_TASK_ID $PROCESSING_TASK_ID $ANALYSIS_TASK_ID; do
    status=$(curl -s -H "X-API-Key: $API_KEY" $BASE_URL/api/tasks/$task_id | jq -r '.data.status')
    progress=$(curl -s -H "X-API-Key: $API_KEY" $BASE_URL/api/tasks/$task_id | jq -r '.data.progress')
    echo "Task $task_id: $status ($(echo "$progress * 100" | bc -l | xargs printf "%.0f")%)"
  done

  # Check if all tasks are complete
  all_complete=true
  for task_id in $COLLECTION_TASK_ID $PROCESSING_TASK_ID $ANALYSIS_TASK_ID; do
    status=$(curl -s -H "X-API-Key: $API_KEY" $BASE_URL/api/tasks/$task_id | jq -r '.data.status')
    if [ "$status" != "Completed" ]; then
      all_complete=false
      break
    fi
  done

  if [ "$all_complete" = true ]; then
    echo "🎉 All tasks completed successfully!"
    break
  fi

  sleep 10
done

# 4. Get final results
echo "=== Final Results ==="
for task_id in $COLLECTION_TASK_ID $PROCESSING_TASK_ID $ANALYSIS_TASK_ID; do
  result=$(curl -s -H "X-API-Key: $API_KEY" $BASE_URL/api/tasks/$task_id | jq '.data.result')
  echo "Task $task_id result:"
  echo "$result" | jq '.'
  echo "---"
done

echo "Workflow completed!"
```

This comprehensive example demonstrates how to create a complete multi-agent workflow with proper coordination, monitoring, and result aggregation.

## Best Practices and Guidelines

### API Usage Best Practices

#### 1. Error Handling and Resilience
```javascript
// Recommended error handling pattern
async function apiCallWithRetry(endpoint, options = {}, maxRetries = 3) {
  let lastError;

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      const response = await fetch(`/api/${endpoint}`, {
        headers: {
          'X-API-Key': 'your-api-key',
          'Content-Type': 'application/json',
          ...options.headers
        },
        ...options
      });

      if (!response.ok) {
        const errorData = await response.json();

        // Don't retry on client errors (4xx)
        if (response.status >= 400 && response.status < 500) {
          throw new ApiError(response.status, errorData);
        }

        // Retry on server errors (5xx) or rate limiting
        if (attempt === maxRetries) {
          throw new ApiError(response.status, errorData);
        }

        // Exponential backoff
        const delay = Math.min(1000 * Math.pow(2, attempt), 30000);
        await new Promise(resolve => setTimeout(resolve, delay));
        continue;
      }

      return await response.json();

    } catch (error) {
      lastError = error;

      if (attempt === maxRetries) {
        throw lastError;
      }

      // Retry on network errors
      const delay = Math.min(1000 * Math.pow(2, attempt), 30000);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
}
```

#### 2. Rate Limiting Awareness
```javascript
// Rate limit aware request queuing
class RateLimitedApiClient {
  constructor(apiKey) {
    this.apiKey = apiKey;
    this.requestQueue = [];
    this.isProcessing = false;
    this.rateLimitRemaining = 1000; // Assume initial limit
    this.rateLimitReset = Date.now() + 60000; // 1 minute from now
  }

  async request(endpoint, options = {}) {
    return new Promise((resolve, reject) => {
      this.requestQueue.push({ endpoint, options, resolve, reject });
      this.processQueue();
    });
  }

  async processQueue() {
    if (this.isProcessing || this.requestQueue.length === 0) {
      return;
    }

    this.isProcessing = true;

    while (this.requestQueue.length > 0) {
      // Check rate limit
      if (this.rateLimitRemaining <= 0 && Date.now() < this.rateLimitReset) {
        const waitTime = this.rateLimitReset - Date.now();
        console.log(`Rate limited, waiting ${waitTime}ms`);
        await new Promise(resolve => setTimeout(resolve, waitTime));
      }

      const { endpoint, options, resolve, reject } = this.requestQueue.shift();

      try {
        const response = await fetch(`/api/${endpoint}`, {
          headers: {
            'X-API-Key': this.apiKey,
            'Content-Type': 'application/json',
            ...options.headers
          },
          ...options
        });

        // Update rate limit info from headers
        const remaining = response.headers.get('X-RateLimit-Remaining');
        const reset = response.headers.get('X-RateLimit-Reset');

        if (remaining) this.rateLimitRemaining = parseInt(remaining);
        if (reset) this.rateLimitReset = parseInt(reset) * 1000;

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const data = await response.json();
        resolve(data);

      } catch (error) {
        reject(error);
      }
    }

    this.isProcessing = false;
  }
}
```

#### 3. WebSocket Connection Management
```javascript
// Production-ready WebSocket client
class ProductionWebSocketClient {
  constructor(apiKey, options = {}) {
    this.apiKey = apiKey;
    this.url = options.url || 'ws://localhost:3001/ws';
    this.reconnectInterval = options.reconnectInterval || 1000;
    this.maxReconnectAttempts = options.maxReconnectAttempts || 5;
    this.reconnectAttempts = 0;
    this.ws = null;
    this.pingInterval = null;
    this.eventListeners = {};
  }

  connect() {
    try {
      const wsUrl = `${this.url}?api_key=${this.apiKey}`;
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        console.log('WebSocket connected');
        this.reconnectAttempts = 0;
        this.startHeartbeat();
        this.emit('connected');
      };

      this.ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);
          this.handleMessage(message);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };

      this.ws.onclose = (event) => {
        console.log('WebSocket closed:', event.code, event.reason);
        this.stopHeartbeat();
        this.handleReconnect();
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        this.emit('error', error);
      };

    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
      this.handleReconnect();
    }
  }

  handleMessage(message) {
    // Handle heartbeat
    if (message.type === 'heartbeat') {
      return; // Ignore heartbeat, connection is alive
    }

    this.emit(message.type, message.data);
  }

  startHeartbeat() {
    this.pingInterval = setInterval(() => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({ type: 'ping' }));
      }
    }, 30000); // Ping every 30 seconds
  }

  stopHeartbeat() {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }
  }

  handleReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max reconnection attempts reached');
      this.emit('maxReconnectAttemptsReached');
      return;
    }

    this.reconnectAttempts++;
    const delay = Math.min(this.reconnectInterval * Math.pow(2, this.reconnectAttempts), 30000);

    console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);

    setTimeout(() => {
      this.connect();
    }, delay);
  }

  on(event, callback) {
    if (!this.eventListeners[event]) {
      this.eventListeners[event] = [];
    }
    this.eventListeners[event].push(callback);
  }

  emit(event, data) {
    const listeners = this.eventListeners[event] || [];
    listeners.forEach(callback => {
      try {
        callback(data);
      } catch (error) {
        console.error(`Error in ${event} listener:`, error);
      }
    });
  }

  disconnect() {
    this.stopHeartbeat();
    if (this.ws) {
      this.ws.close();
    }
  }
}
```

#### 4. Resource Management
```javascript
// Efficient resource usage patterns
class ResourceManager {
  constructor() {
    this.activeRequests = new Set();
    this.cache = new Map();
    this.cacheExpiry = new Map();
  }

  // Cache with TTL
  async getCached(endpoint, ttl = 30000) { // 30 second default TTL
    const cacheKey = endpoint;
    const now = Date.now();

    if (this.cache.has(cacheKey) && this.cacheExpiry.get(cacheKey) > now) {
      return this.cache.get(cacheKey);
    }

    const data = await this.fetch(endpoint);
    this.cache.set(cacheKey, data);
    this.cacheExpiry.set(cacheKey, now + ttl);

    return data;
  }

  // Request deduplication
  async deduplicatedRequest(endpoint, options = {}) {
    const requestKey = JSON.stringify({ endpoint, options });

    if (this.activeRequests.has(requestKey)) {
      // Wait for existing request to complete
      return new Promise((resolve, reject) => {
        const checkComplete = () => {
          if (!this.activeRequests.has(requestKey)) {
            // Request completed, but we need to make another call
            this.request(endpoint, options).then(resolve).catch(reject);
          } else {
            setTimeout(checkComplete, 100);
          }
        };
        checkComplete();
      });
    }

    this.activeRequests.add(requestKey);

    try {
      const result = await this.request(endpoint, options);
      return result;
    } finally {
      this.activeRequests.delete(requestKey);
    }
  }

  async request(endpoint, options = {}) {
    // Implementation of actual API call
    const response = await fetch(`/api/${endpoint}`, {
      headers: {
        'X-API-Key': 'your-api-key',
        'Content-Type': 'application/json',
        ...options.headers
      },
      ...options
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    return await response.json();
  }

  // Cleanup expired cache entries
  cleanup() {
    const now = Date.now();
    for (const [key, expiry] of this.cacheExpiry) {
      if (expiry <= now) {
        this.cache.delete(key);
        this.cacheExpiry.delete(key);
      }
    }
  }
}

// Auto-cleanup every 5 minutes
setInterval(() => {
  if (window.resourceManager) {
    window.resourceManager.cleanup();
  }
}, 300000);
```

#### 5. Monitoring and Observability
```javascript
// Application monitoring and metrics
class ApplicationMonitor {
  constructor() {
    this.metrics = {
      apiCalls: 0,
      errors: 0,
      websocketMessages: 0,
      averageResponseTime: 0
    };
    this.responseTimes = [];
  }

  recordApiCall(responseTime, success = true) {
    this.metrics.apiCalls++;
    if (!success) this.metrics.errors++;

    this.responseTimes.push(responseTime);
    if (this.responseTimes.length > 100) {
      this.responseTimes.shift(); // Keep only last 100 measurements
    }

    this.metrics.averageResponseTime = this.responseTimes.reduce((a, b) => a + b, 0) / this.responseTimes.length;
  }

  recordWebSocketMessage() {
    this.metrics.websocketMessages++;
  }

  getMetrics() {
    return {
      ...this.metrics,
      errorRate: this.metrics.apiCalls > 0 ? (this.metrics.errors / this.metrics.apiCalls) * 100 : 0
    };
  }

  logMetrics() {
    const metrics = this.getMetrics();
    console.log('Application Metrics:', {
      'API Calls': metrics.apiCalls,
      'Errors': metrics.errors,
      'Error Rate': `${metrics.errorRate.toFixed(2)}%`,
      'Avg Response Time': `${metrics.averageResponseTime.toFixed(2)}ms`,
      'WebSocket Messages': metrics.websocketMessages
    });
  }
}

// Global monitor instance
window.appMonitor = new ApplicationMonitor();

// Monitor all fetch requests
const originalFetch = window.fetch;
window.fetch = async (...args) => {
  const startTime = Date.now();
  try {
    const response = await originalFetch(...args);
    const responseTime = Date.now() - startTime;
    window.appMonitor.recordApiCall(responseTime, response.ok);
    return response;
  } catch (error) {
    const responseTime = Date.now() - startTime;
    window.appMonitor.recordApiCall(responseTime, false);
    throw error;
  }
};
```

## MCP Integration

The AI Orchestrator Hub implements the Model Context Protocol (MCP) for seamless integration with external AI tools and services. MCP enables standardized communication between AI models and tools, allowing external systems to interact with the multi-agent hive through a well-defined protocol.

### MCP Architecture

The MCP implementation provides:
- **Standardized Tool Interface**: Consistent API for tool execution
- **Resource Management**: Access to hive resources and data
- **Real-time Communication**: WebSocket-based event streaming
- **Authentication & Security**: Secure access control and rate limiting
- **Extensibility**: Easy addition of new tools and capabilities

### MCP Endpoints

#### Health Check
```http
GET /api/mcp/health
```

Checks the health and connectivity of the MCP service and underlying hive system.

**Parameters:** None

**Authentication:** API key required

**Rate Limiting:** 100 requests per minute per API key

**Response (Success - 200):**
```json
{
  "service": "mcp-http",
  "status": "healthy",
  "hive_connected": true,
  "mcp_version": "2024-11-05",
  "total_agents": 15,
  "active_agents": 12,
  "total_tools": 12,
  "available_resources": 2,
  "uptime_seconds": 36000,
  "timestamp": "2024-01-01T00:00:00Z",
  "version": "1.0.0",
  "request_id": "req-mcp-health-12345"
}
```

**Examples:**
```bash
# Check MCP service health
curl -X GET http://localhost:3001/api/mcp/health \
  -H "Accept: application/json" \
  -H "X-API-Key: your-api-key-here"
```

#### Execute MCP Request
```http
POST /api/mcp/
```

Executes MCP tools and methods using JSON-RPC 2.0 protocol. This is the primary endpoint for MCP tool execution and resource access.

**Request Body:** JSON-RPC 2.0 request

**Authentication:** API key required

**Rate Limiting:** 200 requests per minute per API key

**Content-Type:** `application/json`

**Available MCP Methods:**
- `initialize`: Initialize MCP connection and capabilities
- `tools/list`: List all available MCP tools
- `tools/call`: Execute a specific tool
- `resources/list`: List available MCP resources
- `resources/read`: Read a specific resource

**Available Tools:**
1. **create_swarm_agent**: Create a new agent with specified capabilities
2. **assign_swarm_task**: Assign a task to the most suitable agent
3. **get_swarm_status**: Get comprehensive hive status and metrics
4. **analyze_with_nlp**: Perform natural language processing analysis
5. **coordinate_agents**: Coordinate multiple agents for complex tasks
6. **list_agents**: List agents with optional filtering
7. **list_tasks**: List tasks with optional filtering
8. **get_agent_details**: Get detailed information about a specific agent
9. **get_task_details**: Get detailed information about a specific task
10. **batch_create_agents**: Create multiple agents in a single operation
11. **echo**: Simple echo tool for testing
12. **system_info**: Get system information

**Examples:**
```bash
# Initialize MCP connection
curl -X POST http://localhost:3001/api/mcp/ \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -d '{
    "jsonrpc": "2.0",
    "id": "init-1",
    "method": "initialize",
    "params": {
      "clientInfo": {
        "name": "MyAIClient",
        "version": "1.0.0"
      }
    }
  }'

# Create a new agent via MCP
curl -X POST http://localhost:3001/api/mcp/ \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -d '{
    "jsonrpc": "2.0",
    "id": "create-agent",
    "method": "tools/call",
    "params": {
      "name": "create_swarm_agent",
      "arguments": {
        "name": "AnalysisAgent",
        "agent_type": "worker",
        "capabilities": [
          {
            "name": "data_analysis",
            "proficiency": 0.9
          }
        ]
      }
    }
  }'

# Get swarm status via MCP
curl -X POST http://localhost:3001/api/mcp/ \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -d '{
    "jsonrpc": "2.0",
    "id": "get-status",
    "method": "tools/call",
    "params": {
      "name": "get_swarm_status",
      "arguments": {
        "include_agents": true,
        "include_tasks": true,
        "include_metrics": true
      }
    }
  }'
```

### Security Considerations

1. **API Key Management**
   - Never expose API keys in client-side code
   - Rotate keys regularly
   - Use different keys for different environments
   - Monitor key usage patterns

2. **Data Validation**
   - Always validate input data on both client and server
   - Use parameterized queries to prevent injection attacks
   - Sanitize user inputs
   - Implement proper CORS policies

3. **Connection Security**
   - Use WSS (WebSocket Secure) in production
   - Implement proper SSL/TLS certificates
   - Validate server certificates
   - Use secure API endpoints (HTTPS)

4. **Rate Limiting Awareness**
   - Monitor your usage against rate limits
   - Implement proper backoff strategies
   - Cache frequently accessed data
   - Use batch operations when possible

### Performance Optimization

1. **Caching Strategies**
   - Cache static data (agent capabilities, system status)
   - Implement intelligent cache invalidation
   - Use appropriate TTL values
   - Consider cache size limits

2. **Batch Operations**
   - Use batch APIs when available
   - Combine multiple small requests
   - Implement request deduplication
   - Balance between latency and throughput

3. **Connection Pooling**
   - Reuse WebSocket connections
   - Implement connection pooling for HTTP requests
   - Monitor connection health
   - Handle connection failures gracefully

4. **Lazy Loading**
   - Load data on demand
   - Implement virtual scrolling for large lists
   - Use pagination effectively
   - Preload critical resources

This API documentation provides a comprehensive reference for integrating with the AI Orchestrator Hub. For additional examples and detailed usage patterns, see the [examples directory](examples/).</content>
</xai:function_call">Create file: /workspaces/ai-orchestrator-hub/docs/api.md
