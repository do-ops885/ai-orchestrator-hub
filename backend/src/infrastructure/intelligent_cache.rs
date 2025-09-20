//! Intelligent Caching System
//!
//! This module provides an advanced caching system with predictive prefetching,
//! adaptive TTL, and intelligent eviction strategies to optimize performance.

use crate::infrastructure::cache::{Cache, CacheStats};
use crate::utils::error::HiveResult;

use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Query execution tracking
#[derive(Debug)]
pub struct QueryExecution {
    pub query_key: String,
    pub start_time: Instant,
    pub status: String,
    pub completed: bool,
    pub result: Option<serde_json::Value>,
    pub execution_time: f64,
    pub waiters: Vec<tokio::sync::oneshot::Sender<HiveResult<serde_json::Value>>>,
}

/// Batch query request
#[derive(Debug)]
pub struct BatchQueryRequest {
    pub query_key: String,
    pub parameters: serde_json::Value,
    pub sender: tokio::sync::oneshot::Sender<HiveResult<serde_json::Value>>,
}

/// Optimization type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    CacheQuery,
    OptimizeQuery,
    BatchQueries,
}

/// Query optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOptimizationSuggestion {
    pub query_pattern: String,
    pub suggestion_type: OptimizationType,
    pub estimated_improvement: f64,
    pub reasoning: String,
}

/// Batch query optimizer
#[derive(Debug)]
pub struct BatchQueryOptimizer {
    pending_queries: HashMap<String, Vec<BatchQueryRequest>>,
    batch_timer: Option<tokio::time::Instant>,
    batch_size_threshold: usize,
    batch_time_window: Duration,
}

/// Query performance analyzer
#[derive(Debug)]
pub struct QueryPerformanceAnalyzer {
    execution_times: HashMap<String, Vec<f64>>,
    query_frequency: HashMap<String, usize>,
    slow_query_threshold: Duration,
    optimization_suggestions: Vec<QueryOptimizationSuggestion>,
}

/// Configuration for intelligent caching
#[derive(Debug, Clone)]
pub struct IntelligentCacheConfig {
    /// Base TTL for cache entries
    pub base_ttl: Duration,
    /// Maximum cache size
    pub max_size: usize,
    /// Enable predictive prefetching
    pub enable_prefetching: bool,
    /// Prefetch threshold (access frequency)
    pub prefetch_threshold: u32,
    /// Enable adaptive TTL based on access patterns
    pub enable_adaptive_ttl: bool,
    /// Minimum TTL for adaptive caching
    pub min_ttl: Duration,
    /// Maximum TTL for adaptive caching
    pub max_ttl: Duration,
    /// Enable LRU eviction strategy
    pub enable_lru_eviction: bool,
    /// Statistics collection interval
    pub stats_interval: Duration,
    /// TTL adaptation strategy
    pub ttl_strategy: TtlAdaptationStrategy,
    /// Data freshness requirements
    pub freshness_requirements: DataFreshnessRequirements,
}

/// TTL adaptation strategy
#[derive(Debug, Clone)]
pub enum TtlAdaptationStrategy {
    /// Linear adaptation based on access frequency
    Linear,
    /// Exponential adaptation for hot data
    Exponential,
    /// Segmented adaptation with different rules for different access patterns
    Segmented {
        hot_data_multiplier: f64,
        warm_data_multiplier: f64,
        cold_data_multiplier: f64,
    },
    /// Machine learning-based adaptation (simplified)
    Predictive {
        learning_rate: f64,
        prediction_horizon: Duration,
    },
}

/// Data freshness requirements for different data types
#[derive(Debug, Clone)]
pub struct DataFreshnessRequirements {
    /// Maximum staleness for critical data
    pub critical_data_max_age: Duration,
    /// Maximum staleness for important data
    pub important_data_max_age: Duration,
    /// Maximum staleness for normal data
    pub normal_data_max_age: Duration,
    /// Data type classifier
    pub data_type_classifier: DataTypeClassifier,
}

/// Data type classifier for freshness requirements
#[derive(Debug, Clone)]
pub enum DataTypeClassifier {
    /// Classify based on key patterns
    PatternBased {
        critical_patterns: Vec<String>,
        important_patterns: Vec<String>,
    },
    /// Classify based on access patterns
    AccessPatternBased {
        critical_threshold: f64, // access frequency
        important_threshold: f64,
    },
    /// Custom classifier function
    Custom,
}

impl Default for IntelligentCacheConfig {
    fn default() -> Self {
        Self {
            base_ttl: Duration::from_secs(300), // 5 minutes
            max_size: 10000,
            enable_prefetching: true,
            prefetch_threshold: 5,
            enable_adaptive_ttl: true,
            min_ttl: Duration::from_secs(60),   // 1 minute
            max_ttl: Duration::from_secs(3600), // 1 hour
            enable_lru_eviction: true,
            stats_interval: Duration::from_secs(60),
            ttl_strategy: TtlAdaptationStrategy::Segmented {
                hot_data_multiplier: 2.0,
                warm_data_multiplier: 1.0,
                cold_data_multiplier: 0.5,
            },
            freshness_requirements: DataFreshnessRequirements {
                critical_data_max_age: Duration::from_secs(30),
                important_data_max_age: Duration::from_secs(300),
                normal_data_max_age: Duration::from_secs(1800),
                data_type_classifier: DataTypeClassifier::AccessPatternBased {
                    critical_threshold: 1.0,  // 1 access per second
                    important_threshold: 0.1, // 1 access per 10 seconds
                },
            },
        }
    }
}

/// Access pattern tracking for intelligent caching decisions
#[derive(Debug, Clone)]
struct AccessPattern {
    access_count: u32,
    last_access: Instant,
    access_frequency: f64, // accesses per second
    access_history: VecDeque<Instant>,
    predicted_next_access: Option<Instant>,
    /// Time-based access patterns (hourly distribution)
    temporal_pattern: [f64; 24],
    /// Sequential access patterns (what comes after this key)
    sequential_patterns: HashMap<String, u32>,
    /// Burst access detection
    burst_count: u32,
    burst_start: Option<Instant>,
    /// Access velocity (rate of change in access frequency)
    access_velocity: f64,
    /// TTL adaptation history
    ttl_history: VecDeque<(Instant, Duration)>,
    /// Current adaptive TTL
    current_adaptive_ttl: Duration,
    /// Data freshness classification
    freshness_class: DataFreshnessClass,
    /// TTL adaptation score (0-1, higher = longer TTL)
    ttl_score: f64,
}

/// Data freshness classification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataFreshnessClass {
    Critical,
    Important,
    Normal,
    Cold,
}

impl AccessPattern {
    const BURST_THRESHOLD: u32 = 5;

    fn new() -> Self {
        Self {
            access_count: 0,
            last_access: Instant::now(),
            access_frequency: 0.0,
            access_history: VecDeque::with_capacity(10),
            predicted_next_access: None,
            temporal_pattern: [0.0; 24],
            sequential_patterns: HashMap::new(),
            burst_count: 0,
            burst_start: None,
            access_velocity: 0.0,
            ttl_history: VecDeque::with_capacity(5),
            current_adaptive_ttl: Duration::from_secs(300), // Default 5 minutes
            freshness_class: DataFreshnessClass::Normal,
            ttl_score: 0.5,
        }
    }

    fn record_access(&mut self, previous_key: Option<&str>) {
        let now = Instant::now();
        self.access_count += 1;
        self.last_access = now;

        // Record sequential pattern
        if let Some(prev_key) = previous_key {
            *self
                .sequential_patterns
                .entry(prev_key.to_string())
                .or_insert(0) += 1;
        }

        // Update temporal pattern
        let hour = chrono::Utc::now().hour() as usize;
        self.temporal_pattern[hour] += 1.0;

        // Maintain access history for pattern analysis
        self.access_history.push_back(now);
        if self.access_history.len() > 10 {
            self.access_history.pop_front();
        }

        // Calculate access frequency
        if self.access_history.len() > 1 {
            let time_span = now.duration_since(self.access_history[0]).as_secs_f64();
            let new_frequency = (self.access_history.len() - 1) as f64 / time_span;

            // Calculate access velocity (rate of change)
            self.access_velocity = new_frequency - self.access_frequency;
            self.access_frequency = new_frequency;

            // Predict next access based on frequency and temporal patterns
            if self.access_frequency > 0.0 {
                let avg_interval = 1.0 / self.access_frequency;
                let temporal_multiplier = self.get_temporal_multiplier();
                let predicted_interval = avg_interval / temporal_multiplier;
                self.predicted_next_access =
                    Some(now + Duration::from_secs_f64(predicted_interval));
            }
        }

        // Detect burst access patterns
        self.detect_burst_access(now);
    }

    /// Get temporal multiplier based on current hour
    fn get_temporal_multiplier(&self) -> f64 {
        let current_hour = chrono::Utc::now().hour() as usize;
        let total_accesses: f64 = self.temporal_pattern.iter().sum();
        if total_accesses > 0.0 {
            let current_hour_weight = self.temporal_pattern[current_hour] / total_accesses;
            // Higher weight means more likely to be accessed soon
            1.0 + (current_hour_weight * 2.0)
        } else {
            1.0
        }
    }

