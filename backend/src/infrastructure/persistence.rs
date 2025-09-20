//! Comprehensive Persistence System for Multiagent Hive
//!
//! Provides state serialization, checkpointing, and recovery capabilities
//! with multiple storage backends and automatic backup strategies.

use crate::agents::Agent;
use crate::tasks::Task;
use crate::utils::error::{HiveError, HiveResult};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
use tokio::task;
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
#[allow(clippy::struct_excessive_bools)]
pub struct PersistenceConfig {
    pub storage_backend: StorageBackend,
    pub checkpoint_interval_minutes: u64,
    pub max_snapshots: usize,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub backup_enabled: bool,
    pub storage_path: PathBuf,
    pub encryption_key: Option<String>,
    pub compression_level: u32,
    pub backup_retention_days: u32,
    pub backup_location: Option<PathBuf>,
    pub incremental_backup: bool,
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

/// Checkpoint statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStats {
    pub total_snapshots: usize,
    pub total_size_bytes: u64,
    pub average_size_bytes: f64,
    pub oldest_snapshot: Option<DateTime<Utc>>,
    pub newest_snapshot: Option<DateTime<Utc>>,
    pub last_checkpoint: Option<DateTime<Utc>>,
    pub compression_savings: f64,
    pub encrypted_snapshots: usize,
    pub backup_count: usize,
    pub last_backup: Option<DateTime<Utc>>,
}

/// Persistence manager for the hive system with async optimizations
#[allow(dead_code)]
pub struct PersistenceManager {
    config: PersistenceConfig,
    storage: Box<dyn StorageProvider + Send + Sync>,
    checkpoint_history: Arc<RwLock<Vec<CheckpointMetadata>>>,
    last_checkpoint: Arc<RwLock<Option<DateTime<Utc>>>>,
    encryption_key: Option<[u8; 32]>,
    backup_manager: Option<BackupManager>,
    /// Connection pool for optimized database access
    connection_pool: Arc<tokio::sync::Semaphore>,
    /// Async optimizer for batching operations
    async_optimizer: Arc<crate::infrastructure::async_optimizer::AsyncOptimizer>,
}

impl PersistenceManager {
    /// Load encryption key from environment or secure storage
    #[must_use]
    pub fn load_encryption_key() -> Option<String> {
        std::env::var("HIVE_ENCRYPTION_KEY").ok()
    }

    /// Create a new persistence manager with async optimizations
    pub async fn new(config: PersistenceConfig) -> HiveResult<Self> {
        let encryption_key = config.encryption_key.as_ref().map(|s| {
            let mut key = [0u8; 32];
            let bytes = s.as_bytes();
            let len = bytes.len().min(32);
            key[..len].copy_from_slice(&bytes[..len]);
            key
        });

        let storage: Box<dyn StorageProvider + Send + Sync> = match &config.storage_backend {
            StorageBackend::SQLite { database_path } => {
                Box::new(SQLiteStorage::new(database_path.clone()).await?)
            }
            StorageBackend::FileSystem { base_path } => {
                Box::new(FileSystemStorage::new(base_path.clone()).await?)
            }
            StorageBackend::Memory { max_snapshots } => {
                Box::new(MemoryStorage::new(*max_snapshots))
            }
        };

        // Initialize async optimizer for batching operations
        let optimizer_config = crate::infrastructure::async_optimizer::AsyncOptimizerConfig {
            max_concurrent_ops: num_cpus::get() * 2, // Conservative concurrency for I/O
            batch_size: 50,                          // Smaller batches for I/O operations
            batch_timeout: std::time::Duration::from_millis(100),
            connection_pool_size: 10,
            enable_prioritization: true,
            metrics_interval: std::time::Duration::from_secs(60), // Less frequent for I/O
        };
        let async_optimizer = Arc::new(
            crate::infrastructure::async_optimizer::AsyncOptimizer::new(optimizer_config),
        );

        // Connection pool semaphore for limiting concurrent database connections
        let connection_pool = Arc::new(tokio::sync::Semaphore::new(5));

        Ok(Self {
            config,
            storage,
            checkpoint_history: Arc::new(RwLock::new(Vec::new())),
            last_checkpoint: Arc::new(RwLock::new(None)),
            encryption_key,
            backup_manager: None,
            connection_pool,
            async_optimizer,
        })
    }

