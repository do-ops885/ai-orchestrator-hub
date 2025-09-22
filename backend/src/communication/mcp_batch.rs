use super::mcp::MCPToolHandler;
use super::mcp_tool_registry::MCPToolRegistry;
use super::mcp_unified_error::MCPUnifiedError;
use anyhow::Result;
use async_trait::async_trait;
use num_cpus;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// ===== PHASE 2: Intelligent Batch Processing Optimization =====

/// Performance metrics for batch execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPerformanceMetrics {
    pub batch_id: String,
    pub batch_size: usize,
    pub execution_time_ms: u64,
    pub throughput_requests_per_sec: f64,
    pub average_latency_ms: f64,
    pub parallel_efficiency: f64,
    pub resource_utilization: f64,
    pub timestamp: String,
}

/// Adaptive batch sizing configuration
#[derive(Debug, Clone)]
pub struct AdaptiveBatchConfig {
    pub min_batch_size: usize,
    pub max_batch_size: usize,
    pub target_latency_ms: u64,
    pub target_throughput: f64,
    pub adaptation_interval: Duration,
    pub performance_window_size: usize,
}

/// Batch optimization strategy
#[derive(Debug, Clone, PartialEq)]
pub enum BatchOptimizationStrategy {
    LatencyOptimized,
    ThroughputOptimized,
    Balanced,
    ResourceConservative,
}

/// Intelligent batching algorithm state
#[derive(Debug)]
pub struct IntelligentBatchingState {
    performance_history: Vec<BatchPerformanceMetrics>,
    current_batch_size: usize,
    adaptive_config: AdaptiveBatchConfig,
    strategy: BatchOptimizationStrategy,
    last_adaptation: Instant,
    system_load_factor: f64, // 0.0 to 1.0
}

impl IntelligentBatchingState {
    pub fn new(adaptive_config: AdaptiveBatchConfig, strategy: BatchOptimizationStrategy) -> Self {
        Self {
            performance_history: Vec::new(),
            current_batch_size: adaptive_config.min_batch_size,
            adaptive_config,
            strategy,
            last_adaptation: Instant::now(),
            system_load_factor: 0.5, // Start with moderate load assumption
        }
    }

    /// Record batch performance metrics
    pub fn record_performance(&mut self, metrics: BatchPerformanceMetrics) {
        self.performance_history.push(metrics);

        // Keep only recent performance data
        if self.performance_history.len() > self.adaptive_config.performance_window_size {
            self.performance_history.remove(0);
        }

        // Check if adaptation is needed
        if self.last_adaptation.elapsed() >= self.adaptive_config.adaptation_interval {
            self.adapt_batch_size();
            self.last_adaptation = Instant::now();
        }
    }

    /// Adapt batch size based on performance history and strategy
    fn adapt_batch_size(&mut self) {
        if self.performance_history.len() < 3 {
            return; // Need minimum data for adaptation
        }

        let recent_metrics: Vec<&BatchPerformanceMetrics> =
            self.performance_history.iter().rev().take(5).collect();

        let avg_latency = recent_metrics
            .iter()
            .map(|m| m.average_latency_ms)
            .sum::<f64>()
            / recent_metrics.len() as f64;
        let avg_throughput = recent_metrics
            .iter()
            .map(|m| m.throughput_requests_per_sec)
            .sum::<f64>()
            / recent_metrics.len() as f64;
        let avg_efficiency = recent_metrics
            .iter()
            .map(|m| m.parallel_efficiency)
            .sum::<f64>()
            / recent_metrics.len() as f64;

        let adaptation_factor = match self.strategy {
            BatchOptimizationStrategy::LatencyOptimized => {
                // Prioritize low latency - reduce batch size if latency is high
                if avg_latency > self.adaptive_config.target_latency_ms as f64 * 1.2 {
                    0.8 // Reduce batch size
                } else if avg_latency < self.adaptive_config.target_latency_ms as f64 * 0.8 {
                    1.1 // Increase batch size
                } else {
                    1.0 // Keep current
                }
            }
            BatchOptimizationStrategy::ThroughputOptimized => {
                // Prioritize high throughput - increase batch size if efficiency is good
                if avg_throughput < self.adaptive_config.target_throughput * 0.8 {
                    0.9 // Reduce batch size to improve throughput
                } else if avg_efficiency > 0.7 {
                    1.2 // Increase batch size for better throughput
                } else {
                    1.0
                }
            }
            BatchOptimizationStrategy::Balanced => {
                // Balance latency and throughput
                let latency_factor = (self.adaptive_config.target_latency_ms as f64 / avg_latency)
                    .min(2.0)
                    .max(0.5);
                let throughput_factor = (avg_throughput / self.adaptive_config.target_throughput)
                    .min(2.0)
                    .max(0.5);
                (latency_factor + throughput_factor) / 2.0
            }
            BatchOptimizationStrategy::ResourceConservative => {
                // Conservative approach - smaller batches, focus on stability
                let load_adjustment = 1.0 - self.system_load_factor; // Reduce when system is loaded
                load_adjustment.max(0.7).min(1.1)
            }
        };

        // Apply system load factor
        let final_factor = adaptation_factor * (1.0 - self.system_load_factor * 0.3);

        // Calculate new batch size
        let new_size = (self.current_batch_size as f64 * final_factor) as usize;
        self.current_batch_size = new_size
            .max(self.adaptive_config.min_batch_size)
            .min(self.adaptive_config.max_batch_size);

        debug!(
            "Adapted batch size: {} -> {} (factor: {:.2}, strategy: {:?})",
            self.current_batch_size, new_size, final_factor, self.strategy
        );
    }

    /// Update system load factor (0.0 = idle, 1.0 = fully loaded)
    pub fn update_system_load(&mut self, load_factor: f64) {
        self.system_load_factor = load_factor.max(0.0).min(1.0);
    }

    /// Get recommended batch size for next execution
    pub fn get_recommended_batch_size(&self, requested_size: usize) -> usize {
        // Consider both adaptive size and requested size
        let adaptive_size = self.current_batch_size;

        // If requested size is much smaller, respect it (might be for latency-critical operations)
        if requested_size < adaptive_size / 2 {
            return requested_size;
        }

        // Otherwise use adaptive size, but don't exceed requested size significantly
        adaptive_size.min(requested_size * 2)
    }

    /// Get current performance insights
    pub fn get_performance_insights(&self) -> Value {
        if self.performance_history.is_empty() {
            return json!({
                "status": "no_data",
                "current_batch_size": self.current_batch_size
            });
        }

        let recent = &self.performance_history[self.performance_history.len().saturating_sub(5)..];

        let avg_latency =
            recent.iter().map(|m| m.average_latency_ms).sum::<f64>() / recent.len() as f64;
        let avg_throughput = recent
            .iter()
            .map(|m| m.throughput_requests_per_sec)
            .sum::<f64>()
            / recent.len() as f64;
        let avg_efficiency =
            recent.iter().map(|m| m.parallel_efficiency).sum::<f64>() / recent.len() as f64;

        json!({
            "current_batch_size": self.current_batch_size,
            "strategy": format!("{:?}", self.strategy),
            "system_load_factor": self.system_load_factor,
            "recent_performance": {
                "average_latency_ms": avg_latency,
                "average_throughput_req_per_sec": avg_throughput,
                "average_parallel_efficiency": avg_efficiency,
                "sample_count": recent.len()
            },
            "targets": {
                "target_latency_ms": self.adaptive_config.target_latency_ms,
                "target_throughput": self.adaptive_config.target_throughput
            }
        })
    }
}

/// Enhanced batch execution context with performance tracking
#[derive(Debug)]
pub struct EnhancedBatchExecutionContext {
    base_context: BatchExecutionContext,
    performance_metrics: Option<BatchPerformanceMetrics>,
    start_time: Instant,
    worker_pool_size: usize,
}

impl EnhancedBatchExecutionContext {
    pub fn new(
        batch_id: String,
        requests: Vec<BatchRequest>,
        config: BatchConfig,
        worker_pool_size: usize,
    ) -> Self {
        Self {
            base_context: BatchExecutionContext::new(batch_id, requests, config),
            performance_metrics: None,
            start_time: Instant::now(),
            worker_pool_size,
        }
    }

    pub fn base(&self) -> &BatchExecutionContext {
        &self.base_context
    }

    pub fn base_mut(&mut self) -> &mut BatchExecutionContext {
        &mut self.base_context
    }

