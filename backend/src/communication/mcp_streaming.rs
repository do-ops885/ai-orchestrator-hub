use super::mcp::MCPToolHandler;
use super::mcp_unified_error::MCPUnifiedError;
use anyhow::Result;
use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{Duration, Instant};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// MCP Streaming Response System (Phase 3.1)
/// 
/// Provides streaming capabilities for long-running MCP operations,
/// allowing clients to receive progressive updates and results.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPStreamingResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub stream: bool,
    pub stream_id: String,
    pub data: Option<Value>,
    pub done: bool,
    pub progress: Option<StreamingProgress>,
    pub metadata: Option<StreamingMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingProgress {
    pub current: u64,
    pub total: Option<u64>,
    pub percentage: Option<f64>,
    pub stage: String,
    pub estimated_completion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingMetadata {
    pub started_at: String,
    pub last_update: String,
    pub operation_type: String,
    pub total_chunks: Option<u64>,
    pub chunk_size: Option<u64>,
}

/// Streaming operation status
#[derive(Debug, Clone, PartialEq)]
pub enum StreamStatus {
    Started,
    InProgress,
    Completed,
    Failed(String),
    Cancelled,
}

/// Stream chunk containing partial results
#[derive(Debug, Clone, Serialize)]
pub struct StreamChunk {
    pub chunk_id: u64,
    pub data: Value,
    pub is_final: bool,
    pub timestamp: String,
}

/// Streaming tool handler wrapper
pub struct StreamingMCPToolHandler<T: MCPToolHandler> {
    inner: T,
    chunk_size: usize,
    max_stream_duration: Duration,
}

impl<T: MCPToolHandler> StreamingMCPToolHandler<T> {
    pub fn new(inner: T, chunk_size: usize, max_stream_duration: Duration) -> Self {
        Self {
            inner,
            chunk_size,
            max_stream_duration,
        }
    }

    /// Check if this operation should be streamed
    fn should_stream(&self, params: &Value) -> bool {
        // Check for streaming hints in parameters
        if let Some(stream_hint) = params.get("stream") {
            return stream_hint.as_bool().unwrap_or(false);
        }

        // Check for operations that typically benefit from streaming
        let operation_hints = [
            "batch_create", "analyze_large", "process_bulk", "generate_report",
            "workflow", "analytics", "migration", "backup"
        ];
        
        let description = self.inner.get_description().to_lowercase();
        operation_hints.iter().any(|hint| description.contains(hint))
    }
}

#[async_trait]
impl<T: MCPToolHandler + Clone + 'static> MCPToolHandler for StreamingMCPToolHandler<T> {
    async fn execute(&self, params: &Value) -> Result<Value> {
        if !self.should_stream(params) {
            // Execute normally if streaming is not needed
            return self.inner.execute(params).await;
        }

        // Create streaming operation
        let stream_id = Uuid::new_v4().to_string();
        let _start_time = Instant::now();
        
        info!("Starting streaming operation: {}", stream_id);

        // For streaming, return a stream identifier and start background processing
        let stream_manager = StreamManager::new();
        let stream_handle = stream_manager.create_stream(stream_id.clone()).await;

        // Spawn background task for streaming execution
        let inner_handler = self.inner.clone();
        let params_clone = params.clone();
        let stream_id_clone = stream_id.clone();
        let max_duration = self.max_stream_duration;
        
        tokio::spawn(async move {
            let mut progress = StreamingProgress {
                current: 0,
                total: None,
                percentage: Some(0.0),
                stage: "Starting".to_string(),
                estimated_completion: None,
            };

            // Send initial progress
            let initial_response = MCPStreamingResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                stream: true,
                stream_id: stream_id_clone.clone(),
                data: Some(json!({"status": "started"})),
                done: false,
                progress: Some(progress.clone()),
                metadata: Some(StreamingMetadata {
                    started_at: chrono::Utc::now().to_rfc3339(),
                    last_update: chrono::Utc::now().to_rfc3339(),
                    operation_type: inner_handler.get_description(),
                    total_chunks: None,
                    chunk_size: None,
                }),
            };

            if let Err(e) = stream_handle.send_update(initial_response).await {
                warn!("Failed to send initial streaming update: {}", e);
                return;
            }

            // Execute the actual operation with progress simulation
            match tokio::time::timeout(max_duration, inner_handler.execute(&params_clone)).await {
                Ok(Ok(result)) => {
                    // Send final result
                    progress.current = 100;
                    progress.percentage = Some(100.0);
                    progress.stage = "Completed".to_string();

                    let final_response = MCPStreamingResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        stream: true,
                        stream_id: stream_id_clone,
                        data: Some(result),
                        done: true,
                        progress: Some(progress),
                        metadata: Some(StreamingMetadata {
                            started_at: chrono::Utc::now().to_rfc3339(),
                            last_update: chrono::Utc::now().to_rfc3339(),
                            operation_type: inner_handler.get_description(),
                            total_chunks: Some(1),
                            chunk_size: None,
                        }),
                    };

                    if let Err(e) = stream_handle.send_final(final_response).await {
                        warn!("Failed to send final streaming result: {}", e);
                    }
                }
                Ok(Err(e)) => {
                    // Send error
                    let error_response = MCPStreamingResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        stream: true,
                        stream_id: stream_id_clone,
                        data: Some(json!({"error": e.to_string()})),
                        done: true,
                        progress: None,
                        metadata: None,
                    };

                    if let Err(e) = stream_handle.send_error(error_response).await {
                        warn!("Failed to send streaming error: {}", e);
                    }
                }
                Err(_) => {
                    // Timeout
                    let timeout_response = MCPStreamingResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        stream: true,
                        stream_id: stream_id_clone,
                        data: Some(json!({"error": "Operation timed out"})),
                        done: true,
                        progress: None,
                        metadata: None,
                    };

                    if let Err(e) = stream_handle.send_error(timeout_response).await {
                        warn!("Failed to send streaming timeout: {}", e);
                    }
                }
            }
        });

        Ok(json!({
            "stream_id": stream_id,
            "status": "started",
            "streaming": true,
            "started_at": chrono::Utc::now().to_rfc3339()
        }))
    }
    
    fn get_schema(&self) -> Value {
        let mut schema = self.inner.get_schema();
        
        // Add streaming parameters to schema
        if let Some(properties) = schema.get_mut("properties") {
            if let Some(props_obj) = properties.as_object_mut() {
                props_obj.insert("stream".to_string(), json!({
                    "type": "boolean",
                    "description": "Enable streaming mode for long-running operations",
                    "default": false
                }));
            }
        }
        
        schema
    }
    
    fn get_description(&self) -> String {
        format!("{} (streaming-enabled)", self.inner.get_description())
    }
}

