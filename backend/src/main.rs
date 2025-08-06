mod agent;
mod hive;
mod nlp;
mod task;
mod communication;
mod neural;
mod cpu_optimization;
mod resource_manager;
mod work_stealing_queue;

use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::Response,
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber;

use crate::hive::HiveCoordinator;

#[derive(Clone)]
pub struct AppState {
    pub hive: Arc<RwLock<HiveCoordinator>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Multiagent Hive System");

    // Initialize the hive coordinator
    let hive = Arc::new(RwLock::new(HiveCoordinator::new().await?));
    let app_state = AppState { hive };

    // Build the router
    let app = Router::new()
        .route("/", get(|| async { "Multiagent Hive System API" }))
        .route("/ws", get(websocket_handler))
        .route("/api/agents", get(get_agents).post(create_agent))
        .route("/api/tasks", get(get_tasks).post(create_task))
        .route("/api/hive/status", get(get_hive_status))
        .route("/api/resources", get(get_resource_info)) // Phase 2: Resource monitoring
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    info!("Server running on http://0.0.0.0:3001");
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| communication::handle_websocket(socket, state))
}

async fn get_agents(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    let hive = state.hive.read().await;
    axum::Json(hive.get_agents_info().await)
}

async fn create_agent(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> Result<axum::Json<serde_json::Value>, axum::Json<serde_json::Value>> {
    let hive = state.hive.write().await;
    match hive.create_agent(payload).await {
        Ok(agent_id) => Ok(axum::Json(serde_json::json!({
            "success": true,
            "agent_id": agent_id
        }))),
        Err(e) => Err(axum::Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

async fn get_tasks(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    let hive = state.hive.read().await;
    axum::Json(hive.get_tasks_info().await)
}

async fn create_task(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> Result<axum::Json<serde_json::Value>, axum::Json<serde_json::Value>> {
    let hive = state.hive.write().await;
    match hive.create_task(payload).await {
        Ok(task_id) => Ok(axum::Json(serde_json::json!({
            "success": true,
            "task_id": task_id
        }))),
        Err(e) => Err(axum::Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

async fn get_hive_status(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    let hive = state.hive.read().await;
    axum::Json(hive.get_status().await)
}

async fn get_resource_info(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    let hive = state.hive.read().await;
    axum::Json(hive.get_resource_info().await)
}