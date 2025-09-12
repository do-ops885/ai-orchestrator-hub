//! Enhanced Async Operation Optimizer
//!
//! This module provides comprehensive async operation optimization including
//! batching, concurrency control, and performance monitoring to improve
//! overall system throughput and responsiveness.
//!
//! This module provides optimization for async operations including task batching,
//! connection pooling, and intelligent scheduling to improve throughput and reduce latency.

use crate::utils::error::{HiveError, HiveResult};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::time::interval;
use tracing::{debug, info};

/// Configuration for async operation optimization
#[derive(Debug, Clone)]
pub struct AsyncOptimizerConfig {
    /// Maximum number of concurrent operations
    pub max_concurrent_ops: usize,
    /// Batch size for batched operations
    pub batch_size: usize,
    /// Maximum wait time before processing a partial batch
    pub batch_timeout: Duration,
    /// Connection pool size for external services
    pub connection_pool_size: usize,
    /// Enable operation prioritization
    pub enable_prioritization: bool,
    /// Metrics collection interval
    pub metrics_interval: Duration,
}

impl Default for AsyncOptimizerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_ops: num_cpus::get() * 4,
            batch_size: 100,
            batch_timeout: Duration::from_millis(50),
            connection_pool_size: 20,
            enable_prioritization: true,
            metrics_interval: Duration::from_secs(30),
        }
    }
}

/// Priority levels for async operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperationPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Async operation wrapper with metadata
pub struct AsyncOperation<T> {
    pub id: uuid::Uuid,
    pub priority: OperationPriority,
    pub created_at: Instant,
    pub operation: Box<
        dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = HiveResult<T>> + Send>>
            + Send,
    >,
    pub timeout: Option<Duration>,
}

impl<T> AsyncOperation<T> {
    /// Create a new async operation
    pub fn new<F, Fut>(operation: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = HiveResult<T>> + Send + 'static,
    {
        Self {
            id: uuid::Uuid::new_v4(),
            priority: OperationPriority::Normal,
            created_at: Instant::now(),
            operation: Box::new(move || Box::pin(operation())),
            timeout: None,
        }
    }

    /// Set operation priority
    pub fn with_priority(mut self, priority: OperationPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set operation timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Batch processor for grouping similar operations
pub struct BatchProcessor<T> {
    pending_operations: Arc<Mutex<VecDeque<AsyncOperation<T>>>>,
    config: AsyncOptimizerConfig,
    metrics: Arc<RwLock<BatchMetrics>>,
}

#[derive(Debug, Default, Clone)]
struct BatchMetrics {
    total_operations: u64,
    batched_operations: u64,
    average_batch_size: f64,
    processing_time_ms: u64,
}

impl<T> BatchProcessor<T>
where
    T: Send + 'static,
{
    /// Create a new batch processor
    pub fn new(config: AsyncOptimizerConfig) -> Self {
        Self {
            pending_operations: Arc::new(Mutex::new(VecDeque::new())),
            config,
            metrics: Arc::new(RwLock::new(BatchMetrics::default())),
        }
    }

    /// Add operation to batch queue
    pub async fn add_operation(&self, operation: AsyncOperation<T>) -> HiveResult<()> {
        let mut queue = self.pending_operations.lock().await;

        // Insert based on priority if prioritization is enabled
        if self.config.enable_prioritization {
            let insert_pos = queue
                .iter()
                .position(|op| op.priority < operation.priority)
                .unwrap_or(queue.len());
            queue.insert(insert_pos, operation);
        } else {
            queue.push_back(operation);
        }

        debug!(
            "Added operation to batch queue, current size: {}",
            queue.len()
        );
        Ok(())
    }

    /// Process batched operations
    pub async fn process_batch(&self) -> HiveResult<Vec<HiveResult<T>>> {
        let start_time = Instant::now();
        let operations = {
            let mut queue = self.pending_operations.lock().await;
            let batch_size = std::cmp::min(self.config.batch_size, queue.len());
            queue.drain(..batch_size).collect::<Vec<_>>()
        };

        if operations.is_empty() {
            return Ok(Vec::new());
        }

        info!("Processing batch of {} operations", operations.len());

        // Execute operations concurrently with semaphore for concurrency control
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_ops));
        let mut handles = Vec::new();

        for operation in operations {
            let semaphore_clone = Arc::clone(&semaphore);
            let handle =
                tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.map_err(|e| {
                        HiveError::OperationFailed {
                            reason: format!("Failed to acquire semaphore: {}", e),
                        }
                    })?;

                    let operation_future = (operation.operation)();

                    if let Some(timeout) = operation.timeout {
                        match tokio::time::timeout(timeout, operation_future).await {
                            Ok(result) => result,
                            Err(_) => Err(HiveError::TimeoutError {
                                operation: "batch_operation".to_string(),
                                duration_ms: timeout.as_millis() as u64,
                            }),
                        }
                    } else {
                        operation_future.await
                    }
                });
            handles.push(handle);
        }

