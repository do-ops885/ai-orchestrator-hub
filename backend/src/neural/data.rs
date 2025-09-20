use crate::infrastructure::streaming::{
    DataChunk, NeuralDataStream, StreamConfig, StreamProcessor,
};
use crate::neural::{CpuOptimizer, VectorizedOps};
// use anyhow::Result; // Replaced with HiveResult
use crate::utils::error::{HiveError, HiveResult};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Data preparation and loading system for neural training
#[derive(Debug)]
#[allow(dead_code)]
pub struct DataPipeline {
    /// CPU optimizer for vectorized operations
    cpu_optimizer: CpuOptimizer,
    /// Loaded datasets
    datasets: HashMap<String, Arc<RwLock<Dataset>>>,
    /// Data loaders
    data_loaders: HashMap<String, Arc<RwLock<DataLoader>>>,
    /// Data preprocessing pipelines
    preprocessing: HashMap<String, PreprocessingPipeline>,
    /// Streaming processor for large datasets
    pub stream_processor: Option<StreamProcessor>,
    /// Neural data stream for efficient processing
    neural_stream: Option<NeuralDataStream>,
}

/// Dataset representation
#[derive(Debug, Clone)]
pub struct Dataset {
    pub name: String,
    pub features: Vec<Vec<f32>>,
    pub labels: Vec<Vec<f32>>,
    pub metadata: DatasetMetadata,
}

/// Dataset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub num_samples: usize,
    pub num_features: usize,
    pub num_classes: usize,
    pub feature_names: Vec<String>,
    pub class_names: Vec<String>,
    pub data_type: DataType,
}

/// Data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Tabular,
    Image,
    Text,
    TimeSeries,
    Graph,
}

/// Data loader for batch processing
#[derive(Debug)]
pub struct DataLoader {
    pub dataset: Arc<RwLock<Dataset>>,
    pub batch_size: usize,
    pub shuffle: bool,
    pub current_index: usize,
    pub indices: Vec<usize>,
}

/// Data batch
#[derive(Debug, Clone)]
pub struct DataBatch {
    pub features: Vec<Vec<f32>>,
    pub labels: Vec<Vec<f32>>,
    pub batch_size: usize,
    pub metadata: BatchMetadata,
}

/// Batch metadata
#[derive(Debug, Clone)]
pub struct BatchMetadata {
    pub batch_index: usize,
    pub total_batches: usize,
    pub sample_indices: Vec<usize>,
}

/// Preprocessing pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingPipeline {
    pub steps: Vec<PreprocessingStep>,
}

/// Preprocessing step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreprocessingStep {
    Normalization(NormalizationConfig),
    Standardization(StandardizationConfig),
    Encoding(EncodingConfig),
    Augmentation(AugmentationConfig),
    FeatureSelection(FeatureSelectionConfig),
}

/// Normalization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationConfig {
    pub method: NormalizationMethod,
    pub feature_range: Option<(f32, f32)>,
}

/// Normalization methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizationMethod {
    MinMax,
    L1,
    L2,
    ZScore,
}

/// Standardization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardizationConfig {
    pub with_mean: bool,
    pub with_std: bool,
}

/// Encoding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingConfig {
    pub method: EncodingMethod,
    pub categories: Option<Vec<String>>,
}

/// Encoding methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncodingMethod {
    OneHot,
    Label,
    Ordinal,
    Binary,
}

/// Augmentation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AugmentationConfig {
    pub techniques: Vec<AugmentationTechnique>,
    pub probability: f64,
}

/// Augmentation techniques
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AugmentationTechnique {
    Noise { std: f32 },
    Rotation { degrees: f32 },
    Flip { horizontal: bool, vertical: bool },
    Scale { factor: f32 },
    Translation { x: f32, y: f32 },
}

/// Feature selection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSelectionConfig {
    pub method: FeatureSelectionMethod,
    pub k: usize,
}

/// Feature selection methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureSelectionMethod {
    VarianceThreshold { threshold: f64 },
    SelectKBest { score_func: String },
    RecursiveFeatureElimination,
}

/// Data split configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSplit {
    pub train_ratio: f64,
    pub val_ratio: f64,
    pub test_ratio: f64,
    pub stratify: bool,
}

/// Training data splits
#[derive(Debug, Clone)]
pub struct DataSplits {
    pub train: Dataset,
    pub validation: Dataset,
    pub test: Dataset,
}

/// Memory-efficient streaming data loader for large neural datasets
#[derive(Debug)]
pub struct StreamingDataLoader {
    /// Path to the data source
    data_path: std::path::PathBuf,
    /// Stream configuration
    config: StreamConfig,
    /// Current position in the data stream
    current_position: u64,
    /// Total size of the dataset
    total_size: Option<u64>,
    /// Memory pool for efficient chunk reuse
    memory_pool: Arc<RwLock<Vec<Vec<u8>>>>,
    /// Compression buffer for memory optimization
    compression_buffer: Vec<u8>,
    /// Performance metrics
    metrics: StreamingLoaderMetrics,
}

/// Performance metrics for streaming data loader
#[derive(Debug, Clone, Default)]
pub struct StreamingLoaderMetrics {
    pub total_chunks_processed: u64,
    pub total_bytes_processed: u64,
    pub memory_usage_peak: usize,
    pub memory_usage_current: usize,
    pub compression_ratio: f64,
    pub processing_time_avg: f64,
}

