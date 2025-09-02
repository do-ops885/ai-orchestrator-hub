pub mod mcp;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    pub action: String,
    pub payload: Option<serde_json::Value>,
}

pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let client_id = Uuid::new_v4();

    tracing::info!("WebSocket client {} connected", client_id);

    // Send initial hive status
    if let Ok(status) = send_hive_status(&state).await {
        if let Ok(message) = serde_json::to_string(&status) {
            let _ = sender.send(Message::Text(message)).await;
        }
    }

    // Spawn a task to send periodic updates
    let state_clone = state.clone();
    let sender_clone = Arc::new(RwLock::new(sender));
    let sender_for_updates = Arc::clone(&sender_clone);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            interval.tick().await;

            // Send agent updates
            if let Ok(agents_info) = send_agents_update(&state_clone).await {
                if let Ok(message) = serde_json::to_string(&agents_info) {
                    let mut sender_guard = sender_for_updates.write().await;
                    if sender_guard.send(Message::Text(message)).await.is_err() {
                        break; // Client disconnected
                    }
                }
            }

            // Send metrics update
            if let Ok(metrics) = send_metrics_update(&state_clone).await {
                if let Ok(message) = serde_json::to_string(&metrics) {
                    let mut sender_guard = sender_for_updates.write().await;
                    if sender_guard.send(Message::Text(message)).await.is_err() {
                        break; // Client disconnected
                    }
                }
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    if let Ok(response) = handle_client_message(client_msg, &state).await {
                        if let Ok(response_text) = serde_json::to_string(&response) {
                            let mut sender_guard = sender_clone.write().await;
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
}

async fn handle_client_message(
    message: ClientMessage,
    state: &AppState,
) -> Result<WebSocketMessage, Box<dyn std::error::Error + Send + Sync>> {
    match message.action.as_str() {
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
                }),
                Err(e) => Ok(WebSocketMessage {
                    message_type: "error".to_string(),
                    data: serde_json::json!({
                        "error": e.to_string()
                    }),
                    timestamp: chrono::Utc::now(),
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
                }),
                Err(e) => Ok(WebSocketMessage {
                    message_type: "error".to_string(),
                    data: serde_json::json!({
                        "error": e.to_string()
                    }),
                    timestamp: chrono::Utc::now(),
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
            })
        }
        _ => Ok(WebSocketMessage {
            message_type: "error".to_string(),
            data: serde_json::json!({
                "error": format!("Unknown action: {}", message.action)
            }),
            timestamp: chrono::Utc::now(),
        }),
    }
}

async fn send_hive_status(
    state: &AppState,
) -> Result<WebSocketMessage, Box<dyn std::error::Error + Send + Sync>> {
    let hive = state.hive.read().await;
    let status = hive.get_status().await;
    Ok(WebSocketMessage {
        message_type: "hive_status".to_string(),
        data: status,
        timestamp: chrono::Utc::now(),
    })
}

async fn send_agents_update(
    state: &AppState,
) -> Result<WebSocketMessage, Box<dyn std::error::Error + Send + Sync>> {
    let hive = state.hive.read().await;
    let agents_info = hive.get_agents_info().await;
    Ok(WebSocketMessage {
        message_type: "agents_update".to_string(),
        data: agents_info,
        timestamp: chrono::Utc::now(),
    })
}

async fn send_metrics_update(
    state: &AppState,
) -> Result<WebSocketMessage, Box<dyn std::error::Error + Send + Sync>> {
    let hive = state.hive.read().await;
    let status = hive.get_status().await;
    Ok(WebSocketMessage {
        message_type: "metrics_update".to_string(),
        data: serde_json::json!({
            "metrics": status.get("metrics"),
            "swarm_center": status.get("swarm_center"),
            "total_energy": status.get("total_energy")
        }),
        timestamp: chrono::Utc::now(),
    })
}
