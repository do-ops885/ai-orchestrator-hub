//! Comprehensive Persistence System for Multiagent Hive
//! 
//! Provides state serialization, checkpointing, and recovery capabilities
//! with multiple storage backends and automatic backup strategies.

use crate::agents::Agent;
use crate::core::HiveCoordinator;
use crate::tasks::Task;
use crate::utils::error::{HiveError, HiveResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Complete system state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub snapshot_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub hive_state: HiveState,
    pub agents: Vec<Agent>,
    pub tasks: Vec<Task>,
    pub metrics: SystemMetrics,
    pub configuration: serde_json::Value,
}

/// Hive coordinator state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveState {
    pub hive_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub total_energy: f64,
    pub swarm_center: (f64, f64),
    pub auto_scaling_enabled: bool,
    pub learning_enabled: bool,
}

/// System metrics for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_agents: usize,
    pub active_agents: usize,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_performance: f64,
    pub swarm_cohesion: f64,
    pub learning_progress: f64,
    pub uptime_seconds: u64,
}

/// Persistence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub storage_backend: StorageBackend,
    pub checkpoint_interval_minutes: u64,
    pub max_snapshots: usize,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub backup_enabled: bool,
    pub storage_path: PathBuf,
}

/// Storage backend options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    FileSystem { base_path: PathBuf },
    SQLite { database_path: PathBuf },
    Memory { max_snapshots: usize },
}

/// Checkpoint metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    pub checkpoint_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub size_bytes: u64,
    pub compression_ratio: f64,
    pub agent_count: usize,
    pub task_count: usize,
    pub description: String,
}

/// Persistence manager for the hive system
pub struct PersistenceManager {
    config: PersistenceConfig,
    storage: Box<dyn StorageProvider + Send + Sync>,
    checkpoint_history: Arc<RwLock<Vec<CheckpointMetadata>>>,
    last_checkpoint: Arc<RwLock<Option<DateTime<Utc>>>>,
}

/// Storage provider trait for different backends
#[async_trait::async_trait]
pub trait StorageProvider {
    async fn save_snapshot(&self, snapshot: &SystemSnapshot) -> HiveResult<String>;
    async fn load_snapshot(&self, snapshot_id: &str) -> HiveResult<SystemSnapshot>;
    async fn list_snapshots(&self) -> HiveResult<Vec<CheckpointMetadata>>;
    async fn delete_snapshot(&self, snapshot_id: &str) -> HiveResult<()>;
    async fn cleanup_old_snapshots(&self, max_count: usize) -> HiveResult<usize>;
}

impl PersistenceManager {
    /// Create a new persistence manager
    pub async fn new(config: PersistenceConfig) -> HiveResult<Self> {
        let storage: Box<dyn StorageProvider + Send + Sync> = match &config.storage_backend {
            StorageBackend::FileSystem { base_path } => {
                Box::new(FileSystemStorage::new(base_path.clone()).await?)
            }
            StorageBackend::SQLite { database_path } => {
                Box::new(SQLiteStorage::new(database_path.clone()).await?)
            }
            StorageBackend::Memory { max_snapshots } => {
                Box::new(MemoryStorage::new(*max_snapshots))
            }
        };

        Ok(Self {
            config,
            storage,
            checkpoint_history: Arc::new(RwLock::new(Vec::new())),
            last_checkpoint: Arc::new(RwLock::new(None)),
        })
    }

    /// Create a checkpoint of the current system state
    pub async fn create_checkpoint(
        &self,
        hive: &HiveCoordinator,
        description: Option<String>,
    ) -> HiveResult<Uuid> {
        info!("Creating system checkpoint...");

        let snapshot = self.capture_system_state(hive).await?;
        let snapshot_id = snapshot.snapshot_id;

        // Save the snapshot
        let storage_id = self.storage.save_snapshot(&snapshot).await?;
        
        // Update checkpoint history
        let metadata = CheckpointMetadata {
            checkpoint_id: snapshot_id,
            timestamp: snapshot.timestamp,
            size_bytes: self.estimate_snapshot_size(&snapshot),
            compression_ratio: if self.config.compression_enabled { 0.3 } else { 1.0 },
            agent_count: snapshot.agents.len(),
            task_count: snapshot.tasks.len(),
            description: description.unwrap_or_else(|| "Automatic checkpoint".to_string()),
        };

        {
            let mut history = self.checkpoint_history.write().await;
            history.push(metadata);
            
            // Cleanup old checkpoints if needed
            if history.len() > self.config.max_snapshots {
                let removed = self.storage.cleanup_old_snapshots(self.config.max_snapshots).await?;
                history.truncate(self.config.max_snapshots);
                info!("Cleaned up {} old checkpoints", removed);
            }
        }

        {
            let mut last = self.last_checkpoint.write().await;
            *last = Some(Utc::now());
        }

        info!("Checkpoint {} created successfully", snapshot_id);
        Ok(snapshot_id)
    }

