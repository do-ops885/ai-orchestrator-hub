use super::mcp::MCPToolHandler;
use super::mcp_tool_registry::MCPToolRegistry;
use super::mcp_unified_error::MCPUnifiedError;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// MCP Batch Processing System (Phase 3.2)
/// 
/// Provides efficient batch execution of multiple MCP operations,
/// with support for parallel processing, dependency management,
/// and comprehensive error handling.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    pub id: String,
    pub tool_name: String,
    pub params: Value,
    pub priority: Option<u8>, // 1-10, higher is more priority
    pub depends_on: Option<Vec<String>>, // IDs of requests this depends on
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    pub request_id: String,
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub dependencies_resolved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecutionSummary {
    pub batch_id: String,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub skipped_requests: usize, // Due to dependency failures
    pub total_execution_time_ms: u64,
    pub parallel_efficiency: f64, // Actual time vs sequential time
    pub started_at: String,
    pub completed_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatchRequestStatus {
    Pending,
    WaitingForDependencies,
    Running,
    Completed,
    Failed(String),
    Skipped(String),
}

/// Batch execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    pub max_concurrent: usize,
    pub default_timeout: Duration,
    pub fail_fast: bool, // Stop on first error
    pub dependency_timeout: Duration,
    pub enable_retry: bool,
    pub retry_attempts: u32,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            default_timeout: Duration::from_secs(30),
            fail_fast: false,
            dependency_timeout: Duration::from_secs(300), // 5 minutes
            enable_retry: true,
            retry_attempts: 3,
        }
    }
}

/// Batch execution context
pub struct BatchExecutionContext {
    pub batch_id: String,
    pub requests: Vec<BatchRequest>,
    pub responses: HashMap<String, BatchResponse>,
    pub status_map: HashMap<String, BatchRequestStatus>,
    pub dependency_graph: HashMap<String, Vec<String>>,
    pub config: BatchConfig,
    pub started_at: Instant,
}

impl BatchExecutionContext {
    #[must_use] 
    pub fn new(batch_id: String, requests: Vec<BatchRequest>, config: BatchConfig) -> Self {
        let mut dependency_graph = HashMap::new();
        let mut status_map = HashMap::new();

        // Build dependency graph and initialize status
        for request in &requests {
            dependency_graph.insert(request.id.clone(), request.depends_on.clone().unwrap_or_default());
            status_map.insert(request.id.clone(), BatchRequestStatus::Pending);
        }

        Self {
            batch_id,
            requests,
            responses: HashMap::new(),
            status_map,
            dependency_graph,
            config,
            started_at: Instant::now(),
        }
    }

    /// Get requests ready for execution (dependencies satisfied)
    #[must_use] 
    pub fn get_ready_requests(&self) -> Vec<&BatchRequest> {
        self.requests
            .iter()
            .filter(|req| {
                if self.status_map.get(&req.id) != Some(&BatchRequestStatus::Pending) {
                    return false;
                }

                // Check if all dependencies are completed
                if let Some(deps) = &req.depends_on {
                    deps.iter().all(|dep_id| {
                        matches!(
                            self.status_map.get(dep_id),
                            Some(BatchRequestStatus::Completed)
                        )
                    })
                } else {
                    true
                }
            })
            .collect()
    }

    /// Mark request as failed and cascade to dependents
    pub fn mark_failed_cascade(&mut self, request_id: &str, error: String) {
        self.status_map.insert(request_id.to_string(), BatchRequestStatus::Failed(error.clone()));

        // Find all requests that depend on this one and mark them as skipped
        let dependents: Vec<String> = self.dependency_graph
            .iter()
            .filter(|(_, deps)| deps.contains(&request_id.to_string()))
            .map(|(id, _)| id.clone())
            .collect();

        for dependent_id in dependents {
            if self.status_map.get(&dependent_id) == Some(&BatchRequestStatus::Pending) {
                self.status_map.insert(
                    dependent_id.clone(),
                    BatchRequestStatus::Skipped(format!("Dependency '{request_id}' failed: {error}"))
                );
                // Recursively skip dependents
                self.mark_failed_cascade(&dependent_id, "Transitive dependency failure".to_string());
            }
        }
    }

    /// Check for circular dependencies
    pub fn validate_dependencies(&self) -> Result<(), MCPUnifiedError> {
        let mut visited = HashMap::new();
        let mut rec_stack = HashMap::new();

        for request_id in self.dependency_graph.keys() {
            if !visited.contains_key(request_id) {
                self.dfs_check_cycle(request_id, &mut visited, &mut rec_stack)?;
            }
        }

        Ok(())
    }

