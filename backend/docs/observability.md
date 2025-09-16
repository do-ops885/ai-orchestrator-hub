# Observability Guide

This guide covers monitoring, logging, metrics collection, and observability best practices for the AI Orchestrator Hub.

## Overview

The AI Orchestrator Hub provides comprehensive observability features including:

- **Structured Logging**: JSON-formatted logs with consistent schema
- **Metrics Collection**: Real-time performance and business metrics
- **Health Checks**: Component health monitoring and diagnostics
- **Tracing**: Request tracing and performance profiling
- **Dashboards**: Real-time visualization and alerting

## Logging

### Log Levels

The system supports standard log levels:

- `TRACE`: Detailed execution flow (development only)
- `DEBUG`: Debug information and detailed operations
- `INFO`: General information and normal operations
- `WARN`: Warning conditions that don't affect operation
- `ERROR`: Error conditions that may affect operation

### Structured Logging

All logs follow a consistent JSON format:

```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "level": "info",
  "component": "agent_management",
  "message": "Agent created successfully",
  "agent_id": "uuid-agent-123",
  "request_id": "uuid-request-456",
  "duration_ms": 150,
  "user_id": "user-789"
}
```

### Log Configuration

```env
# Logging configuration
HIVE_LOGGING__LEVEL=info
HIVE_LOGGING__FORMAT=json
HIVE_LOGGING__ENABLE_CONSOLE_LOGGING=true
HIVE_LOGGING__ENABLE_FILE_LOGGING=true
HIVE_LOGGING__LOG_FILE_PATH=./logs/hive.log
HIVE_LOGGING__MAX_FILE_SIZE_MB=100
HIVE_LOGGING__LOG_ROTATION=daily
HIVE_LOGGING__MAX_LOG_FILES=30
```

### Log Categories

#### Application Logs

```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "level": "info",
  "component": "coordinator",
  "message": "Task assigned to agent",
  "task_id": "uuid-task-123",
  "agent_id": "uuid-agent-456",
  "priority": 2
}
```

#### Security Logs

```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "level": "warn",
  "component": "security",
  "message": "Rate limit exceeded",
  "client_ip": "192.168.1.100",
  "endpoint": "/api/agents",
  "limit": 1000,
  "window_seconds": 60
}
```

#### Performance Logs

```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "level": "info",
  "component": "performance",
  "message": "Task execution completed",
  "task_id": "uuid-task-123",
  "execution_time_ms": 2500,
  "memory_usage_mb": 45.2,
  "cpu_usage_percent": 15.8
}
```

#### Error Logs

```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "level": "error",
  "component": "task_executor",
  "message": "Task execution failed",
  "task_id": "uuid-task-123",
  "error_code": "EXECUTION_TIMEOUT",
  "error_details": {
    "timeout_seconds": 300,
    "execution_time_seconds": 320
  },
  "stack_trace": "..."
}
```

## Metrics

### System Metrics

#### CPU and Memory

```json
{
  "cpu_usage_percent": 45.2,
  "memory_usage_percent": 62.8,
  "memory_used_mb": 1024,
  "memory_total_mb": 16384,
  "cpu_cores": 8,
  "load_average_1m": 2.5,
  "load_average_5m": 2.2,
  "load_average_15m": 2.0
}
```

#### Disk and Network

```json
{
  "disk_usage_percent": 34.1,
  "disk_used_gb": 136.4,
  "disk_total_gb": 400,
  "network_rx_bytes": 1524000,
  "network_tx_bytes": 987000,
  "network_connections": 45
}
```

### Application Metrics

#### Agent Metrics

```json
{
  "total_agents": 5,
  "active_agents": 3,
  "idle_agents": 2,
  "failed_agents": 0,
  "agent_creation_rate": 2.5,
  "agent_removal_rate": 0.2,
  "average_agent_performance": 0.82,
  "agent_uptime_hours": 24.5
}
```

#### Task Metrics

```json
{
  "total_tasks_submitted": 150,
  "total_tasks_completed": 142,
  "total_tasks_failed": 8,
  "total_tasks_pending": 5,
  "task_success_rate": 0.947,
  "average_task_duration_ms": 125000,
  "tasks_per_second": 0.015,
  "task_queue_depth": 3
}
```

#### Performance Metrics

