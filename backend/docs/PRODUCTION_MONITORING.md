# Production Monitoring Setup

This document provides comprehensive guidance for setting up production monitoring for the AI Orchestrator Hub system.

## Overview

The production monitoring system provides enterprise-grade observability with:

- **Real-time Metrics Collection**: System, agent, task, and business metrics
- **Intelligent Alerting**: Adaptive thresholds and predictive analytics
- **Comprehensive Dashboards**: Real-time visualization with customizable widgets
- **Prometheus Integration**: Industry-standard metrics export
- **Multi-channel Notifications**: Console, webhooks, email, Slack integration
- **Business Metrics Tracking**: Task completion rates, agent utilization, system throughput

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Application   │    │ Production      │    │   External      │
│                 │    │ Monitoring      │    │   Systems       │
│ ┌─────────────┐ │    │ System          │    │                 │
│ │  Agents     │◄┼────┤                 │    │ ┌─────────────┐ │
│ └─────────────┘ │    │ ┌─────────────┐ │    │ │ Prometheus  │ │
│                 │    │ │ Health      │◄┼────┼─┤             │ │
│ ┌─────────────┐ │    │ │ Monitor     │ │    │ └─────────────┘ │
│ │  Tasks      │◄┼────┼─┤             │ │    │                 │
│ └─────────────┘ │    │ └─────────────┘ │    │ ┌─────────────┐ │
│                 │    │                 │    │ │ Grafana     │ │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ │ Dashboard   │ │
│ │  Metrics    │◄┼────┼─┤ Performance │◄┼────┼─┤             │ │
│ │  Collector  │ │    │ │ Monitor     │ │    │ └─────────────┘ │
│ └─────────────┘ │    │ └─────────────┘ │    │                 │
│                 │    │                 │    │ ┌─────────────┐ │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ │ Alert-      │ │
│ │ Telemetry   │◄┼────┼─┤ Intelligent │◄┼────┼─┤ manager    │ │
│ │ Collector   │ │    │ │ Alerting    │ │    │ └─────────────┘ │
│ └─────────────┘ │    │ └─────────────┘ │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Quick Start

### 1. Basic Setup

```rust
use ai_orchestrator_hub::infrastructure::monitoring::ProductionMonitoringSystem;

let config = ProductionMonitoringConfig::default();
let monitoring = ProductionMonitoringSystem::new(config).await?;
monitoring.start().await?;
```

### 2. With Custom Configuration

```rust
let config = ProductionMonitoringConfig {
    health_check_interval_seconds: 30,
    enable_prometheus_exporter: true,
    prometheus_port: 9090,
    alerting_thresholds: ProductionAlertThresholds {
        system_cpu_critical: 90.0,
        system_memory_critical: 95.0,
        ..Default::default()
    },
    ..Default::default()
};
```

### 3. Add Notification Channels

```rust
monitoring.add_notification_channel(NotificationChannelConfig {
    channel_type: "webhook".to_string(),
    endpoint: Some("https://hooks.slack.com/services/YOUR/WEBHOOK".to_string()),
    enabled: true,
    severity_filter: vec!["warning".to_string(), "critical".to_string()],
}).await?;
```

## Configuration

### Environment-Specific Configuration

The system supports different configurations for development, staging, and production:

- **Development**: Lightweight monitoring with console output
- **Staging**: Pre-production monitoring with external integrations
- **Production**: Full enterprise monitoring with all features enabled

### Configuration Files

- `settings/production_monitoring.toml` - Production configuration
- `settings/staging_monitoring.toml` - Staging configuration
- `settings/development_monitoring.toml` - Development configuration

### Key Configuration Options

```toml
[production_monitoring]
# Monitoring intervals
health_check_interval_seconds = 30
performance_collection_interval_seconds = 15
alert_evaluation_interval_seconds = 60

# External integrations
enable_prometheus_exporter = true
prometheus_port = 9090
enable_grafana_integration = true

# Alerting thresholds
[production_monitoring.alerting_thresholds]
system_cpu_critical = 90.0
system_memory_critical = 95.0
task_failure_rate_critical = 25.0
error_rate_critical_per_minute = 10.0

# Business metrics
[production_monitoring.business_metrics_config]
enable_business_metrics = true
task_completion_target_percentage = 95.0
agent_utilization_target_percentage = 80.0
```

## Metrics Collection

### System Metrics

- **CPU Usage**: Percentage and core utilization
- **Memory Usage**: Percentage and absolute bytes
- **Disk I/O**: Read/write operations per second
- **Network I/O**: Bytes in/out, active connections
- **System Load**: Overall system load average

### Agent Metrics

- **Agent Health**: Status, response time, error rate
- **Agent Performance**: Tasks completed/failed, utilization
- **Agent Lifecycle**: Created, started, stopped events
- **Resource Usage**: CPU, memory, network per agent

### Task Metrics

