# Task Management API

The task management endpoints provide comprehensive control over task creation, execution, and monitoring.

## GET /api/tasks

Retrieves information about all tasks in the system.

### Query Parameters

- `status`: Filter by task status (pending, running, completed, failed, cancelled)
- `priority`: Filter by priority level (low, medium, high, critical)
- `agent_id`: Filter by assigned agent
- `type`: Filter by task type
- `limit`: Maximum number of tasks to return (default: 50)
- `offset`: Pagination offset (default: 0)
- `sort_by`: Sort field (created_at, priority, status)
- `sort_order`: Sort order (asc, desc)

### Response

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
        "started_at": "2024-01-01T00:01:00Z",
        "completed_at": "2024-01-01T00:05:00Z",
        "execution_time_ms": 240000,
        "progress": 1.0,
        "result": {
          "satisfaction_score": 8.5,
          "response_rate": 0.75,
          "key_insights": [...]
        },
        "required_capabilities": [
          {
            "name": "data_analysis",
            "min_proficiency": 0.7
          }
        ]
      }
    ],
    "total_count": 1,
    "pagination": {
      "limit": 50,
      "offset": 0,
      "has_more": false
    },
    "summary": {
      "pending": 5,
      "running": 3,
      "completed": 142,
      "failed": 8,
      "cancelled": 2
    }
  }
}
```

### Example Usage

```bash
# Get all tasks
curl http://localhost:3001/api/tasks

# Get pending tasks
curl "http://localhost:3001/api/tasks?status=pending"

# Get high priority tasks
curl "http://localhost:3001/api/tasks?priority=high"

# Get tasks for specific agent
curl "http://localhost:3001/api/tasks?agent_id=uuid-agent-123"

# Paginate and sort
curl "http://localhost:3001/api/tasks?limit=10&offset=20&sort_by=created_at&sort_order=desc"
```

## POST /api/tasks

Creates a new task for execution by agents.

### Request Body

```json
{
  "description": "Generate monthly sales report",
  "type": "reporting",
  "priority": "high",
  "required_capabilities": [
    {
      "name": "data_analysis",
      "min_proficiency": 0.8
    },
    {
      "name": "reporting",
      "min_proficiency": 0.7
    }
  ],
  "parameters": {
    "date_range": "2024-01",
    "include_charts": true,
    "format": "pdf"
  },
  "timeout_seconds": 3600,
  "max_retries": 3,
  "callback_url": "https://example.com/webhook/task-complete"
}
```

### Required Fields

- `description`: Human-readable task description
- `type`: Task type/category
- `priority`: Task priority (low, medium, high, critical)

### Optional Fields

- `required_capabilities`: Array of required agent capabilities
- `parameters`: Task-specific parameters
- `timeout_seconds`: Maximum execution time
- `max_retries`: Maximum retry attempts
- `callback_url`: Webhook URL for completion notification

### Response

```json
{
  "success": true,
  "data": {
    "task_id": "uuid-new-task",
    "message": "Task created successfully",
    "task": {
      "id": "uuid-new-task",
      "description": "Generate monthly sales report",
      "status": "pending",
      "created_at": "2024-01-01T00:00:00Z",
      "estimated_completion": "2024-01-01T00:30:00Z"
    },
    "matching_agents": 3
  }
}
```

### Error Codes

- `VALIDATION_ERROR`: Invalid task configuration
- `TASK_CREATION_FAILED`: Failed to create task
- `NO_SUITABLE_AGENTS`: No agents available with required capabilities

### Example Usage

```bash
# Create a simple task
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Process customer data",
    "type": "data_processing",
    "priority": "medium",
    "required_capabilities": [
      {
        "name": "data_processing",
        "min_proficiency": 0.7
      }
    ]
  }'

# Create a complex task with parameters
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Generate quarterly analytics report",
    "type": "analytics",
    "priority": "high",
    "required_capabilities": [
      {
        "name": "data_analysis",
        "min_proficiency": 0.8
      },
      {
        "name": "visualization",
        "min_proficiency": 0.7
      }
    ],
    "parameters": {
      "quarter": "Q1",
      "year": 2024,
      "include_forecasting": true,
      "output_formats": ["pdf", "xlsx"]
    },
    "timeout_seconds": 7200,
    "callback_url": "https://myapp.com/webhooks/task-complete"
  }'
```

## GET /api/tasks/{id}

Get detailed information about a specific task.

### Parameters

- `id`: Task UUID

### Response

```json
{
  "success": true,
  "data": {
    "task": {
      "id": "uuid-task-1",
      "description": "Analyze customer satisfaction data",
      "type": "data_analysis",
      "priority": 2,
      "status": "running",
      "assigned_agent": "uuid-agent-1",
      "created_at": "2024-01-01T00:00:00Z",
      "started_at": "2024-01-01T00:01:00Z",
      "progress": 0.75,
      "estimated_completion": "2024-01-01T00:25:00Z",
      "execution_time_ms": 180000,
      "required_capabilities": [...],
      "parameters": {...},
      "attempts": [
        {
          "attempt_number": 1,
          "started_at": "2024-01-01T00:01:00Z",
          "agent_id": "uuid-agent-1",
          "status": "running"
        }
      ],
      "result": null,
      "error": null
    },
    "agent_info": {
      "id": "uuid-agent-1",
      "name": "DataAnalyst-1",
      "capabilities": [...]
    }
  }
}
```

### Example Usage

```bash
# Get task details
curl http://localhost:3001/api/tasks/uuid-task-123