```json
{
  "requests_per_second": 8.3,
  "average_response_time_ms": 25.5,
  "error_rate_per_minute": 0.01,
  "throughput_mbps": 125.5,
  "cache_hit_rate": 0.85,
  "database_connection_pool_usage": 0.6
}
```

### Business Metrics

```json
{
  "active_users": 25,
  "total_projects": 12,
  "completed_workflows": 89,
  "average_workflow_duration_hours": 2.5,
  "user_satisfaction_score": 4.2,
  "system_availability_percent": 99.9
}
```

## Health Checks

### Application Health

```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T00:00:00Z",
  "version": "2.0.0",
  "uptime_seconds": 86400,
  "response_time_ms": 15
}
```

### Component Health

```json
{
  "coordinator": {
    "status": "healthy",
    "uptime_seconds": 86400,
    "active_connections": 12,
    "last_health_check": "2024-01-01T00:00:00Z"
  },
  "agent_management": {
    "status": "healthy",
    "total_agents": 5,
    "active_agents": 3,
    "last_health_check": "2024-01-01T00:00:00Z"
  },
  "task_management": {
    "status": "healthy",
    "queue_size": 8,
    "tasks_per_second": 15.2,
    "last_health_check": "2024-01-01T00:00:00Z"
  },
  "database": {
    "status": "healthy",
    "connection_pool_size": 10,
    "active_connections": 3,
    "last_health_check": "2024-01-01T00:00:00Z"
  }
}
```

### Dependency Health

```json
{
  "database": {
    "status": "healthy",
    "response_time_ms": 5,
    "connection_count": 8
  },
  "cache": {
    "status": "healthy",
    "hit_rate": 0.85,
    "size_mb": 256
  },
  "external_services": {
    "openai_api": {
      "status": "healthy",
      "response_time_ms": 150,
      "rate_limit_remaining": 95
    }
  }
}
```

## Tracing

### Request Tracing

```json
{
  "trace_id": "uuid-trace-123",
  "span_id": "uuid-span-456",
  "parent_span_id": "uuid-span-455",
  "operation": "create_task",
  "start_time": "2024-01-01T00:00:00Z",
  "duration_ms": 250,
  "tags": {
    "user_id": "user-789",
    "priority": "high",
    "agent_count": 3
  },
  "events": [
    {
      "timestamp": "2024-01-01T00:00:00Z",
      "event": "task_validation_started"
    },
    {
      "timestamp": "2024-01-01T00:00:05Z",
      "event": "agent_matching_completed",
      "attributes": {
        "matched_agents": 3,
        "best_match_score": 0.95
      }
    }
  ]
}
```

### Performance Tracing

```json
{
  "trace_id": "uuid-trace-789",
  "operation": "execute_task",
  "total_duration_ms": 2500,
  "spans": [
    {
      "span_id": "span-1",
      "operation": "load_task_data",
      "duration_ms": 150,
      "attributes": {
        "data_size_kb": 25,
        "cache_hit": true
      }
    },
    {
      "span_id": "span-2",
      "operation": "process_data",
      "duration_ms": 1800,
      "attributes": {
        "algorithm": "neural_processing",
        "cpu_usage_percent": 75
      }
    },
    {
      "span_id": "span-3",
      "operation": "save_results",
      "duration_ms": 550,
      "attributes": {
        "result_size_kb": 15,
        "compression_ratio": 0.8
      }
    }
  ]
}
```

## Dashboards

### System Overview Dashboard

```json
{
  "dashboard": {
    "title": "AI Orchestrator Hub - System Overview",
    "refresh": "30s",
    "panels": [
      {
        "title": "Active Agents",
        "type": "graph",
        "targets": [
          {
            "expr": "ai_orchestrator_agents_active",
            "legendFormat": "Active Agents"
          }
        ],
        "thresholds": [
          {"value": 1, "color": "red"},
          {"value": 5, "color": "yellow"},
          {"value": 10, "color": "green"}
        ]
      },
      {
        "title": "Task Success Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "ai_orchestrator_task_success_rate * 100",
            "legendFormat": "Success Rate %"
          }
        ]
      },
      {
        "title": "System CPU Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "ai_orchestrator_cpu_usage_percent",
            "legendFormat": "CPU Usage %"
          }
        ]
      }
    ]
  }
}
```

