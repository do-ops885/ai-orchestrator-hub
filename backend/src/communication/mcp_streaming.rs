use super::mcp::MCPToolHandler;
use super::mcp_unified_error::MCPUnifiedError;
use anyhow::Result;
use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock, Semaphore};
use tokio::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
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

/// ===== PHASE 4: Advanced Streaming with Backpressure =====

/// Backpressure controller for managing stream flow
#[derive(Debug)]
pub struct BackpressureController {
    high_watermark: usize,
    low_watermark: usize,
    current_buffer_size: AtomicUsize,
    is_paused: AtomicBool,
    backpressure_events: AtomicU64,
}

impl BackpressureController {
    #[must_use]
    pub fn new(high_watermark: usize, low_watermark: usize) -> Self {
        Self {
            high_watermark,
            low_watermark,
            current_buffer_size: AtomicUsize::new(0),
            is_paused: AtomicBool::new(false),
            backpressure_events: AtomicU64::new(0),
        }
    }

    /// Check if backpressure should be applied
    pub fn should_apply_backpressure(&self) -> bool {
        let current = self.current_buffer_size.load(Ordering::Relaxed);
        if current > self.high_watermark && !self.is_paused.load(Ordering::Relaxed) {
            self.is_paused.store(true, Ordering::Relaxed);
            self.backpressure_events.fetch_add(1, Ordering::Relaxed);
            warn!(
                "Backpressure applied: buffer size {} exceeds high watermark {}",
                current, self.high_watermark
            );
            true
        } else {
            false
        }
    }

    /// Check if flow can be resumed
    pub fn should_resume_flow(&self) -> bool {
        let current = self.current_buffer_size.load(Ordering::Relaxed);
        if current <= self.low_watermark && self.is_paused.load(Ordering::Relaxed) {
            self.is_paused.store(false, Ordering::Relaxed);
            info!(
                "Flow resumed: buffer size {} below low watermark {}",
                current, self.low_watermark
            );
            true
        } else {
            false
        }
    }

    /// Update buffer size and check backpressure state
    pub fn update_buffer_size(&self, new_size: usize) -> BackpressureState {
        self.current_buffer_size.store(new_size, Ordering::Relaxed);

        if self.should_apply_backpressure() {
            BackpressureState::Paused
        } else if self.should_resume_flow() {
            BackpressureState::Resumed
        } else {
            BackpressureState::Normal
        }
    }

    /// Get current buffer size
    #[must_use]
    pub fn current_buffer_size(&self) -> usize {
        self.current_buffer_size.load(Ordering::Relaxed)
    }

    /// Get backpressure statistics
    #[must_use]
    pub fn stats(&self) -> BackpressureStats {
        BackpressureStats {
            current_buffer_size: self.current_buffer_size(),
            high_watermark: self.high_watermark,
            low_watermark: self.low_watermark,
            is_paused: self.is_paused.load(Ordering::Relaxed),
            backpressure_events: self.backpressure_events.load(Ordering::Relaxed),
        }
    }
}

/// Backpressure state
#[derive(Debug, Clone, PartialEq)]
pub enum BackpressureState {
    Normal,
    Paused,
    Resumed,
}

/// Backpressure statistics
#[derive(Debug, Clone)]
pub struct BackpressureStats {
    pub current_buffer_size: usize,
    pub high_watermark: usize,
    pub low_watermark: usize,
    pub is_paused: bool,
    pub backpressure_events: u64,
}

/// Flow controller for managing concurrent streams and priorities
#[derive(Debug)]
pub struct FlowController {
    max_concurrent_streams: usize,
    active_streams: Arc<Semaphore>,
    priority_queue: Arc<RwLock<HashMap<StreamPriority, Vec<StreamRequest>>>>,
    total_requests_processed: AtomicU64,
}

impl FlowController {
    #[must_use]
    pub fn new(max_concurrent_streams: usize) -> Self {
        Self {
            max_concurrent_streams,
            active_streams: Arc::new(Semaphore::new(max_concurrent_streams)),
            priority_queue: Arc::new(RwLock::new(HashMap::new())),
            total_requests_processed: AtomicU64::new(0),
        }
    }

    /// Acquire a stream slot with priority handling
    pub async fn acquire_stream_slot(
        &self,
        priority: StreamPriority,
    ) -> Result<StreamPermit, MCPUnifiedError> {
        // Try to acquire immediately for high priority
        if matches!(priority, StreamPriority::Critical | StreamPriority::High) {
            match self.active_streams.clone().try_acquire_owned() {
                Ok(permit) => {
                    self.total_requests_processed
                        .fetch_add(1, Ordering::Relaxed);
                    return Ok(StreamPermit { _permit: permit });
                }
                Err(_) => {
                    // Queue the request
                    self.queue_request(StreamRequest {
                        priority,
                        created_at: Instant::now(),
                    })
                    .await;
                }
            }
        }

        // Wait for a slot (with timeout for lower priorities)
        let timeout_duration = match priority {
            StreamPriority::Critical => Duration::from_secs(30),
            StreamPriority::High => Duration::from_secs(15),
            StreamPriority::Normal => Duration::from_secs(5),
            StreamPriority::Low => Duration::from_secs(1),
        };

        match tokio::time::timeout(
            timeout_duration,
            self.active_streams.clone().acquire_owned(),
        )
        .await
        {
            Ok(Ok(permit)) => {
                self.total_requests_processed
                    .fetch_add(1, Ordering::Relaxed);
                Ok(StreamPermit { _permit: permit })
            }
            Ok(Err(_)) => Err(MCPUnifiedError::ResourceAccess {
                resource: "stream_slots".to_string(),
                reason: "Failed to acquire stream permit".to_string(),
                required_permissions: vec!["streaming".to_string()],
                context_chain: vec!["flow_control".to_string()],
            }),
            Err(_) => {
                // Timeout - queue the request
                self.queue_request(StreamRequest {
                    priority,
                    created_at: Instant::now(),
                })
                .await;

                Err(MCPUnifiedError::RateLimit {
                    limit: self.max_concurrent_streams as u32,
                    window: timeout_duration.as_secs().to_string(),
                    retry_after_ms: 1000,
                    client_id: None,
                    context_chain: vec!["flow_control".to_string(), "timeout".to_string()],
                })
            }
        }
    }

