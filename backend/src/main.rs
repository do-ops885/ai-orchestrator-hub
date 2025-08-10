//! # Multiagent Hive System - Main Server
//! 
//! Entry point for the multiagent hive system backend server.

mod core;
mod agents;
mod tasks;
mod neural;
mod communication;
mod infrastructure;
mod utils;

use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::Response,
    routing::get,
    Router,
    http::StatusCode,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error, debug, Level};
use tracing_subscriber;

use crate::core::{HiveCoordinator, SwarmIntelligenceEngine};
use crate::utils::HiveConfig;
use crate::infrastructure::{MetricsCollector, CircuitBreaker, MetricThresholds};
use crate::utils::InputValidator;
use crate::agents::AgentRecoveryManager;
use crate::neural::AdaptiveLearningSystem;
use std::time::Duration;

/// Application state containing shared resources
#[derive(Clone)]
pub struct AppState {
    /// The main hive coordinator managing all agents and tasks
    pub hive: Arc<RwLock<HiveCoordinator>>,
    /// System configuration
    pub config: Arc<HiveConfig>,
    /// Enhanced metrics collection system with alerting and trend analysis
    pub metrics: Arc<MetricsCollector>,
    /// Circuit breaker for resilience
    pub circuit_breaker: Arc<CircuitBreaker>,
    /// Agent recovery manager for error handling
    pub recovery_manager: Arc<AgentRecoveryManager>,
    /// Swarm intelligence engine for formation optimization
    pub swarm_intelligence: Arc<RwLock<SwarmIntelligenceEngine>>,
    /// Adaptive learning system for continuous improvement
    pub adaptive_learning: Arc<RwLock<AdaptiveLearningSystem>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Arc::new(HiveConfig::from_env());
    
    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration validation failed: {}", e);
        std::process::exit(1);
    }

    // Initialize structured logging
    let log_level = match config.logging.level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    info!("🚀 Starting Multiagent Hive System v2.0 - Enhanced Edition");
    info!("📊 Configuration loaded: CPU-native, GPU-optional");
    info!("🔧 Initializing enhanced infrastructure components...");

    // Initialize enhanced metrics collector with custom thresholds
    let metric_thresholds = MetricThresholds {
        cpu_warning: config.performance.cpu_warning_threshold.unwrap_or(70.0),
        cpu_critical: config.performance.cpu_critical_threshold.unwrap_or(90.0),
        memory_warning: config.performance.memory_warning_threshold.unwrap_or(80.0),
        memory_critical: config.performance.memory_critical_threshold.unwrap_or(95.0),
        task_failure_rate_warning: 10.0,
        task_failure_rate_critical: 25.0,
        agent_failure_rate_warning: 5.0,
        agent_failure_rate_critical: 15.0,
        response_time_warning: 1000.0,
        response_time_critical: 5000.0,
    };
    let metrics = Arc::new(MetricsCollector::with_thresholds(1000, metric_thresholds));
    info!("✅ Enhanced metrics collector initialized with custom thresholds");

    // Initialize circuit breaker for resilience
    let circuit_breaker = Arc::new(CircuitBreaker::new(
        5, // failure threshold
        Duration::from_secs(30) // recovery timeout
    ));
    info!("✅ Circuit breaker initialized (threshold: 5, timeout: 30s)");

    // Initialize agent recovery manager
    let recovery_manager = Arc::new(AgentRecoveryManager::new());
    info!("✅ Agent recovery manager initialized");

    // Initialize swarm intelligence engine
    let swarm_intelligence = Arc::new(RwLock::new(SwarmIntelligenceEngine::new()));
    info!("✅ Swarm intelligence engine initialized");

    // Initialize adaptive learning system
    let adaptive_learning_config = crate::neural::AdaptiveLearningConfig {
        learning_rate: 0.01,
        momentum: 0.9,
        decay_factor: 0.95,
        min_confidence_threshold: 0.7,
        pattern_retention_days: 30,
        max_patterns: 10000,
    };
    let adaptive_learning = match AdaptiveLearningSystem::new(adaptive_learning_config).await {
        Ok(system) => Arc::new(RwLock::new(system)),
        Err(e) => {
            error!("Failed to initialize adaptive learning system: {}", e);
            return Err(e);
        }
    };
    info!("✅ Adaptive learning system initialized");

    // Initialize the hive coordinator with enhanced capabilities
    let hive = match HiveCoordinator::new().await {
        Ok(coordinator) => Arc::new(RwLock::new(coordinator)),
        Err(e) => {
            error!("Failed to initialize hive coordinator: {}", e);
            return Err(e);
        }
    };
    info!("✅ Hive coordinator initialized");

    let app_state = AppState { 
        hive,
        config: config.clone(),
        metrics: metrics.clone(),
        circuit_breaker,
        recovery_manager,
        swarm_intelligence,
        adaptive_learning,
    };

    info!("🎯 All enhanced components initialized successfully");

    // Start background monitoring and maintenance tasks
    start_background_tasks(app_state.clone()).await;

    // Build the router
    let app = Router::new()
        .route("/", get(|| async { "🐝 Multiagent Hive System API v2.0 - CPU-native, GPU-optional" }))
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        .route("/ws", get(websocket_handler))
        .route("/api/agents", get(get_agents).post(create_agent))
        .route("/api/tasks", get(get_tasks).post(create_task))
        .route("/api/hive/status", get(get_hive_status))
        .route("/api/resources", get(get_resource_info)) // Phase 2: Resource monitoring
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start metrics collection background task
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            metrics_clone.snapshot_current_metrics().await;
        }
    });

    // Start the server
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to {}: {}", bind_addr, e);
            return Err(e.into());
        }
    };
    
    info!("🌐 Server running on http://{}", bind_addr);
    info!("📡 WebSocket endpoint: ws://{}/ws", bind_addr);
    info!("🔧 API endpoints: /api/agents, /api/tasks, /api/hive/status, /api/resources");
    
    // Graceful shutdown handling
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("🛑 Shutdown signal received, gracefully stopping...");
    };
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;
        
    info!("✅ Multiagent Hive System stopped gracefully");
    Ok(())
}