### Agent Performance Dashboard

```json
{
  "dashboard": {
    "title": "Agent Performance",
    "variables": [
      {
        "name": "agent_id",
        "query": "ai_orchestrator_agents_list",
        "type": "query"
      }
    ],
    "panels": [
      {
        "title": "Agent Task Completion Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "ai_orchestrator_agent_task_completion_rate{agent_id='$agent_id'}",
            "legendFormat": "Completion Rate"
          }
        ]
      },
      {
        "title": "Agent Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "ai_orchestrator_agent_response_time_ms{agent_id='$agent_id'}",
            "legendFormat": "Response Time (ms)"
          }
        ]
      }
    ]
  }
}
```

### Task Analytics Dashboard

```json
{
  "dashboard": {
    "title": "Task Analytics",
    "panels": [
      {
        "title": "Task Queue Depth",
        "type": "graph",
        "targets": [
          {
            "expr": "ai_orchestrator_task_queue_depth",
            "legendFormat": "Queue Depth"
          }
        ]
      },
      {
        "title": "Task Duration by Type",
        "type": "heatmap",
        "targets": [
          {
            "expr": "ai_orchestrator_task_duration_ms",
            "legendFormat": "{{type}}"
          }
        ]
      },
      {
        "title": "Task Failure Reasons",
        "type": "piechart",
        "targets": [
          {
            "expr": "ai_orchestrator_task_failures_total",
            "legendFormat": "{{reason}}"
          }
        ]
      }
    ]
  }
}
```

## Alerting

### Metric-Based Alerts

```json
{
  "alerts": [
    {
      "name": "high_cpu_usage",
      "condition": "cpu_usage_percent > 80",
      "duration": "5m",
      "severity": "warning",
      "channels": ["email", "slack"],
      "message": "CPU usage is above 80% for 5 minutes"
    },
    {
      "name": "low_task_success_rate",
      "condition": "task_success_rate < 0.9",
      "duration": "10m",
      "severity": "critical",
      "channels": ["email", "slack", "webhook"],
      "message": "Task success rate dropped below 90%"
    },
    {
      "name": "agent_failures",
      "condition": "agent_failure_rate > 0.05",
      "duration": "15m",
      "severity": "error",
      "channels": ["email"],
      "message": "Agent failure rate exceeded 5%"
    }
  ]
}
```

### Log-Based Alerts

```json
{
  "log_alerts": [
    {
      "name": "database_connection_errors",
      "pattern": "ERROR.*database.*connection",
      "rate": "5 per minute",
      "severity": "critical",
      "channels": ["email", "slack"]
    },
    {
      "name": "security_violations",
      "pattern": "WARN.*security.*violation",
      "rate": "1 per hour",
      "severity": "warning",
      "channels": ["email"]
    }
  ]
}
```

## Integration with Monitoring Tools

### Prometheus

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'ai-orchestrator'
    static_configs:
      - targets: ['localhost:3001']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'ai-orchestrator-health'
    static_configs:
      - targets: ['localhost:3001']
    metrics_path: '/health'
    scrape_interval: 30s
