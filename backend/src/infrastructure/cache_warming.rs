//! Cache Warming and Prefetching System
//!
//! Provides intelligent cache warming and prefetching capabilities to improve
//! cache hit rates and reduce database query latency.

use crate::infrastructure::cached_query::{CacheKey, CachedQueryManager};
use crate::utils::error::HiveResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Cache warming strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarmingStrategy {
    /// Warm based on historical access patterns
    HistoricalPattern,
    /// Warm based on predicted access patterns
    Predictive,
    /// Warm based on explicit configuration
    Configured,
    /// Warm based on time-based patterns (e.g., daily peaks)
    TimeBased,
    /// Warm based on user behavior analysis
    Behavioral,
}

/// Prefetching strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrefetchStrategy {
    /// Prefetch based on access frequency
    FrequencyBased,
    /// Prefetch based on sequential access patterns
    Sequential,
    /// Prefetch based on graph relationships
    GraphBased,
    /// Prefetch based on time-based predictions
    TimeBased,
    /// Prefetch based on user context
    Contextual,
}

/// Warming configuration
#[derive(Debug, Clone)]
pub struct WarmingConfig {
    /// Enable cache warming
    pub enable_warming: bool,
    /// Warming strategy
    pub warming_strategy: WarmingStrategy,
    /// Prefetch strategy
    pub prefetch_strategy: PrefetchStrategy,
    /// Maximum warming batch size
    pub max_warming_batch: usize,
    /// Warming interval
    pub warming_interval: Duration,
    /// Prefetch threshold (access count)
    pub prefetch_threshold: u32,
    /// Maximum prefetch queue size
    pub max_prefetch_queue: usize,
    /// Prefetch ahead time window
    pub prefetch_window: Duration,
}

impl Default for WarmingConfig {
    fn default() -> Self {
        Self {
            enable_warming: true,
            warming_strategy: WarmingStrategy::HistoricalPattern,
            prefetch_strategy: PrefetchStrategy::FrequencyBased,
            max_warming_batch: 100,
            warming_interval: Duration::from_secs(300), // 5 minutes
            prefetch_threshold: 3,
            max_prefetch_queue: 1000,
            prefetch_window: Duration::from_secs(60), // 1 minute ahead
        }
    }
}

/// Access pattern for cache warming decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPattern {
    pub key: CacheKey,
    pub access_count: u64,
    pub last_access: chrono::DateTime<chrono::Utc>,
    pub access_frequency: f64, // accesses per hour
    pub predicted_next_access: Option<chrono::DateTime<chrono::Utc>>,
    pub related_keys: Vec<CacheKey>,
}

/// Cache warming engine
pub struct CacheWarmingEngine {
    /// Cache manager
    cache_manager: Arc<CachedQueryManager>,
    /// Configuration
    config: WarmingConfig,
    /// Access patterns history
    access_patterns: Arc<RwLock<HashMap<CacheKey, AccessPattern>>>,
    /// Prefetch queue
    prefetch_queue: Arc<RwLock<VecDeque<PrefetchItem>>>,
    /// Warming candidates
    warming_candidates: Arc<RwLock<HashSet<CacheKey>>>,
    /// Warming statistics
    stats: Arc<RwLock<WarmingStats>>,
    /// Startup warming data
    startup_warming_data: Arc<RwLock<StartupWarmingData>>,
    /// Intelligent cache integration
    intelligent_cache: Option<
        Arc<crate::infrastructure::intelligent_cache::IntelligentCache<String, serde_json::Value>>,
    >,
}

/// Startup warming configuration and data
#[derive(Debug, Clone)]
pub struct StartupWarmingData {
    /// Keys to warm at startup
    startup_keys: Vec<CacheKey>,
    /// Priority order for startup warming
    priority_order: Vec<WarmingPriority>,
    /// Warming progress
    progress: WarmingProgress,
    /// Last startup warming time
    last_startup_warming: Option<chrono::DateTime<chrono::Utc>>,
}

