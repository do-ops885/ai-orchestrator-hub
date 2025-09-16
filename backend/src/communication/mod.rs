//! # Standardized Agent Communication Patterns
//!
//! This module provides standardized communication patterns for the multiagent hive system.
//! It includes:
//! - Consistent async message passing interfaces
//! - Standardized error handling and recovery
//! - Proper cancellation and timeout handling
//! - Performance optimizations for concurrent agent operations
//! - Resource management and connection pooling

pub mod mcp;
pub mod mcp_http;
pub mod patterns;
pub mod protocols;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::AppState;
use patterns::{CommunicationChannel, CommunicationResult, MessagePriority};
use protocols::MessageEnvelope;

/// Standardized WebSocket message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: Option<String>,
    pub priority: MessagePriority,
}

/// Standardized client message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    pub action: String,
    pub payload: Option<serde_json::Value>,
    pub correlation_id: Option<String>,
    pub timeout_ms: Option<u64>,
}

/// Communication manager for handling agent interactions
pub struct CommunicationManager {
    channels: Arc<RwLock<std::collections::HashMap<Uuid, CommunicationChannel>>>,
    message_queue: mpsc::UnboundedSender<MessageEnvelope>,
    message_receiver: mpsc::UnboundedReceiver<MessageEnvelope>,
    metrics: Arc<RwLock<CommunicationMetrics>>,
}

#[derive(Debug, Default, Clone)]
pub struct CommunicationMetrics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_failed: u64,
    pub average_response_time_ms: f64,
    pub active_connections: u64,
    pub total_bandwidth_bytes: u64,
}

impl CommunicationManager {
    /// Create a new communication manager
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            channels: Arc::new(RwLock::new(std::collections::HashMap::new())),
            message_queue: tx,
            message_receiver: rx,
            metrics: Arc::new(RwLock::new(CommunicationMetrics::default())),
        }
    }

    /// Send a message with standardized error handling and timeout
    pub async fn send_message(
        &self,
        envelope: MessageEnvelope,
        timeout_duration: Duration,
    ) -> CommunicationResult<()> {
        let start_time = std::time::Instant::now();

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.messages_sent += 1;
        }

        // Send message with timeout
        let send_result = self.message_queue.send(envelope);
        match timeout(timeout_duration, async { send_result }).await {
            Ok(Ok(())) => {
                // Update response time metrics
                let elapsed = start_time.elapsed().as_millis() as f64;
                let mut metrics = self.metrics.write().await;
                metrics.average_response_time_ms =
                    (metrics.average_response_time_ms + elapsed) / 2.0;
                Ok(())
            }
            Ok(Err(_)) => {
                let mut metrics = self.metrics.write().await;
                metrics.messages_failed += 1;
                Err(crate::utils::error::HiveError::Communication {
                    reason: "Message queue closed".to_string(),
                })
            }
            Err(_) => {
                let mut metrics = self.metrics.write().await;
                metrics.messages_failed += 1;
                Err(crate::utils::error::HiveError::Timeout {
                    reason: "Message send timeout".to_string(),
                })
            }
        }
    }

    /// Register a communication channel for an agent
    pub async fn register_channel(&self, agent_id: Uuid, channel: CommunicationChannel) {
        let mut channels = self.channels.write().await;
        channels.insert(agent_id, channel);

        let mut metrics = self.metrics.write().await;
        metrics.active_connections = channels.len() as u64;
    }

    /// Unregister a communication channel
    pub async fn unregister_channel(&self, agent_id: &Uuid) {
        let mut channels = self.channels.write().await;
        channels.remove(agent_id);

        let mut metrics = self.metrics.write().await;
        metrics.active_connections = channels.len() as u64;
    }

    /// Get communication metrics
    pub async fn get_metrics(&self) -> CommunicationMetrics {
        (*self.metrics.read().await).clone()
    }
}