```

### Grafana

```json
// grafana-datasource.json
{
  "name": "AI Orchestrator Hub",
  "type": "prometheus",
  "url": "http://prometheus:9090",
  "access": "proxy",
  "isDefault": false
}
```

### ELK Stack

```yaml
# filebeat.yml
filebeat.inputs:
- type: log
  paths:
    - /var/log/ai-orchestrator/*.log
  json.keys_under_root: true
  json.overwrite_keys: true
  fields:
    service: ai-orchestrator
    environment: production

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
  index: "ai-orchestrator-%{+yyyy.MM.dd}"
```

### Jaeger (Tracing)

```yaml
# jaeger-config.yml
service:
  name: ai-orchestrator
  version: "2.0.0"

reporter:
  collector:
    endpoint: "http://jaeger-collector:14268/api/traces"

sampler:
  type: probabilistic
  param: 0.1
```

## Custom Metrics and Monitoring

### Custom Metrics Collection

```rust
use crate::infrastructure::metrics::MetricsCollector;

pub async fn record_custom_metrics(collector: &MetricsCollector) {
    // Record business metrics
    collector.record_gauge("active_projects", active_projects_count).await;
    collector.record_counter("workflows_completed", completed_count).await;
    collector.record_histogram("workflow_duration", duration_ms).await;

    // Record performance metrics
    collector.record_gauge("cache_hit_rate", cache_hit_rate).await;
    collector.record_counter("api_requests_total", request_count).await;
}
```

### Custom Health Checks

```rust
use crate::infrastructure::health::HealthChecker;

pub async fn custom_health_check(checker: &HealthChecker) -> HealthResult {
    // Check external dependencies
    let db_health = checker.check_database().await?;
    let cache_health = checker.check_cache().await?;
    let external_api_health = checker.check_external_api().await?;

    // Custom business logic checks
    let business_health = check_business_logic().await?;

    Ok(HealthStatus {
        status: if all_healthy(db_health, cache_health, external_api_health, business_health) {
            "healthy"
        } else {
            "degraded"
        },
        details: HealthDetails {
            database: db_health,
            cache: cache_health,
            external_api: external_api_health,
            business_logic: business_health,
        }
    })
}
```

## Best Practices

### Logging Best Practices

1. **Use appropriate log levels**: Don't log debug info in production
2. **Include context**: Add relevant IDs, timestamps, and metadata
3. **Structured logging**: Use JSON format for machine parsing
4. **Log rotation**: Implement log rotation to prevent disk space issues
5. **Security**: Don't log sensitive information

### Metrics Best Practices

1. **Use meaningful names**: Follow naming conventions
2. **Add labels**: Use labels for dimensional metrics
3. **Monitor trends**: Focus on trends rather than absolute values
4. **Set appropriate thresholds**: Avoid alert fatigue
5. **Document metrics**: Maintain metric documentation

### Monitoring Best Practices

1. **Monitor the right things**: Focus on business and user experience metrics
2. **Set up alerts**: Configure alerts for critical issues
3. **Automate responses**: Implement automated remediation where possible
4. **Regular review**: Review and update monitoring regularly
5. **Capacity planning**: Use monitoring data for capacity planning

### Alerting Best Practices

1. **Alert on symptoms**: Alert on user impact rather than technical issues
2. **Escalation policies**: Define clear escalation paths
3. **On-call rotation**: Implement on-call schedules
4. **Alert fatigue**: Minimize false positives
5. **Documentation**: Document alert responses and procedures

## Troubleshooting Observability

### Common Issues

#### Missing Metrics

```bash
# Check if metrics collection is enabled
curl http://localhost:3001/metrics | jq '.'

# Verify configuration
cat .env | grep METRICS

# Check logs for metric collection errors
grep "metrics" /var/log/ai-orchestrator/app.log
```

#### Log Parsing Issues

```bash
# Validate JSON log format
tail -f /var/log/ai-orchestrator/app.log | jq .

# Check for malformed JSON
grep -v "^{" /var/log/ai-orchestrator/app.log

# Fix log format configuration
HIVE_LOGGING__FORMAT=json
```

#### Dashboard Loading Issues

```bash
# Check Grafana logs
docker logs grafana

# Verify data source configuration
curl http://grafana:3000/api/datasources

# Test Prometheus query
curl "http://prometheus:9090/api/v1/query?query=up"
```

#### Alert Configuration Issues

```bash
# Test alert rules
curl http://prometheus:9090/api/v1/rules

# Check alert manager status
curl http://alertmanager:9093/api/v2/status

# Validate alert expressions
promtool check rules alert_rules.yml
```

### Performance Impact

```bash
# Monitor observability overhead
curl http://localhost:3001/metrics | grep observability

# Adjust collection intervals
HIVE_MONITORING__METRICS_COLLECTION_INTERVAL_MS=10000

# Disable verbose logging in production
HIVE_LOGGING__LEVEL=warn
```

### Data Retention

```bash
# Configure retention policies
# Prometheus
--storage.tsdb.retention.time=30d

# Elasticsearch
curl -X PUT "elasticsearch:9200/_ilm/policy/logs_policy" \
  -H 'Content-Type: application/json' \
  -d '{
    "policy": {
      "phases": {
        "delete": {
          "min_age": "30d",
          "actions": {
            "delete": {}
          }
        }
      }
    }
  }'
```

This observability guide provides comprehensive monitoring, logging, and alerting capabilities for the AI Orchestrator Hub.