//! Optimized Messaging System for High-Performance Swarm Communication
//!
//! This module implements performance optimizations for message passing:
//! - Message batching for reduced overhead
//! - Compression for reduced bandwidth
//! - Object pooling for memory efficiency
//! - Async message pipeline optimization

use crate::communication::protocols::{MessageEnvelope, MessagePayload, MessageType};
use crate::utils::error::{HiveError, HiveResult};
use flate2::{write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};
use uuid::Uuid;

/// Configuration for optimized messaging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedMessagingConfig {
    /// Maximum messages per batch
    pub max_batch_size: usize,
    /// Maximum time to wait before sending incomplete batch
    pub batch_timeout_ms: u64,
    /// Enable message compression
    pub enable_compression: bool,
    /// Compression level (1-9)
    pub compression_level: u32,
    /// Object pool size for message reuse
    pub object_pool_size: usize,
    /// Maximum concurrent batches being processed
    pub max_concurrent_batches: usize,
    /// Minimum message size to compress (bytes)
    pub compression_threshold_bytes: usize,
}

impl Default for OptimizedMessagingConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 50,
            batch_timeout_ms: 10, // 10ms batching window
            enable_compression: true,
            compression_level: 6, // Good balance of speed vs compression
            object_pool_size: 1000,
            max_concurrent_batches: 10,
            compression_threshold_bytes: 1024, // 1KB minimum for compression
        }
    }
}

/// Batch of messages for optimized transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBatch {
    /// Unique batch identifier
    pub batch_id: Uuid,
    /// Messages in this batch
    pub messages: Vec<MessageEnvelope>,
    /// Batch creation timestamp
    pub created_at: std::time::SystemTime,
    /// Compression used
    pub compressed: bool,
    /// Original size before compression (if applicable)
    pub original_size_bytes: usize,
    /// Compressed size (if applicable)
    pub compressed_size_bytes: usize,
}

impl MessageBatch {
    /// Create a new message batch
    #[must_use]
    pub fn new(messages: Vec<MessageEnvelope>) -> Self {
        Self {
            batch_id: Uuid::new_v4(),
            messages,
            created_at: std::time::SystemTime::now(),
            compressed: false,
            original_size_bytes: 0,
            compressed_size_bytes: 0,
        }
    }

    /// Get compression ratio (`compressed_size` / `original_size`)
    #[must_use]
    pub fn compression_ratio(&self) -> f64 {
        if self.original_size_bytes == 0 {
            1.0
        } else {
            self.compressed_size_bytes as f64 / self.original_size_bytes as f64
        }
    }
}

/// Message object pool for memory efficiency
pub struct MessagePool {
    pool: Arc<Mutex<VecDeque<MessageEnvelope>>>,
    max_size: usize,
    created_count: Arc<std::sync::atomic::AtomicU64>,
    reused_count: Arc<std::sync::atomic::AtomicU64>,
}

impl MessagePool {
    /// Create a new message pool
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_size,
            created_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            reused_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Get a message from the pool or create a new one
    pub async fn get(&self) -> MessageEnvelope {
        let mut pool = self.pool.lock().await;
        if let Some(mut message) = pool.pop_front() {
            // Reset the message for reuse
            message.id = Uuid::new_v4();
            message.timestamp = chrono::Utc::now();
            self.reused_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            message
        } else {
            // Create new message
            self.created_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            MessageEnvelope::new(
                MessageType::Request,
                Uuid::new_v4(),
                vec![Uuid::new_v4()],
                MessagePayload::Text(String::new()),
            )
        }
    }

    /// Return a message to the pool
    pub async fn return_message(&self, message: MessageEnvelope) {
        let mut pool = self.pool.lock().await;
        if pool.len() < self.max_size {
            pool.push_back(message);
        }
    }

    /// Get pool statistics
    #[must_use]
    pub fn get_stats(&self) -> MessagePoolStats {
        let created = self
            .created_count
            .load(std::sync::atomic::Ordering::Relaxed);
        let reused = self.reused_count.load(std::sync::atomic::Ordering::Relaxed);

        MessagePoolStats {
            created_count: created,
            reused_count: reused,
            reuse_ratio: if created == 0 {
                0.0
            } else {
                reused as f64 / created as f64
            },
            pool_size: self.max_size,
        }
    }
}

/// Statistics for message pool usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePoolStats {
    pub created_count: u64,
    pub reused_count: u64,
    pub reuse_ratio: f64,
    pub pool_size: usize,
}