    fn dfs_check_cycle(
        &self,
        request_id: &str,
        visited: &mut HashMap<String, bool>,
        rec_stack: &mut HashMap<String, bool>,
    ) -> Result<(), MCPUnifiedError> {
        visited.insert(request_id.to_string(), true);
        rec_stack.insert(request_id.to_string(), true);

        if let Some(dependencies) = self.dependency_graph.get(request_id) {
            for dep_id in dependencies {
                if !visited.contains_key(dep_id) {
                    self.dfs_check_cycle(dep_id, visited, rec_stack)?;
                } else if rec_stack.contains_key(dep_id) {
                    return Err(MCPUnifiedError::validation(
                        "dependencies".to_string(),
                        format!("Circular dependency detected: {request_id} -> {dep_id}"),
                        Some(json!({
                            "request_id": request_id,
                            "dependency_id": dep_id,
                            "cycle_path": format!("{} -> {}", request_id, dep_id)
                        })),
                        Some("Dependencies must form a DAG (no cycles)".to_string()),
                    ));
                }
            }
        }

        rec_stack.remove(request_id);
        Ok(())
    }
}

/// Batch processing tool handler
pub struct BatchMCPToolHandler {
    tool_registry: Arc<MCPToolRegistry>,
    active_batches: Arc<RwLock<HashMap<String, BatchExecutionContext>>>,
}