impl StreamingDataLoader {
    /// Create a new memory-efficient streaming data loader
    pub fn new(data_path: std::path::PathBuf, config: StreamConfig) -> Self {
        let memory_pool = Arc::new(RwLock::new(Vec::with_capacity(config.memory_pool_size)));
        let compression_buffer = Vec::with_capacity(config.max_chunk_size);

        Self {
            data_path,
            config,
            current_position: 0,
            total_size: None,
            memory_pool,
            compression_buffer,
            metrics: StreamingLoaderMetrics::default(),
        }
    }

    /// Create streaming data loader with enhanced memory optimization
    pub fn new_optimized(data_path: std::path::PathBuf, config: StreamConfig) -> HiveResult<Self> {
        let mut loader = Self::new(data_path, config);

        // Pre-allocate memory pool for better performance
        {
            let mut pool =
                loader
                    .memory_pool
                    .try_write()
                    .map_err(|e| HiveError::ProcessingError {
                        reason: format!("Failed to acquire write lock on memory pool: {}", e),
                    })?;
            for _ in 0..loader.config.memory_pool_size {
                pool.push(vec![0u8; loader.config.max_chunk_size]);
            }
        }

        loader.metrics.memory_usage_peak =
            loader.config.memory_pool_size * loader.config.max_chunk_size;
        Ok(loader)
    }

    /// Get the next chunk of data as a memory-efficient stream
    pub async fn get_next_chunk_stream(
        &mut self,
    ) -> HiveResult<impl Stream<Item = HiveResult<DataChunk>>> {
        let start_time = std::time::Instant::now();

        // Get chunk from memory pool for efficiency
        let chunk_data = self.get_chunk_from_pool().await;

        // In a real implementation, this would read from the file/network
        // For now, we'll simulate streaming data with memory optimization
        let chunk_size = std::cmp::min(self.config.max_chunk_size, chunk_data.len());
        let simulated_data = &chunk_data[..chunk_size];

        // Apply compression if enabled for memory efficiency
        let final_data = if self.config.enable_compression {
            self.compress_chunk_data(simulated_data)?
        } else {
            simulated_data.to_vec()
        };

        let chunk = DataChunk::new(self.current_position, final_data).with_checksum();

        // Update metrics
        self.metrics.total_chunks_processed += 1;
        self.metrics.total_bytes_processed += chunk.data.len() as u64;
        self.metrics.processing_time_avg = (self.metrics.processing_time_avg
            * (self.metrics.total_chunks_processed - 1) as f64
            + start_time.elapsed().as_secs_f64())
            / self.metrics.total_chunks_processed as f64;

        if self.config.enable_compression && chunk_size > 0 {
            self.metrics.compression_ratio = (self.metrics.compression_ratio
                * (self.metrics.total_chunks_processed - 1) as f64
                + (chunk.data.len() as f64 / chunk_size as f64))
                / self.metrics.total_chunks_processed as f64;
        }

        self.current_position += 1;

        // Return chunk to pool after processing
        self.return_chunk_to_pool(chunk_data).await;

        Ok(futures::stream::iter(vec![Ok(chunk)]))
    }

    /// Get chunk buffer from memory pool
    async fn get_chunk_from_pool(&self) -> Vec<u8> {
        let mut pool = self.memory_pool.write().await;
        if let Some(chunk) = pool.pop() {
            tracing::debug!("Reused chunk from memory pool");
            chunk
        } else {
            tracing::debug!("Allocated new chunk for memory pool");
            vec![0u8; self.config.max_chunk_size]
        }
    }

    /// Return chunk buffer to memory pool for reuse
    async fn return_chunk_to_pool(&self, mut chunk: Vec<u8>) {
        let mut pool = self.memory_pool.write().await;
        if pool.len() < self.config.memory_pool_size {
            chunk.clear();
            chunk.resize(self.config.max_chunk_size, 0);
            pool.push(chunk);
            tracing::debug!("Returned chunk to memory pool");
        }
    }

    /// Compress chunk data for memory efficiency
    fn compress_chunk_data(&mut self, data: &[u8]) -> HiveResult<Vec<u8>> {
        if !self.config.enable_compression {
            return Ok(data.to_vec());
        }

        use std::io::Write;
        self.compression_buffer.clear();

        let mut encoder = flate2::write::GzEncoder::new(
            &mut self.compression_buffer,
            flate2::Compression::new(self.config.compression_level),
        );

        encoder
            .write_all(data)
            .map_err(|e| HiveError::ProcessingError {
                reason: format!("Failed to compress chunk data: {}", e),
            })?;

        encoder.finish().map_err(|e| HiveError::ProcessingError {
            reason: format!("Failed to finish compression: {}", e),
        })?;

        Ok(self.compression_buffer.clone())
    }

    /// Get streaming loader performance metrics
    pub fn get_metrics(&self) -> &StreamingLoaderMetrics {
        &self.metrics
    }

    /// Calculate memory efficiency percentage
    pub fn memory_efficiency(&self) -> HiveResult<f64> {
        if self.metrics.memory_usage_peak == 0 {
            return Ok(100.0);
        }

        let current_usage = self
            .memory_pool
            .try_read()
            .map_err(|e| HiveError::ProcessingError {
                reason: format!("Failed to acquire read lock on memory pool: {}", e),
            })?
            .len()
            * self.config.max_chunk_size;
        Ok((1.0 - (current_usage as f64 / self.metrics.memory_usage_peak as f64)) * 100.0)
    }
}

