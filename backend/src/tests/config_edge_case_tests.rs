//! # Edge Case Tests for Configuration Unwrap Fixes
//!
//! Tests to prevent regression of unwrap_or() to unwrap_or_else(||) changes
//! in the configuration parsing system.

use crate::HiveConfig;
use std::env;

/// Test invalid environment variable values for numeric parsing
#[test]
fn test_invalid_monitoring_interval_values() {
    // Test non-numeric values
    env::set_var("MONITORING_INTERVAL", "invalid");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, 5); // Should use default

    // Test negative values
    env::set_var("MONITORING_INTERVAL", "-10");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, 5); // Should use default

    // Test zero values
    env::set_var("MONITORING_INTERVAL", "0");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, 0); // Zero is valid

    // Test very large values
    env::set_var("MONITORING_INTERVAL", "999999999999999999999");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, 5); // Should use default for overflow

    // Test empty string
    env::set_var("MONITORING_INTERVAL", "");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, 5); // Should use default

    // Test whitespace
    env::set_var("MONITORING_INTERVAL", "  123  ");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, 5); // Should use default for invalid

    // Cleanup
    env::remove_var("MONITORING_INTERVAL");
}

/// Test invalid METRICS_RETENTION values
#[test]
fn test_invalid_metrics_retention_values() {
    // Test non-numeric values
    env::set_var("METRICS_RETENTION", "not_a_number");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.metrics_retention_days, 7); // Should use default

    // Test negative values
    env::set_var("METRICS_RETENTION", "-5");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.metrics_retention_days, 7); // Should use default

    // Test zero
    env::set_var("METRICS_RETENTION", "0");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.metrics_retention_days, 0); // Zero is valid

    // Test very large values
    env::set_var("METRICS_RETENTION", "999999999999999999999");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.metrics_retention_days, 7); // Should use default for overflow

    // Test empty string
    env::set_var("METRICS_RETENTION", "");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.metrics_retention_days, 7); // Should use default

    // Cleanup
    env::remove_var("METRICS_RETENTION");
}

/// Test invalid ALERT_THRESHOLD values
#[test]
fn test_invalid_alert_threshold_values() {
    // Test non-numeric values
    env::set_var("ALERT_THRESHOLD", "not_a_number");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.alert_threshold, 0.8); // Should use default

    // Test negative values
    env::set_var("ALERT_THRESHOLD", "-0.5");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.alert_threshold, 0.8); // Should use default

    // Test values greater than 1.0
    env::set_var("ALERT_THRESHOLD", "1.5");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.alert_threshold, 1.5); // Should accept as-is

    // Test zero
    env::set_var("ALERT_THRESHOLD", "0.0");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.alert_threshold, 0.0); // Zero is valid

    // Test very small decimal
    env::set_var("ALERT_THRESHOLD", "0.0000001");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.alert_threshold, 0.0000001); // Should accept

    // Test empty string
    env::set_var("ALERT_THRESHOLD", "");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.alert_threshold, 0.8); // Should use default

    // Test special characters
    env::set_var("ALERT_THRESHOLD", "0.8abc");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.alert_threshold, 0.8); // Should use default

    // Cleanup
    env::remove_var("ALERT_THRESHOLD");
}

/// Test missing required environment variables
#[test]
fn test_missing_required_env_vars() {
    // Clear all monitoring-related env vars
    env::remove_var("MONITORING_INTERVAL");
    env::remove_var("METRICS_RETENTION");
    env::remove_var("ALERT_THRESHOLD");
    env::remove_var("METRICS_ENDPOINT");
    env::remove_var("HEALTH_ENDPOINT");

    let config = HiveConfig::from_env().unwrap();

    // Should use all defaults
    assert_eq!(config.monitoring.monitoring_interval_secs, 5);
    assert_eq!(config.monitoring.metrics_retention_days, 7);
    assert_eq!(config.monitoring.alert_threshold, 0.8);
    assert_eq!(
        config.monitoring.metrics_endpoint,
        "http://localhost:8000/metrics"
    );
    assert_eq!(
        config.monitoring.health_endpoint,
        "http://localhost:8000/health"
    );
}