impl BatchMCPToolHandler {
    #[must_use] 
    pub fn new(tool_registry: Arc<MCPToolRegistry>) -> Self {
        Self {
            tool_registry,
            active_batches: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Execute a batch of requests
    pub async fn execute_batch(
        &self,
        requests: Vec<BatchRequest>,
        config: Option<BatchConfig>,
    ) -> Result<BatchExecutionSummary, MCPUnifiedError> {
        let batch_id = Uuid::new_v4().to_string();
        let config = config.unwrap_or_default();
        let start_time = Instant::now();

        info!("Starting batch execution: {} with {} requests", batch_id, requests.len());

        // Create execution context
        let context = BatchExecutionContext::new(batch_id.clone(), requests, config);

        // Validate dependencies
        context.validate_dependencies()?;

        // Store context
        self.active_batches.write().await.insert(batch_id.clone(), context);

        // Execute batch
        let summary = self.execute_batch_internal(&batch_id).await?;

        // Clean up
        self.active_batches.write().await.remove(&batch_id);

        info!("Completed batch execution: {} in {}ms", batch_id, start_time.elapsed().as_millis());

        Ok(summary)
    }

    async fn execute_batch_internal(&self, batch_id: &str) -> Result<BatchExecutionSummary, MCPUnifiedError> {
        let semaphore = {
            let batches = self.active_batches.read().await;
            let context = batches.get(batch_id)
                .ok_or_else(|| MCPUnifiedError::Internal {
                    message: format!("Batch context not found: {batch_id}"),
                    source_error: None,
                    recovery_suggestion: Some("Ensure batch is properly initialized".to_string()),
                })?;
            Arc::new(Semaphore::new(context.config.max_concurrent))
        };

        let start_time = Instant::now();
        let mut completed_count = 0;
        let total_requests = {
            let batches = self.active_batches.read().await;
            batches.get(batch_id).ok_or_else(|| MCPUnifiedError::Internal {
                message: format!("Batch context not found: {batch_id}"),
                source_error: None,
                recovery_suggestion: Some("Ensure batch is properly initialized".to_string()),
            })?.requests.len()
        };

        // Main execution loop
        while completed_count < total_requests {
            let ready_requests = {
                let batches = self.active_batches.read().await;
                let context = batches.get(batch_id).ok_or_else(|| MCPUnifiedError::Internal {
                    message: format!("Batch context not found: {batch_id}"),
                    source_error: None,
                    recovery_suggestion: Some("Ensure batch is properly initialized".to_string()),
                })?;
                context.get_ready_requests().into_iter().cloned().collect::<Vec<_>>()
            };

            if ready_requests.is_empty() {
                // Check if we're stuck waiting for dependencies
                let remaining_pending = {
                    let batches = self.active_batches.read().await;
                    let context = batches.get(batch_id).ok_or_else(|| MCPUnifiedError::Internal {
                        message: format!("Batch context not found: {batch_id}"),
                        source_error: None,
                        recovery_suggestion: Some("Ensure batch is properly initialized".to_string()),
                    })?;
                    context.status_map.values()
                        .filter(|status| matches!(status, BatchRequestStatus::Pending))
                        .count()
                };

                if remaining_pending > 0 {
                    warn!("Batch {} has {} pending requests but no ready requests - possible dependency deadlock", batch_id, remaining_pending);
                    
                    // Mark remaining as failed due to dependency timeout
                    let mut batches = self.active_batches.write().await;
                    let context = batches.get_mut(batch_id).ok_or_else(|| MCPUnifiedError::Internal {
                        message: format!("Batch context not found: {batch_id}"),
                        source_error: None,
                        recovery_suggestion: Some("Ensure batch is properly initialized".to_string()),
                    })?;
                    
                    let pending_ids: Vec<String> = context.status_map
                        .iter()
                        .filter(|(_, status)| matches!(status, BatchRequestStatus::Pending))
                        .map(|(id, _)| id.clone())
                        .collect();

                    for id in pending_ids {
                        context.status_map.insert(id, BatchRequestStatus::Failed("Dependency timeout".to_string()));
                        completed_count += 1;
                    }
                }
                break;
            }

            // Execute ready requests in parallel
            let mut tasks = Vec::new();
            
            for request in ready_requests {
                // Mark as running
                {
                    let mut batches = self.active_batches.write().await;
                    let context = batches.get_mut(batch_id).ok_or_else(|| MCPUnifiedError::Internal {
                        message: format!("Batch context not found: {batch_id}"),
                        source_error: None,
                        recovery_suggestion: Some("Ensure batch is properly initialized".to_string()),
                    })?;
                    context.status_map.insert(request.id.clone(), BatchRequestStatus::Running);
                }

                let semaphore_clone = Arc::clone(&semaphore);
                let tool_registry_clone = Arc::clone(&self.tool_registry);
                let batch_id_clone = batch_id.to_string();
                let active_batches_clone = Arc::clone(&self.active_batches);
                let request_clone = request.clone();

                let task: tokio::task::JoinHandle<Result<BatchResponse, MCPUnifiedError>> = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.map_err(|_| MCPUnifiedError::Internal {
                        message: "Failed to acquire semaphore permit".to_string(),
                        source_error: None,
                        recovery_suggestion: Some("Check system resources and concurrency limits".to_string()),
                    })?;
                    
                    let execution_start = Instant::now();
                    let timeout = Duration::from_millis(
                        request_clone.timeout_ms.unwrap_or(30000)
                    );

                    debug!("Executing batch request: {} ({})", request_clone.id, request_clone.tool_name);

                    let result = tokio::time::timeout(
                        timeout,
                        tool_registry_clone.execute_tool(
                            &request_clone.tool_name,
                            &request_clone.params,
                            Some(&request_clone.id),
                            Some(&batch_id_clone),
                        )
                    ).await;

                    let execution_time = execution_start.elapsed();
                    let response = match result {
                        Ok(Ok(value)) => {
                            BatchResponse {
                                request_id: request_clone.id.clone(),
                                success: true,
                                result: Some(value),
                                error: None,
                                execution_time_ms: execution_time.as_millis() as u64,
                                dependencies_resolved: request_clone.depends_on.unwrap_or_default(),
                            }
                        }
                        Ok(Err(e)) => {
                            BatchResponse {
                                request_id: request_clone.id.clone(),
                                success: false,
                                result: None,
                                error: Some(e.to_string()),
                                execution_time_ms: execution_time.as_millis() as u64,
                                dependencies_resolved: request_clone.depends_on.unwrap_or_default(),
                            }
                        }
                        Err(_) => {
                            BatchResponse {
                                request_id: request_clone.id.clone(),
                                success: false,
                                result: None,
                                error: Some("Request timed out".to_string()),
                                execution_time_ms: execution_time.as_millis() as u64,
                                dependencies_resolved: request_clone.depends_on.unwrap_or_default(),
                            }
                        }
                    };

                    // Update context
                    {
                        let mut batches = active_batches_clone.write().await;
                        let context = batches.get_mut(&batch_id_clone).ok_or_else(|| MCPUnifiedError::Internal {
                            message: format!("Batch context not found: {batch_id_clone}"),
                            source_error: None,
                            recovery_suggestion: Some("Ensure batch is properly initialized".to_string()),
                        })?;

                        if response.success {
                            context.status_map.insert(request_clone.id.clone(), BatchRequestStatus::Completed);
                        } else {
                            context.mark_failed_cascade(&request_clone.id, response.error.clone().unwrap_or_default());
                        }

                        context.responses.insert(request_clone.id.clone(), response.clone());
                    }

                    Ok(response)
                });

                tasks.push(task);
            }

            // Wait for all tasks to complete
            for task in tasks {
                match task.await {
                    Ok(Ok(_)) => completed_count += 1,
                    Ok(Err(e)) => {
                        error!("Batch task error: {}", e);
                        completed_count += 1;
                    }
                    Err(e) => {
                        error!("Batch task join failed: {}", e);
                        completed_count += 1;
                    }
                }
            }

            // Small delay to prevent busy waiting
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Generate summary
        let summary = {
            let batches = self.active_batches.read().await;
            let context = batches.get(batch_id).ok_or_else(|| MCPUnifiedError::Internal {
                message: format!("Batch context not found: {batch_id}"),
                source_error: None,
                recovery_suggestion: Some("Ensure batch is properly initialized".to_string()),
            })?;
            
            let successful = context.responses.values().filter(|r| r.success).count();
            let failed = context.responses.values().filter(|r| !r.success).count();
            let skipped = context.status_map.values()
                .filter(|status| matches!(status, BatchRequestStatus::Skipped(_)))
                .count();

            let total_execution_time = start_time.elapsed();
            let sequential_time: u64 = context.responses.values()
                .map(|r| r.execution_time_ms)
                .sum();
            
            let parallel_efficiency = if sequential_time > 0 {
                sequential_time as f64 / total_execution_time.as_millis() as f64
            } else {
                0.0
            };

            BatchExecutionSummary {
                batch_id: batch_id.to_string(),
                total_requests: context.requests.len(),
                successful_requests: successful,
                failed_requests: failed,
                skipped_requests: skipped,
                total_execution_time_ms: total_execution_time.as_millis() as u64,
                parallel_efficiency,
                started_at: chrono::Utc::now().to_rfc3339(),
                completed_at: chrono::Utc::now().to_rfc3339(),
            }
        };

        Ok(summary)
    }

    /// Get batch status
    pub async fn get_batch_status(&self, batch_id: &str) -> Option<HashMap<String, BatchRequestStatus>> {
        let batches = self.active_batches.read().await;
        batches.get(batch_id).map(|context| context.status_map.clone())
    }

    /// Cancel a batch
    pub async fn cancel_batch(&self, batch_id: &str) -> bool {
        self.active_batches.write().await.remove(batch_id).is_some()
    }
}

#[async_trait]
impl MCPToolHandler for BatchMCPToolHandler {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let requests: Vec<BatchRequest> = serde_json::from_value(
            params.get("requests")
                .ok_or_else(|| anyhow::anyhow!("Missing 'requests' parameter"))?
                .clone()
        )?;