- **Task Completion**: Success/failure rates
- **Task Performance**: Duration, throughput
- **Queue Management**: Tasks in queue, processing times
- **Task Distribution**: Load balancing metrics

### Business Metrics

- **Task Completion Rate**: Percentage of successful tasks
- **Agent Utilization**: Percentage of active agent time
- **System Uptime**: Availability percentage
- **System Throughput**: Tasks processed per second
- **Customer Satisfaction**: Service quality metrics

## Alerting System

### Alert Types

1. **Threshold Alerts**: Static threshold violations
2. **Anomaly Alerts**: Statistical anomaly detection
3. **Predictive Alerts**: Future issue prediction
4. **Correlation Alerts**: Related issue grouping

### Alert Severity Levels

- **Info**: Informational alerts
- **Warning**: Potential issues requiring attention
- **Critical**: Immediate action required

### Alert Channels

- **Console**: Local logging and display
- **Webhook**: HTTP endpoints for external systems
- **Email**: SMTP-based notifications
- **Slack/Discord**: Team communication platforms

### Example Alert Rules

```yaml
# CPU usage critical alert
- alert: HighCPUUsageCritical
  expr: ai_orchestrator_cpu_usage_percent > 90
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "Critical CPU usage"
    description: "CPU usage above 90% for 5+ minutes"

# Predictive performance degradation
- alert: PredictedPerformanceDegradation
  expr: predict_linear(ai_orchestrator_system_health_score[1h], 1*3600) < 0.6
  for: 5m
  labels:
    severity: warning
    alert_type: predictive
```

## Dashboard System

### Built-in Widgets

- **System Health Overview**: Overall system status
- **Resource Usage Charts**: CPU, memory, disk, network
- **Agent Health Table**: Individual agent status
- **Task Performance Metrics**: Completion rates, throughput
- **Error Rate Trends**: Error patterns over time
- **Business Metrics**: KPI tracking
- **Active Alerts**: Current alert status
- **Recent Events**: Telemetry event stream

### Custom Widgets

```rust
let custom_widget = DashboardWidget {
    id: "custom_metric".to_string(),
    title: "Custom Metric".to_string(),
    widget_type: WidgetType::Chart,
    position: WidgetPosition { x: 0, y: 0, width: 6, height: 4 },
    config: HashMap::from([
        ("chart_type".to_string(), "line".to_string()),
        ("metric".to_string(), "custom_value".to_string()),
    ]),
    data_source: "custom_metrics".to_string(),
};

dashboard.add_widget(custom_widget).await?;
```

### Real-time Updates

The dashboard supports real-time updates with configurable refresh intervals:

- **Health Data**: 30-second updates
- **Performance Metrics**: 15-second updates
- **Alert Status**: Real-time notifications
- **Business Metrics**: 5-minute updates

## Prometheus Integration

### Metrics Export

The system exports metrics in Prometheus format at `/metrics`:

```text
# HELP ai_orchestrator_cpu_usage_percent Current CPU usage percentage
# TYPE ai_orchestrator_cpu_usage_percent gauge
ai_orchestrator_cpu_usage_percent 45.2

# HELP ai_orchestrator_task_completion_rate Business metric: task completion rate
# TYPE ai_orchestrator_task_completion_rate gauge
ai_orchestrator_task_completion_rate 96.5
```

### Available Metrics

- **System Metrics**: CPU, memory, disk, network usage
- **Agent Metrics**: Health, performance, utilization
- **Task Metrics**: Completion rates, queue sizes, throughput
- **Error Metrics**: Error rates, error types
- **Business Metrics**: Completion rates, uptime, satisfaction
- **Alert Metrics**: Alert counts, severity distribution

### Custom Metrics

```rust
let metric = PrometheusExporter::create_metric(
    "ai_orchestrator_custom_metric".to_string(),
    "Custom application metric".to_string(),
    PrometheusMetricType::Gauge,
    42.0,
    HashMap::from([
        ("service".to_string(), "my_service".to_string()),
    ]),
);

exporter.add_custom_metric(metric).await?;
```

## Grafana Integration

### Pre-configured Dashboard

Import the provided dashboard configuration:

```bash
# Import dashboard via Grafana API
curl -X POST -H "Content-Type: application/json" \
  -d @settings/grafana_dashboards.json \
  http://localhost:3000/api/dashboards/db
```

### Dashboard Features

- **System Overview**: Health status, resource usage
- **Agent Monitoring**: Individual agent performance
- **Task Analytics**: Completion rates, performance trends
- **Error Analysis**: Error patterns and trends
- **Business KPIs**: Key performance indicators
- **Alert Management**: Active alerts and trends

### Custom Panels

Add custom panels for specific metrics:

```json
{
  "title": "Custom Metric",
  "type": "graph",
  "targets": [
    {
      "expr": "ai_orchestrator_custom_metric{service=\"my_service\"}",
      "legendFormat": "Custom Value"
    }
  ]
}
```

