use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::agents::{AgentCapability, AgentMemory, AgentState, AgentType};

/// High-performance memory pool for agent management
/// Separates hot (frequently accessed) and cold (rarely accessed) data
/// Implements object pooling to reduce allocation overhead

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHotData {
    pub id: Uuid,
    pub name: String,
    pub agent_type: AgentType,
    pub state: AgentState,
    pub position: (f32, f32), // f32 for better cache performance
    pub energy: f32,
    pub last_activity: DateTime<Utc>,
    pub current_task_id: Option<Uuid>,
    pub performance_score: f32, // Cached for quick access
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentColdData {
    pub capabilities: Vec<AgentCapability>,
    pub memory: AgentMemory,
    pub social_connections: std::collections::HashMap<Uuid, f64>,
    pub learning_history: Vec<LearningEvent>,
    pub detailed_stats: AgentDetailedStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub improvement: f64,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDetailedStats {
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub total_learning_time: u64, // milliseconds
    pub collaboration_count: u32,
    pub specialization_level: f64,
}

impl Default for AgentDetailedStats {
    fn default() -> Self {
        Self {
            tasks_completed: 0,
            tasks_failed: 0,
            total_learning_time: 0,
            collaboration_count: 0,
            specialization_level: 0.0,
        }
    }
}

/// Memory pool for efficient agent object management
pub struct AgentMemoryPool {
    hot_data_pool: Arc<Mutex<VecDeque<AgentHotData>>>,
    cold_data_pool: Arc<Mutex<VecDeque<AgentColdData>>>,
    pool_size: usize,
    allocation_count: Arc<Mutex<u64>>,
    deallocation_count: Arc<Mutex<u64>>,
}

impl AgentMemoryPool {
    #[must_use]
    pub fn new(initial_pool_size: usize) -> Self {
        let mut hot_pool = VecDeque::with_capacity(initial_pool_size);
        let mut cold_pool = VecDeque::with_capacity(initial_pool_size);

        // Pre-allocate pool objects
        for _ in 0..initial_pool_size {
            hot_pool.push_back(Self::create_default_hot_data());
            cold_pool.push_back(Self::create_default_cold_data());
        }

        Self {
            hot_data_pool: Arc::new(Mutex::new(hot_pool)),
            cold_data_pool: Arc::new(Mutex::new(cold_pool)),
            pool_size: initial_pool_size,
            allocation_count: Arc::new(Mutex::new(0)),
            deallocation_count: Arc::new(Mutex::new(0)),
        }
    }

    fn create_default_hot_data() -> AgentHotData {
        AgentHotData {
            id: Uuid::new_v4(),
            name: String::new(),
            agent_type: AgentType::Worker,
            state: AgentState::Idle,
            position: (0.0, 0.0),
            energy: 100.0,
            last_activity: Utc::now(),
            current_task_id: None,
            performance_score: 0.0,
        }
    }

    fn create_default_cold_data() -> AgentColdData {
        AgentColdData {
            capabilities: Vec::new(),
            memory: AgentMemory::new(),
            social_connections: std::collections::HashMap::new(),
            learning_history: Vec::new(),
            detailed_stats: AgentDetailedStats::default(),
        }
    }

    /// Acquire hot data object from pool
    pub async fn acquire_hot_data(&self) -> AgentHotData {
        let mut pool = self.hot_data_pool.lock().await;
        let mut alloc_count = self.allocation_count.lock().await;
        *alloc_count += 1;

        pool.pop_front().unwrap_or_else(|| {
            tracing::debug!("Hot data pool exhausted, creating new object");
            Self::create_default_hot_data()
        })
    }

    /// Acquire cold data object from pool
    pub async fn acquire_cold_data(&self) -> AgentColdData {
        let mut pool = self.cold_data_pool.lock().await;
        let mut alloc_count = self.allocation_count.lock().await;
        *alloc_count += 1;

        pool.pop_front().unwrap_or_else(|| {
            tracing::debug!("Cold data pool exhausted, creating new object");
            Self::create_default_cold_data()
        })
    }

    /// Return hot data object to pool
    pub async fn release_hot_data(&self, mut data: AgentHotData) {
        // Reset object state for reuse
        data.id = Uuid::new_v4();
        data.name.clear();
        data.agent_type = AgentType::Worker;
        data.state = AgentState::Idle;
        data.position = (0.0, 0.0);
        data.energy = 100.0;
        data.last_activity = Utc::now();
        data.current_task_id = None;
        data.performance_score = 0.0;

        let mut pool = self.hot_data_pool.lock().await;
        let mut dealloc_count = self.deallocation_count.lock().await;
        *dealloc_count += 1;

        if pool.len() < self.pool_size * 2 {
            pool.push_back(data);
        }
        // If pool is too large, let object be dropped
    }

    /// Return cold data object to pool
    pub async fn release_cold_data(&self, mut data: AgentColdData) {
        // Reset object state for reuse
        data.capabilities.clear();
        data.memory = AgentMemory::new();
        data.social_connections.clear();
        data.learning_history.clear();
        data.detailed_stats = AgentDetailedStats::default();

        let mut pool = self.cold_data_pool.lock().await;
        let mut dealloc_count = self.deallocation_count.lock().await;
        *dealloc_count += 1;

        if pool.len() < self.pool_size * 2 {
            pool.push_back(data);
        }
    }

    /// Get pool statistics
    pub async fn get_pool_stats(&self) -> PoolStats {
        let hot_pool_size = self.hot_data_pool.lock().await.len();
        let cold_pool_size = self.cold_data_pool.lock().await.len();
        let allocations = *self.allocation_count.lock().await;
        let deallocations = *self.deallocation_count.lock().await;

        PoolStats {
            hot_pool_size,
            cold_pool_size,
            total_allocations: allocations,
            total_deallocations: deallocations,
            active_objects: allocations - deallocations,
            pool_efficiency: if allocations > 0 {
                (deallocations as f64 / allocations as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Resize pool based on usage patterns
    pub async fn optimize_pool_size(&self, target_size: usize) {
        let mut hot_pool = self.hot_data_pool.lock().await;
        let mut cold_pool = self.cold_data_pool.lock().await;

        // Adjust hot pool
        while hot_pool.len() < target_size {
            hot_pool.push_back(Self::create_default_hot_data());
        }
        while hot_pool.len() > target_size * 2 {
            hot_pool.pop_front();
        }

        // Adjust cold pool
        while cold_pool.len() < target_size {
            cold_pool.push_back(Self::create_default_cold_data());
        }
        while cold_pool.len() > target_size * 2 {
            cold_pool.pop_front();
        }

        tracing::info!(
            "Optimized pool sizes - Hot: {}, Cold: {}",
            hot_pool.len(),
            cold_pool.len()
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub hot_pool_size: usize,
    pub cold_pool_size: usize,
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub active_objects: u64,
    pub pool_efficiency: f64,
}

/// Optimized agent representation with separated hot/cold data
pub struct OptimizedAgent {
    pub hot_data: AgentHotData,
    pub cold_data_id: Uuid, // Reference to cold data storage
}

impl OptimizedAgent {
    #[must_use]
    pub fn new(hot_data: AgentHotData, cold_data_id: Uuid) -> Self {
        Self {
            hot_data,
            cold_data_id,
        }
    }

    /// Quick access to frequently used data
    #[must_use]
    pub fn get_performance_score(&self) -> f32 {
        self.hot_data.performance_score
    }

    pub fn update_position(&mut self, new_position: (f32, f32)) {
        self.hot_data.position = new_position;
        self.hot_data.last_activity = Utc::now();
    }

    pub fn update_energy(&mut self, energy_delta: f32) {
        self.hot_data.energy = (self.hot_data.energy + energy_delta).clamp(0.0, 100.0);
        self.hot_data.last_activity = Utc::now();
    }

    pub fn set_state(&mut self, new_state: AgentState) {
        self.hot_data.state = new_state;
        self.hot_data.last_activity = Utc::now();
    }

    pub fn assign_task(&mut self, task_id: Uuid) {
        self.hot_data.current_task_id = Some(task_id);
        self.hot_data.state = AgentState::Working;
        self.hot_data.last_activity = Utc::now();
    }

    pub fn complete_task(&mut self, performance_score: f32) {
        self.hot_data.current_task_id = None;
        self.hot_data.state = AgentState::Idle;
        self.hot_data.performance_score =
            (self.hot_data.performance_score * 0.9) + (performance_score * 0.1);
        self.hot_data.last_activity = Utc::now();
    }

    /// Check if agent is available for new tasks
    #[must_use]
    pub fn is_available(&self) -> bool {
        matches!(self.hot_data.state, AgentState::Idle) && self.hot_data.energy > 10.0
    }

    /// Calculate distance to another agent (for swarm coordination)
    #[must_use]
    pub fn distance_to(&self, other: &OptimizedAgent) -> f32 {
        let dx = self.hot_data.position.0 - other.hot_data.position.0;
        let dy = self.hot_data.position.1 - other.hot_data.position.1;
        (dx * dx + dy * dy).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_pool_basic_operations() {
        let pool = AgentMemoryPool::new(5);

        // Test hot data acquisition and release
        let hot_data = pool.acquire_hot_data().await;
        #[allow(clippy::float_cmp)]
        assert!((hot_data.energy - 100.0).abs() < f32::EPSILON);

        pool.release_hot_data(hot_data).await;

        // Test cold data acquisition and release
        let cold_data = pool.acquire_cold_data().await;
        assert!(cold_data.capabilities.is_empty());

        pool.release_cold_data(cold_data).await;

        // Check stats
        let stats = pool.get_pool_stats().await;
        assert!((stats.total_allocations - 2).abs() < f32::EPSILON);
        assert!((stats.total_deallocations - 2).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn test_optimized_agent_operations() {
        let hot_data = AgentHotData {
            id: Uuid::new_v4(),
            name: "TestAgent".to_string(),
            agent_type: AgentType::Worker,
            state: AgentState::Idle,
            position: (0.0, 0.0),
            energy: 100.0,
            last_activity: Utc::now(),
            current_task_id: None,
            performance_score: 0.5,
        };

        let mut agent = OptimizedAgent::new(hot_data, Uuid::new_v4());

        assert!(agent.is_available());
        #[allow(clippy::float_cmp)]
        assert!((agent.get_performance_score() - 0.5).abs() < f32::EPSILON);

        // Test task assignment
        let task_id = Uuid::new_v4();
        agent.assign_task(task_id);
        assert!(!agent.is_available());
        assert_eq!(agent.hot_data.current_task_id, Some(task_id));

        // Test task completion
        agent.complete_task(0.8);
        assert!(agent.is_available());
        assert!(agent.hot_data.performance_score > 0.5);
    }

    #[tokio::test]
    async fn test_pool_optimization() {
        let pool = AgentMemoryPool::new(2);

        // Acquire more objects than initial pool size
        let _hot1 = pool.acquire_hot_data().await;
        let _hot2 = pool.acquire_hot_data().await;
        let _hot3 = pool.acquire_hot_data().await; // Should create new object

        let stats = pool.get_pool_stats().await;
        assert!((stats.total_allocations - 3).abs() < f32::EPSILON);

        // Test pool resizing
        pool.optimize_pool_size(10).await;
        let stats_after = pool.get_pool_stats().await;
        assert!(stats_after.hot_pool_size >= 7); // Some objects are still in use
    }
}
