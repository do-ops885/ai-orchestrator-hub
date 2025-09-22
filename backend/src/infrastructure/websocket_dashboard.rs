//! WebSocket Server for Real-time Dashboard Updates
//!
//! Provides WebSocket connectivity for streaming performance metrics
//! to the frontend dashboard in real-time.

use crate::infrastructure::performance_dashboard::{DashboardMetrics, PerformanceDashboard};
use crate::utils::error::{HiveError, HiveResult};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

/// WebSocket dashboard server configuration
#[derive(Debug, Clone)]
pub struct WebSocketDashboardConfig {
    /// Server listening port
    pub port: u16,
    /// Maximum concurrent WebSocket connections
    pub max_connections: usize,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    /// Enable CORS for cross-origin requests
    pub enable_cors: bool,
}

impl Default for WebSocketDashboardConfig {
    fn default() -> Self {
        Self {
            port: 8081,
            max_connections: 100,
            heartbeat_interval_secs: 30,
            enable_cors: true,
        }
    }
}

/// Connected WebSocket client information
#[derive(Debug, Clone)]
struct WebSocketClient {
    id: Uuid,
    connected_at: std::time::Instant,
    last_ping: std::time::Instant,
}

/// WebSocket dashboard server state
#[derive(Clone)]
pub struct WebSocketDashboardState {
    dashboard: Arc<PerformanceDashboard>,
    clients: Arc<RwLock<HashMap<Uuid, WebSocketClient>>>,
    config: WebSocketDashboardConfig,
}

/// WebSocket dashboard server
pub struct WebSocketDashboardServer {
    state: WebSocketDashboardState,
    metrics_receiver: broadcast::Receiver<DashboardMetrics>,
}

impl WebSocketDashboardServer {
    /// Create a new WebSocket dashboard server
    #[must_use] 
    pub fn new(
        dashboard: Arc<PerformanceDashboard>,
        config: WebSocketDashboardConfig,
    ) -> Self {
        let metrics_receiver = dashboard.subscribe();
        
        let state = WebSocketDashboardState {
            dashboard,
            clients: Arc::new(RwLock::new(HashMap::new())),
            config,
        };

        Self {
            state,
            metrics_receiver,
        }
    }

    /// Start the WebSocket server
    pub async fn start(mut self) -> HiveResult<()> {
        let app = self.create_router();
        
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.state.config.port))
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to bind WebSocket server: {e}"),
            })?;

        tracing::info!("WebSocket dashboard server started on port {}", self.state.config.port);

        // Start background tasks
        self.start_metrics_broadcaster().await;
        self.start_heartbeat_monitor().await;

        // Start the server
        axum::serve(listener, app)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("WebSocket server error: {e}"),
            })?;

        Ok(())
    }

    /// Create the Axum router
    fn create_router(&self) -> Router {
        let mut router = Router::new()
            .route("/ws", get(websocket_handler))
            .route("/health", get(health_check))
            .with_state(self.state.clone());

        if self.state.config.enable_cors {
            router = router.layer(CorsLayer::permissive());
        }

        router
    }

    /// Start broadcasting metrics to connected clients
    async fn start_metrics_broadcaster(&mut self) {
        let clients = Arc::clone(&self.state.clients);
        let mut receiver = self.metrics_receiver.resubscribe();

        tokio::spawn(async move {
            while let Ok(metrics) = receiver.recv().await {
                let clients_guard = clients.read().await;
                
                if !clients_guard.is_empty() {
                    let _message = match serde_json::to_string(&metrics) {
                        Ok(json) => json,
                        Err(e) => {
                            tracing::error!("Failed to serialize metrics: {}", e);
                            continue;
                        }
                    };

                    // Note: In a full implementation, we would need to store WebSocket senders
                    // and broadcast to them here. This is a simplified structure.
                    tracing::debug!("Broadcasting metrics to {} clients", clients_guard.len());
                }
            }
        });
    }

    /// Start heartbeat monitoring for connected clients
    async fn start_heartbeat_monitor(&self) {
        let clients = Arc::clone(&self.state.clients);
        let heartbeat_interval = self.state.config.heartbeat_interval_secs;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(heartbeat_interval)
            );

            loop {
                interval.tick().await;
                
                let mut clients_guard = clients.write().await;
                let now = std::time::Instant::now();
                
                // Remove clients that haven't responded to ping in 2x heartbeat interval
                let timeout_threshold = std::time::Duration::from_secs(heartbeat_interval * 2);
                
                clients_guard.retain(|_id, client| {
                    now.duration_since(client.last_ping) < timeout_threshold
                });
                
                tracing::debug!("Active WebSocket clients: {}", clients_guard.len());
            }
        });
    }
}

/// WebSocket upgrade handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketDashboardState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

/// Handle individual WebSocket connections
async fn handle_websocket(socket: WebSocket, state: WebSocketDashboardState) {
    let client_id = Uuid::new_v4();
    let client = WebSocketClient {
        id: client_id,
        connected_at: std::time::Instant::now(),
        last_ping: std::time::Instant::now(),
    };

    // Add client to active connections
    {
        let mut clients = state.clients.write().await;
        clients.insert(client_id, client);
    }

    tracing::info!("WebSocket client {} connected", client_id);

    // Handle the WebSocket connection
    if let Err(e) = handle_client_connection(socket, client_id, state.clone()).await {
        tracing::error!("WebSocket client {} error: {}", client_id, e);
    }

    // Remove client on disconnect
    {
        let mut clients = state.clients.write().await;
        clients.remove(&client_id);
    }

    tracing::info!("WebSocket client {} disconnected", client_id);
}

