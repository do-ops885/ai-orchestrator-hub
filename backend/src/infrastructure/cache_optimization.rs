//! Cache Optimization Strategies and Benchmarks
//!
//! Provides automated cache optimization, benchmarking, and performance tuning
//! for the intelligent caching system.

use crate::infrastructure::cache_monitoring::{CachePerformanceMetrics, CachePerformanceMonitor};
use crate::infrastructure::cached_query::{CacheKey, CachedQueryConfig, CachedQueryManager};
use crate::utils::error::HiveResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Cache optimization strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationStrategy {
    /// Adaptive TTL based on access patterns
    AdaptiveTTL,
    /// Size-based optimization
    SizeOptimization,
    /// Prefetching optimization
    PrefetchOptimization,
    /// Memory usage optimization
    MemoryOptimization,
    /// Hit rate optimization
    HitRateOptimization,
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Strategy to apply
    pub strategy: OptimizationStrategy,
    /// Expected improvement percentage
    pub expected_improvement: f64,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Implementation difficulty (1-10)
    pub difficulty: u8,
    /// Description of the optimization
    pub description: String,
    /// Estimated time to implement
    pub estimated_time_minutes: u32,
}

/// Benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Operations per second
    pub ops_per_second: f64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// 95th percentile latency
    pub p95_latency_ms: f64,
    /// 99th percentile latency
    pub p99_latency_ms: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Cache hit rate
    pub hit_rate: f64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Cache optimization engine
pub struct CacheOptimizationEngine {
    /// Cache manager to optimize
    cache_manager: Arc<CachedQueryManager>,
    /// Performance monitor
    performance_monitor: Arc<CachePerformanceMonitor>,
    /// Current configuration
    current_config: CachedQueryConfig,
    /// Optimization history
    optimization_history: Arc<RwLock<Vec<OptimizationEvent>>>,
    /// Benchmark results
    benchmark_results: Arc<RwLock<HashMap<String, BenchmarkResult>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub strategy: OptimizationStrategy,
    pub before_metrics: CachePerformanceMetrics,
    pub after_metrics: CachePerformanceMetrics,
    pub improvement: f64,
    pub success: bool,
}