    /// Queue a streaming request for later processing
    async fn queue_request(&self, request: StreamRequest) {
        let priority = request.priority;
        let mut queue = self.priority_queue.write().await;
        queue.entry(priority).or_insert_with(Vec::new).push(request);
        debug!("Queued streaming request with priority {:?}", priority);
    }

    /// Process queued requests when slots become available
    pub async fn process_queue(&self) -> Result<(), MCPUnifiedError> {
        let mut queue = self.priority_queue.write().await;

        // Process in priority order
        for priority in &[
            StreamPriority::Critical,
            StreamPriority::High,
            StreamPriority::Normal,
            StreamPriority::Low,
        ] {
            if let Some(requests) = queue.get_mut(priority) {
                if !requests.is_empty() && self.active_streams.available_permits() > 0 {
                    // Sort by creation time (FIFO within priority)
                    requests.sort_by_key(|r| r.created_at);
                    let _request = requests.remove(0);

                    // Try to process this request
                    match self.active_streams.clone().try_acquire_owned() {
                        Ok(permit) => {
                            self.total_requests_processed
                                .fetch_add(1, Ordering::Relaxed);
                            info!("Processed queued request with priority {:?}", priority);
                            // In a real implementation, you'd signal the waiting request here
                            drop(permit); // Release it back since we're just demonstrating
                        }
                        Err(_) => break, // No more permits available
                    }
                }
            }
        }

        Ok(())
    }

    /// Get flow control statistics
    #[must_use]
    pub async fn stats(&self) -> FlowControlStats {
        let queue = self.priority_queue.read().await;
        let queued_requests = queue.values().map(|v| v.len()).sum();

        FlowControlStats {
            max_concurrent_streams: self.max_concurrent_streams,
            available_permits: self.active_streams.available_permits(),
            queued_requests,
            total_requests_processed: self.total_requests_processed.load(Ordering::Relaxed),
        }
    }
}

/// Stream priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StreamPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Queued stream request
#[derive(Debug, Clone)]
pub struct StreamRequest {
    pub priority: StreamPriority,
    pub created_at: Instant,
}

/// Stream permit for resource management
#[derive(Debug)]
pub struct StreamPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
}

/// Flow control statistics
#[derive(Debug, Clone)]
pub struct FlowControlStats {
    pub max_concurrent_streams: usize,
    pub available_permits: usize,
    pub queued_requests: usize,
    pub total_requests_processed: u64,
}

/// Real-time streaming metrics collector
#[derive(Debug)]
pub struct StreamingMetricsCollector {
    active_streams: AtomicUsize,
    total_streams_started: AtomicU64,
    streams_completed: AtomicU64,
    streams_failed: AtomicU64,
    average_stream_duration: AtomicU64,
    total_bytes_streamed: AtomicU64,
    peak_concurrent_streams: AtomicUsize,
    backpressure_events: AtomicU64,
    flow_control_events: AtomicU64,
}