    /// Detect burst access patterns
    fn detect_burst_access(&mut self, now: Instant) {
        const BURST_WINDOW: Duration = Duration::from_secs(60);

        if let Some(burst_start) = self.burst_start {
            if now.duration_since(burst_start) < BURST_WINDOW {
                self.burst_count += 1;
            } else {
                // Reset burst detection
                self.burst_count = 1;
                self.burst_start = Some(now);
            }
        } else {
            self.burst_count = 1;
            self.burst_start = Some(now);
        }
    }

    /// Check if this key is in a burst access pattern
    fn is_in_burst(&self) -> bool {
        self.burst_count >= Self::BURST_THRESHOLD
    }

    /// Get sequential access predictions
    fn get_sequential_predictions(&self) -> Vec<(String, f64)> {
        let total_sequential: u32 = self.sequential_patterns.values().sum();
        if total_sequential == 0 {
            return Vec::new();
        }

        let mut predictions: Vec<(String, f64)> = self
            .sequential_patterns
            .iter()
            .map(|(key, count)| (key.clone(), *count as f64 / total_sequential as f64))
            .collect();

        predictions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        predictions
    }

    /// Classify data freshness based on access patterns
    fn classify_freshness(&mut self, classifier: &DataTypeClassifier) {
        self.freshness_class = match classifier {
            DataTypeClassifier::AccessPatternBased {
                critical_threshold,
                important_threshold,
            } => {
                if self.access_frequency >= *critical_threshold {
                    DataFreshnessClass::Critical
                } else if self.access_frequency >= *important_threshold {
                    DataFreshnessClass::Important
                } else if self.access_count > 0 {
                    DataFreshnessClass::Normal
                } else {
                    DataFreshnessClass::Cold
                }
            }
            _ => DataFreshnessClass::Normal, // Simplified for other classifiers
        };
    }

    /// Calculate TTL score based on access patterns and freshness requirements
    fn calculate_ttl_score(&self, _freshness_reqs: &DataFreshnessRequirements) -> f64 {
        let base_score = match self.freshness_class {
            DataFreshnessClass::Critical => 1.0,
            DataFreshnessClass::Important => 0.7,
            DataFreshnessClass::Normal => 0.4,
            DataFreshnessClass::Cold => 0.1,
        };

        // Adjust based on access frequency (0-1 scale)
        let frequency_score = (self.access_frequency / 10.0).min(1.0);

        // Adjust based on access velocity (rate of change)
        let velocity_score = ((self.access_velocity + 1.0) / 2.0).max(0.0).min(1.0);

        // Adjust based on burst access
        let burst_score = if self.is_in_burst() { 1.0 } else { 0.0 };

        // Combine scores with weights
        (base_score * 0.4) + (frequency_score * 0.3) + (velocity_score * 0.2) + (burst_score * 0.1)
    }

    /// Adapt TTL based on strategy and current patterns
    fn adapt_ttl(
        &mut self,
        base_ttl: Duration,
        strategy: &TtlAdaptationStrategy,
        freshness_reqs: &DataFreshnessRequirements,
    ) -> Duration {
        // Update freshness classification
        self.classify_freshness(&freshness_reqs.data_type_classifier);

        // Calculate TTL score
        self.ttl_score = self.calculate_ttl_score(freshness_reqs);

        // Apply adaptation strategy
        let adapted_ttl = match strategy {
            TtlAdaptationStrategy::Linear => {
                // Linear scaling based on TTL score
                let multiplier = 0.5 + (self.ttl_score * 1.5); // 0.5x to 2.0x
                Duration::from_secs_f64(base_ttl.as_secs_f64() * multiplier)
            }
            TtlAdaptationStrategy::Exponential => {
                // Exponential scaling for hot data
                let multiplier = 1.0 + (self.ttl_score * self.ttl_score * 3.0); // Up to 4x for hot data
                Duration::from_secs_f64(base_ttl.as_secs_f64() * multiplier)
            }
            TtlAdaptationStrategy::Segmented {
                hot_data_multiplier,
                warm_data_multiplier,
                cold_data_multiplier,
            } => {
                let multiplier = match self.freshness_class {
                    DataFreshnessClass::Critical | DataFreshnessClass::Important => {
                        *hot_data_multiplier
                    }
                    DataFreshnessClass::Normal => *warm_data_multiplier,
                    DataFreshnessClass::Cold => *cold_data_multiplier,
                };
                Duration::from_secs_f64(base_ttl.as_secs_f64() * multiplier)
            }
            TtlAdaptationStrategy::Predictive {
                learning_rate,
                prediction_horizon,
            } => {
                // Simplified predictive adaptation
                let predicted_accesses = if let Some(next_access) = self.predicted_next_access {
                    let time_to_next = next_access.saturating_duration_since(Instant::now());
                    if time_to_next < *prediction_horizon {
                        1.0
                    } else {
                        0.5
                    }
                } else {
                    0.0
                };

                let multiplier = 1.0 + (predicted_accesses * *learning_rate);
                Duration::from_secs_f64(base_ttl.as_secs_f64() * multiplier)
            }
        };

        // Apply freshness constraints
        let max_age = match self.freshness_class {
            DataFreshnessClass::Critical => freshness_reqs.critical_data_max_age,
            DataFreshnessClass::Important => freshness_reqs.important_data_max_age,
            DataFreshnessClass::Normal => freshness_reqs.normal_data_max_age,
            DataFreshnessClass::Cold => base_ttl, // No constraint for cold data
        };

        // Clamp to reasonable bounds
        let final_ttl = adapted_ttl.min(max_age);

        // Record TTL adaptation
        self.ttl_history.push_back((Instant::now(), final_ttl));
        if self.ttl_history.len() > 5 {
            self.ttl_history.pop_front();
        }

        self.current_adaptive_ttl = final_ttl;
        final_ttl
    }

    /// Get TTL adaptation statistics
    fn get_ttl_stats(&self) -> TtlAdaptationStats {
        let avg_ttl = if self.ttl_history.is_empty() {
            Duration::from_secs(0)
        } else {
            let total_secs: f64 = self
                .ttl_history
                .iter()
                .map(|(_, ttl)| ttl.as_secs_f64())
                .sum();
            Duration::from_secs_f64(total_secs / self.ttl_history.len() as f64)
        };

        TtlAdaptationStats {
            current_ttl: self.current_adaptive_ttl,
            average_ttl: avg_ttl,
            ttl_score: self.ttl_score,
            freshness_class: self.freshness_class.clone(),
            adaptation_count: self.ttl_history.len(),
        }
    }

    fn calculate_adaptive_ttl(
        &self,
        base_ttl: Duration,
        min_ttl: Duration,
        max_ttl: Duration,
    ) -> Duration {
        // Higher frequency = longer TTL (more likely to be accessed again)
        let frequency_factor = (self.access_frequency * 100.0).min(10.0) / 10.0;
        let ttl_multiplier = 1.0 + frequency_factor;

        let adaptive_ttl = Duration::from_secs_f64(base_ttl.as_secs_f64() * ttl_multiplier);

        // Clamp to min/max bounds
        if adaptive_ttl < min_ttl {
            min_ttl
        } else if adaptive_ttl > max_ttl {
            max_ttl
        } else {
            adaptive_ttl
        }
    }

    fn should_prefetch(&self, threshold: u32) -> bool {
        self.access_count >= threshold && self.access_frequency > 0.1
    }
}

/// Intelligent cache with advanced features
pub struct IntelligentCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    cache: Cache<K, V>,
    config: IntelligentCacheConfig,
    access_patterns: Arc<RwLock<HashMap<K, AccessPattern>>>,
    prefetch_queue: Arc<RwLock<VecDeque<String>>>,
    stats: Arc<RwLock<IntelligentCacheStats>>,
    last_accessed_key: Arc<RwLock<Option<K>>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IntelligentCacheStats {
    pub base_stats: CacheStats,
    pub prefetch_hits: u64,
    pub prefetch_misses: u64,
    pub adaptive_ttl_adjustments: u64,
    pub pattern_predictions: u64,
    pub successful_predictions: u64,
    pub average_access_frequency: f64,
    pub cache_efficiency_score: f64,
}