/// Handle individual client connection lifecycle
async fn handle_client_connection(
    mut socket: WebSocket,
    client_id: Uuid,
    state: WebSocketDashboardState,
) -> HiveResult<()> {
    // Send initial metrics
    let initial_metrics = state.dashboard.get_current_metrics().await?;
    let initial_message = serde_json::to_string(&initial_metrics)
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to serialize initial metrics: {e}"),
        })?;
    
    socket.send(Message::Text(initial_message)).await
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to send initial metrics: {e}"),
        })?;

    // Set up metrics subscription for this client
    let mut metrics_receiver = state.dashboard.subscribe();

    loop {
        tokio::select! {
            // Handle incoming messages from client
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_client_message(&text, client_id, &state).await {
                            tracing::error!("Error handling client message: {}", e);
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        // Respond to ping
                        socket.send(Message::Pong(data)).await
                            .map_err(|e| HiveError::OperationFailed {
                                reason: format!("Failed to send pong: {e}"),
                            })?;
                        
                        // Update last ping time
                        if let Some(client) = state.clients.write().await.get_mut(&client_id) {
                            client.last_ping = std::time::Instant::now();
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        tracing::info!("Client {} requested close", client_id);
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error for client {}: {}", client_id, e);
                        break;
                    }
                    None => {
                        tracing::info!("Client {} connection closed", client_id);
                        break;
                    }
                    _ => {} // Ignore other message types
                }
            }
            
            // Forward metrics updates to client
            metrics = metrics_receiver.recv() => {
                if let Ok(metrics_data) = metrics {
                    let message = serde_json::to_string(&metrics_data)
                        .map_err(|e| HiveError::OperationFailed {
                            reason: format!("Failed to serialize metrics: {e}"),
                        })?;
                    
                    if let Err(e) = socket.send(Message::Text(message)).await {
                        tracing::error!("Failed to send metrics to client {}: {}", client_id, e);
                        break;
                    }
                } else {
                    tracing::error!("Metrics channel closed for client {}", client_id);
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Handle messages from WebSocket clients
async fn handle_client_message(
    message: &str,
    client_id: Uuid,
    state: &WebSocketDashboardState,
) -> HiveResult<()> {
    #[derive(serde::Deserialize)]
    struct ClientMessage {
        action: String,
        data: Option<serde_json::Value>,
    }

    let client_msg: ClientMessage = serde_json::from_str(message)
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Invalid client message format: {e}"),
        })?;

    match client_msg.action.as_str() {
        "acknowledge_alert" => {
            if let Some(data) = client_msg.data {
                if let Some(alert_id) = data.get("alert_id").and_then(|v| v.as_str()) {
                    if let Ok(uuid) = Uuid::parse_str(alert_id) {
                        state.dashboard.acknowledge_alert(uuid).await?;
                        tracing::info!("Client {} acknowledged alert {}", client_id, alert_id);
                    }
                }
            }
        }
        "get_current_metrics" => {
            // This would trigger sending current metrics (already handled in the loop)
            tracing::debug!("Client {} requested current metrics", client_id);
        }
        "ping" => {
            // Update last ping time
            if let Some(client) = state.clients.write().await.get_mut(&client_id) {
                client.last_ping = std::time::Instant::now();
            }
        }
        _ => {
            tracing::warn!("Unknown action from client {}: {}", client_id, client_msg.action);
        }
    }

    Ok(())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "WebSocket Dashboard Server - Healthy"
}

/// WebSocket dashboard connection info
#[derive(Debug, serde::Serialize)]
pub struct DashboardConnectionInfo {
    pub total_connections: usize,
    pub active_connections: usize,
    pub server_uptime_secs: u64,
    pub metrics_broadcast_count: u64,
}

impl WebSocketDashboardState {
    /// Get connection information
    pub async fn get_connection_info(&self, start_time: std::time::Instant) -> DashboardConnectionInfo {
        let clients = self.clients.read().await;
        
        DashboardConnectionInfo {
            total_connections: clients.len(),
            active_connections: clients.len(), // Simplified - all connected clients are active
            server_uptime_secs: start_time.elapsed().as_secs(),
            metrics_broadcast_count: 0, // Would track this in a real implementation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::performance_dashboard::{DashboardConfig, PerformanceDashboard};

    #[tokio::test]
    async fn test_websocket_server_creation() {
        let dashboard_config = DashboardConfig::default();
        let dashboard = Arc::new(PerformanceDashboard::new(dashboard_config));
        let ws_config = WebSocketDashboardConfig::default();
        
        let _server = WebSocketDashboardServer::new(dashboard, ws_config);
        // Server creation should not panic
    }

    #[tokio::test] 
    async fn test_client_message_parsing() {
        let message = r#"{"action": "acknowledge_alert", "data": {"alert_id": "12345"}}"#;
        
        #[derive(serde::Deserialize)]
        struct ClientMessage {
            action: String,
            data: Option<serde_json::Value>,
        }
        
        let parsed: ClientMessage = match serde_json::from_str(message) {
            Ok(p) => p,
            Err(e) => panic!("Failed to parse test message: {}", e),
        };
        assert_eq!(parsed.action, "acknowledge_alert");
        assert!(parsed.data.is_some());
    }
}