//! # Multiagent Hive System - Main Server
//!
//! Entry point for the multiagent hive system backend server.
//!
//! This server implements a sophisticated multiagent hive system with:
//! - `RESTful` API for agent and task management
//! - WebSocket support for real-time communication
//! - Advanced neural processing capabilities
//! - Comprehensive monitoring and observability
//! - Production-ready error handling and configuration

use crate::utils::error::HiveResult;
use multiagent_hive::*;
use tracing::info;

#[tokio::main]
async fn main() -> HiveResult<()> {
    // Initialize the entire system
    let app_state = init::initialize_system().await?;

    // Start background monitoring and maintenance tasks
    server::start_background_tasks(app_state.clone()).await;

    // Create the application router
    let app = server::create_router(app_state.clone());

    // Start the server
    let bind_addr = format!(
        "{}:{}",
        app_state.config.server.host, app_state.config.server.port
    );
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    info!("🌐 Server running on http://{}", bind_addr);
    info!("📡 WebSocket endpoint: ws://{}/ws", bind_addr);
    info!("🔧 API endpoints: /api/agents, /api/tasks, /api/hive/status, /api/resources");
    info!("🤖 MCP HTTP endpoint: http://{}/api/mcp", bind_addr);
    info!("🔍 MCP health check: http://{}/api/mcp/health", bind_addr);

    // Graceful shutdown handling
    let shutdown_signal = async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Failed to install CTRL+C signal handler: {}", e);
                std::process::exit(1);
            }
        }
        info!("🛑 Shutdown signal received, gracefully stopping...");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    info!("✅ Multiagent Hive System stopped gracefully");
    Ok(())
}
