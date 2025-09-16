//! # Core Hive Coordinator Module
//!
//! This module contains the refactored HiveCoordinator with improved modularity.
//! The coordinator has been broken down into focused sub-modules for better maintainability.
//!
//! ## Sub-modules
//!
//! - `core` - Main coordinator struct and core operations
//! - `messages` - Coordination message types and handling
//! - `lifecycle` - System startup and shutdown operations
//! - `status` - Status reporting and analytics
//! - `tests` - Test utilities and helpers

pub mod core;
pub mod lifecycle;
pub mod messages;
pub mod status;

#[cfg(test)]
pub mod tests;

// Re-export main types for backward compatibility
pub use core::HiveCoordinator;
pub use messages::CoordinationMessage;
