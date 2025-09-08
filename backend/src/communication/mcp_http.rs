//! HTTP-based MCP Server for Multiagent Hive System
//!
//! This module provides HTTP endpoints for MCP (Model Context Protocol) communication,
//! allowing MCP clients to connect via HTTP instead of stdin/stdout.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info};

use crate::communication::mcp::{HiveMCPServer, MCPRequest, MCPResponse};
use crate::AppState;

/// HTTP handler for MCP requests
pub async fn handle_mcp_request(
    State(state): State<AppState>,
    Json(request): Json<MCPRequest>,
) -> Result<Json<MCPResponse>, (StatusCode, Json<Value>)> {
    debug!(
        "Received MCP HTTP request: {} (id: {:?})",
        request.method, request.id
    );

    // Create MCP server instance with the shared hive coordinator
    let hive = Arc::clone(&state.hive);
    let mcp_server = HiveMCPServer::new(hive);

    // Handle the request
    let response = mcp_server.handle_request(request).await;

    debug!("MCP HTTP response: {:?}", response.id);

    Ok(Json(response))
}

/// Create MCP HTTP router
pub fn create_mcp_router() -> Router<AppState> {
    Router::new()
        .route("/", post(handle_mcp_request))
        .route("/health", get(mcp_health_check))
}

/// Initialize MCP server for background operation
pub async fn start_mcp_background_service(_state: AppState) {
    info!("🚀 Starting MCP HTTP service as background component");

    // The MCP server is now available via HTTP endpoints
    // No additional background tasks needed since it's integrated into the main server
    info!("📡 MCP HTTP endpoint available at /mcp");
    info!("🔧 Available MCP tools: create_swarm_agent, assign_swarm_task, analyze_with_nlp, get_swarm_status, coordinate_agents");
}

/// Health check for MCP service
pub async fn mcp_health_check(State(state): State<AppState>) -> Json<Value> {
    let hive = state.hive.read().await;
    let status = hive.get_status().await;

    Json(serde_json::json!({
        "service": "mcp-http",
        "status": "healthy",
        "hive_connected": true,
        "total_agents": status.get("total_agents").unwrap_or(&Value::Null),
        "active_agents": status.get("active_agents").unwrap_or(&Value::Null),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