/// Test malformed environment variable formats
#[test]
fn test_malformed_env_var_formats() {
    // Test JSON-like strings that aren't valid numbers
    env::set_var("MONITORING_INTERVAL", "{\"value\": 10}");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, 5); // Should use default

    // Test array-like strings
    env::set_var("METRICS_RETENTION", "[7, 14, 30]");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.metrics_retention_days, 7); // Should use default

    // Test boolean strings
    env::set_var("ALERT_THRESHOLD", "true");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.alert_threshold, 0.8); // Should use default

    // Cleanup
    env::remove_var("MONITORING_INTERVAL");
    env::remove_var("METRICS_RETENTION");
    env::remove_var("ALERT_THRESHOLD");
}

/// Test empty string values
#[test]
fn test_empty_string_env_values() {
    // Test empty strings for all monitoring env vars
    env::set_var("MONITORING_INTERVAL", "");
    env::set_var("METRICS_RETENTION", "");
    env::set_var("ALERT_THRESHOLD", "");
    env::set_var("METRICS_ENDPOINT", "");
    env::set_var("HEALTH_ENDPOINT", "");

    let config = HiveConfig::from_env().unwrap();

    // Should use defaults for numeric values
    assert_eq!(config.monitoring.monitoring_interval_secs, 5);
    assert_eq!(config.monitoring.metrics_retention_days, 7);
    assert_eq!(config.monitoring.alert_threshold, 0.8);

    // Should use empty strings for string values (no unwrap_or_else for these)
    assert_eq!(config.monitoring.metrics_endpoint, "");
    assert_eq!(config.monitoring.health_endpoint, "");

    // Cleanup
    env::remove_var("MONITORING_INTERVAL");
    env::remove_var("METRICS_RETENTION");
    env::remove_var("ALERT_THRESHOLD");
    env::remove_var("METRICS_ENDPOINT");
    env::remove_var("HEALTH_ENDPOINT");
}

/// Test very long environment variable values
#[test]
fn test_very_long_env_values() {
    // Create a very long string
    let long_value = "1".repeat(10000);

    env::set_var("MONITORING_INTERVAL", &long_value);
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, 5); // Should use default for invalid

    // Test with a valid long number
    let long_number =
        "999999999999999999999999999999999999999999999999999999999999999999999999999999";
    env::set_var("METRICS_RETENTION", long_number);
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.metrics_retention_days, 7); // Should use default for overflow

    // Cleanup
    env::remove_var("MONITORING_INTERVAL");
    env::remove_var("METRICS_RETENTION");
}

/// Test special characters in environment variables
#[test]
fn test_special_characters_in_env_vars() {
    // Test various special characters
    let special_values = vec![
        "10\n20", // Newline
        "10\t20", // Tab
        "10 20",  // Space
        "10\x00", // Null byte
        "10\"",   // Quote
        "10'",    // Single quote
        "10\\",   // Backslash
        "10;",    // Semicolon
        "10:",    // Colon
        "10|",    // Pipe
        "10&",    // Ampersand
        "10$",    // Dollar
        "10%",    // Percent
        "10#",    // Hash
        "10@",    // At
        "10!",    // Exclamation
        "10^",    // Caret
        "10*",    // Asterisk
        "10(",    // Parentheses
        "10)",    // Parentheses
        "10[",    // Bracket
        "10]",    // Bracket
        "10{",    // Brace
        "10}",    // Brace
        "10<",    // Less than
        "10>",    // Greater than
        "10?",    // Question mark
        "10/",    // Forward slash
        "10\\",   // Backslash
        "10~",    // Tilde
        "10`",    // Backtick
    ];

    for value in special_values {
        env::set_var("MONITORING_INTERVAL", value);
        let config = HiveConfig::from_env().unwrap();
        assert_eq!(
            config.monitoring.monitoring_interval_secs, 5,
            "Failed for special character value: {}",
            value
        ); // Should use default
    }

    // Cleanup
    env::remove_var("MONITORING_INTERVAL");
}

