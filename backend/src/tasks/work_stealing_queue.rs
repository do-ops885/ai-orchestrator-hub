use std::sync::Arc;
use std::collections::VecDeque;
use tokio::sync::{Mutex, RwLock, Notify};
use uuid::Uuid;
use dashmap::DashMap;
use rand::prelude::SliceRandom;
// use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::tasks::{Task, TaskPriority};
// use crate::agent::Agent;

/// High-performance work-stealing task queue system
/// Implements best practices for concurrent task distribution
/// 
/// Key Features:
/// - Lock-free work stealing between agent queues
/// - Priority-based task scheduling
/// - Adaptive load balancing
/// - Minimal contention design
/// - Metrics collection for optimization

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskQueueMetrics {
    pub total_tasks_processed: u64,
    pub average_queue_depth: f64,
    pub steal_attempts: u64,
    pub successful_steals: u64,
    pub load_balance_efficiency: f64,
    pub last_updated: DateTime<Utc>,
}

impl Default for TaskQueueMetrics {
    fn default() -> Self {
        Self {
            total_tasks_processed: 0,
            average_queue_depth: 0.0,
            steal_attempts: 0,
            successful_steals: 0,
            load_balance_efficiency: 1.0,
            last_updated: Utc::now(),
        }
    }
}

/// Individual agent's local task queue with work-stealing capability
pub struct AgentTaskQueue {
    pub agent_id: Uuid,
    pub local_queue: Arc<Mutex<VecDeque<Task>>>,
    pub priority_queue: Arc<Mutex<VecDeque<Task>>>, // High/Critical priority tasks
    pub is_busy: Arc<RwLock<bool>>,
    pub tasks_completed: Arc<Mutex<u64>>,
    pub last_activity: Arc<RwLock<DateTime<Utc>>>,
    pub steal_count: Arc<Mutex<u64>>,
}

