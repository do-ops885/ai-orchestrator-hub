//! Common types and structures for the monitoring system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: Uuid,
    pub name: String,
    pub agent_type: String,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub version: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Inactive,
    Error,
    Maintenance,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealth {
    pub agent_id: Uuid,
    pub status: HealthStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub response_time: f64,
    pub error_rate: f64,
    pub resource_usage: ResourceHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityStatus {
    pub is_connected: bool,
    pub last_ping: DateTime<Utc>,
    pub latency_ms: f64,
    pub connection_quality: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHealth {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_usage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub timestamp: DateTime<Utc>,
    pub overall_status: HealthStatus,
    pub agent_health: Vec<AgentHealth>,
    pub system_health: SystemHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub active_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStatus {
    pub is_active: bool,
    pub last_update: DateTime<Utc>,
    pub monitored_agents: u32,
    pub alerts_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformance {
    pub agent_id: Uuid,
    pub response_time: f64,
    pub throughput: f64,
    pub error_rate: f64,
    pub success_rate: f64,
    pub resource_utilization: ResourceUtilization,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub disk_io_mb: f64,
    pub network_io_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformance {
    pub timestamp: DateTime<Utc>,
    pub overall_throughput: f64,
    pub average_response_time: f64,
    pub system_load: f64,
    pub active_agents: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatusSummary {
    pub overall_score: f64,
    pub trend: String,
    pub bottlenecks: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorStatusSummary {
    pub communication_patterns: Vec<String>,
    pub decision_patterns: Vec<String>,
    pub adaptation_metrics: AdaptationMetrics,
    pub anomalies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleEvent {
    AgentCreated,
    AgentStarted,
    AgentStopped,
    AgentError,
    TaskAssigned,
    TaskCompleted,
    TaskFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Xml,
    Prometheus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub agent_id: Option<Uuid>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceUtilization {
    pub cpu_cores: u32,
    pub memory_total_gb: f64,
    pub disk_total_gb: f64,
    pub network_bandwidth_mbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub response_time_baseline: f64,
    pub throughput_baseline: f64,
    pub error_rate_baseline: f64,
    pub resource_usage_baseline: ResourceUtilization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationPattern {
    pub pattern_type: String,
    pub frequency: f64,
    pub participants: Vec<Uuid>,
    pub message_volume: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPattern {
    pub decision_type: String,
    pub frequency: f64,
    pub success_rate: f64,
    pub average_time_ms: f64,
    pub complexity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationMetrics {
    pub learning_rate: f64,
    pub adaptation_speed: f64,
    pub performance_improvement: f64,
    pub stability_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: String,
    pub title: String,
    pub widget_type: WidgetType,
    pub position: WidgetPosition,
    pub config: HashMap<String, String>,
    pub data_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    Chart,
    Table,
    Metric,
    Alert,
    Log,
    Map,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub layout: String,
    pub theme: String,
    pub refresh_interval: u32,
    pub widgets: Vec<DashboardWidget>,
}

// Additional types for diagnostics, reporting, automation, and integration
// These would be expanded based on the original monitoring.rs content

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    Health,
    Performance,
    Behavior,
    System,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationTaskType {
    HealthCheck,
    PerformanceOptimization,
    AlertResponse,
    DataCollection,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationType {
    Prometheus,
    Grafana,
    ElasticSearch,
    Webhook,
    Database,
}
