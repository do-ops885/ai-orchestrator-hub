//! Cache Invalidation Manager
//!
//! Provides sophisticated cache invalidation strategies to maintain data consistency
//! while maximizing cache efficiency.

use crate::infrastructure::cached_query::{CacheKey, CachedQueryManager};
use crate::utils::error::{HiveError, HiveResult};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// Invalidation rule for cache entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationRule {
    /// Pattern to match cache keys (supports regex)
    pub key_pattern: String,
    /// TTL for entries matching this rule
    pub ttl: Duration,
    /// Dependencies that trigger invalidation
    pub dependencies: Vec<String>,
    /// Priority for invalidation order (0-255, higher = more important)
    pub priority: u8,
    /// Invalidation strategy for this rule
    pub strategy: InvalidationStrategy,
    /// Maximum age before forced invalidation
    pub max_age: Option<Duration>,
    /// Tags for grouping related rules
    pub tags: Vec<String>,
}

/// Enhanced invalidation rule with advanced features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedInvalidationRule {
    /// Base invalidation rule
    pub base_rule: InvalidationRule,
    /// Conditional invalidation based on access patterns
    pub access_pattern_conditions: Vec<AccessPatternCondition>,
    /// Size-based invalidation thresholds
    pub size_thresholds: Option<SizeThresholds>,
    /// Frequency-based invalidation
    pub frequency_thresholds: Option<FrequencyThresholds>,
}

/// Access pattern condition for conditional invalidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPatternCondition {
    /// Minimum access frequency to trigger invalidation
    pub min_frequency: f64,
    /// Maximum time since last access
    pub max_idle_time: Duration,
    /// Minimum number of accesses
    pub min_access_count: u32,
}

/// Size-based invalidation thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeThresholds {
    /// Maximum size in bytes before invalidation
    pub max_size_bytes: usize,
    /// Size growth rate threshold (percentage)
    pub growth_rate_threshold: f64,
}

/// Frequency-based invalidation thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyThresholds {
    /// Minimum access frequency (accesses per second)
    pub min_frequency: f64,
    /// Maximum access frequency (accesses per second)
    pub max_frequency: f64,
    /// Time window for frequency calculation
    pub frequency_window: Duration,
}

/// Cache dependency graph for tracking relationships
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Forward dependencies: key -> dependent keys
    forward_deps: HashMap<CacheKey, HashSet<CacheKey>>,
    /// Reverse dependencies: key -> keys that depend on it
    reverse_deps: HashMap<CacheKey, HashSet<CacheKey>>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            forward_deps: HashMap::new(),
            reverse_deps: HashMap::new(),
        }
    }

    /// Add a dependency relationship
    pub fn add_dependency(&mut self, key: CacheKey, depends_on: CacheKey) {
        // Forward: key depends on depends_on
        self.forward_deps
            .entry(key.clone())
            .or_default()
            .insert(depends_on.clone());

        // Reverse: depends_on is depended on by key
        self.reverse_deps
            .entry(depends_on)
            .or_default()
            .insert(key);
    }

    /// Remove a dependency relationship
    pub fn remove_dependency(&mut self, key: &CacheKey, depends_on: &CacheKey) {
        if let Some(deps) = self.forward_deps.get_mut(key) {
            deps.remove(depends_on);
        }
        if let Some(rev_deps) = self.reverse_deps.get_mut(depends_on) {
            rev_deps.remove(key);
        }
    }

    /// Get all keys that depend on the given key
    #[must_use] 
    pub fn get_dependents(&self, key: &CacheKey) -> HashSet<CacheKey> {
        self.reverse_deps.get(key).cloned().unwrap_or_default()
    }

    /// Get all keys that the given key depends on
    #[must_use] 
    pub fn get_dependencies(&self, key: &CacheKey) -> HashSet<CacheKey> {
        self.forward_deps.get(key).cloned().unwrap_or_default()
    }

    /// Clear all dependencies
    pub fn clear(&mut self) {
        self.forward_deps.clear();
        self.reverse_deps.clear();
    }
}

