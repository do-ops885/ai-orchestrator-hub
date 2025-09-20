//! Validation Test Module
//!
//! This module tests the parameter validation functions to see exact error messages.

use multiagent_hive::utils::validation::InputValidator;
use serde_json::json;

#[test]
fn test_agent_validation_errors() {
    println!("=== Testing Agent Parameter Validation ===\n");

    // Test 1: Valid agent payload
    println!("Test 1: Valid agent payload");
    let valid_agent = json!({
        "name": "TestAgent",
        "agent_type": "worker"
    });
    match InputValidator::validate_agent_payload(&valid_agent) {
        Ok(_) => println!("✅ Valid agent payload passed"),
        Err(e) => println!("❌ Unexpected error: {}", e),
    }
    println!();

    // Test 2: Missing name field
    println!("Test 2: Missing name field");
    let missing_name = json!({
        "agent_type": "worker"
    });
    match InputValidator::validate_agent_payload(&missing_name) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();

    // Test 3: Empty name
    println!("Test 3: Empty name");
    let empty_name = json!({
        "name": "",
        "agent_type": "worker"
    });
    match InputValidator::validate_agent_payload(&empty_name) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();

    // Test 4: Invalid agent type
    println!("Test 4: Invalid agent type");
    let invalid_type = json!({
        "name": "TestAgent",
        "agent_type": "invalid_type"
    });
    match InputValidator::validate_agent_payload(&invalid_type) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();

    // Test 5: Name too long
    println!("Test 5: Name too long");
    let long_name = json!({
        "name": "a".repeat(101),
        "agent_type": "worker"
    });
    match InputValidator::validate_agent_payload(&long_name) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();

    // Test 6: Invalid characters in name
    println!("Test 6: Invalid characters in name");
    let invalid_chars = json!({
        "name": "Test@Agent#123",
        "agent_type": "worker"
    });
    match InputValidator::validate_agent_payload(&invalid_chars) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();

    // Test 7: Invalid capability proficiency
    println!("Test 7: Invalid capability proficiency");
    let invalid_proficiency = json!({
        "name": "TestAgent",
        "agent_type": "worker",
        "capabilities": [{
            "name": "test_cap",
            "proficiency": 1.5
        }]
    });
    match InputValidator::validate_agent_payload(&invalid_proficiency) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();
}

#[test]
fn test_task_validation_errors() {
    println!("=== Testing Task Parameter Validation ===\n");

    // Test 1: Valid task payload
    println!("Test 1: Valid task payload");
    let valid_task = json!({
        "description": "Test task description"
    });
    match InputValidator::validate_task_payload(&valid_task) {
        Ok(_) => println!("✅ Valid task payload passed"),
        Err(e) => println!("❌ Unexpected error: {}", e),
    }
    println!();

    // Test 2: Missing description
    println!("Test 2: Missing description");
    let missing_desc = json!({});
    match InputValidator::validate_task_payload(&missing_desc) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();

    // Test 3: Empty description
    println!("Test 3: Empty description");
    let empty_desc = json!({
        "description": ""
    });
    match InputValidator::validate_task_payload(&empty_desc) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();

    // Test 4: Invalid priority
    println!("Test 4: Invalid priority");
    let invalid_priority = json!({
        "description": "Test task",
        "priority": "invalid"
    });
    match InputValidator::validate_task_payload(&invalid_priority) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();

    // Test 5: Description too long
    println!("Test 5: Description too long");
    let long_desc = json!({
        "description": "a".repeat(1001)
    });
    match InputValidator::validate_task_payload(&long_desc) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();

    // Test 6: Invalid minimum proficiency
    println!("Test 6: Invalid minimum proficiency");
    let invalid_min_prof = json!({
        "description": "Test task",
        "required_capabilities": [{
            "name": "test_cap",
            "minimum_proficiency": 1.5
        }]
    });
    match InputValidator::validate_task_payload(&invalid_min_prof) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();
}

#[test]
fn test_uuid_validation() {
    println!("=== Testing UUID Validation ===\n");

    // Test 1: Valid UUID
    println!("Test 1: Valid UUID");
    let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
    match InputValidator::validate_uuid(valid_uuid) {
        Ok(uuid) => println!("✅ Valid UUID: {}", uuid),
        Err(e) => println!("❌ Unexpected error: {}", e),
    }
    println!();

    // Test 2: Invalid UUID
    println!("Test 2: Invalid UUID");
    let invalid_uuid = "not-a-uuid";
    match InputValidator::validate_uuid(invalid_uuid) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();
}

#[test]
fn test_resource_limits_validation() {
    println!("=== Testing Resource Limits Validation ===\n");

    // Test 1: Valid resource limits
    println!("Test 1: Valid resource limits");
    match InputValidator::validate_resource_limits(50.0, 60.0) {
        Ok(_) => println!("✅ Valid resource limits passed"),
        Err(e) => println!("❌ Unexpected error: {}", e),
    }
    println!();

    // Test 2: CPU too high
    println!("Test 2: CPU too high");
    match InputValidator::validate_resource_limits(150.0, 60.0) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();

    // Test 3: Memory too high
    println!("Test 3: Memory too high");
    match InputValidator::validate_resource_limits(50.0, 150.0) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();

    // Test 4: Negative values
    println!("Test 4: Negative values");
    match InputValidator::validate_resource_limits(-10.0, -20.0) {
        Ok(_) => println!("❌ Should have failed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }
    println!();
}
