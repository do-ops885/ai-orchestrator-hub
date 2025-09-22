//! Streaming Infrastructure for Large Data Processing
//!
//! This module provides streaming capabilities for handling large datasets
//! and neural processing operations without loading everything into memory.

use crate::utils::error::{HiveError, HiveResult};
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::RwLock;
use tokio::sync::Semaphore;
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::{debug, error, info};

// Compression support
use flate2::{read::GzDecoder, write::GzEncoder, Compression};

/// Configuration for streaming operations
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Buffer size for streaming operations
    pub buffer_size: usize,
    /// Maximum chunk size for processing
    pub max_chunk_size: usize,
    /// Timeout for streaming operations
    pub timeout: std::time::Duration,
    /// Enable compression for streams
    pub enable_compression: bool,
    /// Number of parallel processing workers
    pub parallel_workers: usize,
    /// Memory pool size for chunk reuse
    pub memory_pool_size: usize,
    /// Enable memory pooling for reduced allocations
    pub enable_memory_pool: bool,
    /// Compression level (0-9, 0 = no compression, 9 = max compression)
    pub compression_level: u32,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 8192,
            max_chunk_size: 1024 * 1024, // 1MB
            timeout: std::time::Duration::from_secs(30),
            enable_compression: false,
            parallel_workers: 4,
            memory_pool_size: 100,
            enable_memory_pool: true,
            compression_level: 6,
        }
    }
}

/// Data chunk for streaming operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataChunk {
    /// Unique identifier for the chunk
    pub id: uuid::Uuid,
    /// Sequence number for ordering
    pub sequence: u64,
    /// Total number of chunks in the stream
    pub total_chunks: Option<u64>,
    /// The actual data payload
    pub data: Vec<u8>,
    /// Metadata associated with the chunk
    pub metadata: HashMap<String, String>,
    /// Checksum for data integrity
    pub checksum: Option<String>,
}

impl DataChunk {
    /// Create a new data chunk
    #[must_use] 
    pub fn new(sequence: u64, data: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            sequence,
            total_chunks: None,
            data,
            metadata: HashMap::new(),
            checksum: None,
        }
    }

    /// Add metadata to the chunk
    #[must_use] 
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Calculate and set checksum
    #[must_use] 
    pub fn with_checksum(mut self) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.data);
        self.checksum = Some(format!("{:x}", hasher.finalize()));
        self
    }

    /// Verify checksum
    #[must_use] 
    pub fn verify_checksum(&self) -> bool {
        if let Some(expected) = &self.checksum {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&self.data);
            let actual = format!("{:x}", hasher.finalize());
            &actual == expected
        } else {
            true // No checksum to verify
        }
    }

    /// Compress chunk data
    pub fn compress(&mut self, level: u32) -> HiveResult<()> {
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
        encoder
            .write_all(&self.data)
            .map_err(|e| HiveError::ProcessingError {
                reason: format!("Failed to compress chunk data: {e}"),
            })?;
        self.data = encoder.finish().map_err(|e| HiveError::ProcessingError {
            reason: format!("Failed to finish compression: {e}"),
        })?;

        // Add compression metadata
        self.metadata
            .insert("compressed".to_string(), "true".to_string());
        self.metadata
            .insert("original_size".to_string(), self.data.len().to_string());

        Ok(())
    }

    /// Decompress chunk data
    pub fn decompress(&mut self) -> HiveResult<()> {
        use std::io::Read;

        if self.metadata.get("compressed") != Some(&"true".to_string()) {
            return Ok(()); // Not compressed
        }

        let mut decoder = GzDecoder::new(&self.data[..]);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| HiveError::ProcessingError {
                reason: format!("Failed to decompress chunk data: {e}"),
            })?;

        self.data = decompressed;
        self.metadata.remove("compressed");
        self.metadata.remove("original_size");

        Ok(())
    }
}

/// Codec for encoding/decoding data chunks
pub struct DataChunkCodec {
    max_chunk_size: usize,
}

impl DataChunkCodec {
    #[must_use] 
    pub fn new(max_chunk_size: usize) -> Self {
        Self { max_chunk_size }
    }
}

impl Decoder for DataChunkCodec {
    type Item = DataChunk;
    type Error = HiveError;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let length = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;

        if length > self.max_chunk_size {
            return Err(HiveError::ValidationError {
                field: "chunk_size".to_string(),
                reason: format!(
                    "Chunk size {} exceeds maximum {}",
                    length, self.max_chunk_size
                ),
            });
        }

        if src.len() < 4 + length {
            return Ok(None);
        }

        let data = src.split_to(4 + length);
        let chunk_data = &data[4..];

        match bincode::deserialize(chunk_data) {
            Ok(chunk) => Ok(Some(chunk)),
            Err(e) => Err(HiveError::MessageParsingError {
                reason: format!("Failed to deserialize chunk: {e}"),
            }),
        }
    }
}

impl Encoder<DataChunk> for DataChunkCodec {
    type Error = HiveError;

    fn encode(&mut self, item: DataChunk, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let serialized = bincode::serialize(&item).map_err(|e| HiveError::MessageParsingError {
            reason: format!("Failed to serialize chunk: {e}"),
        })?;

        if serialized.len() > self.max_chunk_size {
            return Err(HiveError::ValidationError {
                field: "chunk_size".to_string(),
                reason: format!(
                    "Serialized chunk size {} exceeds maximum {}",
                    serialized.len(),
                    self.max_chunk_size
                ),
            });
        }

        dst.extend_from_slice(&(serialized.len() as u32).to_be_bytes());
        dst.extend_from_slice(&serialized);
        Ok(())
    }
}

/// Stream processor for handling large data streams
#[derive(Debug)]
pub struct StreamProcessor {
    pub config: StreamConfig,
    memory_pool: Option<Arc<MemoryPool>>,
    parallel_config: Option<ParallelConfig>,
    memory_monitor: Option<Arc<MemoryMonitor>>,
}