/// Invalidation strategy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidationStrategy {
    /// Immediate invalidation
    Immediate,
    /// Lazy invalidation (invalidate on next access)
    Lazy,
    /// Batched invalidation (collect and invalidate in batches)
    Batched { batch_size: usize },
    /// Time-based invalidation with grace period
    GracePeriod { grace_period: Duration },
    /// Sliding window invalidation (invalidate based on time windows)
    SlidingWindow {
        window_size: Duration,
        check_interval: Duration,
    },
    /// Probabilistic invalidation (invalidate with probability based on age)
    Probabilistic {
        base_probability: f64,
        age_factor: f64,
    },
    /// Hybrid strategy combining multiple approaches
    Hybrid {
        immediate_threshold: Duration,
        lazy_threshold: Duration,
        batch_size: usize,
    },
}

/// Cache invalidation manager
pub struct CacheInvalidationManager {
    /// Reference to the cache manager
    cache_manager: Arc<CachedQueryManager>,
    /// Dependency graph for tracking relationships
    dependency_graph: Arc<RwLock<DependencyGraph>>,
    /// Invalidation rules
    rules: Arc<RwLock<Vec<InvalidationRule>>>,
    /// Advanced invalidation rules
    advanced_rules: Arc<RwLock<Vec<AdvancedInvalidationRule>>>,
    /// Pending invalidations for batched strategy
    pending_invalidations: Arc<RwLock<HashSet<CacheKey>>>,
    /// Invalidation statistics
    pub stats: Arc<RwLock<InvalidationStats>>,
    /// Strategy configuration
    strategy: InvalidationStrategy,
    /// Sliding window invalidation state
    sliding_window_state: Arc<RwLock<SlidingWindowState>>,
    /// Probabilistic invalidation state
    probabilistic_state: Arc<RwLock<ProbabilisticState>>,
}

/// State for sliding window invalidation
#[derive(Debug, Clone)]
struct SlidingWindowState {
    /// Last invalidation times for keys
    last_invalidation_times: HashMap<CacheKey, Instant>,
    /// Invalidation counts per window
    invalidation_counts: HashMap<CacheKey, VecDeque<(Instant, u32)>>,
}