impl StreamingMetricsCollector {
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_streams: AtomicUsize::new(0),
            total_streams_started: AtomicU64::new(0),
            streams_completed: AtomicU64::new(0),
            streams_failed: AtomicU64::new(0),
            average_stream_duration: AtomicU64::new(0),
            total_bytes_streamed: AtomicU64::new(0),
            peak_concurrent_streams: AtomicUsize::new(0),
            backpressure_events: AtomicU64::new(0),
            flow_control_events: AtomicU64::new(0),
        }
    }

    /// Record stream start
    pub fn record_stream_start(&self) {
        let active = self.active_streams.fetch_add(1, Ordering::Relaxed) + 1;
        self.total_streams_started.fetch_add(1, Ordering::Relaxed);

        // Update peak
        let mut current_peak = self.peak_concurrent_streams.load(Ordering::Relaxed);
        while active > current_peak {
            match self.peak_concurrent_streams.compare_exchange(
                current_peak,
                active,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_peak) => current_peak = new_peak,
            }
        }
    }

    /// Record stream completion
    pub fn record_stream_completion(&self, duration_ms: u64, bytes_streamed: u64) {
        self.active_streams.fetch_sub(1, Ordering::Relaxed);
        self.streams_completed.fetch_add(1, Ordering::Relaxed);
        self.total_bytes_streamed
            .fetch_add(bytes_streamed, Ordering::Relaxed);

        // Update rolling average duration
        let current_avg = self.average_stream_duration.load(Ordering::Relaxed);
        let total_completed = self.streams_completed.load(Ordering::Relaxed);
        let new_avg = ((current_avg * (total_completed - 1)) + duration_ms) / total_completed;
        self.average_stream_duration
            .store(new_avg, Ordering::Relaxed);
    }

    /// Record stream failure
    pub fn record_stream_failure(&self) {
        self.active_streams.fetch_sub(1, Ordering::Relaxed);
        self.streams_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record backpressure event
    pub fn record_backpressure_event(&self) {
        self.backpressure_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Record flow control event
    pub fn record_flow_control_event(&self) {
        self.flow_control_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current metrics
    #[must_use]
    pub fn get_metrics(&self) -> StreamingMetricsSnapshot {
        StreamingMetricsSnapshot {
            active_streams: self.active_streams.load(Ordering::Relaxed),
            total_streams_started: self.total_streams_started.load(Ordering::Relaxed),
            streams_completed: self.streams_completed.load(Ordering::Relaxed),
            streams_failed: self.streams_failed.load(Ordering::Relaxed),
            average_stream_duration_ms: self.average_stream_duration.load(Ordering::Relaxed),
            total_bytes_streamed: self.total_bytes_streamed.load(Ordering::Relaxed),
            peak_concurrent_streams: self.peak_concurrent_streams.load(Ordering::Relaxed),
            backpressure_events: self.backpressure_events.load(Ordering::Relaxed),
            flow_control_events: self.flow_control_events.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of streaming metrics
#[derive(Debug, Clone)]
pub struct StreamingMetricsSnapshot {
    pub active_streams: usize,
    pub total_streams_started: u64,
    pub streams_completed: u64,
    pub streams_failed: u64,
    pub average_stream_duration_ms: u64,
    pub total_bytes_streamed: u64,
    pub peak_concurrent_streams: usize,
    pub backpressure_events: u64,
    pub flow_control_events: u64,
}

/// Advanced streaming session with backpressure and flow control
#[derive(Debug)]
pub struct AdvancedStreamingSession {
    session_id: String,
    backpressure_controller: Arc<BackpressureController>,
    flow_controller: Arc<FlowController>,
    metrics_collector: Arc<StreamingMetricsCollector>,
    resource_limits: StreamingResourceLimits,
    start_time: Instant,
    bytes_streamed: AtomicU64,
    _permit: Option<StreamPermit>,
}

impl AdvancedStreamingSession {
    /// Create a new advanced streaming session
    pub async fn new(
        session_id: String,
        backpressure_controller: Arc<BackpressureController>,
        flow_controller: Arc<FlowController>,
        metrics_collector: Arc<StreamingMetricsCollector>,
        resource_limits: StreamingResourceLimits,
        priority: StreamPriority,
    ) -> Result<Self, MCPUnifiedError> {
        // Acquire flow control permit
        let permit = flow_controller.acquire_stream_slot(priority).await?;

        // Record metrics
        metrics_collector.record_stream_start();

        Ok(Self {
            session_id,
            backpressure_controller,
            flow_controller,
            metrics_collector,
            resource_limits,
            start_time: Instant::now(),
            bytes_streamed: AtomicU64::new(0),
            _permit: Some(permit),
        })
    }

    /// Check if streaming should be paused due to backpressure
    pub fn should_pause_streaming(&self) -> bool {
        self.backpressure_controller.should_apply_backpressure()
    }

    /// Check if streaming can resume
    pub fn can_resume_streaming(&self) -> bool {
        self.backpressure_controller.should_resume_flow()
    }

    /// Update buffer size and handle backpressure
    pub fn update_buffer_size(&self, new_size: usize) -> BackpressureState {
        let state = self.backpressure_controller.update_buffer_size(new_size);

        match state {
            BackpressureState::Paused => {
                self.metrics_collector.record_backpressure_event();
            }
            BackpressureState::Resumed => {
                // Flow resumed, can continue
            }
            BackpressureState::Normal => {
                // Normal operation
            }
        }

        state
    }

    /// Record bytes streamed
    pub fn record_bytes_streamed(&self, bytes: u64) {
        self.bytes_streamed.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Check resource limits
    pub fn check_resource_limits(&self) -> Result<(), MCPUnifiedError> {
        let bytes_streamed = self.bytes_streamed.load(Ordering::Relaxed);
        if bytes_streamed > self.resource_limits.max_bytes_per_stream {
            return Err(MCPUnifiedError::ResourceAccess {
                resource: "stream_bytes".to_string(),
                reason: format!(
                    "Stream exceeded maximum bytes limit: {} > {}",
                    bytes_streamed, self.resource_limits.max_bytes_per_stream
                ),
                required_permissions: vec!["streaming".to_string()],
                context_chain: vec!["resource_limits".to_string()],
            });
        }

        let duration = self.start_time.elapsed();
        if duration > self.resource_limits.max_stream_duration {
            return Err(MCPUnifiedError::ResourceAccess {
                resource: "stream_duration".to_string(),
                reason: format!(
                    "Stream exceeded maximum duration: {:?} > {:?}",
                    duration, self.resource_limits.max_stream_duration
                ),
                required_permissions: vec!["streaming".to_string()],
                context_chain: vec!["resource_limits".to_string()],
            });
        }

        Ok(())
    }

    /// Complete the streaming session
    pub fn complete_session(self) {
        let duration_ms = self.start_time.elapsed().as_millis() as u64;
        let bytes_streamed = self.bytes_streamed.load(Ordering::Relaxed);

        self.metrics_collector
            .record_stream_completion(duration_ms, bytes_streamed);

        // Permit is automatically dropped when session is dropped
        drop(self);
    }

    /// Fail the streaming session
    pub fn fail_session(self) {
        self.metrics_collector.record_stream_failure();
        // Permit is automatically dropped when session is dropped
        drop(self);
    }

    /// Get session statistics
    #[must_use]
    pub fn stats(&self) -> StreamingSessionStats {
        StreamingSessionStats {
            session_id: self.session_id.clone(),
            duration: self.start_time.elapsed(),
            bytes_streamed: self.bytes_streamed.load(Ordering::Relaxed),
            backpressure_stats: self.backpressure_controller.stats(),
            flow_control_stats: futures::executor::block_on(self.flow_controller.stats()),
        }
    }
}

/// Resource limits for streaming sessions
#[derive(Debug, Clone)]
pub struct StreamingResourceLimits {
    pub max_bytes_per_stream: u64,
    pub max_stream_duration: Duration,
    pub buffer_high_watermark: usize,
    pub buffer_low_watermark: usize,
}

impl Default for StreamingResourceLimits {
    fn default() -> Self {
        Self {
            max_bytes_per_stream: 100 * 1024 * 1024,        // 100MB
            max_stream_duration: Duration::from_secs(3600), // 1 hour
            buffer_high_watermark: 1000,                    // 1000 items
            buffer_low_watermark: 100,                      // 100 items
        }
    }
}

/// Streaming session statistics
#[derive(Debug, Clone)]
pub struct StreamingSessionStats {
    pub session_id: String,
    pub duration: Duration,
    pub bytes_streamed: u64,
    pub backpressure_stats: BackpressureStats,
    pub flow_control_stats: FlowControlStats,
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
            "batch_create",
            "analyze_large",
            "process_bulk",
            "generate_report",
            "workflow",
            "analytics",
            "migration",
            "backup",
        ];

        let description = self.inner.get_description().to_lowercase();
        operation_hints
            .iter()
            .any(|hint| description.contains(hint))
    }
}

#[async_trait]
impl<T: MCPToolHandler + Clone + 'static> MCPToolHandler for StreamingMCPToolHandler<T> {
    async fn execute(&self, params: &Value) -> Result<Value> {
        if !self.should_stream(params) {
            // Execute normally if streaming is not needed
            return self.inner.execute(params).await;
        }

        // Create streaming operation with advanced features
        let stream_id = Uuid::new_v4().to_string();
        let start_time = Instant::now();

        info!("Starting advanced streaming operation: {}", stream_id);

        // Determine stream priority from parameters
        let priority = self.determine_stream_priority(params);

        // Create advanced stream manager (in production, this would be injected)
        let resource_limits = StreamingResourceLimits::default();
        let advanced_manager = Arc::new(AdvancedStreamManager::new(resource_limits));

        // Start background tasks
        let _background_handles = advanced_manager.clone().start_background_tasks();

        // Create advanced streaming session
        let session = match advanced_manager
            .create_advanced_stream(stream_id.clone(), priority)
            .await
        {
            Ok(session) => session,
            Err(e) => {
                error!("Failed to create advanced streaming session: {}", e);
                // Fall back to basic streaming
                let basic_manager = StreamManager::new();
                let stream_handle = basic_manager.create_stream(stream_id.clone()).await;
                return self
                    .execute_basic_streaming(params, stream_id, stream_handle)
                    .await;
            }
        };

        // Create basic stream handle for compatibility
        let stream_handle = advanced_manager.create_stream(stream_id.clone()).await;

        // Spawn background task for advanced streaming execution
        let inner_handler = self.inner.clone();
        let params_clone = params.clone();
        let stream_id_clone = stream_id.clone();
        let max_duration = self.max_stream_duration;
        let session_clone = session;

        tokio::spawn(async move {
            Self::execute_advanced_streaming(
                inner_handler,
                params_clone,
                stream_id_clone,
                max_duration,
                stream_handle,
                session_clone,
            )
            .await;
        });

        Ok(json!({
            "stream_id": stream_id,
            "status": "started",
            "streaming": true,
            "advanced_features": true,
            "started_at": chrono::Utc::now().to_rfc3339()
        }))
    }

    fn get_schema(&self) -> Value {
        let mut schema = self.inner.get_schema();

        // Add streaming parameters to schema
        if let Some(properties) = schema.get_mut("properties") {
            if let Some(props_obj) = properties.as_object_mut() {
                props_obj.insert(
                    "stream".to_string(),
                    json!({
                        "type": "boolean",
                        "description": "Enable streaming mode for long-running operations",
                        "default": false
                    }),
                );
                props_obj.insert(
                    "priority".to_string(),
                    json!({
                        "type": "string",
                        "enum": ["low", "normal", "high", "critical"],
                        "description": "Streaming priority level",
                        "default": "normal"
                    }),
                );
            }
        }

        schema
    }

    fn get_description(&self) -> String {
        format!(
            "{} (advanced streaming-enabled)",
            self.inner.get_description()
        )
    }
}

impl<T: MCPToolHandler + Clone + 'static> StreamingMCPToolHandler<T> {
    /// Determine stream priority from parameters
    fn determine_stream_priority(&self, params: &Value) -> StreamPriority {
        // Check for explicit priority parameter
        if let Some(priority_str) = params.get("priority").and_then(|v| v.as_str()) {
            match priority_str.to_lowercase().as_str() {
                "critical" => return StreamPriority::Critical,
                "high" => return StreamPriority::High,
                "low" => return StreamPriority::Low,
                _ => {} // Fall through to Normal
            }
        }

        // Determine based on operation type
        let operation_desc = self.inner.get_description().to_lowercase();
        if operation_desc.contains("emergency") || operation_desc.contains("critical") {
            StreamPriority::Critical
        } else if operation_desc.contains("urgent") || operation_desc.contains("high") {
            StreamPriority::High
        } else if operation_desc.contains("background") || operation_desc.contains("low") {
            StreamPriority::Low
        } else {
            StreamPriority::Normal
        }
    }

    /// Execute advanced streaming with backpressure handling
    async fn execute_advanced_streaming(
        inner_handler: T,
        params: Value,
        stream_id: String,
        max_duration: Duration,
        stream_handle: StreamHandle,
        session: AdvancedStreamingSession,
    ) {
        let mut progress = StreamingProgress {
            current: 0,
            total: None,
            percentage: Some(0.0),
            stage: "Starting".to_string(),
            estimated_completion: None,
        };

        let _buffer_size = 0usize;

        // Send initial progress
        let initial_response = MCPStreamingResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            stream: true,
            stream_id: stream_id.clone(),
            data: Some(json!({"status": "started", "advanced_features": true})),
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
            session.fail_session();
            return;
        }

        // Execute the actual operation with backpressure-aware streaming
        let execution_result =
            tokio::time::timeout(max_duration, inner_handler.execute(&params)).await;

        match execution_result {
            Ok(Ok(result)) => {
                // Check resource limits before sending final result
                if let Err(e) = session.check_resource_limits() {
                    warn!("Resource limit exceeded during streaming: {}", e);
                    let error_response = MCPStreamingResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        stream: true,
                        stream_id: stream_id.clone(),
                        data: Some(json!({"error": format!("Resource limit exceeded: {}", e)})),
                        done: true,
                        progress: None,
                        metadata: None,
                    };

                    if let Err(send_err) = stream_handle.send_error(error_response).await {
                        warn!("Failed to send resource limit error: {}", send_err);
                    }
                    session.fail_session();
                    return;
                }

                // Send final result
                progress.current = 100;
                progress.percentage = Some(100.0);
                progress.stage = "Completed".to_string();

                // Record bytes streamed (estimate based on result size)
                let result_size = serde_json::to_string(&result)
                    .map(|s| s.len() as u64)
                    .unwrap_or(0);
                session.record_bytes_streamed(result_size);

                let final_response = MCPStreamingResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    stream: true,
                    stream_id,
                    data: Some(result),
                    done: true,
                    progress: Some(progress),
                    metadata: Some(StreamingMetadata {
                        started_at: chrono::Utc::now().to_rfc3339(),
                        last_update: chrono::Utc::now().to_rfc3339(),
                        operation_type: inner_handler.get_description(),
                        total_chunks: Some(1),
                        chunk_size: Some(result_size as u64),
                    }),
                };

                if let Err(e) = stream_handle.send_final(final_response).await {
                    warn!("Failed to send final streaming result: {}", e);
                    session.fail_session();
                } else {
                    session.complete_session();
                }
            }
            Ok(Err(e)) => {
                // Send error
                let error_response = MCPStreamingResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    stream: true,
                    stream_id,
                    data: Some(json!({"error": e.to_string()})),
                    done: true,
                    progress: None,
                    metadata: None,
                };

                if let Err(send_err) = stream_handle.send_error(error_response).await {
                    warn!("Failed to send streaming error: {}", send_err);
                }
                session.fail_session();
            }
            Err(_) => {
                // Timeout
                let timeout_response = MCPStreamingResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    stream: true,
                    stream_id,
                    data: Some(json!({"error": "Operation timed out"})),
                    done: true,
                    progress: None,
                    metadata: None,
                };

                if let Err(e) = stream_handle.send_error(timeout_response).await {
                    warn!("Failed to send streaming timeout: {}", e);
                }
                session.fail_session();
            }
        }
    }

    /// Fallback to basic streaming if advanced features fail
    async fn execute_basic_streaming(
        &self,
        params: &Value,
        stream_id: String,
        stream_handle: StreamHandle,
    ) -> Result<Value> {
        warn!("Falling back to basic streaming for {}", stream_id);

        // Basic streaming implementation (original logic)
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

            // Execute the actual operation
            match tokio::time::timeout(max_duration, inner_handler.execute(&params_clone)).await {
                Ok(Ok(result)) => {
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
            "fallback": true,
            "started_at": chrono::Utc::now().to_rfc3339()
        }))
    }
}

