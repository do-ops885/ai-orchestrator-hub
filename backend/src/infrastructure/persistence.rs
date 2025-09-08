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
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
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

/// Persistence manager for the hive system
#[allow(dead_code)]
pub struct PersistenceManager {
    config: PersistenceConfig,
    storage: Box<dyn StorageProvider + Send + Sync>,
    checkpoint_history: Arc<RwLock<Vec<CheckpointMetadata>>>,
    last_checkpoint: Arc<RwLock<Option<DateTime<Utc>>>>,
    encryption_key: Option<[u8; 32]>,
    backup_manager: Option<BackupManager>,
}

impl PersistenceManager {
    /// Load encryption key from environment or secure storage
    #[must_use]
    pub fn load_encryption_key() -> Option<String> {
        std::env::var("HIVE_ENCRYPTION_KEY").ok()
    }

    /// Create a new persistence manager
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

        Ok(Self {
            config,
            storage,
            checkpoint_history: Arc::new(RwLock::new(Vec::new())),
            last_checkpoint: Arc::new(RwLock::new(None)),
            encryption_key,
            backup_manager: None,
        })
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
        static EMPTY_PATH: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        EMPTY_PATH.get_or_init(PathBuf::new)
    }
}

/// `SQLite` storage implementation
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
        static EMPTY_PATH: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
        EMPTY_PATH.get_or_init(PathBuf::new)
    }
    #[allow(clippy::cast_possible_wrap)]
    async fn save_snapshot(&self, snapshot: &SystemSnapshot) -> HiveResult<String> {
        let json_data =
            serde_json::to_string(snapshot).map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to serialize snapshot: {e}"),
            })?;

        let conn = self.connection.lock().await;
        conn.execute(
            "INSERT INTO snapshots (id, timestamp, version, size_bytes, agent_count, task_count, description, data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                snapshot.snapshot_id.to_string(),
                snapshot.timestamp.to_rfc3339(),
                snapshot.version,
                json_data.len() as i64,
                snapshot.agents.len() as i64,
                snapshot.tasks.len() as i64,
                "Automated checkpoint",
                json_data
            ],
        ).map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to save snapshot to database: {e}"),
        })?;

        Ok(snapshot.snapshot_id.to_string())
    }

    async fn load_snapshot(&self, snapshot_id: &str) -> HiveResult<SystemSnapshot> {
        let conn = self.connection.lock().await;
        let mut stmt = conn
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
    }

    async fn list_snapshots(&self) -> HiveResult<Vec<CheckpointMetadata>> {
        let conn = self.connection.lock().await;
        let mut stmt = conn
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
    }

    async fn delete_snapshot(&self, snapshot_id: &str) -> HiveResult<()> {
        let conn = self.connection.lock().await;
        conn.execute("DELETE FROM snapshots WHERE id = ?1", params![snapshot_id])
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to delete snapshot: {e}"),
            })?;
        Ok(())
    }

    #[allow(clippy::cast_possible_wrap)]
    async fn cleanup_old_snapshots(&self, max_count: usize) -> HiveResult<usize> {
        let conn = self.connection.lock().await;

        // Get count of snapshots to delete
        let mut count_stmt = conn
            .prepare("SELECT COUNT(*) FROM snapshots")
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to prepare count query: {e}"),
            })?;

        let total_count: i64 =
            count_stmt
                .query_row([], |row| row.get(0))
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to get snapshot count: {e}"),
                })?;

        if total_count <= max_count as i64 {
            return Ok(0);
        }

        let to_delete = total_count - max_count as i64;

        // Delete oldest snapshots
        conn.execute(
            "DELETE FROM snapshots WHERE id IN (
                SELECT id FROM snapshots ORDER BY timestamp ASC LIMIT ?1
            )",
            params![to_delete],
        )
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to cleanup old snapshots: {e}"),
        })?;

        Ok(to_delete as usize)
    }
}

/// File system storage implementation
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
        static EMPTY_PATH: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
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
        let id = storage
            .save_snapshot(&snapshot)
            .await
            .expect("Failed to save snapshot");
        let loaded = storage
            .load_snapshot(&id)
            .await
            .expect("Failed to load snapshot");

        assert_eq!(snapshot.snapshot_id, loaded.snapshot_id);
        assert_eq!(snapshot.version, loaded.version);
    }

    #[tokio::test]
    async fn test_filesystem_storage() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage = FileSystemStorage::new(temp_dir.path().to_path_buf())
            .await
            .expect("Failed to create FileSystemStorage");

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
            .expect("Failed to save snapshot");
        let loaded = storage
            .load_snapshot(&snapshot.snapshot_id.to_string())
            .await
            .expect("Failed to load snapshot");

        assert_eq!(snapshot.snapshot_id, loaded.snapshot_id);
        assert_eq!(snapshot.version, loaded.version);
    }
}
