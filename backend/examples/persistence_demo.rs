//! Comprehensive Persistence System Demo
//!
//! Demonstrates the full persistence capabilities including:
//! - Multiple storage backends (Memory, FileSystem, SQLite)
//! - Automatic checkpointing
//! - System state restoration
//! - Checkpoint management and cleanup

use multiagent_hive::{
    infrastructure::{PersistenceConfig, PersistenceManager, StorageBackend},
    HiveCoordinator,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Comprehensive Persistence System Demo");

    // Demo 1: Memory Storage (for testing)
    demo_memory_storage().await?;

    // Demo 2: File System Storage
    demo_filesystem_storage().await?;

    // Demo 3: SQLite Storage
    demo_sqlite_storage().await?;

    // Demo 4: Automatic Checkpointing
    demo_automatic_checkpointing().await?;

    info!("✅ All persistence demos completed successfully!");
    Ok(())
}

async fn demo_memory_storage() -> anyhow::Result<()> {
    info!("\n📦 Demo 1: Memory Storage");

    let config = PersistenceConfig {
        storage_backend: StorageBackend::Memory { max_snapshots: 5 },
        checkpoint_interval_minutes: 1,
        max_snapshots: 5,
        compression_enabled: false,
        encryption_enabled: false,
        backup_enabled: false,
        storage_path: PathBuf::from("/tmp"),
    };

    let persistence = PersistenceManager::new(config).await?;
    let hive = create_test_hive().await?;

    // Create multiple checkpoints
    for i in 1..=3 {
        let checkpoint_id = persistence
            .create_checkpoint(&hive, Some(format!("Memory checkpoint {}", i)))
            .await?;
        info!("Created checkpoint {}: {}", i, checkpoint_id);
    }

    // Get statistics
    let stats = persistence.get_checkpoint_stats().await?;
    info!("Memory storage stats: {:?}", stats);

    Ok(())
}

async fn demo_filesystem_storage() -> anyhow::Result<()> {
    info!("\n📁 Demo 2: File System Storage");

    let temp_dir = tempfile::tempdir()?;
    let config = PersistenceConfig {
        storage_backend: StorageBackend::FileSystem {
            base_path: temp_dir.path().to_path_buf(),
        },
        checkpoint_interval_minutes: 1,
        max_snapshots: 10,
        compression_enabled: true,
        encryption_enabled: false,
        backup_enabled: true,
        storage_path: temp_dir.path().to_path_buf(),
    };

    let persistence = PersistenceManager::new(config).await?;
    let mut hive = create_test_hive().await?;

    // Create checkpoint
    let checkpoint_id = persistence
        .create_checkpoint(&hive, Some("FileSystem checkpoint".to_string()))
        .await?;
    info!("Created filesystem checkpoint: {}", checkpoint_id);

    // Modify hive state
    add_test_agents(&mut hive).await?;

    // Create another checkpoint
    let checkpoint_id_2 = persistence
        .create_checkpoint(&hive, Some("FileSystem checkpoint with agents".to_string()))
        .await?;
    info!("Created second checkpoint: {}", checkpoint_id_2);

    // Restore from first checkpoint
    let mut restored_hive = create_test_hive().await?;
    persistence
        .restore_from_checkpoint(&checkpoint_id.to_string(), &mut restored_hive)
        .await?;
    info!(
        "Restored hive from checkpoint with {} agents",
        restored_hive.agents.len()
    );

    // Get statistics
    let stats = persistence.get_checkpoint_stats().await?;
    info!("FileSystem storage stats: {:?}", stats);

    Ok(())
}

async fn demo_sqlite_storage() -> anyhow::Result<()> {
    info!("\n🗄️  Demo 3: SQLite Storage");

    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("hive_persistence.db");

    let config = PersistenceConfig {
        storage_backend: StorageBackend::SQLite {
            database_path: db_path.clone(),
        },
        checkpoint_interval_minutes: 1,
        max_snapshots: 20,
        compression_enabled: true,
        encryption_enabled: false,
        backup_enabled: true,
        storage_path: temp_dir.path().to_path_buf(),
    };

    let persistence = PersistenceManager::new(config).await?;
    let mut hive = create_test_hive().await?;

    // Create multiple checkpoints with different states
    for i in 1..=5 {
        add_test_agents(&mut hive).await?;

        let checkpoint_id = persistence
            .create_checkpoint(
                &hive,
                Some(format!(
                    "SQLite checkpoint {} with {} agents",
                    i,
                    hive.agents.len()
                )),
            )
            .await?;
        info!("Created SQLite checkpoint {}: {}", i, checkpoint_id);

        sleep(Duration::from_millis(100)).await; // Ensure different timestamps
    }

    // List all checkpoints
    let snapshots = persistence.list_snapshots().await?;
    info!("Found {} snapshots in SQLite database", snapshots.len());
    for snapshot in &snapshots {
        info!(
            "  - {}: {} ({} agents, {} bytes)",
            snapshot.checkpoint_id, snapshot.description, snapshot.agent_count, snapshot.size_bytes
        );
    }

    // Test cleanup
    let cleaned = persistence.cleanup_old_snapshots(3).await?;
    info!("Cleaned up {} old snapshots", cleaned);

    // Get final statistics
    let stats = persistence.get_checkpoint_stats().await?;
    info!("SQLite storage final stats: {:?}", stats);

    info!("SQLite database created at: {:?}", db_path);
    Ok(())
}

async fn demo_automatic_checkpointing() -> anyhow::Result<()> {
    info!("\n⏰ Demo 4: Automatic Checkpointing");

    let temp_dir = tempfile::tempdir()?;
    let config = PersistenceConfig {
        storage_backend: StorageBackend::FileSystem {
            base_path: temp_dir.path().to_path_buf(),
        },
        checkpoint_interval_minutes: 0, // Very short interval for demo (converted to seconds)
        max_snapshots: 3,
        compression_enabled: false,
        encryption_enabled: false,
        backup_enabled: false,
        storage_path: temp_dir.path().to_path_buf(),
    };

    let persistence = PersistenceManager::new(config).await?;
    let hive = Arc::new(create_test_hive().await?);

    // Start automatic checkpointing (every 2 seconds for demo)
    info!("Starting automatic checkpointing every 2 seconds...");
    let _checkpoint_handle = tokio::spawn({
        let persistence = persistence.clone();
        let hive = hive.clone();
        async move {
            let mut interval = tokio::time::interval(Duration::from_secs(2));
            for i in 1..=3 {
                interval.tick().await;
                match persistence
                    .create_checkpoint(&hive, Some(format!("Auto checkpoint {}", i)))
                    .await
                {
                    Ok(id) => info!("Auto checkpoint {} created: {}", i, id),
                    Err(e) => warn!("Auto checkpoint {} failed: {}", i, e),
                }
            }
        }
    });

    // Wait for automatic checkpoints
    sleep(Duration::from_secs(7)).await;

    // Check final statistics
    let stats = persistence.get_checkpoint_stats().await?;
    info!("Automatic checkpointing stats: {:?}", stats);

    Ok(())
}

async fn create_test_hive() -> anyhow::Result<HiveCoordinator> {
    HiveCoordinator::new().await
}

async fn add_test_agents(hive: &mut HiveCoordinator) -> anyhow::Result<()> {
    let agent_config = serde_json::json!({
        "agent_type": "Worker",
        "capabilities": ["general_processing", "coordination"],
        "initial_energy": 100.0
    });

    for _ in 0..2 {
        hive.create_agent(agent_config.clone()).await?;
    }

    Ok(())
}