        // Wait for all operations to complete
        let results = futures::future::join_all(handles).await;
        let final_results: Vec<HiveResult<T>> = results
            .into_iter()
            .map(|handle_result| {
                handle_result
                    .map_err(|e| HiveError::OperationFailed {
                        reason: format!("Task join error: {}", e),
                    })
                    .and_then(|inner| inner)
            })
            .collect();

        // Update metrics
        let processing_time = start_time.elapsed();
        let mut metrics = self.metrics.write().await;
        metrics.total_operations += final_results.len() as u64;
        metrics.batched_operations += 1;
        metrics.average_batch_size = (metrics.average_batch_size
            * (metrics.batched_operations - 1) as f64
            + final_results.len() as f64)
            / metrics.batched_operations as f64;
        metrics.processing_time_ms += processing_time.as_millis() as u64;

        debug!("Batch processing completed in {:?}", processing_time);
        Ok(final_results)
    }

    /// Get batch processing metrics
    pub async fn get_metrics(&self) -> BatchMetrics {
        (*self.metrics.read().await).clone()
    }
}

/// Async operation optimizer with intelligent scheduling
pub struct AsyncOptimizer {
    config: AsyncOptimizerConfig,
    semaphore: Arc<Semaphore>,
    metrics: Arc<RwLock<OptimizerMetrics>>,
}

#[derive(Debug, Default, Clone)]
pub struct OptimizerMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub average_execution_time_ms: f64,
    pub current_concurrent_ops: usize,
    pub peak_concurrent_ops: usize,
    pub queue_depth: usize,
}

impl AsyncOptimizer {
    /// Create a new async optimizer
    pub fn new(config: AsyncOptimizerConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_ops));

        Self {
            config,
            semaphore,
            metrics: Arc::new(RwLock::new(OptimizerMetrics::default())),
        }
    }

    /// Execute an async operation with optimization
    pub async fn execute<T, F, Fut>(&self, operation: F) -> HiveResult<T>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = HiveResult<T>> + Send,
        T: Send,
    {
        let start_time = Instant::now();

        // Acquire semaphore permit for concurrency control
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to acquire execution permit: {}", e),
            })?;

        // Update concurrent operations count
        {
            let mut metrics = self.metrics.write().await;
            metrics.current_concurrent_ops =
                self.config.max_concurrent_ops - self.semaphore.available_permits();
            metrics.peak_concurrent_ops =
                std::cmp::max(metrics.peak_concurrent_ops, metrics.current_concurrent_ops);
        }

        // Execute the operation
        let result = operation().await;
        let execution_time = start_time.elapsed();

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_operations += 1;

            match &result {
                Ok(_) => metrics.successful_operations += 1,
                Err(_) => metrics.failed_operations += 1,
            }

            // Update average execution time
            let total_ops = metrics.total_operations as f64;
            metrics.average_execution_time_ms = (metrics.average_execution_time_ms
                * (total_ops - 1.0)
                + execution_time.as_millis() as f64)
                / total_ops;

            metrics.current_concurrent_ops =
                self.config.max_concurrent_ops - self.semaphore.available_permits();
        }

        debug!("Operation completed in {:?}", execution_time);
        result
    }

    /// Execute multiple operations concurrently with intelligent batching
    pub async fn execute_batch<T, F, Fut>(
        &self,
        operations: Vec<F>,
    ) -> HiveResult<Vec<HiveResult<T>>>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = HiveResult<T>> + Send + 'static,
        T: Send + 'static,
    {
        let batch_processor = BatchProcessor::new(self.config.clone());

        // Add all operations to the batch
        for operation in operations {
            let async_op = AsyncOperation::new(operation);
            batch_processor.add_operation(async_op).await?;
        }

        // Process the batch
        batch_processor.process_batch().await
    }

    /// Get optimizer metrics
    pub async fn get_metrics(&self) -> OptimizerMetrics {
        self.metrics.read().await.clone()
    }

    /// Start background metrics collection
    pub fn start_metrics_collection(&self) -> tokio::task::JoinHandle<()> {
        let metrics = Arc::clone(&self.metrics);
        let interval_duration = self.config.metrics_interval;

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);

            loop {
                interval.tick().await;

                let metrics_snapshot = {
                    let metrics_guard = metrics.read().await;
                    metrics_guard.clone()
                };

                info!(
                    "Async Optimizer Metrics - Total: {}, Success: {}, Failed: {}, Avg Time: {:.2}ms, Concurrent: {}",
                    metrics_snapshot.total_operations,
                    metrics_snapshot.successful_operations,
                    metrics_snapshot.failed_operations,
                    metrics_snapshot.average_execution_time_ms,
                    metrics_snapshot.current_concurrent_ops
                );
            }
        })
    }
}

/// Connection pool optimizer for external services
pub struct ConnectionPoolOptimizer {
    pool_size: usize,
    active_connections: Arc<Mutex<usize>>,
    connection_metrics: Arc<RwLock<ConnectionMetrics>>,
}