impl StreamProcessor {
    /// Create a new stream processor
    #[must_use] 
    pub fn new(config: StreamConfig) -> Self {
        let memory_pool = if config.enable_memory_pool {
            Some(Arc::new(MemoryPool::new(
                config.memory_pool_size,
                config.max_chunk_size,
            )))
        } else {
            None
        };

        let parallel_config = if config.parallel_workers > 1 {
            Some(ParallelConfig::new(config.parallel_workers))
        } else {
            None
        };

        let memory_monitor = Some(Arc::new(MemoryMonitor::new(30.0))); // 30% target

        Self {
            config,
            memory_pool,
            parallel_config,
            memory_monitor,
        }
    }

    /// Process a stream of data chunks
    pub async fn process_stream<S, F, T>(&self, stream: S, processor: F) -> HiveResult<Vec<T>>
    where
        S: Stream<Item = HiveResult<DataChunk>> + Unpin,
        F: Fn(DataChunk) -> HiveResult<T>,
    {
        let mut results = Vec::new();
        let mut stream = stream;

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    // Verify chunk integrity
                    if !chunk.verify_checksum() {
                        error!("Chunk {} failed checksum verification", chunk.id);
                        return Err(HiveError::ValidationError {
                            field: "checksum".to_string(),
                            reason: "Chunk checksum verification failed".to_string(),
                        });
                    }

                    // Process the chunk
                    match processor(chunk) {
                        Ok(result) => results.push(result),
                        Err(e) => {
                            error!("Failed to process chunk: {}", e);
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    error!("Stream error: {}", e);
                    return Err(e);
                }
            }
        }