/// Warming priority levels
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WarmingPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Warming progress tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WarmingProgress {
    pub total_keys: usize,
    pub warmed_keys: usize,
    pub failed_keys: usize,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefetchItem {
    pub key: CacheKey,
    pub priority: u8,
    pub predicted_access_time: chrono::DateTime<chrono::Utc>,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WarmingStats {
    /// Total warming operations
    pub total_warmings: u64,
    /// Successful warming operations
    pub successful_warmings: u64,
    /// Failed warming operations
    pub failed_warmings: u64,
    /// Total prefetch operations
    pub total_prefetches: u64,
    /// Successful prefetch operations
    pub successful_prefetches: u64,
    /// Failed prefetch operations
    pub failed_prefetches: u64,
    /// Cache hits from prefetched data
    pub prefetch_hits: u64,
    /// Warming efficiency (0.0 to 1.0)
    pub warming_efficiency: f64,
    /// Prefetch accuracy (0.0 to 1.0)
    pub prefetch_accuracy: f64,
}

impl CacheWarmingEngine {
    /// Create a new cache warming engine
    #[must_use] 
    pub fn new(cache_manager: Arc<CachedQueryManager>, config: WarmingConfig) -> Self {
        Self {
            cache_manager,
            config,
            access_patterns: Arc::new(RwLock::new(HashMap::new())),
            prefetch_queue: Arc::new(RwLock::new(VecDeque::new())),
            warming_candidates: Arc::new(RwLock::new(HashSet::new())),
            stats: Arc::new(RwLock::new(WarmingStats::default())),
            startup_warming_data: Arc::new(RwLock::new(StartupWarmingData {
                startup_keys: Vec::new(),
                priority_order: vec![
                    WarmingPriority::Critical,
                    WarmingPriority::High,
                    WarmingPriority::Medium,
                    WarmingPriority::Low,
                ],
                progress: WarmingProgress::default(),
                last_startup_warming: None,
            })),
            intelligent_cache: None,
        }
    }

    /// Record access pattern for a cache key
    pub async fn record_access(&self, key: &CacheKey, context: Option<HashMap<String, String>>) {
        let now = chrono::Utc::now();
        let mut patterns = self.access_patterns.write().await;

        let pattern = patterns
            .entry(key.clone())
            .or_insert_with(|| AccessPattern {
                key: key.clone(),
                access_count: 0,
                last_access: now,
                access_frequency: 0.0,
                predicted_next_access: None,
                related_keys: Vec::new(),
            });

        pattern.access_count += 1;
        pattern.last_access = now;

        // Update access frequency (simple moving average)
        let time_since_last = (now - pattern.last_access).num_hours() as f64;
        if time_since_last > 0.0 {
            let instant_frequency = 1.0 / time_since_last.max(0.0167); // Min 1 minute
            pattern.access_frequency = f64::midpoint(pattern.access_frequency, instant_frequency);
        }

        // Predict next access based on frequency
        if pattern.access_frequency > 0.0 {
            let avg_interval_hours = 1.0 / pattern.access_frequency;
            pattern.predicted_next_access =
                Some(now + chrono::Duration::hours(avg_interval_hours as i64));
        }

        // Add to warming candidates if access count meets threshold
        if pattern.access_count >= u64::from(self.config.prefetch_threshold) {
            let mut candidates = self.warming_candidates.write().await;
            candidates.insert(key.clone());
        }

        // Trigger prefetching if enabled
        if self.config.enable_warming {
            self.trigger_prefetching(key, context.unwrap_or_default())
                .await;
        }
    }

    /// Perform cache warming based on current strategy
    pub async fn perform_warming(&self) -> HiveResult<usize> {
        if !self.config.enable_warming {
            return Ok(0);
        }

        info!("Starting cache warming cycle");

        let candidates = {
            let candidates = self.warming_candidates.read().await;
            candidates
                .iter()
                .take(self.config.max_warming_batch).cloned()
                .collect::<Vec<_>>()
        };

        let mut warmed_count = 0;

        for key in candidates {
            match self.warm_key(&key).await {
                Ok(()) => {
                    warmed_count += 1;
                    {
                        let mut stats = self.stats.write().await;
                        stats.total_warmings += 1;
                        stats.successful_warmings += 1;
                    }
                }
                Err(e) => {
                    warn!("Failed to warm cache key {}: {}", key, e);
                    let mut stats = self.stats.write().await;
                    stats.total_warmings += 1;
                    stats.failed_warmings += 1;
                }
            }
        }

        // Update efficiency
        {
            let mut stats = self.stats.write().await;
            if stats.total_warmings > 0 {
                stats.warming_efficiency =
                    stats.successful_warmings as f64 / stats.total_warmings as f64;
            }
        }

        info!("Cache warming completed, warmed {} keys", warmed_count);
        Ok(warmed_count)
    }

    /// Process prefetch queue
    pub async fn process_prefetch_queue(&self) -> HiveResult<usize> {
        let now = chrono::Utc::now();
        let mut processed_count = 0;

        // Get items that should be prefetched now
        let items_to_process: Vec<PrefetchItem> = {
            let mut queue = self.prefetch_queue.write().await;
            let mut items = Vec::new();

            while let Some(item) = queue.front() {
                if item.predicted_access_time
                    <= now
                        + chrono::Duration::from_std(self.config.prefetch_window)
                            .unwrap_or(chrono::Duration::minutes(1))
                {
                    if let Some(removed_item) = queue.pop_front() {
                        items.push(removed_item);
                    }
                } else {
                    break; // Queue is sorted by time, so we can stop here
                }
            }

            items
        };

        for item in items_to_process {
            match self.prefetch_key(&item.key, item.context).await {
                Ok(()) => {
                    processed_count += 1;
                    let mut stats = self.stats.write().await;
                    stats.total_prefetches += 1;
                    stats.successful_prefetches += 1;
                }
                Err(e) => {
                    warn!("Failed to prefetch cache key {}: {}", item.key, e);
                    let mut stats = self.stats.write().await;
                    stats.total_prefetches += 1;
                    stats.failed_prefetches += 1;
                }
            }
        }

        if processed_count > 0 {
            debug!("Processed {} prefetch items", processed_count);
        }

        Ok(processed_count)
    }

    /// Get warming statistics
    pub async fn get_stats(&self) -> WarmingStats {
        let mut stats = self.stats.read().await.clone();

        // Calculate prefetch accuracy
        if stats.total_prefetches > 0 {
            stats.prefetch_accuracy =
                stats.successful_prefetches as f64 / stats.total_prefetches as f64;
        }

        stats
    }

    /// Get access patterns for analysis
    pub async fn get_access_patterns(&self) -> HashMap<CacheKey, AccessPattern> {
        self.access_patterns.read().await.clone()
    }

    /// Add explicit warming candidate
    pub async fn add_warming_candidate(&self, key: CacheKey) {
        let mut candidates = self.warming_candidates.write().await;
        candidates.insert(key);
    }

    /// Set intelligent cache for integration
    pub fn set_intelligent_cache(
        &mut self,
        cache: Arc<
            crate::infrastructure::intelligent_cache::IntelligentCache<String, serde_json::Value>,
        >,
    ) {
        self.intelligent_cache = Some(cache);
    }

    /// Configure startup warming keys
    pub async fn configure_startup_warming(
        &self,
        keys: Vec<CacheKey>,
        priorities: Vec<WarmingPriority>,
    ) {
        let mut startup_data = self.startup_warming_data.write().await;
        startup_data.startup_keys = keys;
        startup_data.priority_order = priorities;
    }

    /// Perform startup cache warming
    pub async fn perform_startup_warming<F, Fut>(&self, fetcher: F) -> HiveResult<WarmingProgress>
    where
        F: Fn(CacheKey) -> Fut + Send + Sync + Clone,
        Fut: std::future::Future<Output = HiveResult<serde_json::Value>> + Send,
    {
        let startup_data = self.startup_warming_data.read().await;
        if startup_data.startup_keys.is_empty() {
            return Ok(WarmingProgress::default());
        }

        info!(
            "Starting startup cache warming for {} keys",
            startup_data.startup_keys.len()
        );

        let mut progress = WarmingProgress {
            total_keys: startup_data.startup_keys.len(),
            warmed_keys: 0,
            failed_keys: 0,
            start_time: Some(chrono::Utc::now()),
            end_time: None,
        };

        // Group keys by priority
        let mut keys_by_priority: HashMap<WarmingPriority, Vec<CacheKey>> = HashMap::new();
        for key in &startup_data.startup_keys {
            // Determine priority (simplified - could be based on access patterns)
            let priority = self.determine_key_priority(key).await;
            keys_by_priority
                .entry(priority)
                .or_default()
                .push(key.clone());
        }

        // Warm keys in priority order
        for priority in &startup_data.priority_order {
            if let Some(keys) = keys_by_priority.get(priority) {
                info!("Warming {} keys with {:?} priority", keys.len(), priority);

                for key in keys {
                    match self.warm_key_with_fetcher(key, fetcher.clone()).await {
                        Ok(()) => {
                            progress.warmed_keys += 1;
                            let mut stats = self.stats.write().await;
                            stats.total_warmings += 1;
                            stats.successful_warmings += 1;
                        }
                        Err(e) => {
                            progress.failed_keys += 1;
                            warn!("Failed to warm startup key {}: {}", key, e);
                            let mut stats = self.stats.write().await;
                            stats.total_warmings += 1;
                            stats.failed_warmings += 1;
                        }
                    }
                }
            }
        }

        progress.end_time = Some(chrono::Utc::now());

        // Update startup data
        {
            let mut startup_data = self.startup_warming_data.write().await;
            startup_data.progress = progress.clone();
            startup_data.last_startup_warming = progress.end_time;
        }

        // Update efficiency
        {
            let mut stats = self.stats.write().await;
            if stats.total_warmings > 0 {
                stats.warming_efficiency =
                    stats.successful_warmings as f64 / stats.total_warmings as f64;
            }
        }

        info!(
            "Startup cache warming completed: {} successful, {} failed",
            progress.warmed_keys, progress.failed_keys
        );

        Ok(progress)
    }

    /// Determine priority for a cache key
    async fn determine_key_priority(&self, key: &CacheKey) -> WarmingPriority {
        let patterns = self.access_patterns.read().await;

        if let Some(pattern) = patterns.get(key) {
            if pattern.access_frequency > 10.0 {
                WarmingPriority::Critical
            } else if pattern.access_frequency > 5.0 {
                WarmingPriority::High
            } else if pattern.access_frequency > 1.0 {
                WarmingPriority::Medium
            } else {
                WarmingPriority::Low
            }
        } else {
            WarmingPriority::Medium // Default priority
        }
    }

    /// Warm key with custom fetcher
    async fn warm_key_with_fetcher<F, Fut>(&self, key: &CacheKey, fetcher: F) -> HiveResult<()>
    where
        F: Fn(CacheKey) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = HiveResult<serde_json::Value>> + Send,
    {
        // Check if already in cache
        if self
            .cache_manager
            .get_cached::<serde_json::Value>(key)
            .await
            .is_some()
        {
            return Ok(());
        }

        // Fetch the data
        let data = fetcher(key.clone()).await?;

        // Cache the data
        let dependencies = vec![]; // Could be determined based on key relationships
        let cache_entry = crate::infrastructure::cached_query::CacheEntry::new(data, dependencies);
        self.cache_manager
            .set_cached(key.clone(), cache_entry)
            .await?;

        debug!("Successfully warmed cache key: {}", key);
        Ok(())
    }

    /// Integrate with intelligent cache prefetching
    pub async fn integrate_intelligent_prefetching(&self) -> HiveResult<()> {
        if let Some(intelligent_cache) = &self.intelligent_cache {
            // Get prefetch recommendations from intelligent cache
            let recommendations = intelligent_cache.get_prefetch_recommendations().await;

            // Convert to prefetch items
            let mut prefetch_items = Vec::new();
            for recommendation in &recommendations {
                let priority = match recommendation.priority {
                    crate::infrastructure::intelligent_cache::PrefetchPriority::Critical => 5,
                    crate::infrastructure::intelligent_cache::PrefetchPriority::High => 4,
                    crate::infrastructure::intelligent_cache::PrefetchPriority::Medium => 3,
                    crate::infrastructure::intelligent_cache::PrefetchPriority::Low => 2,
                };

                let prefetch_item = PrefetchItem {
                    key: CacheKey::Custom(recommendation.key.clone()),
                    priority,
                    predicted_access_time: chrono::Utc::now() + chrono::Duration::seconds(60),
                    context: HashMap::from([
                        ("reason".to_string(), recommendation.reasons.join(", ")),
                        (
                            "benefit".to_string(),
                            recommendation.predicted_benefit.to_string(),
                        ),
                    ]),
                };

                prefetch_items.push(prefetch_item);
            }

            // Add to prefetch queue
            {
                let mut queue = self.prefetch_queue.write().await;
                for item in prefetch_items {
                    queue.push_back(item);
                }

                // Keep queue size manageable
                while queue.len() > self.config.max_prefetch_queue {
                    queue.pop_front();
                }
            }

            info!(
                "Integrated {} prefetch recommendations from intelligent cache",
                recommendations.len()
            );
        }

        Ok(())
    }

    /// Get startup warming progress
    pub async fn get_startup_warming_progress(&self) -> WarmingProgress {
        let startup_data = self.startup_warming_data.read().await;
        startup_data.progress.clone()
    }

    /// Generate enhanced warming report with startup information
    pub async fn generate_enhanced_report(&self) -> serde_json::Value {
        let base_report = self.generate_report().await;
        let startup_data = self.startup_warming_data.read().await;

        

        if let serde_json::Value::Object(mut obj) = base_report {
            obj.insert("startup_warming".to_string(), serde_json::json!({
                "last_startup_warming": startup_data.last_startup_warming.map(|dt| dt.to_rfc3339()),
                "startup_keys_count": startup_data.startup_keys.len(),
                "progress": startup_data.progress,
                "intelligent_cache_integrated": self.intelligent_cache.is_some()
            }));
            serde_json::Value::Object(obj)
        } else {
            base_report
        }
    }

    /// Remove warming candidate
    pub async fn remove_warming_candidate(&self, key: &CacheKey) {
        let mut candidates = self.warming_candidates.write().await;
        candidates.remove(key);
    }

    /// Generate warming report
    pub async fn generate_report(&self) -> serde_json::Value {
        let stats = self.get_stats().await;
        let patterns = self.get_access_patterns().await;
        let candidates_count = {
            let candidates = self.warming_candidates.read().await;
            candidates.len()
        };
        let queue_size = {
            let queue = self.prefetch_queue.read().await;
            queue.len()
        };

        // Find top accessed keys
        let mut top_patterns: Vec<_> = patterns.values().collect();
        top_patterns.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        let top_keys = top_patterns
            .into_iter()
            .take(10)
            .map(|p| {
                serde_json::json!({
                    "key": p.key.to_string(),
                    "access_count": p.access_count,
                    "access_frequency": p.access_frequency,
                    "last_access": p.last_access.to_rfc3339()
                })
            })
            .collect::<Vec<_>>();

        serde_json::json!({
            "warming_stats": {
                "total_warmings": stats.total_warmings,
                "successful_warmings": stats.successful_warmings,
                "warming_efficiency": stats.warming_efficiency,
                "total_prefetches": stats.total_prefetches,
                "successful_prefetches": stats.successful_prefetches,
                "prefetch_accuracy": stats.prefetch_accuracy,
                "prefetch_hits": stats.prefetch_hits
            },
            "current_state": {
                "warming_candidates": candidates_count,
                "prefetch_queue_size": queue_size,
                "tracked_patterns": patterns.len()
            },
            "top_accessed_keys": top_keys,
            "configuration": {
                "enable_warming": self.config.enable_warming,
                "warming_strategy": format!("{:?}", self.config.warming_strategy),
                "prefetch_strategy": format!("{:?}", self.config.prefetch_strategy),
                "max_warming_batch": self.config.max_warming_batch,
                "prefetch_threshold": self.config.prefetch_threshold
            },
            "generated_at": chrono::Utc::now().to_rfc3339()
        })
    }

    // Private methods

    async fn warm_key(&self, key: &CacheKey) -> HiveResult<()> {
        // Check if already in cache
        if self
            .cache_manager
            .get_cached::<serde_json::Value>(key)
            .await
            .is_some()
        {
            return Ok(());
        }

        // For now, we can't actually fetch data without knowing the fetcher
        // In a real implementation, this would use registered data fetchers
        debug!("Would warm cache key: {}", key);

        // Remove from candidates after warming
        let mut candidates = self.warming_candidates.write().await;
        candidates.remove(key);

        Ok(())
    }

    async fn prefetch_key(
        &self,
        key: &CacheKey,
        context: HashMap<String, String>,
    ) -> HiveResult<()> {
        // Similar to warm_key, we can't actually prefetch without fetchers
        debug!(
            "Would prefetch cache key: {} with context {:?}",
            key, context
        );

        Ok(())
    }

    async fn trigger_prefetching(&self, key: &CacheKey, context: HashMap<String, String>) {
        let patterns = self.access_patterns.read().await;
        if let Some(pattern) = patterns.get(key) {
            if pattern.access_count >= u64::from(self.config.prefetch_threshold) {
                if let Some(predicted_time) = pattern.predicted_next_access {
                    let prefetch_item = PrefetchItem {
                        key: key.clone(),
                        priority: 1, // Could be calculated based on access frequency
                        predicted_access_time: predicted_time,
                        context,
                    };

                    let mut queue = self.prefetch_queue.write().await;
                    queue.push_back(prefetch_item);

                    // Keep queue size manageable
                    while queue.len() > self.config.max_prefetch_queue {
                        queue.pop_front();
                    }
                }
            }
        }
    }
}

/// Background warming service
pub struct WarmingService {
    engine: Arc<CacheWarmingEngine>,
    config: WarmingConfig,
}

impl WarmingService {
    #[must_use] 
    pub fn new(engine: Arc<CacheWarmingEngine>, config: WarmingConfig) -> Self {
        Self { engine, config }
    }

    /// Start background warming and prefetching
    #[must_use] 
    pub fn start_service(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut warming_interval = tokio::time::interval(self.config.warming_interval);
            let mut prefetch_interval = tokio::time::interval(Duration::from_secs(30)); // Check prefetch queue every 30 seconds

            loop {
                tokio::select! {
                    _ = warming_interval.tick() => {
                        if let Err(e) = self.engine.perform_warming().await {
                            warn!("Cache warming failed: {}", e);
                        }
                    }
                    _ = prefetch_interval.tick() => {
                        if let Err(e) = self.engine.process_prefetch_queue().await {
                            warn!("Prefetch processing failed: {}", e);
                        }
                    }
                }
            }
        })
    }
}