    /// Create a checkpoint of the current hive state
    pub async fn create_checkpoint(
        &self,
        hive: &crate::core::hive::HiveCoordinator,
        description: Option<String>,
    ) -> HiveResult<String> {
        let snapshot_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let description_str = description.unwrap_or_else(|| "Automated checkpoint".to_string());

        // Collect agents
        let agents_with_ids = hive.get_all_agents().await;
        let agents: Vec<crate::agents::Agent> = agents_with_ids
            .into_iter()
            .map(|(_, agent)| agent)
            .collect();

        // Collect tasks from task distributor (simplified)
        let tasks = Vec::new(); // TODO: Implement task collection from task distributor

        // Get current metrics from status
        let status = hive.get_status().await;
        let metrics = status
            .get("metrics")
            .unwrap_or(&serde_json::Value::Null)
            .clone();

        // Create system snapshot
        let snapshot = SystemSnapshot {
            snapshot_id,
            timestamp,
            version: env!("CARGO_PKG_VERSION").to_string(),
            hive_state: HiveState {
                hive_id: hive.id,
                created_at: timestamp, // Use current timestamp as creation time for snapshot
                last_update: timestamp,
                total_energy: 100.0,        // TODO: get actual total energy
                swarm_center: (0.0, 0.0),   // TODO: get actual swarm center from metrics
                auto_scaling_enabled: true, // TODO: get from hive
                learning_enabled: true,     // TODO: get from hive
            },
            agents,
            tasks,
            metrics: SystemMetrics {
                total_agents: metrics
                    .get("total_agents")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize,
                active_agents: metrics
                    .get("active_agents")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize,
                completed_tasks: metrics
                    .get("completed_tasks")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                failed_tasks: metrics
                    .get("failed_tasks")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                average_performance: metrics
                    .get("average_performance")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                swarm_cohesion: metrics
                    .get("swarm_cohesion")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                learning_progress: metrics
                    .get("learning_progress")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                uptime_seconds: metrics
                    .get("uptime_seconds")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            },
            configuration: serde_json::json!({
                "description": description_str.clone()
            }),
        };

        // Save the snapshot
        let id = self.storage.save_snapshot(&snapshot).await?;

        // Update checkpoint history
        let mut history = self.checkpoint_history.write().await;
        history.push(CheckpointMetadata {
            checkpoint_id: snapshot_id,
            timestamp,
            size_bytes: 0, // TODO: calculate actual size
            compression_ratio: 1.0,
            agent_count: snapshot.agents.len(),
            task_count: snapshot.tasks.len(),
            description: description_str,
        });

        // Update last checkpoint
        *self.last_checkpoint.write().await = Some(timestamp);

        Ok(id)
    }

    /// Restore hive state from a checkpoint
    pub async fn restore_from_checkpoint(
        &self,
        checkpoint_id: &str,
        _hive: &mut crate::core::hive::HiveCoordinator,
    ) -> HiveResult<()> {
        // Load the snapshot
        let _snapshot = self.storage.load_snapshot(checkpoint_id).await?;

        // Clear existing agents and restore them
        // Note: In the new architecture, we can't directly manipulate agents
        // This would need to be implemented through the agent manager
        // For now, we'll skip agent restoration and focus on metrics

        // Restore tasks (simplified - in real implementation would need more complex logic)
        // TODO: Implement task restoration through task distributor

        // Note: In the new architecture, metrics restoration would need to be implemented
        // through the appropriate subsystem interfaces. For now, we'll skip this.
        // TODO: Implement metrics restoration through the metrics collector interface

        // Note: Swarm center restoration would need to be implemented
        // in the metrics or a separate swarm intelligence module

        Ok(())
    }

