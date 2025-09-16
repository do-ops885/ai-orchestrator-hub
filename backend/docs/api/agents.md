# Agent Management API

The agent management endpoints provide comprehensive control over agents in the system.

## GET /api/agents

Retrieves information about all agents in the hive.

### Query Parameters

- `status`: Filter by agent status (active, idle, failed)
- `type`: Filter by agent type (worker, coordinator, specialist, learner)
- `capability`: Filter by required capability
- `limit`: Maximum number of agents to return (default: 50)
- `offset`: Pagination offset (default: 0)

### Response

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
        "created_at": "2024-01-01T00:00:00Z",
        "last_active": "2024-01-01T00:30:00Z",
        "energy_level": 0.9,
        "social_connections": 3
      }
    ],
    "total_count": 1,
    "pagination": {
      "limit": 50,
      "offset": 0,
      "has_more": false
    }
  }
}
```

### Example Usage

```bash
# Get all agents
curl http://localhost:3001/api/agents

# Get active agents only
curl "http://localhost:3001/api/agents?status=active"

# Get agents with specific capability
curl "http://localhost:3001/api/agents?capability=data_processing"

# Paginate results
curl "http://localhost:3001/api/agents?limit=10&offset=20"
```

## POST /api/agents

Creates a new agent in the hive.

### Request Body

```json
{
  "name": "ContentWriter-1",
  "type": "specialist",
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
  ],
  "initial_energy": 100.0,
  "auto_learning": true
}
```

### Required Fields

- `name`: Unique agent name
- `type`: Agent type (worker, coordinator, specialist, learner)
- `capabilities`: Array of agent capabilities

### Optional Fields

- `initial_energy`: Starting energy level (default: 100.0)
- `auto_learning`: Enable automatic learning (default: true)

### Response

```json
{
  "success": true,
  "data": {
    "agent_id": "uuid-new-agent",
    "message": "Agent created successfully",
    "agent": {
      "id": "uuid-new-agent",
      "name": "ContentWriter-1",
      "type": "specialist",
      "state": "Idle",
      "capabilities": [...],
      "created_at": "2024-01-01T00:00:00Z"
    }
  }
}
```

### Error Codes

- `VALIDATION_ERROR`: Invalid agent configuration
- `AGENT_CREATION_FAILED`: Failed to create agent due to system constraints
- `AGENT_LIMIT_EXCEEDED`: Maximum number of agents reached

### Example Usage

```bash
# Create a worker agent
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Worker-1",
    "type": "worker",
    "capabilities": [
      {
        "name": "general_processing",
        "proficiency": 0.7,
        "learning_rate": 0.1
      }
    ]
  }'

# Create a specialist agent
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "DataScientist-1",
    "type": "specialist",
    "capabilities": [
      {
        "name": "machine_learning",
        "proficiency": 0.9,
        "learning_rate": 0.15
      },
      {
        "name": "data_analysis",
        "proficiency": 0.85,
        "learning_rate": 0.12
      }
    ]
  }'
```

## GET /api/agents/{id}

Get detailed information about a specific agent.

### Parameters

- `id`: Agent UUID

### Response

```json
{
  "success": true,
  "data": {
    "agent": {
      "id": "uuid-1",
      "name": "DataProcessor-1",
      "type": "specialist",
      "state": "Active",
      "capabilities": [
        {
          "name": "data_processing",
          "proficiency": 0.85,
          "learning_rate": 0.1,
          "experience_points": 1250,
          "last_used": "2024-01-01T00:25:00Z"
        }
      ],
      "performance_metrics": {
        "tasks_completed": 25,
        "success_rate": 0.92,
        "average_response_time_ms": 1250,
        "error_rate": 0.08
      },
      "learning_history": {
        "total_experience": 2500,
        "capability_improvements": 5,
        "last_learning_cycle": "2024-01-01T00:30:00Z"
      },
      "social_connections": [
        {
          "agent_id": "uuid-2",
          "relationship_strength": 0.8,
          "collaboration_count": 12
        }
      ],
      "energy_level": 0.9,
      "created_at": "2024-01-01T00:00:00Z",
      "last_active": "2024-01-01T00:30:00Z",
      "uptime_seconds": 1800
    }
  }
}
```

### Example Usage

```bash
# Get agent details
curl http://localhost:3001/api/agents/uuid-1234-5678