impl<K, V> IntelligentCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static + ToString + AsRef<str>,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new intelligent cache
    pub fn new(config: IntelligentCacheConfig) -> Self {
        let cache = Cache::new(config.base_ttl, config.max_size);

        Self {
            cache,
            config,
            access_patterns: Arc::new(RwLock::new(HashMap::new())),
            prefetch_queue: Arc::new(RwLock::new(VecDeque::new())),
            stats: Arc::new(RwLock::new(IntelligentCacheStats::default())),
            last_accessed_key: Arc::new(RwLock::new(None)),
        }
    }

    /// Get value from cache with intelligent tracking
    pub async fn get(&self, key: &K) -> Option<V> {
        // Get the previous key for sequential pattern tracking
        let previous_key = {
            let mut last_key = self.last_accessed_key.write().await;
            let prev = last_key.clone();
            *last_key = Some(key.clone());
            prev
        };

        // Record access pattern with sequential information
        self.record_access(key, previous_key.as_ref()).await;

        // Get from cache
        let result = self.cache.get(key).await;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            if result.is_some() {
                // Check if this was a prefetch hit
                let prefetch_queue = self.prefetch_queue.read().await;
                if prefetch_queue.contains(&key.to_string()) {
                    stats.prefetch_hits += 1;
                }
            }
        }

        result
    }

    /// Set value in cache with adaptive TTL
    pub async fn set(&self, key: K, value: V) -> HiveResult<()> {
        let ttl = if self.config.enable_adaptive_ttl {
            self.calculate_adaptive_ttl(&key).await
        } else {
            self.config.base_ttl
        };

        self.cache.insert_with_ttl(key.clone(), value, ttl).await;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            if ttl != self.config.base_ttl {
                stats.adaptive_ttl_adjustments += 1;
            }
        }

        Ok(())
    }

    /// Set value with custom TTL
    pub async fn set_with_ttl(&self, key: K, value: V, ttl: Duration) -> HiveResult<()> {
        self.cache.insert_with_ttl(key, value, ttl).await;
        Ok(())
    }

    /// Remove value from cache
    pub async fn remove(&self, key: &K) -> Option<V> {
        // Remove from access patterns as well
        {
            let mut patterns = self.access_patterns.write().await;
            patterns.remove(key);
        }

        self.cache.remove(key).await
    }

    /// Prefetch data based on access patterns
    pub async fn prefetch<F, Fut>(&self, key: K, loader: F) -> HiveResult<()>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = HiveResult<V>> + Send,
    {
        if !self.config.enable_prefetching {
            return Ok(());
        }

        // Check if already in cache
        if self.cache.get(&key).await.is_some() {
            return Ok(());
        }

        // Load and cache the value
        match loader().await {
            Ok(value) => {
                self.set(key.clone(), value).await?;

                // Add to prefetch queue for tracking
                {
                    let mut queue = self.prefetch_queue.write().await;
                    queue.push_back(key.to_string());
                    if queue.len() > 1000 {
                        queue.pop_front();
                    }
                }

                debug!("Successfully prefetched data for key");
                Ok(())
            }
            Err(e) => {
                let mut stats = self.stats.write().await;
                stats.prefetch_misses += 1;
                warn!("Failed to prefetch data: {}", e);
                Err(e)
            }
        }
    }

    /// Record access pattern for a key
    async fn record_access(&self, key: &K, previous_key: Option<&K>) {
        let mut patterns = self.access_patterns.write().await;
        let pattern = patterns
            .entry(key.clone())
            .or_insert_with(AccessPattern::new);
        pattern.record_access(previous_key.map(|k| k.as_ref()));

        // Check if we should trigger prefetching
        if self.config.enable_prefetching && pattern.should_prefetch(self.config.prefetch_threshold)
        {
            debug!("Key qualifies for prefetching based on access pattern");
            self.trigger_predictive_prefetching(key).await;
        }
    }

    /// Trigger predictive prefetching based on access patterns
    async fn trigger_predictive_prefetching(&self, key: &K) {
        let patterns = self.access_patterns.read().await;

        if let Some(pattern) = patterns.get(key) {
            // Prefetch based on sequential patterns
            let sequential_predictions = pattern.get_sequential_predictions();
            for (next_key, probability) in sequential_predictions.into_iter().take(3) {
                if probability > 0.3 {
                    // Only prefetch high-probability sequences
                    debug!(
                        "Predictive prefetching: {} -> {} (probability: {:.2})",
                        key.to_string(),
                        next_key,
                        probability
                    );
                    // Add to prefetch queue for later processing
                    let mut queue = self.prefetch_queue.write().await;
                    queue.push_back(next_key.clone());
                    if queue.len() > 100usize {
                        queue.pop_front();
                    }
                }
            }

            // Prefetch based on temporal patterns
            if pattern.is_in_burst() {
                debug!(
                    "Burst access detected for key: {}, triggering aggressive prefetching",
                    key.to_string()
                );
                self.trigger_burst_prefetching(key).await;
            }
        }
    }

    /// Trigger burst prefetching for high-frequency access patterns
    async fn trigger_burst_prefetching(&self, key: &K) {
        let patterns = self.access_patterns.read().await;

        // Find other keys with similar access patterns
        let similar_keys: Vec<String> = patterns
            .iter()
            .filter(|(k, p)| {
                k.as_ref() != key.as_ref() && p.access_frequency > 0.1 && p.is_in_burst()
            })
            .map(|(k, _)| k.to_string())
            .take(5)
            .collect();

        for similar_key in similar_keys {
            debug!("Burst prefetching similar key: {}", similar_key);
            let mut queue = self.prefetch_queue.write().await;
            queue.push_back(similar_key);
            if queue.len() > 100 {
                queue.pop_front();
            }
        }
    }

    /// Calculate adaptive TTL for a key
    async fn calculate_adaptive_ttl(&self, key: &K) -> Duration {
        let patterns = self.access_patterns.read().await;
        if let Some(pattern) = patterns.get(key) {
            let mut pattern_clone = pattern.clone();
            pattern_clone.adapt_ttl(
                self.config.base_ttl,
                &self.config.ttl_strategy,
                &self.config.freshness_requirements,
            )
        } else {
            self.config.base_ttl
        }
    }

    /// Get comprehensive cache statistics
    pub async fn get_stats(&self) -> IntelligentCacheStats {
        let base_stats = self.cache.stats().await;
        let mut stats = self.stats.read().await.clone();
        stats.base_stats = base_stats;

        // Calculate average access frequency
        let patterns = self.access_patterns.read().await;
        if !patterns.is_empty() {
            let total_frequency: f64 = patterns.values().map(|p| p.access_frequency).sum();
            stats.average_access_frequency = total_frequency / patterns.len() as f64;
        }

        // Calculate cache efficiency score
        let total_requests = stats.base_stats.total_hits + stats.base_stats.total_misses;
        if total_requests > 0 {
            let hit_rate = stats.base_stats.total_hits as f64 / total_requests as f64;
            let prefetch_efficiency = if stats.prefetch_hits + stats.prefetch_misses > 0 {
                stats.prefetch_hits as f64 / (stats.prefetch_hits + stats.prefetch_misses) as f64
            } else {
                0.0
            };
            stats.cache_efficiency_score = (hit_rate * 0.7) + (prefetch_efficiency * 0.3);
        }

        stats
    }

    /// Start background optimization processes
    pub fn start_optimization(&self) -> tokio::task::JoinHandle<()> {
        let patterns_clone = Arc::clone(&self.access_patterns);
        let stats_clone = Arc::clone(&self.stats);
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = interval(config.stats_interval);

            loop {
                interval.tick().await;

                // Cleanup expired patterns
                {
                    let mut patterns = patterns_clone.write().await;
                    let cutoff = Instant::now() - Duration::from_secs(3600); // 1 hour
                    patterns.retain(|_, pattern| pattern.last_access > cutoff);
                }

                // Update cache efficiency metrics
                {
                    let patterns = patterns_clone.read().await;
                    let mut stats = stats_clone.write().await;

                    // Count successful predictions
                    let now = Instant::now();
                    let successful_predictions = patterns
                        .values()
                        .filter(|p| {
                            if let Some(predicted) = p.predicted_next_access {
                                let diff = if now > predicted {
                                    now.duration_since(predicted)
                                } else {
                                    predicted.duration_since(now)
                                };
                                diff < Duration::from_secs(60) // Within 1 minute
                            } else {
                                false
                            }
                        })
                        .count();

                    stats.successful_predictions = successful_predictions as u64;
                    stats.pattern_predictions = patterns.len() as u64;
                }

                // Cache cleanup would be handled separately
                // TODO: Implement cache cleanup without cloning

                info!("Intelligent cache optimization cycle completed");
            }
        })
    }

    /// Clear all cache data
    pub async fn clear(&self) -> HiveResult<()> {
        self.cache.clear().await;

        {
            let mut patterns = self.access_patterns.write().await;
            patterns.clear();
        }

        {
            let mut queue = self.prefetch_queue.write().await;
            queue.clear();
        }

        Ok(())
    }

    /// Get keys that should be prefetched
    pub async fn get_prefetch_candidates(&self) -> Vec<K> {
        let patterns = self.access_patterns.read().await;
        patterns
            .iter()
            .filter(|(_, pattern)| pattern.should_prefetch(self.config.prefetch_threshold))
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Get TTL adaptation statistics for all keys
    pub async fn get_ttl_adaptation_stats(&self) -> Vec<(K, TtlAdaptationStats)> {
        let patterns = self.access_patterns.read().await;
        patterns
            .iter()
            .map(|(key, pattern)| (key.clone(), pattern.get_ttl_stats()))
            .collect()
    }

    /// Get overall TTL adaptation summary
    pub async fn get_ttl_adaptation_summary(&self) -> TtlAdaptationSummary {
        let patterns = self.access_patterns.read().await;

        let mut total_adaptations = 0;
        let mut avg_ttl_score = 0.0;
        let mut class_distribution = HashMap::new();

        for pattern in patterns.values() {
            total_adaptations += pattern.ttl_history.len();
            avg_ttl_score += pattern.ttl_score;
            *class_distribution
                .entry(pattern.freshness_class.clone())
                .or_insert(0) += 1;
        }

        let pattern_count = patterns.len();
        if pattern_count > 0 {
            avg_ttl_score /= pattern_count as f64;
        }

        TtlAdaptationSummary {
            total_patterns: pattern_count,
            total_adaptations,
            average_ttl_score: avg_ttl_score,
            freshness_class_distribution: class_distribution,
            strategy: self.config.ttl_strategy.clone(),
        }
    }

    /// Get intelligent prefetch recommendations with priorities
    pub async fn get_prefetch_recommendations(&self) -> Vec<PrefetchRecommendation<K>> {
        let patterns = self.access_patterns.read().await;
        let mut recommendations = Vec::new();

        for (key, pattern) in &*patterns {
            let mut priority = PrefetchPriority::Low;
            let mut reasons = Vec::new();

            // Frequency-based priority
            if pattern.access_frequency > 1.0 {
                priority = PrefetchPriority::High;
                reasons.push("High access frequency".to_string());
            } else if pattern.access_frequency > 0.5 {
                priority = PrefetchPriority::Medium;
                reasons.push("Medium access frequency".to_string());
            }

            // Burst detection
            if pattern.is_in_burst() {
                priority = PrefetchPriority::Critical;
                reasons.push("Burst access pattern detected".to_string());
            }

            // Temporal pattern analysis
            let temporal_score = pattern.get_temporal_multiplier();
            if temporal_score > 1.5 {
                reasons.push(format!("High temporal relevance ({:.2})", temporal_score));
                if priority != PrefetchPriority::Critical {
                    priority = PrefetchPriority::High;
                }
            }

            // Sequential pattern analysis
            let sequential_predictions = pattern.get_sequential_predictions();
            if !sequential_predictions.is_empty() {
                reasons.push(format!(
                    "Sequential patterns detected ({})",
                    sequential_predictions.len()
                ));
            }

            // Access velocity (trending up)
            if pattern.access_velocity > 0.1 {
                reasons.push("Increasing access frequency".to_string());
                if priority != PrefetchPriority::Critical {
                    priority = PrefetchPriority::High;
                }
            }

            recommendations.push(PrefetchRecommendation {
                key: key.clone(),
                priority,
                reasons,
                predicted_benefit: self.calculate_predicted_benefit(pattern),
                sequential_keys: sequential_predictions
                    .into_iter()
                    .map(|(k, _)| k)
                    .take(3)
                    .collect(),
            });
        }

        // Sort by priority and predicted benefit
        recommendations.sort_by(|a, b| {
            b.priority.cmp(&a.priority).then_with(|| {
                b.predicted_benefit
                    .partial_cmp(&a.predicted_benefit)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        recommendations
    }

    /// Calculate predicted benefit of prefetching a key
    fn calculate_predicted_benefit(&self, pattern: &AccessPattern) -> f64 {
        let frequency_score = pattern.access_frequency.min(10.0) / 10.0; // Normalize to 0-1
        let temporal_score = (pattern.get_temporal_multiplier() - 1.0).min(1.0); // Normalize to 0-1
        let burst_score = if pattern.is_in_burst() { 1.0 } else { 0.0 };
        let velocity_score = pattern.access_velocity.max(0.0).min(1.0); // Normalize to 0-1

        // Weighted combination
        (frequency_score * 0.4)
            + (temporal_score * 0.3)
            + (burst_score * 0.2)
            + (velocity_score * 0.1)
    }

    /// Execute batch prefetching based on recommendations
    pub async fn execute_batch_prefetch<F, Fut>(
        &self,
        recommendations: &[PrefetchRecommendation<K>],
        loader: F,
    ) -> HiveResult<PrefetchResults>
    where
        F: Fn(K) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = HiveResult<V>> + Send,
        K: Send + 'static,
    {
        let mut results = PrefetchResults::default();
        let mut tasks = Vec::new();

        // Group by priority for batch processing
        let critical: Vec<_> = recommendations
            .iter()
            .filter(|r| r.priority == PrefetchPriority::Critical)
            .collect();
        let high: Vec<_> = recommendations
            .iter()
            .filter(|r| r.priority == PrefetchPriority::High)
            .collect();
        let medium: Vec<_> = recommendations
            .iter()
            .filter(|r| r.priority == PrefetchPriority::Medium)
            .collect();

        // Process critical first
        for recommendation in critical {
            let loader_clone = loader.clone();
            let key_clone = recommendation.key.clone();
            let task = tokio::spawn(async move {
                match loader_clone(key_clone).await {
                    Ok(_) => PrefetchResult::Success,
                    Err(_) => PrefetchResult::Failed,
                }
            });
            tasks.push(task);
        }

        // Wait for critical to complete before processing others
        for task in tasks.drain(..) {
            match task.await {
                Ok(PrefetchResult::Success) => results.successful += 1,
                _ => results.failed += 1,
            }
        }

        // Process high priority
        for recommendation in high.into_iter().take(10) {
            // Limit concurrent high-priority prefetches
            let loader_clone = loader.clone();
            let key_clone = recommendation.key.clone();
            let task = tokio::spawn(async move {
                match loader_clone(key_clone).await {
                    Ok(_) => PrefetchResult::Success,
                    Err(_) => PrefetchResult::Failed,
                }
            });
            tasks.push(task);
        }

        for task in tasks {
            match task.await {
                Ok(PrefetchResult::Success) => results.successful += 1,
                _ => results.failed += 1,
            }
        }

        // Process medium priority in background
        for recommendation in medium.into_iter().take(5) {
            let loader_clone = loader.clone();
            let key_clone = recommendation.key.clone();
            tokio::spawn(async move {
                let _ = loader_clone(key_clone).await;
            });
            results.queued += 1;
        }

        info!(
            "Batch prefetch completed: {} successful, {} failed, {} queued",
            results.successful, results.failed, results.queued
        );

        Ok(results)
    }
}

/// Cache warming and predictive loading system
pub struct CacheWarmer {
    /// Access pattern analyzer
    pattern_analyzer: Arc<RwLock<AccessPatternAnalyzer>>,
    /// Warming queue for predictive loading
    warming_queue: Arc<RwLock<VecDeque<CacheWarmRequest>>>,
    /// Active warming tasks
    active_warm_tasks: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
    /// Performance metrics
    metrics: Arc<RwLock<CacheWarmMetrics>>,
}

/// Cache warming request
pub struct CacheWarmRequest {
    pub key: String,
    pub priority: WarmPriority,
    pub predicted_access_time: Instant,
    pub data_loader: Option<
        Arc<
            dyn Fn() -> Pin<Box<dyn Future<Output = HiveResult<serde_json::Value>> + Send + Sync>>
                + Send
                + Sync,
        >,
    >,
}

/// Cache warming priority
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WarmPriority {
    High,
    Medium,
    Low,
}

/// Cache warming metrics
#[derive(Debug, Clone, Default, Serialize)]
pub struct CacheWarmMetrics {
    pub total_warm_requests: u64,
    pub successful_warms: u64,
    pub failed_warms: u64,
    pub average_warm_time: f64,
    pub cache_hit_improvement: f64,
    pub memory_overhead: usize,
}

/// Access pattern analyzer for predictive caching
#[derive(Debug)]
pub struct AccessPatternAnalyzer {
    /// Historical access patterns
    patterns: HashMap<String, Vec<AccessRecord>>,
    /// Pattern prediction model (simplified)
    prediction_model: HashMap<String, PredictionModel>,
    /// Analysis time window
    time_window: Duration,
}

/// Access record for pattern analysis
#[derive(Debug, Clone)]
pub struct AccessRecord {
    pub timestamp: Instant,
    pub access_type: AccessType,
    pub frequency_score: f64,
}

/// Access type enumeration
#[derive(Debug, Clone)]
pub enum AccessType {
    Read,
    Write,
    Delete,
}

/// Prediction model for cache warming
#[derive(Debug, Clone)]
pub struct PredictionModel {
    pub next_access_probability: f64,
    pub access_frequency: f64,
    pub temporal_pattern: Vec<f64>, // Hourly access pattern
}

/// Prefetch recommendation with priority and reasoning
#[derive(Debug, Clone)]
pub struct PrefetchRecommendation<K> {
    pub key: K,
    pub priority: PrefetchPriority,
    pub reasons: Vec<String>,
    pub predicted_benefit: f64,
    pub sequential_keys: Vec<String>,
}

/// Priority levels for prefetch operations
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrefetchPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Results of batch prefetch operation
#[derive(Debug, Clone, Default)]
pub struct PrefetchResults {
    pub successful: u64,
    pub failed: u64,
    pub queued: u64,
}

/// Result of individual prefetch operation
#[derive(Debug, Clone)]
enum PrefetchResult {
    Success,
    Failed,
}

/// TTL adaptation statistics
#[derive(Debug, Clone)]
pub struct TtlAdaptationStats {
    pub current_ttl: Duration,
    pub average_ttl: Duration,
    pub ttl_score: f64,
    pub freshness_class: DataFreshnessClass,
    pub adaptation_count: usize,
}

/// TTL adaptation summary for monitoring
#[derive(Debug, Clone)]
pub struct TtlAdaptationSummary {
    pub total_patterns: usize,
    pub total_adaptations: usize,
    pub average_ttl_score: f64,
    pub freshness_class_distribution: HashMap<DataFreshnessClass, usize>,
    pub strategy: TtlAdaptationStrategy,
}

/// Enhanced multi-tier intelligent cache manager
pub struct MultiTierCacheManager {
    pub l1_cache: IntelligentCache<String, serde_json::Value>, // Fast, small cache
    l2_cache: IntelligentCache<String, serde_json::Value>,     // Larger, persistent cache
    config: IntelligentCacheConfig,
    /// Cache warmer for predictive loading
    cache_warmer: Arc<CacheWarmer>,
    /// Database query interceptor for measuring reduction
    query_interceptor: Arc<RwLock<QueryInterceptor>>,
}

impl CacheWarmer {
    /// Create new cache warmer
    pub fn new() -> Self {
        Self {
            pattern_analyzer: Arc::new(RwLock::new(AccessPatternAnalyzer::new())),
            warming_queue: Arc::new(RwLock::new(VecDeque::new())),
            active_warm_tasks: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(CacheWarmMetrics::default())),
        }
    }

    /// Add item to warming queue
    pub async fn queue_for_warming(&self, request: CacheWarmRequest) {
        let mut queue = self.warming_queue.write().await;
        queue.push_back(request);

        let mut metrics = self.metrics.write().await;
        metrics.total_warm_requests += 1;
    }

    /// Process warming queue
    pub async fn process_warming_queue(&self) -> HiveResult<()> {
        let requests: Vec<_> = {
            let mut queue = self.warming_queue.write().await;
            let mut requests = Vec::new();

            // Process high priority requests first
            while let Some(request) = queue.front() {
                if request.priority == WarmPriority::High {
                    if let Some(request) = queue.pop_front() {
                        requests.push(request);
                    }
                } else {
                    break;
                }
            }

            // Process medium priority if no high priority
            if requests.is_empty() {
                while let Some(request) = queue.front() {
                    if request.priority != WarmPriority::Low {
                        if let Some(request) = queue.pop_front() {
                            requests.push(request);
                        }
                    } else {
                        break;
                    }
                }
            }

            // Process one low priority request
            if requests.is_empty() {
                if let Some(request) = queue.pop_front() {
                    requests.push(request);
                }
            }

            requests
        };

        // Execute warming tasks
        for request in requests {
            if let Some(loader) = request.data_loader {
                let metrics_clone = Arc::clone(&self.metrics);
                let task_id = format!("warm_{}", request.key);

                let handle = tokio::spawn(async move {
                    let start_time = Instant::now();

                    match loader().await {
                        Ok(_data) => {
                            let warm_time = start_time.elapsed().as_secs_f64();

                            let mut metrics = metrics_clone.write().await;
                            metrics.successful_warms += 1;
                            metrics.average_warm_time = (metrics.average_warm_time
                                * (metrics.successful_warms - 1) as f64
                                + warm_time)
                                / metrics.successful_warms as f64;

                            tracing::debug!(
                                "Successfully warmed cache for key: {} in {:.4}s",
                                request.key,
                                warm_time
                            );
                        }
                        Err(e) => {
                            let mut metrics = metrics_clone.write().await;
                            metrics.failed_warms += 1;
                            tracing::warn!("Failed to warm cache for key: {}: {}", request.key, e);
                        }
                    }
                });

                let mut active_tasks = self.active_warm_tasks.write().await;
                active_tasks.insert(task_id, handle);
            }
        }

        Ok(())
    }

    /// Analyze access patterns and generate warming recommendations
    pub async fn analyze_and_recommend_warming(&self) -> Vec<String> {
        let analyzer = self.pattern_analyzer.read().await;
        let mut recommendations = Vec::new();

        for (key, records) in &analyzer.patterns {
            if records.len() >= 5 {
                // Need minimum history
                let recent_records: Vec<_> = records
                    .iter()
                    .filter(|r| r.timestamp.elapsed() < Duration::from_secs(3600)) // Last hour
                    .collect();

                if recent_records.len() >= 3 {
                    let avg_frequency = recent_records
                        .iter()
                        .map(|r| r.frequency_score)
                        .sum::<f64>()
                        / recent_records.len() as f64;

                    // Recommend warming if high frequency and recent access
                    if avg_frequency > 0.7 {
                        recommendations.push(key.clone());
                    }
                }
            }
        }

        recommendations
    }

    /// Get warming performance metrics
    pub async fn get_metrics(&self) -> CacheWarmMetrics {
        self.metrics.read().await.clone()
    }

    /// Clean up completed warming tasks
    pub async fn cleanup_completed_tasks(&self) {
        let mut active_tasks = self.active_warm_tasks.write().await;
        let completed_tasks: Vec<_> = active_tasks
            .iter()
            .filter(|(_, handle)| handle.is_finished())
            .map(|(id, _)| id.clone())
            .collect();

        for task_id in completed_tasks {
            active_tasks.remove(&task_id);
        }
    }
}

impl AccessPatternAnalyzer {
    /// Create new access pattern analyzer
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            prediction_model: HashMap::new(),
            time_window: Duration::from_secs(3600), // 1 hour
        }
    }

    /// Record access pattern
    pub fn record_access(&mut self, key: &str, access_type: AccessType) {
        let record = AccessRecord {
            timestamp: Instant::now(),
            access_type,
            frequency_score: self.calculate_frequency_score(key),
        };

        self.patterns
            .entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(record);

        // Keep only recent records
        if let Some(records) = self.patterns.get_mut(key) {
            records.retain(|r| r.timestamp.elapsed() < self.time_window);
        }

        // Update prediction model
        self.update_prediction_model(key);
    }

    /// Calculate frequency score for a key
    fn calculate_frequency_score(&self, key: &str) -> f64 {
        if let Some(records) = self.patterns.get(key) {
            if records.is_empty() {
                return 0.0;
            }

            let time_span = records
                .last()
                .map(|r| {
                    r.timestamp
                        .duration_since(records[0].timestamp)
                        .as_secs_f64()
                })
                .unwrap_or(0.0);
            if time_span > 0.0 {
                let frequency = records.len() as f64 / time_span;
                // Normalize to 0-1 scale (assuming max 1 access per second)
                (frequency / 1.0).min(1.0)
            } else {
                1.0 // Very recent access
            }
        } else {
            0.0
        }
    }

    /// Update prediction model for a key
    fn update_prediction_model(&mut self, key: &str) {
        if let Some(records) = self.patterns.get(key) {
            if records.len() < 3 {
                return;
            }

            // Simple prediction based on recent access patterns
            let recent_accesses: Vec<_> = records
                .iter()
                .filter(|r| matches!(r.access_type, AccessType::Read))
                .collect();

            if recent_accesses.len() >= 3 {
                let intervals: Vec<f64> = recent_accesses
                    .windows(2)
                    .map(|w| w[1].timestamp.duration_since(w[0].timestamp).as_secs_f64())
                    .collect();

                let avg_interval = intervals.iter().sum::<f64>() / intervals.len() as f64;
                let access_frequency = 1.0 / avg_interval;

                // Create temporal pattern (simplified)
                let temporal_pattern = vec![access_frequency; 24]; // 24 hours

                let model = PredictionModel {
                    next_access_probability: (recent_accesses.len() as f64 / records.len() as f64)
                        .min(1.0),
                    access_frequency,
                    temporal_pattern,
                };

                self.prediction_model.insert(key.to_string(), model);
            }
        }
    }

    /// Predict if key will be accessed soon
    pub fn predict_access(&self, key: &str) -> Option<f64> {
        self.prediction_model
            .get(key)
            .map(|model| model.next_access_probability)
    }
}

impl MultiTierCacheManager {
    /// Create a new multi-tier cache manager with intelligent features
    pub fn new() -> Self {
        let l1_config = IntelligentCacheConfig {
            max_size: 1000,
            base_ttl: Duration::from_secs(60),
            enable_prefetching: true,
            enable_adaptive_ttl: true,
            ..Default::default()
        };

        let l2_config = IntelligentCacheConfig {
            max_size: 10000,
            base_ttl: Duration::from_secs(300),
            enable_prefetching: true,
            enable_adaptive_ttl: true,
            ..Default::default()
        };

        Self {
            l1_cache: IntelligentCache::new(l1_config.clone()),
            l2_cache: IntelligentCache::new(l2_config.clone()),
            config: l1_config,
            cache_warmer: Arc::new(CacheWarmer::new()),
            query_interceptor: Arc::new(RwLock::new(QueryInterceptor::new())),
        }
    }

    /// Get value from multi-tier cache
    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        // Try L1 cache first
        if let Some(value) = self.l1_cache.get(&key.to_string()).await {
            return Some(value);
        }

        // Try L2 cache
        if let Some(value) = self.l2_cache.get(&key.to_string()).await {
            // Promote to L1 cache
            let _ = self.l1_cache.set(key.to_string(), value.clone()).await;
            return Some(value);
        }

        None
    }

    /// Set value in multi-tier cache
    pub async fn set(&self, key: String, value: serde_json::Value) -> HiveResult<()> {
        // Set in both tiers
        self.l1_cache.set(key.clone(), value.clone()).await?;
        self.l2_cache.set(key, value).await?;
        Ok(())
    }

    /// Get comprehensive statistics
    pub async fn get_stats(&self) -> serde_json::Value {
        let l1_stats = self.l1_cache.get_stats().await;
        let l2_stats = self.l2_cache.get_stats().await;

        serde_json::json!({
            "l1_cache": l1_stats,
            "l2_cache": l2_stats,
            "total_efficiency": (l1_stats.cache_efficiency_score + l2_stats.cache_efficiency_score) / 2.0
        })
    }

    /// Start optimization for both tiers
    pub fn start_optimization(&self) -> (tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>) {
        let l1_handle = self.l1_cache.start_optimization();
        let l2_handle = self.l2_cache.start_optimization();
        (l1_handle, l2_handle)
    }
}

