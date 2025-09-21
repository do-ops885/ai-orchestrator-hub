//! CPU Load Balancer for Optimized Task Distribution
//!
//! This module implements intelligent CPU load balancing to improve performance
//! by distributing tasks across cores based on real-time load monitoring.

use crate::tasks::task::{Task, TaskPriority};
use crate::utils::error::{HiveError, HiveResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};
use uuid::Uuid;

/// Configuration for CPU load balancing
#[derive(Debug, Clone)]
pub struct LoadBalancerConfig {
    /// Number of worker threads per CPU core
    pub threads_per_core: usize,
    /// Load sampling interval in milliseconds
    pub load_sample_interval_ms: u64,
    /// Queue size per worker thread
    pub queue_size_per_worker: usize,
    /// Enable dynamic thread scaling
    pub enable_dynamic_scaling: bool,
    /// Minimum number of worker threads
    pub min_workers: usize,
    /// Maximum number of worker threads
    pub max_workers: usize,
    /// Load threshold for scaling up (0.0-1.0)
    pub scale_up_threshold: f64,
    /// Load threshold for scaling down (0.0-1.0)
    pub scale_down_threshold: f64,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        let cpu_count = num_cpus::get();
        Self {
            threads_per_core: 2,
            load_sample_interval_ms: 100, // 100ms sampling
            queue_size_per_worker: 50,
            enable_dynamic_scaling: true,
            min_workers: cpu_count,
            max_workers: cpu_count * 4,
            scale_up_threshold: 0.8,   // Scale up at 80% load
            scale_down_threshold: 0.3, // Scale down at 30% load
        }
    }
}

/// Statistics for CPU load balancer performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoadBalancerStats {
    /// Current number of active workers
    pub active_workers: usize,
    /// Peak number of workers
    pub peak_workers: usize,
    /// Total tasks processed
    pub tasks_processed: u64,
    /// Tasks currently in queues
    pub tasks_queued: u64,
    /// Average task processing time in milliseconds
    pub avg_processing_time_ms: f64,
    /// Current CPU load average (0.0-1.0)
    pub current_load_avg: f64,
    /// Load distribution across workers (variance)
    pub load_variance: f64,
    /// Number of load balancing operations performed
    pub rebalance_operations: u64,
    /// Worker scaling events
    pub scale_up_events: u64,
    pub scale_down_events: u64,
}

/// Individual worker thread statistics
#[derive(Debug, Clone, Default)]
struct WorkerStats {
    worker_id: usize,
    tasks_processed: Arc<AtomicU64>,
    queue_size: Arc<AtomicUsize>,
    current_load: Arc<std::sync::atomic::AtomicU64>, // Store as u64 for atomic ops
    last_task_time: Arc<RwLock<Option<Instant>>>,
}

impl WorkerStats {
    fn new(worker_id: usize) -> Self {
        Self {
            worker_id,
            tasks_processed: Arc::new(AtomicU64::new(0)),
            queue_size: Arc::new(AtomicUsize::new(0)),
            current_load: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            last_task_time: Arc::new(RwLock::new(None)),
        }
    }

    fn get_load(&self) -> f64 {
        f64::from_bits(self.current_load.load(Ordering::Relaxed))
    }

    fn set_load(&self, load: f64) {
        self.current_load.store(load.to_bits(), Ordering::Relaxed);
    }
}

/// Task wrapper with metadata for load balancing
#[derive(Debug)]
struct LoadBalancedTask {
    task: Task,
    queued_at: Instant,
    priority_boost: f64,
}

impl LoadBalancedTask {
    fn new(task: Task) -> Self {
        let priority_boost = match task.priority {
            TaskPriority::Low => 0.0,
            TaskPriority::Medium => 0.5,
            TaskPriority::High => 1.0,
            TaskPriority::Critical => 2.0,
        };

        Self {
            task,
            queued_at: Instant::now(),
            priority_boost,
        }
    }

    /// Calculate effective priority including wait time boost
    fn effective_priority(&self) -> f64 {
        let wait_time_secs = self.queued_at.elapsed().as_secs_f64();
        let wait_boost = (wait_time_secs / 10.0).min(1.0); // Max 1.0 boost after 10 seconds
        self.priority_boost + wait_boost
    }
}

/// Individual worker thread for task processing
struct WorkerThread {
    id: usize,
    task_queue: Arc<Mutex<VecDeque<LoadBalancedTask>>>,
    stats: WorkerStats,
    semaphore: Arc<Semaphore>,
    shutdown_signal: Arc<tokio::sync::Notify>,
}