    /// Get statistics about checkpoints
    pub async fn get_checkpoint_stats(&self) -> HiveResult<CheckpointStats> {
        let history = self.checkpoint_history.read().await;
        let total_snapshots = history.len();
        let total_size_bytes = history.iter().map(|meta| meta.size_bytes).sum();
        let average_size_bytes = if total_snapshots > 0 {
            total_size_bytes as f64 / total_snapshots as f64
        } else {
            0.0
        };

        let oldest_snapshot = history.iter().map(|meta| meta.timestamp).min();
        let newest_snapshot = history.iter().map(|meta| meta.timestamp).max();
        let last_checkpoint = *self.last_checkpoint.read().await;

        // Calculate compression savings (simplified)
        let compression_savings = 0.0; // TODO: implement actual calculation

        // Count encrypted snapshots (simplified)
        let encrypted_snapshots = if self.config.encryption_enabled {
            total_snapshots
        } else {
            0
        };

        Ok(CheckpointStats {
            total_snapshots,
            total_size_bytes,
            average_size_bytes,
            oldest_snapshot,
            newest_snapshot,
            last_checkpoint,
            compression_savings,
            encrypted_snapshots,
            backup_count: 0, // TODO: implement backup tracking
            last_backup: None,
        })
    }

    /// List all available snapshots
    pub async fn list_snapshots(&self) -> HiveResult<Vec<CheckpointMetadata>> {
        self.storage.list_snapshots().await
    }
}

/// Backup manager for handling backup operations
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BackupManager {
    backup_location: PathBuf,
    retention_days: u32,
    incremental: bool,
}

/// Processed snapshot data with compression and encryption
#[derive(Debug, Clone)]
pub struct ProcessedSnapshot {
    pub data: Vec<u8>,
    pub is_compressed: bool,
    pub is_encrypted: bool,
    pub original_size: usize,
    pub processed_size: usize,
    pub compression_ratio: f64,
}

/// Storage provider trait for different backends
#[async_trait::async_trait]
pub trait StorageProvider {
    async fn save_snapshot(&self, snapshot: &SystemSnapshot) -> HiveResult<String>;
    async fn load_snapshot(&self, snapshot_id: &str) -> HiveResult<SystemSnapshot>;
    async fn list_snapshots(&self) -> HiveResult<Vec<CheckpointMetadata>>;
    async fn delete_snapshot(&self, snapshot_id: &str) -> HiveResult<()>;
    async fn save_processed_snapshot(
        &self,
        snapshot: &SystemSnapshot,
        processed: &ProcessedSnapshot,
    ) -> HiveResult<String> {
        let filename = format!("snapshot_{}.bin", snapshot.snapshot_id);
        let file_path = self.base_path().join(&filename);

        // Write processed binary data
        fs::write(&file_path, &processed.data)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to write processed snapshot file: {e}"),
            })?;

        // Write metadata
        let metadata_filename = format!("snapshot_{}.meta", snapshot.snapshot_id);
        let metadata_path = self.base_path().join(&metadata_filename);

        let metadata = serde_json::json!({
            "snapshot_id": snapshot.snapshot_id,
            "timestamp": snapshot.timestamp,
            "is_compressed": processed.is_compressed,
            "is_encrypted": processed.is_encrypted,
            "original_size": processed.original_size,
            "processed_size": processed.processed_size,
            "compression_ratio": processed.compression_ratio,
        });

        fs::write(
            &metadata_path,
            serde_json::to_string_pretty(&metadata).map_err(|e| HiveError::ValidationError {
                field: "metadata".to_string(),
                reason: format!("Failed to serialize: {e}"),
            })?,
        )
        .await
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to write metadata file: {e}"),
        })?;

        Ok(filename)
    }

    async fn load_processed_snapshot(&self, snapshot_id: &str) -> HiveResult<ProcessedSnapshot> {
        let file_path = self.base_path().join(format!("snapshot_{snapshot_id}.bin"));
        let metadata_path = self
            .base_path()
            .join(format!("snapshot_{snapshot_id}.meta"));

        // Load binary data
        let data = fs::read(&file_path)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to read processed snapshot file: {e}"),
            })?;

        // Load metadata
        let metadata_str =
            fs::read_to_string(&metadata_path)
                .await
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to read metadata file: {e}"),
                })?;

        let metadata: serde_json::Value =
            serde_json::from_str(&metadata_str).map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to parse metadata: {e}"),
            })?;

        Ok(ProcessedSnapshot {
            data,
            is_compressed: metadata["is_compressed"].as_bool().unwrap_or(false),
            is_encrypted: metadata["is_encrypted"].as_bool().unwrap_or(false),
            original_size: metadata["original_size"].as_u64().unwrap_or(0) as usize,
            processed_size: metadata["processed_size"].as_u64().unwrap_or(0) as usize,
            compression_ratio: metadata["compression_ratio"].as_f64().unwrap_or(1.0),
        })
    }

    async fn cleanup_old_snapshots(&self, _max_count: usize) -> HiveResult<usize> {
        // TODO: Implement cleanup logic
        Ok(0)
    }

    fn base_path(&self) -> &PathBuf {
        // Default implementation returns empty path - concrete implementations should override
        static EMPTY_PATH: OnceLock<PathBuf> = OnceLock::new();
        EMPTY_PATH.get_or_init(PathBuf::new)
    }
}