        if requests.is_empty() {
            return Err(anyhow::anyhow!("Batch requests cannot be empty"));
        }

        if requests.len() > 100 {
            return Err(anyhow::anyhow!("Batch size cannot exceed 100 requests"));
        }

        // Parse optional configuration
        let config = if let Some(config_value) = params.get("config") {
            serde_json::from_value(config_value.clone()).unwrap_or_default()
        } else {
            BatchConfig::default()
        };

        let summary = self.execute_batch(requests, Some(config)).await
            .map_err(|e| anyhow::anyhow!("Batch execution failed: {e}"))?;

        Ok(serde_json::to_value(summary)?)
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "requests": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string", "description": "Unique request identifier"},
                            "tool_name": {"type": "string", "description": "Name of the tool to execute"},
                            "params": {"type": "object", "description": "Parameters for the tool"},
                            "priority": {"type": "integer", "minimum": 1, "maximum": 10, "description": "Request priority (1-10)"},
                            "depends_on": {"type": "array", "items": {"type": "string"}, "description": "Request IDs this request depends on"},
                            "timeout_ms": {"type": "integer", "description": "Timeout in milliseconds"}
                        },
                        "required": ["id", "tool_name", "params"]
                    },
                    "minItems": 1,
                    "maxItems": 100
                },
                "config": {
                    "type": "object",
                    "properties": {
                        "max_concurrent": {"type": "integer", "minimum": 1, "maximum": 50, "default": 10},
                        "default_timeout": {"type": "integer", "default": 30000},
                        "fail_fast": {"type": "boolean", "default": false},
                        "dependency_timeout": {"type": "integer", "default": 300_000},
                        "enable_retry": {"type": "boolean", "default": true},
                        "retry_attempts": {"type": "integer", "minimum": 1, "maximum": 10, "default": 3}
                    }
                }
            },
            "required": ["requests"]
        })
    }

    fn get_description(&self) -> String {
        "Execute multiple MCP tools in batch with dependency management and parallel processing".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::communication::mcp_tool_registry::{ToolMetadata, PerformanceTier, CachingStrategy};
    use async_trait::async_trait;

    struct MockTool {
        delay_ms: u64,
        should_fail: bool,
    }

    #[async_trait]
    impl MCPToolHandler for MockTool {
        async fn execute(&self, _params: &Value) -> Result<Value> {
            tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
            if self.should_fail {
                Err(anyhow::anyhow!("Mock tool failure"))
            } else {
                Ok(json!({"result": "success"}))
            }
        }

        fn get_schema(&self) -> Value {
            json!({"type": "object"})
        }

        fn get_description(&self) -> String {
            "Mock tool for testing".to_string()
        }
    }

    #[tokio::test]
    async fn test_batch_execution() {
        let mut registry = MCPToolRegistry::new(300);
        
        // Register mock tools
        let fast_tool = MockTool { delay_ms: 10, should_fail: false };
        let slow_tool = MockTool { delay_ms: 50, should_fail: false };
        
        registry.register_simple_tool(
            "fast_tool".to_string(),
            fast_tool,
            "utilities",
            "Fast mock tool".to_string(),
            CachingStrategy::Never,
        ).unwrap();
        
        registry.register_simple_tool(
            "slow_tool".to_string(),
            slow_tool,
            "utilities",
            "Slow mock tool".to_string(),
            CachingStrategy::Never,
        ).unwrap();

        let batch_handler = BatchMCPToolHandler::new(Arc::new(registry));

        let requests = vec![
            BatchRequest {
                id: "req1".to_string(),
                tool_name: "fast_tool".to_string(),
                params: json!({}),
                priority: Some(5),
                depends_on: None,
                timeout_ms: Some(1000),
            },
            BatchRequest {
                id: "req2".to_string(),
                tool_name: "slow_tool".to_string(),
                params: json!({}),
                priority: Some(3),
                depends_on: Some(vec!["req1".to_string()]),
                timeout_ms: Some(1000),
            },
        ];

        let summary = batch_handler.execute_batch(requests, None).await.unwrap();
        
        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.successful_requests, 2);
        assert_eq!(summary.failed_requests, 0);
        assert!(summary.parallel_efficiency > 0.0);
    }

    #[tokio::test]
    async fn test_dependency_validation() {
        let context = BatchExecutionContext::new(
            "test".to_string(),
            vec![
                BatchRequest {
                    id: "req1".to_string(),
                    tool_name: "tool1".to_string(),
                    params: json!({}),
                    priority: None,
                    depends_on: Some(vec!["req2".to_string()]),
                    timeout_ms: None,
                },
                BatchRequest {
                    id: "req2".to_string(),
                    tool_name: "tool2".to_string(),
                    params: json!({}),
                    priority: None,
                    depends_on: Some(vec!["req1".to_string()]),
                    timeout_ms: None,
                },
            ],
            BatchConfig::default(),
        );

        let result = context.validate_dependencies();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dependency_cascade_failure() {
        let mut context = BatchExecutionContext::new(
            "test".to_string(),
            vec![
                BatchRequest {
                    id: "req1".to_string(),
                    tool_name: "tool1".to_string(),
                    params: json!({}),
                    priority: None,
                    depends_on: None,
                    timeout_ms: None,
                },
                BatchRequest {
                    id: "req2".to_string(),
                    tool_name: "tool2".to_string(),
                    params: json!({}),
                    priority: None,
                    depends_on: Some(vec!["req1".to_string()]),
                    timeout_ms: None,
                },
                BatchRequest {
                    id: "req3".to_string(),
                    tool_name: "tool3".to_string(),
                    params: json!({}),
                    priority: None,
                    depends_on: Some(vec!["req2".to_string()]),
                    timeout_ms: None,
                },
            ],
            BatchConfig::default(),
        );

        context.mark_failed_cascade("req1", "Test failure".to_string());

        assert!(matches!(context.status_map.get("req1"), Some(BatchRequestStatus::Failed(_))));
        assert!(matches!(context.status_map.get("req2"), Some(BatchRequestStatus::Skipped(_))));
        assert!(matches!(context.status_map.get("req3"), Some(BatchRequestStatus::Skipped(_))));
    }
}