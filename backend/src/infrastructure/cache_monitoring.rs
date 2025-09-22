//! Cache Performance Monitoring and Metrics
//!
//! Provides comprehensive monitoring and analytics for the intelligent caching system.
//! Tracks performance metrics, cache efficiency, and optimization opportunities.

use crate::infrastructure::cache_invalidation::CacheInvalidationManager;
use crate::infrastructure::cached_query::CachedQueryManager;
use crate::utils::error::HiveResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Comprehensive cache performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePerformanceMetrics {
    /// Overall cache hit rate
    pub overall_hit_rate: f64,
    /// Hit rate by cache type
    pub hit_rate_by_type: HashMap<String, f64>,
    /// Total queries avoided
    pub total_queries_avoided: u64,
    /// Query reduction percentage
    pub query_reduction_percentage: f64,
    /// Average cache access time in milliseconds
    pub avg_cache_access_time_ms: f64,
    /// Cache memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Cache efficiency score (0.0 to 1.0)
    pub efficiency_score: f64,
    /// Timestamp of last metrics update
    pub last_updated: chrono::DateTime<chrono::Utc>,
    /// Intelligent caching metrics
    pub intelligent_metrics: IntelligentCacheMetrics,
    /// Database load reduction metrics
    pub db_load_reduction: MonitoringDatabaseLoadReductionMetrics,
    /// TTL adaptation metrics
    pub ttl_adaptation: TtlAdaptationMetrics,
}

/// Intelligent caching specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentCacheMetrics {
    /// Prefetch hit rate
    pub prefetch_hit_rate: f64,
    /// Prefetch efficiency
    pub prefetch_efficiency: f64,
    /// Adaptive TTL adjustments
    pub adaptive_ttl_adjustments: u64,
    /// Pattern prediction accuracy
    pub pattern_prediction_accuracy: f64,
    /// Burst access detection rate
    pub burst_detection_rate: f64,
}

/// Database load reduction metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringDatabaseLoadReductionMetrics {
    /// Deduplication savings
    pub deduplication_savings: f64,
    /// Batch processing savings
    pub batch_processing_savings: f64,
    /// Total load reduction percentage
    pub total_load_reduction: f64,
    /// Target achievement status
    pub target_achieved: bool,
}

/// TTL adaptation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtlAdaptationMetrics {
    /// Average TTL score
    pub average_ttl_score: f64,
    /// Freshness class distribution
    pub freshness_distribution: HashMap<String, usize>,
    /// TTL adaptation frequency
    pub adaptation_frequency: f64,
    /// Data staleness reduction
    pub staleness_reduction: f64,
}

/// Cache health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHealthStatus {
    /// Overall health score (0.0 to 1.0)
    pub health_score: f64,
    /// Cache operational status
    pub is_operational: bool,
    /// Memory usage percentage
    pub memory_usage_percentage: f64,
    /// Performance degradation indicators
    pub performance_warnings: Vec<String>,
    /// Recommended optimizations
    pub optimization_recommendations: Vec<String>,
}

/// Cache monitoring configuration
#[derive(Debug, Clone)]
pub struct CacheMonitoringConfig {
    /// Metrics collection interval
    pub collection_interval: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Performance alert thresholds
    pub alert_thresholds: AlertThresholds,
    /// Enable detailed metrics collection
    pub enable_detailed_metrics: bool,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// Minimum acceptable hit rate
    pub min_hit_rate: f64,
    /// Maximum acceptable memory usage percentage
    pub max_memory_usage: f64,
    /// Maximum acceptable cache access time
    pub max_access_time_ms: f64,
}

impl Default for CacheMonitoringConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_secs(60),
            health_check_interval: Duration::from_secs(300),
            alert_thresholds: AlertThresholds {
                min_hit_rate: 0.7,        // 70% minimum hit rate
                max_memory_usage: 0.8,    // 80% maximum memory usage
                max_access_time_ms: 10.0, // 10ms maximum access time
            },
            enable_detailed_metrics: true,
        }
    }
}

/// Cache performance monitor
pub struct CachePerformanceMonitor {
    /// Cache managers to monitor
    cache_managers: Vec<Arc<CachedQueryManager>>,
    /// Invalidation managers to monitor
    invalidation_managers: Vec<Arc<CacheInvalidationManager>>,
    /// Current performance metrics
    metrics: Arc<RwLock<CachePerformanceMetrics>>,
    /// Health status
    health_status: Arc<RwLock<CacheHealthStatus>>,
    /// Configuration
    config: CacheMonitoringConfig,
    /// Historical metrics for trend analysis
    historical_metrics: Arc<RwLock<Vec<(chrono::DateTime<chrono::Utc>, CachePerformanceMetrics)>>>,
}

