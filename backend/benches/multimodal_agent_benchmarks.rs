//! # Multi-Modal Agent Performance Benchmarks
//!
//! Comprehensive benchmarks to measure the performance of the Multi-Modal Agent
//! across different data types, sizes, and complexity levels.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use multiagent_hive::agents::{DataModality, MultiModalAgent};
use multiagent_hive::neural::NLPProcessor;
use std::time::Duration;
use tokio::runtime::Runtime;

// Test data generators
struct BenchmarkDataGenerator;

impl BenchmarkDataGenerator {
    fn generate_text_data(size: usize) -> String {
        let base_text = "This is a comprehensive analysis of system performance and user satisfaction metrics. The implementation demonstrates excellent scalability and maintainability characteristics.";
        (0..size)
            .map(|i| format!("{} Sample text block {}: {}", base_text, i, base_text))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn generate_code_data(complexity: &str) -> String {
        match complexity {
            "simple" => r#"
                function calculateSum(a, b) {
                    return a + b;
                }
                
                const result = calculateSum(5, 10);
                console.log(result);
            "#.to_string(),
            "medium" => r#"
                class DataProcessor {
                    constructor(config) {
                        this.config = config;
                        this.cache = new Map();
                    }
                    
                    async processData(data) {
                        if (this.cache.has(data.id)) {
                            return this.cache.get(data.id);
                        }
                        
                        const result = await this.transform(data);
                        this.cache.set(data.id, result);
                        return result;
                    }
                    
                    transform(data) {
                        return new Promise((resolve) => {
                            setTimeout(() => {
                                resolve({
                                    ...data,
                                    processed: true,
                                    timestamp: Date.now()
                                });
                            }, 100);
                        });
                    }
                }
            "#.to_string(),
            "complex" => r#"
                use std::collections::HashMap;
                use tokio::sync::RwLock;
                use uuid::Uuid;
                
                pub struct AdvancedProcessor<T: Clone + Send + Sync> {
                    cache: RwLock<HashMap<Uuid, T>>,
                    config: ProcessorConfig,
                    metrics: RwLock<ProcessorMetrics>,
                }
                
                impl<T: Clone + Send + Sync> AdvancedProcessor<T> {
                    pub async fn new(config: ProcessorConfig) -> Self {
                        Self {
                            cache: RwLock::new(HashMap::new()),
                            config,
                            metrics: RwLock::new(ProcessorMetrics::default()),
                        }
                    }
                    
                    pub async fn process_batch(&self, items: Vec<(Uuid, T)>) -> Result<Vec<T>, ProcessorError> {
                        let mut results = Vec::with_capacity(items.len());
                        let start_time = std::time::Instant::now();
                        
                        for (id, item) in items {
                            let result = self.process_single(id, item).await?;
                            results.push(result);
                        }
                        
                        let mut metrics = self.metrics.write().await;
                        metrics.update_batch_metrics(results.len(), start_time.elapsed());
                        
                        Ok(results)
                    }
                    
                    async fn process_single(&self, id: Uuid, item: T) -> Result<T, ProcessorError> {
                        // Check cache first
                        {
                            let cache = self.cache.read().await;
                            if let Some(cached) = cache.get(&id) {
                                return Ok(cached.clone());
                            }
                        }
                        
                        // Process item
                        let processed = self.apply_transformations(item).await?;
                        
                        // Update cache
                        {
                            let mut cache = self.cache.write().await;
                            cache.insert(id, processed.clone());
                        }
                        
                        Ok(processed)
                    }
                    
                    async fn apply_transformations(&self, item: T) -> Result<T, ProcessorError> {
                        // Complex transformation logic would go here
                        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        Ok(item)
                    }
                }
            "#.to_string(),
            _ => Self::generate_code_data("simple")
        }
    }

    fn generate_json_data(record_count: usize) -> String {
        let records: Vec<serde_json::Value> = (0..record_count)
            .map(|i| {
                serde_json::json!({
                    "id": i,
                    "name": format!("Record {}", i),
                    "email": format!("user{}@example.com", i),
                    "active": i % 2 == 0,
                    "score": (i as f64 * 0.1) % 1.0,
                    "metadata": {
                        "created_at": "2024-01-01T00:00:00Z",
                        "updated_at": "2024-01-01T00:00:00Z",
                        "tags": vec![format!("tag_{}", i % 5), "benchmark".to_string()],
                        "properties": {
                            "level": i % 10,
                            "category": format!("cat_{}", i % 3)
                        }
                    }
                })
            })
            .collect();

        serde_json::json!({
            "total": record_count,
            "records": records,
            "metadata": {
                "generated_for": "benchmark",
                "version": "1.0.0",
                "complexity": "medium"
            }
        })
        .to_string()
    }

    fn generate_mixed_content() -> String {
        format!(
            r#"
# System Analysis Report

This report analyzes the performance characteristics of our data processing pipeline.

## Code Implementation

Here's the main processing function:

```python
async def process_data_pipeline(data_source, transformers):
    """
    Processes data through a series of transformers.
    
    Args:
        data_source: Input data source
        transformers: List of transformation functions
    
    Returns:
        ProcessedData: Transformed data with metadata
    """
    results = []
    
    for transformer in transformers:
        try:
            result = await transformer.process(data_source)
            results.append(result)
        except ProcessingError as e:
            logger.error(f"Transformation failed: {{e}}")
            raise
    
    return ProcessedData(
        data=results,
        metadata=generate_metadata(),
        timestamp=datetime.utcnow()
    )
```

## Performance Metrics

The system shows the following performance characteristics:

{}

## Analysis Summary

The data indicates strong performance across all metrics with room for optimization in the error handling pathways.
            "#,
            Self::generate_json_data(50)
        )
    }
}

// Benchmark functions
async fn create_multimodal_agent() -> MultiModalAgent {
    let nlp_processor = NLPProcessor::new()
        .await
        .expect("Failed to create NLP processor");
    MultiModalAgent::new("BenchmarkAgent".to_string(), Some(nlp_processor))
        .await
        .expect("Failed to create MultiModalAgent")
}

fn bench_text_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("text_analysis");