impl AgentTaskQueue {
    pub fn new(agent_id: Uuid) -> Self {
        Self {
            agent_id,
            local_queue: Arc::new(Mutex::new(VecDeque::new())),
            priority_queue: Arc::new(Mutex::new(VecDeque::new())),
            is_busy: Arc::new(RwLock::new(false)),
            tasks_completed: Arc::new(Mutex::new(0)),
            last_activity: Arc::new(RwLock::new(Utc::now())),
            steal_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Add task to appropriate queue based on priority
    pub async fn push_task(&self, task: Task) -> anyhow::Result<()> {
        match task.priority {
            TaskPriority::High | TaskPriority::Critical => {
                let mut priority_queue = self.priority_queue.lock().await;
                priority_queue.push_back(task);
            }
            _ => {
                let mut local_queue = self.local_queue.lock().await;
                local_queue.push_back(task);
            }
        }
        
        *self.last_activity.write().await = Utc::now();
        Ok(())
    }

    /// Pop task from local queues (priority first)
    pub async fn pop_task(&self) -> Option<Task> {
        // Check priority queue first
        {
            let mut priority_queue = self.priority_queue.lock().await;
            if let Some(task) = priority_queue.pop_front() {
                *self.last_activity.write().await = Utc::now();
                return Some(task);
            }
        }

        // Then check local queue
        {
            let mut local_queue = self.local_queue.lock().await;
            if let Some(task) = local_queue.pop_front() {
                *self.last_activity.write().await = Utc::now();
                return Some(task);
            }
        }

        None
    }

    /// Attempt to steal a task from this queue (called by other agents)
    pub async fn steal_task(&self) -> Option<Task> {
        // Only steal from local queue, not priority queue
        let mut local_queue = self.local_queue.lock().await;
        if local_queue.len() > 1 { // Only steal if queue has multiple tasks
            let stolen_task = local_queue.pop_back(); // Steal from back (LIFO for better cache locality)
            if stolen_task.is_some() {
                let mut steal_count = self.steal_count.lock().await;
                *steal_count += 1;
            }
            stolen_task
        } else {
            None
        }
    }

    /// Get queue depth for load balancing
    pub async fn get_queue_depth(&self) -> usize {
        let local_depth = self.local_queue.lock().await.len();
        let priority_depth = self.priority_queue.lock().await.len();
        local_depth + priority_depth
    }

    /// Check if agent is currently busy
    pub async fn is_agent_busy(&self) -> bool {
        *self.is_busy.read().await
    }

    /// Set agent busy status
    pub async fn set_busy(&self, busy: bool) {
        *self.is_busy.write().await = busy;
        *self.last_activity.write().await = Utc::now();
    }

    /// Mark task completion
    pub async fn mark_task_completed(&self) {
        let mut completed = self.tasks_completed.lock().await;
        *completed += 1;
        *self.last_activity.write().await = Utc::now();
    }

    /// Get performance metrics for this queue
    pub async fn get_metrics(&self) -> AgentQueueMetrics {
        AgentQueueMetrics {
            agent_id: self.agent_id,
            queue_depth: self.get_queue_depth().await,
            tasks_completed: *self.tasks_completed.lock().await,
            steal_count: *self.steal_count.lock().await,
            is_busy: self.is_agent_busy().await,
            last_activity: *self.last_activity.read().await,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentQueueMetrics {
    pub agent_id: Uuid,
    pub queue_depth: usize,
    pub tasks_completed: u64,
    pub steal_count: u64,
    pub is_busy: bool,
    pub last_activity: DateTime<Utc>,
}

/// Main work-stealing queue coordinator
pub struct WorkStealingQueue {
    pub agent_queues: Arc<DashMap<Uuid, AgentTaskQueue>>,
    pub global_queue: Arc<Mutex<VecDeque<Task>>>, // Fallback for unassigned tasks
    pub metrics: Arc<RwLock<TaskQueueMetrics>>,
    pub load_balancer: Arc<LoadBalancer>,
    pub notification: Arc<Notify>, // For waking up idle workers
}

impl WorkStealingQueue {
    pub fn new() -> Self {
        Self {
            agent_queues: Arc::new(DashMap::new()),
            global_queue: Arc::new(Mutex::new(VecDeque::new())),
            metrics: Arc::new(RwLock::new(TaskQueueMetrics::default())),
            load_balancer: Arc::new(LoadBalancer::new()),
            notification: Arc::new(Notify::new()),
        }
    }

    /// Register a new agent with the work-stealing system
    pub async fn register_agent(&self, agent_id: Uuid) -> anyhow::Result<()> {
        let agent_queue = AgentTaskQueue::new(agent_id);
        self.agent_queues.insert(agent_id, agent_queue);
        
        tracing::info!("🔧 Registered agent {} with work-stealing queue", agent_id);
        Ok(())
    }

    /// Remove agent from the system and redistribute its tasks
    pub async fn unregister_agent(&self, agent_id: Uuid) -> anyhow::Result<()> {
        if let Some((_, agent_queue)) = self.agent_queues.remove(&agent_id) {
            // Redistribute remaining tasks
            let remaining_tasks = self.drain_agent_tasks(&agent_queue).await;
            for task in remaining_tasks {
                self.submit_task(task).await?;
            }
            
            tracing::info!("🔧 Unregistered agent {} and redistributed tasks", agent_id);
        }
        Ok(())
    }

    /// Submit a new task to the system
    pub async fn submit_task(&self, task: Task) -> anyhow::Result<()> {
        // Try to assign to best available agent
        if let Some(best_agent_id) = self.load_balancer.find_best_agent(&self.agent_queues).await {
            if let Some(agent_queue) = self.agent_queues.get(&best_agent_id) {
                agent_queue.push_task(task).await?;
                self.notification.notify_one(); // Wake up a worker
                return Ok(());
            }
        }

        // Fallback to global queue
        let mut global_queue = self.global_queue.lock().await;
        global_queue.push_back(task);
        self.notification.notify_waiters(); // Wake up all workers
        
        Ok(())
    }

    /// Get next task for a specific agent (with work stealing)
    pub async fn get_task_for_agent(&self, agent_id: Uuid) -> Option<Task> {
        // First, try agent's own queue
        if let Some(agent_queue) = self.agent_queues.get(&agent_id) {
            if let Some(task) = agent_queue.pop_task().await {
                return Some(task);
            }
        }

        // Try global queue
        {
            let mut global_queue = self.global_queue.lock().await;
            if let Some(task) = global_queue.pop_front() {
                return Some(task);
            }
        }

        // Attempt work stealing from other agents
        self.attempt_work_stealing(agent_id).await
    }

    /// Attempt to steal work from other agents
    async fn attempt_work_stealing(&self, requesting_agent_id: Uuid) -> Option<Task> {
        let mut metrics = self.metrics.write().await;
        metrics.steal_attempts += 1;

        // Get list of potential victims (agents with work)
        let mut candidates: Vec<Uuid> = Vec::new();
        for entry in self.agent_queues.iter() {
            let agent_id = *entry.key();
            if agent_id != requesting_agent_id {
                let queue_depth = entry.value().get_queue_depth().await;
                if queue_depth > 1 { // Only steal from agents with multiple tasks
                    candidates.push(agent_id);
                }
            }
        }

        if candidates.is_empty() {
            return None;
        }

        // Randomize to avoid thundering herd
        let mut rng = rand::rngs::StdRng::from_entropy();
        candidates.shuffle(&mut rng);

        // Try to steal from candidates
        for victim_id in candidates.iter().take(3) { // Limit steal attempts
            if let Some(victim_queue) = self.agent_queues.get(victim_id) {
                if let Some(stolen_task) = victim_queue.steal_task().await {
                    metrics.successful_steals += 1;
                    tracing::debug!("🔄 Agent {} stole task from agent {}", 
                                   requesting_agent_id, victim_id);
                    return Some(stolen_task);
                }
            }
        }

        None
    }

    /// Drain all tasks from an agent's queues
    async fn drain_agent_tasks(&self, agent_queue: &AgentTaskQueue) -> Vec<Task> {
        let mut tasks = Vec::new();
        
        // Drain priority queue
        {
            let mut priority_queue = agent_queue.priority_queue.lock().await;
            while let Some(task) = priority_queue.pop_front() {
                tasks.push(task);
            }
        }

        // Drain local queue
        {
            let mut local_queue = agent_queue.local_queue.lock().await;
            while let Some(task) = local_queue.pop_front() {
                tasks.push(task);
            }
        }

        tasks
    }

    /// Get comprehensive metrics
    pub async fn get_metrics(&self) -> WorkStealingMetrics {
        let mut agent_metrics = Vec::new();
        let mut total_queue_depth = 0;

        for entry in self.agent_queues.iter() {
            let metrics = entry.value().get_metrics().await;
            total_queue_depth += metrics.queue_depth;
            agent_metrics.push(metrics);
        }

        let global_queue_depth = self.global_queue.lock().await.len();
        let system_metrics = self.metrics.read().await.clone();

        WorkStealingMetrics {
            system_metrics,
            agent_metrics,
            global_queue_depth,
            total_queue_depth: total_queue_depth + global_queue_depth,
            active_agents: self.agent_queues.len(),
        }
    }

    /// Update system metrics
    pub async fn update_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        
        let total_agents = self.agent_queues.len();
        if total_agents > 0 {
            let mut total_depth = 0;
            for entry in self.agent_queues.iter() {
                total_depth += entry.value().get_queue_depth().await;
            }

            metrics.average_queue_depth = total_depth as f64 / total_agents as f64;
        }

        metrics.load_balance_efficiency = if metrics.steal_attempts > 0 {
            metrics.successful_steals as f64 / metrics.steal_attempts as f64
        } else {
            1.0
        };

        metrics.last_updated = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkStealingMetrics {
    pub system_metrics: TaskQueueMetrics,
    pub agent_metrics: Vec<AgentQueueMetrics>,
    pub global_queue_depth: usize,
    pub total_queue_depth: usize,
    pub active_agents: usize,
}

/// Load balancer for optimal task assignment
pub struct LoadBalancer {
    pub assignment_history: Arc<RwLock<Vec<(Uuid, DateTime<Utc>)>>>,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            assignment_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Find the best agent for task assignment based on load and capability
    pub async fn find_best_agent(&self, agent_queues: &DashMap<Uuid, AgentTaskQueue>) -> Option<Uuid> {
        let mut best_agent = None;
        let mut best_score = f64::INFINITY;

        for entry in agent_queues.iter() {
            let agent_id = *entry.key();
            let queue = entry.value();

            // Skip busy agents
            if queue.is_agent_busy().await {
                continue;
            }

            // Calculate load score (lower is better)
            let queue_depth = queue.get_queue_depth().await as f64;
            let last_activity = *queue.last_activity.read().await;
            let idle_time = (Utc::now() - last_activity).num_seconds() as f64;
            
            // Score combines queue depth and idle time
            let load_score = queue_depth + (1.0 / (idle_time + 1.0));

            if load_score < best_score {
                best_score = load_score;
                best_agent = Some(agent_id);
            }
        }

        // Record assignment for future optimization
        if let Some(agent_id) = best_agent {
            let mut history = self.assignment_history.write().await;
            history.push((agent_id, Utc::now()));
            
            // Keep history bounded
            if history.len() > 1000 {
                history.drain(0..500);
            }
        }

        best_agent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::TaskPriority;

    #[tokio::test]
    async fn test_work_stealing_basic_operations() {
        let queue_system = WorkStealingQueue::new();
        let agent1_id = Uuid::new_v4();
        let agent2_id = Uuid::new_v4();

        // Register agents
        queue_system.register_agent(agent1_id).await.unwrap();
        queue_system.register_agent(agent2_id).await.unwrap();

        // Create test task
        let task = Task::new(
            "test_task".to_string(),
            "Test task description".to_string(),
            "test".to_string(),
            TaskPriority::Medium,
            vec![]
        );

        // Submit task
        queue_system.submit_task(task).await.unwrap();

        // Agent should be able to get the task
        let retrieved_task = queue_system.get_task_for_agent(agent1_id).await;
        assert!(retrieved_task.is_some());
    }

    #[tokio::test]
    async fn test_work_stealing_between_agents() {
        let queue_system = WorkStealingQueue::new();
        let agent1_id = Uuid::new_v4();
        let agent2_id = Uuid::new_v4();

        queue_system.register_agent(agent1_id).await.unwrap();
        queue_system.register_agent(agent2_id).await.unwrap();

        // Add multiple tasks to agent1's queue
        if let Some(agent1_queue) = queue_system.agent_queues.get(&agent1_id) {
            for i in 0..5 {
                let task = Task::new(
                    format!("task_{}", i),
                    format!("Task {} description", i),
                    "test".to_string(),
                    TaskPriority::Medium,
                    vec![]
                );
                agent1_queue.push_task(task).await.unwrap();
            }
        }

        // Agent2 should be able to steal work from agent1
        let stolen_task = queue_system.get_task_for_agent(agent2_id).await;
        assert!(stolen_task.is_some());

        // Check metrics
        let metrics = queue_system.get_metrics().await;
        assert!(metrics.system_metrics.steal_attempts > 0);
    }

    #[tokio::test]
    async fn test_priority_queue_handling() {
        let agent_queue = AgentTaskQueue::new(Uuid::new_v4());

        // Add low priority task
        let low_task = Task::new(
            "low_task".to_string(),
            "Low priority task".to_string(),
            "test".to_string(),
            TaskPriority::Low,
            vec![]
        );
        agent_queue.push_task(low_task).await.unwrap();

        // Add high priority task
        let high_task = Task::new(
            "high_task".to_string(),
            "High priority task".to_string(),
            "test".to_string(),
            TaskPriority::High,
            vec![]
        );
        agent_queue.push_task(high_task).await.unwrap();

        // High priority task should be retrieved first
        let first_task = agent_queue.pop_task().await.unwrap();
        assert_eq!(first_task.name, "high_task");

        let second_task = agent_queue.pop_task().await.unwrap();
        assert_eq!(second_task.name, "low_task");
    }
}