impl CachePerformanceMonitor {
    /// Create a new cache performance monitor
    #[must_use] 
    pub fn new(
        cache_managers: Vec<Arc<CachedQueryManager>>,
        invalidation_managers: Vec<Arc<CacheInvalidationManager>>,
        config: CacheMonitoringConfig,
    ) -> Self {
        let initial_metrics = CachePerformanceMetrics {
            overall_hit_rate: 0.0,
            hit_rate_by_type: HashMap::new(),
            total_queries_avoided: 0,
            query_reduction_percentage: 0.0,
            avg_cache_access_time_ms: 0.0,
            memory_usage_bytes: 0,
            efficiency_score: 0.0,
            last_updated: chrono::Utc::now(),
            intelligent_metrics: IntelligentCacheMetrics {
                prefetch_hit_rate: 0.0,
                prefetch_efficiency: 0.0,
                adaptive_ttl_adjustments: 0,
                pattern_prediction_accuracy: 0.0,
                burst_detection_rate: 0.0,
            },
            db_load_reduction: MonitoringDatabaseLoadReductionMetrics {
                deduplication_savings: 0.0,
                batch_processing_savings: 0.0,
                total_load_reduction: 0.0,
                target_achieved: false,
            },
            ttl_adaptation: TtlAdaptationMetrics {
                average_ttl_score: 0.5,
                freshness_distribution: HashMap::new(),
                adaptation_frequency: 0.0,
                staleness_reduction: 0.0,
            },
        };

        let initial_health = CacheHealthStatus {
            health_score: 1.0,
            is_operational: true,
            memory_usage_percentage: 0.0,
            performance_warnings: Vec::new(),
            optimization_recommendations: Vec::new(),
        };

        Self {
            cache_managers,
            invalidation_managers,
            metrics: Arc::new(RwLock::new(initial_metrics)),
            health_status: Arc::new(RwLock::new(initial_health)),
            config,
            historical_metrics: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get cache managers (for testing)
    #[must_use] 
    pub fn cache_managers(&self) -> &Vec<Arc<CachedQueryManager>> {
        &self.cache_managers
    }

    /// Collect current performance metrics
    pub async fn collect_metrics(&self) -> HiveResult<()> {
        let start_time = Instant::now();
        debug!("Starting cache metrics collection");

        let mut total_hits = 0u64;
        let mut total_misses = 0u64;
        let mut total_queries_avoided = 0u64;
        let mut total_db_queries = 0u64;
        let mut hit_rate_by_type = HashMap::new();
        let mut total_memory = 0usize;

        // Collect metrics from all cache managers
        for (i, cache_manager) in self.cache_managers.iter().enumerate() {
            let stats = cache_manager.get_stats().await;
            total_hits += stats.cache_hits;
            total_misses += stats.cache_misses;
            total_queries_avoided += stats.queries_avoided;
            total_db_queries += stats.db_queries;

            // Estimate memory usage (simplified)
            total_memory += (stats.cache_hits + stats.cache_misses) as usize * 1024; // Rough estimate

            let cache_type = format!("cache_{i}");
            let type_hit_rate = if stats.cache_hits + stats.cache_misses > 0 {
                stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64
            } else {
                0.0
            };
            hit_rate_by_type.insert(cache_type, type_hit_rate);
        }

        // Calculate overall metrics
        let overall_hit_rate = if total_hits + total_misses > 0 {
            total_hits as f64 / (total_hits + total_misses) as f64
        } else {
            0.0
        };

        let query_reduction_percentage = if total_db_queries + total_queries_avoided > 0 {
            total_queries_avoided as f64 / (total_db_queries + total_queries_avoided) as f64 * 100.0
        } else {
            0.0
        };

        // Calculate efficiency score
        let efficiency_score =
            (overall_hit_rate * 0.6) + ((query_reduction_percentage / 100.0) * 0.4);

        let collection_time = start_time.elapsed().as_millis() as f64;

        // Collect intelligent caching metrics
        let intelligent_metrics = self.collect_intelligent_metrics().await;

        // Collect database load reduction metrics
        let db_load_reduction = self.collect_db_load_reduction_metrics().await;

        // Collect TTL adaptation metrics
        let ttl_adaptation = self.collect_ttl_adaptation_metrics().await;

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.overall_hit_rate = overall_hit_rate;
            metrics.hit_rate_by_type = hit_rate_by_type;
            metrics.total_queries_avoided = total_queries_avoided;
            metrics.query_reduction_percentage = query_reduction_percentage;
            metrics.avg_cache_access_time_ms = collection_time;
            metrics.memory_usage_bytes = total_memory;
            metrics.efficiency_score = efficiency_score;
            metrics.intelligent_metrics = intelligent_metrics;
            metrics.db_load_reduction = db_load_reduction;
            metrics.ttl_adaptation = ttl_adaptation;
            metrics.last_updated = chrono::Utc::now();
        }

        // Store historical data
        {
            let metrics = self.metrics.read().await.clone();
            let mut historical = self.historical_metrics.write().await;
            historical.push((chrono::Utc::now(), metrics));

            // Keep only last 100 data points
            if historical.len() > 100 {
                historical.remove(0);
            }
        }

        debug!(
            "Cache metrics collection completed in {:.2}ms",
            collection_time
        );
        Ok(())
    }

    /// Perform health check
    pub async fn perform_health_check(&self) -> HiveResult<()> {
        debug!("Starting cache health check");

        let metrics = self.metrics.read().await;
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Check hit rate
        if metrics.overall_hit_rate < self.config.alert_thresholds.min_hit_rate {
            warnings.push(format!(
                "Low cache hit rate: {:.2}% (threshold: {:.2}%)",
                metrics.overall_hit_rate * 100.0,
                self.config.alert_thresholds.min_hit_rate * 100.0
            ));
            recommendations
                .push("Consider increasing cache TTL or enabling prefetching".to_string());
        }

        // Check memory usage (simplified)
        let memory_usage_percentage = metrics.memory_usage_bytes as f64 / 100_000_000.0; // Assume 100MB max
        if memory_usage_percentage > self.config.alert_thresholds.max_memory_usage {
            warnings.push(format!(
                "High memory usage: {:.2}% (threshold: {:.2}%)",
                memory_usage_percentage * 100.0,
                self.config.alert_thresholds.max_memory_usage * 100.0
            ));
            recommendations
                .push("Consider reducing cache size or enabling LRU eviction".to_string());
        }

        // Check access time
        if metrics.avg_cache_access_time_ms > self.config.alert_thresholds.max_access_time_ms {
            warnings.push(format!(
                "Slow cache access: {:.2}ms (threshold: {:.2}ms)",
                metrics.avg_cache_access_time_ms, self.config.alert_thresholds.max_access_time_ms
            ));
            recommendations.push(
                "Consider optimizing cache data structures or reducing cache size".to_string(),
            );
        }

        // Calculate health score
        let health_score = if warnings.is_empty() {
            1.0
        } else {
            0.5 - (warnings.len() as f64 * 0.1).min(0.4)
        };

        // Update health status
        {
            let mut health = self.health_status.write().await;
            health.health_score = health_score.max(0.0);
            health.is_operational = true; // Would check actual operational status
            health.memory_usage_percentage = memory_usage_percentage;
            health.performance_warnings = warnings;
            health.optimization_recommendations = recommendations;
        }

        debug!(
            "Cache health check completed, health score: {:.2}",
            health_score
        );
        Ok(())
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> CachePerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Get current health status
    pub async fn get_health_status(&self) -> CacheHealthStatus {
        self.health_status.read().await.clone()
    }

    /// Get performance trends
    pub async fn get_performance_trends(
        &self,
    ) -> Vec<(chrono::DateTime<chrono::Utc>, CachePerformanceMetrics)> {
        self.historical_metrics.read().await.clone()
    }

    /// Generate performance report
    pub async fn generate_report(&self) -> serde_json::Value {
        let metrics = self.get_metrics().await;
        let health = self.get_health_status().await;
        let trends = self.get_performance_trends().await;

        // Calculate trend analysis
        let trend_analysis = if trends.len() >= 2 {
            let recent = &trends[trends.len() - 1].1;
            let previous = &trends[trends.len() - 2].1;

            let hit_rate_change = recent.overall_hit_rate - previous.overall_hit_rate;
            let efficiency_change = recent.efficiency_score - previous.efficiency_score;

            serde_json::json!({
                "hit_rate_trend": if hit_rate_change > 0.0 { "improving" } else if hit_rate_change < 0.0 { "declining" } else { "stable" },
                "efficiency_trend": if efficiency_change > 0.0 { "improving" } else if efficiency_change < 0.0 { "declining" } else { "stable" },
                "hit_rate_change_percentage": hit_rate_change * 100.0,
                "efficiency_change_percentage": efficiency_change * 100.0
            })
        } else {
            serde_json::json!({
                "message": "Insufficient data for trend analysis"
            })
        };

        serde_json::json!({
            "performance_metrics": metrics,
            "health_status": health,
            "trend_analysis": trend_analysis,
            "recommendations": {
                "immediate_actions": health.optimization_recommendations,
                "performance_targets": {
                    "target_hit_rate": 0.85,
                    "target_efficiency": 0.80,
                    "current_hit_rate": metrics.overall_hit_rate,
                    "current_efficiency": metrics.efficiency_score
                }
            },
            "generated_at": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Start background monitoring
    #[must_use] 
    pub fn start_monitoring(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut metrics_interval = tokio::time::interval(self.config.collection_interval);
            let mut health_interval = tokio::time::interval(self.config.health_check_interval);

            loop {
                tokio::select! {
                    _ = metrics_interval.tick() => {
                        if let Err(e) = self.collect_metrics().await {
                            warn!("Failed to collect cache metrics: {}", e);
                        }
                    }
                    _ = health_interval.tick() => {
                        if let Err(e) = self.perform_health_check().await {
                            warn!("Failed to perform cache health check: {}", e);
                        }
                    }
                }
            }
        })
    }

    /// Collect intelligent caching metrics
    async fn collect_intelligent_metrics(&self) -> IntelligentCacheMetrics {
        // This would collect metrics from intelligent cache instances
        // For now, return default values
        IntelligentCacheMetrics {
            prefetch_hit_rate: 0.0,
            prefetch_efficiency: 0.0,
            adaptive_ttl_adjustments: 0,
            pattern_prediction_accuracy: 0.0,
            burst_detection_rate: 0.0,
        }
    }

    /// Collect database load reduction metrics
    async fn collect_db_load_reduction_metrics(&self) -> MonitoringDatabaseLoadReductionMetrics {
        let mut total_deduplication_savings = 0.0;
        let mut total_batch_savings = 0.0;
        let mut total_load_reduction = 0.0;
        let mut target_achieved = false;

        // Collect metrics from cache managers
        for cache_manager in &self.cache_managers {
            let load_metrics = cache_manager.get_load_reduction_metrics().await;
            total_deduplication_savings += load_metrics.deduplication_savings;
            total_batch_savings += load_metrics.batch_savings;
            total_load_reduction += load_metrics.total_reduction_percentage;
            if load_metrics.target_achieved {
                target_achieved = true;
            }
        }

        MonitoringDatabaseLoadReductionMetrics {
            deduplication_savings: total_deduplication_savings,
            batch_processing_savings: total_batch_savings,
            total_load_reduction,
            target_achieved,
        }
    }

    /// Collect TTL adaptation metrics
    async fn collect_ttl_adaptation_metrics(&self) -> TtlAdaptationMetrics {
        // This would collect metrics from intelligent cache instances
        // For now, return default values
        TtlAdaptationMetrics {
            average_ttl_score: 0.5,
            freshness_distribution: HashMap::new(),
            adaptation_frequency: 0.0,
            staleness_reduction: 0.0,
        }
    }

    /// Get detailed intelligent cache metrics
    pub async fn get_intelligent_metrics(&self) -> IntelligentCacheMetrics {
        self.metrics.read().await.intelligent_metrics.clone()
    }

    /// Get database load reduction metrics
    pub async fn get_db_load_reduction_metrics(&self) -> MonitoringDatabaseLoadReductionMetrics {
        self.metrics.read().await.db_load_reduction.clone()
    }

    /// Get TTL adaptation metrics
    pub async fn get_ttl_adaptation_metrics(&self) -> TtlAdaptationMetrics {
        self.metrics.read().await.ttl_adaptation.clone()
    }

    /// Generate comprehensive performance dashboard
    pub async fn generate_performance_dashboard(&self) -> serde_json::Value {
        let metrics = self.get_metrics().await;
        let intelligent = self.get_intelligent_metrics().await;
        let db_load = self.get_db_load_reduction_metrics().await;
        let ttl = self.get_ttl_adaptation_metrics().await;
        let health = self.get_health_status().await;

        serde_json::json!({
            "summary": {
                "overall_hit_rate": format!("{:.1}%", metrics.overall_hit_rate * 100.0),
                "query_reduction": format!("{:.1}%", metrics.query_reduction_percentage),
                "db_load_reduction": format!("{:.1}%", db_load.total_load_reduction),
                "target_achieved": db_load.target_achieved,
                "efficiency_score": format!("{:.1}%", metrics.efficiency_score * 100.0),
                "health_score": format!("{:.1}%", health.health_score * 100.0)
            },
            "intelligent_caching": {
                "prefetch_hit_rate": format!("{:.1}%", intelligent.prefetch_hit_rate * 100.0),
                "adaptive_ttl_adjustments": intelligent.adaptive_ttl_adjustments,
                "pattern_prediction_accuracy": format!("{:.1}%", intelligent.pattern_prediction_accuracy * 100.0)
            },
            "database_optimization": {
                "deduplication_savings": format!("{:.1}%", db_load.deduplication_savings),
                "batch_processing_savings": format!("{:.1}%", db_load.batch_processing_savings),
                "total_load_reduction": format!("{:.1}%", db_load.total_load_reduction)
            },
            "ttl_adaptation": {
                "average_ttl_score": format!("{:.2}", ttl.average_ttl_score),
                "adaptation_frequency": format!("{:.2}/sec", ttl.adaptation_frequency),
                "freshness_distribution": ttl.freshness_distribution
            },
            "alerts": {
                "warnings": health.performance_warnings,
                "recommendations": health.optimization_recommendations
            },
            "generated_at": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Reset monitoring data
    pub async fn reset(&self) -> HiveResult<()> {
        {
            let mut metrics = self.metrics.write().await;
            *metrics = CachePerformanceMetrics {
                overall_hit_rate: 0.0,
                hit_rate_by_type: HashMap::new(),
                total_queries_avoided: 0,
                query_reduction_percentage: 0.0,
                avg_cache_access_time_ms: 0.0,
                memory_usage_bytes: 0,
                efficiency_score: 0.0,
                last_updated: chrono::Utc::now(),
                intelligent_metrics: IntelligentCacheMetrics {
                    prefetch_hit_rate: 0.0,
                    prefetch_efficiency: 0.0,
                    adaptive_ttl_adjustments: 0,
                    pattern_prediction_accuracy: 0.0,
                    burst_detection_rate: 0.0,
                },
            db_load_reduction: MonitoringDatabaseLoadReductionMetrics {
                    deduplication_savings: 0.0,
                    batch_processing_savings: 0.0,
                    total_load_reduction: 0.0,
                    target_achieved: false,
                },
                ttl_adaptation: TtlAdaptationMetrics {
                    average_ttl_score: 0.0,
                    freshness_distribution: HashMap::new(),
                    adaptation_frequency: 0.0,
                    staleness_reduction: 0.0,
                },
            };
        }

        {
            let mut health = self.health_status.write().await;
            *health = CacheHealthStatus {
                health_score: 1.0,
                is_operational: true,
                memory_usage_percentage: 0.0,
                performance_warnings: Vec::new(),
                optimization_recommendations: Vec::new(),
            };
        }

        {
            let mut historical = self.historical_metrics.write().await;
            historical.clear();
        }

        info!("Cache monitoring data reset");
        Ok(())
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            min_hit_rate: 0.7,
            max_memory_usage: 0.8,
            max_access_time_ms: 10.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cached_query::CachedQueryConfig;

    #[tokio::test]
    async fn test_cache_performance_monitor() -> Result<(), Box<dyn std::error::Error>> {
        let cache_manager = Arc::new(CachedQueryManager::new(CachedQueryConfig::default()));
        let invalidation_manager = Arc::new(CacheInvalidationManager::new(
            cache_manager.clone(),
            crate::infrastructure::cache_invalidation::InvalidationStrategy::Immediate,
        ));

        let monitor = Arc::new(CachePerformanceMonitor::new(
            vec![cache_manager],
            vec![invalidation_manager],
            CacheMonitoringConfig::default(),
        ));

        // Collect initial metrics
        monitor.collect_metrics().await?;
        monitor.perform_health_check().await?;

        let metrics = monitor.get_metrics().await;
        let health = monitor.get_health_status().await;

        assert!(metrics.last_updated <= chrono::Utc::now());
        assert!(health.is_operational);

        let report = monitor.generate_report().await;
        assert!(report.is_object());

        Ok(())
    }
}