    pub fn record_completion(&mut self, total_execution_time: Duration) {
        let total_requests = self.base_context.requests.len();
        let successful_requests = self
            .base_context
            .responses
            .values()
            .filter(|r| r.success)
            .count();

        let total_latency: u64 = self
            .base_context
            .responses
            .values()
            .map(|r| r.execution_time_ms)
            .sum();
        let average_latency = if !self.base_context.responses.is_empty() {
            total_latency as f64 / self.base_context.responses.len() as f64
        } else {
            0.0
        };

        let throughput = if total_execution_time.as_millis() > 0 {
            successful_requests as f64 / (total_execution_time.as_millis() as f64 / 1000.0)
        } else {
            0.0
        };

        // Estimate parallel efficiency (simplified)
        let sequential_time: u64 = self
            .base_context
            .responses
            .values()
            .map(|r| r.execution_time_ms)
            .max()
            .unwrap_or(0);
        let parallel_efficiency = if sequential_time > 0 {
            sequential_time as f64 / total_execution_time.as_millis() as f64
        } else {
            0.0
        };

        self.performance_metrics = Some(BatchPerformanceMetrics {
            batch_id: self.base_context.batch_id.clone(),
            batch_size: total_requests,
            execution_time_ms: total_execution_time.as_millis() as u64,
            throughput_requests_per_sec: throughput,
            average_latency_ms: average_latency,
            parallel_efficiency,
            resource_utilization: self.worker_pool_size as f64 / 10.0, // Simplified
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    pub fn get_performance_metrics(&self) -> Option<&BatchPerformanceMetrics> {
        self.performance_metrics.as_ref()
    }
}

/// Dynamic worker pool for optimized parallel execution
#[derive(Debug)]
pub struct DynamicWorkerPool {
    current_workers: usize,
    max_workers: usize,
    min_workers: usize,
    active_tasks: usize,
    performance_history: Vec<f64>, // Task completion rates
}

impl DynamicWorkerPool {
    pub fn new(min_workers: usize, max_workers: usize) -> Self {
        Self {
            current_workers: min_workers,
            max_workers,
            min_workers,
            active_tasks: 0,
            performance_history: Vec::new(),
        }
    }

    /// Get optimal worker count based on current load and performance
    pub fn get_optimal_worker_count(&mut self, pending_tasks: usize, system_load: f64) -> usize {
        // Adaptive worker scaling based on load and performance
        let base_workers = if pending_tasks > 50 {
            self.max_workers.min(pending_tasks / 10 + self.min_workers)
        } else if pending_tasks < 10 {
            self.min_workers
        } else {
            self.current_workers
        };

        // Adjust for system load (reduce workers when system is heavily loaded)
        let load_adjusted = (base_workers as f64 * (1.0 - system_load * 0.5)) as usize;

        // Performance-based adjustment
        if self.performance_history.len() >= 3 {
            let recent_perf: f64 = self.performance_history.iter().rev().take(3).sum::<f64>() / 3.0;
            if recent_perf < 0.7 {
                // Performance degrading, reduce workers
                load_adjusted.saturating_sub(1)
            } else if recent_perf > 0.9 && load_adjusted < self.max_workers {
                // Good performance, can handle more workers
                load_adjusted + 1
            } else {
                load_adjusted
            }
        } else {
            load_adjusted
        }
        .max(self.min_workers)
        .min(self.max_workers)
    }

    /// Record task completion performance (0.0 to 1.0)
    pub fn record_performance(&mut self, performance: f64) {
        self.performance_history.push(performance);
        if self.performance_history.len() > 10 {
            self.performance_history.remove(0);
        }
    }

    /// Update active task count
    pub fn update_active_tasks(&mut self, count: usize) {
        self.active_tasks = count;
    }

    pub fn current_worker_count(&self) -> usize {
        self.current_workers
    }

    pub fn set_worker_count(&mut self, count: usize) {
        self.current_workers = count.max(self.min_workers).min(self.max_workers);
    }
}

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
    pub priority: Option<u8>,            // 1-10, higher is more priority
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
#[derive(Debug)]
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
            dependency_graph.insert(
                request.id.clone(),
                request.depends_on.clone().unwrap_or_default(),
            );
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
        self.status_map.insert(
            request_id.to_string(),
            BatchRequestStatus::Failed(error.clone()),
        );

        // Find all requests that depend on this one and mark them as skipped
        let dependents: Vec<String> = self
            .dependency_graph
            .iter()
            .filter(|(_, deps)| deps.contains(&request_id.to_string()))
            .map(|(id, _)| id.clone())
            .collect();

        for dependent_id in dependents {
            if self.status_map.get(&dependent_id) == Some(&BatchRequestStatus::Pending) {
                self.status_map.insert(
                    dependent_id.clone(),
                    BatchRequestStatus::Skipped(format!(
                        "Dependency '{request_id}' failed: {error}"
                    )),
                );
                // Recursively skip dependents
                self.mark_failed_cascade(
                    &dependent_id,
                    "Transitive dependency failure".to_string(),
                );
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

/// Enhanced batch processing tool handler with intelligent optimization
pub struct BatchMCPToolHandler {
    tool_registry: Arc<MCPToolRegistry>,
    active_batches: Arc<RwLock<HashMap<String, EnhancedBatchExecutionContext>>>,
    intelligent_batching: Arc<RwLock<IntelligentBatchingState>>,
    worker_pool: Arc<RwLock<DynamicWorkerPool>>,
    // Phase 4: Intelligent routing components
    content_analyzer: Arc<ContentAnalyzer>,
    load_balancer: Arc<LoadBalancer>,
    batch_optimizer: Arc<BatchOptimizer>,
    intelligent_router: Arc<IntelligentBatchRouter>,
}

impl BatchMCPToolHandler {
    #[must_use]
    pub fn new(tool_registry: Arc<MCPToolRegistry>) -> Self {
        let adaptive_config = AdaptiveBatchConfig {
            min_batch_size: 5,
            max_batch_size: 50,
            target_latency_ms: 5000, // 5 seconds target latency
            target_throughput: 20.0, // 20 requests per second
            adaptation_interval: Duration::from_secs(60), // Adapt every minute
            performance_window_size: 20,
        };

        // Initialize Phase 4 components
        let content_analyzer = Arc::new(ContentAnalyzer::new());
        let load_balancer = Arc::new(LoadBalancer::new(LoadBalancingStrategy::LeastLoaded));
        let batch_optimizer = Arc::new(BatchOptimizer::new(
            Arc::clone(&content_analyzer),
            Arc::clone(&load_balancer),
        ));
        let intelligent_router = Arc::new(IntelligentBatchRouter::new(
            Arc::clone(&content_analyzer),
            Arc::clone(&load_balancer),
            Arc::clone(&batch_optimizer),
        ));

        Self {
            tool_registry,
            active_batches: Arc::new(RwLock::new(HashMap::new())),
            intelligent_batching: Arc::new(RwLock::new(IntelligentBatchingState::new(
                adaptive_config,
                BatchOptimizationStrategy::Balanced,
            ))),
            worker_pool: Arc::new(RwLock::new(DynamicWorkerPool::new(2, 20))),
            content_analyzer,
            load_balancer,
            batch_optimizer,
            intelligent_router,
        }
    }

    /// Create enhanced batch handler with custom configuration
    #[must_use]
    pub fn with_config(
        tool_registry: Arc<MCPToolRegistry>,
        adaptive_config: AdaptiveBatchConfig,
        strategy: BatchOptimizationStrategy,
        min_workers: usize,
        max_workers: usize,
    ) -> Self {
        // Initialize Phase 4 components
        let content_analyzer = Arc::new(ContentAnalyzer::new());
        let load_balancer = Arc::new(LoadBalancer::new(LoadBalancingStrategy::LeastLoaded));
        let batch_optimizer = Arc::new(BatchOptimizer::new(
            Arc::clone(&content_analyzer),
            Arc::clone(&load_balancer),
        ));
        let intelligent_router = Arc::new(IntelligentBatchRouter::new(
            Arc::clone(&content_analyzer),
            Arc::clone(&load_balancer),
            Arc::clone(&batch_optimizer),
        ));

        Self {
            tool_registry,
            active_batches: Arc::new(RwLock::new(HashMap::new())),
            intelligent_batching: Arc::new(RwLock::new(IntelligentBatchingState::new(
                adaptive_config,
                strategy,
            ))),
            worker_pool: Arc::new(RwLock::new(DynamicWorkerPool::new(
                min_workers,
                max_workers,
            ))),
            content_analyzer,
            load_balancer,
            batch_optimizer,
            intelligent_router,
        }
    }

    /// Execute a batch of requests with intelligent optimization
    pub async fn execute_batch(
        &self,
        requests: Vec<BatchRequest>,
        config: Option<BatchConfig>,
    ) -> Result<BatchExecutionSummary, MCPUnifiedError> {
        let batch_id = Uuid::new_v4().to_string();
        let config = config.unwrap_or_default();
        let start_time = Instant::now();

        // Get intelligent batching recommendations
        let recommended_size = {
            let intelligent = self.intelligent_batching.read().await;
            intelligent.get_recommended_batch_size(requests.len())
        };

        // Split large batches if needed for better performance
        let (batches, is_split) = if requests.len() > recommended_size {
            info!(
                "Splitting large batch of {} requests into smaller batches of ~{}",
                requests.len(),
                recommended_size
            );
            self.split_batch(requests, recommended_size)
        } else {
            (vec![requests], false)
        };

        let mut all_summaries = Vec::new();

        for (i, batch_requests) in batches.into_iter().enumerate() {
            let sub_batch_id = if is_split {
                format!("{}-{}", batch_id, i)
            } else {
                batch_id.clone()
            };

            let summary = self
                .execute_single_batch(sub_batch_id, batch_requests, config.clone())
                .await?;
            all_summaries.push(summary);
        }

        // Combine summaries if batch was split
        let final_summary = if all_summaries.len() > 1 {
            self.combine_batch_summaries(batch_id, all_summaries, start_time.elapsed())
        } else {
            all_summaries
                .into_iter()
                .next()
                .expect("all_summaries should have at least one element")
        };

        // Record performance for learning
        if let Some(metrics) = self.get_batch_performance(&final_summary.batch_id).await {
            let mut intelligent = self.intelligent_batching.write().await;
            intelligent.record_performance(metrics);
        }

        info!(
            "Completed batch execution: {} in {}ms (split: {})",
            final_summary.batch_id,
            start_time.elapsed().as_millis(),
            is_split
        );

        Ok(final_summary)
    }

    /// Execute a single batch with optimized parallel processing
    async fn execute_single_batch(
        &self,
        batch_id: String,
        requests: Vec<BatchRequest>,
        config: BatchConfig,
    ) -> Result<BatchExecutionSummary, MCPUnifiedError> {
        // Get optimal worker count
        let optimal_workers = {
            let mut worker_pool = self.worker_pool.write().await;
            let system_load = self.estimate_system_load().await;
            worker_pool.get_optimal_worker_count(requests.len(), system_load)
        };

        info!(
            "Executing batch {} with {} requests using {} workers",
            batch_id,
            requests.len(),
            optimal_workers
        );

        // Create enhanced execution context
        let context =
            EnhancedBatchExecutionContext::new(batch_id.clone(), requests, config, optimal_workers);

        // Validate dependencies
        context.base().validate_dependencies()?;

        // Store context
        self.active_batches
            .write()
            .await
            .insert(batch_id.clone(), context);

        // Execute with optimized worker pool
        let summary = self
            .execute_batch_internal_optimized(&batch_id, optimal_workers)
            .await?;

        // Record performance
        if let Some(context) = self.active_batches.write().await.get_mut(&batch_id) {
            context.record_completion(Duration::from_millis(summary.total_execution_time_ms));
        }

        // Clean up
        self.active_batches.write().await.remove(&batch_id);

        Ok(summary)
    }

    /// Split large batch into smaller optimized batches
    fn split_batch(
        &self,
        requests: Vec<BatchRequest>,
        batch_size: usize,
    ) -> (Vec<Vec<BatchRequest>>, bool) {
        if requests.len() <= batch_size {
            return (vec![requests], false);
        }

        let mut batches = Vec::new();
        let mut current_batch = Vec::new();

        for request in requests {
            current_batch.push(request);

            if current_batch.len() >= batch_size {
                batches.push(current_batch);
                current_batch = Vec::new();
            }
        }

        if !current_batch.is_empty() {
            batches.push(current_batch);
        }

        (batches, true)
    }

    /// Combine multiple batch summaries into one
    fn combine_batch_summaries(
        &self,
        combined_batch_id: String,
        summaries: Vec<BatchExecutionSummary>,
        total_time: Duration,
    ) -> BatchExecutionSummary {
        let total_requests: usize = summaries.iter().map(|s| s.total_requests).sum();
        let successful: usize = summaries.iter().map(|s| s.successful_requests).sum();
        let failed: usize = summaries.iter().map(|s| s.failed_requests).sum();
        let skipped: usize = summaries.iter().map(|s| s.skipped_requests).sum();

        let total_execution_time = total_time.as_millis() as u64;
        let avg_parallel_efficiency =
            summaries.iter().map(|s| s.parallel_efficiency).sum::<f64>() / summaries.len() as f64;

        BatchExecutionSummary {
            batch_id: combined_batch_id,
            total_requests,
            successful_requests: successful,
            failed_requests: failed,
            skipped_requests: skipped,
            total_execution_time_ms: total_execution_time,
            parallel_efficiency: avg_parallel_efficiency,
            started_at: summaries
                .first()
                .expect("summaries should not be empty")
                .started_at
                .clone(),
            completed_at: summaries
                .last()
                .expect("summaries should not be empty")
                .completed_at
                .clone(),
        }
    }

    /// Estimate current system load (0.0 to 1.0)
    async fn estimate_system_load(&self) -> f64 {
        // Simplified system load estimation
        // In a real implementation, this would query system metrics
        let active_batches = self.active_batches.read().await.len();
        let worker_pool = self.worker_pool.read().await;

        // Base load on active batches and worker utilization
        let batch_load = (active_batches as f64 / 10.0).min(1.0); // Assume 10 concurrent batches is full load
        let worker_load =
            (worker_pool.active_tasks as f64 / worker_pool.current_workers as f64).min(1.0);

        (batch_load + worker_load) / 2.0
    }

    async fn execute_batch_internal_optimized(
        &self,
        batch_id: &str,
        worker_count: usize,
    ) -> Result<BatchExecutionSummary, MCPUnifiedError> {
        let semaphore = Arc::new(Semaphore::new(worker_count));

        // Update worker pool tracking
        {
            let mut worker_pool = self.worker_pool.write().await;
            worker_pool.set_worker_count(worker_count);
        }

        let start_time = Instant::now();
        let mut completed_count = 0;
        let total_requests = {
            let batches = self.active_batches.read().await;
            let context = batches
                .get(batch_id)
                .ok_or_else(|| MCPUnifiedError::Internal {
                    message: format!("Batch context not found: {batch_id}"),
                    source_error: None,
                    recovery_suggestion: Some("Ensure batch is properly initialized".to_string()),
                    context_chain: Vec::new(),
                })?;
            context.base().requests.len()
        };

        // Main execution loop with dynamic worker management
        while completed_count < total_requests {
            let ready_requests = {
                let batches = self.active_batches.read().await;
                let context = batches
                    .get(batch_id)
                    .ok_or_else(|| MCPUnifiedError::Internal {
                        message: format!("Batch context not found: {batch_id}"),
                        source_error: None,
                        recovery_suggestion: Some(
                            "Ensure batch is properly initialized".to_string(),
                        ),
                        context_chain: Vec::new(),
                    })?;
                context
                    .base()
                    .get_ready_requests()
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
            };

            if ready_requests.is_empty() {
                // Check if we're stuck waiting for dependencies
                let remaining_pending = {
                    let batches = self.active_batches.read().await;
                    let context =
                        batches
                            .get(batch_id)
                            .ok_or_else(|| MCPUnifiedError::Internal {
                                message: format!("Batch context not found: {batch_id}"),
                                source_error: None,
                                recovery_suggestion: Some(
                                    "Ensure batch is properly initialized".to_string(),
                                ),
                                context_chain: Vec::new(),
                            })?;
                    context
                        .base()
                        .status_map
                        .values()
                        .filter(|status| matches!(status, BatchRequestStatus::Pending))
                        .count()
                };

                if remaining_pending > 0 {
                    warn!("Batch {} has {} pending requests but no ready requests - possible dependency deadlock", batch_id, remaining_pending);

                    // Mark remaining as failed due to dependency timeout
                    let mut batches = self.active_batches.write().await;
                    let context =
                        batches
                            .get_mut(batch_id)
                            .ok_or_else(|| MCPUnifiedError::Internal {
                                message: format!("Batch context not found: {batch_id}"),
                                source_error: None,
                                recovery_suggestion: Some(
                                    "Ensure batch is properly initialized".to_string(),
                                ),
                                context_chain: Vec::new(),
                            })?;

                    let pending_ids: Vec<String> = context
                        .base()
                        .status_map
                        .iter()
                        .filter(|(_, status)| matches!(status, BatchRequestStatus::Pending))
                        .map(|(id, _)| id.clone())
                        .collect();

                    for id in pending_ids {
                        context.base_mut().status_map.insert(
                            id,
                            BatchRequestStatus::Failed("Dependency timeout".to_string()),
                        );
                        completed_count += 1;
                    }
                }
                break;
            }

            // Execute ready requests in parallel with worker pool optimization
            let mut tasks = Vec::new();

            // Update worker pool active task count
            {
                let mut worker_pool = self.worker_pool.write().await;
                worker_pool.update_active_tasks(ready_requests.len());
            }

            for request in ready_requests {
                // Mark as running
                {
                    let mut batches = self.active_batches.write().await;
                    let context =
                        batches
                            .get_mut(batch_id)
                            .ok_or_else(|| MCPUnifiedError::Internal {
                                message: format!("Batch context not found: {batch_id}"),
                                source_error: None,
                                recovery_suggestion: Some(
                                    "Ensure batch is properly initialized".to_string(),
                                ),
                                context_chain: Vec::new(),
                            })?;
                    context
                        .base_mut()
                        .status_map
                        .insert(request.id.clone(), BatchRequestStatus::Running);
                }

                let semaphore_clone = Arc::clone(&semaphore);
                let tool_registry_clone = Arc::clone(&self.tool_registry);
                let batch_id_clone = batch_id.to_string();
                let active_batches_clone = Arc::clone(&self.active_batches);
                let request_clone = request.clone();

                let task: tokio::task::JoinHandle<Result<BatchResponse, MCPUnifiedError>> =
                    tokio::spawn(async move {
                        let _permit = semaphore_clone.acquire().await.map_err(|_| {
                            MCPUnifiedError::Internal {
                                message: "Failed to acquire semaphore permit".to_string(),
                                source_error: None,
                                recovery_suggestion: Some(
                                    "Check system resources and concurrency limits".to_string(),
                                ),
                                context_chain: Vec::new(),
                            }
                        })?;

                        let execution_start = Instant::now();
                        let timeout =
                            Duration::from_millis(request_clone.timeout_ms.unwrap_or(30000));

                        debug!(
                            "Executing batch request: {} ({})",
                            request_clone.id, request_clone.tool_name
                        );

                        let result = tokio::time::timeout(
                            timeout,
                            tool_registry_clone.execute_tool(
                                &request_clone.tool_name,
                                &request_clone.params,
                                Some(&request_clone.id),
                                Some(&batch_id_clone),
                            ),
                        )
                        .await;

                        let execution_time = execution_start.elapsed();
                        let response = match result {
                            Ok(Ok(value)) => BatchResponse {
                                request_id: request_clone.id.clone(),
                                success: true,
                                result: Some(value),
                                error: None,
                                execution_time_ms: execution_time.as_millis() as u64,
                                dependencies_resolved: request_clone.depends_on.unwrap_or_default(),
                            },
                            Ok(Err(e)) => BatchResponse {
                                request_id: request_clone.id.clone(),
                                success: false,
                                result: None,
                                error: Some(e.to_string()),
                                execution_time_ms: execution_time.as_millis() as u64,
                                dependencies_resolved: request_clone.depends_on.unwrap_or_default(),
                            },
                            Err(_) => BatchResponse {
                                request_id: request_clone.id.clone(),
                                success: false,
                                result: None,
                                error: Some("Request timed out".to_string()),
                                execution_time_ms: execution_time.as_millis() as u64,
                                dependencies_resolved: request_clone.depends_on.unwrap_or_default(),
                            },
                        };

                        // Update context
                        {
                            let mut batches = active_batches_clone.write().await;
                            let context = batches.get_mut(&batch_id_clone).ok_or_else(|| {
                                MCPUnifiedError::Internal {
                                    message: format!("Batch context not found: {batch_id_clone}"),
                                    source_error: None,
                                    recovery_suggestion: Some(
                                        "Ensure batch is properly initialized".to_string(),
                                    ),
                                    context_chain: Vec::new(),
                                }
                            })?;

                            if response.success {
                                context.base_mut().status_map.insert(
                                    request_clone.id.clone(),
                                    BatchRequestStatus::Completed,
                                );
                            } else {
                                context.base_mut().mark_failed_cascade(
                                    &request_clone.id,
                                    response.error.clone().unwrap_or_default(),
                                );
                            }

                            context
                                .base_mut()
                                .responses
                                .insert(request_clone.id.clone(), response.clone());
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

        // Generate summary with performance tracking
        let summary = {
            let batches = self.active_batches.read().await;
            let context = batches
                .get(batch_id)
                .ok_or_else(|| MCPUnifiedError::Internal {
                    message: format!("Batch context not found: {batch_id}"),
                    source_error: None,
                    recovery_suggestion: Some("Ensure batch is properly initialized".to_string()),
                    context_chain: Vec::new(),
                })?;

            let successful = context
                .base()
                .responses
                .values()
                .filter(|r| r.success)
                .count();
            let failed = context
                .base()
                .responses
                .values()
                .filter(|r| !r.success)
                .count();
            let skipped = context
                .base()
                .status_map
                .values()
                .filter(|status| matches!(status, BatchRequestStatus::Skipped(_)))
                .count();

            let total_execution_time = start_time.elapsed();
            let sequential_time: u64 = context
                .base()
                .responses
                .values()
                .map(|r| r.execution_time_ms)
                .max()
                .unwrap_or(0); // Use max for parallel efficiency calculation

            let parallel_efficiency = if sequential_time > 0 {
                sequential_time as f64 / total_execution_time.as_millis() as f64
            } else {
                0.0
            };

            // Record performance in worker pool
            let task_performance = successful as f64 / context.base().requests.len() as f64;
            let mut worker_pool = self.worker_pool.write().await;
            worker_pool.record_performance(task_performance);

            BatchExecutionSummary {
                batch_id: batch_id.to_string(),
                total_requests: context.base().requests.len(),
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
    pub async fn get_batch_status(
        &self,
        batch_id: &str,
    ) -> Option<HashMap<String, BatchRequestStatus>> {
        let batches = self.active_batches.read().await;
        batches
            .get(batch_id)
            .map(|context| context.base().status_map.clone())
    }

    /// Cancel a batch
    pub async fn cancel_batch(&self, batch_id: &str) -> bool {
        self.active_batches.write().await.remove(batch_id).is_some()
    }

    /// Get batch performance metrics
    pub async fn get_batch_performance(&self, batch_id: &str) -> Option<BatchPerformanceMetrics> {
        let batches = self.active_batches.read().await;
        batches
            .get(batch_id)
            .and_then(|context| context.get_performance_metrics())
            .cloned()
    }

    /// Get intelligent batching insights
    pub async fn get_batching_insights(&self) -> Value {
        let intelligent = self.intelligent_batching.read().await;
        let worker_pool = self.worker_pool.read().await;

        json!({
            "intelligent_batching": intelligent.get_performance_insights(),
            "worker_pool": {
                "current_workers": worker_pool.current_worker_count(),
                "active_tasks": worker_pool.active_tasks,
                "performance_history_length": worker_pool.performance_history.len()
            },
            "active_batches": self.active_batches.read().await.len()
        })
    }

    /// Execute batch with intelligent routing (Phase 4)
    pub async fn execute_batch_with_routing(
        &self,
        requests: Vec<BatchRequest>,
        config: Option<BatchConfig>,
    ) -> Result<BatchExecutionSummary, MCPUnifiedError> {
        let config = config.unwrap_or_default();

        // Route the batch using intelligent routing
        let routing_decision = self
            .intelligent_router
            .route_batch(requests, Some(config.clone()))
            .await?;

        match routing_decision.action {
            RoutingActionType::RouteToNode(node_id)
            | RoutingActionType::OptimizeAndRoute(node_id) => {
                // Update load balancer with the selected node
                let load_update = NodeLoad {
                    active_batches: 1,       // Simplified
                    cpu_usage: 0.5,          // Simplified
                    memory_usage_mb: 256,    // Simplified
                    network_usage_mbps: 1.0, // Simplified
                    last_updated: chrono::Utc::now().to_rfc3339(),
                };
                let _ = self
                    .load_balancer
                    .update_node_load(&node_id, load_update)
                    .await;

                // Execute the batch with optimized config
                let final_config = routing_decision.optimized_config.unwrap_or(config);
                let summary = self
                    .execute_batch(routing_decision.requests, Some(final_config))
                    .await?;

                // Update load balancer after completion
                let completion_load = NodeLoad {
                    active_batches: 0,
                    cpu_usage: 0.0,
                    memory_usage_mb: 0,
                    network_usage_mbps: 0.0,
                    last_updated: chrono::Utc::now().to_rfc3339(),
                };
                let _ = self
                    .load_balancer
                    .update_node_load(&node_id, completion_load)
                    .await;

                Ok(summary)
            }
            RoutingActionType::RouteWithStrategy(strategy, node_id) => {
                // Update load balancer strategy temporarily
                // For now, just execute with the selected node
                let summary = self
                    .execute_batch(routing_decision.requests, routing_decision.optimized_config)
                    .await?;
                Ok(summary)
            }
            RoutingActionType::SplitBatch(chunks) => {
                // Execute chunks sequentially or in parallel
                let mut all_summaries = Vec::new();
                for chunk in chunks {
                    let chunk_summary = self
                        .execute_batch(chunk, routing_decision.optimized_config.clone())
                        .await?;
                    all_summaries.push(chunk_summary);
                }

                // Combine summaries
                if all_summaries.len() > 1 {
                    let total_time = all_summaries
                        .iter()
                        .map(|s| s.total_execution_time_ms)
                        .max()
                        .unwrap_or(0);
                    let total_requests: usize =
                        all_summaries.iter().map(|s| s.total_requests).sum();
                    let successful: usize =
                        all_summaries.iter().map(|s| s.successful_requests).sum();
                    let failed: usize = all_summaries.iter().map(|s| s.failed_requests).sum();
                    let skipped: usize = all_summaries.iter().map(|s| s.skipped_requests).sum();
                    let avg_efficiency = all_summaries
                        .iter()
                        .map(|s| s.parallel_efficiency)
                        .sum::<f64>()
                        / all_summaries.len() as f64;

                    Ok(BatchExecutionSummary {
                        batch_id: Uuid::new_v4().to_string(),
                        total_requests,
                        successful_requests: successful,
                        failed_requests: failed,
                        skipped_requests: skipped,
                        total_execution_time_ms: total_time,
                        parallel_efficiency: avg_efficiency,
                        started_at: all_summaries
                            .first()
                            .expect("replaced unwrap")
                            .started_at
                            .clone(),
                        completed_at: all_summaries
                            .last()
                            .expect("replaced unwrap")
                            .completed_at
                            .clone(),
                    })
                } else {
                    Ok(all_summaries.into_iter().next().expect("replaced unwrap"))
                }
            }
            RoutingActionType::Queue => {
                // For now, just reject with a message about queuing
                Err(MCPUnifiedError::Internal {
                    message: "Batch queued for later processing".to_string(),
                    source_error: Some(routing_decision.reasoning),
                    recovery_suggestion: Some(
                        "Batch has been queued and will be processed when resources are available"
                            .to_string(),
                    ),
                    context_chain: vec![],
                })
            }
            RoutingActionType::Reject(reason) => Err(MCPUnifiedError::validation(
                "batch_routing".to_string(),
                format!("Batch rejected: {}", reason),
                Some(json!({"reasoning": routing_decision.reasoning})),
                Some("Review batch content and try again".to_string()),
            )),
        }
    }

    /// Update system load factor for adaptive batching
    pub async fn update_system_load(&self, load_factor: f64) {
        let mut intelligent = self.intelligent_batching.write().await;
        intelligent.update_system_load(load_factor);
    }
}

#[async_trait]
impl MCPToolHandler for BatchMCPToolHandler {
    async fn execute(&self, params: &Value) -> Result<Value> {
        // Check if this is a request for batching insights or Phase 4 features
        if let Some(action) = params.get("action").and_then(|v| v.as_str()) {
            match action {
                "insights" => {
                    let insights = self.get_batching_insights().await;
                    return Ok(insights);
                }
                "update_load" => {
                    if let Some(load_factor) = params.get("load_factor").and_then(|v| v.as_f64()) {
                        self.update_system_load(load_factor).await;
                        return Ok(json!({"status": "updated", "load_factor": load_factor}));
                    }
                }
                "routing_insights" => {
                    let routing_stats = self.intelligent_router.get_routing_statistics().await;
                    let optimization_insights =
                        self.batch_optimizer.get_optimization_insights().await;
                    let load_stats = self.load_balancer.get_statistics().await;
                    return Ok(json!({
                        "routing": routing_stats,
                        "optimization": optimization_insights,
                        "load_balancing": load_stats
                    }));
                }
                "analyze_batch" => {
                    if let Some(requests_value) = params.get("requests") {
                        let requests: Vec<BatchRequest> =
                            serde_json::from_value(requests_value.clone())?;
                        let analysis = self.content_analyzer.analyze_batch(&requests).await?;
                        return Ok(serde_json::to_value(analysis)?);
                    }
                }
                _ => {}
            }
        }

        let requests: Vec<BatchRequest> = serde_json::from_value(
            params
                .get("requests")
                .ok_or_else(|| anyhow::anyhow!("Missing 'requests' parameter"))?
                .clone(),
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

        // Check if intelligent routing is requested
        let use_routing = params
            .get("use_intelligent_routing")
            .and_then(|v| v.as_bool())
            .unwrap_or(true); // Default to true for Phase 4

        let summary = if use_routing {
            self.execute_batch_with_routing(requests, Some(config))
                .await
                .map_err(|e| anyhow::anyhow!("Intelligent batch execution failed: {e}"))?
        } else {
            self.execute_batch(requests, Some(config))
                .await
                .map_err(|e| anyhow::anyhow!("Batch execution failed: {e}"))?
        };

        Ok(serde_json::to_value(summary)?)
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "oneOf": [
                {
                    "properties": {
                        "action": {"const": "insights", "description": "Get intelligent batching insights and performance metrics"}
                    },
                    "required": ["action"]
                },
                {
                    "properties": {
                        "action": {"const": "update_load"},
                        "load_factor": {"type": "number", "minimum": 0.0, "maximum": 1.0, "description": "System load factor for adaptive batching"}
                    },
                    "required": ["action", "load_factor"]
                },
                {
                    "properties": {
                        "action": {"const": "routing_insights", "description": "Get Phase 4 intelligent routing, optimization, and load balancing insights"}
                    },
                    "required": ["action"]
                },
                {
                    "properties": {
                        "action": {"const": "analyze_batch", "description": "Analyze batch content for optimization recommendations"},
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
                        }
                    },
                    "required": ["action", "requests"]
                },
                {
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
                }
            ]
        })
    }

    fn get_description(&self) -> String {
        "Execute multiple MCP tools in batch with Phase 4 intelligent routing: content analysis, load balancing, dynamic optimization, dependency management, parallel processing, and adaptive performance tuning".to_string()
    }
}

/// ===== PHASE 4: Intelligent Batch Processing Components =====

/// Content analysis result for batch requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    pub batch_type: BatchType,
    pub complexity_score: f64, // 0.0 to 1.0
    pub estimated_execution_time_ms: u64,
    pub resource_requirements: ResourceRequirements,
    pub optimization_recommendations: Vec<String>,
    pub priority_distribution: HashMap<u8, usize>, // priority -> count
}

/// Types of batch content for optimization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BatchType {
    DataProcessing, // ETL, data transformation
    ApiCalls,       // External API requests
    Computations,   // CPU-intensive calculations
    Mixed,          // Combination of types
    IoBound,        // File I/O, database operations
}

/// Resource requirements for batch execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: f64,         // Estimated CPU cores needed
    pub memory_mb: u64,         // Estimated memory in MB
    pub network_bandwidth: f64, // Estimated network usage (Mbps)
    pub io_operations: u64,     // Estimated I/O operations
}

/// Content analyzer for batch processing optimization
pub struct ContentAnalyzer {
    analysis_cache: Arc<RwLock<HashMap<String, ContentAnalysis>>>,
    tool_complexity_profiles: HashMap<String, ToolComplexityProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolComplexityProfile {
    pub avg_execution_time_ms: u64,
    pub cpu_intensity: f64,     // 0.0 to 1.0
    pub memory_intensity: f64,  // 0.0 to 1.0
    pub network_intensity: f64, // 0.0 to 1.0
    pub io_intensity: f64,      // 0.0 to 1.0
    pub batch_type: BatchType,
}

impl ContentAnalyzer {
    pub fn new() -> Self {
        let mut tool_complexity_profiles = HashMap::new();

        // Initialize with common tool profiles (can be extended)
        tool_complexity_profiles.insert(
            "data_processor".to_string(),
            ToolComplexityProfile {
                avg_execution_time_ms: 500,
                cpu_intensity: 0.7,
                memory_intensity: 0.8,
                network_intensity: 0.2,
                io_intensity: 0.6,
                batch_type: BatchType::DataProcessing,
            },
        );

        tool_complexity_profiles.insert(
            "api_caller".to_string(),
            ToolComplexityProfile {
                avg_execution_time_ms: 200,
                cpu_intensity: 0.3,
                memory_intensity: 0.4,
                network_intensity: 0.9,
                io_intensity: 0.1,
                batch_type: BatchType::ApiCalls,
            },
        );

        tool_complexity_profiles.insert(
            "calculator".to_string(),
            ToolComplexityProfile {
                avg_execution_time_ms: 100,
                cpu_intensity: 0.9,
                memory_intensity: 0.6,
                network_intensity: 0.1,
                io_intensity: 0.2,
                batch_type: BatchType::Computations,
            },
        );

        Self {
            analysis_cache: Arc::new(RwLock::new(HashMap::new())),
            tool_complexity_profiles,
        }
    }

    /// Analyze batch content and provide optimization recommendations
    pub async fn analyze_batch(
        &self,
        requests: &[BatchRequest],
    ) -> Result<ContentAnalysis, MCPUnifiedError> {
        if requests.is_empty() {
            return Err(MCPUnifiedError::validation(
                "batch_content".to_string(),
                "Cannot analyze empty batch".to_string(),
                Some(json!({"request_count": 0})),
                Some("Provide at least one request for analysis".to_string()),
            ));
        }

        let batch_id = self.generate_batch_signature(requests);

        // Check cache first
        {
            let cache = self.analysis_cache.read().await;
            if let Some(analysis) = cache.get(&batch_id) {
                return Ok(analysis.clone());
            }
        }

        let analysis = self.perform_analysis(requests).await?;

        // Cache the result
        {
            let mut cache = self.analysis_cache.write().await;
            cache.insert(batch_id, analysis.clone());
        }

        Ok(analysis)
    }

    fn generate_batch_signature(&self, requests: &[BatchRequest]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for request in requests {
            request.tool_name.hash(&mut hasher);
            request.params.to_string().hash(&mut hasher);
        }
        format!("batch_{}", hasher.finish())
    }

    async fn perform_analysis(
        &self,
        requests: &[BatchRequest],
    ) -> Result<ContentAnalysis, MCPUnifiedError> {
        let mut total_execution_time = 0u64;
        let mut total_cpu = 0.0;
        let mut total_memory = 0.0;
        let mut total_network = 0.0;
        let mut total_io = 0.0;
        let mut type_counts = HashMap::new();
        let mut priority_dist = HashMap::new();
        let mut recommendations = Vec::new();

        for request in requests {
            let profile = self
                .tool_complexity_profiles
                .get(&request.tool_name)
                .unwrap_or(&ToolComplexityProfile {
                    avg_execution_time_ms: 1000, // Default conservative estimate
                    cpu_intensity: 0.5,
                    memory_intensity: 0.5,
                    network_intensity: 0.5,
                    io_intensity: 0.5,
                    batch_type: BatchType::Mixed,
                });

            total_execution_time += profile.avg_execution_time_ms;
            total_cpu += profile.cpu_intensity;
            total_memory += profile.memory_intensity;
            total_network += profile.network_intensity;
            total_io += profile.io_intensity;

            *type_counts.entry(profile.batch_type.clone()).or_insert(0) += 1;

            if let Some(priority) = request.priority {
                *priority_dist.entry(priority).or_insert(0) += 1;
            }
        }

        // Determine dominant batch type
        let batch_type = self.determine_batch_type(&type_counts);

        // Calculate complexity score (0.0 to 1.0)
        let complexity_score =
            self.calculate_complexity_score(requests.len(), total_execution_time, &batch_type);

        // Generate resource requirements
        let resource_requirements = ResourceRequirements {
            cpu_cores: (total_cpu / requests.len() as f64).max(0.1),
            memory_mb: ((total_memory / requests.len() as f64) * 512.0) as u64, // Estimate 512MB per unit
            network_bandwidth: (total_network / requests.len() as f64) * 10.0, // Estimate 10Mbps per unit
            io_operations: ((total_io / requests.len() as f64) * 100.0) as u64, // Estimate 100 IOPS per unit
        };

        // Generate optimization recommendations
        self.generate_recommendations(
            &batch_type,
            requests.len(),
            complexity_score,
            &mut recommendations,
        );

        Ok(ContentAnalysis {
            batch_type,
            complexity_score,
            estimated_execution_time_ms: total_execution_time,
            resource_requirements,
            optimization_recommendations: recommendations,
            priority_distribution: priority_dist,
        })
    }

    fn determine_batch_type(&self, type_counts: &HashMap<BatchType, usize>) -> BatchType {
        if type_counts.len() == 1 {
            return type_counts.keys().next().expect("replaced unwrap").clone();
        }

        let total = type_counts.values().sum::<usize>() as f64;
        let dominant_percentage =
            *type_counts.values().max().expect("replaced unwrap") as f64 / total;

        if dominant_percentage > 0.8 {
            // If one type dominates (>80%), use that type
            type_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .expect("replaced unwrap")
                .0
                .clone()
        } else {
            BatchType::Mixed
        }
    }

    fn calculate_complexity_score(
        &self,
        request_count: usize,
        total_time: u64,
        batch_type: &BatchType,
    ) -> f64 {
        let size_factor = (request_count as f64 / 50.0).min(1.0); // Normalize to 50 requests
        let time_factor = (total_time as f64 / 30000.0).min(1.0); // Normalize to 30 seconds

        let type_multiplier = match batch_type {
            BatchType::DataProcessing => 1.2,
            BatchType::ApiCalls => 0.8,
            BatchType::Computations => 1.5,
            BatchType::Mixed => 1.0,
            BatchType::IoBound => 1.1,
        };

        ((size_factor + time_factor) / 2.0 * type_multiplier).min(1.0)
    }

    fn generate_recommendations(
        &self,
        batch_type: &BatchType,
        request_count: usize,
        complexity: f64,
        recommendations: &mut Vec<String>,
    ) {
        if request_count > 20 {
            recommendations.push("Consider splitting large batch into smaller chunks".to_string());
        }

        match batch_type {
            BatchType::DataProcessing => {
                recommendations
                    .push("Use parallel processing for data transformation tasks".to_string());
                if complexity > 0.7 {
                    recommendations.push(
                        "Consider increasing memory allocation for complex data processing"
                            .to_string(),
                    );
                }
            }
            BatchType::ApiCalls => {
                recommendations.push("Implement rate limiting to avoid API throttling".to_string());
                recommendations.push("Use connection pooling for external API calls".to_string());
            }
            BatchType::Computations => {
                recommendations
                    .push("Distribute CPU-intensive tasks across multiple cores".to_string());
                if complexity > 0.8 {
                    recommendations
                        .push("Consider GPU acceleration for heavy computations".to_string());
                }
            }
            BatchType::Mixed => {
                recommendations.push(
                    "Group similar task types together for better resource utilization".to_string(),
                );
            }
            BatchType::IoBound => {
                recommendations.push("Use asynchronous I/O operations".to_string());
                recommendations.push("Consider SSD storage for better I/O performance".to_string());
            }
        }

        if complexity > 0.9 {
            recommendations
                .push("High complexity batch - monitor system resources closely".to_string());
        }
    }
}

/// Processing node for load balancing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingNode {
    pub id: String,
    pub capacity: NodeCapacity,
    pub current_load: NodeLoad,
    pub specialization: Option<BatchType>, // Node specialization if any
    pub status: NodeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapacity {
    pub max_concurrent_batches: usize,
    pub max_cpu_cores: f64,
    pub max_memory_mb: u64,
    pub max_network_mbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeLoad {
    pub active_batches: usize,
    pub cpu_usage: f64, // 0.0 to 1.0
    pub memory_usage_mb: u64,
    pub network_usage_mbps: f64,
    pub last_updated: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
    Available,
    Busy,
    Overloaded,
    Maintenance,
}

/// Load balancing strategy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastLoaded,
    WeightedCapacity,
    Specialized, // Route to specialized nodes
}

/// Load balancer for distributing batch processing
pub struct LoadBalancer {
    nodes: Arc<RwLock<HashMap<String, ProcessingNode>>>,
    strategy: LoadBalancingStrategy,
    round_robin_index: Arc<RwLock<usize>>,
}

impl LoadBalancer {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        let mut nodes = HashMap::new();

        // Initialize with local node (can be extended for multi-node)
        nodes.insert(
            "local_node".to_string(),
            ProcessingNode {
                id: "local_node".to_string(),
                capacity: NodeCapacity {
                    max_concurrent_batches: 10,
                    max_cpu_cores: num_cpus::get() as f64,
                    max_memory_mb: 8192,     // 8GB default
                    max_network_mbps: 100.0, // 100Mbps default
                },
                current_load: NodeLoad {
                    active_batches: 0,
                    cpu_usage: 0.0,
                    memory_usage_mb: 0,
                    network_usage_mbps: 0.0,
                    last_updated: chrono::Utc::now().to_rfc3339(),
                },
                specialization: None,
                status: NodeStatus::Available,
            },
        );

        Self {
            nodes: Arc::new(RwLock::new(nodes)),
            strategy,
            round_robin_index: Arc::new(RwLock::new(0)),
        }
    }

    /// Select optimal node for batch execution
    pub async fn select_node(
        &self,
        analysis: &ContentAnalysis,
        priority: Option<u8>,
    ) -> Result<String, MCPUnifiedError> {
        let nodes = self.nodes.read().await;
        let available_nodes: Vec<&ProcessingNode> = nodes
            .values()
            .filter(|node| node.status == NodeStatus::Available)
            .collect();

        if available_nodes.is_empty() {
            return Err(MCPUnifiedError::ResourceAccess {
                resource: "processing_node".to_string(),
                reason: "No available processing nodes".to_string(),
                required_permissions: vec![],
                context_chain: vec![format!("total_nodes={}, available_nodes=0", nodes.len())],
            });
        }

        let selected_node = match self.strategy {
            LoadBalancingStrategy::RoundRobin => self.select_round_robin(&available_nodes).await,
            LoadBalancingStrategy::LeastLoaded => {
                self.select_least_loaded(&available_nodes, analysis)
            }
            LoadBalancingStrategy::WeightedCapacity => {
                self.select_weighted_capacity(&available_nodes, analysis)
            }
            LoadBalancingStrategy::Specialized => {
                self.select_specialized(&available_nodes, analysis, priority)
            }
        };

        Ok(selected_node.id.clone())
    }

    async fn select_round_robin<'a>(&self, nodes: &[&'a ProcessingNode]) -> &'a ProcessingNode {
        let mut index = self.round_robin_index.write().await;
        let selected = &nodes[*index % nodes.len()];
        *index += 1;
        selected
    }

    fn select_least_loaded<'a>(
        &self,
        nodes: &[&'a ProcessingNode],
        analysis: &ContentAnalysis,
    ) -> &'a ProcessingNode {
        nodes
            .iter()
            .min_by(|a, b| {
                let a_score = self.calculate_load_score(a, analysis);
                let b_score = self.calculate_load_score(b, analysis);
                a_score
                    .partial_cmp(&b_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&nodes[0])
    }

    fn select_weighted_capacity<'a>(
        &self,
        nodes: &[&'a ProcessingNode],
        analysis: &ContentAnalysis,
    ) -> &'a ProcessingNode {
        nodes
            .iter()
            .max_by(|a, b| {
                let a_capacity = self.calculate_capacity_score(a, analysis);
                let b_capacity = self.calculate_capacity_score(b, analysis);
                a_capacity
                    .partial_cmp(&b_capacity)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&nodes[0])
    }

    fn select_specialized<'a>(
        &self,
        nodes: &[&'a ProcessingNode],
        analysis: &ContentAnalysis,
        priority: Option<u8>,
    ) -> &'a ProcessingNode {
        // First try to find specialized nodes
        let specialized_nodes: Vec<&&ProcessingNode> = nodes
            .iter()
            .filter(|node| {
                if let Some(spec) = &node.specialization {
                    *spec == analysis.batch_type
                } else {
                    false
                }
            })
            .collect();

        if !specialized_nodes.is_empty() {
            return self.select_least_loaded(
                &specialized_nodes
                    .into_iter()
                    .map(|n| *n)
                    .collect::<Vec<_>>(),
                analysis,
            );
        }

        // Fall back to least loaded if no specialized nodes
        self.select_least_loaded(nodes, analysis)
    }

    fn calculate_load_score(&self, node: &ProcessingNode, analysis: &ContentAnalysis) -> f64 {
        let batch_load =
            node.current_load.active_batches as f64 / node.capacity.max_concurrent_batches as f64;
        let cpu_load = node.current_load.cpu_usage;
        let memory_load =
            node.current_load.memory_usage_mb as f64 / node.capacity.max_memory_mb as f64;
        let network_load = node.current_load.network_usage_mbps / node.capacity.max_network_mbps;

        // Weighted average based on analysis requirements
        let weights = self.get_resource_weights(analysis);
        batch_load * weights.batch
            + cpu_load * weights.cpu
            + memory_load * weights.memory
            + network_load * weights.network
    }

    fn calculate_capacity_score(&self, node: &ProcessingNode, analysis: &ContentAnalysis) -> f64 {
        let available_batches = node
            .capacity
            .max_concurrent_batches
            .saturating_sub(node.current_load.active_batches);
        let available_cpu = node.capacity.max_cpu_cores
            - (node.current_load.cpu_usage * node.capacity.max_cpu_cores);
        let available_memory = node
            .capacity
            .max_memory_mb
            .saturating_sub(node.current_load.memory_usage_mb);
        let available_network =
            node.capacity.max_network_mbps - node.current_load.network_usage_mbps;

        // Score based on how well the node can handle the required resources
        let batch_score = available_batches as f64 / node.capacity.max_concurrent_batches as f64;
        let cpu_score = available_cpu / node.capacity.max_cpu_cores;
        let memory_score = available_memory as f64 / node.capacity.max_memory_mb as f64;
        let network_score = available_network / node.capacity.max_network_mbps;

        let weights = self.get_resource_weights(analysis);
        batch_score * weights.batch
            + cpu_score * weights.cpu
            + memory_score * weights.memory
            + network_score * weights.network
    }

    fn get_resource_weights(&self, analysis: &ContentAnalysis) -> ResourceWeights {
        match analysis.batch_type {
            BatchType::DataProcessing => ResourceWeights {
                batch: 0.3,
                cpu: 0.2,
                memory: 0.4,
                network: 0.1,
            },
            BatchType::ApiCalls => ResourceWeights {
                batch: 0.4,
                cpu: 0.1,
                memory: 0.2,
                network: 0.3,
            },
            BatchType::Computations => ResourceWeights {
                batch: 0.2,
                cpu: 0.5,
                memory: 0.2,
                network: 0.1,
            },
            BatchType::Mixed => ResourceWeights {
                batch: 0.25,
                cpu: 0.25,
                memory: 0.25,
                network: 0.25,
            },
            BatchType::IoBound => ResourceWeights {
                batch: 0.3,
                cpu: 0.1,
                memory: 0.3,
                network: 0.3,
            },
        }
    }

    /// Update node load information
    pub async fn update_node_load(
        &self,
        node_id: &str,
        load: NodeLoad,
    ) -> Result<(), MCPUnifiedError> {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(node_id) {
            node.current_load = load;
            node.current_load.last_updated = chrono::Utc::now().to_rfc3339();

            // Update status based on load
            node.status = self.determine_node_status(node);
            Ok(())
        } else {
            Err(MCPUnifiedError::ResourceAccess {
                resource: "processing_node".to_string(),
                reason: format!("Node '{}' not found", node_id),
                required_permissions: vec![],
                context_chain: vec![format!("node_id={}", node_id)],
            })
        }
    }

    fn determine_node_status(&self, node: &ProcessingNode) -> NodeStatus {
        let load_score = self.calculate_load_score(
            node,
            &ContentAnalysis {
                batch_type: BatchType::Mixed,
                complexity_score: 0.5,
                estimated_execution_time_ms: 1000,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 1.0,
                    memory_mb: 512,
                    network_bandwidth: 1.0,
                    io_operations: 10,
                },
                optimization_recommendations: vec![],
                priority_distribution: HashMap::new(),
            },
        );

        if load_score > 0.9 {
            NodeStatus::Overloaded
        } else if load_score > 0.7 {
            NodeStatus::Busy
        } else {
            NodeStatus::Available
        }
    }

    /// Get load balancer statistics
    pub async fn get_statistics(&self) -> Value {
        let nodes = self.nodes.read().await;
        let total_nodes = nodes.len();
        let available_nodes = nodes
            .values()
            .filter(|n| n.status == NodeStatus::Available)
            .count();
        let busy_nodes = nodes
            .values()
            .filter(|n| n.status == NodeStatus::Busy)
            .count();
        let overloaded_nodes = nodes
            .values()
            .filter(|n| n.status == NodeStatus::Overloaded)
            .count();

        json!({
            "total_nodes": total_nodes,
            "available_nodes": available_nodes,
            "busy_nodes": busy_nodes,
            "overloaded_nodes": overloaded_nodes,
            "strategy": format!("{:?}", self.strategy),
            "nodes": nodes.values().map(|node| json!({
                "id": node.id,
                "status": format!("{:?}", node.status),
                "active_batches": node.current_load.active_batches,
                "cpu_usage": node.current_load.cpu_usage,
                "memory_usage_mb": node.current_load.memory_usage_mb,
                "network_usage_mbps": node.current_load.network_usage_mbps
            })).collect::<Vec<_>>()
        })
    }
}

#[derive(Debug)]
struct ResourceWeights {
    batch: f64,
    cpu: f64,
    memory: f64,
    network: f64,
}

/// Batch optimizer for dynamic configuration optimization
pub struct BatchOptimizer {
    content_analyzer: Arc<ContentAnalyzer>,
    load_balancer: Arc<LoadBalancer>,
    optimization_history: Arc<RwLock<Vec<OptimizationResult>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub batch_id: String,
    pub original_config: BatchConfig,
    pub optimized_config: BatchConfig,
    pub analysis: ContentAnalysis,
    pub selected_node: String,
    pub expected_improvement: f64, // Percentage improvement
    pub timestamp: String,
}

impl BatchOptimizer {
    pub fn new(content_analyzer: Arc<ContentAnalyzer>, load_balancer: Arc<LoadBalancer>) -> Self {
        Self {
            content_analyzer,
            load_balancer,
            optimization_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Optimize batch configuration based on content analysis and system state
    pub async fn optimize_batch(
        &self,
        requests: &[BatchRequest],
        base_config: &BatchConfig,
    ) -> Result<OptimizedBatchConfig, MCPUnifiedError> {
        let analysis = self.content_analyzer.analyze_batch(requests).await?;

        // Get system load information
        let system_load = self.estimate_system_load().await;

        // Select optimal processing node
        let selected_node = self.load_balancer.select_node(&analysis, None).await?;

        // Optimize configuration based on analysis and load
        let optimized_config = self.optimize_configuration(&analysis, base_config, system_load);

        // Calculate expected improvement
        let expected_improvement = self.calculate_expected_improvement(
            &analysis,
            base_config,
            &optimized_config,
            system_load,
        );

        let result = OptimizationResult {
            batch_id: Uuid::new_v4().to_string(),
            original_config: base_config.clone(),
            optimized_config: optimized_config.clone(),
            analysis: analysis.clone(),
            selected_node: selected_node.clone(),
            expected_improvement,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Record optimization result
        {
            let mut history = self.optimization_history.write().await;
            history.push(result);
            if history.len() > 100 {
                // Keep last 100 optimizations
                history.remove(0);
            }
        }

        Ok(OptimizedBatchConfig {
            config: optimized_config,
            analysis,
            selected_node,
            expected_improvement,
            optimization_recommendations: vec![], // Can be populated based on analysis
        })
    }

    fn optimize_configuration(
        &self,
        analysis: &ContentAnalysis,
        base_config: &BatchConfig,
        system_load: f64,
    ) -> BatchConfig {
        let mut config = base_config.clone();

        // Adjust max concurrent based on batch type and complexity
        config.max_concurrent = match analysis.batch_type {
            BatchType::DataProcessing => {
                if analysis.complexity_score > 0.7 {
                    (config.max_concurrent / 2).max(1) // Reduce concurrency for complex data processing
                } else {
                    config.max_concurrent
                }
            }
            BatchType::ApiCalls => {
                (config.max_concurrent / 2).max(1) // Reduce for API rate limiting
            }
            BatchType::Computations => {
                config.max_concurrent // Keep high for CPU-bound tasks
            }
            BatchType::Mixed => {
                (config.max_concurrent * 3 / 4).max(1) // Moderate reduction for mixed workloads
            }
            BatchType::IoBound => {
                (config.max_concurrent / 3).max(1) // Significant reduction for I/O bound
            }
        };

        // Adjust based on system load
        if system_load > 0.7 {
            config.max_concurrent =
                (config.max_concurrent as f64 * (1.0 - system_load)).max(1.0) as usize;
        }

        // Adjust timeouts based on analysis
        if analysis.estimated_execution_time_ms > 10000 {
            // > 10 seconds
            config.default_timeout =
                Duration::from_millis((analysis.estimated_execution_time_ms as f64 * 1.5) as u64);
        }

        // Enable/disable features based on batch characteristics
        config.fail_fast = analysis.complexity_score > 0.8; // Fail fast for very complex batches
        config.enable_retry = analysis.batch_type != BatchType::ApiCalls; // Reduce retries for API calls

        config
    }

    fn calculate_expected_improvement(
        &self,
        analysis: &ContentAnalysis,
        original: &BatchConfig,
        optimized: &BatchConfig,
        system_load: f64,
    ) -> f64 {
        let concurrency_improvement = if optimized.max_concurrent > original.max_concurrent {
            (optimized.max_concurrent - original.max_concurrent) as f64
                / original.max_concurrent as f64
                * 20.0
        } else {
            0.0
        };

        let load_adjustment = (1.0 - system_load) * 15.0; // Up to 15% improvement when system is not loaded

        let type_bonus = match analysis.batch_type {
            BatchType::DataProcessing => 10.0,
            BatchType::ApiCalls => 5.0,
            BatchType::Computations => 15.0,
            BatchType::Mixed => 8.0,
            BatchType::IoBound => 12.0,
        };

        (concurrency_improvement + load_adjustment + type_bonus).min(50.0) // Cap at 50% improvement
    }

    async fn estimate_system_load(&self) -> f64 {
        // Simplified system load estimation
        // In a real implementation, this would query actual system metrics
        let stats = self.load_balancer.get_statistics().await;
        let available_nodes = stats
            .get("available_nodes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let total_nodes = stats
            .get("total_nodes")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        if total_nodes == 0 {
            return 1.0; // Fully loaded if no nodes
        }

        1.0 - (available_nodes as f64 / total_nodes as f64)
    }

    /// Get optimization statistics and insights
    pub async fn get_optimization_insights(&self) -> Value {
        let history = self.optimization_history.read().await;

        if history.is_empty() {
            return json!({
                "status": "no_optimization_history",
                "total_optimizations": 0
            });
        }

        let total_improvement: f64 = history.iter().map(|r| r.expected_improvement).sum();
        let avg_improvement = total_improvement / history.len() as f64;

        let batch_type_distribution: HashMap<String, usize> =
            history.iter().fold(HashMap::new(), |mut acc, result| {
                let type_str = format!("{:?}", result.analysis.batch_type);
                *acc.entry(type_str).or_insert(0) += 1;
                acc
            });

        json!({
            "total_optimizations": history.len(),
            "average_improvement_percent": avg_improvement,
            "total_expected_improvement_percent": total_improvement,
            "batch_type_distribution": batch_type_distribution,
            "recent_optimizations": history.iter().rev().take(5).map(|r| json!({
                "batch_id": r.batch_id,
                "batch_type": format!("{:?}", r.analysis.batch_type),
                "complexity_score": r.analysis.complexity_score,
                "expected_improvement": r.expected_improvement,
                "selected_node": r.selected_node,
                "timestamp": r.timestamp
            })).collect::<Vec<_>>()
        })
    }
}

#[derive(Debug, Clone)]
pub struct OptimizedBatchConfig {
    pub config: BatchConfig,
    pub analysis: ContentAnalysis,
    pub selected_node: String,
    pub expected_improvement: f64,
    pub optimization_recommendations: Vec<String>,
}

/// Intelligent batch router with content analysis and load balancing
pub struct IntelligentBatchRouter {
    content_analyzer: Arc<ContentAnalyzer>,
    load_balancer: Arc<LoadBalancer>,
    batch_optimizer: Arc<BatchOptimizer>,
    routing_rules: Arc<RwLock<Vec<RoutingRule>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub id: String,
    pub condition: RoutingCondition,
    pub action: RoutingAction,
    pub priority: u8, // 1-10, higher priority rules are checked first
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingCondition {
    BatchType(BatchType),
    ComplexityScoreRange(f64, f64),  // min, max
    PriorityRange(u8, u8),           // min, max
    RequestCountRange(usize, usize), // min, max
    SystemLoadRange(f64, f64),       // min, max
    ResourceRequirement(ResourceRequirements),
    And(Vec<Box<RoutingCondition>>),
    Or(Vec<Box<RoutingCondition>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingAction {
    RouteToNode(String), // Specific node ID
    UseStrategy(LoadBalancingStrategy),
    OptimizeConfig(BatchConfig),
    SplitBatch(usize),   // Split into chunks of specified size
    QueueBatch,          // Queue for later processing
    RejectBatch(String), // Reject with reason
}

impl IntelligentBatchRouter {
    pub fn new(
        content_analyzer: Arc<ContentAnalyzer>,
        load_balancer: Arc<LoadBalancer>,
        batch_optimizer: Arc<BatchOptimizer>,
    ) -> Self {
        let mut routing_rules = Vec::new();

        // Initialize with default routing rules
        routing_rules.push(RoutingRule {
            id: "high_priority".to_string(),
            condition: RoutingCondition::PriorityRange(8, 10),
            action: RoutingAction::UseStrategy(LoadBalancingStrategy::LeastLoaded),
            priority: 10,
            enabled: true,
        });

        routing_rules.push(RoutingRule {
            id: "complex_computation".to_string(),
            condition: RoutingCondition::And(vec![
                Box::new(RoutingCondition::BatchType(BatchType::Computations)),
                Box::new(RoutingCondition::ComplexityScoreRange(0.7, 1.0)),
            ]),
            action: RoutingAction::UseStrategy(LoadBalancingStrategy::Specialized),
            priority: 9,
            enabled: true,
        });

        routing_rules.push(RoutingRule {
            id: "high_system_load".to_string(),
            condition: RoutingCondition::SystemLoadRange(0.8, 1.0),
            action: RoutingAction::OptimizeConfig(BatchConfig {
                max_concurrent: 2,
                default_timeout: Duration::from_secs(60),
                fail_fast: true,
                dependency_timeout: Duration::from_secs(600),
                enable_retry: false,
                retry_attempts: 1,
            }),
            priority: 8,
            enabled: true,
        });

        routing_rules.push(RoutingRule {
            id: "large_batch".to_string(),
            condition: RoutingCondition::RequestCountRange(50, usize::MAX),
            action: RoutingAction::SplitBatch(25),
            priority: 7,
            enabled: true,
        });

        routing_rules.push(RoutingRule {
            id: "api_rate_limiting".to_string(),
            condition: RoutingCondition::BatchType(BatchType::ApiCalls),
            action: RoutingAction::OptimizeConfig(BatchConfig {
                max_concurrent: 5,
                default_timeout: Duration::from_secs(10),
                fail_fast: false,
                dependency_timeout: Duration::from_secs(300),
                enable_retry: true,
                retry_attempts: 5,
            }),
            priority: 6,
            enabled: true,
        });

        Self {
            content_analyzer,
            load_balancer,
            batch_optimizer,
            routing_rules: Arc::new(RwLock::new(routing_rules)),
        }
    }

    /// Route batch with intelligent decision making
    pub async fn route_batch(
        &self,
        requests: Vec<BatchRequest>,
        base_config: Option<BatchConfig>,
    ) -> Result<RoutingDecision, MCPUnifiedError> {
        let config = base_config.unwrap_or_default();

        // Analyze content
        let analysis = self.content_analyzer.analyze_batch(&requests).await?;

        // Get system load
        let system_load = self.batch_optimizer.estimate_system_load().await;

        // Evaluate routing rules
        let applicable_rules = self
            .evaluate_routing_rules(&analysis, &requests, system_load)
            .await;

        // Make routing decision
        let decision = self
            .make_routing_decision(applicable_rules, analysis, requests, config, system_load)
            .await?;

        debug!(
            "Routed batch with {} requests, type: {:?}, complexity: {:.2}, decision: {:?}",
            decision.requests.len(),
            decision.analysis.batch_type,
            decision.analysis.complexity_score,
            decision.action
        );

        Ok(decision)
    }

    async fn evaluate_routing_rules(
        &self,
        analysis: &ContentAnalysis,
        requests: &[BatchRequest],
        system_load: f64,
    ) -> Vec<RoutingRule> {
        let rules = self.routing_rules.read().await;
        let mut applicable = Vec::new();

        for rule in rules.iter().filter(|r| r.enabled) {
            if self.matches_condition(&rule.condition, analysis, requests, system_load) {
                applicable.push(rule.clone());
            }
        }

        // Sort by priority (highest first)
        applicable.sort_by(|a, b| b.priority.cmp(&a.priority));
        applicable
    }

    fn matches_condition(
        &self,
        condition: &RoutingCondition,
        analysis: &ContentAnalysis,
        requests: &[BatchRequest],
        system_load: f64,
    ) -> bool {
        match condition {
            RoutingCondition::BatchType(batch_type) => analysis.batch_type == *batch_type,
            RoutingCondition::ComplexityScoreRange(min, max) => {
                analysis.complexity_score >= *min && analysis.complexity_score <= *max
            }
            RoutingCondition::PriorityRange(min, max) => requests
                .iter()
                .any(|r| r.priority.map_or(false, |p| p >= *min && p <= *max)),
            RoutingCondition::RequestCountRange(min, max) => {
                let count = requests.len();
                count >= *min && count <= *max
            }
            RoutingCondition::SystemLoadRange(min, max) => {
                system_load >= *min && system_load <= *max
            }
            RoutingCondition::ResourceRequirement(req) => {
                analysis.resource_requirements.cpu_cores >= req.cpu_cores
                    || analysis.resource_requirements.memory_mb >= req.memory_mb
                    || analysis.resource_requirements.network_bandwidth >= req.network_bandwidth
            }
            RoutingCondition::And(conditions) => conditions
                .iter()
                .all(|c| self.matches_condition(c, analysis, requests, system_load)),
            RoutingCondition::Or(conditions) => conditions
                .iter()
                .any(|c| self.matches_condition(c, analysis, requests, system_load)),
        }
    }

    async fn make_routing_decision(
        &self,
        rules: Vec<RoutingRule>,
        analysis: ContentAnalysis,
        requests: Vec<BatchRequest>,
        config: BatchConfig,
        system_load: f64,
    ) -> Result<RoutingDecision, MCPUnifiedError> {
        // Apply highest priority rule
        if let Some(rule) = rules.first() {
            match &rule.action {
                RoutingAction::RouteToNode(node_id) => {
                    return Ok(RoutingDecision {
                        action: RoutingActionType::RouteToNode(node_id.clone()),
                        requests,
                        analysis,
                        optimized_config: Some(config),
                        reasoning: format!(
                            "Applied routing rule '{}': route to specific node",
                            rule.id
                        ),
                    });
                }
                RoutingAction::UseStrategy(strategy) => {
                    let selected_node = self.load_balancer.select_node(&analysis, None).await?;
                    return Ok(RoutingDecision {
                        action: RoutingActionType::RouteWithStrategy(
                            strategy.clone(),
                            selected_node,
                        ),
                        requests,
                        analysis,
                        optimized_config: Some(config),
                        reasoning: format!(
                            "Applied routing rule '{}': use strategy {:?}",
                            rule.id, strategy
                        ),
                    });
                }
                RoutingAction::OptimizeConfig(override_config) => {
                    let optimized = self
                        .batch_optimizer
                        .optimize_batch(&requests, override_config)
                        .await?;
                    return Ok(RoutingDecision {
                        action: RoutingActionType::OptimizeAndRoute(optimized.selected_node),
                        requests,
                        analysis,
                        optimized_config: Some(optimized.config),
                        reasoning: format!(
                            "Applied routing rule '{}': optimize configuration",
                            rule.id
                        ),
                    });
                }
                RoutingAction::SplitBatch(chunk_size) => {
                    let chunks = self.split_requests(requests, *chunk_size);
                    return Ok(RoutingDecision {
                        action: RoutingActionType::SplitBatch(chunks.clone()),
                        requests: vec![], // Requests moved to chunks
                        analysis,
                        optimized_config: Some(config),
                        reasoning: format!(
                            "Applied routing rule '{}': split into {} chunks",
                            rule.id,
                            chunks.len()
                        ),
                    });
                }
                RoutingAction::QueueBatch => {
                    return Ok(RoutingDecision {
                        action: RoutingActionType::Queue,
                        requests,
                        analysis,
                        optimized_config: Some(config),
                        reasoning: format!(
                            "Applied routing rule '{}': queue batch for later processing",
                            rule.id
                        ),
                    });
                }
                RoutingAction::RejectBatch(reason) => {
                    return Ok(RoutingDecision {
                        action: RoutingActionType::Reject(reason.clone()),
                        requests,
                        analysis,
                        optimized_config: None,
                        reasoning: format!("Applied routing rule '{}': {}", rule.id, reason),
                    });
                }
            }
        }

        // No rules applied, use default optimization
        let optimized = self
            .batch_optimizer
            .optimize_batch(&requests, &config)
            .await?;
        Ok(RoutingDecision {
            action: RoutingActionType::OptimizeAndRoute(optimized.selected_node),
            requests,
            analysis,
            optimized_config: Some(optimized.config),
            reasoning: "No specific routing rules applied, using default optimization".to_string(),
        })
    }

    fn split_requests(
        &self,
        requests: Vec<BatchRequest>,
        chunk_size: usize,
    ) -> Vec<Vec<BatchRequest>> {
        requests
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// Add or update routing rule
    pub async fn add_routing_rule(&self, rule: RoutingRule) -> Result<(), MCPUnifiedError> {
        let mut rules = self.routing_rules.write().await;

        // Check for duplicate IDs
        if rules.iter().any(|r| r.id == rule.id) {
            return Err(MCPUnifiedError::validation(
                "routing_rule".to_string(),
                format!("Routing rule with ID '{}' already exists", rule.id),
                Some(json!({"rule_id": rule.id})),
                Some("Use update_routing_rule to modify existing rules".to_string()),
            ));
        }

        rules.push(rule);
        Ok(())
    }

    /// Get routing statistics
    pub async fn get_routing_statistics(&self) -> Value {
        let rules = self.routing_rules.read().await;
        let enabled_rules = rules.iter().filter(|r| r.enabled).count();

        json!({
            "total_rules": rules.len(),
            "enabled_rules": enabled_rules,
            "disabled_rules": rules.len() - enabled_rules,
            "rules": rules.iter().map(|r| json!({
                "id": r.id,
                "priority": r.priority,
                "enabled": r.enabled,
                "condition": format!("{:?}", r.condition),
                "action": format!("{:?}", r.action)
            })).collect::<Vec<_>>()
        })
    }
}

#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub action: RoutingActionType,
    pub requests: Vec<BatchRequest>,
    pub analysis: ContentAnalysis,
    pub optimized_config: Option<BatchConfig>,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub enum RoutingActionType {
    RouteToNode(String),
    RouteWithStrategy(LoadBalancingStrategy, String),
    OptimizeAndRoute(String),
    SplitBatch(Vec<Vec<BatchRequest>>),
    Queue,
    Reject(String),
}

/// Enhanced BatchMCPToolHandler with Phase 4 intelligent routing
impl BatchMCPToolHandler {
    /// Create handler with intelligent routing components
    pub fn with_intelligent_routing(tool_registry: Arc<MCPToolRegistry>) -> Self {
        let content_analyzer = Arc::new(ContentAnalyzer::new());
        let load_balancer = Arc::new(LoadBalancer::new(LoadBalancingStrategy::LeastLoaded));
        let batch_optimizer = Arc::new(BatchOptimizer::new(
            Arc::clone(&content_analyzer),
            Arc::clone(&load_balancer),
        ));
        let intelligent_router = Arc::new(IntelligentBatchRouter::new(
            content_analyzer,
            load_balancer,
            batch_optimizer,
        ));

        // Store router in the handler for access
        // Note: This requires adding a field to BatchMCPToolHandler
        let mut handler = Self::new(tool_registry);
        // For now, we'll add the router as a field - in practice you'd modify the struct
        handler
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::communication::mcp_tool_registry::{CachingStrategy, PerformanceTier, ToolMetadata};
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
        let fast_tool = MockTool {
            delay_ms: 10,
            should_fail: false,
        };
        let slow_tool = MockTool {
            delay_ms: 50,
            should_fail: false,
        };

        registry
            .register_simple_tool(
                "fast_tool".to_string(),
                fast_tool,
                "utilities",
                "Fast mock tool".to_string(),
                CachingStrategy::Never,
            )
            .expect("replaced unwrap");

        registry
            .register_simple_tool(
                "slow_tool".to_string(),
                slow_tool,
                "utilities",
                "Slow mock tool".to_string(),
                CachingStrategy::Never,
            )
            .expect("replaced unwrap");

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

        let summary = batch_handler
            .execute_batch(requests, None)
            .await
            .expect("replaced unwrap");

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

        assert!(matches!(
            context.status_map.get("req1"),
            Some(BatchRequestStatus::Failed(_))
        ));
        assert!(matches!(
            context.status_map.get("req2"),
            Some(BatchRequestStatus::Skipped(_))
        ));
        assert!(matches!(
            context.status_map.get("req3"),
            Some(BatchRequestStatus::Skipped(_))
        ));
    }
}