# Get task with agent information
curl "http://localhost:3001/api/tasks/uuid-task-123?include=agent"
```

## PUT /api/tasks/{id}

Update task configuration or reassign to different agent.

### Parameters

- `id`: Task UUID

### Request Body

```json
{
  "priority": "critical",
  "parameters": {
    "urgent": true,
    "deadline": "2024-01-01T02:00:00Z"
  },
  "reassign_to_agent": "uuid-agent-2"
}
```

### Response

```json
{
  "success": true,
  "data": {
    "task_id": "uuid-task-1",
    "message": "Task updated successfully",
    "changes": [
      "priority: high → critical",
      "parameters: added urgent flag",
      "agent: uuid-agent-1 → uuid-agent-2"
    ]
  }
}
```

### Example Usage

```bash
# Update task priority
curl -X PUT http://localhost:3001/api/tasks/uuid-task-123 \
  -H "Content-Type: application/json" \
  -d '{"priority": "critical"}'

# Reassign task to different agent
curl -X PUT http://localhost:3001/api/tasks/uuid-task-123 \
  -H "Content-Type: application/json" \
  -d '{"reassign_to_agent": "uuid-agent-456"}'
```

## DELETE /api/tasks/{id}

Cancel a task.

### Parameters

- `id`: Task UUID

### Query Parameters

- `reason`: Cancellation reason

### Response

```json
{
  "success": true,
  "data": {
    "task_id": "uuid-task-1",
    "message": "Task cancelled successfully",
    "status": "cancelled",
    "cancelled_at": "2024-01-01T00:15:00Z"
  }
}
```

### Example Usage

```bash
# Cancel task
curl -X DELETE http://localhost:3001/api/tasks/uuid-task-123