/// Advanced stream manager with backpressure and flow control
pub struct AdvancedStreamManager {
    base_manager: StreamManager,
    backpressure_controller: Arc<BackpressureController>,
    flow_controller: Arc<FlowController>,
    metrics_collector: Arc<StreamingMetricsCollector>,
    resource_limits: StreamingResourceLimits,
    cleanup_task: Option<tokio::task::JoinHandle<()>>,
}

impl AdvancedStreamManager {
    /// Create a new advanced stream manager
    #[must_use]
    pub fn new(resource_limits: StreamingResourceLimits) -> Self {
        let backpressure_controller = Arc::new(BackpressureController::new(
            resource_limits.buffer_high_watermark,
            resource_limits.buffer_low_watermark,
        ));

        let flow_controller = Arc::new(FlowController::new(50)); // Default max concurrent streams

        let metrics_collector = Arc::new(StreamingMetricsCollector::new());

        Self {
            base_manager: StreamManager::new(),
            backpressure_controller,
            flow_controller,
            metrics_collector,
            resource_limits,
            cleanup_task: None,
        }
    }

    /// Create a new advanced streaming session
    pub async fn create_advanced_stream(
        &self,
        stream_id: String,
        priority: StreamPriority,
    ) -> Result<AdvancedStreamingSession, MCPUnifiedError> {
        AdvancedStreamingSession::new(
            stream_id,
            Arc::clone(&self.backpressure_controller),
            Arc::clone(&self.flow_controller),
            Arc::clone(&self.metrics_collector),
            self.resource_limits.clone(),
            priority,
        )
        .await
    }