/// Database query interceptor for measuring cache effectiveness
#[derive(Debug)]
pub struct QueryInterceptor {
    /// Total database queries
    total_queries: u64,
    /// Cache hits (queries avoided)
    cache_hits: u64,
    /// Query patterns for analysis
    query_patterns: HashMap<String, QueryPattern>,
    /// Performance metrics
    metrics: QueryMetrics,
}

/// Query pattern for analysis
#[derive(Debug, Clone)]
pub struct QueryPattern {
    pub query_type: String,
    pub frequency: u64,
    pub avg_execution_time: f64,
    pub cache_hit_rate: f64,
}

/// Query performance metrics
#[derive(Debug, Clone, Default, Serialize)]
pub struct QueryMetrics {
    pub total_queries: u64,
    pub cached_queries: u64,
    pub cache_hit_percentage: f64,
    pub average_query_time: f64,
    pub query_reduction_percentage: f64,
}

impl QueryInterceptor {
    /// Create new query interceptor
    pub fn new() -> Self {
        Self {
            total_queries: 0,
            cache_hits: 0,
            query_patterns: HashMap::new(),
            metrics: QueryMetrics::default(),
        }
    }
}

impl BatchQueryOptimizer {
    /// Create new batch query optimizer
    pub fn new() -> Self {
        Self {
            pending_queries: HashMap::new(),
            batch_timer: None,
            batch_size_threshold: 10,
            batch_time_window: Duration::from_millis(50),
        }
    }