/// Memory-efficient streaming batch for large neural datasets
#[derive(Debug)]
pub struct StreamingBatch {
    pub features: Vec<Vec<f32>>,
    pub labels: Vec<Vec<f32>>,
    pub batch_index: usize,
    pub total_batches: usize,
    pub memory_usage: usize,
    /// Memory pool reference for efficient memory management
    memory_pool: Option<Arc<RwLock<Vec<Vec<f32>>>>>,
}

/// Memory-efficient neural dataset with streaming support
#[derive(Debug)]
pub struct MemoryEfficientDataset {
    /// Dataset metadata
    pub metadata: DatasetMetadata,
    /// Memory pool for feature vectors
    feature_pool: Arc<RwLock<Vec<Vec<f32>>>>,
    /// Memory pool for label vectors
    label_pool: Arc<RwLock<Vec<Vec<f32>>>>,
    /// Current memory usage
    memory_usage: Arc<RwLock<usize>>,
    /// Maximum memory limit
    max_memory_limit: usize,
    /// Performance metrics
    metrics: DatasetMetrics,
}

/// Performance metrics for memory-efficient dataset
#[derive(Debug, Clone, Default)]
pub struct DatasetMetrics {
    pub total_samples_loaded: usize,
    pub memory_efficiency_percentage: f64,
    pub average_load_time: f64,
    pub cache_hit_rate: f64,
    pub compression_ratio: f64,
}

impl StreamingBatch {
    /// Create new streaming batch with memory pool
    pub fn new(
        features: Vec<Vec<f32>>,
        labels: Vec<Vec<f32>>,
        batch_index: usize,
        total_batches: usize,
        memory_pool: Option<Arc<RwLock<Vec<Vec<f32>>>>>,
    ) -> Self {
        let memory_usage = Self::calculate_memory_usage(&features, &labels);

        Self {
            features,
            labels,
            batch_index,
            total_batches,
            memory_usage,
            memory_pool,
        }
    }

    /// Get batch size
    pub fn size(&self) -> usize {
        self.features.len()
    }

    /// Calculate memory efficiency (lower is better)
    pub fn memory_efficiency(&self) -> f64 {
        if self.size() == 0 {
            return 0.0;
        }
        self.memory_usage as f64 / self.size() as f64
    }

    /// Calculate memory usage of features and labels
    fn calculate_memory_usage(features: &[Vec<f32>], labels: &[Vec<f32>]) -> usize {
        let features_size = features.iter().map(|f| f.len() * 4).sum::<usize>();
        let labels_size = labels.iter().map(|l| l.len() * 4).sum::<usize>();
        features_size + labels_size
    }

    /// Optimize memory usage by reusing buffers from pool
    pub async fn optimize_memory(&mut self) -> HiveResult<()> {
        if let Some(pool) = &self.memory_pool {
            let mut pool_write = pool.write().await;

            // Return unused buffers to pool
            for feature_vec in &mut self.features {
                if pool_write.len() < 1000 {
                    // Limit pool size
                    let mut buffer = vec![0.0f32; feature_vec.capacity()];
                    buffer[..feature_vec.len()].copy_from_slice(feature_vec);
                    pool_write.push(buffer);
                }
            }

            for label_vec in &mut self.labels {
                if pool_write.len() < 1000 {
                    let mut buffer = vec![0.0f32; label_vec.capacity()];
                    buffer[..label_vec.len()].copy_from_slice(label_vec);
                    pool_write.push(buffer);
                }
            }
        }

        Ok(())
    }
}

impl MemoryEfficientDataset {
    /// Create new memory-efficient dataset
    pub fn new(metadata: DatasetMetadata, max_memory_limit: usize) -> Self {
        let feature_pool = Arc::new(RwLock::new(Vec::with_capacity(1000)));
        let label_pool = Arc::new(RwLock::new(Vec::with_capacity(1000)));
        let memory_usage = Arc::new(RwLock::new(0));

        Self {
            metadata,
            feature_pool,
            label_pool,
            memory_usage,
            max_memory_limit,
            metrics: DatasetMetrics::default(),
        }
    }

    /// Load dataset sample with memory optimization
    pub async fn load_sample(&mut self, _index: usize) -> HiveResult<(Vec<f32>, Vec<f32>)> {
        let start_time = std::time::Instant::now();

        // Check memory limit
        let current_usage = *self.memory_usage.read().await;
        if current_usage >= self.max_memory_limit {
            self.evict_old_samples().await?;
        }

        // Get or create feature vector from pool
        let feature_vec = self.get_feature_vector_from_pool().await;
        let label_vec = self.get_label_vector_from_pool().await;

        // In a real implementation, this would load actual data
        // For now, simulate loading with memory-efficient allocation
        let feature_size = self.metadata.num_features;
        let label_size = self.metadata.num_classes;

        let features = if feature_vec.len() >= feature_size {
            feature_vec[..feature_size].to_vec()
        } else {
            vec![rand::random::<f32>() * 2.0 - 1.0; feature_size]
        };

        let labels = if label_vec.len() >= label_size {
            label_vec[..label_size].to_vec()
        } else {
            vec![if rand::random::<bool>() { 1.0 } else { 0.0 }; label_size]
        };

        // Update memory usage
        let sample_memory = (features.len() + labels.len()) * 4;
        *self.memory_usage.write().await += sample_memory;

        // Update metrics
        self.metrics.total_samples_loaded += 1;
        self.metrics.average_load_time = (self.metrics.average_load_time
            * (self.metrics.total_samples_loaded - 1) as f64
            + start_time.elapsed().as_secs_f64())
            / self.metrics.total_samples_loaded as f64;

        Ok((features, labels))
    }