/// Test boundary conditions for numeric parsing
#[test]
fn test_numeric_boundary_conditions() {
    // Test maximum u64 values
    env::set_var("MONITORING_INTERVAL", &u64::MAX.to_string());
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, u64::MAX); // Should accept

    // Test minimum u64 values
    env::set_var("MONITORING_INTERVAL", "0");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, 0); // Should accept

    // Test maximum u64 minus 1
    env::set_var("MONITORING_INTERVAL", &(u64::MAX - 1).to_string());
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, u64::MAX - 1); // Should accept

    // Test very large numbers that might cause overflow
    env::set_var("MONITORING_INTERVAL", "18446744073709551616"); // u64::MAX + 1
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, 5); // Should use default for overflow

    // Cleanup
    env::remove_var("MONITORING_INTERVAL");
}

/// Test concurrent environment variable access
#[tokio::test]
async fn test_concurrent_env_var_access() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let counter = Arc::new(Mutex::new(0));

    // Spawn multiple tasks that set and read env vars
    let mut handles = vec![];

    for i in 0..10 {
        let counter_clone = counter.clone();
        let handle = tokio::spawn(async move {
            // Set different values
            env::set_var("MONITORING_INTERVAL", i.to_string());
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;

            // Read config
            let config = HiveConfig::from_env().unwrap();
            let mut count = counter_clone.lock().await;
            *count += 1;

            // Verify config has expected value or default
            assert!(
                config.monitoring.monitoring_interval_secs == i as u64
                    || config.monitoring.monitoring_interval_secs == 5
            );
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let final_count = *counter.lock().await;
    assert_eq!(final_count, 10);

    // Cleanup
    env::remove_var("MONITORING_INTERVAL");
}

/// Test environment variable with Unicode characters
#[test]
fn test_unicode_env_values() {
    // Test Unicode numeric representations
    let unicode_values = vec![
        "１０", // Full-width digits
        "十",   // Chinese character for 10
        "十",   // Japanese character for 10
        "١٠",   // Arabic-Indic digits
        "Ⅹ",    // Roman numeral
        "十",   // Korean character for 10
    ];

    for value in unicode_values {
        env::set_var("MONITORING_INTERVAL", value);
        let config = HiveConfig::from_env().unwrap();
        assert_eq!(
            config.monitoring.monitoring_interval_secs, 5,
            "Failed for Unicode value: {}",
            value
        ); // Should use default
    }

    // Cleanup
    env::remove_var("MONITORING_INTERVAL");
}

/// Test rapid environment variable changes
#[test]
fn test_rapid_env_changes() {
    for i in 0..100 {
        env::set_var("MONITORING_INTERVAL", i.to_string());
        let config = HiveConfig::from_env().unwrap();

        // Should either get the current value or default (due to parsing)
        assert!(
            config.monitoring.monitoring_interval_secs == i as u64
                || config.monitoring.monitoring_interval_secs == 5
        );
    }

    // Cleanup
    env::remove_var("MONITORING_INTERVAL");
}

/// Test environment variable case sensitivity
#[test]
fn test_env_var_case_sensitivity() {
    // Test different cases
    env::set_var("monitoring_interval", "10");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, 5); // Should use default (wrong case)

    env::set_var("MONITORING_INTERVAL", "10");
    let config = HiveConfig::from_env().unwrap();
    assert_eq!(config.monitoring.monitoring_interval_secs, 10); // Should accept (correct case)

    // Cleanup
    env::remove_var("monitoring_interval");
    env::remove_var("MONITORING_INTERVAL");
}

/// Test environment variable with leading/trailing whitespace
#[test]
fn test_whitespace_env_values() {
    // Test various whitespace combinations
    let whitespace_values = vec![
        " 10",
        "10 ",
        " 10 ",
        "\t10",
        "10\t",
        "\t10\t",
        "\n10",
        "10\n",
        "\n10\n",
        "\r10",
        "10\r",
        "\r10\r",
        " \t\n\r10 \t\n\r",
    ];

    for value in whitespace_values {
        env::set_var("MONITORING_INTERVAL", value);
        let config = HiveConfig::from_env().unwrap();
        assert_eq!(
            config.monitoring.monitoring_interval_secs, 5,
            "Failed for whitespace value: {:?}",
            value
        ); // Should use default
    }

    // Cleanup
    env::remove_var("MONITORING_INTERVAL");
}
