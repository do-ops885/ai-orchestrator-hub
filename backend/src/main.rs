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
use tracing::{info, warn, error, Level};
use tracing_subscriber;

use crate::core::HiveCoordinator;
use crate::utils::HiveConfig;
use crate::infrastructure::MetricsCollector;
use crate::utils::InputValidator;

#[derive(Clone)]
pub struct AppState {
    pub hive: Arc<RwLock<HiveCoordinator>>,
    pub config: Arc<HiveConfig>,
    pub metrics: Arc<MetricsCollector>,
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

    info!("🚀 Starting Multiagent Hive System v2.0");
    info!("📊 Configuration loaded: CPU-native, GPU-optional");

    // Initialize metrics collector
    let metrics = Arc::new(MetricsCollector::new(1000));

    // Initialize the hive coordinator with configuration
    let hive = match HiveCoordinator::new().await {
        Ok(coordinator) => Arc::new(RwLock::new(coordinator)),
        Err(e) => {
            error!("Failed to initialize hive coordinator: {}", e);
            return Err(e);
        }
    };

    let app_state = AppState { 
        hive,
        config: config.clone(),
        metrics: metrics.clone(),
    };

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