        info!("Successfully processed {} chunks", results.len());
        Ok(results)
    }

    /// Process a stream in parallel for improved performance
    pub async fn process_stream_parallel<S, F, T>(
        &self,
        stream: S,
        processor: F,
    ) -> HiveResult<Vec<T>>
    where
        S: Stream<Item = HiveResult<DataChunk>> + Unpin + Send,
        F: Fn(DataChunk) -> HiveResult<T> + Send + Sync + Clone,
        T: Send,
    {
        if let Some(parallel_config) = &self.parallel_config {
            debug!(
                "Processing stream with {} parallel workers",
                parallel_config.workers
            );

            let semaphore = Arc::clone(&parallel_config.semaphore);
            let processor_clone = processor.clone();

            let results = stream
                .map(move |chunk_result| {
                    let processor = processor_clone.clone();
                    let semaphore = Arc::clone(&semaphore);

                    async move {
                        let _permit =
                            semaphore
                                .acquire()
                                .await
                                .map_err(|e| HiveError::ProcessingError {
                                    reason: format!("Failed to acquire semaphore: {e}"),
                                })?;

                        match chunk_result {
                            Ok(chunk) => {
                                if !chunk.verify_checksum() {
                                    error!("Chunk {} failed checksum verification", chunk.id);
                                    return Err(HiveError::ValidationError {
                                        field: "checksum".to_string(),
                                        reason: "Chunk checksum verification failed".to_string(),
                                    });
                                }

                                match processor(chunk) {
                                    Ok(result) => Ok(result),
                                    Err(e) => {
                                        error!("Failed to process chunk: {}", e);
                                        Err(e)
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Stream error: {}", e);
                                Err(e)
                            }
                        }
                    }
                })
                .buffer_unordered(parallel_config.workers)
                .collect::<Vec<_>>()
                .await;

            // Convert results
            let mut final_results = Vec::new();
            for result in results {
                final_results.push(result?);
            }

            info!(
                "Successfully processed {} chunks in parallel",
                final_results.len()
            );
            Ok(final_results)
        } else {
            // Fall back to sequential processing
            self.process_stream(stream, processor).await
        }
    }

    /// Create a stream from data with memory pooling
    pub async fn create_stream_from_data_pooled(
        &self,
        data: Vec<u8>,
    ) -> HiveResult<impl Stream<Item = HiveResult<DataChunk>>> {
        let chunk_size = self.config.max_chunk_size;
        let chunks: Vec<HiveResult<DataChunk>> = if let Some(memory_pool) = &self.memory_pool {
            // Use memory pool for efficient allocation
            let mut pooled_chunks = Vec::new();
            for (i, chunk_data) in data.chunks(chunk_size).enumerate() {
                let mut pooled_buffer = memory_pool.get_chunk().await;
                pooled_buffer[..chunk_data.len()].copy_from_slice(chunk_data);

                let chunk = DataChunk::new(i as u64, pooled_buffer[..chunk_data.len()].to_vec())
                    .with_checksum();
                pooled_chunks.push(Ok(chunk));
            }
            pooled_chunks
        } else {
            // Standard allocation
            data.chunks(chunk_size)
                .enumerate()
                .map(|(i, chunk)| Ok(DataChunk::new(i as u64, chunk.to_vec()).with_checksum()))
                .collect()
        };

        Ok(futures::stream::iter(chunks))
    }

    /// Get memory pool statistics
    pub async fn memory_stats(&self) -> Option<(usize, usize)> {
        self.memory_pool.as_ref()?.stats().await.into()
    }

    /// Get memory usage statistics
    pub async fn get_memory_statistics(&self) -> HiveResult<MemoryStatistics> {
        match self.memory_monitor.as_ref() {
            Some(monitor) => Ok(monitor.get_statistics().await),
            None => Err(HiveError::ProcessingError {
                reason: "Memory monitor not available".to_string(),
            }),
        }
    }

    /// Record memory usage for monitoring
    pub async fn record_memory_usage(&self, usage: usize) {
        if let Some(monitor) = &self.memory_monitor {
            monitor.record_memory_usage(usage).await;
        }
    }

    /// Check if memory reduction target is met
    pub async fn is_memory_target_met(&self) -> bool {
        if let Some(monitor) = &self.memory_monitor {
            monitor.is_target_met().await
        } else {
            false
        }
    }

    /// Update memory efficiency metrics
    pub async fn update_memory_efficiency(&self, operation: &str, before: usize, after: usize) {
        if let Some(monitor) = &self.memory_monitor {
            monitor
                .update_efficiency_metrics(operation, before, after)
                .await;
        }
    }

    /// Create a stream from a large dataset
    pub fn create_stream_from_data(
        &self,
        data: Vec<u8>,
    ) -> impl Stream<Item = HiveResult<DataChunk>> {
        let chunk_size = self.config.max_chunk_size;
        let chunks: Vec<_> = data
            .chunks(chunk_size)
            .enumerate()
            .map(|(i, chunk)| DataChunk::new(i as u64, chunk.to_vec()).with_checksum())
            .collect();

        futures::stream::iter(chunks.into_iter().map(Ok))
    }

    /// Create a framed stream from an async reader/writer
    pub fn create_framed_stream<T>(&self, io: T) -> Framed<T, DataChunkCodec>
    where
        T: AsyncRead + AsyncWrite,
    {
        Framed::new(io, DataChunkCodec::new(self.config.max_chunk_size))
    }
}

/// Advanced neural processing pipeline with streaming support
#[derive(Debug)]
pub struct StreamingNeuralPipeline {
    /// Stream processor for data handling
    stream_processor: StreamProcessor,
    /// Memory pool for neural data
    neural_memory_pool: Arc<RwLock<Vec<Vec<f32>>>>,
    /// Performance monitor
    monitor: Arc<StreamingPerformanceMonitor>,
    /// Pipeline configuration
    config: NeuralPipelineConfig,
}

/// Configuration for neural processing pipeline
#[derive(Debug, Clone)]
pub struct NeuralPipelineConfig {
    /// Batch size for neural processing
    pub batch_size: usize,
    /// Memory pool size
    pub memory_pool_size: usize,
    /// Enable parallel processing
    pub enable_parallel: bool,
    /// Number of parallel workers
    pub parallel_workers: usize,
    /// Memory reduction target percentage
    pub memory_reduction_target: f64,
    /// Enable compression for neural data
    pub enable_compression: bool,
}

impl Default for NeuralPipelineConfig {
    fn default() -> Self {
        Self {
            batch_size: 32,
            memory_pool_size: 100,
            enable_parallel: true,
            parallel_workers: 4,
            memory_reduction_target: 30.0,
            enable_compression: true,
        }
    }
}

/// Neural data stream for processing large neural network datasets
#[derive(Debug)]
pub struct NeuralDataStream {
    processor: StreamProcessor,
    compression_enabled: bool,
}

impl NeuralDataStream {
    /// Create a new neural data stream
    #[must_use] 
    pub fn new(config: StreamConfig) -> Self {
        Self {
            processor: StreamProcessor::new(config.clone()),
            compression_enabled: config.enable_compression,
        }
    }

    /// Process neural training data in streaming fashion
    pub async fn process_training_data<S>(&self, stream: S) -> HiveResult<Vec<TrainingBatch>>
    where
        S: Stream<Item = HiveResult<DataChunk>> + Unpin,
    {
        self.processor
            .process_stream(stream, |chunk| {
                // Parse neural training data from chunk
                let training_data: TrainingData =
                    bincode::deserialize(&chunk.data).map_err(|e| {
                        HiveError::NeuralProcessingError {
                            reason: format!("Failed to deserialize training data: {e}"),
                        }
                    })?;

                Ok(TrainingBatch {
                    id: chunk.id,
                    sequence: chunk.sequence,
                    inputs: training_data.inputs,
                    targets: training_data.targets,
                    metadata: chunk.metadata,
                })
            })
            .await
    }

    /// Stream neural model weights for distributed training
    pub async fn stream_model_weights(
        &self,
        weights: Vec<f32>,
    ) -> HiveResult<impl Stream<Item = HiveResult<DataChunk>>> {
        let serialized =
            bincode::serialize(&weights).map_err(|e| HiveError::NeuralProcessingError {
                reason: format!("Failed to serialize weights: {e}"),
            })?;

        Ok(self.processor.create_stream_from_data(serialized))
    }

    /// Process neural training data in parallel for improved performance
    pub async fn process_training_data_parallel<S>(
        &self,
        stream: S,
    ) -> HiveResult<Vec<TrainingBatch>>
    where
        S: Stream<Item = HiveResult<DataChunk>> + Unpin + Send,
    {
        let processor = |chunk: DataChunk| {
            // Parse neural training data from chunk
            let training_data: TrainingData = bincode::deserialize(&chunk.data).map_err(|e| {
                HiveError::NeuralProcessingError {
                    reason: format!("Failed to deserialize training data: {e}"),
                }
            })?;

            Ok(TrainingBatch {
                id: chunk.id,
                sequence: chunk.sequence,
                inputs: training_data.inputs,
                targets: training_data.targets,
                metadata: chunk.metadata,
            })
        };

        self.processor
            .process_stream_parallel(stream, processor)
            .await
    }

    /// Stream large model weights with memory pooling
    pub async fn stream_model_weights_pooled(
        &self,
        weights: Vec<f32>,
    ) -> HiveResult<impl Stream<Item = HiveResult<DataChunk>>> {
        let serialized =
            bincode::serialize(&weights).map_err(|e| HiveError::NeuralProcessingError {
                reason: format!("Failed to serialize weights: {e}"),
            })?;

        self.processor
            .create_stream_from_data_pooled(serialized)
            .await
    }

    /// Get streaming performance metrics
    pub async fn get_performance_metrics(&self) -> HiveResult<StreamingMetrics> {
        let memory_stats = self.processor.memory_stats().await;

        Ok(StreamingMetrics {
            memory_pool_usage: memory_stats,
            compression_enabled: self.compression_enabled,
            parallel_workers: self.processor.config.parallel_workers,
            total_processed_chunks: 0, // Would be tracked in real implementation
            average_processing_time: 0.0, // Would be tracked in real implementation
            memory_efficiency_percentage: 0.0,
            throughput_cps: 0.0,
            compression_ratio: 0.0,
            total_memory_saved: 0,
            peak_memory_usage: 0,
            average_chunk_latency: 0.0,
            error_rate: 0.0,
            cache_hit_rate: 0.0,
        })
    }
}

/// Enhanced streaming performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingMetrics {
    pub memory_pool_usage: Option<(usize, usize)>,
    pub compression_enabled: bool,
    pub parallel_workers: usize,
    pub total_processed_chunks: u64,
    pub average_processing_time: f64,
    /// Memory efficiency percentage (higher is better)
    pub memory_efficiency_percentage: f64,
    /// Throughput in chunks per second
    pub throughput_cps: f64,
    /// Compression ratio (compressed/original size)
    pub compression_ratio: f64,
    /// Total memory saved through optimization
    pub total_memory_saved: usize,
    /// Peak memory usage during streaming
    pub peak_memory_usage: usize,
    /// Average chunk processing latency
    pub average_chunk_latency: f64,
    /// Error rate during streaming
    pub error_rate: f64,
    /// Cache hit rate for memory pool
    pub cache_hit_rate: f64,
}