    /// Get feature vector from memory pool
    async fn get_feature_vector_from_pool(&self) -> Vec<f32> {
        let mut pool = self.feature_pool.write().await;
        if let Some(vec) = pool.pop() {
            vec
        } else {
            Vec::with_capacity(self.metadata.num_features)
        }
    }

    /// Get label vector from memory pool
    async fn get_label_vector_from_pool(&self) -> Vec<f32> {
        let mut pool = self.label_pool.write().await;
        if let Some(vec) = pool.pop() {
            vec
        } else {
            Vec::with_capacity(self.metadata.num_classes)
        }
    }

    /// Return vectors to memory pool for reuse
    pub async fn return_vectors_to_pool(&self, features: Vec<f32>, labels: Vec<f32>) {
        let mut feature_pool = self.feature_pool.write().await;
        let mut label_pool = self.label_pool.write().await;

        if feature_pool.len() < 1000 {
            feature_pool.push(features);
        }

        if label_pool.len() < 1000 {
            label_pool.push(labels);
        }
    }

    /// Evict old samples to free memory
    async fn evict_old_samples(&self) -> HiveResult<()> {
        let mut feature_pool = self.feature_pool.write().await;
        let mut label_pool = self.label_pool.write().await;

        // Remove half of the pooled vectors to free memory
        let feature_evict_count = feature_pool.len() / 2;
        let label_evict_count = label_pool.len() / 2;

        feature_pool.drain(0..feature_evict_count);
        label_pool.drain(0..label_evict_count);

        // Update memory usage estimate
        let evicted_memory =
            (feature_evict_count + label_evict_count) * self.metadata.num_features * 4;
        let mut memory_usage = self.memory_usage.write().await;
        *memory_usage = memory_usage.saturating_sub(evicted_memory);

        tracing::debug!(
            "Evicted {} samples to free memory",
            feature_evict_count + label_evict_count
        );

        Ok(())
    }

    /// Get current memory usage
    pub async fn get_memory_usage(&self) -> usize {
        *self.memory_usage.read().await
    }

    /// Get dataset performance metrics
    pub fn get_metrics(&self) -> &DatasetMetrics {
        &self.metrics
    }

    /// Calculate memory efficiency
    pub async fn memory_efficiency(&self) -> f64 {
        let current_usage = self.get_memory_usage().await;
        if self.max_memory_limit == 0 {
            return 100.0;
        }

        (1.0 - (current_usage as f64 / self.max_memory_limit as f64)) * 100.0
    }

    /// Create streaming batch from dataset
    pub async fn create_streaming_batch(
        &mut self,
        batch_size: usize,
        batch_index: usize,
        total_batches: usize,
    ) -> HiveResult<StreamingBatch> {
        let mut features = Vec::with_capacity(batch_size);
        let mut labels = Vec::with_capacity(batch_size);

        for _ in 0..batch_size {
            let (feature_vec, label_vec) = self.load_sample(batch_index * batch_size).await?;
            features.push(feature_vec);
            labels.push(label_vec);
        }

        let memory_pool = Some(Arc::clone(&self.feature_pool));

        Ok(StreamingBatch::new(
            features,
            labels,
            batch_index,
            total_batches,
            memory_pool,
        ))
    }
}

impl Default for DataPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::unused_self)]
impl DataPipeline {
    /// Create a new data pipeline
    #[must_use]
    pub fn new() -> Self {
        Self {
            cpu_optimizer: CpuOptimizer::new(),
            datasets: HashMap::new(),
            data_loaders: HashMap::new(),
            preprocessing: HashMap::new(),
            stream_processor: None,
            neural_stream: None,
        }
    }

    /// Create a new data pipeline with streaming support
    pub fn new_with_streaming(stream_config: StreamConfig) -> Self {
        let stream_processor = StreamProcessor::new(stream_config.clone());
        let neural_stream = NeuralDataStream::new(stream_config);

        Self {
            cpu_optimizer: CpuOptimizer::new(),
            datasets: HashMap::new(),
            data_loaders: HashMap::new(),
            preprocessing: HashMap::new(),
            stream_processor: Some(stream_processor),
            neural_stream: Some(neural_stream),
        }
    }

    /// Load dataset from file
    pub async fn load_dataset(
        &mut self,
        name: &str,
        path: &Path,
        data_type: DataType,
    ) -> HiveResult<()> {
        tracing::info!("📊 Loading dataset '{}' from {:?}", name, path);

        // In a real implementation, this would read from various file formats
        // For now, we'll create a mock dataset
        let dataset = self.create_mock_dataset(name, data_type).await?;

        self.datasets
            .insert(name.to_string(), Arc::new(RwLock::new(dataset)));
        tracing::info!("✅ Dataset '{}' loaded successfully", name);
        Ok(())
    }