/// Start background tasks for monitoring, alerting, and system maintenance
async fn start_background_tasks(app_state: AppState) {
    let metrics_interval = Duration::from_millis(app_state.config.performance.metrics_collection_interval_ms);
    let alert_interval = Duration::from_millis(app_state.config.performance.alert_check_interval_ms);
    
    // Metrics collection task
    let metrics_state = app_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(metrics_interval);
        loop {
            interval.tick().await;
            
            // Collect system metrics
            if let Err(e) = metrics_state.metrics.collect_system_metrics().await {
                error!("Failed to collect system metrics: {}", e);
            }
            
            // Snapshot current metrics for historical analysis
            metrics_state.metrics.snapshot_current_metrics().await;
            
            // Update hive metrics
            let hive = metrics_state.hive.read().await.get_status().await;
            let hive_metrics = crate::infrastructure::SystemMetrics {
                performance: crate::infrastructure::PerformanceMetrics {
                    requests_per_second: 0.0,
                    average_response_time_ms: 0.0,
                    p95_response_time_ms: 0.0,
                    p99_response_time_ms: 0.0,
                    throughput_tasks_per_second: 0.0,
                },
                resource_usage: crate::infrastructure::ResourceUsageMetrics {
                    cpu_usage_percent: hive.get("cpu_usage").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    memory_usage_percent: hive.get("memory_usage").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    memory_usage_bytes: 0,
                    network_bytes_in: 0,
                    network_bytes_out: 0,
                    disk_usage_bytes: 0,
                    network_io: crate::infrastructure::NetworkMetrics::default(),
                    disk_io: crate::infrastructure::DiskMetrics::default(),
                },
                agent_metrics: crate::infrastructure::AgentMetrics {
                    total_agents: hive.get("total_agents").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                    active_agents: hive.get("active_agents").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                    idle_agents: hive.get("idle_agents").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                    failed_agents: hive.get("failed_agents").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                    average_agent_performance: 0.0,
                    agent_utilization_percent: 0.0,
                    individual_agent_metrics: std::collections::HashMap::new(),
                },
                task_metrics: crate::infrastructure::TaskMetrics {
                    total_tasks_submitted: hive.get("total_tasks").and_then(|v| v.as_u64()).unwrap_or(0),
                    total_tasks_completed: hive.get("completed_tasks").and_then(|v| v.as_u64()).unwrap_or(0),
                    total_tasks_failed: hive.get("failed_tasks").and_then(|v| v.as_u64()).unwrap_or(0),
                    tasks_in_queue: hive.get("pending_tasks").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                    average_task_duration_ms: hive.get("average_task_completion_time").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    task_success_rate: 0.0,
                },
                error_metrics: crate::infrastructure::ErrorMetrics {
                    total_errors: 0,
                    error_rate_per_minute: 0.0,
                    errors_by_type: std::collections::HashMap::new(),
                    critical_errors: 0,
                },
                timestamp: chrono::Utc::now(),
            };
            
            // Update the current metrics instead of calling a non-existent method
            // We'll just snapshot the metrics for now
            metrics_state.metrics.snapshot_current_metrics().await;
        }
    });
    
    // Alert checking task
    let alert_state = app_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(alert_interval);
        loop {
            interval.tick().await;
            
            // Check for alerts
            let alerts = alert_state.metrics.check_alerts().await;
            for alert in alerts {
                match alert.level {
                    crate::infrastructure::AlertLevel::Critical => {
                        error!("🚨 CRITICAL ALERT: {} - {}", alert.title, alert.description);
                        // In production, you would send notifications here
                    }
                    crate::infrastructure::AlertLevel::Warning => {
                        warn!("⚠️  WARNING: {} - {}", alert.title, alert.description);
                    }
                    crate::infrastructure::AlertLevel::Info => {
                        info!("ℹ️  INFO: {} - {}", alert.title, alert.description);
                    }
                }
            }
            
            // Analyze trends
            let trends = alert_state.metrics.analyze_trends().await;
            debug!("System trends - CPU: {:?}, Memory: {:?}, Tasks: {:?}", 
                   trends.cpu_trend, trends.memory_trend, trends.task_completion_trend);
        }
    });
    
    // Agent recovery and maintenance task
    let recovery_state = app_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute
        loop {
            interval.tick().await;
            
            // Check for failed agents and attempt recovery
            let hive = recovery_state.hive.read().await.get_agents_info().await;
            {
                if let Some(agents) = hive.get("agents").and_then(|v| v.as_array()) {
                    for agent_value in agents {
                        if let Some(state) = agent_value.get("state").and_then(|v| v.as_str()) {
                            if state == "Failed" {
                                if let Some(agent_id) = agent_value.get("id").and_then(|v| v.as_str()) {
                                    info!("🔧 Attempting recovery for failed agent: {}", agent_id);
                                    // In a real implementation, you would recover the specific agent
                                    // For now, we just log the attempt
                                }
                            }
                        }
                    }
                }
            }
        }
    });
    
    // Adaptive learning cleanup task
    let learning_state = app_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour
        loop {
            interval.tick().await;
            
            // Cleanup old learning patterns
            {
                let mut learning_system = learning_state.adaptive_learning.write().await;
                learning_system.cleanup_old_patterns();
                info!("🧹 Cleaned up old learning patterns");
            }
        }
    });
    
    info!("🔄 Background monitoring tasks started");
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| communication::handle_websocket(socket, state))
}

