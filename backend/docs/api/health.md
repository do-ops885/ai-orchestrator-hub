# Health Check API

The health check endpoints provide comprehensive system health monitoring and diagnostics.

## GET /health

Provides comprehensive health check information about all system components.

### Response

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

### Response Fields

- `status`: Overall system health ("healthy", "degraded", "unhealthy")
- `timestamp`: Current server timestamp
- `response_time_ms`: API response time in milliseconds
- `version`: System version
- `architecture`: System architecture type
- `modules`: Health status of each system module
- `system_info`: Additional system information

### Module Health Status

Each module reports:
- `status`: Module health ("healthy", "degraded", "unhealthy")
- `uptime_seconds`: Module uptime
- Module-specific metrics (varies by module)

### HTTP Status Codes

- `200`: System is healthy
- `503`: System is unhealthy or degraded

### Example Usage

```bash
# Basic health check
curl http://localhost:3001/health

# Check specific module health
curl http://localhost:3001/health | jq '.data.modules.agent_management.status'

# Monitor health in a loop
while true; do
  curl -s http://localhost:3001/health | jq '.data.status'
  sleep 30
done
```

## GET /api/modules/status

Returns the status of all modular subsystems.

### Response

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

### Response Fields

- `modules`: Status of each module
- `inter_module_communication`: Communication metrics between modules

## GET /api/modules/{module_name}/health

Returns detailed health information for a specific module.

### Parameters

- `module_name`: Name of the module (coordinator, agent_management, task_management, background_processes, metrics_collection)

### Response

```json
{
  "success": true,
  "data": {
    "module": "agent_management",
    "status": "healthy",
    "health_score": 0.95,
    "last_check": "2024-01-01T00:00:00Z",
    "checks": {
      "agent_creation": {
        "status": "healthy",
        "response_time_ms": 25,
        "success_rate": 0.98
      },
      "agent_monitoring": {
        "status": "healthy",
        "active_agents": 5,
        "idle_agents": 2
      },
      "memory_usage": {
        "status": "healthy",
        "usage_percent": 45.2,
        "available_mb": 1024
      }
    },
    "recommendations": []
  }
}
```

### Example Usage

```bash
# Check agent management module health
curl http://localhost:3001/api/modules/agent_management/health

# Check all modules
for module in coordinator agent_management task_management background_processes metrics_collection; do
  echo "=== $module ==="
  curl -s http://localhost:3001/api/modules/$module/health | jq '.data.status'
done
```

## Health Check Configuration

Configure health check behavior using environment variables:

```env
# Health check intervals
HIVE_MONITORING__HEALTH_CHECK_INTERVAL_SECS=30

# Health check timeouts
HIVE_MONITORING__HEALTH_CHECK_TIMEOUT_SECS=10

# Module-specific thresholds
HIVE_MONITORING__AGENT_HEALTH_THRESHOLD=0.8
HIVE_MONITORING__TASK_SUCCESS_THRESHOLD=0.9
```

## Troubleshooting Health Issues

### Common Health Check Failures

1. **Database Connection Issues**:
   ```bash
   # Check database connectivity
   curl http://localhost:3001/health | jq '.data.modules.database.status'
   ```

2. **High Resource Usage**:
   ```bash
   # Check resource usage
   curl http://localhost:3001/api/resources
   ```

3. **Module Communication Issues**:
   ```bash
   # Check inter-module communication
   curl http://localhost:3001/api/modules/status | jq '.data.inter_module_communication'
   ```

### Health Check Best Practices

1. **Monitor Regularly**: Set up automated health monitoring
2. **Alert on Failures**: Configure alerts for health check failures
3. **Check Dependencies**: Ensure all dependencies are healthy
4. **Review Logs**: Check logs when health checks fail

### Load Balancer Health Checks

For load balancer configuration, use the `/health` endpoint:

```nginx
# Nginx configuration
location /health {
    proxy_pass http://backend:3001/health;
    proxy_connect_timeout 5s;
    proxy_send_timeout 5s;
    proxy_read_timeout 5s;
}
```

```haproxy
# HAProxy configuration
backend ai_orchestrator
    balance roundrobin
    option httpchk GET /health
    http-check expect status 200
    server backend1 backend1:3001 check
    server backend2 backend2:3001 check
```