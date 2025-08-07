use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};

/// Advanced telemetry and observability system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub id: uuid::Uuid,
    pub timestamp: u64,
    pub event_type: EventType,
    pub source: String,
    pub data: serde_json::Value,
    pub tags: HashMap<String, String>,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    AgentCreated,
    AgentStateChanged,
    TaskSubmitted,
    TaskCompleted,
    TaskFailed,
    SystemAlert,
    PerformanceMetric,
    ResourceUsage,
    Error,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Real-time telemetry collector
pub struct TelemetryCollector {
    events: Arc<RwLock<Vec<TelemetryEvent>>>,
    max_events: usize,
    subscribers: Arc<RwLock<Vec<Box<dyn TelemetrySubscriber + Send + Sync>>>>,
    metrics: Arc<RwLock<TelemetryMetrics>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryMetrics {
    pub total_events: u64,
    pub events_by_type: HashMap<String, u64>,
    pub events_by_severity: HashMap<String, u64>,
    pub average_events_per_minute: f64,
    pub last_event_timestamp: u64,
    pub uptime_seconds: u64,
    pub start_time: u64,
}

/// Trait for telemetry subscribers (webhooks, databases, etc.)
pub trait TelemetrySubscriber {
    fn on_event(&self, event: &TelemetryEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn name(&self) -> &str;
}

/// Console subscriber for development
pub struct ConsoleTelemetrySubscriber;

impl TelemetrySubscriber for ConsoleTelemetrySubscriber {
    fn on_event(&self, event: &TelemetryEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event.severity {
            Severity::Debug => tracing::debug!("📊 Telemetry: {:?}", event),
            Severity::Info => tracing::info!("📊 Telemetry: {:?}", event),
            Severity::Warning => tracing::warn!("📊 Telemetry: {:?}", event),
            Severity::Error => tracing::error!("📊 Telemetry: {:?}", event),
            Severity::Critical => tracing::error!("🚨 CRITICAL Telemetry: {:?}", event),
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "console"
    }
}

/// Webhook subscriber for external integrations
pub struct WebhookTelemetrySubscriber {
    url: String,
    client: reqwest::Client,
}

impl WebhookTelemetrySubscriber {
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
        }
    }
}

impl TelemetrySubscriber for WebhookTelemetrySubscriber {
    fn on_event(&self, _event: &TelemetryEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would be async
        // For now, we'll just log the intent
        info!("📡 Would send telemetry event to webhook: {}", self.url);
        Ok(())
    }

    fn name(&self) -> &str {
        "webhook"
    }
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    pub fn new(max_events: usize) -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
            subscribers: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(TelemetryMetrics {
                total_events: 0,
                events_by_type: HashMap::new(),
                events_by_severity: HashMap::new(),
                average_events_per_minute: 0.0,
                last_event_timestamp: start_time,
                uptime_seconds: 0,
                start_time,
            })),
        }
    }

    /// Record a telemetry event
    pub async fn record_event(&self, event_type: EventType, source: String, data: serde_json::Value, severity: Severity) {
        let event = TelemetryEvent {
            id: uuid::Uuid::new_v4(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            event_type: event_type.clone(),
            source,
            data,
            tags: HashMap::new(),
            severity: severity.clone(),
        };

        // Store event
        {
            let mut events = self.events.write().await;
            events.push(event.clone());
            
            // Maintain max events limit
            if events.len() > self.max_events {
                events.remove(0);
            }
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_events += 1;
            metrics.last_event_timestamp = event.timestamp;
            
            let type_key = format!("{:?}", event_type);
            *metrics.events_by_type.entry(type_key).or_insert(0) += 1;
            
            let severity_key = format!("{:?}", severity);
            *metrics.events_by_severity.entry(severity_key).or_insert(0) += 1;
            
            // Calculate uptime and events per minute
            metrics.uptime_seconds = event.timestamp - metrics.start_time;
            if metrics.uptime_seconds > 0 {
                metrics.average_events_per_minute = (metrics.total_events as f64 / metrics.uptime_seconds as f64) * 60.0;
            }
        }

        // Notify subscribers
        {
            let subscribers = self.subscribers.read().await;
            for subscriber in subscribers.iter() {
                if let Err(e) = subscriber.on_event(&event) {
                    warn!("Telemetry subscriber '{}' failed: {}", subscriber.name(), e);
                }
            }
        }
    }

    /// Add a telemetry subscriber
    pub async fn add_subscriber(&self, subscriber: Box<dyn TelemetrySubscriber + Send + Sync>) {
        let mut subscribers = self.subscribers.write().await;
        info!("📊 Added telemetry subscriber: {}", subscriber.name());
        subscribers.push(subscriber);
    }

    /// Get recent events
    pub async fn get_recent_events(&self, limit: usize) -> Vec<TelemetryEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(limit).cloned().collect()
    }

    /// Get events by type
    pub async fn get_events_by_type(&self, event_type: &EventType, limit: usize) -> Vec<TelemetryEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .rev()
            .filter(|e| std::mem::discriminant(&e.event_type) == std::mem::discriminant(event_type))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get telemetry metrics
    pub async fn get_metrics(&self) -> TelemetryMetrics {
        self.metrics.read().await.clone()
    }

    /// Start background cleanup and aggregation
    pub fn start_background_tasks(self: Arc<Self>) {
        let collector = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                
                // Cleanup old events (keep last 24 hours)
                let cutoff_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() - (24 * 60 * 60); // 24 hours ago

                {
                    let mut events = collector.events.write().await;
                    events.retain(|event| event.timestamp > cutoff_time);
                }

                info!("🧹 Telemetry cleanup completed");
            }
        });
    }

    /// Generate system health report
    pub async fn generate_health_report(&self) -> serde_json::Value {
        let metrics = self.get_metrics().await;
        let recent_errors = self.get_events_by_type(&EventType::Error, 10).await;
        let recent_alerts = self.get_events_by_type(&EventType::SystemAlert, 5).await;

        serde_json::json!({
            "system_health": {
                "uptime_hours": metrics.uptime_seconds as f64 / 3600.0,
                "total_events": metrics.total_events,
                "events_per_minute": metrics.average_events_per_minute,
                "error_rate": metrics.events_by_severity.get("Error").unwrap_or(&0),
                "critical_events": metrics.events_by_severity.get("Critical").unwrap_or(&0)
            },
            "recent_errors": recent_errors,
            "recent_alerts": recent_alerts,
            "event_distribution": metrics.events_by_type,
            "severity_distribution": metrics.events_by_severity
        })
    }
}

/// Convenience macros for telemetry
#[macro_export]
macro_rules! telemetry_info {
    ($collector:expr, $source:expr, $($key:expr => $value:expr),*) => {
        $collector.record_event(
            $crate::telemetry::EventType::Custom("info".to_string()),
            $source.to_string(),
            serde_json::json!({ $($key: $value),* }),
            $crate::telemetry::Severity::Info
        ).await;
    };
}

#[macro_export]
macro_rules! telemetry_error {
    ($collector:expr, $source:expr, $error:expr) => {
        $collector.record_event(
            $crate::telemetry::EventType::Error,
            $source.to_string(),
            serde_json::json!({ "error": $error.to_string() }),
            $crate::telemetry::Severity::Error
        ).await;
    };
}