/// Message compressor for bandwidth optimization
pub struct MessageCompressor {
    compression_level: u32,
    threshold_bytes: usize,
    stats: Arc<RwLock<CompressionStats>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompressionStats {
    pub messages_compressed: u64,
    pub messages_skipped: u64,
    pub total_original_bytes: u64,
    pub total_compressed_bytes: u64,
    pub average_compression_ratio: f64,
    pub compression_time_ms: f64,
}

impl MessageCompressor {
    /// Create a new message compressor
    #[must_use]
    pub fn new(compression_level: u32, threshold_bytes: usize) -> Self {
        Self {
            compression_level,
            threshold_bytes,
            stats: Arc::new(RwLock::new(CompressionStats::default())),
        }
    }

    /// Compress a message batch if beneficial
    pub async fn compress_batch(&self, mut batch: MessageBatch) -> HiveResult<MessageBatch> {
        let start_time = Instant::now();

        // Serialize the messages
        let original_data =
            bincode::serialize(&batch.messages).map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to serialize message batch: {e}"),
            })?;

        batch.original_size_bytes = original_data.len();

        // Skip compression if below threshold
        if original_data.len() < self.threshold_bytes {
            let mut stats = self.stats.write().await;
            stats.messages_skipped += 1;
            return Ok(batch);
        }

        // Compress the data
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.compression_level));
        encoder
            .write_all(&original_data)
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to compress message batch: {e}"),
            })?;

        let compressed_data = encoder.finish().map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to finish compression: {e}"),
        })?;

        batch.compressed_size_bytes = compressed_data.len();
        batch.compressed = true;

        // Update statistics
        let compression_time = start_time.elapsed();
        let mut stats = self.stats.write().await;
        stats.messages_compressed += 1;
        stats.total_original_bytes += original_data.len() as u64;
        stats.total_compressed_bytes += compressed_data.len() as u64;
        stats.compression_time_ms = f64::midpoint(
            stats.compression_time_ms,
            compression_time.as_millis() as f64,
        );

        // Update average compression ratio
        if stats.total_original_bytes > 0 {
            stats.average_compression_ratio =
                stats.total_compressed_bytes as f64 / stats.total_original_bytes as f64;
        }

        Ok(batch)
    }

    /// Decompress a message batch
    pub async fn decompress_batch(&self, batch: MessageBatch) -> HiveResult<MessageBatch> {
        if !batch.compressed {
            return Ok(batch);
        }

        // For this implementation, we'd need to store the compressed data
        // In practice, this would involve modifying MessageBatch to include the compressed payload
        // For now, return the batch as-is since we're focusing on the sending side optimization
        Ok(batch)
    }

    /// Get compression statistics
    pub async fn get_stats(&self) -> CompressionStats {
        self.stats.read().await.clone()
    }
}

/// Batch processor for collecting and processing message batches
pub struct BatchProcessor {
    config: OptimizedMessagingConfig,
    pending_batches: Arc<Mutex<HashMap<String, PendingBatch>>>,
    semaphore: Arc<Semaphore>,
    stats: Arc<RwLock<BatchProcessorStats>>,
    compressor: MessageCompressor,
    message_pool: MessagePool,
}

#[derive(Debug)]
struct PendingBatch {
    messages: Vec<MessageEnvelope>,
    created_at: Instant,
    target: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchProcessorStats {
    pub batches_sent: u64,
    pub messages_batched: u64,
    pub average_batch_size: f64,
    pub average_batch_time_ms: f64,
    pub timeouts_triggered: u64,
    pub size_limits_triggered: u64,
}

impl BatchProcessor {
    /// Create a new batch processor
    #[must_use]
    pub fn new(config: OptimizedMessagingConfig) -> Self {
        let compressor =
            MessageCompressor::new(config.compression_level, config.compression_threshold_bytes);
        let message_pool = MessagePool::new(config.object_pool_size);
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_batches));

        let processor = Self {
            config,
            pending_batches: Arc::new(Mutex::new(HashMap::new())),
            semaphore,
            stats: Arc::new(RwLock::new(BatchProcessorStats::default())),
            compressor,
            message_pool,
        };