/// State for probabilistic invalidation
#[derive(Debug, Clone)]
struct ProbabilisticState {
    /// Random seed for probabilistic decisions
    seed: u64,
    /// Age distribution tracking
    age_distribution: HashMap<CacheKey, Duration>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InvalidationStats {
    /// Total invalidations performed
    pub total_invalidations: u64,
    /// Cascade invalidations (due to dependencies)
    pub cascade_invalidations: u64,
    /// Batched invalidations
    pub batched_invalidations: u64,
    /// Lazy invalidations
    pub lazy_invalidations: u64,
    /// Grace period hits
    pub grace_period_hits: u64,
    /// Average invalidation time
    pub avg_invalidation_time_ms: f64,
}

impl CacheInvalidationManager {
    /// Create a new invalidation manager
    #[must_use] 
    pub fn new(cache_manager: Arc<CachedQueryManager>, strategy: InvalidationStrategy) -> Self {
        Self {
            cache_manager,
            dependency_graph: Arc::new(RwLock::new(DependencyGraph::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            advanced_rules: Arc::new(RwLock::new(Vec::new())),
            pending_invalidations: Arc::new(RwLock::new(HashSet::new())),
            stats: Arc::new(RwLock::new(InvalidationStats::default())),
            strategy,
            sliding_window_state: Arc::new(RwLock::new(SlidingWindowState {
                last_invalidation_times: HashMap::new(),
                invalidation_counts: HashMap::new(),
            })),
            probabilistic_state: Arc::new(RwLock::new(ProbabilisticState {
                seed: rand::random(),
                age_distribution: HashMap::new(),
            })),
        }
    }

    /// Invalidate a single cache key
    pub async fn invalidate_key(&self, key: &CacheKey) -> HiveResult<()> {
        let start_time = Instant::now();

        debug!("Invalidating cache key: {}", key);

        // Invalidate the key itself
        self.cache_manager.invalidate_key(key).await;

        // Handle cascade invalidation based on dependencies
        self.handle_cascade_invalidation(key).await?;

        // Update statistics
        let elapsed = start_time.elapsed();
        {
            let mut stats = self.stats.write().await;
            stats.total_invalidations += 1;
            stats.avg_invalidation_time_ms =
                f64::midpoint(stats.avg_invalidation_time_ms, elapsed.as_millis() as f64);
        }

        Ok(())
    }

    /// Invalidate multiple keys with batching
    pub async fn invalidate_keys(&self, keys: &[CacheKey]) -> HiveResult<()> {
        if let InvalidationStrategy::Batched { batch_size } = &self.strategy {
            self.invalidate_keys_batched(keys, *batch_size).await
        } else {
            for key in keys {
                self.invalidate_key(key).await?;
            }
            Ok(())
        }
    }

    /// Invalidate keys based on a pattern
    pub async fn invalidate_by_pattern(&self, pattern: &str) -> HiveResult<usize> {
        // This is a simplified implementation
        // In a real system, you'd use regex or glob matching
        let mut invalidated_count = 0;

        // For now, just invalidate keys that contain the pattern
        let keys_to_check: Vec<CacheKey> = {
            // This would need to be implemented in the cache manager
            // For now, we'll use a placeholder
            vec![]
        };

        for key in keys_to_check {
            if key.to_string().contains(pattern) {
                self.invalidate_key(&key).await?;
                invalidated_count += 1;
            }
        }

        info!(
            "Invalidated {} keys matching pattern: {}",
            invalidated_count, pattern
        );
        Ok(invalidated_count)
    }

    /// Add a dependency relationship
    pub async fn add_dependency(&self, key: CacheKey, depends_on: CacheKey) {
        let mut graph = self.dependency_graph.write().await;
        graph.add_dependency(key, depends_on);
    }

    /// Remove a dependency relationship
    pub async fn remove_dependency(&self, key: &CacheKey, depends_on: &CacheKey) {
        let mut graph = self.dependency_graph.write().await;
        graph.remove_dependency(key, depends_on);
    }

    /// Add an invalidation rule
    pub async fn add_rule(&self, rule: InvalidationRule) {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        // Sort by priority (higher priority first)
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Add an advanced invalidation rule
    pub async fn add_advanced_rule(&self, rule: AdvancedInvalidationRule) {
        let mut rules = self.advanced_rules.write().await;
        rules.push(rule);
        // Sort by priority (higher priority first)
        rules.sort_by(|a, b| b.base_rule.priority.cmp(&a.base_rule.priority));
    }

    /// Invalidate based on sliding window strategy
    pub async fn invalidate_sliding_window(&self, window_size: Duration) -> HiveResult<usize> {
        let mut invalidated_count = 0;
        let now = Instant::now();

        let mut state = self.sliding_window_state.write().await;

        // Clean old entries from sliding windows
        for counts in state.invalidation_counts.values_mut() {
            while let Some((timestamp, _)) = counts.front() {
                if now.duration_since(*timestamp) > window_size {
                    counts.pop_front();
                } else {
                    break;
                }
            }
        }

        // Invalidate keys that exceed threshold in current window
        let keys_to_check: Vec<CacheKey> = state.invalidation_counts.keys().cloned().collect();

        for key in keys_to_check {
            if let Some(counts) = state.invalidation_counts.get(&key) {
                let total_invalidations: u32 = counts.iter().map(|(_, count)| count).sum();

                // Invalidate if too many invalidations in window
                if total_invalidations > 10 {
                    self.invalidate_key(&key).await?;
                    invalidated_count += 1;

                    // Reset the window for this key
                    state.invalidation_counts.remove(&key);
                    state.last_invalidation_times.insert(key, now);
                }
            }
        }

        Ok(invalidated_count)
    }

    /// Invalidate based on probabilistic strategy
    pub async fn invalidate_probabilistic(
        &self,
        base_probability: f64,
        age_factor: f64,
    ) -> HiveResult<usize> {
        let mut invalidated_count = 0;
        let now = Instant::now();

        let mut state = self.probabilistic_state.write().await;

        // Get all cache keys (this would need to be implemented in the cache manager)
        let all_keys: Vec<CacheKey> = vec![]; // Placeholder - would need cache manager method

        for key in all_keys {
            let age = if let Some(last_invalidation) = state.age_distribution.get(&key) {
                now.duration_since(Instant::now().checked_sub(*last_invalidation).unwrap())
            } else {
                Duration::from_secs(0)
            };

            // Calculate invalidation probability based on age
            let age_seconds = age.as_secs_f64();
            let probability = base_probability + (age_factor * age_seconds / 3600.0); // Increase with age
            let probability = probability.min(1.0); // Cap at 100%

            // Generate random number for probabilistic decision
            let random_value: f64 = rand::random();
            if random_value < probability {
                self.invalidate_key(&key).await?;
                invalidated_count += 1;
                state.age_distribution.insert(key, Duration::from_secs(0));
            }
        }

        Ok(invalidated_count)
    }

    /// Invalidate based on pattern matching with regex support
    pub async fn invalidate_by_regex_pattern(&self, pattern: &str) -> HiveResult<usize> {
        use regex::Regex;

        let regex = Regex::new(pattern).map_err(|e| HiveError::ValidationError {
            field: "pattern".to_string(),
            reason: format!("Invalid regex pattern: {e}"),
        })?;

        let mut invalidated_count = 0;

        // Get all cache keys (this would need to be implemented in the cache manager)
        let all_keys: Vec<CacheKey> = vec![]; // Placeholder - would need cache manager method

        for key in all_keys {
            if regex.is_match(&key.to_string()) {
                self.invalidate_key(&key).await?;
                invalidated_count += 1;
            }
        }

        info!(
            "Invalidated {} keys matching regex pattern: {}",
            invalidated_count, pattern
        );
        Ok(invalidated_count)
    }

    /// Invalidate based on access pattern conditions
    pub async fn invalidate_by_access_pattern(&self, key: &CacheKey) -> HiveResult<bool> {
        let advanced_rules = self.advanced_rules.read().await;

        for rule in &*advanced_rules {
            // Check if key matches the pattern
            if key.to_string().contains(&rule.base_rule.key_pattern) {
                // Check access pattern conditions
                for condition in &rule.access_pattern_conditions {
                    // This would need access pattern data from the cache
                    // For now, we'll use placeholder logic
                    let should_invalidate =
                        self.check_access_pattern_condition(key, condition).await;

                    if should_invalidate {
                        self.invalidate_key(key).await?;
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Check if an access pattern condition is met
    async fn check_access_pattern_condition(
        &self,
        _key: &CacheKey,
        _condition: &AccessPatternCondition,
    ) -> bool {
        // This would need to be implemented with actual access pattern tracking
        // For now, return false as placeholder
        false
    }

    /// Invalidate based on size thresholds
    pub async fn invalidate_by_size_threshold(&self, max_size_bytes: usize) -> HiveResult<usize> {
        let mut invalidated_count = 0;

        // Get cache size information (this would need to be implemented in the cache manager)
        let cache_sizes: HashMap<CacheKey, usize> = HashMap::new(); // Placeholder

        for (key, size) in cache_sizes {
            if size > max_size_bytes {
                self.invalidate_key(&key).await?;
                invalidated_count += 1;
            }
        }

        info!(
            "Invalidated {} keys exceeding size threshold: {} bytes",
            invalidated_count, max_size_bytes
        );
        Ok(invalidated_count)
    }

    /// Get invalidation recommendations based on current cache state
    pub async fn get_invalidation_recommendations(&self) -> Vec<InvalidationRecommendation> {
        let mut recommendations = Vec::new();

        // Analyze cache state and generate recommendations
        // This would include analyzing hit rates, memory usage, access patterns, etc.

        // Example recommendations
        recommendations.push(InvalidationRecommendation {
            key_pattern: "temp_*".to_string(),
            reason: "Temporary data with low access frequency".to_string(),
            priority: InvalidationPriority::Low,
            estimated_savings: 1024 * 1024, // 1MB
        });

        recommendations.push(InvalidationRecommendation {
            key_pattern: "stale_*".to_string(),
            reason: "Stale data exceeding TTL".to_string(),
            priority: InvalidationPriority::Medium,
            estimated_savings: 5 * 1024 * 1024, // 5MB
        });

        recommendations
    }

    /// Apply invalidation recommendations
    pub async fn apply_recommendations(
        &self,
        recommendations: &[InvalidationRecommendation],
    ) -> HiveResult<usize> {
        let mut total_invalidated = 0;

        for recommendation in recommendations {
            let count = self
                .invalidate_by_pattern(&recommendation.key_pattern)
                .await?;
            total_invalidated += count;

            info!(
                "Applied recommendation: {} - invalidated {} keys",
                recommendation.reason, count
            );
        }

        Ok(total_invalidated)
    }

    /// Get invalidation statistics
    pub async fn get_stats(&self) -> InvalidationStats {
        self.stats.read().await.clone()
    }

    /// Process pending invalidations (for batched strategy)
    pub async fn process_pending_invalidations(&self) -> HiveResult<usize> {
        let pending: Vec<CacheKey> = {
            let mut pending_invalidations = self.pending_invalidations.write().await;
            let keys = pending_invalidations.drain().collect();
            keys
        };

        let count = pending.len();
        if count > 0 {
            info!("Processing {} pending invalidations", count);
            for key in &pending {
                self.invalidate_key(key).await?;
            }

            {
                let mut stats = self.stats.write().await;
                stats.batched_invalidations += count as u64;
            }
        }

        Ok(count)
    }

    /// Handle cascade invalidation based on dependencies
    async fn handle_cascade_invalidation(&self, key: &CacheKey) -> HiveResult<()> {
        let graph = self.dependency_graph.read().await;
        let dependents = graph.get_dependents(key);
        let dependents_count = dependents.len();

        if dependents_count > 0 {
            debug!(
                "Invalidating {} dependent keys for: {}",
                dependents_count, key
            );

            for dependent in &dependents {
                self.cache_manager.invalidate_key(dependent).await;
            }

            {
                let mut stats = self.stats.write().await;
                stats.cascade_invalidations += dependents_count as u64;
            }
        }

        Ok(())
    }

    /// Invalidate keys in batches
    async fn invalidate_keys_batched(
        &self,
        keys: &[CacheKey],
        batch_size: usize,
    ) -> HiveResult<()> {
        for chunk in keys.chunks(batch_size) {
            let mut pending = self.pending_invalidations.write().await;
            for key in chunk {
                pending.insert(key.clone());
            }
        }

        // Process the batch
        self.process_pending_invalidations().await?;
        Ok(())
    }

    /// Check if a key should be invalidated based on rules
    pub async fn should_invalidate(&self, key: &CacheKey) -> bool {
        let rules = self.rules.read().await;

        for rule in &*rules {
            if key.to_string().contains(&rule.key_pattern) {
                // Check if any dependencies have been modified
                for dep_pattern in &rule.dependencies {
                    // This would need more sophisticated dependency tracking
                    // For now, we'll use a simple check
                    if key.to_string().contains(dep_pattern) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Clean up expired rules and dependencies
    pub async fn cleanup(&self) -> HiveResult<()> {
        // Remove rules that haven't been used recently
        // This is a simplified cleanup - in practice you'd track usage

        info!("Cache invalidation cleanup completed");
        Ok(())
    }

    /// Get dependency information for debugging
    pub async fn get_dependency_info(&self, key: &CacheKey) -> DependencyInfo {
        let graph = self.dependency_graph.read().await;

        DependencyInfo {
            key: key.clone(),
            depends_on: graph.get_dependencies(key).into_iter().collect(),
            depended_by: graph.get_dependents(key).into_iter().collect(),
        }
    }
}

/// Dependency information for a cache key
#[derive(Debug, Clone)]
pub struct DependencyInfo {
    pub key: CacheKey,
    pub depends_on: Vec<CacheKey>,
    pub depended_by: Vec<CacheKey>,
}

/// Invalidation recommendation
#[derive(Debug, Clone)]
pub struct InvalidationRecommendation {
    pub key_pattern: String,
    pub reason: String,
    pub priority: InvalidationPriority,
    pub estimated_savings: usize, // bytes
}

/// Priority levels for invalidation recommendations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Specialized invalidation manager for agent-related caches
pub struct AgentCacheInvalidationManager {
    pub base_manager: Arc<CacheInvalidationManager>,
}

impl AgentCacheInvalidationManager {
    #[must_use] 
    pub fn new(base_manager: Arc<CacheInvalidationManager>) -> Self {
        Self { base_manager }
    }

    /// Invalidate agent data when agent is updated
    pub async fn invalidate_agent(&self, agent_id: Uuid) -> HiveResult<()> {
        let keys = vec![CacheKey::Agent(agent_id), CacheKey::AgentMetrics(agent_id)];

        self.base_manager.invalidate_keys(&keys).await?;
        info!("Invalidated cache for agent: {}", agent_id);
        Ok(())
    }

    /// Invalidate all agent-related caches
    pub async fn invalidate_all_agents(&self) -> HiveResult<()> {
        // This would need to be implemented to get all agent keys
        // For now, use pattern invalidation
        self.base_manager.invalidate_by_pattern("agent").await?;
        info!("Invalidated all agent caches");
        Ok(())
    }

    /// Set up agent-specific invalidation rules
    pub async fn setup_agent_rules(&self) -> HiveResult<()> {
        let rules = vec![
            InvalidationRule {
                key_pattern: "agent".to_string(),
                ttl: Duration::from_secs(300),
                dependencies: vec!["agent_metrics".to_string()],
                priority: 1,
                max_age: None,
                strategy: InvalidationStrategy::Immediate,
                tags: vec![],
            },
            InvalidationRule {
                key_pattern: "agent_metrics".to_string(),
                ttl: Duration::from_secs(60),
                dependencies: vec!["agent".to_string()],
                priority: 2,
                max_age: None,
                strategy: InvalidationStrategy::Immediate,
                tags: vec![],
            },
        ];

        for rule in rules {
            self.base_manager.add_rule(rule).await;
        }

        Ok(())
    }
}

/// Specialized invalidation manager for task-related caches
pub struct TaskCacheInvalidationManager {
    pub base_manager: Arc<CacheInvalidationManager>,
}

impl TaskCacheInvalidationManager {
    #[must_use] 
    pub fn new(base_manager: Arc<CacheInvalidationManager>) -> Self {
        Self { base_manager }
    }

    /// Invalidate task data when task is updated
    pub async fn invalidate_task(&self, task_id: Uuid) -> HiveResult<()> {
        let keys = vec![CacheKey::Task(task_id), CacheKey::TaskMetrics(task_id)];

        self.base_manager.invalidate_keys(&keys).await?;
        info!("Invalidated cache for task: {}", task_id);
        Ok(())
    }

    /// Invalidate task assignment caches
    pub async fn invalidate_task_assignment(
        &self,
        agent_id: Uuid,
        task_id: Uuid,
    ) -> HiveResult<()> {
        let keys = vec![
            CacheKey::Agent(agent_id),
            CacheKey::Task(task_id),
            CacheKey::AgentMetrics(agent_id),
            CacheKey::TaskMetrics(task_id),
        ];

        self.base_manager.invalidate_keys(&keys).await?;
        info!(
            "Invalidated assignment caches for agent {} and task {}",
            agent_id, task_id
        );
        Ok(())
    }

    /// Set up task-specific invalidation rules
    pub async fn setup_task_rules(&self) -> HiveResult<()> {
        let rules = vec![
            InvalidationRule {
                key_pattern: "task".to_string(),
                ttl: Duration::from_secs(180),
                dependencies: vec!["task_metrics".to_string(), "agent".to_string()],
                priority: 1,
                max_age: None,
                strategy: InvalidationStrategy::Immediate,
                tags: vec![],
            },
            InvalidationRule {
                key_pattern: "task_metrics".to_string(),
                ttl: Duration::from_secs(30),
                dependencies: vec!["task".to_string()],
                priority: 3,
                max_age: None,
                strategy: InvalidationStrategy::Immediate,
                tags: vec![],
            },
        ];

        for rule in rules {
            self.base_manager.add_rule(rule).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cached_query::CachedQueryConfig;

    #[tokio::test]
    async fn test_dependency_graph() {
        let mut graph = DependencyGraph::new();

        let key1 = CacheKey::Agent(Uuid::new_v4());
        let key2 = CacheKey::Task(Uuid::new_v4());
        let key3 = CacheKey::AgentMetrics(Uuid::new_v4());

        // key1 depends on key2
        graph.add_dependency(key1.clone(), key2.clone());
        // key3 depends on key1
        graph.add_dependency(key3.clone(), key1.clone());

        assert!(graph.get_dependencies(&key1).contains(&key2));
        assert!(graph.get_dependents(&key1).contains(&key3));
        assert!(graph.get_dependents(&key2).contains(&key1));
    }

    #[tokio::test]
    async fn test_invalidation_manager() -> Result<(), Box<dyn std::error::Error>> {
        let cache_manager = Arc::new(CachedQueryManager::new(CachedQueryConfig::default()));
        let invalidation_manager = Arc::new(CacheInvalidationManager::new(
            cache_manager,
            InvalidationStrategy::Immediate,
        ));

        let key = CacheKey::Custom("test_key".to_string());

        // Add a dependency
        let dep_key = CacheKey::Custom("dep_key".to_string());
        invalidation_manager
            .add_dependency(key.clone(), dep_key.clone())
            .await;

        // Invalidate the dependency
        invalidation_manager.invalidate_key(&dep_key).await?;

        // Check that dependent key was also invalidated
        let stats = invalidation_manager.get_stats().await;
        assert!(stats.cascade_invalidations > 0);

        Ok(())
    }
}
