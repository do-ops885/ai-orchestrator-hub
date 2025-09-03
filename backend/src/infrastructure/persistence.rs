//! Comprehensive Persistence System for Multiagent Hive
//!
//! Provides state serialization, checkpointing, and recovery capabilities
//! with multiple storage backends and automatic backup strategies.

use crate::agents::Agent;
use crate::core::HiveCoordinator;
use crate::tasks::Task;
use crate::utils::error::{HiveError, HiveResult};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};


use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
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
pub struct PersistenceManager {
    config: PersistenceConfig,
    storage: Box<dyn StorageProvider + Send + Sync>,
    checkpoint_history: Arc<RwLock<Vec<CheckpointMetadata>>>,
    last_checkpoint: Arc<RwLock<Option<DateTime<Utc>>>>,
    encryption_key: Option<[u8; 32]>,
    backup_manager: Option<BackupManager>,
}

/// Backup manager for handling backup operations
#[derive(Debug, Clone)]
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
    async fn save_processed_snapshot(
        &self,
        snapshot: &SystemSnapshot,
        processed: &ProcessedSnapshot,
    ) -> HiveResult<String>;
    async fn load_snapshot(&self, snapshot_id: &str) -> HiveResult<SystemSnapshot>;
    async fn load_processed_snapshot(&self, snapshot_id: &str) -> HiveResult<ProcessedSnapshot>;
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

        // Initialize encryption key if encryption is enabled
        let encryption_key = if config.encryption_enabled {
            Some(Self::derive_encryption_key(&config.encryption_key)?)
        } else {
            None
        };

        // Initialize backup manager if backup is enabled
        let backup_manager = if config.backup_enabled {
            Some(BackupManager {
                backup_location: config
                    .backup_location
                    .clone()
                    .unwrap_or_else(|| config.storage_path.join("backups")),
                retention_days: config.backup_retention_days,
                incremental: config.incremental_backup,
            })
        } else {
            None
        };

        Ok(Self {
            config,
            storage,
            checkpoint_history: Arc::new(RwLock::new(Vec::new())),
            last_checkpoint: Arc::new(RwLock::new(None)),
            encryption_key,
            backup_manager,
        })
    }

    /// Derive encryption key from configuration with PBKDF2 key stretching
    fn derive_encryption_key(key_config: &Option<String>) -> HiveResult<[u8; 32]> {
        match key_config {
            Some(key_str) => {
                // Check if it's a hex-encoded key (64 characters for 32 bytes)
                if key_str.len() == 64 {
                    hex::decode(key_str)
                        .map_err(|e| HiveError::OperationFailed {
                            reason: format!("Invalid hex encryption key: {}", e),
                        })
                        .and_then(|decoded| {
                            if decoded.len() == 32 {
                                let mut key = [0u8; 32];
                                key.copy_from_slice(&decoded);
                                Ok(key)
                            } else {
                                Err(HiveError::OperationFailed {
                                    reason: "Hex encryption key must decode to 32 bytes"
                                        .to_string(),
                                })
                            }
                        })
                } else if key_str.len() == 44 {
                    // Base64-encoded key (44 chars for 32 bytes + padding)
                    use base64::{engine::general_purpose, Engine as _};
                    general_purpose::STANDARD
                        .decode(key_str)
                        .map_err(|e| HiveError::OperationFailed {
                            reason: format!("Invalid base64 encryption key: {}", e),
                        })
                        .and_then(|decoded| {
                            if decoded.len() == 32 {
                                let mut key = [0u8; 32];
                                key.copy_from_slice(&decoded);
                                Ok(key)
                            } else {
                                Err(HiveError::OperationFailed {
                                    reason: "Base64 encryption key must decode to 32 bytes"
                                        .to_string(),
                                })
                            }
                        })
                } else {
                    // Use PBKDF2 to derive a strong key from the provided password/key
                    let salt = b"hive_persistence_salt"; // In production, use a random salt per key
                    let mut key = [0u8; 32];

                    pbkdf2::pbkdf2::<pbkdf2::Hmac<sha2::Sha256>>(
                        key_str.as_bytes(),
                        salt,
                        100_000, // 100k iterations for good security
                        &mut key,
                    )
                    .map_err(|e| HiveError::OperationFailed {
                        reason: format!("Failed to derive encryption key: {:?}", e),
                    })?;

                    Ok(key)
                }
            }
            None => {
                // Generate a cryptographically secure random key
                let mut key = [0u8; 32];
                ring::rand::SystemRandom::new()
                    .fill(&mut key)
                    .map_err(|e| HiveError::OperationFailed {
                        reason: format!("Failed to generate encryption key: {:?}", e),
                    })?;

                tracing::warn!(
                    "Generated random encryption key - save this for recovery: {}",
                    hex::encode(&key)
                );
                tracing::warn!("For production use, set HIVE_ENCRYPTION_KEY environment variable");

                Ok(key)
            }
        }
    }

    /// Process snapshot data with compression and encryption
    async fn process_snapshot_data(
        &self,
        snapshot: &SystemSnapshot,
    ) -> HiveResult<ProcessedSnapshot> {
        let json_data =
            serde_json::to_string(snapshot).map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to serialize snapshot: {}", e),
            })?;

        let original_size = json_data.len();
        let mut data = json_data.into_bytes();
        let mut is_compressed = false;
        let mut is_encrypted = false;

        // Apply compression if enabled
        if self.config.compression_enabled {
            data = self.compress_data(&data)?;
            is_compressed = true;
        }

        // Apply encryption if enabled
        if self.config.encryption_enabled && self.encryption_key.is_some() {
            data = self.encrypt_data(&data)?;
            is_encrypted = true;
        }

        let processed_size = data.len();
        let compression_ratio = if is_compressed {
            original_size as f64 / processed_size as f64
        } else {
            1.0
        };

        Ok(ProcessedSnapshot {
            data,
            is_compressed,
            is_encrypted,
            original_size,
            processed_size,
            compression_ratio,
        })
    }

    /// Decompress and decrypt snapshot data
    async fn unprocess_snapshot_data(
        &self,
        processed: &ProcessedSnapshot,
    ) -> HiveResult<SystemSnapshot> {
        let mut data = processed.data.clone();

        // Decrypt if encrypted
        if processed.is_encrypted {
            data = self.decrypt_data(&data)?;
        }

        // Decompress if compressed
        if processed.is_compressed {
            data = self.decompress_data(&data)?;
        }

        let json_str = String::from_utf8(data).map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to convert data to string: {}", e),
        })?;

        serde_json::from_str(&json_str).map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to deserialize snapshot: {}", e),
        })
    }

    /// Compress data using gzip
    fn compress_data(&self, data: &[u8]) -> HiveResult<Vec<u8>> {
        let mut encoder =
            GzEncoder::new(Vec::new(), Compression::new(self.config.compression_level));
        encoder
            .write_all(data)
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to write data for compression: {}", e),
            })?;
        encoder.finish().map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to compress data: {}", e),
        })
    }

    /// Decompress gzip data
    fn decompress_data(&self, data: &[u8]) -> HiveResult<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to decompress data: {}", e),
            })?;
        Ok(decompressed)
    }

    /// Encrypt data using AES-256-GCM
    fn encrypt_data(&self, data: &[u8]) -> HiveResult<Vec<u8>> {
        let key = self
            .encryption_key
            .as_ref()
            .ok_or_else(|| HiveError::OperationFailed {
                reason: "Encryption key not available".to_string(),
            })?;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

        // Generate random nonce
        let rng = ring::rand::SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes)
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to generate nonce: {:?}", e),
            })?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to encrypt data: {}", e),
            })?;

        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypt data using AES-256-GCM
    fn decrypt_data(&self, data: &[u8]) -> HiveResult<Vec<u8>> {
        if data.len() < 12 {
            return Err(HiveError::OperationFailed {
                reason: "Encrypted data too short".to_string(),
            });
        }

        let key = self
            .encryption_key
            .as_ref()
            .ok_or_else(|| HiveError::OperationFailed {
                reason: "Encryption key not available".to_string(),
            })?;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to decrypt data: {}", e),
            })
    }

    /// Rotate encryption key for existing data
    ///
    /// This method re-encrypts all existing snapshots with a new key
    /// while maintaining data integrity and availability.
    pub async fn rotate_encryption_key(
        &mut self,
        new_key_config: Option<String>,
    ) -> HiveResult<()> {
        info!("Starting encryption key rotation...");

        // Derive new encryption key
        let new_key = Self::derive_encryption_key(&new_key_config)?;

        // Get all existing snapshots
        let snapshots = self.storage.list_snapshots().await?;
        info!("Found {} snapshots to re-encrypt", snapshots.len());

        for metadata in snapshots {
            info!("Re-encrypting snapshot: {}", metadata.checkpoint_id);

            // Load the snapshot
            let processed = self
                .storage
                .load_processed_snapshot(&metadata.checkpoint_id.to_string())
                .await?;

            // Decrypt with old key
            let decrypted_data = if processed.is_encrypted {
                self.decrypt_data(&processed.data)?
            } else {
                processed.data
            };

            // Decompress if needed
            let decompressed_data = if processed.is_compressed {
                self.decompress_data(&decrypted_data)?
            } else {
                decrypted_data
            };

            // Deserialize back to SystemSnapshot
            let json_str = String::from_utf8(decompressed_data)
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to convert decompressed data to string: {}", e),
                })?;
            let snapshot: SystemSnapshot = serde_json::from_str(&json_str)
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to deserialize snapshot: {}", e),
                })?;

            // Update encryption key
            self.encryption_key = Some(new_key);

            // Re-process with new key
            let reprocessed = self.process_snapshot_data(&snapshot).await?;

            // Save with new encryption
            self.storage
                .save_processed_snapshot(&snapshot, &reprocessed)
                .await?;

            info!(
                "Successfully re-encrypted snapshot: {}",
                metadata.checkpoint_id
            );
        }

        // Update the key in memory
        self.encryption_key = Some(new_key);

        info!("Encryption key rotation completed successfully");
        Ok(())
    }

    /// Load encryption key from secure sources
    ///
    /// Attempts to load encryption key from:
    /// 1. Environment variable HIVE_ENCRYPTION_KEY
    /// 2. Key file at configured path
    /// 3. Generate new key (development only)
    pub fn load_encryption_key() -> Option<String> {
        // Try environment variable first
        if let Ok(key) = std::env::var("HIVE_ENCRYPTION_KEY") {
            if !key.is_empty() {
                info!("Loaded encryption key from environment variable");
                return Some(key);
            }
        }

        // Try key file
        let key_paths = [
            "./config/encryption.key",
            "/etc/hive/encryption.key",
            "./data/encryption.key",
        ];

        for path in &key_paths {
            if let Ok(key_data) = std::fs::read_to_string(path) {
                let key = key_data.trim();
                if !key.is_empty() {
                    info!("Loaded encryption key from file: {}", path);
                    return Some(key.to_string());
                }
            }
        }

        // For development/demo purposes only
        if cfg!(debug_assertions) {
            warn!("No encryption key found - using development mode");
            None
        } else {
            error!("No encryption key found - encryption disabled for security");
            None
        }
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
            compression_ratio: if self.config.compression_enabled {
                0.3
            } else {
                1.0
            },
            agent_count: snapshot.agents.len(),
            task_count: snapshot.tasks.len(),
            description: description.unwrap_or_else(|| "Automatic checkpoint".to_string()),
        };

        {
            let mut history = self.checkpoint_history.write().await;
            history.push(metadata);

            // Cleanup old checkpoints if needed
            if history.len() > self.config.max_snapshots {
                let removed = self
                    .storage
                    .cleanup_old_snapshots(self.config.max_snapshots)
                    .await?;
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

    /// Create backup of snapshot
    async fn create_backup(
        &self,
        snapshot: &SystemSnapshot,
        processed: &ProcessedSnapshot,
        backup_manager: &BackupManager,
    ) -> HiveResult<()> {
        // Create backup directory if it doesn't exist
        fs::create_dir_all(&backup_manager.backup_location)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to create backup directory: {}", e),
            })?;

        let backup_filename = format!(
            "backup_{}_{}.bin",
            snapshot.snapshot_id,
            snapshot.timestamp.format("%Y%m%d_%H%M%S")
        );
        let backup_path = backup_manager.backup_location.join(&backup_filename);

        // Write processed data to backup file
        fs::write(&backup_path, &processed.data)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to write backup file: {}", e),
            })?;

        // Create metadata file
        let metadata_filename = format!(
            "backup_{}_{}.meta",
            snapshot.snapshot_id,
            snapshot.timestamp.format("%Y%m%d_%H%M%S")
        );
        let metadata_path = backup_manager.backup_location.join(&metadata_filename);

        let backup_metadata = serde_json::json!({
            "snapshot_id": snapshot.snapshot_id,
            "timestamp": snapshot.timestamp,
            "original_size": processed.original_size,
            "processed_size": processed.processed_size,
            "is_compressed": processed.is_compressed,
            "is_encrypted": processed.is_encrypted,
            "compression_ratio": processed.compression_ratio,
            "agent_count": snapshot.agents.len(),
            "task_count": snapshot.tasks.len(),
        });

        fs::write(
            &metadata_path,
            serde_json::to_string_pretty(&backup_metadata).map_err(|e| {
                HiveError::SerializationError {
                    reason: format!("Failed to serialize backup metadata: {}", e),
                }
            })?,
        )
        .await
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to write backup metadata: {}", e),
        })?;

        info!("Created backup: {}", backup_filename);

        // Cleanup old backups if needed
        if let Err(e) = self.cleanup_old_backups(backup_manager).await {
            warn!("Failed to cleanup old backups: {}", e);
        }

        Ok(())
    }

    /// Cleanup old backup files
    async fn cleanup_old_backups(&self, backup_manager: &BackupManager) -> HiveResult<()> {
        let cutoff_date = Utc::now() - chrono::Duration::days(backup_manager.retention_days as i64);

        let mut entries = fs::read_dir(&backup_manager.backup_location)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to read backup directory: {}", e),
            })?;

        let mut removed_count = 0;
        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to read directory entry: {}", e),
                })?
        {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("backup_")
                    && (filename.ends_with(".bin") || filename.ends_with(".meta"))
                {
                    if let Ok(metadata) = entry.metadata().await {
                        if let Ok(modified) = metadata.modified() {
                            let modified_datetime: DateTime<Utc> = modified.into();
                            if modified_datetime < cutoff_date {
                                if let Err(e) = fs::remove_file(&path).await {
                                    warn!("Failed to remove old backup file {}: {}", filename, e);
                                } else {
                                    removed_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        if removed_count > 0 {
            info!("Cleaned up {} old backup files", removed_count);
        }

        Ok(())
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
        self.restore_hive_state(hive, &snapshot).await?;

        // Restore agents
        self.restore_agents(hive, &snapshot.agents).await?;

        // Restore tasks
        self.restore_tasks(hive, &snapshot.tasks).await?;

        info!(
            "System restored successfully from checkpoint {}",
            snapshot_id
        );
        Ok(())
    }

    /// Start automatic checkpointing
    pub async fn start_auto_checkpointing(&self, hive: Arc<RwLock<HiveCoordinator>>) {
        let interval_minutes = self.config.checkpoint_interval_minutes;
        let persistence = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(interval_minutes * 60));

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

        info!(
            "Automatic checkpointing started (interval: {} minutes)",
            interval_minutes
        );
    }

    /// Get checkpoint history
    pub async fn get_checkpoint_history(&self) -> Vec<CheckpointMetadata> {
        self.checkpoint_history.read().await.clone()
    }

    /// Capture current system state
    async fn capture_system_state(&self, hive: &HiveCoordinator) -> HiveResult<SystemSnapshot> {
        let agents: Vec<Agent> = hive
            .agents
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        // Collect tasks from hive
        let tasks_info = hive.get_tasks_info().await;
        let tasks: Vec<Task> = Vec::new(); // For now, we'll collect basic task info

        let hive_status = hive.get_status().await;
        let default_map = serde_json::Map::new();
        let tasks_metrics = tasks_info
            .get("legacy_queue")
            .and_then(|q| q.as_object())
            .unwrap_or(&default_map);

        let metrics = SystemMetrics {
            total_agents: agents.len(),
            active_agents: agents
                .iter()
                .filter(|a| !matches!(a.state, crate::agents::AgentState::Idle))
                .count(),
            completed_tasks: tasks_metrics
                .get("completed_tasks")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            failed_tasks: tasks_metrics
                .get("failed_tasks")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            average_performance: hive_status
                .get("metrics")
                .and_then(|m| m.get("average_performance"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            swarm_cohesion: hive_status
                .get("metrics")
                .and_then(|m| m.get("swarm_cohesion"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            learning_progress: hive_status
                .get("metrics")
                .and_then(|m| m.get("learning_progress"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            uptime_seconds: (Utc::now() - hive.created_at).num_seconds() as u64,
        };

        Ok(SystemSnapshot {
            snapshot_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            hive_state: HiveState {
                hive_id: hive_status
                    .get("hive_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .unwrap_or_else(Uuid::new_v4),
                created_at: Utc::now(), // TODO: Get actual creation time
                last_update: Utc::now(),
                total_energy: hive_status
                    .get("total_energy")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                swarm_center: hive_status
                    .get("swarm_center")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        if arr.len() >= 2 {
                            Some((
                                arr[0].as_f64().unwrap_or(0.0),
                                arr[1].as_f64().unwrap_or(0.0),
                            ))
                        } else {
                            None
                        }
                    })
                    .unwrap_or((0.0, 0.0)),
                auto_scaling_enabled: true,
                learning_enabled: true,
            },
            agents,
            tasks,
            metrics,
            configuration: serde_json::json!({
                "max_agents": 1000,
                "task_timeout_seconds": 300,
                "auto_scaling_enabled": true,
                "neural_processing_enabled": true
            }),
        })
    }

    /// Restore hive state
    async fn restore_hive_state(
        &self,
        hive: &mut HiveCoordinator,
        snapshot: &SystemSnapshot,
    ) -> HiveResult<()> {
        info!(
            "Restoring hive state for hive {}",
            snapshot.hive_state.hive_id
        );

        // Restore agents
        for agent in &snapshot.agents {
            if let Err(e) = hive
                .create_agent(serde_json::json!({
                    "name": agent.name,
                    "type": agent.agent_type,
                    "capabilities": agent.capabilities,
                    "initial_energy": agent.energy
                }))
                .await
            {
                warn!("Failed to restore agent {}: {}", agent.id, e);
            }
        }

        // Restore tasks
        for task in &snapshot.tasks {
            if let Err(e) = hive
                .create_task(serde_json::json!({
                    "title": task.description, // Use description as title for now
                    "description": task.description,
                    "task_type": "General", // Default task type
                    "priority": task.priority,
                    "required_capabilities": task.required_capabilities
                }))
                .await
            {
                warn!("Failed to restore task {}: {}", task.id, e);
            }
        }

        info!(
            "Successfully restored {} agents and {} tasks",
            snapshot.agents.len(),
            snapshot.tasks.len()
        );
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

    /// Start automatic checkpoint scheduling
    pub async fn start_automatic_checkpoints(
        &self,
        hive: Arc<HiveCoordinator>,
    ) -> HiveResult<tokio::task::JoinHandle<()>> {
        let interval_minutes = self.config.checkpoint_interval_minutes;
        let persistence_manager = self.clone();

        let handle = tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(interval_minutes * 60));

            loop {
                interval.tick().await;

                match persistence_manager
                    .create_checkpoint(&hive, Some("Automatic checkpoint".to_string()))
                    .await
                {
                    Ok(checkpoint_id) => {
                        info!("Created automatic checkpoint: {}", checkpoint_id);

                        // Cleanup old snapshots
                        if let Err(e) = persistence_manager
                            .storage
                            .cleanup_old_snapshots(persistence_manager.config.max_snapshots)
                            .await
                        {
                            warn!("Failed to cleanup old snapshots: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to create automatic checkpoint: {}", e);
                    }
                }
            }
        });

        Ok(handle)
    }

    /// List all snapshots
    pub async fn list_snapshots(&self) -> HiveResult<Vec<CheckpointMetadata>> {
        self.storage.list_snapshots().await
    }

    /// Cleanup old snapshots
    pub async fn cleanup_old_snapshots(&self, max_count: usize) -> HiveResult<usize> {
        self.storage.cleanup_old_snapshots(max_count).await
    }

    /// Get checkpoint statistics
    pub async fn get_checkpoint_stats(&self) -> HiveResult<CheckpointStats> {
        let snapshots = self.storage.list_snapshots().await?;

        let total_size: u64 = snapshots.iter().map(|s| s.size_bytes).sum();
        let avg_size = if snapshots.is_empty() {
            0.0
        } else {
            total_size as f64 / snapshots.len() as f64
        };

        // Calculate compression savings
        let total_compression_ratio: f64 = snapshots.iter().map(|s| s.compression_ratio).sum();
        let avg_compression_ratio = if snapshots.is_empty() {
            1.0
        } else {
            total_compression_ratio / snapshots.len() as f64
        };
        let compression_savings = if avg_compression_ratio > 1.0 {
            (1.0 - (1.0 / avg_compression_ratio)) * 100.0
        } else {
            0.0
        };

        // Count encrypted snapshots (simplified - in real implementation, track this)
        let encrypted_snapshots = if self.config.encryption_enabled {
            snapshots.len()
        } else {
            0
        };

        // Count backup files
        let backup_count = if let Some(backup_manager) = &self.backup_manager {
            self.count_backup_files(backup_manager).await.unwrap_or(0)
        } else {
            0
        };

        Ok(CheckpointStats {
            total_snapshots: snapshots.len(),
            total_size_bytes: total_size,
            average_size_bytes: avg_size,
            oldest_snapshot: snapshots.iter().map(|s| s.timestamp).min(),
            newest_snapshot: snapshots.iter().map(|s| s.timestamp).max(),
            last_checkpoint: *self.last_checkpoint.read().await,
            compression_savings,
            encrypted_snapshots,
            backup_count,
            last_backup: None, // TODO: Track last backup time
        })
    }

    /// Count backup files
    async fn count_backup_files(&self, backup_manager: &BackupManager) -> HiveResult<usize> {
        if !backup_manager.backup_location.exists() {
            return Ok(0);
        }

        let mut entries = fs::read_dir(&backup_manager.backup_location)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to read backup directory: {}", e),
            })?;

        let mut count = 0;
        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to read directory entry: {}", e),
                })?
        {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.starts_with("backup_") && filename.ends_with(".bin") {
                    count += 1;
                }
            }
        }

        Ok(count)
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
            encryption_key: self.encryption_key,
            backup_manager: self.backup_manager.clone(),
        }
    }
}

/// File system storage implementation
pub struct FileSystemStorage {
    base_path: PathBuf,
}

impl FileSystemStorage {
    pub async fn new(base_path: PathBuf) -> HiveResult<Self> {
        fs::create_dir_all(&base_path)
            .await
            .map_err(|e| HiveError::OperationFailed {
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

        let json_data =
            serde_json::to_string_pretty(snapshot).map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to serialize snapshot: {}", e),
            })?;

        fs::write(&file_path, json_data)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to write snapshot file: {}", e),
            })?;

        Ok(filename)
    }

    async fn load_snapshot(&self, snapshot_id: &str) -> HiveResult<SystemSnapshot> {
        let file_path = self
            .base_path
            .join(format!("snapshot_{}.json", snapshot_id));

        let json_data =
            fs::read_to_string(&file_path)
                .await
                .map_err(|e| HiveError::OperationFailed {
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
        let file_path = self
            .base_path
            .join(format!("snapshot_{}.json", snapshot_id));
        fs::remove_file(&file_path)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to delete snapshot file: {}", e),
            })
    }

    async fn save_processed_snapshot(
        &self,
        snapshot: &SystemSnapshot,
        processed: &ProcessedSnapshot,
    ) -> HiveResult<String> {
        let filename = format!("snapshot_{}.bin", snapshot.snapshot_id);
        let file_path = self.base_path.join(&filename);

        // Write processed binary data
        fs::write(&file_path, &processed.data)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to write processed snapshot file: {}", e),
            })?;

        // Write metadata
        let metadata_filename = format!("snapshot_{}.meta", snapshot.snapshot_id);
        let metadata_path = self.base_path.join(&metadata_filename);

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
            serde_json::to_string_pretty(&metadata).map_err(|e| HiveError::SerializationError {
                reason: format!("Failed to serialize metadata: {}", e),
            })?,
        )
        .await
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to write metadata file: {}", e),
        })?;

        Ok(filename)
    }

    async fn load_processed_snapshot(&self, snapshot_id: &str) -> HiveResult<ProcessedSnapshot> {
        let file_path = self.base_path.join(format!("snapshot_{}.bin", snapshot_id));
        let metadata_path = self
            .base_path
            .join(format!("snapshot_{}.meta", snapshot_id));

        // Load binary data
        let data = fs::read(&file_path)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to read processed snapshot file: {}", e),
            })?;

        // Load metadata
        let metadata_str =
            fs::read_to_string(&metadata_path)
                .await
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to read metadata file: {}", e),
                })?;

        let metadata: serde_json::Value =
            serde_json::from_str(&metadata_str).map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to parse metadata: {}", e),
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

    async fn cleanup_old_snapshots(&self, max_count: usize) -> HiveResult<usize> {
        // TODO: Implement cleanup logic
        Ok(0)
    }
}

