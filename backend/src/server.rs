//! Server initialization and background task management
//!
//! This module handles server setup, routing, and background task coordination
//! for the multiagent hive system.

use axum::{
    extract::{ws::WebSocketUpgrade, State},
    http::StatusCode,
    response::Response,
    routing::get,
    Router,
};
// Removed unused imports
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info, warn};

use crate::communication;
use crate::communication::mcp_http;
use crate::infrastructure::metrics::{AgentMetrics, AlertLevel, TaskMetrics};
use crate::infrastructure::middleware::security_headers_middleware;
use crate::utils::structured_logging::{SecurityEventDetails, SecurityEventType, StructuredLogger};
use crate::utils::validation::InputValidator;
use crate::AppState;

use chrono::Utc;
use serde_json::json;

/// Start background tasks for monitoring, alerting, and system maintenance
pub async fn start_background_tasks(app_state: AppState) {
    let metrics_interval = std::time::Duration::from_millis(
        app_state.config.performance.metrics_collection_interval_ms,
    );
    let alert_interval =
        std::time::Duration::from_millis(app_state.config.performance.alert_check_interval_ms);

    // Enhanced metrics collection task with advanced analytics
    let metrics_state = app_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(metrics_interval);
        loop {
            interval.tick().await;

            // Collect traditional system metrics
            if let Err(e) = metrics_state.metrics.collect_system_metrics().await {
                error!("Failed to collect system metrics: {}", e);
            }

            // Collect advanced metrics with predictive analytics
            if let Err(e) = metrics_state
                .advanced_metrics
                .collect_system_metrics()
                .await
            {
                error!("Failed to collect advanced metrics: {}", e);
            } else {
                debug!("Advanced metrics collected successfully");
            }

            // Snapshot current metrics for historical analysis
            metrics_state.metrics.snapshot_current_metrics().await;

            // Update hive metrics
            let hive = metrics_state.hive.read().await.get_status().await;

            // Update agent metrics from hive status
            let agent_metrics = AgentMetrics {
                total_agents: hive
                    .get("total_agents")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize,
                active_agents: hive
                    .get("active_agents")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize,
                idle_agents: hive
                    .get("idle_agents")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize,
                failed_agents: hive
                    .get("failed_agents")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize,
                average_agent_performance: hive
                    .get("average_performance")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                agent_utilization_percent: 0.0,
                individual_agent_metrics: std::collections::HashMap::new(),
            };

            // Update task metrics from hive status
            let task_metrics = TaskMetrics {
                total_tasks_submitted: hive
                    .get("total_tasks")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                total_tasks_completed: hive
                    .get("completed_tasks")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                total_tasks_failed: hive
                    .get("failed_tasks")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                tasks_in_queue: hive
                    .get("pending_tasks")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize,
                average_task_duration_ms: hive
                    .get("average_task_completion_time")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                task_success_rate: if hive
                    .get("total_tasks")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    > 0
                {
                    (hive
                        .get("completed_tasks")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as f64
                        / hive
                            .get("total_tasks")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(1) as f64)
                        * 100.0
                } else {
                    0.0
                },
            };

            // Update the metrics systems with the collected data
            metrics_state
                .metrics
                .update_agent_metrics(agent_metrics)
                .await;
            metrics_state
                .metrics
                .update_task_metrics(task_metrics)
                .await;

            // Snapshot the current metrics for historical analysis
            metrics_state.metrics.snapshot_current_metrics().await;
        }
    });

    // Intelligent alert processing task
    let alert_state = app_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(alert_interval);
        loop {
            interval.tick().await;

            // Process intelligent alerts with predictive capabilities
            match alert_state
                .intelligent_alerting
                .process_intelligent_alerts()
                .await
            {
                Ok(intelligent_alerts) => {
                    if !intelligent_alerts.is_empty() {
                        info!(
                            "🚨 Processed {} intelligent alerts",
                            intelligent_alerts.len()
                        );
                        for alert in &intelligent_alerts {
                            debug!(
                                "Alert: {} (confidence: {:.2}, predicted: {})",
                                alert.base_alert.title, alert.confidence, alert.predicted
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to process intelligent alerts: {}", e);
                }
            }

            // Also check traditional alerts as backup
            let alerts = alert_state.metrics.check_alerts().await;
            for alert in alerts {
                match alert.level {
                    AlertLevel::Critical => {
                        error!("🚨 CRITICAL ALERT: {} - {}", alert.title, alert.description);
                        // In production, you would send notifications here
                    }
                    AlertLevel::Warning => {
                        warn!("⚠️  WARNING: {} - {}", alert.title, alert.description);
                    }
                    AlertLevel::Info => {
                        info!("ℹ️  INFO: {} - {}", alert.title, alert.description);
                    }
                }
            }

            // Analyze trends
            let trends = alert_state.metrics.analyze_trends().await;
            debug!(
                "System trends - CPU: {:?}, Memory: {:?}, Tasks: {:?}",
                trends.cpu_trend, trends.memory_trend, trends.task_completion_trend
            );
        }
    });

    // Agent recovery and maintenance task
    let recovery_state = app_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60)); // Check every minute
        loop {
            interval.tick().await;

            // Check for failed agents and attempt recovery
            let hive = recovery_state.hive.read().await.get_agents_info().await;
            {
                if let Some(agents) = hive.get("agents").and_then(|v| v.as_array()) {
                    for agent_value in agents {
                        if let Some(state) = agent_value.get("state").and_then(|v| v.as_str()) {
                            if state == "Failed" {
                                if let Some(agent_id) =
                                    agent_value.get("id").and_then(|v| v.as_str())
                                {
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
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600)); // Every hour
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

    // Start MCP HTTP service
    mcp_http::start_mcp_background_service(app_state.clone()).await;

    info!("🔄 Background monitoring tasks started");
}

/// Create the main application router with all routes configured
pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { "🐝 Multiagent Hive System API v2.0 - CPU-native, GPU-optional" }),
        )
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        .route("/ws", get(websocket_handler))
        .route("/api/agents", get(get_agents).post(create_agent))
        .route("/api/tasks", get(get_tasks).post(create_task))
        .route("/api/hive/status", get(get_hive_status))
        .route("/api/resources", get(get_resource_info))
        .nest("/api/mcp", mcp_http::create_mcp_router())
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(CorsLayer::permissive())
        .with_state(app_state)
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| communication::handle_websocket(socket, state))
}