/// Stream manager for handling multiple concurrent streams
pub struct StreamManager {
    active_streams: Arc<RwLock<HashMap<String, StreamHandle>>>,
}

impl StreamManager {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            active_streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_stream(&self, stream_id: String) -> StreamHandle {
        let (tx, _rx) = broadcast::channel(1000);
        let handle = StreamHandle::new(stream_id.clone(), tx);
        
        self.active_streams.write().await.insert(stream_id, handle.clone());
        handle
    }

    pub async fn get_stream(&self, stream_id: &str) -> Option<StreamHandle> {
        self.active_streams.read().await.get(stream_id).cloned()
    }

    pub async fn close_stream(&self, stream_id: &str) {
        self.active_streams.write().await.remove(stream_id);
        debug!("Closed stream: {}", stream_id);
    }

    pub async fn list_active_streams(&self) -> Vec<String> {
        self.active_streams.read().await.keys().cloned().collect()
    }
}

/// Handle for individual stream operations
#[derive(Clone)]
pub struct StreamHandle {
    stream_id: String,
    sender: broadcast::Sender<MCPStreamingResponse>,
    status: Arc<RwLock<StreamStatus>>,
}

impl StreamHandle {
    #[must_use] 
    pub fn new(stream_id: String, sender: broadcast::Sender<MCPStreamingResponse>) -> Self {
        Self {
            stream_id,
            sender,
            status: Arc::new(RwLock::new(StreamStatus::Started)),
        }
    }

    pub async fn send_update(&self, response: MCPStreamingResponse) -> Result<(), MCPUnifiedError> {
        *self.status.write().await = StreamStatus::InProgress;
        
        self.sender.send(response).map_err(|e| {
            MCPUnifiedError::Internal {
                message: format!("Failed to send stream update: {e}"),
                source_error: Some(e.to_string()),
                recovery_suggestion: Some("Check if stream receiver is still active".to_string()),
            }
        })?;
        
        Ok(())
    }

    pub async fn send_final(&self, response: MCPStreamingResponse) -> Result<(), MCPUnifiedError> {
        *self.status.write().await = StreamStatus::Completed;
        
        self.sender.send(response).map_err(|e| {
            MCPUnifiedError::Internal {
                message: format!("Failed to send final stream result: {e}"),
                source_error: Some(e.to_string()),
                recovery_suggestion: Some("Check if stream receiver is still active".to_string()),
            }
        })?;
        
        Ok(())
    }

    pub async fn send_error(&self, response: MCPStreamingResponse) -> Result<(), MCPUnifiedError> {
        *self.status.write().await = StreamStatus::Failed("Operation failed".to_string());
        
        self.sender.send(response).map_err(|e| {
            MCPUnifiedError::Internal {
                message: format!("Failed to send stream error: {e}"),
                source_error: Some(e.to_string()),
                recovery_suggestion: Some("Check if stream receiver is still active".to_string()),
            }
        })?;
        
        Ok(())
    }

    pub async fn get_status(&self) -> StreamStatus {
        self.status.read().await.clone()
    }

