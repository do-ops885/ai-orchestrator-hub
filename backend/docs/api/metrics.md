# Metrics API

The metrics endpoints provide comprehensive performance monitoring and system analytics.

## GET /metrics

Returns current system metrics and performance trends from all modular subsystems.

### Response

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

### Response Fields

- `architecture`: System architecture type
- `current_metrics`: Real-time metrics from each module
- `trends`: Performance trends over time
- `collection_timestamp`: When metrics were collected
- `modular_benefits`: Benefits of modular architecture

## GET /api/modules/{module_name}/metrics

Returns detailed metrics for a specific module.

### Parameters

- `module_name`: Name of the module (coordinator, agent_management, task_management, background_processes, metrics_collection)

### Response

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
      },
      "capability_distribution": {
        "data_processing": 3,
        "analytics": 2,
        "reporting": 1
      }
    },
    "timestamp": "2024-01-01T00:00:00Z",
    "time_range": "last_24_hours"
  }
}
```

### Example Usage

```bash
# Get all system metrics
curl http://localhost:3001/metrics

# Get agent management metrics
curl http://localhost:3001/api/modules/agent_management/metrics

# Get task management metrics
curl http://localhost:3001/api/modules/task_management/metrics

# Monitor metrics over time
while true; do
  curl -s http://localhost:3001/metrics | jq '.data.current_metrics.agent_management.total_agents'
  sleep 60
done
```

## GET /metrics/export/{format}

Exports metrics in different formats for external monitoring systems.

### Parameters

- `format`: Export format (json, prometheus, graphite, influxdb)

### Prometheus Format Response

```
# HELP ai_orchestrator_agents_total Total number of agents
# TYPE ai_orchestrator_agents_total gauge
ai_orchestrator_agents_total 5

# HELP ai_orchestrator_agents_active Number of active agents
# TYPE ai_orchestrator_agents_active gauge
ai_orchestrator_agents_active 3

# HELP ai_orchestrator_tasks_completed_total Total tasks completed
# TYPE ai_orchestrator_tasks_completed_total counter
ai_orchestrator_tasks_completed_total 142

# HELP ai_orchestrator_cpu_usage_percent CPU usage percentage
# TYPE ai_orchestrator_cpu_usage_percent gauge
ai_orchestrator_cpu_usage_percent 45.2

# HELP ai_orchestrator_memory_usage_percent Memory usage percentage
# TYPE ai_orchestrator_memory_usage_percent gauge
ai_orchestrator_memory_usage_percent 62.8
```

### Example Usage

```bash
# Export as Prometheus format
curl http://localhost:3001/metrics/export/prometheus

# Export as JSON
curl http://localhost:3001/metrics/export/json

# Export as InfluxDB line protocol
curl http://localhost:3001/metrics/export/influxdb
```

## GET /metrics/history

Returns historical metrics data for trend analysis.

### Query Parameters

- `start_time`: Start time for historical data (ISO 8601 format)
- `end_time`: End time for historical data (ISO 8601 format)
- `interval`: Aggregation interval (1m, 5m, 1h, 1d)
- `metrics`: Comma-separated list of metrics to include

### Response

```json
{
  "success": true,
  "data": {
    "time_range": {
      "start": "2024-01-01T00:00:00Z",
      "end": "2024-01-01T01:00:00Z",
      "interval": "5m"
    },
    "metrics": {
      "agent_count": [
        {"timestamp": "2024-01-01T00:00:00Z", "value": 3},
        {"timestamp": "2024-01-01T00:05:00Z", "value": 4},
        {"timestamp": "2024-01-01T00:10:00Z", "value": 5}
      ],
      "task_completion_rate": [
        {"timestamp": "2024-01-01T00:00:00Z", "value": 0.92},
        {"timestamp": "2024-01-01T00:05:00Z", "value": 0.94},
        {"timestamp": "2024-01-01T00:10:00Z", "value": 0.96}
      ],
      "cpu_usage": [
        {"timestamp": "2024-01-01T00:00:00Z", "value": 45.2},
        {"timestamp": "2024-01-01T00:05:00Z", "value": 48.1},
        {"timestamp": "2024-01-01T00:10:00Z", "value": 52.3}
      ]
    }
  }
}
```

### Example Usage

```bash
# Get last hour of data with 5-minute intervals
curl "http://localhost:3001/metrics/history?interval=5m&start_time=2024-01-01T00:00:00Z"