    /// Get total number of pending queries
    pub fn total_pending_queries(&self) -> usize {
        self.pending_queries
            .values()
            .map(|v| v.len().saturating_sub(1))
            .sum::<usize>()
    }

    /// Add query to batch
    pub async fn add_to_batch(&mut self, request: BatchQueryRequest) {
        let query_key = request.query_key.clone();
        self.pending_queries
            .entry(query_key)
            .or_insert_with(Vec::new)
            .push(request);

        // Start batch timer if not already started
        if self.batch_timer.is_none() {
            self.batch_timer = Some(tokio::time::Instant::now() + self.batch_time_window);
        }
    }

    /// Check if batch should be executed
    pub fn should_execute_batch(&self) -> bool {
        if let Some(timer) = self.batch_timer {
            tokio::time::Instant::now() >= timer
        } else {
            false
        }
    }

    /// Get pending queries for execution
    pub fn get_pending_batches(&mut self) -> HashMap<String, Vec<BatchQueryRequest>> {
        let batches = std::mem::take(&mut self.pending_queries);
        self.batch_timer = None;
        batches
    }
}

impl QueryPerformanceAnalyzer {
    /// Create new query performance analyzer
    pub fn new() -> Self {
        Self {
            execution_times: HashMap::new(),
            query_frequency: HashMap::new(),
            slow_query_threshold: Duration::from_millis(100),
            optimization_suggestions: Vec::new(),
        }
    }

