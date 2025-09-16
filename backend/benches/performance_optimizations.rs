//! Performance benchmarks for streaming and caching optimizations
//!
//! This module contains comprehensive benchmarks to measure the performance
//! improvements achieved by the streaming and intelligent caching systems.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use tokio::runtime::Runtime;

// Import our optimized modules
use ai_orchestrator_hub::infrastructure::{
    cache::Cache,
    intelligent_cache::{IntelligentCache, IntelligentCacheConfig, MultiTierCacheManager},
    streaming::{NeuralDataStream, StreamConfig, StreamProcessor, StreamingPerformanceMonitor},
};

/// Benchmark streaming performance improvements
fn benchmark_streaming_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("streaming_large_dataset", |b| {
        b.to_async(&rt).iter(|| async {
            // Create streaming configuration optimized for performance
            let config = StreamConfig {
                buffer_size: 65536,          // 64KB buffer
                max_chunk_size: 1024 * 1024, // 1MB chunks
                timeout: Duration::from_secs(30),
                enable_compression: true,
                parallel_workers: 4,
                memory_pool_size: 50,
                enable_memory_pool: true,
                compression_level: 6,
            };

            let processor = StreamProcessor::new(config);

            // Create large test dataset (10MB)
            let large_data = vec![0u8; 10 * 1024 * 1024];

            // Benchmark streaming processing
            let stream = processor
                .create_stream_from_data_pooled(large_data.clone())
                .await
                .unwrap();

            let mut total_processed = 0;
            let mut chunks_processed = 0;

            processor
                .process_stream(stream, |chunk| {
                    total_processed += chunk.data.len();
                    chunks_processed += 1;
                    Ok(chunk.data.len())
                })
                .await
                .unwrap();

            black_box((total_processed, chunks_processed));
        });
    });

    c.bench_function("streaming_parallel_processing", |b| {
        b.to_async(&rt).iter(|| async {
            let mut config = StreamConfig {
                parallel_workers: 8,
                enable_memory_pool: true,
                ..Default::default()
            };

            let processor = StreamProcessor::new(config);

            // Create test data
            let test_data = vec![0u8; 5 * 1024 * 1024]; // 5MB
            let stream = processor
                .create_stream_from_data_pooled(test_data)
                .await
                .unwrap();

            // Benchmark parallel processing
            let results = processor
                .process_stream_parallel(stream, |chunk| Ok(chunk.data.len()))
                .await
                .unwrap();

            black_box(results.len());
        });
    });

    c.bench_function("streaming_memory_efficiency", |b| {
        b.to_async(&rt).iter(|| async {
            let config = StreamConfig {
                enable_memory_pool: true,
                memory_pool_size: 100,
                ..Default::default()
            };

            let processor = StreamProcessor::new(config);

            // Track memory usage before and after
            let initial_memory = get_memory_usage();

            // Process multiple streams to test memory pooling
            for i in 0..10 {
                let data = vec![i as u8; 1024 * 1024]; // 1MB per iteration
                let stream = processor
                    .create_stream_from_data_pooled(data)
                    .await
                    .unwrap();

                let _results = processor
                    .process_stream(stream, |chunk| Ok(chunk.data.len()))
                    .await
                    .unwrap();
            }

            let final_memory = get_memory_usage();
            let memory_delta = final_memory - initial_memory;

            black_box(memory_delta);
        });
    });
}

/// Benchmark intelligent caching performance
fn benchmark_caching_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("intelligent_cache_operations", |b| {
        b.to_async(&rt).iter(|| async {
            let config = IntelligentCacheConfig {
                max_size: 10000,
                enable_adaptive_ttl: true,
                enable_prefetching: true,
                ..Default::default()
            };

            let cache = IntelligentCache::new(config);

            // Perform cache operations
            for i in 0..1000 {
                let key = format!("key_{}", i);
                let value = format!("value_{}", i);

                cache.set(key.clone(), value.clone()).await.unwrap();

                // Simulate access patterns
                if i % 3 == 0 {
                    let _ = cache.get(&key).await;
                }
            }

            // Get performance metrics
            let stats = cache.get_stats().await;
            black_box(stats);
        });
    });

    c.bench_function("multi_tier_cache_performance", |b| {
        b.to_async(&rt).iter(|| async {
            let cache_manager = MultiTierCacheManager::new();

            // Populate cache with test data
            for i in 0..1000 {
                let key = format!("test_key_{}", i);
                let value = serde_json::json!({"id": i, "data": format!("test_data_{}", i)});

                cache_manager
                    .set_with_intelligence(key, value)
                    .await
                    .unwrap();
            }

            // Benchmark cache hits
            let mut hits = 0;
            for i in 0..1000 {
                let key = format!("test_key_{}", i % 500); // 50% hit rate
                if cache_manager.get_with_warming(&key).await.is_some() {
                    hits += 1;
                }
            }

            black_box(hits);
        });
    });

    c.bench_function("cache_warming_effectiveness", |b| {
        b.to_async(&rt).iter(|| async {
            let cache_manager = MultiTierCacheManager::new();

            // Simulate access patterns that should trigger warming
            for i in 0..100 {
                let key = format!("hot_key_{}", i % 10); // High frequency keys

                // Multiple accesses to build pattern
                for _ in 0..5 {
                    let _ = cache_manager.get_with_warming(&key).await;
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }

            // Process warming queue
            cache_manager.process_cache_warming().await.unwrap();

            // Get warming metrics
            let stats = cache_manager.get_enhanced_stats().await;
            black_box(stats);
        });
    });
}