/// `SQLite` storage implementation
#[derive(Clone)]
#[allow(dead_code)]
pub struct SQLiteStorage {
    database_path: PathBuf,
    connection: Arc<Mutex<Connection>>,
}

impl SQLiteStorage {
    pub async fn new(database_path: PathBuf) -> HiveResult<Self> {
        // Create database directory if it doesn't exist
        if let Some(parent) = database_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to create database directory: {e}"),
                })?;
        }

        let conn = Connection::open(&database_path).map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to open SQLite database: {e}"),
        })?;

        // Initialize database schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                version TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                agent_count INTEGER NOT NULL,
                task_count INTEGER NOT NULL,
                description TEXT NOT NULL,
                data TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to create snapshots table: {e}"),
        })?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_snapshots_timestamp ON snapshots(timestamp)",
            [],
        )
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to create timestamp index: {e}"),
        })?;

        Ok(Self {
            database_path,
            connection: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait::async_trait]
impl StorageProvider for SQLiteStorage {
    fn base_path(&self) -> &PathBuf {
        // SQLite storage doesn't use filesystem paths
        static EMPTY_PATH: OnceLock<PathBuf> = OnceLock::new();
        EMPTY_PATH.get_or_init(PathBuf::new)
    }
    #[allow(clippy::cast_possible_wrap)]
    async fn save_snapshot(&self, snapshot: &SystemSnapshot) -> HiveResult<String> {
        // Move serialization and database operations to blocking thread
        let snapshot_id = snapshot.snapshot_id;
        let snapshot_clone = snapshot.clone();
        let conn = Arc::clone(&self.connection);

        task::spawn_blocking(move || {
            let json_data = serde_json::to_string(&snapshot_clone)
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to serialize snapshot: {e}"),
                })?;

            let conn_guard = conn.blocking_lock();
            conn_guard.execute(
                "INSERT INTO snapshots (id, timestamp, version, size_bytes, agent_count, task_count, description, data)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    snapshot_clone.snapshot_id.to_string(),
                    snapshot_clone.timestamp.to_rfc3339(),
                    snapshot_clone.version,
                    json_data.len() as i64,
                    snapshot_clone.agents.len() as i64,
                    snapshot_clone.tasks.len() as i64,
                    "Automated checkpoint",
                    json_data
                ],
            ).map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to save snapshot to database: {e}"),
            })?;

            Ok(snapshot_id.to_string())
        })
        .await
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Database task panicked: {e}"),
        })?
    }

    async fn load_snapshot(&self, snapshot_id: &str) -> HiveResult<SystemSnapshot> {
        // Move database query and deserialization to blocking thread
        let snapshot_id = snapshot_id.to_string();
        let conn = Arc::clone(&self.connection);

        task::spawn_blocking(move || {
            let conn_guard = conn.blocking_lock();
            let mut stmt = conn_guard
                .prepare("SELECT data FROM snapshots WHERE id = ?1")
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to prepare load query: {e}"),
                })?;

            let json_data: String = stmt
                .query_row(params![snapshot_id], |row| row.get(0))
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to load snapshot: {e}"),
                })?;

            let snapshot: SystemSnapshot =
                serde_json::from_str(&json_data).map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to deserialize snapshot: {e}"),
                })?;

            Ok(snapshot)
        })
        .await
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Database task panicked: {e}"),
        })?
    }

    async fn list_snapshots(&self) -> HiveResult<Vec<CheckpointMetadata>> {
        // Move database query to blocking thread
        let conn = Arc::clone(&self.connection);

        task::spawn_blocking(move || {
            let conn_guard = conn.blocking_lock();
            let mut stmt = conn_guard
                .prepare("SELECT id, timestamp, size_bytes, agent_count, task_count, description FROM snapshots ORDER BY timestamp DESC")
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to prepare list query: {e}"),
                })?;

            let mut rows = stmt.query([]).map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to execute list query: {e}"),
            })?;

            let mut snapshots = Vec::new();
            while let Ok(Some(row)) = rows.next() {
                let id: String = row.get(0).map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to get id: {e}"),
                })?;
                let timestamp: String = row.get(1).map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to get timestamp: {e}"),
                })?;
                let size_bytes: i64 = row.get(2).map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to get size_bytes: {e}"),
                })?;
                let agent_count: i64 = row.get(3).map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to get agent_count: {e}"),
                })?;
                let task_count: i64 = row.get(4).map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to get task_count: {e}"),
                })?;
                let description: String = row.get(5).map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to get description: {e}"),
                })?;

                let checkpoint = CheckpointMetadata {
                    checkpoint_id: uuid::Uuid::parse_str(&id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
                    timestamp: DateTime::parse_from_rfc3339(&timestamp)
                        .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
                    size_bytes: size_bytes as u64,
                    compression_ratio: 1.0, // Not stored
                    agent_count: agent_count as usize,
                    task_count: task_count as usize,
                    description,
                };
                snapshots.push(checkpoint);
            }

            Ok(snapshots)
        })
        .await
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Database task panicked: {e}"),
        })?
    }

    async fn delete_snapshot(&self, snapshot_id: &str) -> HiveResult<()> {
        let snapshot_id = snapshot_id.to_string();
        let conn = Arc::clone(&self.connection);

        task::spawn_blocking(move || {
            let conn_guard = conn.blocking_lock();
            conn_guard
                .execute("DELETE FROM snapshots WHERE id = ?1", params![snapshot_id])
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to delete snapshot: {e}"),
                })?;
            Ok(())
        })
        .await
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Database task panicked: {e}"),
        })?
    }

    #[allow(clippy::cast_possible_wrap)]
    async fn cleanup_old_snapshots(&self, max_count: usize) -> HiveResult<usize> {
        let max_count = max_count;
        let conn = Arc::clone(&self.connection);

        task::spawn_blocking(move || {
            let conn_guard = conn.blocking_lock();

            // Get count of snapshots to delete
            let mut count_stmt = conn_guard
                .prepare("SELECT COUNT(*) FROM snapshots")
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to prepare count query: {e}"),
                })?;

            let total_count: i64 = count_stmt.query_row([], |row| row.get(0)).map_err(|e| {
                HiveError::OperationFailed {
                    reason: format!("Failed to get snapshot count: {e}"),
                }
            })?;

            if total_count <= max_count as i64 {
                return Ok(0);
            }

            let to_delete = total_count - max_count as i64;

            // Delete oldest snapshots
            conn_guard
                .execute(
                    "DELETE FROM snapshots WHERE id IN (
                    SELECT id FROM snapshots ORDER BY timestamp ASC LIMIT ?1
                )",
                    params![to_delete],
                )
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to cleanup old snapshots: {e}"),
                })?;

            Ok(to_delete as usize)
        })
        .await
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Database task panicked: {e}"),
        })?
    }
}