    /// Start background monitoring and cleanup tasks
    pub fn start_background_tasks(self: Arc<Self>) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = Vec::new();

        // Flow control queue processor
        let flow_processor = {
            let manager = Arc::clone(&self);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(100));
                loop {
                    interval.tick().await;
                    if let Err(e) = manager.flow_controller.process_queue().await {
                        error!("Error processing flow control queue: {}", e);
                    }
                }
            })
        };
        handles.push(flow_processor);

        // Metrics reporting task
        let metrics_reporter = {
            let manager = Arc::clone(&self);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    let metrics = manager.metrics_collector.get_metrics();
                    info!("Streaming metrics: active={}, completed={}, failed={}, backpressure_events={}",
                         metrics.active_streams, metrics.streams_completed, metrics.streams_failed, metrics.backpressure_events);
                }
            })
        };
        handles.push(metrics_reporter);

        // Backpressure monitoring task
        let backpressure_monitor = {
            let manager = Arc::clone(&self);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    let stats = manager.backpressure_controller.stats();
                    if stats.is_paused {
                        warn!(
                            "Backpressure active: buffer_size={}, high_watermark={}",
                            stats.current_buffer_size, stats.high_watermark
                        );
                    }
                }
            })
        };
        handles.push(backpressure_monitor);

        info!(
            "Started {} advanced streaming background tasks",
            handles.len()
        );
        handles
    }

    /// Get comprehensive streaming statistics
    #[must_use]
    pub async fn get_comprehensive_stats(&self) -> ComprehensiveStreamingStats {
        let active_streams = self.base_manager.list_active_streams().await.len();
        let backpressure_stats = self.backpressure_controller.stats();
        let flow_control_stats = self.flow_controller.stats().await;
        let metrics = self.metrics_collector.get_metrics();

        ComprehensiveStreamingStats {
            active_streams,
            backpressure_stats,
            flow_control_stats,
            metrics,
            resource_limits: self.resource_limits.clone(),
        }
    }

    // Delegate methods to base manager
    pub async fn create_stream(&self, stream_id: String) -> StreamHandle {
        self.base_manager.create_stream(stream_id).await
    }

    pub async fn get_stream(&self, stream_id: &str) -> Option<StreamHandle> {
        self.base_manager.get_stream(stream_id).await
    }

    pub async fn close_stream(&self, stream_id: &str) {
        self.base_manager.close_stream(stream_id).await
    }

    pub async fn list_active_streams(&self) -> Vec<String> {
        self.base_manager.list_active_streams().await
    }
}

