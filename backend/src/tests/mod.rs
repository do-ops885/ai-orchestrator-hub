//! # Comprehensive Unit Tests for Multiagent Hive System
//!
//! This module contains unit tests for all major components of the multiagent hive system.
//! Tests are organized by module and cover both positive and negative test cases.

pub mod agent_tests;
pub mod hive_tests;
pub mod integration_tests;
pub mod neural_tests;
pub mod task_tests;

// Edge case tests for unwrap fixes
pub mod adaptive_verification_edge_case_tests;
pub mod config_edge_case_tests;
pub mod error_handling_edge_case_tests;
pub mod testing_edge_case_tests;

// Test utilities and common fixtures
pub mod test_utils;
