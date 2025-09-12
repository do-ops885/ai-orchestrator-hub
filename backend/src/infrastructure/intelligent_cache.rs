//! Intelligent Caching System
//!
//! This module provides an advanced caching system with predictive prefetching,
//! adaptive TTL, and intelligent eviction strategies to optimize performance.

use crate::infrastructure::cache::{Cache, CacheStats};
use crate::utils::error::HiveResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, info, warn};

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
}

impl AccessPattern {
    fn new() -> Self {
        Self {
            access_count: 0,
            last_access: Instant::now(),
            access_frequency: 0.0,
            access_history: VecDeque::with_capacity(10),
            predicted_next_access: None,
        }
    }

    fn record_access(&mut self) {
        let now = Instant::now();
        self.access_count += 1;
        self.last_access = now;

        // Maintain access history for pattern analysis
        self.access_history.push_back(now);
        if self.access_history.len() > 10 {
            self.access_history.pop_front();
        }

        // Calculate access frequency
        if self.access_history.len() > 1 {
            let time_span = now.duration_since(self.access_history[0]).as_secs_f64();
            self.access_frequency = (self.access_history.len() - 1) as f64 / time_span;

            // Predict next access based on frequency
            if self.access_frequency > 0.0 {
                let avg_interval = 1.0 / self.access_frequency;
                self.predicted_next_access = Some(now + Duration::from_secs_f64(avg_interval));
            }
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
    prefetch_queue: Arc<RwLock<VecDeque<K>>>,
    stats: Arc<RwLock<IntelligentCacheStats>>,
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
    K: Clone + Eq + Hash + Send + Sync + 'static,
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
        }
    }

    /// Get value from cache with intelligent tracking
    pub async fn get(&self, key: &K) -> Option<V> {
        // Record access pattern
        self.record_access(key).await;

        // Get from cache
        let result = self.cache.get(key).await;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            if result.is_some() {
                // Check if this was a prefetch hit
                let prefetch_queue = self.prefetch_queue.read().await;
                if prefetch_queue.contains(key) {
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
                    queue.push_back(key);
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
    async fn record_access(&self, key: &K) {
        let mut patterns = self.access_patterns.write().await;
        let pattern = patterns
            .entry(key.clone())
            .or_insert_with(AccessPattern::new);
        pattern.record_access();

        // Check if we should trigger prefetching
        if self.config.enable_prefetching && pattern.should_prefetch(self.config.prefetch_threshold)
        {
            debug!("Key qualifies for prefetching based on access pattern");
        }
    }

    /// Calculate adaptive TTL for a key
    async fn calculate_adaptive_ttl(&self, key: &K) -> Duration {
        let patterns = self.access_patterns.read().await;
        if let Some(pattern) = patterns.get(key) {
            pattern.calculate_adaptive_ttl(
                self.config.base_ttl,
                self.config.min_ttl,
                self.config.max_ttl,
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
}

/// Multi-tier intelligent cache manager
pub struct MultiTierCacheManager {
    l1_cache: IntelligentCache<String, serde_json::Value>, // Fast, small cache
    l2_cache: IntelligentCache<String, serde_json::Value>, // Larger, persistent cache
    config: IntelligentCacheConfig,
}

impl MultiTierCacheManager {
    /// Create a new multi-tier cache manager
    pub fn new() -> Self {
        let l1_config = IntelligentCacheConfig {
            max_size: 1000,
            base_ttl: Duration::from_secs(60),
            ..Default::default()
        };

        let l2_config = IntelligentCacheConfig {
            max_size: 10000,
            base_ttl: Duration::from_secs(300),
            ..Default::default()
        };

        Self {
            l1_cache: IntelligentCache::new(l1_config.clone()),
            l2_cache: IntelligentCache::new(l2_config.clone()),
            config: l1_config,
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

impl Default for MultiTierCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
