//! Legacy Hive Coordinator - Backward Compatibility
//!
//! This module provides backward compatibility by re-exporting the new modular
//! HiveCoordinator. This allows existing code to continue working while we
//! transition to the new modular architecture.

// Re-export the new modular HiveCoordinator for backward compatibility
pub use super::hive::HiveCoordinator;
