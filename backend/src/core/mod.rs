/// Dynamic agent auto-scaling system
pub mod auto_scaling;
/// Intelligent fallback system for agent selection
pub mod fallback;
/// Core hive coordination and management system (modular)
pub mod hive;
/// Legacy hive module - deprecated in favor of modular hive
pub mod hive_legacy;
/// Enhanced swarm coordination with neural intelligence
pub mod swarm_coordination;
/// Swarm intelligence algorithms and formation optimization
pub mod swarm_intelligence;

pub use fallback::*;
// Re-export the main coordinator for backward compatibility
pub use hive::HiveCoordinator;
pub use hive_legacy::HiveCoordinator as LegacyHiveCoordinator;
pub use swarm_intelligence::*;