# Get specific metrics
curl "http://localhost:3001/metrics/history?metrics=agent_count,task_completion_rate"
```

## POST /metrics/alerts

Creates custom metric alerts and thresholds.

### Request Body

```json
{
  "alerts": [
    {
      "name": "high_cpu_usage",
      "metric": "cpu_usage_percent",
      "condition": "greater_than",
      "threshold": 80.0,
      "duration_minutes": 5,
      "severity": "warning",
      "channels": ["email", "webhook"]
    },
    {
      "name": "low_task_success_rate",
      "metric": "task_success_rate",
      "condition": "less_than",
      "threshold": 0.9,
      "duration_minutes": 10,
      "severity": "critical",
      "channels": ["slack", "webhook"]
    }
  ]
}
```

### Response

```json
{
  "success": true,
  "data": {
    "alerts_created": 2,
    "alert_ids": ["alert-123", "alert-456"]
  }
}
```

## GET /metrics/alerts

Retrieves current alert status and history.

### Response

```json
{
  "success": true,
  "data": {
    "active_alerts": [
      {
        "id": "alert-123",
        "name": "high_cpu_usage",
        "status": "active",
        "triggered_at": "2024-01-01T00:15:00Z",
        "current_value": 85.2,
        "threshold": 80.0,
        "severity": "warning"
      }
    ],
    "resolved_alerts": [
      {
        "id": "alert-456",
        "name": "memory_usage",
        "status": "resolved",
        "triggered_at": "2024-01-01T00:10:00Z",
        "resolved_at": "2024-01-01T00:12:00Z",
        "peak_value": 92.1,
        "threshold": 90.0
      }
    ],
    "alert_summary": {
      "total_active": 1,
      "by_severity": {
        "critical": 0,
        "warning": 1,
        "info": 0
      }
    }
  }
}
```

## Metrics Configuration

Configure metrics collection using environment variables:

```env
# Metrics collection settings
HIVE_MONITORING__METRICS_COLLECTION_INTERVAL_MS=5000
HIVE_MONITORING__METRICS_RETENTION_DAYS=30

# Alert thresholds
HIVE_MONITORING__CPU_ALERT_THRESHOLD=80.0
HIVE_MONITORING__MEMORY_ALERT_THRESHOLD=85.0
HIVE_MONITORING__DISK_ALERT_THRESHOLD=90.0

# Export settings
HIVE_MONITORING__ENABLE_PROMETHEUS_EXPORT=true
HIVE_MONITORING__PROMETHEUS_PORT=9090
```

## Integration Examples

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'ai-orchestrator'
    static_configs:
      - targets: ['localhost:3001']
    metrics_path: '/metrics/export/prometheus'
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "AI Orchestrator Hub",
    "panels": [
      {
        "title": "Agent Count",
        "type": "graph",
        "targets": [
          {
            "expr": "ai_orchestrator_agents_total",
            "legendFormat": "Total Agents"
          }
        ]
      },
      {
        "title": "Task Success Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "ai_orchestrator_task_success_rate",
            "legendFormat": "Success Rate"
          }
        ]
      }
    ]
  }
}
```

### Custom Monitoring Script

```python
import requests
import time
from prometheus_client import CollectorRegistry, Gauge, push_to_gateway

# Create metrics
registry = CollectorRegistry()
agent_count = Gauge('ai_orchestrator_agents_total', 'Total number of agents', registry=registry)
task_rate = Gauge('ai_orchestrator_task_completion_rate', 'Task completion rate', registry=registry)

while True:
    # Fetch metrics from API
    response = requests.get('http://localhost:3001/metrics')
    data = response.json()['data']['current_metrics']

    # Update Prometheus metrics
    agent_count.set(data['agent_management']['total_agents'])
    task_rate.set(data['task_management']['task_success_rate'])

    # Push to Prometheus Pushgateway
    push_to_gateway('localhost:9091', job='ai-orchestrator', registry=registry)

    time.sleep(60)
```

## Troubleshooting Metrics

### Common Issues

1. **Missing Metrics**:
   ```bash
   # Check if metrics collection is enabled
   curl http://localhost:3001/metrics | jq '.data'
   ```

2. **High Cardinality**:
   ```env
   # Reduce metrics granularity
   HIVE_MONITORING__METRICS_COLLECTION_INTERVAL_MS=10000
   ```

3. **Storage Issues**:
   ```bash
   # Check metrics storage size
   curl http://localhost:3001/metrics | jq '.data.current_metrics.metrics_collection.storage_size_mb'
   ```

4. **Performance Impact**:
   ```env
   # Optimize metrics collection
   HIVE_MONITORING__ENABLE_HIGH_FREQUENCY_METRICS=false
   ```

### Best Practices

1. **Monitor Key Metrics**: Focus on business-critical metrics
2. **Set Appropriate Thresholds**: Avoid alert fatigue
3. **Use Historical Data**: Analyze trends over time
4. **Automate Responses**: Set up automated remediation
5. **Regular Review**: Update dashboards and alerts regularly