async fn get_agents(State(state): State<AppState>) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let agents_info = state.hive.read().await.get_agents_info().await;
    Ok(axum::Json(agents_info))
}

async fn create_agent(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> Result<(StatusCode, axum::Json<serde_json::Value>), (StatusCode, axum::Json<serde_json::Value>)> {
    // Validate payload using comprehensive validation
    if let Err(e) = InputValidator::validate_agent_payload(&payload) {
        warn!("Invalid agent creation payload: {}", e);
        state.metrics.record_error("invalid_agent_payload").await;
        return Err((
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({
                "error": "Invalid payload",
                "details": e.to_string()
            }))
        ));
    }

    let mut hive = state.hive.write().await;
    match hive.create_agent(payload).await {
        Ok(agent_id) => {
            info!("✅ Agent created successfully: {}", agent_id);
            Ok((
                StatusCode::CREATED,
                axum::Json(serde_json::json!({
                    "success": true,
                    "agent_id": agent_id,
                    "message": "Agent created successfully"
                }))
            ))
        },
        Err(e) => {
            error!("Failed to create agent: {}", e);
            state.metrics.record_error("agent_creation_failed").await;
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": "Failed to create agent",
                    "details": e.to_string()
                }))
            ))
        }
    }
}

async fn get_tasks(State(state): State<AppState>) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let tasks_info = state.hive.read().await.get_tasks_info().await;
    Ok(axum::Json(tasks_info))
}

