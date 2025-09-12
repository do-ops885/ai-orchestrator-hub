# Multiagent Hive API Documentation

This document provides comprehensive documentation for the Multiagent Hive System REST API and WebSocket endpoints.

## Overview

The API provides programmatic access to manage agents, tasks, and monitor the hive system. All endpoints return standardized JSON responses with consistent error handling.

### Modular Architecture

The Multiagent Hive System has been refactored into a modular architecture with 6 focused modules:

1. **`coordinator.rs`** - Core coordination logic and main system interface
2. **`agent_management.rs`** - Agent lifecycle management, registration, and monitoring
3. **`task_management.rs`** - Task distribution, execution coordination, and work-stealing queues
4. **`background_processes.rs`** - Background process management (learning cycles, swarm coordination, metrics collection)
5. **`metrics_collection.rs`** - Comprehensive metrics collection, aggregation, and reporting
6. **`mod.rs`** - Module organization, exports, and integration testing

This modular design improves maintainability, testability, and allows for independent development of each subsystem while maintaining backward compatibility.

### Architecture Benefits

- **Separation of Concerns**: Each module handles a specific aspect of the system
- **Improved Testability**: Modules can be tested independently with focused unit tests
- **Better Performance**: Specialized modules optimize their specific functionality
- **Enhanced Monitoring**: Detailed metrics collection from each subsystem
- **Scalability**: Modules can be scaled independently based on workload
- **Backward Compatibility**: Existing API endpoints remain unchanged

### Module API Reference

#### Agent Management Module (`agent_management.rs`)

**Public Methods:**
- `new(resource_manager, coordination_tx)` - Create new agent manager
- `create_agent(config)` - Create and register new agent
- `remove_agent(agent_id)` - Remove agent from system
- `get_agent(agent_id)` - Get agent by ID
- `get_all_agents()` - Get all active agents
- `update_agent_metrics(agent_id, execution_time, success)` - Update agent performance metrics
- `get_status()` - Get agent status summary
- `get_analytics()` - Get detailed agent analytics
- `run_learning_cycle(nlp_processor)` - Run learning cycle for all agents
- `get_agent_count()` - Get total agent count

**Agent Configuration Format:**
```json
{
  "type": "worker|coordinator|specialist|learner",
  "name": "AgentName",
  "capabilities": [
    {
      "name": "capability_name",
      "proficiency": 0.0-1.0,
      "learning_rate": 0.0-1.0
    }
  ]
}
```

#### Task Management Module (`task_management.rs`)

**Public Methods:**
- `new(resource_manager, coordination_tx)` - Create new task distributor
- `create_task(config)` - Create and queue new task
- `execute_task_with_verification(task_id, agent_id)` - Execute task with verification
- `distribute_tasks(agents)` - Distribute tasks to available agents
- `get_status()` - Get task status summary
- `get_analytics()` - Get detailed task analytics

**Task Configuration Format:**
```json
{
  "type": "computation|data_analysis|reporting",
  "title": "Task Title",
  "description": "Task Description",
  "priority": "low|medium|high|critical",
  "required_capabilities": [
    {
      "name": "capability_name",
      "minimum_proficiency": 0.0-1.0
    }
  ]
}
```

#### Background Processes Module (`background_processes.rs`)

**Public Methods:**
- `new(coordination_tx)` - Create new process manager
- `start_all_processes(agent_manager, task_distributor, metrics_collector, resource_manager)` - Start all background processes
- `stop_all_processes()` - Stop all background processes
- `update_config(new_config)` - Update process configuration
- `get_process_status()` - Get current process status

**Process Configuration:**
```rust
ProcessConfig {
    work_stealing_interval: Duration,
    learning_interval: Duration,
    swarm_coordination_interval: Duration,
    metrics_collection_interval: Duration,
    resource_monitoring_interval: Duration,
}
```

#### Metrics Collection Module (`metrics_collection.rs`)

**Public Methods:**
- `new(coordination_tx)` - Create new metrics collector
- `record_agent_event(event_type, agent_id)` - Record agent lifecycle events
- `record_task_completion(task_id, agent_id, success)` - Record task completion
- `update_metrics(new_metrics)` - Update system metrics
- `collect_periodic_metrics()` - Collect periodic metrics snapshot
- `get_current_metrics()` - Get current metrics
- `get_enhanced_metrics()` - Get enhanced metrics with trends
- `get_metrics_summary()` - Get metrics summary for dashboard
- `reset_daily_counters()` - Reset daily counters
- `export_metrics(format)` - Export metrics (json|prometheus)