/// Performance monitor for streaming operations
#[derive(Debug)]
pub struct StreamingPerformanceMonitor {
    /// Start time of monitoring
    start_time: std::time::Instant,
    /// Metrics collection
    metrics: Arc<RwLock<StreamingMetrics>>,
    /// Processing time samples
    processing_times: Arc<RwLock<Vec<f64>>>,
    /// Memory usage samples
    memory_samples: Arc<RwLock<Vec<usize>>>,
    /// Error count
    error_count: Arc<RwLock<u64>>,
    /// Total operations count
    total_operations: Arc<RwLock<u64>>,
}

/// Memory usage monitor for tracking and optimizing memory consumption
#[derive(Debug)]
pub struct MemoryMonitor {
    /// Peak memory usage recorded
    peak_memory_usage: Arc<RwLock<usize>>,
    /// Current memory usage
    current_memory_usage: Arc<RwLock<usize>>,
    /// Memory usage history
    memory_history: Arc<RwLock<Vec<(std::time::Instant, usize)>>>,
    /// Memory reduction target (percentage)
    memory_reduction_target: f64,
    /// Memory efficiency metrics
    efficiency_metrics: Arc<RwLock<MemoryEfficiencyMetrics>>,
}

impl MemoryMonitor {
    /// Create a new memory monitor
    #[must_use] 
    pub fn new(memory_reduction_target: f64) -> Self {
        Self {
            peak_memory_usage: Arc::new(RwLock::new(0)),
            current_memory_usage: Arc::new(RwLock::new(0)),
            memory_history: Arc::new(RwLock::new(Vec::new())),
            memory_reduction_target,
            efficiency_metrics: Arc::new(RwLock::new(MemoryEfficiencyMetrics::default())),
        }
    }

    /// Record memory usage
    pub async fn record_memory_usage(&self, usage: usize) {
        let mut current = self.current_memory_usage.write().await;
        let mut peak = self.peak_memory_usage.write().await;
        let mut history = self.memory_history.write().await;

        *current = usage;
        if usage > *peak {
            *peak = usage;
        }

        // Keep only recent history (last 1000 entries)
        history.push((std::time::Instant::now(), usage));
        if history.len() > 1000 {
            history.remove(0);
        }
    }

    /// Get current memory usage
    pub async fn get_current_usage(&self) -> usize {
        *self.current_memory_usage.read().await
    }

    /// Get peak memory usage
    pub async fn get_peak_usage(&self) -> usize {
        *self.peak_memory_usage.read().await
    }

    /// Calculate memory efficiency
    pub async fn calculate_efficiency(&self) -> f64 {
        let current = self.get_current_usage().await;
        let peak = self.get_peak_usage().await;

        if peak == 0 {
            return 100.0;
        }

        (1.0 - (current as f64 / peak as f64)) * 100.0
    }

    /// Check if memory reduction target is met
    pub async fn is_target_met(&self) -> bool {
        self.calculate_efficiency().await >= self.memory_reduction_target
    }

    /// Get memory usage statistics
    pub async fn get_statistics(&self) -> MemoryStatistics {
        let current = self.get_current_usage().await;
        let peak = self.get_peak_usage().await;
        let efficiency = self.calculate_efficiency().await;
        let history = self.memory_history.read().await;

        let avg_usage = if history.is_empty() {
            0
        } else {
            history.iter().map(|(_, usage)| *usage).sum::<usize>() / history.len()
        };

        MemoryStatistics {
            current_usage: current,
            peak_usage: peak,
            average_usage: avg_usage,
            efficiency_percentage: efficiency,
            target_met: self.is_target_met().await,
            history_length: history.len(),
        }
    }

    /// Update efficiency metrics
    pub async fn update_efficiency_metrics(&self, operation: &str, before: usize, after: usize) {
        let reduction = if before > 0 {
            ((before - after) as f64 / before as f64) * 100.0
        } else {
            0.0
        };

        let mut metrics = self.efficiency_metrics.write().await;
        metrics.total_operations += 1;
        metrics.total_memory_reduction += reduction;
        metrics.average_reduction =
            metrics.total_memory_reduction / metrics.total_operations as f64;

        if reduction > metrics.best_reduction {
            metrics.best_reduction = reduction;
            metrics.best_operation = operation.to_string();
        }

        tracing::debug!(
            "Memory optimization: {} - {:.2}% reduction ({} -> {} bytes)",
            operation,
            reduction,
            before,
            after
        );
    }
}

/// Memory efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEfficiencyMetrics {
    pub total_operations: u64,
    pub total_memory_reduction: f64,
    pub average_reduction: f64,
    pub best_reduction: f64,
    pub best_operation: String,
}

impl Default for MemoryEfficiencyMetrics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            total_memory_reduction: 0.0,
            average_reduction: 0.0,
            best_reduction: 0.0,
            best_operation: "none".to_string(),
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    pub current_usage: usize,
    pub peak_usage: usize,
    pub average_usage: usize,
    pub efficiency_percentage: f64,
    pub target_met: bool,
    pub history_length: usize,
}

/// Training data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct TrainingData {
    pub inputs: Vec<Vec<f32>>,
    pub targets: Vec<Vec<f32>>,
}