/// Handle WebSocket connections with standardized patterns
pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (sender, mut receiver) = socket.split();
    let client_id = Uuid::new_v4();

    tracing::info!("WebSocket client {} connected", client_id);

    // Create communication manager for this connection
    let comm_manager = Arc::new(CommunicationManager::new());

    // Register the connection
    let channel = CommunicationChannel::WebSocket {
        sender: Arc::new(RwLock::new(sender)),
        client_id,
    };
    comm_manager.register_channel(client_id, channel).await;

    // Send initial hive status with standardized message format
    if let Ok(status) = send_hive_status(&state, &comm_manager, client_id).await {
        if let Ok(message) = serde_json::to_string(&status) {
            if let Some(channel) = comm_manager.channels.read().await.get(&client_id) {
                if let CommunicationChannel::WebSocket { sender, .. } = channel {
                    let mut sender_guard = sender.write().await;
                    let _ = sender_guard.send(Message::Text(message)).await;
                }
            }
        }
    }

    // Spawn task for periodic updates with cancellation support
    let state_clone = state.clone();
    let comm_manager_clone = Arc::clone(&comm_manager);
    let client_id_clone = client_id;

    let update_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Send agent updates
                    if let Ok(agents_info) = send_agents_update(&state_clone, &comm_manager_clone, client_id_clone).await {
                        if let Ok(message) = serde_json::to_string(&agents_info) {
                            if let Some(CommunicationChannel::WebSocket { sender, .. }) = comm_manager_clone
                                .channels
                                .read()
                                .await
                                .get(&client_id_clone)
                            {
                                let mut sender_guard = sender.write().await;
                                if sender_guard.send(Message::Text(message)).await.is_err() {
                                    break; // Client disconnected
                                }
                            }
                        }
                    }

                    // Send metrics update
                    if let Ok(metrics) = send_metrics_update(&state_clone, &comm_manager_clone, client_id_clone).await {
                        if let Ok(message) = serde_json::to_string(&metrics) {
                            if let Some(CommunicationChannel::WebSocket { sender, .. }) = comm_manager_clone
                                .channels
                                .read()
                                .await
                                .get(&client_id_clone)
                            {
                                let mut sender_guard = sender.write().await;
                                if sender_guard.send(Message::Text(message)).await.is_err() {
                                    break; // Client disconnected
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    // Handle incoming messages with standardized processing
    let comm_manager_clone = Arc::clone(&comm_manager);
    let message_handler = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        let timeout_duration = client_msg
                            .timeout_ms
                            .map(Duration::from_millis)
                            .unwrap_or(Duration::from_secs(30));

                        match tokio::time::timeout(
                            timeout_duration,
                            handle_client_message(
                                client_msg,
                                &state,
                                &comm_manager_clone,
                                client_id,
                            ),
                        )
                        .await
                        {
                            Ok(Ok(response)) => {
                                if let Ok(response_text) = serde_json::to_string(&response) {
                                    if let Some(CommunicationChannel::WebSocket {
                                        sender, ..
                                    }) = comm_manager_clone.channels.read().await.get(&client_id)
                                    {
                                        let mut sender_guard = sender.write().await;
                                        if sender_guard
                                            .send(Message::Text(response_text))
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                }
                            }
                            Ok(Err(e)) => {
                                tracing::error!("Error handling client message: {}", e);
                                let error_response = WebSocketMessage {
                                    message_type: "error".to_string(),
                                    data: serde_json::json!({
                                        "error": e.to_string()
                                    }),
                                    timestamp: chrono::Utc::now(),
                                    correlation_id: None,
                                    priority: MessagePriority::High,
                                };
                                if let Ok(error_text) = serde_json::to_string(&error_response) {
                                    if let Some(CommunicationChannel::WebSocket {
                                        sender, ..
                                    }) = comm_manager_clone.channels.read().await.get(&client_id)
                                    {
                                        let mut sender_guard = sender.write().await;
                                        let _ = sender_guard.send(Message::Text(error_text)).await;
                                    }
                                }
                            }
                            Err(_) => {
                                tracing::warn!(
                                    "Client message processing timeout for {}",
                                    client_id
                                );
                                let timeout_response = WebSocketMessage {
                                    message_type: "error".to_string(),
                                    data: serde_json::json!({
                                        "error": "Request timeout"
                                    }),
                                    timestamp: chrono::Utc::now(),
                                    correlation_id: None,
                                    priority: MessagePriority::High,
                                };
                                if let Ok(timeout_text) = serde_json::to_string(&timeout_response) {
                                    if let Some(CommunicationChannel::WebSocket {
                                        sender, ..
                                    }) = comm_manager_clone.channels.read().await.get(&client_id)
                                    {
                                        let mut sender_guard = sender.write().await;
                                        let _ =
                                            sender_guard.send(Message::Text(timeout_text)).await;
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("WebSocket client {} disconnected", client_id);
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket error for client {}: {}", client_id, e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = update_task => {},
        _ = message_handler => {},
    }

    // Cleanup
    comm_manager.unregister_channel(&client_id).await;
}

/// Handle client messages with standardized error handling
async fn handle_client_message(
    message: ClientMessage,
    state: &AppState,
    comm_manager: &CommunicationManager,
    client_id: Uuid,
) -> Result<WebSocketMessage, Box<dyn std::error::Error + Send + Sync>> {
    let correlation_id = message.correlation_id.clone();

    let result = match message.action.as_str() {
        "create_agent" => {
            let payload = message.payload.unwrap_or(serde_json::json!({}));
            let hive = state.hive.write().await;
            match hive.create_agent(payload).await {
                Ok(agent_id) => Ok(WebSocketMessage {
                    message_type: "agent_created".to_string(),
                    data: serde_json::json!({
                        "success": true,
                        "agent_id": agent_id
                    }),
                    timestamp: chrono::Utc::now(),
                    correlation_id,
                    priority: MessagePriority::Normal,
                }),
                Err(e) => Ok(WebSocketMessage {
                    message_type: "error".to_string(),
                    data: serde_json::json!({
                        "error": e.to_string()
                    }),
                    timestamp: chrono::Utc::now(),
                    correlation_id,
                    priority: MessagePriority::High,
                }),
            }
        }
        "create_task" => {
            let payload = message.payload.unwrap_or(serde_json::json!({}));
            let hive = state.hive.write().await;
            match hive.create_task(payload).await {
                Ok(task_id) => Ok(WebSocketMessage {
                    message_type: "task_created".to_string(),
                    data: serde_json::json!({
                        "success": true,
                        "task_id": task_id
                    }),
                    timestamp: chrono::Utc::now(),
                    correlation_id,
                    priority: MessagePriority::Normal,
                }),
                Err(e) => Ok(WebSocketMessage {
                    message_type: "error".to_string(),
                    data: serde_json::json!({
                        "error": e.to_string()
                    }),
                    timestamp: chrono::Utc::now(),
                    correlation_id,
                    priority: MessagePriority::High,
                }),
            }
        }
        "get_status" => {
            let hive = state.hive.read().await;
            let status = hive.get_status().await;
            Ok(WebSocketMessage {
                message_type: "hive_status".to_string(),
                data: status,
                timestamp: chrono::Utc::now(),
                correlation_id,
                priority: MessagePriority::Normal,
            })
        }
        _ => Ok(WebSocketMessage {
            message_type: "error".to_string(),
            data: serde_json::json!({
                "error": format!("Unknown action: {}", message.action)
            }),
            timestamp: chrono::Utc::now(),
            correlation_id,
            priority: MessagePriority::High,
        }),
    };

    // Update metrics
    let mut metrics = comm_manager.metrics.write().await;
    metrics.messages_received += 1;

    result
}

/// Send hive status with standardized message format
async fn send_hive_status(
    state: &AppState,
    comm_manager: &CommunicationManager,
    client_id: Uuid,
) -> Result<WebSocketMessage, Box<dyn std::error::Error + Send + Sync>> {
    let hive = state.hive.read().await;
    let status = hive.get_status().await;

    // Update bandwidth metrics
    let status_size = serde_json::to_string(&status)?.len() as u64;
    let mut metrics = comm_manager.metrics.write().await;
    metrics.total_bandwidth_bytes += status_size;

    Ok(WebSocketMessage {
        message_type: "hive_status".to_string(),
        data: status,
        timestamp: chrono::Utc::now(),
        correlation_id: Some(format!("status_{}", client_id)),
        priority: MessagePriority::Normal,
    })
}

/// Send agents update with standardized message format
async fn send_agents_update(
    state: &AppState,
    comm_manager: &CommunicationManager,
    client_id: Uuid,
) -> Result<WebSocketMessage, Box<dyn std::error::Error + Send + Sync>> {
    let hive = state.hive.read().await;
    let agents_info = hive.get_agents_info().await;

    // Update bandwidth metrics
    let agents_size = serde_json::to_string(&agents_info)?.len() as u64;
    let mut metrics = comm_manager.metrics.write().await;
    metrics.total_bandwidth_bytes += agents_size;

    Ok(WebSocketMessage {
        message_type: "agents_update".to_string(),
        data: agents_info,
        timestamp: chrono::Utc::now(),
        correlation_id: Some(format!("agents_{}", client_id)),
        priority: MessagePriority::Low,
    })
}

/// Send metrics update with standardized message format
async fn send_metrics_update(
    state: &AppState,
    comm_manager: &CommunicationManager,
    client_id: Uuid,
) -> Result<WebSocketMessage, Box<dyn std::error::Error + Send + Sync>> {
    let hive = state.hive.read().await;
    let status = hive.get_status().await;

    let metrics_data = serde_json::json!({
        "metrics": status.get("metrics"),
        "swarm_center": status.get("swarm_center"),
        "total_energy": status.get("total_energy")
    });

    // Update bandwidth metrics
    let metrics_size = serde_json::to_string(&metrics_data)?.len() as u64;
    let mut metrics = comm_manager.metrics.write().await;
    metrics.total_bandwidth_bytes += metrics_size;

    Ok(WebSocketMessage {
        message_type: "metrics_update".to_string(),
        data: metrics_data,
        timestamp: chrono::Utc::now(),
        correlation_id: Some(format!("metrics_{}", client_id)),
        priority: MessagePriority::Low,
    })
}