    #[must_use] 
    pub fn subscribe(&self) -> broadcast::Receiver<MCPStreamingResponse> {
        self.sender.subscribe()
    }
}

/// Tool for managing streaming operations
pub struct StreamingControlTool {
    stream_manager: Arc<StreamManager>,
}

impl StreamingControlTool {
    #[must_use] 
    pub fn new(stream_manager: Arc<StreamManager>) -> Self {
        Self { stream_manager }
    }
}

#[async_trait]
impl MCPToolHandler for StreamingControlTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let action = params
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: action"))?;

        match action {
            "list" => {
                let streams = self.stream_manager.list_active_streams().await;
                Ok(json!({
                    "active_streams": streams,
                    "count": streams.len()
                }))
            }
            "status" => {
                let stream_id = params
                    .get("stream_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: stream_id for status action"))?;

                if let Some(handle) = self.stream_manager.get_stream(stream_id).await {
                    let status = handle.get_status().await;
                    Ok(json!({
                        "stream_id": stream_id,
                        "status": format!("{:?}", status)
                    }))
                } else {
                    Ok(json!({
                        "stream_id": stream_id,
                        "status": "not_found"
                    }))
                }
            }
            "close" => {
                let stream_id = params
                    .get("stream_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: stream_id for close action"))?;

                self.stream_manager.close_stream(stream_id).await;
                Ok(json!({
                    "stream_id": stream_id,
                    "status": "closed"
                }))
            }
            _ => Err(anyhow::anyhow!("Invalid action: {action}. Supported actions: list, status, close"))
        }
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list", "status", "close"],
                    "description": "Action to perform on streaming operations"
                },
                "stream_id": {
                    "type": "string",
                    "description": "Stream ID (required for status and close actions)"
                }
            },
            "required": ["action"],
            "oneOf": [
                {
                    "properties": {
                        "action": {"const": "list"}
                    }
                },
                {
                    "properties": {
                        "action": {"const": "status"}
                    },
                    "required": ["stream_id"]
                },
                {
                    "properties": {
                        "action": {"const": "close"}
                    },
                    "required": ["stream_id"]
                }
            ]
        })
    }

    fn get_description(&self) -> String {
        "Manage streaming operations: list active streams, check status, or close streams".to_string()
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockSlowTool;

    #[async_trait]
    impl MCPToolHandler for MockSlowTool {
        async fn execute(&self, _params: &Value) -> Result<Value> {
            // Simulate slow operation
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(json!({"result": "completed"}))
        }

        fn get_schema(&self) -> Value {
            json!({"type": "object"})
        }

        fn get_description(&self) -> String {
            "batch_create_test_operation".to_string()
        }
    }

    #[tokio::test]
    async fn test_streaming_detection() {
        let tool = MockSlowTool;
        let streaming_tool = StreamingMCPToolHandler::new(
            tool,
            1000,
            Duration::from_secs(30),
        );

        // Should detect streaming need based on description
        let params = json!({});
        assert!(streaming_tool.should_stream(&params));

        // Should detect explicit streaming request
        let params_with_stream = json!({"stream": true});
        assert!(streaming_tool.should_stream(&params_with_stream));
    }

    #[tokio::test]
    async fn test_stream_manager() {
        let manager = StreamManager::new();
        let stream_id = "test-stream-123".to_string();

        let handle = manager.create_stream(stream_id.clone()).await;
        assert_eq!(handle.stream_id, stream_id);

        let retrieved = manager.get_stream(&stream_id).await;
        assert!(retrieved.is_some());

        let active_streams = manager.list_active_streams().await;
        assert_eq!(active_streams.len(), 1);
        assert_eq!(active_streams[0], stream_id);

        manager.close_stream(&stream_id).await;
        let active_streams_after = manager.list_active_streams().await;
        assert_eq!(active_streams_after.len(), 0);
    }

    #[tokio::test]
    async fn test_streaming_control_tool() {
        let manager = Arc::new(StreamManager::new());
        let control_tool = StreamingControlTool::new(Arc::clone(&manager));

        // Test list action
        let list_params = json!({"action": "list"});
        let result = control_tool.execute(&list_params).await.unwrap();
        assert_eq!(result["count"], 0);

        // Create a stream
        let _handle = manager.create_stream("test-stream".to_string()).await;

        // Test list action with active stream
        let result = control_tool.execute(&list_params).await.unwrap();
        assert_eq!(result["count"], 1);

        // Test status action
        let status_params = json!({
            "action": "status",
            "stream_id": "test-stream"
        });
        let result = control_tool.execute(&status_params).await.unwrap();
        assert_eq!(result["stream_id"], "test-stream");

        // Test close action
        let close_params = json!({
            "action": "close",
            "stream_id": "test-stream"
        });
        let result = control_tool.execute(&close_params).await.unwrap();
        assert_eq!(result["status"], "closed");
    }
}