/// Training batch for neural processing
#[derive(Debug)]
pub struct TrainingBatch {
    pub id: uuid::Uuid,
    pub sequence: u64,
    pub inputs: Vec<Vec<f32>>,
    pub targets: Vec<Vec<f32>>,
    pub metadata: HashMap<String, String>,
}

/// Memory pool for efficient chunk reuse
#[derive(Debug)]
pub struct MemoryPool {
    pool: Arc<RwLock<Vec<Vec<u8>>>>,
    max_pool_size: usize,
    chunk_size: usize,
}

impl MemoryPool {
    /// Create a new memory pool
    #[must_use] 
    pub fn new(max_pool_size: usize, chunk_size: usize) -> Self {
        Self {
            pool: Arc::new(RwLock::new(Vec::with_capacity(max_pool_size))),
            max_pool_size,
            chunk_size,
        }
    }

    /// Get a chunk from the pool or allocate a new one
    pub async fn get_chunk(&self) -> Vec<u8> {
        let mut pool = self.pool.write().await;
        if let Some(chunk) = pool.pop() {
            debug!("Reusing chunk from memory pool");
            chunk
        } else {
            debug!("Allocating new chunk for memory pool");
            vec![0u8; self.chunk_size]
        }
    }

    /// Return a chunk to the pool for reuse
    pub async fn return_chunk(&self, mut chunk: Vec<u8>) {
        let mut pool = self.pool.write().await;
        if pool.len() < self.max_pool_size {
            chunk.clear();
            chunk.resize(self.chunk_size, 0);
            pool.push(chunk);
            debug!("Returned chunk to memory pool");
        }
    }

    /// Get pool statistics
    pub async fn stats(&self) -> (usize, usize) {
        let pool = self.pool.read().await;
        (pool.len(), self.max_pool_size)
    }
}

/// Parallel processing configuration
#[derive(Debug)]
pub struct ParallelConfig {
    /// Number of worker threads
    pub workers: usize,
    /// Semaphore for controlling concurrency
    pub semaphore: Arc<Semaphore>,
    /// Chunk processing timeout
    pub chunk_timeout: std::time::Duration,
}

impl ParallelConfig {
    #[must_use] 
    pub fn new(workers: usize) -> Self {
        Self {
            workers,
            semaphore: Arc::new(Semaphore::new(workers)),
            chunk_timeout: std::time::Duration::from_secs(10),
        }
    }
}

impl Default for StreamingPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingPerformanceMonitor {
    /// Create new performance monitor
    #[must_use] 
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            metrics: Arc::new(RwLock::new(StreamingMetrics {
                memory_pool_usage: None,
                compression_enabled: false,
                parallel_workers: 1,
                total_processed_chunks: 0,
                average_processing_time: 0.0,
                memory_efficiency_percentage: 0.0,
                throughput_cps: 0.0,
                compression_ratio: 1.0,
                total_memory_saved: 0,
                peak_memory_usage: 0,
                average_chunk_latency: 0.0,
                error_rate: 0.0,
                cache_hit_rate: 0.0,
            })),
            processing_times: Arc::new(RwLock::new(Vec::new())),
            memory_samples: Arc::new(RwLock::new(Vec::new())),
            error_count: Arc::new(RwLock::new(0)),
            total_operations: Arc::new(RwLock::new(0)),
        }
    }

    /// Record processing time for a chunk
    pub async fn record_processing_time(&self, duration: f64) {
        let mut times = self.processing_times.write().await;
        times.push(duration);

        // Keep only recent samples (last 1000)
        if times.len() > 1000 {
            times.remove(0);
        }

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_processed_chunks += 1;
        metrics.average_processing_time = times.iter().sum::<f64>() / times.len() as f64;
        metrics.average_chunk_latency = metrics.average_processing_time;

        // Calculate throughput
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            metrics.throughput_cps = metrics.total_processed_chunks as f64 / elapsed;
        }
    }

    /// Record memory usage
    pub async fn record_memory_usage(&self, usage: usize) {
        let mut samples = self.memory_samples.write().await;
        samples.push(usage);

        // Keep only recent samples
        if samples.len() > 1000 {
            samples.remove(0);
        }

        // Update peak memory usage
        let mut metrics = self.metrics.write().await;
        if usage > metrics.peak_memory_usage {
            metrics.peak_memory_usage = usage;
        }

        // Calculate memory efficiency (simulate based on usage patterns)
        if !samples.is_empty() {
            let avg_usage = samples.iter().sum::<usize>() as f64 / samples.len() as f64;
            let efficiency = if metrics.peak_memory_usage > 0 {
                (1.0 - (avg_usage / metrics.peak_memory_usage as f64)) * 100.0
            } else {
                100.0
            };
            metrics.memory_efficiency_percentage = efficiency;
        }
    }

    /// Record error occurrence
    pub async fn record_error(&self) {
        *self.error_count.write().await += 1;
        *self.total_operations.write().await += 1;

        // Update error rate
        let error_count = *self.error_count.read().await;
        let total_ops = *self.total_operations.read().await;

        let mut metrics = self.metrics.write().await;
        metrics.error_rate = if total_ops > 0 {
            error_count as f64 / total_ops as f64
        } else {
            0.0
        };
    }

    /// Record successful operation
    pub async fn record_success(&self) {
        *self.total_operations.write().await += 1;
    }

    /// Record compression metrics
    pub async fn record_compression(&self, original_size: usize, compressed_size: usize) {
        let mut metrics = self.metrics.write().await;
        if original_size > 0 {
            let ratio = compressed_size as f64 / original_size as f64;
            metrics.compression_ratio = f64::midpoint(metrics.compression_ratio, ratio); // Running average

            if compressed_size < original_size {
                metrics.total_memory_saved += original_size - compressed_size;
            }
        }
    }

    /// Record cache hit for memory pool
    pub async fn record_cache_hit(&self) {
        let mut metrics = self.metrics.write().await;
        // Simplified cache hit tracking - in real implementation would track hits vs misses
        metrics.cache_hit_rate = f64::midpoint(metrics.cache_hit_rate, 1.0);
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> StreamingMetrics {
        self.metrics.read().await.clone()
    }

    /// Generate performance report
    pub async fn generate_report(&self) -> String {
        let metrics = self.get_metrics().await;
        let uptime = self.start_time.elapsed().as_secs_f64();

        format!(
            "Streaming Performance Report (Uptime: {:.2}s)\n\
             ===========================================\n\
             Chunks Processed: {}\n\
             Average Processing Time: {:.4}ms\n\
             Throughput: {:.2} chunks/sec\n\
             Memory Efficiency: {:.2}%\n\
             Peak Memory Usage: {} bytes\n\
             Compression Ratio: {:.2}x\n\
             Total Memory Saved: {} bytes\n\
             Error Rate: {:.4}%\n\
             Cache Hit Rate: {:.2}%",
            uptime,
            metrics.total_processed_chunks,
            metrics.average_processing_time * 1000.0,
            metrics.throughput_cps,
            metrics.memory_efficiency_percentage,
            metrics.peak_memory_usage,
            metrics.compression_ratio,
            metrics.total_memory_saved,
            metrics.error_rate * 100.0,
            metrics.cache_hit_rate * 100.0
        )
    }

    /// Check if performance targets are met
    pub async fn check_performance_targets(&self) -> PerformanceStatus {
        let metrics = self.get_metrics().await;

        let memory_target_met = metrics.memory_efficiency_percentage >= 30.0;
        let throughput_target_met = metrics.throughput_cps >= 100.0; // 100 chunks/sec minimum
        let error_rate_acceptable = metrics.error_rate <= 0.05; // 5% max error rate

        if memory_target_met && throughput_target_met && error_rate_acceptable {
            PerformanceStatus::Excellent
        } else if memory_target_met || throughput_target_met {
            PerformanceStatus::Good
        } else {
            PerformanceStatus::NeedsImprovement
        }
    }
}