# Get agent with expanded metrics
curl "http://localhost:3001/api/agents/uuid-1234-5678?include=metrics,learning,social"
```

## PUT /api/agents/{id}

Update an existing agent's configuration.

### Parameters

- `id`: Agent UUID

### Request Body

```json
{
  "name": "UpdatedAgentName",
  "capabilities": [
    {
      "name": "data_processing",
      "proficiency": 0.9,
      "learning_rate": 0.12
    },
    {
      "name": "new_capability",
      "proficiency": 0.6,
      "learning_rate": 0.15
    }
  ],
  "auto_learning": false
}
```

### Response

```json
{
  "success": true,
  "data": {
    "agent_id": "uuid-1",
    "message": "Agent updated successfully",
    "changes": [
      "name: DataProcessor-1 → UpdatedAgentName",
      "capabilities: added new_capability",
      "auto_learning: true → false"
    ]
  }
}
```

### Example Usage

```bash
# Update agent capabilities
curl -X PUT http://localhost:3001/api/agents/uuid-1234-5678 \
  -H "Content-Type: application/json" \
  -d '{
    "capabilities": [
      {
        "name": "advanced_processing",
        "proficiency": 0.95,
        "learning_rate": 0.08
      }
    ]
  }'
```

## DELETE /api/agents/{id}

Remove an agent from the system.

### Parameters

- `id`: Agent UUID

### Query Parameters

- `force`: Force deletion even if agent has active tasks (default: false)
- `cleanup_data`: Remove all associated data (default: true)

### Response

```json
{
  "success": true,
  "data": {
    "agent_id": "uuid-1",
    "message": "Agent removed successfully",
    "cleanup_summary": {
      "tasks_reassigned": 3,
      "data_removed": true,
      "social_connections_updated": 5
    }
  }
}
```

### Example Usage

```bash
# Remove agent gracefully
curl -X DELETE http://localhost:3001/api/agents/uuid-1234-5678