/// Cache warming strategy implementations
pub mod strategies {
    use super::{Arc, CacheWarmingEngine, HiveResult, CacheKey, Duration};

    /// Historical pattern warming strategy
    pub struct HistoricalPatternStrategy {
        engine: Arc<CacheWarmingEngine>,
    }

    impl HistoricalPatternStrategy {
        #[must_use] 
        pub fn new(engine: Arc<CacheWarmingEngine>) -> Self {
            Self { engine }
        }

        /// Analyze historical patterns and add warming candidates
        pub async fn analyze_patterns(&self) -> HiveResult<Vec<CacheKey>> {
            let patterns = self.engine.get_access_patterns().await;

            // Find keys with high access frequency
            let mut high_frequency_keys: Vec<_> = patterns
                .values()
                .filter(|p| p.access_frequency > 1.0) // More than 1 access per hour
                .map(|p| p.key.clone())
                .collect();

            // Sort by access frequency
            high_frequency_keys.sort_by(|a, b| {
                let freq_a = patterns.get(a).map_or(0.0, |p| p.access_frequency);
                let freq_b = patterns.get(b).map_or(0.0, |p| p.access_frequency);
                freq_b
                    .partial_cmp(&freq_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            Ok(high_frequency_keys.into_iter().take(50).collect())
        }
    }

    /// Predictive warming strategy
    pub struct PredictiveStrategy {
        engine: Arc<CacheWarmingEngine>,
    }

    impl PredictiveStrategy {
        #[must_use] 
        pub fn new(engine: Arc<CacheWarmingEngine>) -> Self {
            Self { engine }
        }

        /// Predict keys that will be accessed soon
        pub async fn predict_accesses(&self, time_window: Duration) -> HiveResult<Vec<CacheKey>> {
            let patterns = self.engine.get_access_patterns().await;
            let now = chrono::Utc::now();
            let window_end =
                now + chrono::Duration::from_std(time_window).unwrap_or(chrono::Duration::hours(1));

            let mut predicted_keys: Vec<_> = patterns
                .values()
                .filter(|p| {
                    if let Some(predicted_time) = p.predicted_next_access {
                        predicted_time >= now && predicted_time <= window_end
                    } else {
                        false
                    }
                })
                .map(|p| p.key.clone())
                .collect();

            // Sort by predicted access time (soonest first)
            predicted_keys.sort_by(|a, b| {
                let time_a = patterns
                    .get(a)
                    .and_then(|p| p.predicted_next_access)
                    .unwrap_or(now);
                let time_b = patterns
                    .get(b)
                    .and_then(|p| p.predicted_next_access)
                    .unwrap_or(now);
                time_a.cmp(&time_b)
            });

            Ok(predicted_keys)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_warming_engine() -> Result<(), Box<dyn std::error::Error>> {
        let cache_manager = Arc::new(CachedQueryManager::new(
            crate::infrastructure::cached_query::CachedQueryConfig::default(),
        ));
        let engine = Arc::new(CacheWarmingEngine::new(
            cache_manager,
            WarmingConfig::default(),
        ));

        // Record some access patterns
        let key1 = CacheKey::Custom("test_key_1".to_string());
        let key2 = CacheKey::Custom("test_key_2".to_string());

        engine.record_access(&key1, None).await;
        engine.record_access(&key2, None).await;

        // Check that patterns were recorded
        let patterns = engine.get_access_patterns().await;
        assert!(patterns.contains_key(&key1));
        assert!(patterns.contains_key(&key2));

        // Test warming (should not fail even without actual data)
        let warmed_count = engine.perform_warming().await?;
        assert_eq!(warmed_count, 0); // No candidates should be warmed without meeting threshold

        // Test prefetch processing
        let processed_count = engine.process_prefetch_queue().await?;
        assert_eq!(processed_count, 0); // No items in queue

        Ok(())
    }

    #[tokio::test]
    async fn test_warming_stats() -> Result<(), Box<dyn std::error::Error>> {
        let cache_manager = Arc::new(CachedQueryManager::new(
            crate::infrastructure::cached_query::CachedQueryConfig::default(),
        ));
        let engine = Arc::new(CacheWarmingEngine::new(
            cache_manager,
            WarmingConfig::default(),
        ));

        let stats = engine.get_stats().await;
        assert_eq!(stats.total_warmings, 0);
        assert_eq!(stats.successful_warmings, 0);

        Ok(())
    }
}