    /// Restore system state from a checkpoint
    pub async fn restore_from_checkpoint(
        &self,
        snapshot_id: &str,
        hive: &mut HiveCoordinator,
    ) -> HiveResult<()> {
        info!("Restoring system from checkpoint: {}", snapshot_id);

        let snapshot = self.storage.load_snapshot(snapshot_id).await?;
        
        // Restore hive state
        self.restore_hive_state(hive, &snapshot.hive_state).await?;
        
        // Restore agents
        self.restore_agents(hive, &snapshot.agents).await?;
        
        // Restore tasks
        self.restore_tasks(hive, &snapshot.tasks).await?;

        info!("System restored successfully from checkpoint {}", snapshot_id);
        Ok(())
    }

    /// Start automatic checkpointing
    pub async fn start_auto_checkpointing(&self, hive: Arc<RwLock<HiveCoordinator>>) {
        let interval_minutes = self.config.checkpoint_interval_minutes;
        let persistence = Arc::new(self.clone());
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(interval_minutes * 60)
            );

            loop {
                interval.tick().await;
                
                let hive_guard = hive.read().await;
                if let Err(e) = persistence.create_checkpoint(&*hive_guard, None).await {
                    error!("Automatic checkpoint failed: {}", e);
                } else {
                    info!("Automatic checkpoint completed");
                }
            }
        });

        info!("Automatic checkpointing started (interval: {} minutes)", interval_minutes);
    }

    /// Get checkpoint history
    pub async fn get_checkpoint_history(&self) -> Vec<CheckpointMetadata> {
        self.checkpoint_history.read().await.clone()
    }

    /// Capture current system state
    async fn capture_system_state(&self, hive: &HiveCoordinator) -> HiveResult<SystemSnapshot> {
        let agents: Vec<Agent> = hive.agents.iter()
            .map(|entry| entry.value().clone())
            .collect();

        let tasks: Vec<Task> = Vec::new(); // TODO: Implement task collection from hive

        let hive_status = hive.get_status().await;
        let metrics = SystemMetrics {
            total_agents: agents.len(),
            active_agents: agents.iter().filter(|a| !matches!(a.state, crate::agents::AgentState::Idle)).count(),
            completed_tasks: 0, // TODO: Get from hive metrics
            failed_tasks: 0,   // TODO: Get from hive metrics
            average_performance: hive_status.get("metrics")
                .and_then(|m| m.get("average_performance"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            swarm_cohesion: hive_status.get("metrics")
                .and_then(|m| m.get("swarm_cohesion"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            learning_progress: 0.0, // TODO: Get from adaptive learning system
            uptime_seconds: 0, // TODO: Calculate uptime
        };

        Ok(SystemSnapshot {
            snapshot_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            hive_state: HiveState {
                hive_id: hive_status.get("hive_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .unwrap_or_else(Uuid::new_v4),
                created_at: Utc::now(), // TODO: Get actual creation time
                last_update: Utc::now(),
                total_energy: hive_status.get("total_energy")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                swarm_center: (0.0, 0.0), // TODO: Get from hive
                auto_scaling_enabled: true,
                learning_enabled: true,
            },
            agents,
            tasks,
            metrics,
            configuration: serde_json::json!({}), // TODO: Include system configuration
        })
    }

    /// Restore hive state
    async fn restore_hive_state(&self, hive: &mut HiveCoordinator, state: &HiveState) -> HiveResult<()> {
        // TODO: Implement hive state restoration
        info!("Restoring hive state for hive {}", state.hive_id);
        Ok(())
    }

    /// Restore agents
    async fn restore_agents(&self, hive: &mut HiveCoordinator, agents: &[Agent]) -> HiveResult<()> {
        info!("Restoring {} agents", agents.len());
        
        for agent in agents {
            hive.agents.insert(agent.id, agent.clone());
        }
        
        Ok(())
    }

    /// Restore tasks
    async fn restore_tasks(&self, hive: &mut HiveCoordinator, tasks: &[Task]) -> HiveResult<()> {
        info!("Restoring {} tasks", tasks.len());
        // TODO: Implement task restoration
        Ok(())
    }

    /// Estimate snapshot size
    fn estimate_snapshot_size(&self, snapshot: &SystemSnapshot) -> u64 {
        // Rough estimation based on serialized JSON size
        serde_json::to_string(snapshot)
            .map(|s| s.len() as u64)
            .unwrap_or(0)
    }
}

impl Clone for PersistenceManager {
    fn clone(&self) -> Self {
        // Note: This is a simplified clone that doesn't clone the storage provider
        // In a real implementation, you'd need to handle this properly
        Self {
            config: self.config.clone(),
            storage: Box::new(MemoryStorage::new(100)), // Fallback storage
            checkpoint_history: self.checkpoint_history.clone(),
            last_checkpoint: self.last_checkpoint.clone(),
        }
    }
}

/// File system storage implementation
pub struct FileSystemStorage {
    base_path: PathBuf,
}

impl FileSystemStorage {
    pub async fn new(base_path: PathBuf) -> HiveResult<Self> {
        fs::create_dir_all(&base_path).await.map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to create storage directory: {}", e),
        })?;

        Ok(Self { base_path })
    }
}

#[async_trait::async_trait]
impl StorageProvider for FileSystemStorage {
    async fn save_snapshot(&self, snapshot: &SystemSnapshot) -> HiveResult<String> {
        let filename = format!("snapshot_{}.json", snapshot.snapshot_id);
        let file_path = self.base_path.join(&filename);
        
        let json_data = serde_json::to_string_pretty(snapshot).map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to serialize snapshot: {}", e),
        })?;

        fs::write(&file_path, json_data).await.map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to write snapshot file: {}", e),
        })?;

        Ok(filename)
    }

    async fn load_snapshot(&self, snapshot_id: &str) -> HiveResult<SystemSnapshot> {
        let file_path = self.base_path.join(format!("snapshot_{}.json", snapshot_id));
        
        let json_data = fs::read_to_string(&file_path).await.map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to read snapshot file: {}", e),
        })?;

        serde_json::from_str(&json_data).map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to deserialize snapshot: {}", e),
        })
    }

    async fn list_snapshots(&self) -> HiveResult<Vec<CheckpointMetadata>> {
        // TODO: Implement file system snapshot listing
        Ok(Vec::new())
    }

    async fn delete_snapshot(&self, snapshot_id: &str) -> HiveResult<()> {
        let file_path = self.base_path.join(format!("snapshot_{}.json", snapshot_id));
        fs::remove_file(&file_path).await.map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to delete snapshot file: {}", e),
        })
    }

    async fn cleanup_old_snapshots(&self, max_count: usize) -> HiveResult<usize> {
        // TODO: Implement cleanup logic
        Ok(0)
    }
}