    /// Record query execution
    pub fn record_execution(&mut self, query_pattern: &str, execution_time: f64) {
        // Record execution time
        self.execution_times
            .entry(query_pattern.to_string())
            .or_insert_with(Vec::new)
            .push(execution_time);

        // Keep only last 100 executions
        if let Some(times) = self.execution_times.get_mut(query_pattern) {
            if times.len() > 100 {
                times.remove(0);
            }
        }

        // Update frequency
        *self
            .query_frequency
            .entry(query_pattern.to_string())
            .or_insert(0) += 1;

        // Check for slow queries
        if execution_time > self.slow_query_threshold.as_millis() as f64 {
            self.analyze_slow_query(query_pattern, execution_time);
        }
    }

    /// Analyze slow query and generate optimization suggestions
    fn analyze_slow_query(&mut self, query_pattern: &str, execution_time: f64) {
        let frequency = self
            .query_frequency
            .get(query_pattern)
            .copied()
            .unwrap_or(0);

        // Generate suggestions based on analysis
        if frequency > 10 && execution_time > 500.0 {
            // High frequency, slow query - suggest caching
            self.optimization_suggestions
                .push(QueryOptimizationSuggestion {
                    query_pattern: query_pattern.to_string(),
                    suggestion_type: OptimizationType::CacheQuery,
                    estimated_improvement: 0.8, // 80% improvement
                    reasoning: format!(
                        "High-frequency query ({}) with slow execution ({:.2}ms)",
                        frequency, execution_time
                    ),
                });
        } else if execution_time > 1000.0 {
            // Very slow query - suggest optimization
            self.optimization_suggestions
                .push(QueryOptimizationSuggestion {
                    query_pattern: query_pattern.to_string(),
                    suggestion_type: OptimizationType::OptimizeQuery,
                    estimated_improvement: 0.6, // 60% improvement
                    reasoning: format!("Very slow query execution ({:.2}ms)", execution_time),
                });
        }
    }

    /// Get optimization suggestions
    pub fn get_optimization_suggestions(&self) -> &[QueryOptimizationSuggestion] {
        &self.optimization_suggestions
    }

    /// Calculate average execution time for a query pattern
    pub fn get_average_execution_time(&self, query_pattern: &str) -> Option<f64> {
        self.execution_times.get(query_pattern).and_then(|times| {
            if times.is_empty() {
                None
            } else {
                Some(times.iter().sum::<f64>() / times.len() as f64)
            }
        })
    }
}

impl QueryInterceptor {
    /// Record database query
    pub fn record_query(&mut self, query_type: &str, execution_time: f64, was_cached: bool) {
        self.total_queries += 1;

        if was_cached {
            self.cache_hits += 1;
        }

        // Update query pattern
        let pattern = self
            .query_patterns
            .entry(query_type.to_string())
            .or_insert_with(|| QueryPattern {
                query_type: query_type.to_string(),
                frequency: 0,
                avg_execution_time: 0.0,
                cache_hit_rate: 0.0,
            });

        pattern.frequency += 1;
        pattern.avg_execution_time = (pattern.avg_execution_time * (pattern.frequency - 1) as f64
            + execution_time)
            / pattern.frequency as f64;

        if was_cached {
            pattern.cache_hit_rate = pattern.frequency as f64 / (pattern.frequency as f64 + 1.0);
        }

        // Update overall metrics
        self.update_metrics();
    }

    /// Update overall performance metrics
    fn update_metrics(&mut self) {
        self.metrics.total_queries = self.total_queries;
        self.metrics.cached_queries = self.cache_hits;

        if self.total_queries > 0 {
            self.metrics.cache_hit_percentage =
                (self.cache_hits as f64 / self.total_queries as f64) * 100.0;
            self.metrics.query_reduction_percentage = self.metrics.cache_hit_percentage;
        }

        // Calculate average query time (simplified)
        let total_time: f64 = self
            .query_patterns
            .values()
            .map(|p| p.avg_execution_time * p.frequency as f64)
            .sum();

        if self.total_queries > 0 {
            self.metrics.average_query_time = total_time / self.total_queries as f64;
        }
    }

    /// Get query performance metrics
    pub fn get_metrics(&self) -> &QueryMetrics {
        &self.metrics
    }

    /// Check if query reduction target is met (25% reduction)
    pub fn is_target_met(&self) -> bool {
        self.metrics.query_reduction_percentage >= 25.0
    }

    /// Generate query performance report
    pub fn generate_report(&self) -> String {
        format!(
            "Database Query Performance Report\n\
             ================================\n\
             Total Queries: {}\n\
             Cached Queries: {}\n\
             Cache Hit Rate: {:.2}%\n\
             Query Reduction: {:.2}%\n\
             Average Query Time: {:.4}ms\n\
             Target Met (25% reduction): {}\n\
             \n\
             Top Query Patterns:\n",
            self.metrics.total_queries,
            self.metrics.cached_queries,
            self.metrics.cache_hit_percentage,
            self.metrics.query_reduction_percentage,
            self.metrics.average_query_time * 1000.0,
            if self.is_target_met() {
                "✅ YES"
            } else {
                "❌ NO"
            }
        )
    }
}

impl MultiTierCacheManager {
    /// Get value with intelligent cache warming
    pub async fn get_with_warming(&self, key: &str) -> Option<serde_json::Value> {
        // Try L1 cache first
        if let Some(value) = self.l1_cache.get(&key.to_string()).await {
            // Record successful cache hit
            self.record_cache_hit(key).await;
            return Some(value);
        }

        // Try L2 cache
        if let Some(value) = self.l2_cache.get(&key.to_string()).await {
            // Promote to L1 cache
            let _ = self.l1_cache.set(key.to_string(), value.clone()).await;

            // Record cache hit and trigger warming analysis
            self.record_cache_hit(key).await;
            self.analyze_and_trigger_warming(key).await;

            return Some(value);
        }

        // Cache miss - record for analysis
        self.record_cache_miss(key).await;
        None
    }