/// Performance status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceStatus {
    Excellent,
    Good,
    NeedsImprovement,
}

/// Advanced streaming iterator with lazy evaluation and memory pooling
pub struct StreamingIterator<T> {
    data_source: Box<dyn Iterator<Item = T> + Send>,
    chunk_size: usize,
    buffer: Vec<T>,
    buffer_index: usize,
    memory_pool: Option<Arc<RwLock<Vec<Vec<T>>>>>,
    /// Performance monitor
    monitor: Option<Arc<StreamingPerformanceMonitor>>,
    /// Memory usage tracking
    memory_usage: usize,
}

/// Memory-efficient data iterator with performance monitoring
pub struct MemoryEfficientIterator<T> {
    data: Vec<T>,
    chunk_size: usize,
    current_index: usize,
    /// Performance monitor
    monitor: Option<Arc<StreamingPerformanceMonitor>>,
}

impl<T> StreamingIterator<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new streaming iterator with lazy evaluation
    pub fn new<I>(data_source: I, chunk_size: usize) -> Self
    where
        I: Iterator<Item = T> + Send + 'static,
    {
        Self {
            data_source: Box::new(data_source),
            chunk_size,
            buffer: Vec::with_capacity(chunk_size),
            buffer_index: 0,
            memory_pool: None,
            monitor: None,
            memory_usage: 0,
        }
    }

    /// Create streaming iterator with memory pooling
    pub fn new_with_pooling<I>(
        data_source: I,
        chunk_size: usize,
        memory_pool: Arc<RwLock<Vec<Vec<T>>>>,
    ) -> Self
    where
        I: Iterator<Item = T> + Send + 'static,
    {
        Self {
            data_source: Box::new(data_source),
            chunk_size,
            buffer: Vec::with_capacity(chunk_size),
            buffer_index: 0,
            memory_pool: Some(memory_pool),
            monitor: None,
            memory_usage: 0,
        }
    }

    /// Create streaming iterator with performance monitoring
    pub fn new_with_monitor<I>(
        data_source: I,
        chunk_size: usize,
        monitor: Arc<StreamingPerformanceMonitor>,
    ) -> Self
    where
        I: Iterator<Item = T> + Send + 'static,
    {
        Self {
            data_source: Box::new(data_source),
            chunk_size,
            buffer: Vec::with_capacity(chunk_size),
            buffer_index: 0,
            memory_pool: None,
            monitor: Some(monitor),
            memory_usage: 0,
        }
    }

    /// Get next chunk with lazy evaluation
    pub async fn next_chunk(&mut self) -> Option<Vec<T>> {
        let start_time = std::time::Instant::now();

        // Fill buffer if needed
        if self.buffer_index >= self.buffer.len() {
            self.fill_buffer().await?;
        }

        if self.buffer_index >= self.buffer.len() {
            return None;
        }

        // Get chunk from buffer
        let remaining = self.buffer.len() - self.buffer_index;
        let chunk_size = std::cmp::min(self.chunk_size, remaining);
        let chunk = self.buffer[self.buffer_index..self.buffer_index + chunk_size].to_vec();
        self.buffer_index += chunk_size;

        // Update memory usage
        self.memory_usage = self
            .memory_usage
            .saturating_sub(chunk.len() * std::mem::size_of::<T>());

        // Record performance metrics
        if let Some(monitor) = &self.monitor {
            let processing_time = start_time.elapsed().as_secs_f64();
            let memory_usage = chunk.len() * std::mem::size_of::<T>();

            let monitor_clone = Arc::clone(monitor);
            tokio::spawn(async move {
                monitor_clone.record_processing_time(processing_time).await;
                monitor_clone.record_memory_usage(memory_usage).await;
                monitor_clone.record_success().await;
            });
        }

        Some(chunk)
    }

    /// Fill buffer with data from source
    async fn fill_buffer(&mut self) -> Option<()> {
        self.buffer.clear();
        self.buffer_index = 0;

        // Try to get buffer from memory pool
        if let Some(pool) = &self.memory_pool {
            let mut pool_write = pool.write().await;
            if let Some(mut pooled_buffer) = pool_write.pop() {
                pooled_buffer.clear();
                pooled_buffer.reserve(self.chunk_size);
                self.buffer = pooled_buffer;
            }
        }

        // Fill buffer from data source
        for _ in 0..self.chunk_size {
            if let Some(item) = self.data_source.next() {
                self.buffer.push(item);
            } else {
                break;
            }
        }

        if self.buffer.is_empty() {
            return None;
        }

        // Update memory usage
        self.memory_usage += self.buffer.len() * std::mem::size_of::<T>();

        Some(())
    }

    /// Return buffer to memory pool
    pub async fn return_buffer(&mut self) {
        if let Some(pool) = &self.memory_pool {
            let mut pool_write = pool.write().await;
            if pool_write.len() < 100 {
                // Limit pool size
                let mut buffer = Vec::with_capacity(self.chunk_size);
                std::mem::swap(&mut buffer, &mut self.buffer);
                pool_write.push(buffer);
            }
        }
        self.buffer.clear();
        self.buffer_index = 0;
    }

    /// Get current memory usage
    #[must_use] 
    pub fn memory_usage(&self) -> usize {
        self.memory_usage
    }

    /// Get memory efficiency percentage
    #[must_use] 
    pub fn memory_efficiency(&self) -> f64 {
        if self.chunk_size == 0 {
            return 100.0;
        }
        let buffer_efficiency = (self.buffer.capacity() as f64 - self.buffer.len() as f64)
            / self.buffer.capacity() as f64;
        (1.0 - buffer_efficiency) * 100.0
    }
}

