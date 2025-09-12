//! Streaming Infrastructure for Large Data Processing
//!
//! This module provides streaming capabilities for handling large datasets
//! and neural processing operations without loading everything into memory.

use crate::utils::error::{HiveError, HiveResult};
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::{error, info};

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
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 8192,
            max_chunk_size: 1024 * 1024, // 1MB
            timeout: std::time::Duration::from_secs(30),
            enable_compression: false,
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
    pub metadata: std::collections::HashMap<String, String>,
    /// Checksum for data integrity
    pub checksum: Option<String>,
}

impl DataChunk {
    /// Create a new data chunk
    pub fn new(sequence: u64, data: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            sequence,
            total_chunks: None,
            data,
            metadata: std::collections::HashMap::new(),
            checksum: None,
        }
    }

    /// Add metadata to the chunk
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Calculate and set checksum
    pub fn with_checksum(mut self) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.data);
        self.checksum = Some(format!("{:x}", hasher.finalize()));
        self
    }

    /// Verify checksum
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
}

/// Codec for encoding/decoding data chunks
pub struct DataChunkCodec {
    max_chunk_size: usize,
}

impl DataChunkCodec {
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
                reason: format!("Failed to deserialize chunk: {}", e),
            }),
        }
    }
}

impl Encoder<DataChunk> for DataChunkCodec {
    type Error = HiveError;

    fn encode(&mut self, item: DataChunk, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let serialized = bincode::serialize(&item).map_err(|e| HiveError::MessageParsingError {
            reason: format!("Failed to serialize chunk: {}", e),
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
pub struct StreamProcessor {
    config: StreamConfig,
}

impl StreamProcessor {
    /// Create a new stream processor
    pub fn new(config: StreamConfig) -> Self {
        Self { config }
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

/// Neural data stream for processing large neural network datasets
pub struct NeuralDataStream {
    processor: StreamProcessor,
}

impl NeuralDataStream {
    /// Create a new neural data stream
    pub fn new(config: StreamConfig) -> Self {
        Self {
            processor: StreamProcessor::new(config),
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
                            reason: format!("Failed to deserialize training data: {}", e),
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
                reason: format!("Failed to serialize weights: {}", e),
            })?;

        Ok(self.processor.create_stream_from_data(serialized))
    }
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
    pub metadata: std::collections::HashMap<String, String>,
}

/// Memory-efficient data iterator
pub struct MemoryEfficientIterator<T> {
    data: Vec<T>,
    chunk_size: usize,
    current_index: usize,
}

impl<T> MemoryEfficientIterator<T> {
    /// Create a new memory-efficient iterator
    pub fn new(data: Vec<T>, chunk_size: usize) -> Self {
        Self {
            data,
            chunk_size,
            current_index: 0,
        }
    }
}

impl<T: Clone> Iterator for MemoryEfficientIterator<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.data.len() {
            return None;
        }

        let end_index = std::cmp::min(self.current_index + self.chunk_size, self.data.len());
        let chunk = self.data[self.current_index..end_index].to_vec();
        self.current_index = end_index;

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
}