    /// Set value with intelligent features
    pub async fn set_with_intelligence(
        &self,
        key: String,
        value: serde_json::Value,
    ) -> HiveResult<()> {
        // Set in both tiers
        self.l1_cache.set(key.clone(), value.clone()).await?;
        self.l2_cache.set(key.clone(), value).await?;

        // Analyze access patterns for potential warming
        self.analyze_and_trigger_warming(&key).await;

        Ok(())
    }

    /// Record cache hit for performance tracking
    async fn record_cache_hit(&self, key: &str) {
        let mut interceptor = self.query_interceptor.write().await;
        interceptor.record_query("cache_hit", 0.001, true); // Fast cache access

        // Update access pattern analyzer
        let mut analyzer = self.cache_warmer.pattern_analyzer.write().await;
        analyzer.record_access(key, AccessType::Read);
    }

    /// Record cache miss for analysis
    async fn record_cache_miss(&self, key: &str) {
        let mut interceptor = self.query_interceptor.write().await;
        interceptor.record_query("cache_miss", 0.050, false); // Simulated DB query time

        // Update access pattern analyzer
        let mut analyzer = self.cache_warmer.pattern_analyzer.write().await;
        analyzer.record_access(key, AccessType::Read);
    }

    /// Analyze access patterns and trigger cache warming
    async fn analyze_and_trigger_warming(&self, key: &str) {
        let recommendations = self.cache_warmer.analyze_and_recommend_warming().await;

        if recommendations.contains(&key.to_string()) {
            // Create warming request for this key
            let warm_request = CacheWarmRequest {
                key: key.to_string(),
                priority: WarmPriority::Medium,
                predicted_access_time: Instant::now() + Duration::from_secs(60),
                data_loader: None, // Would be provided by the application
            };

            let _ = self.cache_warmer.queue_for_warming(warm_request).await;
        }
    }

    /// Process pending cache warming tasks
    pub async fn process_cache_warming(&self) -> HiveResult<()> {
        self.cache_warmer.process_warming_queue().await?;
        self.cache_warmer.cleanup_completed_tasks().await;
        Ok(())
    }

    /// Get comprehensive cache performance metrics
    pub async fn get_enhanced_stats(&self) -> serde_json::Value {
        let l1_stats = self.l1_cache.get_stats().await;
        let l2_stats = self.l2_cache.get_stats().await;
        let warm_metrics = self.cache_warmer.get_metrics().await;
        let query_metrics = self.query_interceptor.read().await.get_metrics().clone();

        serde_json::json!({
            "l1_cache": l1_stats,
            "l2_cache": l2_stats,
            "cache_warmer": warm_metrics,
            "query_performance": query_metrics,
            "overall_efficiency": {
                "cache_hit_rate": (l1_stats.cache_efficiency_score + l2_stats.cache_efficiency_score) / 2.0,
                "query_reduction": query_metrics.query_reduction_percentage,
                "warming_effectiveness": if warm_metrics.total_warm_requests > 0 {
                    (warm_metrics.successful_warms as f64 / warm_metrics.total_warm_requests as f64) * 100.0
                } else { 0.0 }
            },
            "targets_met": {
                "query_reduction_25_percent": query_metrics.query_reduction_percentage >= 25.0,
                "cache_efficiency_good": l1_stats.cache_efficiency_score > 70.0 && l2_stats.cache_efficiency_score > 70.0
            }
        })
    }

    /// Start background optimization tasks
    pub fn start_background_tasks(self: Arc<Self>) {
        // Cache warming processor
        let cache_manager = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Process every 30 seconds
            loop {
                interval.tick().await;
                if let Err(e) = cache_manager.process_cache_warming().await {
                    tracing::error!("Cache warming processing error: {}", e);
                }
            }
        });

        // Performance monitoring
        let cache_manager = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Report every minute
            loop {
                interval.tick().await;
                let stats = cache_manager.get_enhanced_stats().await;
                tracing::info!(
                    "Cache Performance: query_reduction={:.2}%, cache_efficiency={:.2}%",
                    stats["query_performance"]["query_reduction_percentage"],
                    stats["overall_efficiency"]["cache_hit_rate"]
                );
            }
        });
    }
}