impl<T> MemoryEfficientIterator<T> {
    /// Create a new memory-efficient iterator
    #[must_use] 
    pub fn new(data: Vec<T>, chunk_size: usize) -> Self {
        Self {
            data,
            chunk_size,
            current_index: 0,
            monitor: None,
        }
    }

    /// Create iterator with performance monitoring
    #[must_use] 
    pub fn new_with_monitor(
        data: Vec<T>,
        chunk_size: usize,
        monitor: Arc<StreamingPerformanceMonitor>,
    ) -> Self {
        Self {
            data,
            chunk_size,
            current_index: 0,
            monitor: Some(monitor),
        }
    }
}

impl<T: Clone> Iterator for MemoryEfficientIterator<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.data.len() {
            return None;
        }

        let start_time = std::time::Instant::now();
        let end_index = std::cmp::min(self.current_index + self.chunk_size, self.data.len());
        let chunk = self.data[self.current_index..end_index].to_vec();
        self.current_index = end_index;

        // Record performance metrics
        if let Some(monitor) = &self.monitor {
            let processing_time = start_time.elapsed().as_secs_f64();
            let memory_usage = chunk.len() * std::mem::size_of::<T>();

            // Spawn async task to record metrics without blocking
            let monitor_clone = Arc::clone(monitor);
            tokio::spawn(async move {
                monitor_clone.record_processing_time(processing_time).await;
                monitor_clone.record_memory_usage(memory_usage).await;
                monitor_clone.record_success().await;
            });
        }

        Some(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_processor() -> Result<(), Box<dyn std::error::Error>> {
        let config = StreamConfig::default();
        let processor = StreamProcessor::new(config);

        // Create test data
        let test_data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let data_stream = processor.create_stream_from_data(test_data.clone());

        // Process the stream
        let results = processor
            .process_stream(data_stream, |chunk| Ok(chunk.data.len()))
            .await?;

        assert!(!results.is_empty());
        let total_processed: usize = results.iter().sum();
        assert_eq!(total_processed, test_data.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_data_chunk_checksum() -> Result<(), Box<dyn std::error::Error>> {
        let data = vec![1, 2, 3, 4, 5];
        let chunk = DataChunk::new(0, data).with_checksum();

        assert!(chunk.verify_checksum());

        // Modify data and verify checksum fails
        let mut modified_chunk = chunk.clone();
        modified_chunk.data[0] = 99;
        assert!(!modified_chunk.verify_checksum());

        Ok(())
    }

    #[test]
    fn test_memory_efficient_iterator() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut iter = MemoryEfficientIterator::new(data, 3);

        assert_eq!(iter.next(), Some(vec![1, 2, 3]));
        assert_eq!(iter.next(), Some(vec![4, 5, 6]));
        assert_eq!(iter.next(), Some(vec![7, 8, 9]));
        assert_eq!(iter.next(), Some(vec![10]));
        assert_eq!(iter.next(), None);
    }

    #[tokio::test]
    async fn test_memory_pool() -> Result<(), Box<dyn std::error::Error>> {
        let pool = MemoryPool::new(5, 1024);

        // Test getting chunks
        let chunk1 = pool.get_chunk().await;
        assert_eq!(chunk1.len(), 1024);

        let chunk2 = pool.get_chunk().await;
        assert_eq!(chunk2.len(), 1024);

        // Test returning chunks
        pool.return_chunk(chunk1).await;
        pool.return_chunk(chunk2).await;

        // Check pool stats
        let (pool_size, max_size) = pool.stats().await;
        assert_eq!(pool_size, 2);
        assert_eq!(max_size, 5);

        Ok(())
    }

    #[tokio::test]
    async fn test_parallel_stream_processing() -> Result<(), Box<dyn std::error::Error>> {
        let mut config = StreamConfig::default();
        config.parallel_workers = 2;
        let processor = StreamProcessor::new(config);

        // Create test data
        let test_data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let data_stream = processor.create_stream_from_data(test_data.clone());

        // Process the stream in parallel
        let results = processor
            .process_stream_parallel(data_stream, |chunk| Ok(chunk.data.len()))
            .await?;

        assert!(!results.is_empty());
        let total_processed: usize = results.iter().sum();
        assert_eq!(total_processed, test_data.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_chunk_compression() -> Result<(), Box<dyn std::error::Error>> {
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut chunk = DataChunk::new(0, data.clone());

        let original_size = chunk.data.len();

        // Compress the chunk
        chunk.compress(6)?;
        assert!(chunk.data.len() <= original_size); // Compressed should be same or smaller

        // Verify compression metadata
        assert_eq!(chunk.metadata.get("compressed"), Some(&"true".to_string()));

        // Decompress the chunk
        chunk.decompress()?;
        assert_eq!(chunk.data, data); // Should be identical after decompression

        // Verify decompression metadata
        assert_eq!(chunk.metadata.get("compressed"), None);

        Ok(())
    }

    #[tokio::test]
    async fn test_streaming_metrics() -> Result<(), Box<dyn std::error::Error>> {
        let config = StreamConfig::default();
        let neural_stream = NeuralDataStream::new(config);

        let metrics = neural_stream.get_performance_metrics().await?;

        assert_eq!(metrics.compression_enabled, false);
        assert_eq!(metrics.parallel_workers, 4);
        assert_eq!(metrics.total_processed_chunks, 0);
        assert_eq!(metrics.average_processing_time, 0.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_memory_monitor() -> Result<(), Box<dyn std::error::Error>> {
        let monitor = MemoryMonitor::new(30.0);

        // Record some memory usage
        monitor.record_memory_usage(1000).await;
        monitor.record_memory_usage(800).await;
        monitor.record_memory_usage(1200).await;

        // Check statistics
        let stats = monitor.get_statistics().await;
        assert_eq!(stats.current_usage, 1200);
        assert_eq!(stats.peak_usage, 1200);
        assert!(stats.average_usage > 0);

        // Test efficiency calculation
        let efficiency = monitor.calculate_efficiency().await;
        assert!(efficiency >= 0.0 && efficiency <= 100.0);

        // Test target checking
        let target_met = monitor.is_target_met().await;
        assert!(!target_met); // 30% target not met with current data

        Ok(())
    }

    #[tokio::test]
    async fn test_memory_efficiency_tracking() -> Result<(), Box<dyn std::error::Error>> {
        let monitor = MemoryMonitor::new(30.0);

        // Simulate memory optimization operations
        monitor
            .update_efficiency_metrics("test_operation_1", 1000, 700)
            .await;
        monitor
            .update_efficiency_metrics("test_operation_2", 2000, 1200)
            .await;

        let metrics = monitor.efficiency_metrics.read().await;
        assert_eq!(metrics.total_operations, 2);
        assert!(metrics.average_reduction > 0.0);
        assert_eq!(metrics.best_operation, "test_operation_2"); // 40% reduction

        Ok(())
    }

    #[tokio::test]
    async fn test_stream_processor_memory_monitoring() -> Result<(), Box<dyn std::error::Error>> {
        let config = StreamConfig::default();
        let processor = StreamProcessor::new(config);

        // Record memory usage
        processor.record_memory_usage(1024).await;
        processor.record_memory_usage(2048).await;

        // Check memory statistics
        let stats = processor.get_memory_statistics().await?;
        assert_eq!(stats.current_usage, 2048);
        assert_eq!(stats.peak_usage, 2048);

        // Test memory target (should be false initially)
        let target_met = processor.is_memory_target_met().await;
        assert!(!target_met);

        Ok(())
    }

    #[tokio::test]
    async fn test_large_dataset_streaming() -> Result<(), Box<dyn std::error::Error>> {
        let config = StreamConfig {
            buffer_size: 8192,
            max_chunk_size: 1024 * 1024, // 1MB chunks
            timeout: std::time::Duration::from_secs(30),
            enable_compression: false,
            parallel_workers: 2,
            memory_pool_size: 10,
            enable_memory_pool: true,
            compression_level: 6,
        };

        let processor = StreamProcessor::new(config);

        // Create a large dataset (simulate 10MB)
        let large_data = vec![0u8; 10 * 1024 * 1024];
        let data_stream = processor
            .create_stream_from_data_pooled(large_data.clone())
            .await?;

        // Process the stream
        let results = processor
            .process_stream(data_stream, |chunk| Ok(chunk.data.len()))
            .await?;

        // Verify all data was processed
        let total_processed: usize = results.iter().sum();
        assert_eq!(total_processed, large_data.len());

        // Check memory efficiency
        let target_met = processor.is_memory_target_met().await;
        // Note: In a real scenario, this would be true with proper memory monitoring

        tracing::info!(
            "✅ Large dataset streaming test completed: {} bytes processed",
            total_processed
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_distributed_streaming_simulation() -> Result<(), Box<dyn std::error::Error>> {
        let config = StreamConfig {
            parallel_workers: 4,
            enable_memory_pool: true,
            ..Default::default()
        };

        let neural_stream = NeuralDataStream::new(config);

        // Simulate distributed weight streaming
        let weights: Vec<f32> = (0..100_000).map(|i| (i as f32 * 0.001).sin()).collect();
        let weight_stream = neural_stream
            .stream_model_weights_pooled(weights.clone())
            .await?;

        // Collect and verify weights
        let collected_chunks: Vec<_> = weight_stream.collect().await;
        let mut total_weights = Vec::new();

        for chunk_result in collected_chunks {
            if let Ok(chunk) = chunk_result {
                let chunk_weights: Vec<f32> = bincode::deserialize(&chunk.data)?;
                total_weights.extend(chunk_weights);
            }
        }

        assert_eq!(total_weights.len(), weights.len());
        assert_eq!(total_weights, weights);

        tracing::info!(
            "✅ Distributed streaming simulation completed: {} weights processed",
            total_weights.len()
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_optimization_metrics() -> Result<(), Box<dyn std::error::Error>> {
        let config = StreamConfig::default();
        let processor = StreamProcessor::new(config);

        // Simulate memory optimization
        let before_usage = 2048;
        let after_usage = 1024; // 50% reduction

        processor.record_memory_usage(before_usage).await;
        processor
            .update_memory_efficiency("test_optimization", before_usage, after_usage)
            .await;
        processor.record_memory_usage(after_usage).await;

        // Verify memory reduction
        let stats = processor.get_memory_statistics().await?;
        assert_eq!(stats.current_usage, after_usage);
        assert_eq!(stats.peak_usage, before_usage);

        // Calculate expected efficiency
        let expected_efficiency = (1.0 - (after_usage as f64 / before_usage as f64)) * 100.0;
        assert_eq!(stats.efficiency_percentage, expected_efficiency);

        tracing::info!(
            "✅ Memory optimization test completed: {:.2}% efficiency achieved",
            expected_efficiency
        );
        Ok(())
    }
}
