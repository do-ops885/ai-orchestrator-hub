//! Standalone Dashboard Server Binary
//!
//! Runs the performance dashboard WebSocket server as a standalone service.

use multiagent_hive::infrastructure::{
    performance_dashboard::{DashboardConfig, PerformanceDashboard},
    websocket_dashboard::{WebSocketDashboardConfig, WebSocketDashboardServer},
};
use std::sync::Arc;
use tokio;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 Starting AI Orchestrator Hub Dashboard Server...");

    // Create dashboard configuration
    let dashboard_config = DashboardConfig {
        collection_interval_ms: 1000, // 1 second
        data_retention_hours: 24,     // 24 hours
        max_data_points: 300,         // 5 minutes of data
        enable_alerting: true,
        websocket_port: 8081,
        refresh_rate_ms: 1000,
    };

    // Create performance dashboard
    let dashboard = Arc::new(PerformanceDashboard::new(dashboard_config));
    
    // Establish performance baseline
    dashboard.set_baseline(PerformanceDashboard::collect_current_metrics().await).await?;
    
    info!("📊 Performance dashboard initialized with baseline metrics");

    // Create WebSocket server configuration
    let ws_config = WebSocketDashboardConfig {
        port: 8081,
        max_connections: 100,
        heartbeat_interval_secs: 30,
        enable_cors: true,
    };

    // Create and start WebSocket server
    let server = WebSocketDashboardServer::new(Arc::clone(&dashboard), ws_config);
    
    info!("🌐 WebSocket server starting on port 8081...");
    info!("📈 Dashboard available at: http://localhost:3000/dashboard");
    info!("🔌 WebSocket endpoint: ws://localhost:8081/ws");
    
    // Start the server (this will run indefinitely)
    server.start().await?;

    Ok(())
}