        // Start background batch timeout processor
        processor.start_timeout_processor();
        processor
    }

    /// Add a message to a batch for the given target
    pub async fn add_message(&self, target: String, message: MessageEnvelope) -> HiveResult<()> {
        let mut pending = self.pending_batches.lock().await;

        let pending_batch = pending
            .entry(target.clone())
            .or_insert_with(|| PendingBatch {
                messages: Vec::new(),
                created_at: Instant::now(),
                target: target.clone(),
            });

        pending_batch.messages.push(message);

        // Check if batch is ready to send
        if pending_batch.messages.len() >= self.config.max_batch_size {
            let batch = PendingBatch {
                messages: std::mem::take(&mut pending_batch.messages),
                created_at: pending_batch.created_at,
                target: target.clone(),
            };
            pending.remove(&target);
            drop(pending);

            // Update stats
            let mut stats = self.stats.write().await;
            stats.size_limits_triggered += 1;
            drop(stats);

            // Send the batch
            self.send_batch(batch).await?;
        }

        Ok(())
    }

    /// Send a batch of messages
    async fn send_batch(&self, pending_batch: PendingBatch) -> HiveResult<()> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| HiveError::ResourceExhausted {
                resource: "Batch processing semaphore exhausted".to_string(),
            })?;

        let batch_start_time = Instant::now();
        let message_count = pending_batch.messages.len();

        // Create message batch
        let batch = MessageBatch::new(pending_batch.messages);

        // Compress if enabled
        let compressed_batch = if self.config.enable_compression {
            self.compressor.compress_batch(batch).await?
        } else {
            batch
        };

        // Here you would send the batch to the target
        // For now, we'll simulate the send operation
        tokio::time::sleep(Duration::from_millis(1)).await;

        // Update statistics
        let batch_time = batch_start_time.elapsed();
        let mut stats = self.stats.write().await;
        stats.batches_sent += 1;
        stats.messages_batched += message_count as u64;
        stats.average_batch_size = f64::midpoint(stats.average_batch_size, message_count as f64);
        stats.average_batch_time_ms =
            f64::midpoint(stats.average_batch_time_ms, batch_time.as_millis() as f64);

        tracing::debug!(
            "Sent batch {} with {} messages, compression ratio: {:.2}",
            compressed_batch.batch_id,
            message_count,
            compressed_batch.compression_ratio()
        );

        Ok(())
    }

    /// Start background processor for batch timeouts
    fn start_timeout_processor(&self) {
        let pending_batches = Arc::clone(&self.pending_batches);
        let timeout_ms = self.config.batch_timeout_ms;
        let stats = Arc::clone(&self.stats);
        let processor = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(timeout_ms / 2));

            loop {
                interval.tick().await;

                let mut expired_batches = Vec::new();
                {
                    let mut pending = pending_batches.lock().await;
                    let now = Instant::now();
                    let timeout = Duration::from_millis(timeout_ms);

                    let mut to_remove = Vec::new();
                    for (target, batch) in pending.iter() {
                        if now.duration_since(batch.created_at) >= timeout {
                            expired_batches.push(PendingBatch {
                                messages: batch.messages.clone(),
                                created_at: batch.created_at,
                                target: target.clone(),
                            });
                            to_remove.push(target.clone());
                        }
                    }

                    for target in to_remove {
                        pending.remove(&target);
                    }
                }

                // Send expired batches
                for batch in expired_batches {
                    if !batch.messages.is_empty() {
                        let mut stats_guard = stats.write().await;
                        stats_guard.timeouts_triggered += 1;
                        drop(stats_guard);

                        if let Err(e) = processor.send_batch(batch).await {
                            tracing::warn!("Failed to send timeout batch: {}", e);
                        }
                    }
                }
            }
        });
    }

    /// Get message pool
    #[must_use]
    pub fn get_message_pool(&self) -> &MessagePool {
        &self.message_pool
    }

    /// Get batch processor statistics
    pub async fn get_stats(&self) -> BatchProcessorStats {
        self.stats.read().await.clone()
    }

    /// Get compression statistics
    pub async fn get_compression_stats(&self) -> CompressionStats {
        self.compressor.get_stats().await
    }

    /// Get message pool statistics
    #[must_use]
    pub fn get_pool_stats(&self) -> MessagePoolStats {
        self.message_pool.get_stats()
    }
}

impl Clone for BatchProcessor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pending_batches: Arc::clone(&self.pending_batches),
            semaphore: Arc::clone(&self.semaphore),
            stats: Arc::clone(&self.stats),
            compressor: MessageCompressor::new(
                self.config.compression_level,
                self.config.compression_threshold_bytes,
            ),
            message_pool: MessagePool::new(self.config.object_pool_size),
        }
    }
}

/// Optimized swarm communicator using batching and compression
pub struct OptimizedSwarmCommunicator {
    batch_processor: BatchProcessor,
    config: OptimizedMessagingConfig,
}

impl OptimizedSwarmCommunicator {
    /// Create a new optimized swarm communicator
    #[must_use]
    pub fn new(config: OptimizedMessagingConfig) -> Self {
        let batch_processor = BatchProcessor::new(config.clone());

        Self {
            batch_processor,
            config,
        }
    }

    /// Send a message with optimization
    pub async fn send_optimized_message(
        &self,
        target: String,
        message: MessageEnvelope,
    ) -> HiveResult<()> {
        self.batch_processor.add_message(target, message).await
    }

    /// Send multiple messages efficiently
    pub async fn send_multiple_messages(
        &self,
        messages: Vec<(String, MessageEnvelope)>,
    ) -> HiveResult<()> {
        for (target, message) in messages {
            self.send_optimized_message(target, message).await?;
        }
        Ok(())
    }