#### Coordinator Module (`coordinator.rs`)

**Public Methods:**
- `new()` - Create new hive coordinator
- `start()` - Start coordinator and background processes
- `create_agent(config)` - Create new agent
- `remove_agent(agent_id)` - Remove agent
- `get_agent(agent_id)` - Get agent by ID
- `get_all_agents()` - Get all agents
- `create_task(config)` - Create new task
- `get_status()` - Get comprehensive system status
- `get_enhanced_analytics()` - Get detailed analytics
- `execute_task_with_verification(task_id, agent_id)` - Execute task with verification
- `shutdown()` - Gracefully shutdown coordinator
- `get_agents_info()` - Get agent information
- `get_tasks_info()` - Get task information
- `get_resource_info()` - Get resource information
- `get_memory_stats()` - Get memory statistics
- `check_queue_health()` - Check queue health
- `check_agent_health()` - Check agent health

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

Provides comprehensive health check information about all system components, now including detailed status for each modular subsystem.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "timestamp": "2024-01-01T00:00:00Z",
    "response_time_ms": 15,
    "version": "2.0.0",
    "architecture": "modular",
    "modules": {
      "coordinator": {
        "status": "healthy",
        "uptime_seconds": 3600,
        "active_connections": 12
      },
      "agent_management": {
        "status": "healthy",
        "total_agents": 5,
        "active_agents": 3,
        "agent_creation_rate": 2.5
      },
      "task_management": {
        "status": "healthy",
        "queue_size": 8,
        "tasks_per_second": 15.2,
        "success_rate": 0.94
      },
      "background_processes": {
        "status": "healthy",
        "active_processes": 5,
        "process_uptime_seconds": 3600
      },
      "metrics_collection": {
        "status": "healthy",
        "metrics_per_second": 25.0,
        "storage_size_mb": 45.2
      },
      "resource_manager": {
        "status": "healthy",
        "memory_usage_percent": 65.2,
        "cpu_usage_percent": 45.8,
        "available_memory_mb": 2048,
        "cpu_cores": 8
      }
    },
    "system_info": {
      "cpu_native": true,
      "gpu_optional": true,
      "phase_3_active": true,
      "modular_architecture": true,
      "swarm_cohesion": 0.82,
      "learning_progress": 0.71
    }
  }
}
```

### GET /metrics

Returns current system metrics and performance trends from all modular subsystems.

**Response:**
```json
{
  "success": true,
  "data": {
    "architecture": "modular",
    "current_metrics": {
      "coordinator": {
        "active_connections": 12,
        "requests_per_second": 8.3,
        "average_response_time_ms": 25.5,
        "error_rate_per_minute": 0.01
      },
      "agent_management": {
        "total_agents": 5,
        "active_agents": 3,
        "idle_agents": 2,
        "failed_agents": 0,
        "average_agent_performance": 0.82,
        "agent_creation_rate": 2.5,
        "agent_removal_rate": 0.2
      },
      "task_management": {
        "total_tasks_submitted": 150,
        "total_tasks_completed": 142,
        "total_tasks_failed": 3,
        "tasks_in_queue": 5,
        "average_task_duration_ms": 1250,
        "task_success_rate": 94.7,
        "work_stealing_efficiency": 0.89
      },
      "background_processes": {
        "active_processes": 5,
        "process_cpu_usage_percent": 15.2,
        "process_memory_usage_mb": 45.8,
        "learning_cycles_completed": 24,
        "swarm_coordination_updates": 180
      },
      "metrics_collection": {
        "metrics_per_second": 25.0,
        "storage_size_mb": 45.2,
        "retention_days": 30,
        "export_formats_supported": ["json", "prometheus"]
      },
      "system": {
        "cpu_usage_percent": 45.2,
        "memory_usage_percent": 62.8,
        "disk_usage_percent": 34.1,
        "network_throughput_mbps": 125.5
      }
    },
    "trends": {
      "agent_growth_trend": "increasing",
      "task_completion_trend": "stable",
      "performance_trend": "improving",
      "resource_usage_trend": "stable"
    },
    "collection_timestamp": "2024-01-01T00:00:00Z",
    "modular_benefits": {
      "separation_of_concerns": true,
      "independent_scaling": true,
      "improved_monitoring": true,
      "better_testability": true
    }
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

## Modular System Endpoints

### GET /api/modules/status

Returns the status of all modular subsystems.

**Response:**
```json
{
  "success": true,
  "data": {
    "modules": {
      "coordinator": {
        "status": "healthy",
        "version": "2.0.0",
        "uptime_seconds": 3600
      },
      "agent_management": {
        "status": "healthy",
        "agents_managed": 5,
        "performance_score": 0.85
      },
      "task_management": {
        "status": "healthy",
        "tasks_processed": 142,
        "queue_efficiency": 0.94
      },
      "background_processes": {
        "status": "healthy",
        "processes_running": 5,
        "last_learning_cycle": "2024-01-01T00:30:00Z"
      },
      "metrics_collection": {
        "status": "healthy",
        "metrics_collected": 25000,
        "storage_efficiency": 0.92
      }
    },
    "inter_module_communication": {
      "messages_per_second": 45.2,
      "coordination_efficiency": 0.98,
      "error_rate": 0.001
    }
  }
}
```

### GET /api/modules/{module_name}/metrics

Returns detailed metrics for a specific module.

**Parameters:**
- `module_name`: Name of the module (coordinator, agent_management, task_management, background_processes, metrics_collection)

**Example:** `GET /api/modules/agent_management/metrics`

**Response:**
```json
{
  "success": true,
  "data": {
    "module": "agent_management",
    "metrics": {
      "total_agents": 5,
      "active_agents": 3,
      "agent_lifecycle_events": 12,
      "performance_distribution": {
        "excellent": 2,
        "good": 2,
        "needs_improvement": 1
      },
      "resource_utilization": {
        "cpu_percent": 15.2,
        "memory_mb": 45.8
      }
    },
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

### POST /api/modules/{module_name}/config

Updates configuration for a specific module.

**Parameters:**
- `module_name`: Name of the module to configure

**Request Body:**
```json
{
  "config": {
    "enabled": true,
    "interval_seconds": 30,
    "thresholds": {
      "warning": 70,
      "critical": 90
    }
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "module": "background_processes",
    "config_updated": true,
    "restart_required": false
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
      "capabilities": [...],
      "module": "agent_management"
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
    "timestamp": "2024-01-01T00:00:00Z",
    "module": "agent_management"
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
      "status": "pending",
      "module": "task_management"
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
    "result": { ... },
    "module": "task_management"
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
    "retry_count": 2,
    "module": "task_management"
  }
}
```

#### Modular System Events

```json
{
  "type": "module_status_changed",
  "data": {
    "module": "background_processes",
    "old_status": "healthy",
    "new_status": "degraded",
    "reason": "High CPU usage in learning process",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

```json
{
  "type": "inter_module_message",
  "data": {
    "from_module": "task_management",
    "to_module": "metrics_collection",
    "message_type": "TaskCompleted",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

```json
{
  "type": "coordination_event",
  "data": {
    "event_type": "ResourceAlert",
    "module": "coordinator",
    "details": {
      "resource": "CPU",
      "usage": 0.95,
      "threshold": 0.9
    },
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

#### System Events

```json
{
  "type": "hive_status_update",
  "data": {
    "metrics": { ... },
    "alerts": [...],
    "modular_health": {
      "all_modules_healthy": true,
      "degraded_modules": [],
      "module_coordination_efficiency": 0.98
    }
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
    "affected_modules": ["background_processes", "task_management"],
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

```json
{
  "type": "modular_performance_alert",
  "data": {
    "level": "info",
    "title": "Modular Architecture Benefits",
    "description": "System performance improved by 15% due to modular design",
    "metrics": {
      "response_time_improvement": 0.15,
      "resource_efficiency_gain": 0.12,
      "scalability_improvement": 0.18
    },
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

## Practical Examples

### Creating and Managing Agents

#### Basic Agent Creation
```javascript
const axios = require('axios');

async function createWorkerAgent() {
    try {
        const response = await axios.post('http://localhost:3001/api/agents', {
            name: 'DataProcessor-1',
            type: 'worker',
            capabilities: [
                {
                    name: 'data_processing',
                    proficiency: 0.85,
                    learning_rate: 0.1
                },
                {
                    name: 'analytics',
                    proficiency: 0.75,
                    learning_rate: 0.12
                }
            ]
        });

        console.log('Agent created:', response.data);
        return response.data.data.agent_id;
    } catch (error) {
        console.error('Error creating agent:', error.response.data);
    }
}
```

#### Creating a Specialist Agent
```javascript
async function createSpecialistAgent() {
    try {
        const response = await axios.post('http://localhost:3001/api/agents', {
            name: 'ContentWriter-1',
            type: 'specialist',
            capabilities: [
                {
                    name: 'content_writing',
                    proficiency: 0.9,
                    learning_rate: 0.08
                },
                {
                    name: 'editing',
                    proficiency: 0.85,
                    learning_rate: 0.1
                },
                {
                    name: 'seo_optimization',
                    proficiency: 0.7,
                    learning_rate: 0.15
                }
            ]
        });

        console.log('Specialist agent created:', response.data);
    } catch (error) {
        console.error('Error creating specialist:', error.response.data);
    }
}
```

#### Retrieving Agent Information
```javascript
async function getAllAgents() {
    try {
        const response = await axios.get('http://localhost:3001/api/agents');
        console.log('All agents:', response.data.data.agents);

        // Get specific agent details
        const agentId = response.data.data.agents[0].id;
        const agentResponse = await axios.get(`http://localhost:3001/api/agents/${agentId}`);
        console.log('Agent details:', agentResponse.data);
    } catch (error) {
        console.error('Error retrieving agents:', error.response.data);
    }
}
```

### Task Creation and Execution

#### Creating Tasks with Different Priorities
```javascript
async function createTasks() {
    const tasks = [
        {
            description: "Process customer data for analytics",
            type: "data_analysis",
            priority: 2,
            required_capabilities: [
                {
                    name: "data_processing",
                    min_proficiency: 0.7
                }
            ]
        },
        {
            description: "Generate monthly sales report",
            type: "reporting",
            priority: 1,
            required_capabilities: [
                {
                    name: "analytics",
                    min_proficiency: 0.8
                }
            ]
        },
        {
            description: "Optimize database queries",
            type: "optimization",
            priority: 3,
            required_capabilities: [
                {
                    name: "database_optimization",
                    min_proficiency: 0.9
                }
            ]
        }
    ];

    for (const task of tasks) {
        try {
            const response = await axios.post('http://localhost:3001/api/tasks', task);
            console.log('Task created:', response.data.data.task_id);
        } catch (error) {
            console.error('Error creating task:', error.response.data);
        }
    }
}
```

#### Monitoring Task Execution
```javascript
async function monitorTasks() {
    try {
        const response = await axios.get('http://localhost:3001/api/tasks');
        const tasks = response.data.data.tasks;

        console.log('Task Status Summary:');
        const statusCounts = {};
        tasks.forEach(task => {
            statusCounts[task.status] = (statusCounts[task.status] || 0) + 1;
        });

        console.log('Status counts:', statusCounts);

        // Get detailed task analytics
        const analyticsResponse = await axios.get('http://localhost:3001/api/tasks/analytics');
        console.log('Task analytics:', analyticsResponse.data);
    } catch (error) {
        console.error('Error monitoring tasks:', error.response.data);
    }
}
```

### Metrics Collection and Analysis

#### Retrieving System Metrics
```javascript
async function getSystemMetrics() {
    try {
        const response = await axios.get('http://localhost:3001/metrics');
        const metrics = response.data.data;

        console.log('System Metrics:');
        console.log('- Agent metrics:', metrics.current_metrics.agent_management);
        console.log('- Task metrics:', metrics.current_metrics.task_management);
        console.log('- System performance:', metrics.current_metrics.system);

        // Check trends
        console.log('Performance trends:', metrics.trends);
    } catch (error) {
        console.error('Error retrieving metrics:', error.response.data);
    }
}
```

#### Exporting Metrics for External Monitoring
```javascript
async function exportMetrics() {
    try {
        // Export as JSON
        const jsonResponse = await axios.get('http://localhost:3001/api/metrics/export/json');
        console.log('JSON Metrics:', jsonResponse.data);

        // Export as Prometheus format
        const prometheusResponse = await axios.get('http://localhost:3001/api/metrics/export/prometheus');
        console.log('Prometheus Metrics:', prometheusResponse.data);
    } catch (error) {
        console.error('Error exporting metrics:', error.response.data);
    }
}
```

### Background Process Management

#### Monitoring Background Processes
```javascript
async function monitorBackgroundProcesses() {
    try {
        const response = await axios.get('http://localhost:3001/api/modules/status');
        const modules = response.data.data.modules;

        console.log('Background Process Status:');
        console.log('- Learning cycles:', modules.background_processes.last_learning_cycle);
        console.log('- Active processes:', modules.background_processes.processes_running);
        console.log('- Swarm coordination:', modules.background_processes.swarm_coordination_updates);
    } catch (error) {
        console.error('Error monitoring processes:', error.response.data);
    }
}
```

#### Configuring Process Intervals
```javascript
async function updateProcessConfig() {
    try {
        const newConfig = {
            config: {
                enabled: true,
                interval_seconds: 15, // Reduced from default 30
                thresholds: {
                    warning: 75,
                    critical: 90
                }
            }
        };

        const response = await axios.post('http://localhost:3001/api/modules/background_processes/config', newConfig);
        console.log('Process config updated:', response.data);
    } catch (error) {
        console.error('Error updating config:', error.response.data);
    }
}
```

## Migration Guide

### From Monolithic to Modular Architecture

#### Code Changes Required

**1. Agent Management Migration**
```javascript
// OLD: Direct agent creation
const agent = await hive.createAgent({ type: 'worker', name: 'Agent1' });

// NEW: Use modular agent management
const agentManager = hive.getModule('agent_management');
const agent = await agentManager.createAgent({
    type: 'worker',
    name: 'Agent1',
    capabilities: [
        { name: 'data_processing', proficiency: 0.8, learning_rate: 0.1 }
    ]
});
```

**2. Task Distribution Migration**
```javascript
// OLD: Simple task creation
const taskId = await hive.createTask({ description: 'Process data' });

// NEW: Enhanced task configuration with capabilities
const taskDistributor = hive.getModule('task_management');
const taskId = await taskDistributor.createTask({
    description: 'Process customer data',
    type: 'data_analysis',
    priority: 'high',
    required_capabilities: [
        { name: 'data_processing', min_proficiency: 0.7 }
    ]
});
```

**3. Metrics Collection Migration**
```javascript
// OLD: Basic metrics
const metrics = await hive.getMetrics();

// NEW: Enhanced modular metrics
const metricsCollector = hive.getModule('metrics_collection');
const currentMetrics = await metricsCollector.getCurrentMetrics();
const trends = await metricsCollector.getEnhancedMetrics();
const summary = await metricsCollector.getMetricsSummary();
```

#### Configuration Updates

**Environment Variables:**
```bash
# OLD
HIVE_MAX_AGENTS=100
HIVE_TASK_TIMEOUT=300

# NEW: Module-specific configuration
HIVE_AGENT_MANAGER_MAX_AGENTS=100
HIVE_TASK_DISTRIBUTOR_TIMEOUT=300
HIVE_METRICS_COLLECTION_INTERVAL=10
HIVE_BACKGROUND_PROCESSES_LEARNING_INTERVAL=30
```

**Configuration File:**
```json
{
  "modules": {
    "agent_management": {
      "max_agents": 100,
      "learning_enabled": true,
      "performance_tracking": true
    },
    "task_management": {
      "work_stealing_enabled": true,
      "max_queue_size": 1000,
      "execution_timeout_seconds": 300
    },
    "background_processes": {
      "learning_interval_seconds": 30,
      "swarm_coordination_interval_seconds": 5,
      "resource_monitoring_interval_seconds": 5
    },
    "metrics_collection": {
      "collection_interval_seconds": 10,
      "retention_days": 30,
      "export_formats": ["json", "prometheus"]
    }
  }
}
```

#### API Endpoint Changes

| Old Endpoint | New Modular Endpoint | Purpose |
|-------------|---------------------|---------|
| `GET /agents` | `GET /api/agents` | List all agents |
| `POST /agents` | `POST /api/agents` | Create agent |
| `GET /tasks` | `GET /api/tasks` | List all tasks |
| `POST /tasks` | `POST /api/tasks` | Create task |
| `GET /metrics` | `GET /metrics` | System metrics |
| `GET /health` | `GET /health` | Health check |
| `GET /status` | `GET /api/hive/status` | System status |

#### Backward Compatibility

The modular architecture maintains full backward compatibility:

- All existing API endpoints continue to work
- Existing client code requires no changes
- Legacy configuration formats are supported
- Migration can be done incrementally

#### Performance Improvements

**Before Migration:**
- Single-threaded processing
- Limited scalability
- Basic metrics collection
- Manual resource management

**After Migration:**
- Multi-threaded modular processing
- Independent scaling per module
- Comprehensive metrics and analytics
- Automated resource optimization
- Work-stealing task distribution
- Background process management

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
import asyncio
import websockets

class HiveClient:
    def __init__(self, base_url='http://localhost:3001'):
        self.base_url = base_url
        self.session = requests.Session()

    def create_agent(self, agent_config):
        """Create a new agent with modular configuration"""
        try:
            response = self.session.post(
                f'{self.base_url}/api/agents',
                json=agent_config,
                headers={'Content-Type': 'application/json'}
            )

            if response.status_code == 200:
                result = response.json()
                print(f'Agent created: {result["data"]["agent_id"]}')
                return result["data"]["agent_id"]
            else:
                print(f'Error creating agent: {response.json()}')
                return None

        except requests.exceptions.RequestException as e:
            print(f'Request failed: {e}')
            return None

    def create_task(self, task_config):
        """Create a new task with modular task management"""
        try:
            response = self.session.post(
                f'{self.base_url}/api/tasks',
                json=task_config,
                headers={'Content-Type': 'application/json'}
            )

            if response.status_code == 200:
                result = response.json()
                print(f'Task created: {result["data"]["task_id"]}')
                return result["data"]["task_id"]
            else:
                print(f'Error creating task: {response.json()}')
                return None

        except requests.exceptions.RequestException as e:
            print(f'Request failed: {e}')
            return None

    def get_system_metrics(self):
        """Retrieve comprehensive system metrics"""
        try:
            response = self.session.get(f'{self.base_url}/metrics')
            if response.status_code == 200:
                return response.json()["data"]
            else:
                print(f'Error getting metrics: {response.json()}')
                return None
        except requests.exceptions.RequestException as e:
            print(f'Request failed: {e}')
            return None

    def get_module_status(self, module_name):
        """Get status of a specific module"""
        try:
            response = self.session.get(f'{self.base_url}/api/modules/{module_name}/metrics')
            if response.status_code == 200:
                return response.json()["data"]
            else:
                print(f'Error getting module status: {response.json()}')
                return None
        except requests.exceptions.RequestException as e:
            print(f'Request failed: {e}')
            return None

# Usage examples
def main():
    client = HiveClient()

    # Create a worker agent
    agent_config = {
        "name": "PythonWorker-1",
        "type": "worker",
        "capabilities": [
            {
                "name": "data_processing",
                "proficiency": 0.85,
                "learning_rate": 0.1
            },
            {
                "name": "analytics",
                "proficiency": 0.75,
                "learning_rate": 0.12
            }
        ]
    }

    agent_id = client.create_agent(agent_config)

    # Create a task
    task_config = {
        "description": "Process sales data with analytics",
        "type": "data_analysis",
        "priority": 2,
        "required_capabilities": [
            {
                "name": "data_processing",
                "min_proficiency": 0.7
            }
        ]
    }

    task_id = client.create_task(task_config)

    # Get system metrics
    metrics = client.get_system_metrics()
    if metrics:
        print("System Metrics:")
        print(f"- Agent count: {metrics['current_metrics']['agent_management']['total_agents']}")
        print(f"- Task success rate: {metrics['current_metrics']['task_management']['success_rate']}")

    # Get specific module metrics
    agent_metrics = client.get_module_status('agent_management')
    if agent_metrics:
        print(f"Agent module metrics: {agent_metrics}")

async def websocket_client():
    """WebSocket client for real-time updates"""
    uri = "ws://localhost:3001/ws"

    async with websockets.connect(uri) as websocket:
        print("Connected to Hive WebSocket")

        # Listen for messages
        async for message in websocket:
            data = json.loads(message)
            print(f"Received: {data}")

            # Handle different message types
            if data['type'] == 'agent_created':
                print(f"New agent: {data['data']['agent']['name']}")
            elif data['type'] == 'task_completed':
                print(f"Task completed: {data['data']['task_id']}")
            elif data['type'] == 'system_alert':
                print(f"System alert: {data['data']['title']}")

if __name__ == "__main__":
    main()
    # Uncomment to run WebSocket client
    # asyncio.run(websocket_client())
```

### WebSocket JavaScript Client

```javascript
class ModularHiveWebSocketClient {
    constructor(url = 'ws://localhost:3001/ws') {
        this.url = url;
        this.ws = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.messageHandlers = new Map();
        this.setupDefaultHandlers();
    }

    setupDefaultHandlers() {
        // Agent-related events
        this.messageHandlers.set('agent_created', (data) => {
            console.log('🆕 New agent created:', data.agent.name);
            console.log('   Module: agent_management');
            console.log('   Capabilities:', data.agent.capabilities);
        });

        this.messageHandlers.set('agent_status_changed', (data) => {
            console.log('🔄 Agent status changed:', data.agent_id);
            console.log(`   ${data.old_status} → ${data.new_status}`);
        });

        // Task-related events
        this.messageHandlers.set('task_created', (data) => {
            console.log('📋 New task created:', data.task.description);
            console.log('   Module: task_management');
            console.log('   Priority:', data.task.priority);
        });

        this.messageHandlers.set('task_completed', (data) => {
            console.log('✅ Task completed:', data.task_id);
            console.log('   Agent:', data.agent_id);
            console.log('   Execution time:', data.execution_time_ms, 'ms');
        });

        this.messageHandlers.set('task_failed', (data) => {
            console.error('❌ Task failed:', data.task_id);
            console.error('   Agent:', data.agent_id);
            console.error('   Error:', data.error);
            console.error('   Retry count:', data.retry_count);
        });

        // Modular system events
        this.messageHandlers.set('module_status_changed', (data) => {
            console.log(`📊 Module ${data.module} status: ${data.old_status} → ${data.new_status}`);
            if (data.reason) {
                console.log('   Reason:', data.reason);
            }
        });

        this.messageHandlers.set('inter_module_message', (data) => {
            console.log('📨 Inter-module communication:');
            console.log(`   ${data.from_module} → ${data.to_module}`);
            console.log('   Message type:', data.message_type);
        });

        // System events
        this.messageHandlers.set('system_alert', (data) => {
            const level = data.level.toUpperCase();
            console.log(`🚨 ${level} ALERT: ${data.title}`);
            console.log('   Description:', data.description);
            if (data.affected_modules) {
                console.log('   Affected modules:', data.affected_modules.join(', '));
            }
        });

        this.messageHandlers.set('modular_performance_alert', (data) => {
            console.log('⚡ Performance improvement detected:');
            console.log(`   Response time: ${data.metrics.response_time_improvement * 100}% better`);
            console.log(`   Resource efficiency: ${data.metrics.resource_efficiency_gain * 100}% better`);
        });

        this.messageHandlers.set('coordination_event', (data) => {
            console.log('🎯 Coordination event:', data.event_type);
            console.log('   Module:', data.module);
            console.log('   Details:', data.details);
        });
    }

    connect() {
        try {
            console.log('🔌 Connecting to Hive WebSocket...');
            this.ws = new WebSocket(this.url);

            this.ws.onopen = () => {
                console.log('✅ Connected to Hive WebSocket');
                this.reconnectAttempts = 0;
            };

            this.ws.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    this.handleMessage(message);
                } catch (error) {
                    console.error('❌ Failed to parse WebSocket message:', error);
                }
            };

            this.ws.onclose = (event) => {
                console.log('🔌 WebSocket connection closed:', event.code, event.reason);
                this.attemptReconnect();
            };

            this.ws.onerror = (error) => {
                console.error('❌ WebSocket error:', error);
            };

        } catch (error) {
            console.error('❌ Failed to connect:', error);
        }
    }

    handleMessage(message) {
        const handler = this.messageHandlers.get(message.type);
        if (handler) {
            handler(message.data);
        } else {
            console.log('❓ Unknown message type:', message.type);
            console.log('   Data:', message.data);
        }

        // Log timestamp for all messages
        console.log(`   🕒 ${new Date(message.timestamp).toLocaleTimeString()}`);
    }

    attemptReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);

            console.log(`🔄 Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts}) in ${delay}ms...`);

            setTimeout(() => {
                this.connect();
            }, delay);
        } else {
            console.error('❌ Max reconnection attempts reached. Giving up.');
        }
    }

    disconnect() {
        if (this.ws) {
            console.log('🔌 Disconnecting from Hive WebSocket...');
            this.ws.close();
        }
    }

    // Add custom message handler
    on(eventType, handler) {
        this.messageHandlers.set(eventType, handler);
    }

    // Remove message handler
    off(eventType) {
        this.messageHandlers.delete(eventType);
    }
}

// Usage with modular event handling
const client = new ModularHiveWebSocketClient();
client.connect();

// Add custom handler for specific events
client.on('custom_module_event', (data) => {
    console.log('🎨 Custom module event:', data);
});

// Handle connection and cleanup
process.on('SIGINT', () => {
    console.log('\n🛑 Shutting down...');
    client.disconnect();
    process.exit(0);
});
```

## Versioning

The API follows semantic versioning:

- **v1.0**: Initial release with basic agent and task management
- **v2.0**: Enhanced with neural processing, verification systems, and advanced monitoring
- **v2.1 (Modular)**: Refactored into modular architecture with 6 focused modules:
  - Improved separation of concerns
  - Enhanced testability and maintainability
  - Better performance monitoring per module
  - Independent scaling capabilities
  - Backward compatibility maintained
- **v3.0 (Phase 3)**: Complete modular refactoring with enhanced capabilities:
  - 6 specialized modules with independent operation
  - Advanced work-stealing task distribution
  - Comprehensive metrics collection and analytics
  - Background process management
  - Real-time modular event streaming
  - Enhanced agent learning and adaptation

## Module-Specific API Versions

Each module maintains its own version for independent updates:

- **agent_management**: v1.2.0 - Enhanced learning cycles and performance tracking
- **task_management**: v1.3.0 - Work-stealing queues and advanced distribution
- **background_processes**: v1.1.0 - Swarm coordination and resource monitoring
- **metrics_collection**: v1.4.0 - Multi-format export and trend analysis
- **coordinator**: v2.0.0 - Unified modular interface
- **mod.rs**: v1.0.0 - Module organization and exports

## Support

For API support or questions:

### Health and Diagnostics
- **System Health**: `GET /health` (includes modular subsystem status)
- **Module Status**: `GET /api/modules/status`
- **Module Metrics**: `GET /api/modules/{module_name}/metrics`
- **Comprehensive Metrics**: `GET /metrics` (includes all module metrics)

### Troubleshooting
- **Agent Issues**: Check agent_management module metrics and logs
- **Task Issues**: Monitor task_management queue health and distribution
- **Performance Issues**: Review metrics_collection trends and background_processes
- **System Alerts**: Subscribe to WebSocket events for real-time notifications

### Real-time Monitoring
- **WebSocket Events**: `ws://localhost:3001/ws` for real-time modular updates
- **Event Types**: agent_created, task_completed, module_status_changed, system_alert
- **Modular Events**: Inter-module communication and coordination events

### Documentation Resources
- **Module Documentation**: Each module has detailed inline documentation
- **API Examples**: Comprehensive examples for all major use cases
- **Migration Guide**: Step-by-step guide for upgrading from monolithic architecture
- **Configuration Guide**: Module-specific configuration options and best practices

### Getting Help
1. Check the `/health` endpoint for system status
2. Review WebSocket events for real-time diagnostics
3. Examine module-specific metrics for detailed insights
4. Consult the migration guide for architecture questions
5. Review system logs with modular context information

### Best Practices
- **Monitoring**: Use WebSocket events for real-time system monitoring
- **Metrics**: Leverage comprehensive metrics for performance optimization
- **Configuration**: Tune module-specific settings for optimal performance
- **Scaling**: Scale modules independently based on workload requirements
- **Updates**: Update modules independently using semantic versioning