    /// Load large dataset using streaming for memory efficiency
    pub async fn load_dataset_streaming(
        &mut self,
        name: &str,
        path: &Path,
        data_type: DataType,
    ) -> HiveResult<String> {
        tracing::info!("🔄 Loading large dataset '{}' using streaming", name);

        let stream_processor = self.stream_processor.as_ref().ok_or_else(|| {
            crate::utils::error::HiveError::ProcessingError {
                reason: "Streaming not enabled for this data pipeline".to_string(),
            }
        })?;

        // Create streaming data loader
        let loader_id = format!("{}_streaming", name);
        let _streaming_loader =
            StreamingDataLoader::new(path.to_path_buf(), stream_processor.config.clone());

        // Store the streaming loader
        self.data_loaders.insert(
            loader_id.clone(),
            Arc::new(RwLock::new(DataLoader {
                dataset: Arc::new(RwLock::new(Dataset {
                    name: name.to_string(),
                    features: Vec::new(), // Will be loaded on-demand
                    labels: Vec::new(),
                    metadata: DatasetMetadata {
                        num_samples: 0, // Will be determined during streaming
                        num_features: 0,
                        num_classes: 0,
                        feature_names: Vec::new(),
                        class_names: Vec::new(),
                        data_type,
                    },
                })),
                batch_size: 32, // Default batch size
                shuffle: false,
                current_index: 0,
                indices: Vec::new(),
            })),
        );

        tracing::info!("✅ Streaming dataset loader '{}' created", loader_id);
        Ok(loader_id)
    }

    /// Get next streaming batch from data loader
    pub async fn get_next_streaming_batch(
        &self,
        loader_id: &str,
    ) -> HiveResult<Option<StreamingBatch>> {
        let loader =
            self.data_loaders
                .get(loader_id)
                .ok_or_else(|| HiveError::ProcessingError {
                    reason: format!("Data loader '{}' not found", loader_id),
                })?;

        let mut loader = loader.write().await;

        // For streaming, we'll simulate loading from disk/network
        // In a real implementation, this would read from the actual data source
        if loader.current_index >= 1000 {
            // Simulate 1000 batches
            return Ok(None);
        }

        // Generate streaming batch data
        let batch_size = loader.batch_size;
        let mut features = Vec::new();
        let mut labels = Vec::new();

        for _ in 0..batch_size {
            // Simulate feature data (in real implementation, read from file/stream)
            let feature = vec![rand::random::<f32>() * 2.0 - 1.0; 784]; // MNIST-like
            let label = vec![if rand::random::<bool>() { 1.0 } else { 0.0 }]; // Binary classification

            features.push(feature);
            labels.push(label);
        }

        let memory_usage = self.estimate_batch_memory(&features, &labels);

        let batch = StreamingBatch {
            features,
            labels,
            batch_index: loader.current_index,
            total_batches: 1000,
            memory_usage,
            memory_pool: None,
        };

        loader.current_index += 1;

        Ok(Some(batch))
    }

    /// Estimate memory usage of a batch
    fn estimate_batch_memory(&self, features: &[Vec<f32>], labels: &[Vec<f32>]) -> usize {
        let features_size = features.iter().map(|f| f.len() * 4).sum::<usize>();
        let labels_size = labels.iter().map(|l| l.len() * 4).sum::<usize>();
        features_size + labels_size
    }

    /// Create data loader for a dataset
    pub async fn create_data_loader(
        &mut self,
        dataset_name: &str,
        batch_size: usize,
        shuffle: bool,
    ) -> HiveResult<String> {
        let dataset = Arc::clone(self.datasets.get(dataset_name).ok_or_else(|| {
            HiveError::ProcessingError {
                reason: format!("Dataset '{}' not found", dataset_name),
            }
        })?);

        let num_samples = dataset.read().await.features.len();
        let indices: Vec<usize> = (0..num_samples).collect();

        let loader = DataLoader {
            dataset,
            batch_size,
            shuffle,
            current_index: 0,
            indices,
        };

        let loader_id = format!("{dataset_name}_loader");
        self.data_loaders
            .insert(loader_id.clone(), Arc::new(RwLock::new(loader)));

        tracing::info!(
            "🔄 Created data loader '{}' with batch size {}",
            loader_id,
            batch_size
        );
        Ok(loader_id)
    }

    /// Get next batch from data loader
    pub async fn get_next_batch(&self, loader_id: &str) -> HiveResult<Option<DataBatch>> {
        let loader =
            self.data_loaders
                .get(loader_id)
                .ok_or_else(|| HiveError::ProcessingError {
                    reason: format!("Data loader '{}' not found", loader_id),
                })?;

        let mut loader = loader.write().await;

        if loader.current_index >= loader.indices.len() {
            return Ok(None); // No more batches
        }

        let start_idx = loader.current_index;
        let end_idx = (start_idx + loader.batch_size).min(loader.indices.len());
        let batch_indices: Vec<usize> = loader.indices[start_idx..end_idx].to_vec();

        // Extract batch data - read dataset separately to avoid borrow conflict
        let (batch_features, batch_labels, sample_indices) = {
            let dataset = loader.dataset.read().await;
            let mut batch_features = Vec::new();
            let mut batch_labels = Vec::new();
            let mut sample_indices = Vec::new();

            for &idx in &batch_indices {
                batch_features.push(dataset.features[idx].clone());
                batch_labels.push(dataset.labels[idx].clone());
                sample_indices.push(idx);
            }
            (batch_features, batch_labels, sample_indices)
        };

        let total_batches = loader.indices.len().div_ceil(loader.batch_size);
        let batch_index = loader.current_index / loader.batch_size;

        loader.current_index = end_idx;

        // Shuffle if requested and this is the end of an epoch
        if loader.shuffle && loader.current_index >= loader.indices.len() {
            self.shuffle_indices(&mut loader.indices).await?;
            loader.current_index = 0;
        }

        let batch = DataBatch {
            features: batch_features,
            labels: batch_labels,
            batch_size: batch_indices.len(),
            metadata: BatchMetadata {
                batch_index,
                total_batches,
                sample_indices,
            },
        };

        Ok(Some(batch))
    }