/// Comprehensive streaming statistics
#[derive(Debug, Clone)]
pub struct ComprehensiveStreamingStats {
    pub active_streams: usize,
    pub backpressure_stats: BackpressureStats,
    pub flow_control_stats: FlowControlStats,
    pub metrics: StreamingMetricsSnapshot,
    pub resource_limits: StreamingResourceLimits,
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

        self.active_streams
            .write()
            .await
            .insert(stream_id, handle.clone());
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

        self.sender
            .send(response)
            .map_err(|e| MCPUnifiedError::Internal {
                message: format!("Failed to send stream update: {e}"),
                source_error: Some(e.to_string()),
                recovery_suggestion: Some("Check if stream receiver is still active".to_string()),
                context_chain: Vec::new(),
            })?;

        Ok(())
    }

    pub async fn send_final(&self, response: MCPStreamingResponse) -> Result<(), MCPUnifiedError> {
        *self.status.write().await = StreamStatus::Completed;

        self.sender
            .send(response)
            .map_err(|e| MCPUnifiedError::Internal {
                message: format!("Failed to send final stream result: {e}"),
                source_error: Some(e.to_string()),
                recovery_suggestion: Some("Check if stream receiver is still active".to_string()),
                context_chain: Vec::new(),
            })?;

        Ok(())
    }

    pub async fn send_error(&self, response: MCPStreamingResponse) -> Result<(), MCPUnifiedError> {
        *self.status.write().await = StreamStatus::Failed("Operation failed".to_string());

        self.sender
            .send(response)
            .map_err(|e| MCPUnifiedError::Internal {
                message: format!("Failed to send stream error: {e}"),
                source_error: Some(e.to_string()),
                recovery_suggestion: Some("Check if stream receiver is still active".to_string()),
                context_chain: Vec::new(),
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
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing required parameter: stream_id for status action")
                    })?;

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
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing required parameter: stream_id for close action")
                    })?;

                self.stream_manager.close_stream(stream_id).await;
                Ok(json!({
                    "stream_id": stream_id,
                    "status": "closed"
                }))
            }
            _ => Err(anyhow::anyhow!(
                "Invalid action: {action}. Supported actions: list, status, close"
            )),
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
        "Manage streaming operations: list active streams, check status, or close streams"
            .to_string()
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
        let streaming_tool = StreamingMCPToolHandler::new(tool, 1000, Duration::from_secs(30));

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
        let result = control_tool
            .execute(&list_params)
            .await
            .expect("replaced unwrap");
        assert_eq!(result["count"], 0);

        // Create a stream
        let _handle = manager.create_stream("test-stream".to_string()).await;

        // Test list action with active stream
        let result = control_tool
            .execute(&list_params)
            .await
            .expect("replaced unwrap");
        assert_eq!(result["count"], 1);

        // Test status action
        let status_params = json!({
            "action": "status",
            "stream_id": "test-stream"
        });
        let result = control_tool
            .execute(&status_params)
            .await
            .expect("replaced unwrap");
        assert_eq!(result["stream_id"], "test-stream");

        // Test close action
        let close_params = json!({
            "action": "close",
            "stream_id": "test-stream"
        });
        let result = control_tool
            .execute(&close_params)
            .await
            .expect("replaced unwrap");
        assert_eq!(result["status"], "closed");
    }

    #[tokio::test]
    async fn test_backpressure_controller() {
        let controller = BackpressureController::new(100, 20);

        // Initially normal
        assert!(!controller.should_apply_backpressure());

        // Update to high watermark
        let state = controller.update_buffer_size(100);
        assert_eq!(state, BackpressureState::Paused);
        assert!(controller.should_apply_backpressure());

        // Update below low watermark
        let state = controller.update_buffer_size(10);
        assert_eq!(state, BackpressureState::Resumed);
        assert!(!controller.should_apply_backpressure());

        // Check stats
        let stats = controller.stats();
        assert_eq!(stats.high_watermark, 100);
        assert_eq!(stats.low_watermark, 20);
        assert_eq!(stats.backpressure_events, 1);
    }

    #[tokio::test]
    async fn test_flow_controller() {
        let controller = FlowController::new(2);

        // Acquire first slot
        let _permit1 = controller
            .acquire_stream_slot(StreamPriority::Normal)
            .await
            .expect("replaced unwrap");
        assert_eq!(controller.stats().await.available_permits, 1);

        // Acquire second slot
        let _permit2 = controller
            .acquire_stream_slot(StreamPriority::Normal)
            .await
            .expect("replaced unwrap");
        assert_eq!(controller.stats().await.available_permits, 0);

        // Try to acquire third slot (should fail)
        let result = controller.acquire_stream_slot(StreamPriority::Normal).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_streaming_metrics_collector() {
        let collector = StreamingMetricsCollector::new();

        // Record stream start
        collector.record_stream_start();
        let metrics = collector.get_metrics();
        assert_eq!(metrics.active_streams, 1);
        assert_eq!(metrics.total_streams_started, 1);

        // Record completion
        collector.record_stream_completion(1000, 1024);
        let metrics = collector.get_metrics();
        assert_eq!(metrics.active_streams, 0);
        assert_eq!(metrics.streams_completed, 1);
        assert_eq!(metrics.total_bytes_streamed, 1024);

        // Record failure
        collector.record_stream_start();
        collector.record_stream_failure();
        let metrics = collector.get_metrics();
        assert_eq!(metrics.streams_failed, 1);
    }

    #[tokio::test]
    async fn test_advanced_stream_manager() {
        let resource_limits = StreamingResourceLimits::default();
        let manager = Arc::new(AdvancedStreamManager::new(resource_limits));

        // Create advanced stream
        let session = manager
            .create_advanced_stream("test-stream".to_string(), StreamPriority::Normal)
            .await
            .expect("replaced unwrap");

        // Check initial state
        assert!(!session.should_pause_streaming());

        // Update buffer size to trigger backpressure
        let state = session.update_buffer_size(1000); // Above default high watermark
        assert_eq!(state, BackpressureState::Paused);

        // Check if streaming should pause
        assert!(session.should_pause_streaming());

        // Update buffer size below low watermark
        let state = session.update_buffer_size(50);
        assert_eq!(state, BackpressureState::Resumed);

        // Complete session
        session.complete_session();

        // Check comprehensive stats
        let stats = manager.get_comprehensive_stats().await;
        assert_eq!(stats.metrics.streams_completed, 1);
    }

    #[tokio::test]
    async fn test_stream_priority_determination() {
        let tool = MockSlowTool;
        let streaming_tool = StreamingMCPToolHandler::new(tool, 1000, Duration::from_secs(30));

        // Test explicit priority
        let params = json!({"priority": "critical"});
        assert_eq!(
            streaming_tool.determine_stream_priority(&params),
            StreamPriority::Critical
        );

        // Test operation-based priority
        let params = json!({});
        // MockSlowTool has description "batch_create_test_operation" so should be Normal
        assert_eq!(
            streaming_tool.determine_stream_priority(&params),
            StreamPriority::Normal
        );
    }

    #[tokio::test]
    async fn test_resource_limits_checking() {
        let resource_limits = StreamingResourceLimits {
            max_bytes_per_stream: 100,
            max_stream_duration: Duration::from_millis(100),
            ..Default::default()
        };

        let manager = Arc::new(AdvancedStreamManager::new(resource_limits));
        let session = manager
            .create_advanced_stream("test-stream".to_string(), StreamPriority::Normal)
            .await
            .expect("replaced unwrap");

        // Record bytes within limit
        session.record_bytes_streamed(50);
        assert!(session.check_resource_limits().is_ok());

        // Record bytes exceeding limit
        session.record_bytes_streamed(60); // Total 110 > 100
        assert!(session.check_resource_limits().is_err());

        session.fail_session();
    }
}