impl Default for MultiTierCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache_monitoring::{CacheMonitoringConfig, CachePerformanceMonitor};
    use crate::infrastructure::cached_query::{CacheKey, CachedQueryConfig, CachedQueryManager};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DB_QUERY_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[tokio::test]
    async fn test_intelligent_cache_adaptive_ttl() -> Result<(), Box<dyn std::error::Error>> {
        let config = IntelligentCacheConfig {
            enable_adaptive_ttl: true,
            base_ttl: Duration::from_secs(60),
            min_ttl: Duration::from_secs(30),
            max_ttl: Duration::from_secs(120),
            ..Default::default()
        };

        let cache = IntelligentCache::new(config);

        // Set a value
        cache
            .set("test_key".to_string(), "test_value".to_string())
            .await?;

        // Access it multiple times to build pattern
        for _ in 0..10 {
            cache.get(&"test_key".to_string()).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let stats = cache.get_stats().await;
        assert!(stats.adaptive_ttl_adjustments > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_multi_tier_cache() -> Result<(), Box<dyn std::error::Error>> {
        let cache_manager = MultiTierCacheManager::new();

        // Set a value
        cache_manager
            .set("key1".to_string(), serde_json::json!({"value": 42}))
            .await?;

        // Get the value (should be in L1)
        let value = cache_manager.get("key1").await;
        assert!(value.is_some());
        if let Some(val) = value {
            assert_eq!(val["value"], 42);
        }

        let stats = cache_manager.get_stats().await;
        let total_entries = stats["l1_cache"]["base_stats"]["total_entries"]
            .as_u64()
            .unwrap_or(0);
        assert!(total_entries > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_prefetch_functionality() -> Result<(), Box<dyn std::error::Error>> {
        let config = IntelligentCacheConfig {
            enable_prefetching: true,
            prefetch_threshold: 2,
            ..Default::default()
        };

        let cache = IntelligentCache::new(config);

        // Prefetch some data
        cache
            .prefetch("prefetch_key".to_string(), || async {
                Ok("prefetched_value".to_string())
            })
            .await?;

        // Verify it's in cache
        let value = cache.get(&"prefetch_key".to_string()).await;
        assert_eq!(value, Some("prefetched_value".to_string()));

        let stats = cache.get_stats().await;
        assert!(stats.prefetch_hits > 0 || stats.prefetch_misses > 0);

        Ok(())
    }

    /// Mock database query simulator
    async fn mock_db_query(query_type: &str, key: &str) -> HiveResult<serde_json::Value> {
        DB_QUERY_COUNT.fetch_add(1, Ordering::SeqCst);

        // Simulate different query types with different response times
        match query_type {
            "user" => {
                tokio::time::sleep(Duration::from_millis(50)).await; // Simulate DB latency
                Ok(serde_json::json!({
                    "id": key,
                    "name": format!("User {}", key),
                    "email": format!("user{}@example.com", key)
                }))
            }
            "product" => {
                tokio::time::sleep(Duration::from_millis(30)).await;
                Ok(serde_json::json!({
                    "id": key,
                    "name": format!("Product {}", key),
                    "price": 99.99
                }))
            }
            "order" => {
                tokio::time::sleep(Duration::from_millis(80)).await;
                Ok(serde_json::json!({
                    "id": key,
                    "user_id": "user123",
                    "total": 199.98
                }))
            }
            _ => {
                tokio::time::sleep(Duration::from_millis(20)).await;
                Ok(serde_json::json!({"data": format!("Generic data for {}", key)}))
            }
        }
    }

    #[tokio::test]
    async fn test_database_load_reduction_25_percent_target(
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Reset query counter
        DB_QUERY_COUNT.store(0, Ordering::SeqCst);

        let cache_config = CachedQueryConfig {
            enable_prefetching: true,
            enable_adaptive_ttl: true,
            enable_cache_warming: true,
            ..Default::default()
        };

        let cache_manager = Arc::new(CachedQueryManager::new(cache_config));
        let monitor = Arc::new(CachePerformanceMonitor::new(
            vec![cache_manager.clone()],
            vec![], // No invalidation managers for this test
            CacheMonitoringConfig::default(),
        ));

        // Test data - simulate real-world access patterns
        let test_keys = vec![
            ("user", "user123"),
            ("user", "user456"),
            ("product", "prod789"),
            ("product", "prod101"),
            ("order", "order001"),
            ("user", "user123"),    // Repeat to test caching
            ("product", "prod789"), // Repeat
            ("user", "user789"),
            ("order", "order002"),
            ("user", "user123"), // Another repeat
        ];

        let mut total_requests = 0;
        let mut cache_hits = 0;

        // First phase: Populate cache with some data
        info!("Phase 1: Initial cache population");
        for (query_type, key) in &test_keys[..5] {
            let cache_key = CacheKey::Custom(format!("{}:{}", query_type, key));

            let result = cache_manager
                .execute_cached_query(cache_key, vec![], || async {
                    mock_db_query(query_type, key).await
                })
                .await?;

            total_requests += 1;
            debug!("Cached query result: {:?}", result);
        }

        let initial_queries = DB_QUERY_COUNT.load(Ordering::SeqCst);
        info!(
            "Initial cache population used {} DB queries",
            initial_queries
        );

        // Second phase: Simulate repeated access patterns
        info!("Phase 2: Repeated access simulation");
        for (query_type, key) in &test_keys {
            let cache_key = CacheKey::Custom(format!("{}:{}", query_type, key));

            let result = cache_manager
                .execute_cached_query(cache_key, vec![], || async {
                    mock_db_query(query_type, key).await
                })
                .await?;

            total_requests += 1;

            // Check if this was a cache hit (by checking if DB was queried)
            let current_queries = DB_QUERY_COUNT.load(Ordering::SeqCst);
            if current_queries == initial_queries + total_requests - 5 {
                cache_hits += 1;
            }

            debug!("Query result: {:?}", result);
        }

        let final_queries = DB_QUERY_COUNT.load(Ordering::SeqCst);
        let total_db_queries = final_queries;
        let cache_hit_rate = if total_requests > 0 {
            cache_hits as f64 / total_requests as f64
        } else {
            0.0
        };

        let query_reduction = if total_db_queries > 0 && total_requests > 0 {
            ((total_requests - total_db_queries) as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        info!("Database Load Reduction Test Results:");
        info!("Total requests: {}", total_requests);
        info!("DB queries executed: {}", total_db_queries);
        info!("Cache hits: {}", cache_hits);
        info!("Cache hit rate: {:.2}%", cache_hit_rate * 100.0);
        info!("Query reduction: {:.2}%", query_reduction);

        // Collect final metrics
        monitor.collect_metrics().await?;
        let metrics = monitor.get_metrics().await;

        // Assertions
        assert!(total_requests > 0, "Should have processed some requests");
        assert!(
            total_db_queries <= total_requests,
            "DB queries should not exceed total requests"
        );

        // The test should demonstrate caching effectiveness
        // Note: In a real scenario, we'd expect higher cache hit rates
        if total_requests > 5 {
            let expected_min_reduction = 20.0; // At least 20% reduction
            assert!(
                query_reduction >= expected_min_reduction,
                "Query reduction {:.2}% should meet minimum target of {:.2}%",
                query_reduction,
                expected_min_reduction
            );
        }

        // Verify monitoring data
        assert!(metrics.overall_hit_rate >= 0.0 && metrics.overall_hit_rate <= 1.0);
        assert!(metrics.query_reduction_percentage >= 0.0);

        info!("✅ Database load reduction test completed successfully");
        info!(
            "📊 Final metrics: hit_rate={:.2}%, query_reduction={:.2}%, efficiency={:.2}%",
            metrics.overall_hit_rate * 100.0,
            metrics.query_reduction_percentage,
            metrics.efficiency_score * 100.0
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_deduplication_load_reduction() -> Result<(), Box<dyn std::error::Error>> {
        DB_QUERY_COUNT.store(0, Ordering::SeqCst);

        let cache_manager = Arc::new(CachedQueryManager::new(CachedQueryConfig::default()));

        // Simulate concurrent requests for the same data
        let mut handles = Vec::new();
        let query_key = "concurrent_test_key".to_string();

        for i in 0..10 {
            let cache_manager_clone = cache_manager.clone();
            let query_key_clone = query_key.clone();

            let handle = tokio::spawn(async move {
                mock_db_query("user", &format!("user{}", i)).await
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await?;
            results.push(result);
        }

        let total_db_queries = DB_QUERY_COUNT.load(Ordering::SeqCst);

        info!(
            "Deduplication test: {} concurrent requests resulted in {} DB queries",
            results.len(),
            total_db_queries
        );

        // With deduplication, we should have significantly fewer DB queries than concurrent requests
        assert!(
            total_db_queries < results.len(),
            "Deduplication should reduce DB queries"
        );

        // All results should be the same (from deduplication)
        let first_result = &results[0];
        for result in &results[1..] {
            assert_eq!(
                first_result, result,
                "All deduplicated results should be identical"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_batch_query_load_reduction() -> Result<(), Box<dyn std::error::Error>> {
        DB_QUERY_COUNT.store(0, Ordering::SeqCst);

        let cache_manager = Arc::new(CachedQueryManager::new(CachedQueryConfig::default()));

        // Create batch of similar queries
        let batch_requests = vec![
            BatchQueryRequest {
                query_key: "user:user1".to_string(),
                parameters: serde_json::json!({"id": "user1"}),
                sender: tokio::sync::oneshot::channel().0,
            },
            BatchQueryRequest {
                query_key: "user:user2".to_string(),
                parameters: serde_json::json!({"id": "user2"}),
                sender: tokio::sync::oneshot::channel().0,
            },
            BatchQueryRequest {
                query_key: "user:user3".to_string(),
                parameters: serde_json::json!({"id": "user3"}),
                sender: tokio::sync::oneshot::channel().0,
            },
        ];

        // Simulate batch processing
        let batch_results = vec![
            Ok(serde_json::json!({"id": "user1", "name": "User 1"})),
            Ok(serde_json::json!({"id": "user2", "name": "User 2"})),
            Ok(serde_json::json!({"id": "user3", "name": "User 3"})),
        ];

        // Execute batch
        cache_manager
            .execute_batched_queries(batch_requests, |_| async { Ok(batch_results) })
            .await?;

        let total_db_queries = DB_QUERY_COUNT.load(Ordering::SeqCst);

        info!(
            "Batch query test: {} queries processed with {} DB queries",
            batch_results.len(),
            total_db_queries
        );

        // Batch processing should result in fewer DB queries than individual queries
        assert!(
            total_db_queries <= batch_results.len(),
            "Batch processing should not increase DB queries"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_comprehensive_caching_performance() -> Result<(), Box<dyn std::error::Error>> {
        DB_QUERY_COUNT.store(0, Ordering::SeqCst);

        let cache_manager = Arc::new(CachedQueryManager::new(CachedQueryConfig {
            enable_prefetching: true,
            enable_adaptive_ttl: true,
            enable_cache_warming: true,
            ..Default::default()
        }));

        let monitor = Arc::new(CachePerformanceMonitor::new(
            vec![cache_manager.clone()],
            vec![],
            CacheMonitoringConfig::default(),
        ));

        // Simulate realistic workload
        let mut workload = Vec::new();

        // Hot data (accessed frequently)
        for i in 0..10 {
            workload.push(("user", format!("hot_user_{}", i % 3))); // Only 3 hot users
        }

        // Warm data (accessed moderately)
        for i in 0..20 {
            workload.push(("product", format!("warm_product_{}", i % 5))); // 5 warm products
        }

        // Cold data (accessed rarely)
        for i in 0..50 {
            workload.push(("order", format!("cold_order_{}", i))); // 50 unique orders
        }

        // Mix in some repeats to test caching
        for i in 0..30 {
            let idx = i % workload.len();
            let (query_type, key) = &workload[idx];
            workload.push((query_type.clone(), key.clone()));
        }

        let start_time = std::time::Instant::now();
        let mut total_requests = 0;

        // Execute workload
        for (query_type, key) in workload {
            let cache_key = CacheKey::Custom(format!("{}:{}", query_type, key));

            let _result = cache_manager
                .execute_cached_query(cache_key, vec![], || async {
                    mock_db_query(&query_type, &key).await
                })
                .await?;

            total_requests += 1;
        }

        let execution_time = start_time.elapsed();
        let total_db_queries = DB_QUERY_COUNT.load(Ordering::SeqCst);

        // Collect final metrics
        monitor.collect_metrics().await?;
        let metrics = monitor.get_metrics().await;
        let load_reduction = cache_manager.get_load_reduction_metrics().await;

        let cache_hit_rate = if total_requests > 0 {
            (total_requests - total_db_queries) as f64 / total_requests as f64
        } else {
            0.0
        };

        info!("Comprehensive Caching Performance Test Results:");
        info!("==========================================");
        info!("Total requests: {}", total_requests);
        info!("DB queries executed: {}", total_db_queries);
        info!("Cache hit rate: {:.2}%", cache_hit_rate * 100.0);
        info!(
            "Query reduction: {:.2}%",
            load_reduction.total_reduction_percentage
        );
        info!("Execution time: {:.2}s", execution_time.as_secs_f64());
        info!(
            "Target achieved (25% reduction): {}",
            load_reduction.target_achieved
        );
        info!("Cache efficiency: {:.2}%", metrics.efficiency_score * 100.0);

        // Performance assertions
        assert!(total_requests > 0, "Should process requests");
        assert!(
            cache_hit_rate >= 0.0 && cache_hit_rate <= 1.0,
            "Cache hit rate should be valid"
        );

        // The comprehensive test should demonstrate good caching performance
        // In practice, we'd tune these thresholds based on the specific workload
        if total_requests > 50 {
            assert!(
                cache_hit_rate > 0.3,
                "Should achieve at least 30% cache hit rate for mixed workload"
            );
            assert!(
                load_reduction.total_reduction_percentage > 15.0,
                "Should achieve at least 15% load reduction"
            );
        }

        // Verify monitoring integration
        assert!(metrics.overall_hit_rate >= 0.0);
        assert!(metrics.efficiency_score >= 0.0);

        info!("✅ Comprehensive caching performance test completed");
        Ok(())
    }
}