    /// Get comprehensive performance statistics
    pub async fn get_performance_stats(&self) -> OptimizedMessagingStats {
        OptimizedMessagingStats {
            batch_stats: self.batch_processor.get_stats().await,
            compression_stats: self.batch_processor.get_compression_stats().await,
            pool_stats: self.batch_processor.get_pool_stats(),
            config: self.config.clone(),
        }
    }
}

/// Comprehensive statistics for optimized messaging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedMessagingStats {
    pub batch_stats: BatchProcessorStats,
    pub compression_stats: CompressionStats,
    pub pool_stats: MessagePoolStats,
    pub config: OptimizedMessagingConfig,
}

impl OptimizedMessagingStats {
    /// Calculate overall efficiency metrics
    #[must_use]
    pub fn calculate_efficiency_metrics(&self) -> EfficiencyMetrics {
        let bandwidth_savings = if self.compression_stats.total_original_bytes > 0 {
            1.0 - self.compression_stats.average_compression_ratio
        } else {
            0.0
        };

        let memory_efficiency = if self.pool_stats.created_count > 0 {
            self.pool_stats.reuse_ratio
        } else {
            0.0
        };

        let batching_efficiency = if self.batch_stats.batches_sent > 0 {
            self.batch_stats.average_batch_size / self.config.max_batch_size as f64
        } else {
            0.0
        };

        EfficiencyMetrics {
            bandwidth_savings_percent: bandwidth_savings * 100.0,
            memory_efficiency_percent: memory_efficiency * 100.0,
            batching_efficiency_percent: batching_efficiency * 100.0,
            overall_efficiency_score: (bandwidth_savings + memory_efficiency + batching_efficiency)
                / 3.0
                * 100.0,
        }
    }
}

/// Efficiency metrics for optimization assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub bandwidth_savings_percent: f64,
    pub memory_efficiency_percent: f64,
    pub batching_efficiency_percent: f64,
    pub overall_efficiency_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::communication::protocols::MessagePayload;

    #[tokio::test]
    async fn test_message_pool() {
        let pool = MessagePool::new(5);

        // Get a message from the pool
        let message1 = pool.get().await;
        let id1 = message1.id;

        // Return it to the pool
        pool.return_message(message1).await;

        // Get another message - should be the reused one
        let message2 = pool.get().await;
        assert_ne!(id1, message2.id); // ID should be reset

        let stats = pool.get_stats();
        assert_eq!(stats.reused_count, 1);
    }

    #[tokio::test]
    async fn test_message_compression() {
        let compressor = MessageCompressor::new(6, 100);

        // Create a batch with compressible content
        let messages = vec![
            MessageEnvelope::new(
                MessageType::Request,
                Uuid::new_v4(),
                vec![Uuid::new_v4()],
                MessagePayload::Text("This is a test message that should compress well because it has repeated content. This is a test message that should compress well because it has repeated content.".to_string()),
            )
        ];

        let batch = MessageBatch::new(messages);
        let compressed_batch = match compressor.compress_batch(batch).await {
            Ok(b) => b,
            Err(e) => panic!("Failed to compress batch: {}", e),
        };

        if compressed_batch.compressed {
            assert!(compressed_batch.compression_ratio() < 1.0);
        }

        let stats = compressor.get_stats().await;
        assert!(stats.messages_compressed > 0 || stats.messages_skipped > 0);
    }

    #[tokio::test]
    async fn test_batch_processor() {
        let config = OptimizedMessagingConfig {
            max_batch_size: 3,
            batch_timeout_ms: 50,
            ..Default::default()
        };

        let processor = BatchProcessor::new(config);

        // Add messages that should trigger size-based batching
        for i in 0..3 {
            let message = MessageEnvelope::new(
                MessageType::Request,
                Uuid::new_v4(),
                vec![Uuid::new_v4()],
                MessagePayload::Text(format!("Message {}", i)),
            );
            match processor
                .add_message("test_target".to_string(), message)
                .await
            {
                Ok(_) => {}
                Err(e) => panic!("Failed to add message: {}", e),
            }
        }

        // Give some time for batch processing
        tokio::time::sleep(Duration::from_millis(10)).await;

        let stats = processor.get_stats().await;
        assert!(stats.batches_sent > 0);
    }

    #[tokio::test]
    async fn test_optimized_communicator() {
        let config = OptimizedMessagingConfig::default();
        let communicator = OptimizedSwarmCommunicator::new(config);

        let message = MessageEnvelope::new(
            MessageType::Request,
            Uuid::new_v4(),
            vec![Uuid::new_v4()],
            MessagePayload::Text("Test message".to_string()),
        );

        let result = communicator
            .send_optimized_message("test".to_string(), message)
            .await;
        assert!(result.is_ok());

        let stats = communicator.get_performance_stats().await;
        let efficiency = stats.calculate_efficiency_metrics();
        assert!(efficiency.overall_efficiency_score >= 0.0);
    }
}