// ===== PHASE 2: Memory Management and Resource Limits =====

/// Memory usage statistics for streaming operations
#[derive(Debug, Clone)]
pub struct StreamingMemoryStats {
    pub active_streams: usize,
    pub total_memory_used_bytes: usize,
    pub average_memory_per_stream: usize,
    pub peak_memory_usage: usize,
    pub streams_cleaned_up: u64,
    pub memory_reclaimed_bytes: u64,
    pub last_cleanup: Option<Instant>,
}

/// Resource limits configuration
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_concurrent_streams: usize,
    pub max_memory_per_stream_bytes: usize,
    pub max_total_memory_bytes: usize,
    pub stream_timeout: Duration,
    pub cleanup_interval: Duration,
    pub memory_check_interval: Duration,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 50,
            max_memory_per_stream_bytes: 10 * 1024 * 1024, // 10MB per stream
            max_total_memory_bytes: 500 * 1024 * 1024,     // 500MB total
            stream_timeout: Duration::from_secs(3600),     // 1 hour
            cleanup_interval: Duration::from_secs(300),    // 5 minutes
            memory_check_interval: Duration::from_secs(60), // 1 minute
        }
    }
}

/// Enhanced stream manager with memory management and resource limits
pub struct MemoryManagedStreamManager {
    base_manager: StreamManager,
    memory_stats: Arc<RwLock<StreamingMemoryStats>>,
    resource_limits: ResourceLimits,
    semaphore: Arc<Semaphore>, // Limits concurrent streams
    stream_memory_usage: Arc<RwLock<HashMap<String, usize>>>, // stream_id -> memory usage
    stream_creation_times: Arc<RwLock<HashMap<String, Instant>>>, // stream_id -> creation time
}