impl WorkerThread {
    fn new(id: usize, queue_capacity: usize) -> Self {
        Self {
            id,
            task_queue: Arc::new(Mutex::new(VecDeque::with_capacity(queue_capacity))),
            stats: WorkerStats::new(id),
            semaphore: Arc::new(Semaphore::new(queue_capacity)),
            shutdown_signal: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Start the worker thread
    async fn start(self: Arc<Self>) {
        let worker_id = self.id;
        let task_queue = Arc::clone(&self.task_queue);
        let stats = self.stats.clone();
        let shutdown = Arc::clone(&self.shutdown_signal);

        tokio::spawn(async move {
            tracing::debug!("Worker {} started", worker_id);

            loop {
                tokio::select! {
                    _ = shutdown.notified() => {
                        tracing::debug!("Worker {} shutting down", worker_id);
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {
                        // Check for tasks to process
                        let task_opt = {
                            let mut queue = task_queue.lock().await;
                            queue.pop_front()
                        };

                        if let Some(lb_task) = task_opt {
                            let start_time = Instant::now();
                            stats.queue_size.fetch_sub(1, Ordering::Relaxed);
                            
                            // Update load based on queue size
                            let queue_size = stats.queue_size.load(Ordering::Relaxed);
                            let load = queue_size as f64 / 50.0; // Normalize to queue capacity
                            stats.set_load(load.min(1.0));

                            // Process the task (placeholder - would call actual task execution)
                            Self::process_task(lb_task.task).await;

                            // Update statistics
                            let processing_time = start_time.elapsed();
                            stats.tasks_processed.fetch_add(1, Ordering::Relaxed);
                            
                            let mut last_task_time = stats.last_task_time.write().await;
                            *last_task_time = Some(start_time);
                        } else {
                            // No tasks, reduce load
                            stats.set_load(0.0);
                        }
                    }
                }
            }
        });
    }

    /// Add a task to this worker's queue
    async fn add_task(&self, task: LoadBalancedTask) -> HiveResult<()> {
        // Check if we can acquire a permit (non-blocking)
        if let Ok(_permit) = self.semaphore.try_acquire() {
            let mut queue = self.task_queue.lock().await;
            queue.push_back(task);
            self.stats.queue_size.fetch_add(1, Ordering::Relaxed);
            Ok(())
        } else {
            Err(HiveError::ResourceExhausted {
                resource: format!("Worker {} queue is full", self.id),
            })
        }
    }

    /// Get current queue size
    async fn queue_size(&self) -> usize {
        self.stats.queue_size.load(Ordering::Relaxed)
    }

    /// Get current load (0.0-1.0)
    fn current_load(&self) -> f64 {
        self.stats.get_load()
    }

    /// Placeholder for actual task processing
    async fn process_task(task: Task) {
        // Simulate task processing time based on priority
        let process_time = match task.priority {
            TaskPriority::Low => Duration::from_millis(50),
            TaskPriority::Medium => Duration::from_millis(30),
            TaskPriority::High => Duration::from_millis(20),
            TaskPriority::Critical => Duration::from_millis(10),
        };

        tokio::time::sleep(process_time).await;
    }

    /// Shutdown the worker
    async fn shutdown(&self) {
        self.shutdown_signal.notify_one();
    }
}

/// Main CPU load balancer
pub struct CpuLoadBalancer {
    config: LoadBalancerConfig,
    workers: Arc<RwLock<Vec<Arc<WorkerThread>>>>,
    stats: Arc<RwLock<LoadBalancerStats>>,
    task_counter: Arc<AtomicU64>,
    rebalance_counter: Arc<AtomicU64>,
}

impl CpuLoadBalancer {
    /// Create a new CPU load balancer
    pub fn new(config: LoadBalancerConfig) -> Self {
        let balancer = Self {
            config: config.clone(),
            workers: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(LoadBalancerStats::default())),
            task_counter: Arc::new(AtomicU64::new(0)),
            rebalance_counter: Arc::new(AtomicU64::new(0)),
        };

        // Initialize workers
        let balancer_clone = balancer.clone();
        tokio::spawn(async move {
            balancer_clone.initialize_workers().await;
            balancer_clone.start_load_monitoring().await;
        });

        balancer
    }

    /// Initialize worker threads
    async fn initialize_workers(&self) {
        let mut workers = self.workers.write().await;
        
        for i in 0..self.config.min_workers {
            let worker = Arc::new(WorkerThread::new(i, self.config.queue_size_per_worker));
            worker.clone().start().await;
            workers.push(worker);
        }

        let mut stats = self.stats.write().await;
        stats.active_workers = workers.len();
        stats.peak_workers = workers.len();

        tracing::info!("Initialized {} worker threads", workers.len());
    }

    /// Start background load monitoring and auto-scaling
    async fn start_load_monitoring(&self) {
        let workers = Arc::clone(&self.workers);
        let stats = Arc::clone(&self.stats);
        let config = self.config.clone();
        let rebalance_counter = Arc::clone(&self.rebalance_counter);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(config.load_sample_interval_ms));

            loop {
                interval.tick().await;

                // Calculate current system load
                let current_load = Self::calculate_system_load(&workers).await;
                
                // Update statistics
                {
                    let mut stats_guard = stats.write().await;
                    stats_guard.current_load_avg = current_load;
                    stats_guard.rebalance_operations = rebalance_counter.load(Ordering::Relaxed);
                }

                // Check if scaling is needed
                if config.enable_dynamic_scaling {
                    if current_load > config.scale_up_threshold {
                        Self::scale_up(&workers, &config, &stats).await;
                    } else if current_load < config.scale_down_threshold {
                        Self::scale_down(&workers, &config, &stats).await;
                    }
                }
            }
        });
    }

    /// Calculate system load across all workers
    async fn calculate_system_load(workers: &Arc<RwLock<Vec<Arc<WorkerThread>>>>) -> f64 {
        let workers_guard = workers.read().await;
        if workers_guard.is_empty() {
            return 0.0;
        }

        let total_load: f64 = workers_guard
            .iter()
            .map(|worker| worker.current_load())
            .sum();

        total_load / workers_guard.len() as f64
    }

    /// Scale up by adding more workers
    async fn scale_up(
        workers: &Arc<RwLock<Vec<Arc<WorkerThread>>>>,
        config: &LoadBalancerConfig,
        stats: &Arc<RwLock<LoadBalancerStats>>,
    ) {
        let mut workers_guard = workers.write().await;
        
        if workers_guard.len() < config.max_workers {
            let new_worker_id = workers_guard.len();
            let worker = Arc::new(WorkerThread::new(new_worker_id, config.queue_size_per_worker));
            worker.clone().start().await;
            workers_guard.push(worker);

            let mut stats_guard = stats.write().await;
            stats_guard.active_workers = workers_guard.len();
            stats_guard.peak_workers = stats_guard.peak_workers.max(workers_guard.len());
            stats_guard.scale_up_events += 1;

            tracing::info!("Scaled up to {} workers", workers_guard.len());
        }
    }

    /// Scale down by removing excess workers
    async fn scale_down(
        workers: &Arc<RwLock<Vec<Arc<WorkerThread>>>>,
        config: &LoadBalancerConfig,
        stats: &Arc<RwLock<LoadBalancerStats>>,
    ) {
        let mut workers_guard = workers.write().await;
        
        if workers_guard.len() > config.min_workers {
            if let Some(worker) = workers_guard.pop() {
                worker.shutdown().await;

                let mut stats_guard = stats.write().await;
                stats_guard.active_workers = workers_guard.len();
                stats_guard.scale_down_events += 1;

                tracing::info!("Scaled down to {} workers", workers_guard.len());
            }
        }
    }

    /// Submit a task for load-balanced processing
    pub async fn submit_task(&self, task: Task) -> HiveResult<()> {
        let lb_task = LoadBalancedTask::new(task);
        let optimal_worker = self.find_optimal_worker().await?;
        
        optimal_worker.add_task(lb_task).await?;
        
        self.task_counter.fetch_add(1, Ordering::Relaxed);
        self.rebalance_counter.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }

    /// Find the optimal worker for task assignment
    async fn find_optimal_worker(&self) -> HiveResult<Arc<WorkerThread>> {
        let workers = self.workers.read().await;
        
        if workers.is_empty() {
            return Err(HiveError::ResourceExhausted {
                resource: "No workers available".to_string(),
            });
        }

        // Find worker with lowest load
        let mut best_worker = None;
        let mut best_score = f64::INFINITY;

        for worker in workers.iter() {
            let load = worker.current_load();
            let queue_size = worker.queue_size().await as f64;
            
            // Combined score: lower is better
            let score = load * 0.7 + (queue_size / self.config.queue_size_per_worker as f64) * 0.3;
            
            if score < best_score {
                best_score = score;
                best_worker = Some(Arc::clone(worker));
            }
        }

        best_worker.ok_or_else(|| HiveError::OperationFailed {
            reason: "Could not find suitable worker".to_string(),
        })
    }

    /// Get comprehensive load balancer statistics
    pub async fn get_stats(&self) -> LoadBalancerStats {
        let mut stats = self.stats.read().await.clone();
        
        // Update real-time counters
        stats.tasks_processed = self.task_counter.load(Ordering::Relaxed);
        stats.rebalance_operations = self.rebalance_counter.load(Ordering::Relaxed);
        
        // Calculate load variance
        let workers = self.workers.read().await;
        if !workers.is_empty() {
            let loads: Vec<f64> = workers.iter().map(|w| w.current_load()).collect();
            let mean_load = loads.iter().sum::<f64>() / loads.len() as f64;
            let variance = loads
                .iter()
                .map(|load| (load - mean_load).powi(2))
                .sum::<f64>() / loads.len() as f64;
            stats.load_variance = variance;
            
            // Update queue sizes
            let mut total_queued = 0;
            for worker in workers.iter() {
                total_queued += worker.queue_size().await;
            }
            stats.tasks_queued = total_queued as u64;
        }
        
        stats
    }

    /// Get current worker count
    pub async fn worker_count(&self) -> usize {
        self.workers.read().await.len()
    }

    /// Get efficiency metrics
    pub async fn get_efficiency_metrics(&self) -> LoadBalancerEfficiency {
        let stats = self.get_stats().await;
        
        let load_distribution_score = if stats.load_variance < 0.1 {
            1.0 // Excellent distribution
        } else if stats.load_variance < 0.3 {
            0.7 // Good distribution
        } else {
            0.3 // Poor distribution
        };

        let utilization_score = stats.current_load_avg;
        
        let throughput_score = if stats.avg_processing_time_ms < 50.0 {
            1.0
        } else if stats.avg_processing_time_ms < 100.0 {
            0.7
        } else {
            0.3
        };

        LoadBalancerEfficiency {
            load_distribution_score: load_distribution_score * 100.0,
            utilization_score: utilization_score * 100.0,
            throughput_score: throughput_score * 100.0,
            overall_efficiency: (load_distribution_score + utilization_score + throughput_score) / 3.0 * 100.0,
        }
    }
}