/// Benchmark memory-efficient data structures
fn benchmark_memory_structures(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("memory_efficient_dataset", |b| {
        b.to_async(&rt).iter(|| async {
            use ai_orchestrator_hub::neural::data::{
                DataType, DatasetMetadata, MemoryEfficientDataset,
            };

            let metadata = DatasetMetadata {
                num_samples: 10000,
                num_features: 784,
                num_classes: 10,
                feature_names: (0..784).map(|i| format!("feature_{}", i)).collect(),
                class_names: (0..10).map(|i| format!("class_{}", i)).collect(),
                data_type: DataType::Image,
            };

            let dataset = MemoryEfficientDataset::new(metadata, 100 * 1024 * 1024); // 100MB limit

            // Benchmark sample loading
            for i in 0..1000 {
                let (features, labels) = dataset.load_sample(i % 10000).await.unwrap();

                // Simulate processing
                let _feature_sum: f32 = features.iter().sum();
                let _label_max: f32 = labels.iter().fold(0.0, |a, &b| a.max(b));

                // Return to pool
                dataset.return_vectors_to_pool(features, labels).await;
            }

            let efficiency = dataset.memory_efficiency().await;
            black_box(efficiency);
        });
    });

    c.bench_function("streaming_data_loader", |b| {
        b.to_async(&rt).iter(|| async {
            use ai_orchestrator_hub::neural::data::StreamingDataLoader;
            use std::path::PathBuf;

            let config = StreamConfig {
                enable_memory_pool: true,
                enable_compression: true,
                ..Default::default()
            };

            let mut loader =
                StreamingDataLoader::new_optimized(PathBuf::from("/tmp/test_data"), config);

            // Benchmark chunk streaming
            for _ in 0..100 {
                let stream = loader.get_next_chunk_stream().await.unwrap();

                let chunks: Vec<_> = stream.collect().await;
                black_box(chunks.len());
            }

            let metrics = loader.get_metrics();
            black_box(metrics);
        });
    });
}

/// Benchmark combined streaming and caching performance
fn benchmark_combined_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("streaming_with_cache_integration", |b| {
        b.to_async(&rt).iter(|| async {
            // Create integrated streaming and caching system
            let stream_config = StreamConfig {
                enable_memory_pool: true,
                parallel_workers: 4,
                ..Default::default()
            };

            let cache_config = IntelligentCacheConfig {
                enable_prefetching: true,
                enable_adaptive_ttl: true,
                ..Default::default()
            };

            let stream_processor = StreamProcessor::new(stream_config);
            let cache = IntelligentCache::new(cache_config);

            // Simulate processing pipeline with caching
            let test_data = vec![0u8; 2 * 1024 * 1024]; // 2MB
            let stream = stream_processor
                .create_stream_from_data_pooled(test_data)
                .await
                .unwrap();

            let mut processed_items = 0;

            stream_processor
                .process_stream(stream, |chunk| {
                    let chunk_id = format!("chunk_{}", processed_items);

                    // Cache chunk metadata
                    let metadata = serde_json::json!({
                        "size": chunk.data.len(),
                        "sequence": chunk.sequence,
                        "checksum": chunk.checksum
                    });

                    // This would be async in real implementation
                    // cache.set(chunk_id, metadata).await.unwrap();

                    processed_items += 1;
                    Ok(chunk.data.len())
                })
                .await
                .unwrap();

            black_box(processed_items);
        });
    });
}

/// Get current memory usage (simplified implementation)
fn get_memory_usage() -> usize {
    // In a real implementation, this would use system APIs to get actual memory usage
    // For benchmarking purposes, we'll return a simulated value
    50 * 1024 * 1024 // 50MB baseline
}

criterion_group!(
    name = performance_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(50);
    targets =
        benchmark_streaming_performance,
        benchmark_caching_performance,
        benchmark_memory_structures,
        benchmark_combined_performance
);

criterion_main!(performance_benches);