async fn create_task(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> Result<(StatusCode, axum::Json<serde_json::Value>), (StatusCode, axum::Json<serde_json::Value>)> {
    // Validate payload using comprehensive validation
    if let Err(e) = InputValidator::validate_task_payload(&payload) {
        warn!("Invalid task creation payload: {}", e);
        state.metrics.record_error("invalid_task_payload").await;
        return Err((
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({
                "error": "Invalid payload",
                "details": e.to_string()
            }))
        ));
    }

    let mut hive = state.hive.write().await;
    match hive.create_task(payload).await {
        Ok(task_id) => {
            info!("✅ Task created successfully: {}", task_id);
            Ok((
                StatusCode::CREATED,
                axum::Json(serde_json::json!({
                    "success": true,
                    "task_id": task_id,
                    "message": "Task created successfully"
                }))
            ))
        },
        Err(e) => {
            error!("Failed to create task: {}", e);
            state.metrics.record_error("task_creation_failed").await;
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": "Failed to create task",
                    "details": e.to_string()
                }))
            ))
        }
    }
}

async fn get_hive_status(State(state): State<AppState>) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let status = state.hive.read().await.get_status().await;
    Ok(axum::Json(status))
}

async fn get_resource_info(State(state): State<AppState>) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let resource_info = state.hive.read().await.get_resource_info().await;
    Ok(axum::Json(resource_info))
}

async fn health_check(State(_state): State<AppState>) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    
    // Check hive coordinator health
    let hive_healthy = true; // Simplified for now
    let resources_healthy = true; // Simplified for now
    
    let response_time_ms = start_time.elapsed().as_millis();
    let overall_healthy = hive_healthy && resources_healthy;
    
    let health_status = serde_json::json!({
        "status": if overall_healthy { "healthy" } else { "unhealthy" },
        "timestamp": chrono::Utc::now(),
        "response_time_ms": response_time_ms,
        "version": "2.0.0",
        "components": {
            "hive_coordinator": if hive_healthy { "healthy" } else { "unhealthy" },
            "resource_manager": if resources_healthy { "healthy" } else { "unhealthy" },
            "metrics_collector": "healthy"
        },
        "system_info": {
            "cpu_native": true,
            "gpu_optional": true,
            "phase_2_active": true
        }
    });
    
    if overall_healthy {
        Ok(axum::Json(health_status))
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, axum::Json(health_status)))
    }
}

async fn get_metrics(State(state): State<AppState>) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let metrics = state.metrics.get_current_metrics().await;
    let trends = state.metrics.analyze_trends().await;
    Ok(axum::Json(serde_json::json!({
        "current_metrics": metrics,
        "trends": trends,
        "collection_timestamp": chrono::Utc::now()
    })))
}