impl Clone for CpuLoadBalancer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            workers: Arc::clone(&self.workers),
            stats: Arc::clone(&self.stats),
            task_counter: Arc::clone(&self.task_counter),
            rebalance_counter: Arc::clone(&self.rebalance_counter),
        }
    }
}

/// Load balancer efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerEfficiency {
    pub load_distribution_score: f64,
    pub utilization_score: f64,
    pub throughput_score: f64,
    pub overall_efficiency: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::task::TaskPriority;

    #[tokio::test]
    async fn test_load_balancer_creation() {
        let config = LoadBalancerConfig::default();
        let balancer = CpuLoadBalancer::new(config);
        
        // Give time for initialization
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let worker_count = balancer.worker_count().await;
        assert!(worker_count >= num_cpus::get());
    }

    #[tokio::test]
    async fn test_task_submission() {
        let config = LoadBalancerConfig {
            min_workers: 2,
            max_workers: 4,
            ..Default::default()
        };
        let balancer = CpuLoadBalancer::new(config);
        
        // Give time for initialization
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let task = Task {
            id: Uuid::new_v4(),
            task_type: "test".to_string(),
            description: "Test task".to_string(),
            priority: TaskPriority::Medium,
            created_at: chrono::Utc::now(),
            agent_id: Some(Uuid::new_v4()),
            metadata: std::collections::HashMap::new(),
        };
        
        let result = balancer.submit_task(task).await;
        assert!(result.is_ok());
        
        let stats = balancer.get_stats().await;
        assert_eq!(stats.tasks_processed, 1);
    }

    #[tokio::test]
    async fn test_efficiency_metrics() {
        let config = LoadBalancerConfig::default();
        let balancer = CpuLoadBalancer::new(config);
        
        // Give time for initialization
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let efficiency = balancer.get_efficiency_metrics().await;
        assert!(efficiency.overall_efficiency >= 0.0);
        assert!(efficiency.overall_efficiency <= 100.0);
    }
}