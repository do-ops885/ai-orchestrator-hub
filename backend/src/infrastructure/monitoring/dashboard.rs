//! Enhanced Dashboard System
//!
//! Provides comprehensive dashboard functionality for monitoring visualization
//! with real-time updates, customizable widgets, and interactive features.

use super::production_monitoring::ProductionMonitoringSystem;
use super::types::{DashboardConfig, DashboardWidget, WidgetPosition, WidgetType};
use crate::utils::error::HiveResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct EnhancedDashboard {
    config: Arc<RwLock<DashboardConfig>>,
    widgets: Arc<RwLock<HashMap<String, DashboardWidget>>>,
    production_monitoring: Option<Arc<ProductionMonitoringSystem>>,
    active: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidgetData {
    pub widget_id: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub id: String,
    pub name: String,
    pub description: String,
    pub widgets: Vec<WidgetPosition>,
    pub theme: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for EnhancedDashboard {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedDashboard {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(DashboardConfig {
                layout: "grid".to_string(),
                theme: "dark".to_string(),
                refresh_interval: 5000,
                widgets: vec![],
            })),
            widgets: Arc::new(RwLock::new(HashMap::new())),
            production_monitoring: None,
            active: Arc::new(RwLock::new(false)),
        }
    }

    #[must_use] 
    pub fn with_production_monitoring(
        mut self,
        monitoring: Arc<ProductionMonitoringSystem>,
    ) -> Self {
        self.production_monitoring = Some(monitoring);
        self
    }

    pub async fn start(&self) -> HiveResult<()> {
        *self.active.write().await = true;
        tracing::info!("Enhanced dashboard started");
        Ok(())
    }

    pub async fn stop(&self) -> HiveResult<()> {
        *self.active.write().await = false;
        tracing::info!("Enhanced dashboard stopped");
        Ok(())
    }

    pub async fn get_dashboard_config(&self) -> HiveResult<DashboardConfig> {
        Ok(self.config.read().await.clone())
    }

    pub async fn update_dashboard_config(&self, config: DashboardConfig) -> HiveResult<()> {
        *self.config.write().await = config;
        Ok(())
    }

    /// Create default dashboard with comprehensive widgets
    pub async fn create_default_dashboard(&self) -> HiveResult<()> {
        let mut config = self.config.write().await;

        config.layout = "grid".to_string();
        config.theme = "dark".to_string();
        config.refresh_interval = 30;

        // Create comprehensive set of widgets
        let widgets = self.create_default_widgets().await;
        config.widgets = widgets;

        Ok(())
    }

    /// Create default set of monitoring widgets
    async fn create_default_widgets(&self) -> Vec<DashboardWidget> {
        vec![
            // System Health Overview
            DashboardWidget {
                id: "system_health_overview".to_string(),
                title: "System Health Overview".to_string(),
                widget_type: WidgetType::Metric,
                position: WidgetPosition {
                    x: 0,
                    y: 0,
                    width: 4,
                    height: 2,
                },
                config: HashMap::from([
                    ("metric_type".to_string(), "system_health".to_string()),
                    ("show_trend".to_string(), "true".to_string()),
                    ("alert_threshold".to_string(), "warning".to_string()),
                ]),
                data_source: "production_monitoring".to_string(),
            },
            // CPU Usage Chart
            DashboardWidget {
                id: "cpu_usage_chart".to_string(),
                title: "CPU Usage".to_string(),
                widget_type: WidgetType::Chart,
                position: WidgetPosition {
                    x: 4,
                    y: 0,
                    width: 4,
                    height: 2,
                },
                config: HashMap::from([
                    ("chart_type".to_string(), "line".to_string()),
                    ("time_range".to_string(), "1h".to_string()),
                    ("show_thresholds".to_string(), "true".to_string()),
                    ("warning_threshold".to_string(), "75".to_string()),
                    ("critical_threshold".to_string(), "90".to_string()),
                ]),
                data_source: "metrics_collector".to_string(),
            },
            // Memory Usage Chart
            DashboardWidget {
                id: "memory_usage_chart".to_string(),
                title: "Memory Usage".to_string(),
                widget_type: WidgetType::Chart,
                position: WidgetPosition {
                    x: 8,
                    y: 0,
                    width: 4,
                    height: 2,
                },
                config: HashMap::from([
                    ("chart_type".to_string(), "area".to_string()),
                    ("time_range".to_string(), "1h".to_string()),
                    ("show_thresholds".to_string(), "true".to_string()),
                    ("warning_threshold".to_string(), "80".to_string()),
                    ("critical_threshold".to_string(), "95".to_string()),
                ]),
                data_source: "metrics_collector".to_string(),
            },
            // Agent Health Status
            DashboardWidget {
                id: "agent_health_status".to_string(),
                title: "Agent Health Status".to_string(),
                widget_type: WidgetType::Table,
                position: WidgetPosition {
                    x: 0,
                    y: 2,
                    width: 6,
                    height: 3,
                },
                config: HashMap::from([
                    ("show_status".to_string(), "true".to_string()),
                    ("show_response_time".to_string(), "true".to_string()),
                    ("show_error_rate".to_string(), "true".to_string()),
                    ("auto_refresh".to_string(), "true".to_string()),
                ]),
                data_source: "health_monitor".to_string(),
            },
            // Task Performance Metrics
            DashboardWidget {
                id: "task_performance".to_string(),
                title: "Task Performance".to_string(),
                widget_type: WidgetType::Metric,
                position: WidgetPosition {
                    x: 6,
                    y: 2,
                    width: 3,
                    height: 2,
                },
                config: HashMap::from([
                    (
                        "metrics".to_string(),
                        "completion_rate,throughput,queue_size".to_string(),
                    ),
                    ("show_trend".to_string(), "true".to_string()),
                    ("target_completion_rate".to_string(), "95".to_string()),
                ]),
                data_source: "task_metrics".to_string(),
            },
            // Active Alerts
            DashboardWidget {
                id: "active_alerts".to_string(),
                title: "Active Alerts".to_string(),
                widget_type: WidgetType::Alert,
                position: WidgetPosition {
                    x: 9,
                    y: 2,
                    width: 3,
                    height: 2,
                },
                config: HashMap::from([
                    ("show_critical".to_string(), "true".to_string()),
                    ("show_warning".to_string(), "true".to_string()),
                    ("max_alerts".to_string(), "10".to_string()),
                    ("auto_refresh".to_string(), "true".to_string()),
                ]),
                data_source: "intelligent_alerting".to_string(),
            },
            // Business Metrics Overview
            DashboardWidget {
                id: "business_metrics".to_string(),
                title: "Business Metrics".to_string(),
                widget_type: WidgetType::Metric,
                position: WidgetPosition {
                    x: 0,
                    y: 5,
                    width: 4,
                    height: 2,
                },
                config: HashMap::from([
                    (
                        "metrics".to_string(),
                        "task_completion_rate,agent_utilization,system_uptime".to_string(),
                    ),
                    ("show_targets".to_string(), "true".to_string()),
                    ("show_trend".to_string(), "true".to_string()),
                ]),
                data_source: "business_metrics".to_string(),
            },
            // System Throughput
            DashboardWidget {
                id: "system_throughput".to_string(),
                title: "System Throughput".to_string(),
                widget_type: WidgetType::Chart,
                position: WidgetPosition {
                    x: 4,
                    y: 5,
                    width: 4,
                    height: 2,
                },
                config: HashMap::from([
                    ("chart_type".to_string(), "bar".to_string()),
                    ("time_range".to_string(), "1h".to_string()),
                    ("metric".to_string(), "tasks_per_second".to_string()),
                    ("show_average".to_string(), "true".to_string()),
                ]),
                data_source: "performance_monitor".to_string(),
            },
            // Error Rate Trends
            DashboardWidget {
                id: "error_rate_trends".to_string(),
                title: "Error Rate Trends".to_string(),
                widget_type: WidgetType::Chart,
                position: WidgetPosition {
                    x: 8,
                    y: 5,
                    width: 4,
                    height: 2,
                },
                config: HashMap::from([
                    ("chart_type".to_string(), "line".to_string()),
                    ("time_range".to_string(), "24h".to_string()),
                    ("show_thresholds".to_string(), "true".to_string()),
                    ("warning_threshold".to_string(), "2".to_string()),
                    ("critical_threshold".to_string(), "10".to_string()),
                ]),
                data_source: "error_metrics".to_string(),
            },
            // Recent Telemetry Events
            DashboardWidget {
                id: "recent_telemetry".to_string(),
                title: "Recent Events".to_string(),
                widget_type: WidgetType::Log,
                position: WidgetPosition {
                    x: 0,
                    y: 7,
                    width: 8,
                    height: 3,
                },
                config: HashMap::from([
                    ("max_events".to_string(), "50".to_string()),
                    (
                        "filter_severity".to_string(),
                        "warning,critical,error".to_string(),
                    ),
                    ("auto_scroll".to_string(), "true".to_string()),
                ]),
                data_source: "telemetry_collector".to_string(),
            },
            // System Resource Map
            DashboardWidget {
                id: "system_resource_map".to_string(),
                title: "Resource Utilization Map".to_string(),
                widget_type: WidgetType::Map,
                position: WidgetPosition {
                    x: 8,
                    y: 7,
                    width: 4,
                    height: 3,
                },
                config: HashMap::from([
                    ("show_cpu".to_string(), "true".to_string()),
                    ("show_memory".to_string(), "true".to_string()),
                    ("show_disk".to_string(), "true".to_string()),
                    ("show_network".to_string(), "true".to_string()),
                    ("color_scheme".to_string(), "heatmap".to_string()),
                ]),
                data_source: "resource_monitor".to_string(),
            },
        ]
    }

    /// Get widget data for a specific widget
    pub async fn get_widget_data(
        &self,
        widget_id: &str,
    ) -> HiveResult<Option<DashboardWidgetData>> {
        let widgets = self.widgets.read().await;
        let widget = match widgets.get(widget_id) {
            Some(w) => w,
            None => return Ok(None),
        };

        let data = match widget.data_source.as_str() {
            "production_monitoring" => {
                if let Some(monitoring) = &self.production_monitoring {
                    let health_snapshot = monitoring.get_system_health_snapshot().await?;
                    serde_json::json!({
                        "overall_status": health_snapshot.overall_status,
                        "agent_count": health_snapshot.agent_health.len(),
                        "system_cpu": health_snapshot.system_health.cpu_usage,
                        "system_memory": health_snapshot.system_health.memory_usage,
                        "active_connections": health_snapshot.system_health.active_connections
                    })
                } else {
                    serde_json::json!(null)
                }
            }
            "metrics_collector" => {
                if let Some(monitoring) = &self.production_monitoring {
                    let metrics_collector = monitoring.get_metrics_collector();
                    let metrics = metrics_collector.get_current_metrics().await;
                    serde_json::json!({
                        "cpu_usage": metrics.resource_usage.cpu_usage_percent,
                        "memory_usage": metrics.resource_usage.memory_usage_percent,
                        "network_io": {
                            "bytes_in": metrics.resource_usage.network_bytes_in,
                            "bytes_out": metrics.resource_usage.network_bytes_out
                        },
                        "disk_io": {
                            "reads_per_second": metrics.resource_usage.disk_io.reads_per_second,
                            "writes_per_second": metrics.resource_usage.disk_io.writes_per_second
                        }
                    })
                } else {
                    serde_json::json!(null)
                }
            }
            "health_monitor" => {
                if let Some(monitoring) = &self.production_monitoring {
                    let health_snapshot = monitoring.get_system_health_snapshot().await?;
                    let agent_health: Vec<serde_json::Value> = health_snapshot
                        .agent_health
                        .iter()
                        .map(|h| {
                            serde_json::json!({
                                "agent_id": h.agent_id,
                                "status": h.status,
                                "response_time": h.response_time,
                                "error_rate": h.error_rate,
                                "cpu_usage": h.resource_usage.cpu_usage,
                                "memory_usage": h.resource_usage.memory_usage
                            })
                        })
                        .collect();
                    serde_json::json!(agent_health)
                } else {
                    serde_json::json!([])
                }
            }
            "task_metrics" => {
                if let Some(monitoring) = &self.production_monitoring {
                    let metrics_collector = monitoring.get_metrics_collector();
                    let metrics = metrics_collector.get_current_metrics().await;
                    let completion_rate = if metrics.task_metrics.total_tasks_submitted > 0 {
                        (metrics.task_metrics.total_tasks_completed as f64
                            / metrics.task_metrics.total_tasks_submitted as f64)
                            * 100.0
                    } else {
                        0.0
                    };
                    serde_json::json!({
                        "completion_rate": completion_rate,
                        "throughput": metrics.performance.throughput_tasks_per_second,
                        "queue_size": metrics.task_metrics.tasks_in_queue,
                        "average_duration": metrics.task_metrics.average_task_duration_ms,
                        "total_submitted": metrics.task_metrics.total_tasks_submitted,
                        "total_completed": metrics.task_metrics.total_tasks_completed
                    })
                } else {
                    serde_json::json!(null)
                }
            }
            "intelligent_alerting" => {
                if let Some(monitoring) = &self.production_monitoring {
                    match monitoring.process_alerts().await {
                        Ok(alerts) => {
                            let alert_data: Vec<serde_json::Value> = alerts
                                .iter()
                                .take(10)
                                .map(|a| {
                                    serde_json::json!({
                                        "title": a.base_alert.title,
                                        "description": a.base_alert.description,
                                        "level": a.base_alert.level,
                                        "timestamp": a.base_alert.timestamp,
                                        "predicted": a.predicted,
                                        "confidence": a.confidence
                                    })
                                })
                                .collect();
                            serde_json::json!(alert_data)
                        }
                        Err(e) => {
                            tracing::warn!("Failed to get alerts for dashboard: {}", e);
                            serde_json::json!([])
                        }
                    }
                } else {
                    serde_json::json!([])
                }
            }
            "business_metrics" => {
                if let Some(monitoring) = &self.production_monitoring {
                    let business_metrics = monitoring.get_business_metrics().await;
                    serde_json::json!({
                        "task_completion_rate": business_metrics.task_completion_rate,
                        "agent_utilization_rate": business_metrics.agent_utilization_rate,
                        "system_uptime_percentage": business_metrics.system_uptime_percentage,
                        "customer_satisfaction_score": business_metrics.customer_satisfaction_score,
                        "total_tasks_processed": business_metrics.total_tasks_processed,
                        "system_throughput": business_metrics.system_throughput_tasks_per_second
                    })
                } else {
                    serde_json::json!(null)
                }
            }
            "performance_monitor" => {
                if let Some(monitoring) = &self.production_monitoring {
                    let performance_summary = monitoring.get_performance_summary().await?;
                    serde_json::json!({
                        "overall_score": performance_summary.overall_score,
                        "trend": performance_summary.trend,
                        "bottlenecks": performance_summary.bottlenecks,
                        "recommendations": performance_summary.recommendations
                    })
                } else {
                    serde_json::json!(null)
                }
            }
            "error_metrics" => {
                if let Some(monitoring) = &self.production_monitoring {
                    let metrics_collector = monitoring.get_metrics_collector();
                    let metrics = metrics_collector.get_current_metrics().await;
                    serde_json::json!({
                        "total_errors": metrics.error_metrics.total_errors,
                        "error_rate_per_minute": metrics.error_metrics.error_rate_per_minute,
                        "critical_errors": metrics.error_metrics.critical_errors,
                        "errors_by_type": metrics.error_metrics.errors_by_type
                    })
                } else {
                    serde_json::json!(null)
                }
            }
            "telemetry_collector" => {
                if let Some(monitoring) = &self.production_monitoring {
                    let telemetry_collector = monitoring.get_telemetry_collector();
                    let recent_events = telemetry_collector.get_recent_events(50).await;
                    let event_data: Vec<serde_json::Value> = recent_events
                        .iter()
                        .map(|e| {
                            serde_json::json!({
                                "timestamp": e.timestamp,
                                "event_type": e.event_type,
                                "source": e.source,
                                "severity": e.severity,
                                "data": e.data
                            })
                        })
                        .collect();
                    serde_json::json!(event_data)
                } else {
                    serde_json::json!([])
                }
            }
            "resource_monitor" => {
                if let Some(monitoring) = &self.production_monitoring {
                    let metrics_collector = monitoring.get_metrics_collector();
                    let metrics = metrics_collector.get_current_metrics().await;
                    serde_json::json!({
                        "cpu_usage": metrics.resource_usage.cpu_usage_percent,
                        "memory_usage": metrics.resource_usage.memory_usage_percent,
                        "disk_usage": metrics.resource_usage.disk_io.disk_usage_percent,
                        "network_connections": metrics.resource_usage.network_io.connections_active
                    })
                } else {
                    serde_json::json!(null)
                }
            }
            _ => serde_json::json!(null),
        };

        let widget_data = DashboardWidgetData {
            widget_id: widget_id.to_string(),
            timestamp: Utc::now(),
            data,
            metadata: HashMap::from([
                ("data_source".to_string(), widget.data_source.clone()),
                (
                    "widget_type".to_string(),
                    format!("{:?}", widget.widget_type),
                ),
            ]),
        };

        Ok(Some(widget_data))
    }

    /// Get all widget data for the dashboard
    pub async fn get_all_widget_data(&self) -> HiveResult<HashMap<String, DashboardWidgetData>> {
        let config = self.config.read().await;
        let mut all_data = HashMap::new();

        for widget in &config.widgets {
            if let Some(data) = self.get_widget_data(&widget.id).await? {
                all_data.insert(widget.id.clone(), data);
            }
        }

        Ok(all_data)
    }

    /// Add a custom widget to the dashboard
    pub async fn add_widget(&self, widget: DashboardWidget) -> HiveResult<()> {
        let mut widgets = self.widgets.write().await;
        widgets.insert(widget.id.clone(), widget);
        Ok(())
    }

    /// Remove a widget from the dashboard
    pub async fn remove_widget(&self, widget_id: &str) -> HiveResult<()> {
        let mut widgets = self.widgets.write().await;
        widgets.remove(widget_id);
        Ok(())
    }

    /// Update widget configuration
    pub async fn update_widget_config(
        &self,
        widget_id: &str,
        config: HashMap<String, String>,
    ) -> HiveResult<()> {
        let mut widgets = self.widgets.write().await;
        if let Some(widget) = widgets.get_mut(widget_id) {
            widget.config = config;
            Ok(())
        } else {
            Err(crate::utils::error::HiveError::NotFound {
                resource: format!("Widget {widget_id}"),
            })
        }
    }

    /// Create a dashboard layout preset
    pub async fn create_layout_preset(
        &self,
        name: String,
        description: String,
    ) -> HiveResult<DashboardLayout> {
        let config = self.config.read().await;
        let layout = DashboardLayout {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            widgets: config.widgets.iter().map(|w| w.position.clone()).collect(),
            theme: config.theme.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Ok(layout)
    }

    /// Export dashboard configuration
    pub async fn export_dashboard_config(&self) -> HiveResult<String> {
        let config = self.config.read().await;
        let json = serde_json::to_string_pretty(&*config)?;
        Ok(json)
    }

    /// Import dashboard configuration
    pub async fn import_dashboard_config(&self, config_json: &str) -> HiveResult<()> {
        let config: DashboardConfig = serde_json::from_str(config_json)?;
        *self.config.write().await = config;
        Ok(())
    }

    /// Get dashboard health status
    pub async fn get_dashboard_health(&self) -> HiveResult<serde_json::Value> {
        let is_active = *self.active.read().await;
        let config = self.config.read().await;
        let widget_count = config.widgets.len();

        let health = serde_json::json!({
            "status": if is_active { "healthy" } else { "inactive" },
            "widget_count": widget_count,
            "theme": config.theme,
            "refresh_interval": config.refresh_interval,
            "last_updated": Utc::now()
        });

        Ok(health)
    }
}

// Legacy Dashboard for backward compatibility
#[derive(Clone)]
pub struct Dashboard;

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Dashboard {
    #[must_use] 
    pub fn new() -> Self {
        Self
    }

    pub async fn get_dashboard_config(&self) -> HiveResult<DashboardConfig> {
        Ok(DashboardConfig {
            layout: "grid".to_string(),
            theme: "dark".to_string(),
            refresh_interval: 30,
            widgets: vec![],
        })
    }
}