    for size in [100, 500, 1000, 2000].iter() {
        let data = BenchmarkDataGenerator::generate_text_data(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("words", size), &data, |b, data| {
            b.to_async(&rt).iter(|| async {
                let mut agent = create_multimodal_agent().await;
                let result = agent.analyze_multimodal_data(black_box(data)).await;
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_code_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("code_analysis");

    for complexity in ["simple", "medium", "complex"].iter() {
        let data = BenchmarkDataGenerator::generate_code_data(complexity);

        group.bench_with_input(
            BenchmarkId::new("complexity", complexity),
            &data,
            |b, data| {
                b.to_async(&rt).iter(|| async {
                    let mut agent = create_multimodal_agent().await;
                    let result = agent.analyze_multimodal_data(black_box(data)).await;
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn bench_data_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("data_analysis");

    for record_count in [10, 50, 100, 500].iter() {
        let data = BenchmarkDataGenerator::generate_json_data(*record_count);

        group.throughput(Throughput::Elements(*record_count as u64));
        group.bench_with_input(
            BenchmarkId::new("records", record_count),
            &data,
            |b, data| {
                b.to_async(&rt).iter(|| async {
                    let mut agent = create_multimodal_agent().await;
                    let result = agent.analyze_multimodal_data(black_box(data)).await;
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn bench_mixed_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("mixed_analysis");
    let data = BenchmarkDataGenerator::generate_mixed_content();

    group.bench_function("comprehensive", |b| {
        b.to_async(&rt).iter(|| async {
            let mut agent = create_multimodal_agent().await;
            let result = agent.analyze_multimodal_data(black_box(&data)).await;
            black_box(result)
        });
    });

    group.finish();
}

fn bench_learning_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("learning_performance");

    group.bench_function("adaptive_learning", |b| {
        b.to_async(&rt).iter(|| async {
            let mut agent = create_multimodal_agent().await;

            // Perform multiple analyses to trigger learning
            for i in 0..10 {
                let data = format!(
                    "Learning iteration {} with various complexity levels and patterns.",
                    i
                );
                let _result = agent.analyze_multimodal_data(&data).await;
            }

            // Measure final analysis performance
            let final_data = "Final analysis to measure learned performance improvements.";
            let result = agent.analyze_multimodal_data(black_box(final_data)).await;
            black_box(result)
        });
    });

    group.finish();
}

fn bench_concurrent_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_analysis");

    for concurrent_tasks in [2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("tasks", concurrent_tasks),
            concurrent_tasks,
            |b, &task_count| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();

                    for i in 0..task_count {
                        let handle = tokio::spawn(async move {
                            let mut agent = create_multimodal_agent().await;
                            let data =
                                format!("Concurrent analysis task {} with unique content.", i);
                            agent.analyze_multimodal_data(&data).await
                        });
                        handles.push(handle);
                    }

                    // Wait for all tasks to complete
                    let results = futures::future::join_all(handles).await;
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_efficiency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_efficiency");

    group.bench_function("history_management", |b| {
        b.to_async(&rt).iter(|| async {
            let mut agent = create_multimodal_agent().await;

            // Generate many analyses to test memory management
            for i in 0..200 {
                let data = format!("Memory test iteration {} with growing content size.", i);
                let _result = agent.analyze_multimodal_data(&data).await;
            }

            // Test history cleanup
            agent.clear_analysis_history();

            // Verify performance after cleanup
            let final_data = "Performance test after history cleanup.";
            let result = agent.analyze_multimodal_data(black_box(final_data)).await;
            black_box(result)
        });
    });

    group.finish();
}

// Criterion configuration
criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(50)
        .warm_up_time(Duration::from_secs(3));
    targets =
        bench_text_analysis,
        bench_code_analysis,
        bench_data_analysis,
        bench_mixed_analysis,
        bench_learning_performance,
        bench_concurrent_analysis,
        bench_memory_efficiency
);

criterion_main!(benches);