# Force remove agent
curl -X DELETE "http://localhost:3001/api/agents/uuid-1234-5678?force=true"
```

## POST /api/agents/{id}/pause

Temporarily pause an agent.

### Parameters

- `id`: Agent UUID

### Request Body

```json
{
  "reason": "maintenance",
  "duration_minutes": 30
}
```

### Response

```json
{
  "success": true,
  "data": {
    "agent_id": "uuid-1",
    "status": "paused",
    "resume_at": "2024-01-01T00:30:00Z",
    "active_tasks_reassigned": 2
  }
}
```

## POST /api/agents/{id}/resume

Resume a paused agent.

### Parameters

- `id`: Agent UUID

### Response

```json
{
  "success": true,
  "data": {
    "agent_id": "uuid-1",
    "status": "active",
    "message": "Agent resumed successfully"
  }
}
```

## GET /api/agents/{id}/tasks

Get tasks assigned to a specific agent.

### Parameters

- `id`: Agent UUID

### Query Parameters

- `status`: Filter by task status (pending, running, completed, failed)
- `limit`: Maximum number of tasks to return
- `offset`: Pagination offset

### Response

```json
{
  "success": true,
  "data": {
    "agent_id": "uuid-1",
    "tasks": [
      {
        "id": "task-uuid",
        "description": "Process data",
        "status": "running",
        "assigned_at": "2024-01-01T00:15:00Z",
        "progress": 0.75
      }
    ],
    "total_count": 1
  }
}
```

## POST /api/agents/{id}/learn

Trigger learning cycle for an agent.

### Parameters

- `id`: Agent UUID

### Request Body

```json
{
  "learning_type": "capability_improvement",
  "target_capability": "data_processing",
  "intensity": "high"
}
```

### Response

```json
{
  "success": true,
  "data": {
    "agent_id": "uuid-1",
    "learning_cycle_id": "learn-uuid",
    "status": "started",
    "estimated_completion_minutes": 15
  }
}
```

## GET /api/agents/{id}/performance

Get detailed performance metrics for an agent.

### Parameters

- `id`: Agent UUID

### Query Parameters

- `time_range`: Time range for metrics (1h, 24h, 7d, 30d)

### Response

```json
{
  "success": true,
  "data": {
    "agent_id": "uuid-1",
    "time_range": "24h",
    "metrics": {
      "tasks_completed": 25,
      "success_rate": 0.92,
      "average_response_time_ms": 1250,
      "error_rate": 0.08,
      "capability_improvements": 3,
      "energy_efficiency": 0.85,
      "collaboration_score": 0.78
    },
    "trends": {
      "performance_trend": "improving",
      "response_time_trend": "stable",
      "error_rate_trend": "decreasing"
    }
  }
}
```

## Bulk Operations

### POST /api/agents/bulk/create

Create multiple agents at once.

### Request Body

```json
{
  "agents": [
    {
      "name": "Worker-1",
      "type": "worker",
      "capabilities": [...]
    },
    {
      "name": "Worker-2",
      "type": "worker",
      "capabilities": [...]
    }
  ]
}
```

### POST /api/agents/bulk/delete

Delete multiple agents.

### Request Body

```json
{
  "agent_ids": ["uuid-1", "uuid-2", "uuid-3"],
  "force": false
}
```

## Agent Types

### Worker Agents
General-purpose agents that can handle various tasks.

```json
{
  "type": "worker",
  "capabilities": [
    {
      "name": "general_processing",
      "proficiency": 0.7
    }
  ]
}
```

### Specialist Agents
Domain-specific agents with advanced capabilities.

```json
{
  "type": "specialist",
  "capabilities": [
    {
      "name": "machine_learning",
      "proficiency": 0.9
    },
    {
      "name": "data_science",
      "proficiency": 0.85
    }
  ]
}
```

### Coordinator Agents
Agents that manage and coordinate other agents.

```json
{
  "type": "coordinator",
  "capabilities": [
    {
      "name": "task_distribution",
      "proficiency": 0.95
    },
    {
      "name": "resource_management",
      "proficiency": 0.9
    }
  ]
}
```

### Learner Agents
Agents focused on continuous learning and adaptation.

```json
{
  "type": "learner",
  "capabilities": [
    {
      "name": "pattern_recognition",
      "proficiency": 0.8
    },
    {
      "name": "adaptive_learning",
      "proficiency": 0.85
    }
  ]
}
```

## Best Practices

### Agent Creation
1. Use descriptive names that indicate agent purpose
2. Set appropriate proficiency levels based on agent experience
3. Include multiple capabilities for flexibility
4. Enable auto-learning for continuous improvement

### Agent Management
1. Monitor agent performance regularly
2. Update capabilities as agents learn new skills
3. Balance workload across agents
4. Remove underperforming agents when necessary

### Capability Management
1. Start with realistic proficiency levels
2. Adjust learning rates based on agent performance
3. Add new capabilities as needed
4. Monitor capability utilization

### Troubleshooting

#### Agent Not Responding
```bash
# Check agent status
curl http://localhost:3001/api/agents/uuid-123

# Check agent tasks
curl http://localhost:3001/api/agents/uuid-123/tasks

# Check system resources
curl http://localhost:3001/api/resources
```

#### Poor Performance
```bash
# Get performance metrics
curl http://localhost:3001/api/agents/uuid-123/performance

# Check system metrics
curl http://localhost:3001/metrics

# Adjust agent configuration
curl -X PUT http://localhost:3001/api/agents/uuid-123 \
  -H "Content-Type: application/json" \
  -d '{"learning_rate": 0.08}'
```

#### High Error Rates
```bash
# Check agent error logs
curl "http://localhost:3001/api/agents/uuid-123/tasks?status=failed"

# Update agent capabilities
curl -X PUT http://localhost:3001/api/agents/uuid-123 \
  -H "Content-Type: application/json" \
  -d '{"capabilities": [...]}
```

This comprehensive API provides full control over agent lifecycle, capabilities, and performance monitoring.