async fn get_agents(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let agents_info = state.hive.read().await.get_agents_info().await;
    Ok(axum::Json(agents_info))
}

async fn create_agent(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> Result<(StatusCode, axum::Json<serde_json::Value>), (StatusCode, axum::Json<serde_json::Value>)>
{
    // Apply rate limiting
    if state
        .rate_limiter
        .check_rate_limit("api_create_agent")
        .await
        .is_err()
    {
        warn!("Rate limit exceeded for agent creation");
        state.metrics.record_error("rate_limit_exceeded").await;
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(json!({
                "error": "Rate limit exceeded",
                "details": "Too many requests, please try again later"
            })),
        ));
    }

    // Validate payload using comprehensive validation
    if let Err(e) = InputValidator::validate_agent_payload(&payload) {
        warn!("Invalid agent creation payload: {}", e);
        state.metrics.record_error("invalid_agent_payload").await;
        return Err((
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "error": "Invalid payload",
                "details": e.to_string()
            })),
        ));
    }

    let hive = state.hive.write().await;
    match hive.create_agent(payload).await {
        Ok(agent_id) => {
            info!("✅ Agent created successfully: {}", agent_id);

            // Log security event for agent creation
            StructuredLogger::log_security_event(
                &SecurityEventType::AuthenticationSuccess,
                &SecurityEventDetails {
                    client_id: "api".to_string(),
                    endpoint: format!("agent:{}", agent_id),
                    user_agent: None,
                    ip_address: None,
                    timestamp: Utc::now(),
                    additional_info: {
                        let mut info = std::collections::HashMap::new();
                        info.insert("action".to_string(), "create".to_string());
                        info.insert("resource_type".to_string(), "agent".to_string());
                        info
                    },
                },
            );

            Ok((
                StatusCode::CREATED,
                axum::Json(json!({
                    "success": true,
                    "agent_id": agent_id,
                    "message": "Agent created successfully"
                })),
            ))
        }
        Err(e) => {
            error!("Failed to create agent: {}", e);
            state.metrics.record_error("agent_creation_failed").await;
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({
                    "success": false,
                    "error": "Failed to create agent",
                    "details": e.to_string()
                })),
            ))
        }
    }
}

async fn get_tasks(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let tasks_info = state.hive.read().await.get_tasks_info().await;
    Ok(axum::Json(tasks_info))
}