    /// Convert regular batch to streaming batch
    pub fn to_streaming_batch(&self, batch: DataBatch) -> StreamingBatch {
        let memory_usage = self.estimate_batch_memory(&batch.features, &batch.labels);

        StreamingBatch {
            memory_pool: Some(Arc::new(RwLock::new(Vec::new()))),
            features: batch.features,
            labels: batch.labels,
            batch_index: batch.metadata.batch_index,
            total_batches: batch.metadata.total_batches,
            memory_usage,
        }
    }

    /// Apply preprocessing pipeline to dataset
    pub async fn apply_preprocessing(
        &self,
        dataset_name: &str,
        pipeline: &PreprocessingPipeline,
    ) -> HiveResult<()> {
        let dataset =
            self.datasets
                .get(dataset_name)
                .ok_or_else(|| HiveError::ProcessingError {
                    reason: format!("Dataset '{}' not found", dataset_name),
                })?;

        let mut dataset = dataset.write().await;

        tracing::info!(
            "🔧 Applying preprocessing pipeline to dataset '{}'",
            dataset_name
        );

        for step in &pipeline.steps {
            match step {
                PreprocessingStep::Normalization(config) => {
                    self.apply_normalization(&mut dataset, config).await?;
                }
                PreprocessingStep::Standardization(config) => {
                    self.apply_standardization(&mut dataset, config).await?;
                }
                PreprocessingStep::Encoding(config) => {
                    self.apply_encoding(&mut dataset, config).await?;
                }
                PreprocessingStep::Augmentation(config) => {
                    self.apply_augmentation(&mut dataset, config).await?;
                }
                PreprocessingStep::FeatureSelection(config) => {
                    self.apply_feature_selection(&mut dataset, config).await?;
                }
            }
        }

        tracing::info!("✅ Preprocessing pipeline applied successfully");
        Ok(())
    }

    /// Split dataset into train/validation/test sets
    pub async fn split_dataset(
        &self,
        dataset_name: &str,
        split: &DataSplit,
    ) -> HiveResult<DataSplits> {
        let dataset = self
            .datasets
            .get(dataset_name)
            .ok_or_else(|| HiveError::ProcessingError {
                reason: format!("Dataset '{}' not found", dataset_name),
            })?
            .read()
            .await;

        let total_samples = dataset.features.len();
        let train_size = (total_samples as f64 * split.train_ratio) as usize;
        let val_size = (total_samples as f64 * split.val_ratio) as usize;
        #[allow(clippy::no_effect_underscore_binding)]
        let _test_size = total_samples - train_size - val_size;

        // Create indices for each split
        let mut indices: Vec<usize> = (0..total_samples).collect();
        if split.stratify {
            // In a real implementation, this would stratify by class labels
            self.shuffle_indices(&mut indices).await?;
        }

        let train_indices = indices[0..train_size].to_vec();
        let val_indices = indices[train_size..train_size + val_size].to_vec();
        let test_indices = indices[train_size + val_size..].to_vec();

        // Create split datasets
        let train_dataset = self.create_split_dataset(&dataset, &train_indices, "train");
        let val_dataset = self.create_split_dataset(&dataset, &val_indices, "validation");
        let test_dataset = self.create_split_dataset(&dataset, &test_indices, "test");

        Ok(DataSplits {
            train: train_dataset,
            validation: val_dataset,
            test: test_dataset,
        })
    }

    /// Create mock dataset for testing
    async fn create_mock_dataset(&self, name: &str, data_type: DataType) -> HiveResult<Dataset> {
        // Create a simple mock dataset
        let num_samples = 1000;
        let num_features = 10;
        let num_classes = 2;

        let mut features = Vec::new();
        let mut labels = Vec::new();

        for _ in 0..num_samples {
            let mut sample_features = Vec::new();
            for _ in 0..num_features {
                sample_features.push(rand::random::<f32>() * 2.0 - 1.0);
            }

            // Simple classification based on first feature
            let label = if sample_features[0] > 0.0 { 1.0 } else { 0.0 };
            labels.push(vec![label]);

            features.push(sample_features);
        }

        let metadata = DatasetMetadata {
            num_samples,
            num_features,
            num_classes,
            feature_names: (0..num_features).map(|i| format!("feature_{i}")).collect(),
            class_names: (0..num_classes).map(|i| format!("class_{i}")).collect(),
            data_type,
        };

        Ok(Dataset {
            name: name.to_string(),
            features,
            labels,
            metadata,
        })
    }