## Notification Channels

### Webhook Configuration

```rust
let webhook_config = NotificationChannelConfig {
    channel_type: "webhook".to_string(),
    endpoint: Some("https://api.example.com/webhooks/alerts".to_string()),
    enabled: true,
    severity_filter: vec!["warning".to_string(), "critical".to_string()],
};

monitoring.add_notification_channel(webhook_config).await?;
```

### Slack Integration

```rust
let slack_config = NotificationChannelConfig {
    channel_type: "webhook".to_string(),
    endpoint: Some("https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK".to_string()),
    enabled: true,
    severity_filter: vec!["critical".to_string()],
};
```

### Email Notifications

```rust
let email_config = NotificationChannelConfig {
    channel_type: "email".to_string(),
    endpoint: Some("alerts@company.com".to_string()),
    enabled: true,
    severity_filter: vec!["warning".to_string(), "critical".to_string()],
};
```

## Best Practices

### Alert Configuration

1. **Start Conservative**: Begin with higher thresholds
2. **Tune Gradually**: Adjust based on normal operation patterns
3. **Use Correlation**: Group related alerts to reduce noise
4. **Enable Predictive**: Use predictive alerts for early warning
5. **Regular Review**: Review and update alert rules quarterly

### Dashboard Organization

1. **Logical Grouping**: Group related metrics together
2. **Clear Labeling**: Use descriptive titles and labels
3. **Color Coding**: Use consistent color schemes
4. **Refresh Rates**: Balance real-time needs with performance
5. **User Roles**: Different dashboards for different user types

### Performance Considerations

1. **Metrics Retention**: Configure appropriate retention periods
2. **Sampling Rates**: Use appropriate collection intervals
3. **Resource Limits**: Monitor monitoring system resource usage
4. **Network Efficiency**: Optimize data transfer for remote monitoring
5. **Storage Optimization**: Use efficient storage formats

### Security Considerations

1. **Access Control**: Secure monitoring endpoints
2. **Data Encryption**: Encrypt sensitive metrics data
3. **Audit Logging**: Log all monitoring system access
4. **Network Security**: Secure communication channels
5. **Credential Management**: Secure API keys and tokens

## Troubleshooting

### Common Issues

#### High CPU Usage from Monitoring

**Symptoms**: Monitoring system consuming excessive CPU
**Solution**:
- Increase collection intervals
- Reduce metrics retention period
- Disable unnecessary metrics collection

#### Alert Noise

**Symptoms**: Too many alerts, alert fatigue
**Solutions**:
- Increase alert thresholds
- Enable alert suppression
- Use alert correlation
- Implement alert escalation

#### Dashboard Performance

**Symptoms**: Slow dashboard loading or updates
**Solutions**:
- Reduce widget count
- Increase refresh intervals
- Optimize data queries
- Use data aggregation

#### Missing Metrics

**Symptoms**: Expected metrics not appearing
**Solutions**:
- Check Prometheus configuration
- Verify metric names and labels
- Check network connectivity
- Review metric collection logs

### Debug Commands

```bash
# Check monitoring system health
curl http://localhost:9090/health

# View current metrics
curl http://localhost:9090/metrics | head -50

# Check system status
curl http://localhost:8080/api/monitoring/status

# View alert history
curl http://localhost:8080/api/monitoring/alerts?limit=10
```

## API Reference

### Monitoring System API

```rust
// Start monitoring
monitoring.start().await?;

// Get health snapshot
let health = monitoring.get_system_health_snapshot().await?;

// Get performance summary
let performance = monitoring.get_performance_summary().await?;

// Get business metrics
let business = monitoring.get_business_metrics().await?;

// Process alerts
let alerts = monitoring.process_alerts().await?;
```

### Dashboard API

```rust
// Get dashboard configuration
let config = dashboard.get_dashboard_config().await?;

// Get widget data
let data = dashboard.get_widget_data("cpu_usage").await?;

// Add custom widget
dashboard.add_widget(custom_widget).await?;
```

### Prometheus API

```rust
// Generate metrics
let metrics = exporter.generate_metrics().await?;

// Add custom metric
exporter.add_custom_metric(metric).await?;
```

## Examples

See `examples/production_monitoring_example.rs` for a complete working example of setting up and using the production monitoring system.

## Support

For issues or questions regarding the production monitoring system:

1. Check the troubleshooting section above
2. Review the configuration files for examples
3. Check the application logs for error messages
4. Consult the main project documentation
5. Open an issue in the project repository

## Changelog

### Version 1.0.0
- Initial production monitoring system
- Comprehensive metrics collection
- Intelligent alerting with adaptive thresholds
- Enhanced dashboard with real-time updates
- Prometheus integration
- Multi-channel notifications
- Business metrics tracking
- Environment-specific configurations