#[derive(Debug, Default, Clone)]
pub struct ConnectionMetrics {
    pub total_connections_created: u64,
    pub active_connections: usize,
    pub peak_connections: usize,
    pub connection_errors: u64,
    pub average_connection_time_ms: f64,
}

impl ConnectionPoolOptimizer {
    /// Create a new connection pool optimizer
    pub fn new(pool_size: usize) -> Self {
        Self {
            pool_size,
            active_connections: Arc::new(Mutex::new(0)),
            connection_metrics: Arc::new(RwLock::new(ConnectionMetrics::default())),
        }
    }

    /// Acquire a connection with optimization
    pub async fn acquire_connection<F, Fut, T>(&self, connection_factory: F) -> HiveResult<T>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = HiveResult<T>> + Send,
        T: Send,
    {
        let start_time = Instant::now();

        // Check if we can create a new connection
        let can_create = {
            let mut active = self.active_connections.lock().await;
            if *active < self.pool_size {
                *active += 1;
                true
            } else {
                false
            }
        };

        if !can_create {
            return Err(HiveError::ResourceExhausted {
                resource: "connection_pool".to_string(),
            });
        }

        // Create connection
        let result = connection_factory().await;
        let connection_time = start_time.elapsed();

        // Update metrics
        {
            let mut metrics = self.connection_metrics.write().await;
            metrics.total_connections_created += 1;

            let active_count = *self.active_connections.lock().await;
            metrics.active_connections = active_count;
            metrics.peak_connections = std::cmp::max(metrics.peak_connections, active_count);

            match &result {
                Ok(_) => {
                    let total_connections = metrics.total_connections_created as f64;
                    metrics.average_connection_time_ms = (metrics.average_connection_time_ms
                        * (total_connections - 1.0)
                        + connection_time.as_millis() as f64)
                        / total_connections;
                }
                Err(_) => {
                    metrics.connection_errors += 1;
                    // Release the connection slot on error
                    let mut active = self.active_connections.lock().await;
                    *active = active.saturating_sub(1);
                }
            }
        }

        result
    }

    /// Release a connection
    pub async fn release_connection(&self) {
        let mut active = self.active_connections.lock().await;
        *active = active.saturating_sub(1);

        let mut metrics = self.connection_metrics.write().await;
        metrics.active_connections = *active;
    }

    /// Get connection pool metrics
    pub async fn get_metrics(&self) -> ConnectionMetrics {
        self.connection_metrics.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_async_optimizer() -> Result<(), Box<dyn std::error::Error>> {
        let config = AsyncOptimizerConfig {
            max_concurrent_ops: 2,
            ..Default::default()
        };
        let optimizer = AsyncOptimizer::new(config);

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = optimizer
            .execute(move || async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Ok::<u32, HiveError>(42)
            })
            .await?;

        assert_eq!(result, 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        let metrics = optimizer.get_metrics().await;
        assert_eq!(metrics.total_operations, 1);
        assert_eq!(metrics.successful_operations, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_batch_processor() -> Result<(), Box<dyn std::error::Error>> {
        let config = AsyncOptimizerConfig {
            batch_size: 3,
            max_concurrent_ops: 2,
            ..Default::default()
        };
        let processor = BatchProcessor::new(config);

        // Add operations with different priorities
        for i in 0..5 {
            let priority = if i % 2 == 0 {
                OperationPriority::High
            } else {
                OperationPriority::Normal
            };
            let operation = AsyncOperation::new(move || async move { Ok::<u32, HiveError>(i) })
                .with_priority(priority);

            processor.add_operation(operation).await?;
        }

        let results = processor.process_batch().await?;
        assert_eq!(results.len(), 3); // Batch size limit

        let metrics = processor.get_metrics().await;
        assert_eq!(metrics.batched_operations, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_connection_pool_optimizer() -> Result<(), Box<dyn std::error::Error>> {
        let pool = ConnectionPoolOptimizer::new(2);

        let result1 = pool
            .acquire_connection(|| async { Ok::<String, HiveError>("connection1".to_string()) })
            .await?;

        let result2 = pool
            .acquire_connection(|| async { Ok::<String, HiveError>("connection2".to_string()) })
            .await?;

        assert_eq!(result1, "connection1");
        assert_eq!(result2, "connection2");

        // Third connection should fail (pool exhausted)
        let result3 = pool
            .acquire_connection(|| async { Ok::<String, HiveError>("connection3".to_string()) })
            .await;

        assert!(result3.is_err());

        // Release a connection and try again
        pool.release_connection().await;
        let result4 = pool
            .acquire_connection(|| async { Ok::<String, HiveError>("connection4".to_string()) })
            .await?;

        assert_eq!(result4, "connection4");

        let metrics = pool.get_metrics().await;
        assert_eq!(metrics.total_connections_created, 3);

        Ok(())
    }
}