impl MemoryManagedStreamManager {
    /// Create a new memory-managed stream manager
    pub fn new(resource_limits: ResourceLimits) -> Self {
        Self {
            base_manager: StreamManager::new(),
            memory_stats: Arc::new(RwLock::new(StreamingMemoryStats {
                active_streams: 0,
                total_memory_used_bytes: 0,
                average_memory_per_stream: 0,
                peak_memory_usage: 0,
                streams_cleaned_up: 0,
                memory_reclaimed_bytes: 0,
                last_cleanup: None,
            })),
            resource_limits: resource_limits.clone(),
            semaphore: Arc::new(Semaphore::new(resource_limits.max_concurrent_streams)),
            stream_memory_usage: Arc::new(RwLock::new(HashMap::new())),
            stream_creation_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new stream with memory limits
    pub async fn create_stream(&self, stream_id: String) -> Result<StreamHandle, MCPUnifiedError> {
        // Check concurrent stream limit
        let permit =
            self.semaphore
                .clone()
                .try_acquire_owned()
                .map_err(|_| MCPUnifiedError::RateLimit {
                    limit: self.resource_limits.max_concurrent_streams as u32,
                    window: "1 second".to_string(),
                    retry_after_ms: 1000,
                    client_id: None,
                    context_chain: vec!["streaming".to_string(), "resource_limits".to_string()],
                })?;

        // Check memory limits
        let current_memory = {
            let stats = self.memory_stats.read().await;
            stats.total_memory_used_bytes
        };

        if current_memory >= self.resource_limits.max_total_memory_bytes {
            return Err(MCPUnifiedError::ResourceAccess {
                resource: "memory".to_string(),
                reason: "Total memory limit exceeded".to_string(),
                required_permissions: vec!["memory_management".to_string()],
                context_chain: vec!["streaming".to_string(), "memory_limits".to_string()],
            });
        }

        // Create the stream
        let handle = self.base_manager.create_stream(stream_id.clone()).await;

        // Track memory usage and creation time
        {
            let mut memory_usage = self.stream_memory_usage.write().await;
            let mut creation_times = self.stream_creation_times.write().await;

            memory_usage.insert(stream_id.clone(), 1024); // Initial 1KB estimate
            creation_times.insert(stream_id.clone(), Instant::now());

            let mut stats = self.memory_stats.write().await;
            stats.active_streams += 1;
            stats.total_memory_used_bytes += 1024;
            stats.average_memory_per_stream = stats.total_memory_used_bytes / stats.active_streams;
            stats.peak_memory_usage = stats.peak_memory_usage.max(stats.total_memory_used_bytes);
        }

        // Forget the permit so it lives as long as the stream
        std::mem::forget(permit);

        Ok(handle)
    }

    /// Update memory usage for a stream
    pub async fn update_stream_memory(
        &self,
        stream_id: &str,
        memory_bytes: usize,
    ) -> Result<(), MCPUnifiedError> {
        let mut memory_usage = self.stream_memory_usage.write().await;

        if let Some(current_usage) = memory_usage.get_mut(stream_id) {
            let old_usage = *current_usage;
            *current_usage = memory_bytes;

            let mut stats = self.memory_stats.write().await;
            stats.total_memory_used_bytes =
                stats.total_memory_used_bytes - old_usage + memory_bytes;

            // Check per-stream limit
            if memory_bytes > self.resource_limits.max_memory_per_stream_bytes {
                return Err(MCPUnifiedError::ResourceAccess {
                    resource: "stream_memory".to_string(),
                    reason: format!("Stream memory limit exceeded: {} bytes", memory_bytes),
                    required_permissions: vec!["memory_management".to_string()],
                    context_chain: vec!["streaming".to_string(), "per_stream_limits".to_string()],
                });
            }

            // Check total memory limit
            if stats.total_memory_used_bytes > self.resource_limits.max_total_memory_bytes {
                return Err(MCPUnifiedError::ResourceAccess {
                    resource: "total_memory".to_string(),
                    reason: "Total memory limit exceeded".to_string(),
                    required_permissions: vec!["memory_management".to_string()],
                    context_chain: vec!["streaming".to_string(), "total_memory_limits".to_string()],
                });
            }

            stats.average_memory_per_stream = if stats.active_streams > 0 {
                stats.total_memory_used_bytes / stats.active_streams
            } else {
                0
            };
            stats.peak_memory_usage = stats.peak_memory_usage.max(stats.total_memory_used_bytes);
        }

        Ok(())
    }

    /// Close a stream and clean up resources
    pub async fn close_stream(&self, stream_id: &str) {
        // Get memory usage before closing
        let memory_freed = {
            let mut memory_usage = self.stream_memory_usage.write().await;
            memory_usage.remove(stream_id).unwrap_or(0)
        };

        // Remove creation time
        {
            let mut creation_times = self.stream_creation_times.write().await;
            creation_times.remove(stream_id);
        }

        // Close the stream
        self.base_manager.close_stream(stream_id).await;

        // Update stats
        {
            let mut stats = self.memory_stats.write().await;
            stats.active_streams = stats.active_streams.saturating_sub(1);
            stats.total_memory_used_bytes =
                stats.total_memory_used_bytes.saturating_sub(memory_freed);
            stats.streams_cleaned_up += 1;
            stats.memory_reclaimed_bytes += memory_freed as u64;
        }

        // Release semaphore permit (this is a simplified approach)
        // In a real implementation, you'd need to track permits per stream
    }

    /// Perform automatic cleanup of expired streams
    pub async fn cleanup_expired_streams(&self) {
        let expired_streams: Vec<String> = {
            let creation_times = self.stream_creation_times.read().await;
            creation_times
                .iter()
                .filter(|(_, created_at)| {
                    created_at.elapsed() > self.resource_limits.stream_timeout
                })
                .map(|(stream_id, _)| stream_id.clone())
                .collect()
        };

        let expired_count = expired_streams.len();

        for stream_id in expired_streams {
            warn!("Cleaning up expired stream: {}", stream_id);
            self.close_stream(&stream_id).await;
        }

        if expired_count > 0 {
            let mut stats = self.memory_stats.write().await;
            stats.last_cleanup = Some(Instant::now());
        }
    }

    /// Get memory statistics
    pub async fn get_memory_stats(&self) -> StreamingMemoryStats {
        self.memory_stats.read().await.clone()
    }

    /// Check if system is approaching memory limits
    pub async fn check_memory_pressure(&self) -> MemoryPressureLevel {
        let stats = self.memory_stats.read().await;
        let usage_percentage = if self.resource_limits.max_total_memory_bytes > 0 {
            (stats.total_memory_used_bytes as f64
                / self.resource_limits.max_total_memory_bytes as f64)
                * 100.0
        } else {
            0.0
        };

        match usage_percentage {
            p if p >= 90.0 => MemoryPressureLevel::Critical,
            p if p >= 75.0 => MemoryPressureLevel::High,
            p if p >= 50.0 => MemoryPressureLevel::Medium,
            _ => MemoryPressureLevel::Low,
        }
    }

    /// Start background memory management tasks
    pub fn start_memory_management_tasks(self: Arc<Self>) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = Vec::new();

        // Periodic cleanup task
        let cleanup_handle = {
            let manager = Arc::clone(&self);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(manager.resource_limits.cleanup_interval);
                loop {
                    interval.tick().await;
                    manager.cleanup_expired_streams().await;
                }
            })
        };
        handles.push(cleanup_handle);

        // Memory monitoring task
        let monitoring_handle = {
            let manager = Arc::clone(&self);
            tokio::spawn(async move {
                let mut interval =
                    tokio::time::interval(manager.resource_limits.memory_check_interval);
                loop {
                    interval.tick().await;
                    let pressure = manager.check_memory_pressure().await;
                    match pressure {
                        MemoryPressureLevel::Critical => {
                            warn!(
                                "Critical memory pressure detected - {} streams active, {} MB used",
                                manager.memory_stats.read().await.active_streams,
                                manager.memory_stats.read().await.total_memory_used_bytes
                                    / (1024 * 1024)
                            );
                        }
                        MemoryPressureLevel::High => {
                            info!("High memory pressure detected");
                        }
                        _ => {}
                    }
                }
            })
        };
        handles.push(monitoring_handle);

        info!(
            "Started {} background memory management tasks",
            handles.len()
        );
        handles
    }

    // Delegate other methods to base manager
    pub async fn get_stream(&self, stream_id: &str) -> Option<StreamHandle> {
        self.base_manager.get_stream(stream_id).await
    }

    pub async fn list_active_streams(&self) -> Vec<String> {
        self.base_manager.list_active_streams().await
    }
}

/// Memory pressure levels for monitoring
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryPressureLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for MemoryManagedStreamManager {
    fn default() -> Self {
        Self::new(ResourceLimits::default())
    }
}