    /// Shuffle indices for data randomization
    async fn shuffle_indices(&self, indices: &mut [usize]) -> HiveResult<()> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        indices.shuffle(&mut rng);
        Ok(())
    }

    /// Apply normalization preprocessing
    async fn apply_normalization(
        &self,
        dataset: &mut Dataset,
        config: &NormalizationConfig,
    ) -> HiveResult<()> {
        tracing::info!("📏 Applying normalization: {:?}", config.method);

        match config.method {
            NormalizationMethod::MinMax => {
                self.apply_minmax_normalization(dataset, config.feature_range)
                    .await?;
            }
            NormalizationMethod::L1 => {
                self.apply_l1_normalization(dataset).await?;
            }
            NormalizationMethod::L2 => {
                self.apply_l2_normalization(dataset).await?;
            }
            NormalizationMethod::ZScore => {
                self.apply_zscore_normalization(dataset).await?;
            }
        }

        Ok(())
    }

    /// Apply Min-Max normalization
    async fn apply_minmax_normalization(
        &self,
        dataset: &mut Dataset,
        feature_range: Option<(f32, f32)>,
    ) -> HiveResult<()> {
        let (min_val, max_val) = feature_range.unwrap_or((0.0, 1.0));

        for feature_idx in 0..dataset.metadata.num_features {
            let feature_values: Vec<f32> = dataset
                .features
                .iter()
                .map(|sample| sample[feature_idx])
                .collect();

            let feature_min = feature_values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let feature_max = feature_values
                .iter()
                .fold(f32::NEG_INFINITY, |a, &b| a.max(b));

            if (feature_max - feature_min).abs() < 1e-6 {
                continue; // Skip constant features
            }

            for sample in &mut dataset.features {
                let normalized = min_val
                    + (sample[feature_idx] - feature_min) * (max_val - min_val)
                        / (feature_max - feature_min);
                sample[feature_idx] = normalized;
            }
        }

        Ok(())
    }

    /// Apply L1 normalization
    async fn apply_l1_normalization(&self, dataset: &mut Dataset) -> HiveResult<()> {
        for sample in &mut dataset.features {
            let l1_norm: f32 = sample.iter().map(|x| x.abs()).sum();
            if l1_norm > 0.0 {
                for feature in sample {
                    *feature /= l1_norm;
                }
            }
        }
        Ok(())
    }

    /// Apply L2 normalization
    async fn apply_l2_normalization(&self, dataset: &mut Dataset) -> HiveResult<()> {
        for sample in &mut dataset.features {
            let l2_norm = VectorizedOps::vector_norm(&sample.clone());
            if l2_norm > 0.0 {
                for feature in sample {
                    *feature /= l2_norm;
                }
            }
        }
        Ok(())
    }

    /// Apply Z-score normalization
    async fn apply_zscore_normalization(&self, dataset: &mut Dataset) -> HiveResult<()> {
        for feature_idx in 0..dataset.metadata.num_features {
            let feature_values: Vec<f32> = dataset
                .features
                .iter()
                .map(|sample| sample[feature_idx])
                .collect();

            let mean = feature_values.iter().sum::<f32>() / feature_values.len() as f32;
            let variance = feature_values
                .iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f32>()
                / feature_values.len() as f32;
            let std = variance.sqrt();

            if std > 0.0 {
                for sample in &mut dataset.features {
                    sample[feature_idx] = (sample[feature_idx] - mean) / std;
                }
            }
        }
        Ok(())
    }

    /// Apply standardization preprocessing
    async fn apply_standardization(
        &self,
        dataset: &mut Dataset,
        config: &StandardizationConfig,
    ) -> HiveResult<()> {
        tracing::info!("📊 Applying standardization");

        for feature_idx in 0..dataset.metadata.num_features {
            let feature_values: Vec<f32> = dataset
                .features
                .iter()
                .map(|sample| sample[feature_idx])
                .collect();

            if config.with_mean {
                let mean = feature_values.iter().sum::<f32>() / feature_values.len() as f32;
                for sample in &mut dataset.features {
                    sample[feature_idx] -= mean;
                }
            }

            if config.with_std {
                let variance = feature_values.iter().map(|x| x.powi(2)).sum::<f32>()
                    / feature_values.len() as f32;
                let std = variance.sqrt();

                if std > 0.0 {
                    for sample in &mut dataset.features {
                        sample[feature_idx] /= std;
                    }
                }
            }
        }

        Ok(())
    }

    /// Apply encoding preprocessing
    async fn apply_encoding(
        &self,
        _dataset: &mut Dataset,
        config: &EncodingConfig,
    ) -> HiveResult<()> {
        tracing::info!("🔢 Applying encoding: {:?}", config.method);

        match config.method {
            EncodingMethod::OneHot => {
                // One-hot encoding would require categorical features
                tracing::warn!("One-hot encoding requires categorical features");
            }
            EncodingMethod::Label => {
                // Label encoding for categorical features
                tracing::warn!("Label encoding requires categorical features");
            }
            EncodingMethod::Ordinal => {
                // Ordinal encoding for ordered categorical features
                tracing::warn!("Ordinal encoding requires categorical features");
            }
            EncodingMethod::Binary => {
                // Binary encoding for high-cardinality categorical features
                tracing::warn!("Binary encoding requires categorical features");
            }
        }

        Ok(())
    }

    /// Apply augmentation preprocessing
    async fn apply_augmentation(
        &self,
        dataset: &mut Dataset,
        config: &AugmentationConfig,
    ) -> HiveResult<()> {
        tracing::info!("🎨 Applying data augmentation");

        let mut augmented_features = Vec::new();
        let mut augmented_labels = Vec::new();

        for (features, labels) in dataset.features.iter().zip(&dataset.labels) {
            augmented_features.push(features.clone());
            augmented_labels.push(labels.clone());

            // Apply augmentation with given probability
            if rand::random::<f64>() < config.probability {
                for technique in &config.techniques {
                    if let AugmentationTechnique::Noise { std } = technique {
                        let mut augmented = features.clone();
                        for feature in &mut augmented {
                            *feature += rand::random::<f32>() * *std;
                        }
                        augmented_features.push(augmented);
                        augmented_labels.push(labels.clone());
                    } else {
                        // Other augmentation techniques would be implemented for specific data types
                    }
                }
            }
        }

        dataset.features = augmented_features;
        dataset.labels = augmented_labels;
        dataset.metadata.num_samples = dataset.features.len();

        Ok(())
    }

    /// Apply feature selection preprocessing
    async fn apply_feature_selection(
        &self,
        dataset: &mut Dataset,
        config: &FeatureSelectionConfig,
    ) -> HiveResult<()> {
        tracing::info!("🎯 Applying feature selection: k={}", config.k);

        match config.method {
            FeatureSelectionMethod::VarianceThreshold { threshold } => {
                self.apply_variance_threshold_selection(dataset, threshold)
                    .await?;
            }
            FeatureSelectionMethod::SelectKBest { .. } => {
                self.apply_select_k_best_selection(dataset, config.k)
                    .await?;
            }
            FeatureSelectionMethod::RecursiveFeatureElimination => {
                self.apply_rfe_selection(dataset, config.k).await?;
            }
        }

        Ok(())
    }

    /// Apply variance threshold feature selection
    async fn apply_variance_threshold_selection(
        &self,
        dataset: &mut Dataset,
        threshold: f64,
    ) -> HiveResult<()> {
        let mut selected_indices = Vec::new();

        for feature_idx in 0..dataset.metadata.num_features {
            let feature_values: Vec<f32> = dataset
                .features
                .iter()
                .map(|sample| sample[feature_idx])
                .collect();

            let mean = feature_values.iter().sum::<f32>() / feature_values.len() as f32;
            let variance = feature_values
                .iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f32>()
                / feature_values.len() as f32;

            if variance >= threshold as f32 {
                selected_indices.push(feature_idx);
            }
        }

        self.select_features(dataset, &selected_indices).await?;
        Ok(())
    }

    /// Apply select k best feature selection
    async fn apply_select_k_best_selection(
        &self,
        dataset: &mut Dataset,
        k: usize,
    ) -> HiveResult<()> {
        // Simple implementation: select features with highest variance
        let mut feature_variances = Vec::new();

        for feature_idx in 0..dataset.metadata.num_features {
            let feature_values: Vec<f32> = dataset
                .features
                .iter()
                .map(|sample| sample[feature_idx])
                .collect();

            let mean = feature_values.iter().sum::<f32>() / feature_values.len() as f32;
            let variance = feature_values
                .iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f32>()
                / feature_values.len() as f32;

            feature_variances.push((feature_idx, variance));
        }

        feature_variances
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let selected_indices: Vec<usize> = feature_variances
            .into_iter()
            .take(k)
            .map(|(idx, _)| idx)
            .collect();

        self.select_features(dataset, &selected_indices).await?;
        Ok(())
    }

    /// Apply recursive feature elimination
    async fn apply_rfe_selection(&self, dataset: &mut Dataset, k: usize) -> HiveResult<()> {
        // Simplified RFE: iteratively remove least important features
        let mut remaining_features: Vec<usize> = (0..dataset.metadata.num_features).collect();

        while remaining_features.len() > k {
            // Simple heuristic: remove feature with lowest variance
            let mut feature_variances = Vec::new();

            for &feature_idx in &remaining_features {
                let feature_values: Vec<f32> = dataset
                    .features
                    .iter()
                    .map(|sample| sample[feature_idx])
                    .collect();

                let mean = feature_values.iter().sum::<f32>() / feature_values.len() as f32;
                let variance = feature_values
                    .iter()
                    .map(|x| (x - mean).powi(2))
                    .sum::<f32>()
                    / feature_values.len() as f32;

                feature_variances.push((feature_idx, variance));
            }

            feature_variances
                .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            if let Some((worst_feature, _)) = feature_variances.first() {
                remaining_features.retain(|&x| x != *worst_feature);
            }
        }

        self.select_features(dataset, &remaining_features).await?;
        Ok(())
    }

    /// Select specific features from dataset
    async fn select_features(
        &self,
        dataset: &mut Dataset,
        selected_indices: &[usize],
    ) -> HiveResult<()> {
        for sample in &mut dataset.features {
            let selected_features: Vec<f32> =
                selected_indices.iter().map(|&idx| sample[idx]).collect();
            *sample = selected_features;
        }

        dataset.metadata.num_features = selected_indices.len();
        dataset.metadata.feature_names = selected_indices
            .iter()
            .map(|&idx| dataset.metadata.feature_names[idx].clone())
            .collect();

        Ok(())
    }

    /// Create dataset split
    fn create_split_dataset(
        &self,
        original: &Dataset,
        indices: &[usize],
        split_name: &str,
    ) -> Dataset {
        let mut features = Vec::new();
        let mut labels = Vec::new();

        for &idx in indices {
            features.push(original.features[idx].clone());
            labels.push(original.labels[idx].clone());
        }

        let mut metadata = original.metadata.clone();
        metadata.num_samples = indices.len();

        Dataset {
            name: format!("{}_{}", original.name, split_name),
            features,
            labels,
            metadata,
        }
    }
}

impl Default for DataSplit {
    fn default() -> Self {
        Self {
            train_ratio: 0.7,
            val_ratio: 0.15,
            test_ratio: 0.15,
            stratify: false,
        }
    }
}