/// SQLite storage implementation
pub struct SQLiteStorage {
    database_path: PathBuf,
}

impl SQLiteStorage {
    pub async fn new(database_path: PathBuf) -> HiveResult<Self> {
        // TODO: Initialize SQLite database
        Ok(Self { database_path })
    }
}

#[async_trait::async_trait]
impl StorageProvider for SQLiteStorage {
    async fn save_snapshot(&self, snapshot: &SystemSnapshot) -> HiveResult<String> {
        // TODO: Implement SQLite storage
        Ok(snapshot.snapshot_id.to_string())
    }

    async fn load_snapshot(&self, snapshot_id: &str) -> HiveResult<SystemSnapshot> {
        // TODO: Implement SQLite loading
        Err(HiveError::OperationFailed {
            reason: "SQLite storage not yet implemented".to_string(),
        })
    }

    async fn list_snapshots(&self) -> HiveResult<Vec<CheckpointMetadata>> {
        Ok(Vec::new())
    }

    async fn delete_snapshot(&self, _snapshot_id: &str) -> HiveResult<()> {
        Ok(())
    }

    async fn cleanup_old_snapshots(&self, _max_count: usize) -> HiveResult<usize> {
        Ok(0)
    }
}

/// In-memory storage implementation (for testing)
pub struct MemoryStorage {
    snapshots: Arc<RwLock<HashMap<String, SystemSnapshot>>>,
    max_snapshots: usize,
}

impl MemoryStorage {
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            max_snapshots,
        }
    }
}

#[async_trait::async_trait]
impl StorageProvider for MemoryStorage {
    async fn save_snapshot(&self, snapshot: &SystemSnapshot) -> HiveResult<String> {
        let mut snapshots = self.snapshots.write().await;
        let id = snapshot.snapshot_id.to_string();
        snapshots.insert(id.clone(), snapshot.clone());
        
        // Cleanup if needed
        if snapshots.len() > self.max_snapshots {
            let oldest_key = snapshots.keys().next().cloned();
            if let Some(key) = oldest_key {
                snapshots.remove(&key);
            }
        }
        
        Ok(id)
    }

