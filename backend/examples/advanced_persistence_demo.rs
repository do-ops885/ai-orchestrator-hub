//! Advanced Persistence System Demo
//! 
//! Demonstrates the enhanced persistence capabilities including:
//! - Encryption with AES-256-GCM
//! - Compression with configurable levels
//! - Automatic backup management
//! - Advanced statistics and monitoring

use multiagent_hive::{
    HiveCoordinator,
    infrastructure::{
        PersistenceManager, PersistenceConfig, StorageBackend,
    },
};
use tokio::time::{sleep, Duration};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Advanced Persistence System Demo");

    // Demo 1: Encryption and Compression
    demo_encryption_compression().await?;

    // Demo 2: Backup Management
    demo_backup_management().await?;

    // Demo 3: Advanced Statistics
    demo_advanced_statistics().await?;

    // Demo 4: Performance Comparison
    demo_performance_comparison().await?;

    info!("✅ All advanced persistence demos completed successfully!");
    Ok(())
}

async fn demo_encryption_compression() -> anyhow::Result<()> {
    info!("\n🔐 Demo 1: Encryption and Compression");
    
    let temp_dir = tempfile::tempdir()?;
    
    // Configuration with encryption and compression enabled
    let config = PersistenceConfig {
        storage_backend: StorageBackend::SQLite { 
            database_path: temp_dir.path().join("encrypted_hive.db") 
        },
        checkpoint_interval_minutes: 1,
        max_snapshots: 10,
        compression_enabled: true,
        encryption_enabled: true,
        backup_enabled: true,
        storage_path: temp_dir.path().to_path_buf(),
        encryption_key: Some("4f8a9b2c3d1e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f01".to_string()), // 64 hex chars = 32 bytes
        compression_level: 6, // Good balance of speed and compression
        backup_retention_days: 7,
        backup_location: Some(temp_dir.path().join("backups")),
        incremental_backup: false,
    };

    let persistence = PersistenceManager::new(config).await?;
    let mut hive = create_test_hive().await?;

    // Add some agents to make the data more substantial
    for i in 0..5 {
        add_test_agents(&mut hive).await?;
        info!("Added agents batch {}, total agents: {}", i + 1, hive.agents.len());
    }

    // Create encrypted and compressed checkpoint
    let checkpoint_id = persistence.create_checkpoint(
        &hive, 
        Some("Encrypted and compressed checkpoint".to_string())
    ).await?;
    info!("Created encrypted checkpoint: {}", checkpoint_id);

    // Verify we can restore from encrypted checkpoint
    let mut restored_hive = create_test_hive().await?;
    persistence.restore_from_checkpoint(&checkpoint_id.to_string(), &mut restored_hive).await?;
    info!("Successfully restored from encrypted checkpoint with {} agents", restored_hive.agents.len());

    // Get statistics
    let stats = persistence.get_checkpoint_stats().await?;
    info!("Encryption/Compression stats: {:?}", stats);

    Ok(())
}

async fn demo_backup_management() -> anyhow::Result<()> {
    info!("\n💾 Demo 2: Backup Management");
    
    let temp_dir = tempfile::tempdir()?;
    let backup_dir = temp_dir.path().join("backups");
    
    let config = PersistenceConfig {
        storage_backend: StorageBackend::FileSystem { 
            base_path: temp_dir.path().join("storage") 
        },
        checkpoint_interval_minutes: 1,
        max_snapshots: 5,
        compression_enabled: true,
        encryption_enabled: false, // Disable encryption for this demo
        backup_enabled: true,
        storage_path: temp_dir.path().to_path_buf(),
        encryption_key: None,
        compression_level: 9, // Maximum compression
        backup_retention_days: 1, // Short retention for demo
        backup_location: Some(backup_dir.clone()),
        incremental_backup: false,
    };

    let persistence = PersistenceManager::new(config).await?;
    let mut hive = create_test_hive().await?;

    // Create multiple checkpoints with backups
    for i in 1..=3 {
        add_test_agents(&mut hive).await?;
        
        let checkpoint_id = persistence.create_checkpoint(
            &hive, 
            Some(format!("Backup demo checkpoint {}", i))
        ).await?;
        info!("Created checkpoint {} with backup: {}", i, checkpoint_id);
        
        sleep(Duration::from_millis(100)).await; // Ensure different timestamps
    }

    // List backup files
    if backup_dir.exists() {
        let mut entries = tokio::fs::read_dir(&backup_dir).await?;
        let mut backup_count = 0;
        while let Some(entry) = entries.next_entry().await? {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.starts_with("backup_") {
                    backup_count += 1;
                    info!("Found backup file: {}", filename);
                }
            }
        }
        info!("Total backup files created: {}", backup_count);
    }

    let stats = persistence.get_checkpoint_stats().await?;
    info!("Backup management stats: {:?}", stats);

    Ok(())
}