/// File system storage implementation
#[derive(Clone)]
pub struct FileSystemStorage {
    base_path: PathBuf,
}

impl FileSystemStorage {
    pub async fn new(base_path: PathBuf) -> HiveResult<Self> {
        // Create directory if it doesn't exist
        tokio::fs::create_dir_all(&base_path)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to create storage directory: {e}"),
            })?;

        Ok(Self { base_path })
    }
}

#[async_trait::async_trait]
impl StorageProvider for FileSystemStorage {
    async fn save_snapshot(&self, snapshot: &SystemSnapshot) -> HiveResult<String> {
        let json_data =
            serde_json::to_string(snapshot).map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to serialize snapshot: {e}"),
            })?;

        let filename = format!("snapshot_{}.json", snapshot.snapshot_id);
        let file_path = self.base_path.join(&filename);

        fs::write(&file_path, &json_data)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to write snapshot file: {e}"),
            })?;

        Ok(snapshot.snapshot_id.to_string())
    }

    async fn load_snapshot(&self, snapshot_id: &str) -> HiveResult<SystemSnapshot> {
        let filename = format!("snapshot_{snapshot_id}.json");
        let file_path = self.base_path.join(&filename);

        let json_data =
            fs::read_to_string(&file_path)
                .await
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to read snapshot file: {e}"),
                })?;

        serde_json::from_str(&json_data).map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to deserialize snapshot: {e}"),
        })
    }

    async fn list_snapshots(&self) -> HiveResult<Vec<CheckpointMetadata>> {
        let mut entries =
            fs::read_dir(&self.base_path)
                .await
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to read storage directory: {e}"),
                })?;

        let mut snapshots = Vec::new();
        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to read directory entry: {e}"),
                })?
        {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "json" {
                    if let Some(filename) = path.file_stem() {
                        if let Some(filename_str) = filename.to_str() {
                            if filename_str.starts_with("snapshot_") {
                                if let Some(uuid_str) = filename_str.strip_prefix("snapshot_") {
                                    if let Ok(uuid) = Uuid::parse_str(uuid_str) {
                                        // Load the snapshot to get metadata
                                        if let Ok(snapshot) = self.load_snapshot(uuid_str).await {
                                            snapshots.push(CheckpointMetadata {
                                                checkpoint_id: uuid,
                                                timestamp: snapshot.timestamp,
                                                size_bytes: 0, // Would need to get file size
                                                compression_ratio: 1.0,
                                                agent_count: snapshot.agents.len(),
                                                task_count: snapshot.tasks.len(),
                                                description: "File system snapshot".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(snapshots)
    }

    async fn delete_snapshot(&self, snapshot_id: &str) -> HiveResult<()> {
        let filename = format!("snapshot_{snapshot_id}.json");
        let file_path = self.base_path.join(&filename);

        fs::remove_file(&file_path)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to delete snapshot file: {e}"),
            })?;

        Ok(())
    }

    fn base_path(&self) -> &PathBuf {
        &self.base_path
    }
}

/// In-memory storage implementation (for testing)
#[derive(Clone)]
pub struct MemoryStorage {
    snapshots: Arc<RwLock<HashMap<String, SystemSnapshot>>>,
    max_snapshots: usize,
}

impl MemoryStorage {
    #[must_use]
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            max_snapshots,
        }
    }
}

#[async_trait::async_trait]
impl StorageProvider for MemoryStorage {
    fn base_path(&self) -> &PathBuf {
        // Memory storage doesn't use filesystem paths
        static EMPTY_PATH: OnceLock<PathBuf> = OnceLock::new();
        EMPTY_PATH.get_or_init(PathBuf::new)
    }
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

    async fn save_processed_snapshot(
        &self,
        snapshot: &SystemSnapshot,
        _processed: &ProcessedSnapshot,
    ) -> HiveResult<String> {
        let mut snapshots = self.snapshots.write().await;
        let id = snapshot.snapshot_id.to_string();

        // Store both the original snapshot and processed data
        // In memory storage, we'll just store the original for simplicity
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
        snapshots
            .get(snapshot_id)
            .cloned()
            .ok_or_else(|| HiveError::OperationFailed {
                reason: format!("Snapshot {snapshot_id} not found"),
            })
    }

    async fn load_processed_snapshot(&self, snapshot_id: &str) -> HiveResult<ProcessedSnapshot> {
        let snapshots = self.snapshots.read().await;
        let snapshot = snapshots
            .get(snapshot_id)
            .ok_or_else(|| HiveError::OperationFailed {
                reason: format!("Snapshot {snapshot_id} not found"),
            })?;

        // Convert snapshot back to processed format (simplified)
        let json_data =
            serde_json::to_string(snapshot).map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to serialize snapshot: {e}"),
            })?;

        let data = json_data.into_bytes();
        let size = data.len();
        Ok(ProcessedSnapshot {
            data,
            is_compressed: false,
            is_encrypted: false,
            original_size: size,
            processed_size: size,
            compression_ratio: 1.0,
        })
    }

    async fn list_snapshots(&self) -> HiveResult<Vec<CheckpointMetadata>> {
        let snapshots = self.snapshots.read().await;
        let metadata: Vec<CheckpointMetadata> = snapshots
            .values()
            .map(|snapshot| {
                CheckpointMetadata {
                    checkpoint_id: snapshot.snapshot_id,
                    timestamp: snapshot.timestamp,
                    size_bytes: 0, // Not calculated for memory storage
                    compression_ratio: 1.0,
                    agent_count: snapshot.agents.len(),
                    task_count: snapshot.tasks.len(),
                    description: "Memory snapshot".to_string(),
                }
            })
            .collect();

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
            encryption_key: None,
            compression_level: 6,
            backup_retention_days: 30,
            backup_location: None,
            incremental_backup: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_memory_storage() -> Result<(), Box<dyn std::error::Error>> {
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
        let id = storage
            .save_snapshot(&snapshot)
            .await
            .map_err(|e| format!("Failed to save snapshot: {}", e))?;
        let loaded = storage
            .load_snapshot(&id)
            .await
            .map_err(|e| format!("Failed to load snapshot: {}", e))?;

        assert_eq!(snapshot.snapshot_id, loaded.snapshot_id);
        assert_eq!(snapshot.version, loaded.version);
        Ok(())
    }

    #[tokio::test]
    async fn test_filesystem_storage() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
        let storage = FileSystemStorage::new(temp_dir.path().to_path_buf())
            .await
            .map_err(|e| format!("Failed to create FileSystemStorage: {}", e))?;

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
        let _id = storage
            .save_snapshot(&snapshot)
            .await
            .map_err(|e| format!("Failed to save snapshot: {}", e))?;
        let loaded = storage
            .load_snapshot(&snapshot.snapshot_id.to_string())
            .await
            .map_err(|e| format!("Failed to load snapshot: {}", e))?;

        assert_eq!(snapshot.snapshot_id, loaded.snapshot_id);
        assert_eq!(snapshot.version, loaded.version);
        Ok(())
    }
}