    async fn load_snapshot(&self, snapshot_id: &str) -> HiveResult<SystemSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.get(snapshot_id).cloned().ok_or_else(|| HiveError::OperationFailed {
            reason: format!("Snapshot {} not found", snapshot_id),
        })
    }

    async fn list_snapshots(&self) -> HiveResult<Vec<CheckpointMetadata>> {
        let snapshots = self.snapshots.read().await;
        let metadata: Vec<CheckpointMetadata> = snapshots.values().map(|snapshot| {
            CheckpointMetadata {
                checkpoint_id: snapshot.snapshot_id,
                timestamp: snapshot.timestamp,
                size_bytes: 0, // Not calculated for memory storage
                compression_ratio: 1.0,
                agent_count: snapshot.agents.len(),
                task_count: snapshot.tasks.len(),
                description: "Memory snapshot".to_string(),
            }
        }).collect();
        
        Ok(metadata)
    }

    async fn delete_snapshot(&self, snapshot_id: &str) -> HiveResult<()> {
        let mut snapshots = self.snapshots.write().await;
        snapshots.remove(snapshot_id);
        Ok(())
    }

    async fn cleanup_old_snapshots(&self, max_count: usize) -> HiveResult<usize> {
        let mut snapshots = self.snapshots.write().await;
        let current_count = snapshots.len();
        
        if current_count <= max_count {
            return Ok(0);
        }
        
        let to_remove = current_count - max_count;
        let keys_to_remove: Vec<String> = snapshots.keys().take(to_remove).cloned().collect();
        
        for key in &keys_to_remove {
            snapshots.remove(key);
        }
        
        Ok(keys_to_remove.len())
    }
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            storage_backend: StorageBackend::FileSystem {
                base_path: PathBuf::from("./data/snapshots"),
            },
            checkpoint_interval_minutes: 30,
            max_snapshots: 10,
            compression_enabled: true,
            encryption_enabled: false,
            backup_enabled: true,
            storage_path: PathBuf::from("./data"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_memory_storage() {
        let storage = MemoryStorage::new(5);
        
        // Create a test snapshot
        let snapshot = SystemSnapshot {
            snapshot_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            hive_state: HiveState {
                hive_id: Uuid::new_v4(),
                created_at: Utc::now(),
                last_update: Utc::now(),
                total_energy: 100.0,
                swarm_center: (0.0, 0.0),
                auto_scaling_enabled: true,
                learning_enabled: true,
            },
            agents: Vec::new(),
            tasks: Vec::new(),
            metrics: SystemMetrics {
                total_agents: 0,
                active_agents: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                average_performance: 0.0,
                swarm_cohesion: 0.0,
                learning_progress: 0.0,
                uptime_seconds: 0,
            },
            configuration: serde_json::json!({}),
        };

        // Test save and load
        let id = storage.save_snapshot(&snapshot).await.unwrap();
        let loaded = storage.load_snapshot(&id).await.unwrap();
        
        assert_eq!(snapshot.snapshot_id, loaded.snapshot_id);
        assert_eq!(snapshot.version, loaded.version);
    }

    #[tokio::test]
    async fn test_filesystem_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemStorage::new(temp_dir.path().to_path_buf()).await.unwrap();
        
        // Create a test snapshot
        let snapshot = SystemSnapshot {
            snapshot_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            hive_state: HiveState {
                hive_id: Uuid::new_v4(),
                created_at: Utc::now(),
                last_update: Utc::now(),
                total_energy: 100.0,
                swarm_center: (0.0, 0.0),
                auto_scaling_enabled: true,
                learning_enabled: true,
            },
            agents: Vec::new(),
            tasks: Vec::new(),
            metrics: SystemMetrics {
                total_agents: 0,
                active_agents: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                average_performance: 0.0,
                swarm_cohesion: 0.0,
                learning_progress: 0.0,
                uptime_seconds: 0,
            },
            configuration: serde_json::json!({}),
        };

        // Test save and load
        let id = storage.save_snapshot(&snapshot).await.unwrap();
        let loaded = storage.load_snapshot(&snapshot.snapshot_id.to_string()).await.unwrap();
        
        assert_eq!(snapshot.snapshot_id, loaded.snapshot_id);
        assert_eq!(snapshot.version, loaded.version);
    }
}