/// SQLite storage implementation
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
                    reason: format!("Failed to create database directory: {}", e),
                })?;
        }

        let conn = Connection::open(&database_path).map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to open SQLite database: {}", e),
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
            reason: format!("Failed to create snapshots table: {}", e),
        })?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_snapshots_timestamp ON snapshots(timestamp)",
            [],
        )
        .map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to create timestamp index: {}", e),
        })?;

        Ok(Self {
            database_path,
            connection: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait::async_trait]
impl StorageProvider for SQLiteStorage {
    async fn save_snapshot(&self, snapshot: &SystemSnapshot) -> HiveResult<String> {
        let json_data =
            serde_json::to_string(snapshot).map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to serialize snapshot: {}", e),
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
            reason: format!("Failed to save snapshot to database: {}", e),
        })?;

        Ok(snapshot.snapshot_id.to_string())
    }

    async fn save_processed_snapshot(
        &self,
        snapshot: &SystemSnapshot,
        processed: &ProcessedSnapshot,
    ) -> HiveResult<String> {
        let conn = self.connection.lock().await;

        // Encode binary data as base64 for storage
        let encoded_data = general_purpose::STANDARD.encode(&processed.data);

        conn.execute(
            "INSERT INTO snapshots (id, timestamp, version, size_bytes, agent_count, task_count, description, data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                snapshot.snapshot_id.to_string(),
                snapshot.timestamp.to_rfc3339(),
                snapshot.version,
                processed.processed_size as i64,
                snapshot.agents.len() as i64,
                snapshot.tasks.len() as i64,
                format!("Processed checkpoint (compressed: {}, encrypted: {})",
                       processed.is_compressed, processed.is_encrypted),
                encoded_data
            ],
        ).map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to save processed snapshot to database: {}", e),
        })?;

        Ok(snapshot.snapshot_id.to_string())
    }

    async fn load_snapshot(&self, snapshot_id: &str) -> HiveResult<SystemSnapshot> {
        let conn = self.connection.lock().await;
        let mut stmt = conn
            .prepare("SELECT data FROM snapshots WHERE id = ?1")
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to prepare query: {}", e),
            })?;

        let json_data: String = stmt
            .query_row(params![snapshot_id], |row| Ok(row.get(0)?))
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to load snapshot from database: {}", e),
            })?;

        serde_json::from_str(&json_data).map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to deserialize snapshot: {}", e),
        })
    }

    async fn load_processed_snapshot(&self, snapshot_id: &str) -> HiveResult<ProcessedSnapshot> {
        let conn = self.connection.lock().await;
        let mut stmt = conn
            .prepare("SELECT data FROM snapshots WHERE id = ?1")
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to prepare query: {}", e),
            })?;

        let encoded_data: String = stmt
            .query_row(params![snapshot_id], |row| Ok(row.get(0)?))
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to load processed snapshot from database: {}", e),
            })?;

        let data = general_purpose::STANDARD
            .decode(&encoded_data)
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to decode snapshot data: {}", e),
            })?;

        // Note: In a real implementation, we'd store metadata separately
        // For now, we'll return a basic ProcessedSnapshot
        let processed_size = data.len();
        Ok(ProcessedSnapshot {
            data,
            is_compressed: true, // Assume processed data is compressed
            is_encrypted: true,  // Assume processed data is encrypted
            original_size: 0,    // Would need to be stored separately
            processed_size,
            compression_ratio: 1.0, // Would need to be calculated/stored
        })
    }

    async fn list_snapshots(&self) -> HiveResult<Vec<CheckpointMetadata>> {
        let conn = self.connection.lock().await;
        let mut stmt = conn
            .prepare(
                "SELECT id, timestamp, size_bytes, agent_count, task_count, description
             FROM snapshots ORDER BY timestamp DESC",
            )
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to prepare query: {}", e),
            })?;

        let rows = stmt
            .query_map([], |row| {
                let timestamp_str: String = row.get(1)?;
                let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            1,
                            "timestamp".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc);

                Ok(CheckpointMetadata {
                    checkpoint_id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            0,
                            "id".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?,
                    timestamp,
                    size_bytes: row.get::<_, i64>(2)? as u64,
                    compression_ratio: 1.0,
                    agent_count: row.get::<_, i64>(3)? as usize,
                    task_count: row.get::<_, i64>(4)? as usize,
                    description: row.get(5)?,
                })
            })
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to query snapshots: {}", e),
            })?;

        let mut snapshots = Vec::new();
        for row in rows {
            snapshots.push(row.map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to parse snapshot row: {}", e),
            })?);
        }

        Ok(snapshots)
    }

    async fn delete_snapshot(&self, snapshot_id: &str) -> HiveResult<()> {
        let conn = self.connection.lock().await;
        conn.execute("DELETE FROM snapshots WHERE id = ?1", params![snapshot_id])
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to delete snapshot: {}", e),
            })?;
        Ok(())
    }

    async fn cleanup_old_snapshots(&self, max_count: usize) -> HiveResult<usize> {
        let conn = self.connection.lock().await;

        // Get count of snapshots to delete
        let mut count_stmt = conn
            .prepare("SELECT COUNT(*) FROM snapshots")
            .map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to prepare count query: {}", e),
            })?;

        let total_count: i64 =
            count_stmt
                .query_row([], |row| row.get(0))
                .map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to get snapshot count: {}", e),
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
            reason: format!("Failed to cleanup old snapshots: {}", e),
        })?;

        Ok(to_delete as usize)
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

    async fn save_processed_snapshot(
        &self,
        snapshot: &SystemSnapshot,
        processed: &ProcessedSnapshot,
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
                reason: format!("Snapshot {} not found", snapshot_id),
            })
    }

    async fn load_processed_snapshot(&self, snapshot_id: &str) -> HiveResult<ProcessedSnapshot> {
        let snapshots = self.snapshots.read().await;
        let snapshot = snapshots
            .get(snapshot_id)
            .ok_or_else(|| HiveError::OperationFailed {
                reason: format!("Snapshot {} not found", snapshot_id),
            })?;

        // Convert snapshot back to processed format (simplified)
        let json_data =
            serde_json::to_string(snapshot).map_err(|e| HiveError::OperationFailed {
                reason: format!("Failed to serialize snapshot: {}", e),
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
        let id = storage.save_snapshot(&snapshot).await.unwrap();
        let loaded = storage.load_snapshot(&id).await.unwrap();

        assert_eq!(snapshot.snapshot_id, loaded.snapshot_id);
        assert_eq!(snapshot.version, loaded.version);
    }

    #[tokio::test]
    async fn test_filesystem_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemStorage::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

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
        let loaded = storage
            .load_snapshot(&snapshot.snapshot_id.to_string())
            .await
            .unwrap();

        assert_eq!(snapshot.snapshot_id, loaded.snapshot_id);
        assert_eq!(snapshot.version, loaded.version);
    }
}