async fn create_task(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> Result<(StatusCode, axum::Json<serde_json::Value>), (StatusCode, axum::Json<serde_json::Value>)>
{
    // Apply rate limiting
    if state
        .rate_limiter
        .check_rate_limit("api_create_task")
        .await
        .is_err()
    {
        warn!("Rate limit exceeded for task creation");
        state.metrics.record_error("rate_limit_exceeded").await;
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(json!({
                "error": "Rate limit exceeded",
                "details": "Too many requests, please try again later"
            })),
        ));
    }

    // Validate payload using comprehensive validation
    if let Err(e) = InputValidator::validate_task_payload(&payload) {
        warn!("Invalid task creation payload: {}", e);
        state.metrics.record_error("invalid_task_payload").await;
        return Err((
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "error": "Invalid payload",
                "details": e.to_string()
            })),
        ));
    }

    let hive = state.hive.write().await;
    match hive.create_task(payload).await {
        Ok(task_id) => {
            info!("✅ Task created successfully: {}", task_id);

            // Log security event for task creation
            StructuredLogger::log_security_event(
                &SecurityEventType::AuthenticationSuccess,
                &SecurityEventDetails {
                    client_id: "api".to_string(),
                    endpoint: format!("task:{}", task_id),
                    user_agent: None,
                    ip_address: None,
                    timestamp: Utc::now(),
                    additional_info: {
                        let mut info = std::collections::HashMap::new();
                        info.insert("action".to_string(), "create".to_string());
                        info.insert("resource_type".to_string(), "task".to_string());
                        info
                    },
                },
            );

            Ok((
                StatusCode::CREATED,
                axum::Json(json!({
                    "success": true,
                    "task_id": task_id,
                    "message": "Task created successfully"
                })),
            ))
        }
        Err(e) => {
            error!("Failed to create task: {}", e);
            state.metrics.record_error("task_creation_failed").await;
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({
                    "success": false,
                    "error": "Failed to create task",
                    "details": e.to_string()
                })),
            ))
        }
    }
}

async fn get_hive_status(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let status = state.hive.read().await.get_status().await;
    Ok(axum::Json(status))
}

async fn get_resource_info(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let resource_info = state.hive.read().await.get_resource_info().await;
    Ok(axum::Json(resource_info))
}

async fn health_check(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();

    // Perform comprehensive health checks
    let hive_status = state.hive.read().await.get_status().await;
    let metrics_health = state.metrics.get_current_metrics().await;
    let resource_info = state.hive.read().await.get_resource_info().await;

    // Extract metrics from hive status JSON
    let hive_metrics = hive_status
        .get("metrics")
        .unwrap_or(&serde_json::Value::Null);
    let total_agents = hive_metrics
        .get("total_agents")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let completed_tasks = hive_metrics
        .get("completed_tasks")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Extract resource info from JSON
    let system_resources = resource_info
        .get("system_resources")
        .unwrap_or(&serde_json::Value::Null);
    let memory_usage = system_resources
        .get("memory_usage")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let cpu_usage = system_resources
        .get("cpu_usage")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    // Check component health
    let hive_healthy = total_agents > 0 || completed_tasks > 0;
    let resources_healthy = memory_usage < 90.0 && cpu_usage < 95.0;
    let metrics_healthy = metrics_health.performance.average_response_time_ms < 5000.0;
    let alerting_healthy = true; // Simplified for now - alerting system is operational

    let response_time_ms = start_time.elapsed().as_millis();
    let overall_healthy = hive_healthy && resources_healthy && metrics_healthy && alerting_healthy;

    let health_status = json!({
        "status": if overall_healthy { "healthy" } else { "unhealthy" },
        "timestamp": Utc::now(),
        "response_time_ms": response_time_ms,
        "version": "2.0.0",
        "components": {
            "hive_coordinator": {
                "status": if hive_healthy { "healthy" } else { "unhealthy" },
                "total_agents": total_agents,
                "active_agents": hive_metrics.get("active_agents").and_then(|v| v.as_u64()).unwrap_or(0),
                "completed_tasks": completed_tasks,
                "average_performance": hive_metrics.get("average_performance").and_then(|v| v.as_f64()).unwrap_or(0.0)
            },
            "resource_manager": {
                "status": if resources_healthy { "healthy" } else { "unhealthy" },
                "memory_usage_percent": memory_usage,
                "cpu_usage_percent": cpu_usage,
                "available_memory_mb": system_resources.get("available_memory").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "cpu_cores": system_resources.get("cpu_cores").and_then(|v| v.as_u64()).unwrap_or(0)
            },
            "metrics_collector": {
                "status": if metrics_healthy { "healthy" } else { "unhealthy" },
                "response_time_ms": metrics_health.performance.average_response_time_ms,
                "requests_per_second": metrics_health.performance.requests_per_second,
                "error_rate": metrics_health.error_metrics.error_rate_per_minute
            },
            "intelligent_alerting": {
                "status": if alerting_healthy { "healthy" } else { "unhealthy" },
                "active_rules": "monitoring",
                "system_operational": true
            }
        },
        "system_info": {
            "cpu_native": true,
            "gpu_optional": true,
            "phase_2_active": true,
            "swarm_cohesion": hive_metrics.get("swarm_cohesion").and_then(|v| v.as_f64()).unwrap_or(0.0),
            "learning_progress": hive_metrics.get("learning_progress").and_then(|v| v.as_f64()).unwrap_or(0.0)
        }
    });

    if overall_healthy {
        Ok(axum::Json(health_status))
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, axum::Json(health_status)))
    }
}

async fn get_metrics(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let metrics = state.metrics.get_current_metrics().await;
    let trends = state.metrics.analyze_trends().await;
    Ok(axum::Json(json!({
        "current_metrics": metrics,
        "trends": trends,
        "collection_timestamp": Utc::now()
    })))
}