# Cancel with reason
curl -X DELETE "http://localhost:3001/api/tasks/uuid-task-123?reason=duplicate"
```

## POST /api/tasks/{id}/retry

Retry a failed task.

### Parameters

- `id`: Task UUID

### Request Body

```json
{
  "reassign": true,
  "new_parameters": {
    "retry_attempt": 2,
    "use_backup_data": true
  }
}
```

### Response

```json
{
  "success": true,
  "data": {
    "task_id": "uuid-task-1",
    "message": "Task retry initiated",
    "new_attempt_number": 2,
    "assigned_agent": "uuid-agent-2"
  }
}
```

## GET /api/tasks/{id}/logs

Get execution logs for a task.

### Parameters

- `id`: Task UUID

### Query Parameters

- `level`: Log level filter (debug, info, warn, error)
- `limit`: Maximum number of log entries
- `since`: Only logs after this timestamp

### Response

```json
{
  "success": true,
  "data": {
    "task_id": "uuid-task-1",
    "logs": [
      {
        "timestamp": "2024-01-01T00:01:00Z",
        "level": "info",
        "message": "Task started",
        "agent_id": "uuid-agent-1"
      },
      {
        "timestamp": "2024-01-01T00:02:00Z",
        "level": "info",
        "message": "Loading data from database",
        "agent_id": "uuid-agent-1"
      },
      {
        "timestamp": "2024-01-01T00:03:00Z",
        "level": "warn",
        "message": "Slow query detected",
        "agent_id": "uuid-agent-1",
        "details": {
          "query_time_ms": 2500,
          "table": "customer_data"
        }
      }
    ],
    "total_entries": 15,
    "has_more": true
  }
}
```

## POST /api/tasks/bulk/create

Create multiple tasks at once.

### Request Body

```json
{
  "tasks": [
    {
      "description": "Process batch 1",
      "type": "data_processing",
      "priority": "medium",
      "parameters": {"batch_id": 1}
    },
    {
      "description": "Process batch 2",
      "type": "data_processing",
      "priority": "medium",
      "parameters": {"batch_id": 2}
    }
  ]
}
```

### Response

```json
{
  "success": true,
  "data": {
    "tasks_created": 2,
    "task_ids": ["uuid-task-1", "uuid-task-2"],
    "summary": {
      "successful": 2,
      "failed": 0
    }
  }
}
```

## POST /api/tasks/bulk/cancel

Cancel multiple tasks.

### Request Body

```json
{
  "task_ids": ["uuid-task-1", "uuid-task-2"],
  "reason": "batch_cancelled"
}
```

### Response

```json
{
  "success": true,
  "data": {
    "tasks_cancelled": 2,
    "summary": {
      "successful": 2,
      "already_completed": 0,
      "not_found": 0
    }
  }
}
```

## GET /api/tasks/analytics

Get task execution analytics and statistics.

### Query Parameters

- `time_range`: Time range (1h, 24h, 7d, 30d)
- `group_by`: Group results by (agent, type, priority, status)

### Response

```json
{
  "success": true,
  "data": {
    "time_range": "24h",
    "summary": {
      "total_tasks": 150,
      "completed_tasks": 142,
      "failed_tasks": 8,
      "success_rate": 0.947,
      "average_execution_time_ms": 125000,
      "tasks_per_hour": 6.25
    },
    "breakdown": {
      "by_type": {
        "data_processing": {
          "count": 80,
          "success_rate": 0.95,
          "avg_time_ms": 90000
        },
        "analytics": {
          "count": 45,
          "success_rate": 0.93,
          "avg_time_ms": 180000
        }
      },
      "by_priority": {
        "high": {
          "count": 30,
          "success_rate": 0.97,
          "avg_time_ms": 60000
        },
        "medium": {
          "count": 100,
          "success_rate": 0.94,
          "avg_time_ms": 135000
        }
      },
      "by_agent": {
        "uuid-agent-1": {
          "tasks_completed": 25,
          "success_rate": 0.92,
          "avg_time_ms": 120000
        }
      }
    },
    "trends": {
      "task_volume_trend": "increasing",
      "success_rate_trend": "stable",
      "performance_trend": "improving"
    }
  }
}
```

## Task Types

### Data Processing Tasks
Tasks involving data transformation, cleaning, and processing.

```json
{
  "type": "data_processing",
  "description": "Clean and transform customer data",
  "parameters": {
    "source_table": "raw_customers",
    "target_table": "clean_customers",
    "transformations": ["remove_duplicates", "standardize_formats"]
  }
}
```

### Analytics Tasks
Tasks requiring data analysis and insights generation.

```json
{
  "type": "analytics",
  "description": "Generate sales performance report",
  "parameters": {
    "metrics": ["revenue", "conversion_rate", "customer_acquisition"],
    "time_period": "monthly",
    "include_forecasting": true
  }
}
```

### Reporting Tasks
Tasks focused on generating reports and visualizations.

```json
{
  "type": "reporting",
  "description": "Create executive dashboard",
  "parameters": {
    "report_type": "dashboard",
    "include_charts": true,
    "recipients": ["executive@company.com"]
  }
}
```

### Integration Tasks
Tasks involving external system integration.

```json
{
  "type": "integration",
  "description": "Sync data with external CRM",
  "parameters": {
    "external_system": "salesforce",
    "sync_direction": "bidirectional",
    "conflict_resolution": "last_modified_wins"
  }
}
```

## Priority Levels

### Critical (4)
Immediate attention required, system-critical tasks.

### High (3)
Important tasks that should be prioritized.

### Medium (2)
Standard priority tasks.

### Low (1)
Background tasks that can be deferred.

## Best Practices

### Task Creation
1. Use clear, descriptive task descriptions
2. Set appropriate priority levels
3. Define required capabilities accurately
4. Include relevant parameters
5. Set reasonable timeouts

### Task Monitoring
1. Monitor task queues regularly
2. Set up alerts for failed tasks
3. Review execution logs for issues
4. Track performance metrics

### Error Handling
1. Implement proper retry logic
2. Handle partial failures gracefully
3. Provide meaningful error messages
4. Log detailed error information

### Performance Optimization
1. Use appropriate task batching
2. Balance workload across agents
3. Monitor queue depths
4. Optimize task parameters

## Troubleshooting

### Tasks Not Being Assigned
```bash
# Check available agents
curl http://localhost:3001/api/agents?status=idle

# Check agent capabilities
curl http://localhost:3001/api/agents/uuid-agent-123

# Verify task requirements
curl http://localhost:3001/api/tasks/uuid-task-123
```

### Tasks Failing Repeatedly
```bash
# Check task logs
curl http://localhost:3001/api/tasks/uuid-task-123/logs

# Check agent performance
curl http://localhost:3001/api/agents/uuid-agent-123/performance

# Review task parameters
curl http://localhost:3001/api/tasks/uuid-task-123
```

### Queue Backlog
```bash
# Check queue status
curl http://localhost:3001/api/tasks?status=pending

# Get queue analytics
curl http://localhost:3001/api/tasks/analytics

# Scale up agents if needed
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{"name": "Worker-X", "type": "worker", "capabilities": [...]}
```

### Slow Task Execution
```bash
# Check system resources
curl http://localhost:3001/api/resources

# Monitor task performance
curl http://localhost:3001/api/tasks/uuid-task-123

# Check agent workload
curl http://localhost:3001/api/agents/uuid-agent-123/tasks
```

This comprehensive API provides full control over the task lifecycle, from creation to completion, with detailed monitoring and analytics capabilities.