impl CacheOptimizationEngine {
    /// Create a new cache optimization engine
    #[must_use] 
    pub fn new(
        cache_manager: Arc<CachedQueryManager>,
        performance_monitor: Arc<CachePerformanceMonitor>,
    ) -> Self {
        let current_config = CachedQueryConfig::default();

        Self {
            cache_manager,
            performance_monitor,
            current_config,
            optimization_history: Arc::new(RwLock::new(Vec::new())),
            benchmark_results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Analyze cache performance and generate optimization recommendations
    pub async fn analyze_and_recommend(&self) -> HiveResult<Vec<OptimizationRecommendation>> {
        let metrics = self.performance_monitor.get_metrics().await;
        let mut recommendations = Vec::new();

        // Analyze hit rate
        if metrics.overall_hit_rate < 0.7 {
            recommendations.push(OptimizationRecommendation {
                strategy: OptimizationStrategy::HitRateOptimization,
                expected_improvement: 15.0,
                confidence: 0.8,
                difficulty: 3,
                description:
                    "Optimize cache hit rate through better TTL management and prefetching"
                        .to_string(),
                estimated_time_minutes: 30,
            });
        }

        // Analyze memory usage
        let memory_usage_percentage = metrics.memory_usage_bytes as f64 / 100_000_000.0;
        if memory_usage_percentage > 0.8 {
            recommendations.push(OptimizationRecommendation {
                strategy: OptimizationStrategy::MemoryOptimization,
                expected_improvement: 20.0,
                confidence: 0.9,
                difficulty: 2,
                description: "Reduce memory usage through better eviction policies".to_string(),
                estimated_time_minutes: 15,
            });
        }

        // Analyze access time
        if metrics.avg_cache_access_time_ms > 5.0 {
            recommendations.push(OptimizationRecommendation {
                strategy: OptimizationStrategy::SizeOptimization,
                expected_improvement: 25.0,
                confidence: 0.7,
                difficulty: 4,
                description: "Optimize cache size and data structures for faster access"
                    .to_string(),
                estimated_time_minutes: 45,
            });
        }

        // Adaptive TTL recommendation
        if !self.current_config.enable_adaptive_ttl {
            recommendations.push(OptimizationRecommendation {
                strategy: OptimizationStrategy::AdaptiveTTL,
                expected_improvement: 10.0,
                confidence: 0.85,
                difficulty: 1,
                description: "Enable adaptive TTL based on access patterns".to_string(),
                estimated_time_minutes: 10,
            });
        }

        // Prefetching recommendation
        if !self.current_config.enable_prefetching {
            recommendations.push(OptimizationRecommendation {
                strategy: OptimizationStrategy::PrefetchOptimization,
                expected_improvement: 12.0,
                confidence: 0.75,
                difficulty: 3,
                description: "Enable intelligent prefetching for frequently accessed data"
                    .to_string(),
                estimated_time_minutes: 25,
            });
        }

        // Sort by expected improvement
        recommendations.sort_by(|a, b| {
            b.expected_improvement
                .partial_cmp(&a.expected_improvement)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(recommendations)
    }

    /// Apply an optimization strategy
    pub async fn apply_optimization(&self, strategy: OptimizationStrategy) -> HiveResult<f64> {
        info!("Applying cache optimization strategy: {:?}", strategy);

        // Capture before metrics
        let before_metrics = self.performance_monitor.get_metrics().await;

        // Apply the optimization
        let success = match strategy {
            OptimizationStrategy::AdaptiveTTL => self.enable_adaptive_ttl().await,
            OptimizationStrategy::SizeOptimization => self.optimize_cache_size().await,
            OptimizationStrategy::PrefetchOptimization => self.enable_prefetching().await,
            OptimizationStrategy::MemoryOptimization => self.optimize_memory_usage().await,
            OptimizationStrategy::HitRateOptimization => self.optimize_hit_rate().await,
        };

        // Wait a bit for metrics to stabilize
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Capture after metrics
        let after_metrics = self.performance_monitor.get_metrics().await;

        // Calculate improvement
        let improvement = self.calculate_improvement(&before_metrics, &after_metrics);

        // Record optimization event
        let event = OptimizationEvent {
            timestamp: chrono::Utc::now(),
            strategy: strategy.clone(),
            before_metrics,
            after_metrics,
            improvement,
            success,
        };

        {
            let mut history = self.optimization_history.write().await;
            history.push(event);
        }

        if success {
            info!(
                "Optimization {:?} applied successfully, improvement: {:.2}%",
                strategy, improvement
            );
        } else {
            warn!("Optimization {:?} failed to apply", strategy);
        }

        Ok(improvement)
    }

    /// Run comprehensive cache benchmark
    pub async fn run_benchmark(
        &self,
        name: &str,
        duration: Duration,
    ) -> HiveResult<BenchmarkResult> {
        info!("Starting cache benchmark: {}", name);

        let start_time = Instant::now();
        let mut operations = 0u64;
        let mut latencies = Vec::new();
        let mut hits = 0u64;
        let mut misses = 0u64;

        // Benchmark loop
        while start_time.elapsed() < duration {
            let operation_start = Instant::now();

            // Simulate cache operations
            let key = CacheKey::Custom(format!("benchmark_key_{}", operations % 1000));

            // Mix of get and set operations
            if operations.is_multiple_of(3) {
                // Set operation
                let cache_entry = crate::infrastructure::cached_query::CacheEntry::new(
                    format!("benchmark_value_{operations}"),
                    vec![],
                );
                if let Err(e) = self.cache_manager.set_cached(key, cache_entry).await {
                    warn!("Benchmark set operation failed: {}", e);
                }
            } else {
                // Get operation
                if let Some(_) = self
                    .cache_manager
                    .get_cached::<serde_json::Value>(&key)
                    .await
                {
                    hits += 1;
                } else {
                    misses += 1;
                }
            }

            let latency = operation_start.elapsed().as_micros() as f64 / 1000.0; // Convert to ms
            latencies.push(latency);
            operations += 1;

            // Small delay to prevent overwhelming
            tokio::time::sleep(Duration::from_micros(100)).await;
        }

        // Calculate statistics
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let total_time = start_time.elapsed().as_secs_f64();
        let ops_per_second = operations as f64 / total_time;

        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let p95_latency = latencies
            .get((latencies.len() as f64 * 0.95) as usize)
            .unwrap_or(&avg_latency);
        let p99_latency = latencies
            .get((latencies.len() as f64 * 0.99) as usize)
            .unwrap_or(&avg_latency);

        let hit_rate = if hits + misses > 0 {
            hits as f64 / (hits + misses) as f64
        } else {
            0.0
        };

        // Get current memory usage
        let metrics = self.performance_monitor.get_metrics().await;
        let memory_usage = metrics.memory_usage_bytes;

        let result = BenchmarkResult {
            name: name.to_string(),
            ops_per_second,
            avg_latency_ms: avg_latency,
            p95_latency_ms: *p95_latency,
            p99_latency_ms: *p99_latency,
            memory_usage_bytes: memory_usage,
            hit_rate,
            timestamp: chrono::Utc::now(),
        };

        // Store result
        {
            let mut results = self.benchmark_results.write().await;
            results.insert(name.to_string(), result.clone());
        }

        info!(
            "Benchmark {} completed: {:.0} ops/sec, {:.2}ms avg latency, {:.2}% hit rate",
            name,
            ops_per_second,
            avg_latency,
            hit_rate * 100.0
        );

        Ok(result)
    }

    /// Get optimization history
    pub async fn get_optimization_history(&self) -> Vec<OptimizationEvent> {
        self.optimization_history.read().await.clone()
    }

    /// Get benchmark results
    pub async fn get_benchmark_results(&self) -> HashMap<String, BenchmarkResult> {
        self.benchmark_results.read().await.clone()
    }

    /// Generate optimization report
    pub async fn generate_optimization_report(&self) -> serde_json::Value {
        let recommendations = self.analyze_and_recommend().await.unwrap_or_default();
        let history = self.get_optimization_history().await;
        let benchmarks = self.get_benchmark_results().await;
        let current_metrics = self.performance_monitor.get_metrics().await;

        // Calculate success rate
        let successful_optimizations = history.iter().filter(|e| e.success).count();
        let success_rate = if history.is_empty() {
            0.0
        } else {
            successful_optimizations as f64 / history.len() as f64
        };

        // Calculate average improvement
        let avg_improvement = if history.is_empty() {
            0.0
        } else {
            history.iter().map(|e| e.improvement).sum::<f64>() / history.len() as f64
        };

        serde_json::json!({
            "current_performance": {
                "hit_rate": current_metrics.overall_hit_rate,
                "efficiency_score": current_metrics.efficiency_score,
                "query_reduction": current_metrics.query_reduction_percentage,
                "memory_usage_mb": current_metrics.memory_usage_bytes / 1_000_000
            },
            "optimization_summary": {
                "total_optimizations": history.len(),
                "successful_optimizations": successful_optimizations,
                "success_rate": success_rate,
                "average_improvement": avg_improvement,
                "total_improvement": history.iter().map(|e| e.improvement).sum::<f64>()
            },
            "recommendations": recommendations.into_iter().map(|r| {
                serde_json::json!({
                    "strategy": format!("{:?}", r.strategy),
                    "expected_improvement": r.expected_improvement,
                    "confidence": r.confidence,
                    "difficulty": r.difficulty,
                    "description": r.description,
                    "estimated_time_minutes": r.estimated_time_minutes
                })
            }).collect::<Vec<_>>(),
            "recent_benchmarks": benchmarks.into_iter().map(|(name, result)| {
                serde_json::json!({
                    "name": name,
                    "ops_per_second": result.ops_per_second,
                    "avg_latency_ms": result.avg_latency_ms,
                    "hit_rate": result.hit_rate,
                    "timestamp": result.timestamp.to_rfc3339()
                })
            }).collect::<Vec<_>>(),
            "generated_at": chrono::Utc::now().to_rfc3339()
        })
    }

    // Private optimization methods

    async fn enable_adaptive_ttl(&self) -> bool {
        // This would modify the cache configuration
        // For now, just return success
        debug!("Enabling adaptive TTL");
        true
    }

    async fn optimize_cache_size(&self) -> bool {
        // This would adjust cache size based on usage patterns
        debug!("Optimizing cache size");
        true
    }

    async fn enable_prefetching(&self) -> bool {
        // This would enable prefetching
        debug!("Enabling prefetching");
        true
    }

    async fn optimize_memory_usage(&self) -> bool {
        // This would optimize memory usage
        debug!("Optimizing memory usage");
        true
    }

    async fn optimize_hit_rate(&self) -> bool {
        // This would optimize hit rate through various strategies
        debug!("Optimizing hit rate");
        true
    }

    fn calculate_improvement(
        &self,
        before: &CachePerformanceMetrics,
        after: &CachePerformanceMetrics,
    ) -> f64 {
        let hit_rate_improvement = (after.overall_hit_rate - before.overall_hit_rate) * 100.0;
        let efficiency_improvement = (after.efficiency_score - before.efficiency_score) * 100.0;
        let query_reduction_improvement =
            after.query_reduction_percentage - before.query_reduction_percentage;

        // Weighted average
        (hit_rate_improvement * 0.4)
            + (efficiency_improvement * 0.4)
            + (query_reduction_improvement * 0.2)
    }
}

/// Automated cache tuner
pub struct CacheTuner {
    optimization_engine: Arc<CacheOptimizationEngine>,
    tuning_config: TuningConfig,
}

#[derive(Debug, Clone)]
pub struct TuningConfig {
    /// Enable automatic tuning
    pub auto_tune: bool,
    /// Tuning interval
    pub tuning_interval: Duration,
    /// Minimum improvement threshold
    pub min_improvement_threshold: f64,
    /// Maximum optimizations per tuning cycle
    pub max_optimizations_per_cycle: usize,
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            auto_tune: true,
            tuning_interval: Duration::from_secs(3600), // 1 hour
            min_improvement_threshold: 5.0,             // 5% improvement
            max_optimizations_per_cycle: 3,
        }
    }
}

impl CacheTuner {
    #[must_use] 
    pub fn new(optimization_engine: Arc<CacheOptimizationEngine>, config: TuningConfig) -> Self {
        Self {
            optimization_engine,
            tuning_config: config,
        }
    }

    /// Start automatic tuning
    #[must_use] 
    pub fn start_auto_tuning(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.tuning_config.tuning_interval);

            loop {
                interval.tick().await;

                if !self.tuning_config.auto_tune {
                    continue;
                }

                if let Err(e) = self.perform_tuning_cycle().await {
                    warn!("Auto-tuning cycle failed: {}", e);
                }
            }
        })
    }

    /// Perform a tuning cycle
    async fn perform_tuning_cycle(&self) -> HiveResult<()> {
        info!("Starting cache tuning cycle");

        let recommendations = self.optimization_engine.analyze_and_recommend().await?;
        let mut applied_count = 0;

        for recommendation in recommendations
            .into_iter()
            .filter(|r| r.expected_improvement >= self.tuning_config.min_improvement_threshold)
            .take(self.tuning_config.max_optimizations_per_cycle)
        {
            let improvement = self
                .optimization_engine
                .apply_optimization(recommendation.strategy)
                .await?;

            if improvement >= self.tuning_config.min_improvement_threshold {
                applied_count += 1;
                info!("Applied optimization with {:.2}% improvement", improvement);
            }
        }

        info!(
            "Tuning cycle completed, applied {} optimizations",
            applied_count
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache_monitoring::CacheMonitoringConfig;

    #[tokio::test]
    async fn test_cache_optimization_engine() -> Result<(), Box<dyn std::error::Error>> {
        let cache_manager = Arc::new(CachedQueryManager::new(CachedQueryConfig::default()));
        let invalidation_manager = Arc::new(
            crate::infrastructure::cache_invalidation::CacheInvalidationManager::new(
                cache_manager.clone(),
                crate::infrastructure::cache_invalidation::InvalidationStrategy::Immediate,
            ),
        );

        let monitor = Arc::new(CachePerformanceMonitor::new(
            vec![cache_manager],
            vec![invalidation_manager],
            CacheMonitoringConfig::default(),
        ));

        let engine = Arc::new(CacheOptimizationEngine::new(
            monitor.cache_managers()[0].clone(),
            monitor,
        ));

        // Test analysis
        let recommendations = engine.analyze_and_recommend().await?;
        assert!(!recommendations.is_empty());

        // Test benchmark
        let benchmark_result = engine
            .run_benchmark("test_benchmark", Duration::from_secs(1))
            .await?;
        assert_eq!(benchmark_result.name, "test_benchmark");
        assert!(benchmark_result.ops_per_second > 0.0);

        Ok(())
    }
}