async fn demo_advanced_statistics() -> anyhow::Result<()> {
    info!("\n📊 Demo 3: Advanced Statistics");
    
    let temp_dir = tempfile::tempdir()?;
    
    let config = PersistenceConfig {
        storage_backend: StorageBackend::SQLite { 
            database_path: temp_dir.path().join("stats_hive.db") 
        },
        checkpoint_interval_minutes: 1,
        max_snapshots: 20,
        compression_enabled: true,
        encryption_enabled: true,
        backup_enabled: true,
        storage_path: temp_dir.path().to_path_buf(),
        encryption_key: None, // Auto-generate key
        compression_level: 3, // Fast compression
        backup_retention_days: 30,
        backup_location: Some(temp_dir.path().join("backups")),
        incremental_backup: true,
    };

    let persistence = PersistenceManager::new(config).await?;
    let mut hive = create_test_hive().await?;

    // Create checkpoints with varying data sizes
    let checkpoint_sizes = vec![2, 5, 10, 15, 20]; // Number of agent batches
    let mut checkpoint_ids = Vec::new();

    for (i, size) in checkpoint_sizes.iter().enumerate() {
        // Add agents to reach target size
        while hive.agents.len() < size * 2 {
            add_test_agents(&mut hive).await?;
        }
        
        let checkpoint_id = persistence.create_checkpoint(
            &hive, 
            Some(format!("Statistics demo checkpoint {} ({} agents)", i + 1, hive.agents.len()))
        ).await?;
        checkpoint_ids.push(checkpoint_id);
        
        info!("Created checkpoint {} with {} agents", i + 1, hive.agents.len());
        sleep(Duration::from_millis(50)).await;
    }

    // Get comprehensive statistics
    let stats = persistence.get_checkpoint_stats().await?;
    info!("\n📈 Comprehensive Statistics:");
    info!("  Total snapshots: {}", stats.total_snapshots);
    info!("  Total size: {} bytes", stats.total_size_bytes);
    info!("  Average size: {:.2} bytes", stats.average_size_bytes);
    info!("  Compression savings: {:.1}%", stats.compression_savings);
    info!("  Encrypted snapshots: {}", stats.encrypted_snapshots);
    info!("  Backup count: {}", stats.backup_count);
    
    if let Some(oldest) = stats.oldest_snapshot {
        info!("  Oldest snapshot: {}", oldest.format("%Y-%m-%d %H:%M:%S"));
    }
    if let Some(newest) = stats.newest_snapshot {
        info!("  Newest snapshot: {}", newest.format("%Y-%m-%d %H:%M:%S"));
    }

    // List all snapshots with details
    let snapshots = persistence.list_snapshots().await?;
    info!("\n📋 Snapshot Details:");
    for snapshot in &snapshots {
        info!("  {} - {} agents, {} bytes, {:.2}x compression", 
            snapshot.checkpoint_id,
            snapshot.agent_count,
            snapshot.size_bytes,
            snapshot.compression_ratio
        );
    }

    Ok(())
}

async fn demo_performance_comparison() -> anyhow::Result<()> {
    info!("\n⚡ Demo 4: Performance Comparison");
    
    let temp_dir = tempfile::tempdir()?;
    
    // Test different configurations
    let configs = vec![
        ("No compression, no encryption", PersistenceConfig {
            storage_backend: StorageBackend::Memory { max_snapshots: 10 },
            checkpoint_interval_minutes: 1,
            max_snapshots: 10,
            compression_enabled: false,
            encryption_enabled: false,
            backup_enabled: false,
            storage_path: temp_dir.path().to_path_buf(),
            encryption_key: None,
            compression_level: 1,
            backup_retention_days: 7,
            backup_location: None,
            incremental_backup: false,
        }),
        ("Fast compression, no encryption", PersistenceConfig {
            storage_backend: StorageBackend::Memory { max_snapshots: 10 },
            checkpoint_interval_minutes: 1,
            max_snapshots: 10,
            compression_enabled: true,
            encryption_enabled: false,
            backup_enabled: false,
            storage_path: temp_dir.path().to_path_buf(),
            encryption_key: None,
            compression_level: 1, // Fastest compression
            backup_retention_days: 7,
            backup_location: None,
            incremental_backup: false,
        }),
        ("Max compression + encryption", PersistenceConfig {
            storage_backend: StorageBackend::Memory { max_snapshots: 10 },
            checkpoint_interval_minutes: 1,
            max_snapshots: 10,
            compression_enabled: true,
            encryption_enabled: true,
            backup_enabled: false,
            storage_path: temp_dir.path().to_path_buf(),
            encryption_key: Some("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()),
            compression_level: 9, // Maximum compression
            backup_retention_days: 7,
            backup_location: None,
            incremental_backup: false,
        }),
    ];

    for (name, config) in configs {
        info!("\n🔬 Testing: {}", name);
        
        let persistence = PersistenceManager::new(config).await?;
        let mut hive = create_test_hive().await?;
        
        // Add substantial data
        for _ in 0..10 {
            add_test_agents(&mut hive).await?;
        }
        
        // Measure checkpoint creation time
        let start = std::time::Instant::now();
        let checkpoint_id = persistence.create_checkpoint(
            &hive, 
            Some(format!("Performance test: {}", name))
        ).await?;
        let duration = start.elapsed();
        
        info!("  Checkpoint created in: {:?}", duration);
        info!("  Checkpoint ID: {}", checkpoint_id);
        
        // Get stats for this configuration
        let stats = persistence.get_checkpoint_stats().await?;
        if stats.total_snapshots > 0 {
            info!("  Average size: {:.2} bytes", stats.average_size_bytes);
            if stats.compression_savings > 0.0 {
                info!("  Compression savings: {:.1}%", stats.compression_savings);
            }
        }
    }

    Ok(())
}

async fn create_test_hive() -> anyhow::Result<HiveCoordinator> {
    HiveCoordinator::new().await
}

async fn add_test_agents(hive: &mut HiveCoordinator) -> anyhow::Result<()> {
    let agent_config = serde_json::json!({
        "agent_type": "Worker",
        "capabilities": ["general_processing", "coordination", "analysis"],
        "initial_energy": 100.0
    });
    
    for _ in 0..2 {
        hive.create_agent(agent_config.clone()).await?;
    }
    
    Ok(())
}