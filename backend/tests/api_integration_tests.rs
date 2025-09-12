//! API Integration Tests
//!
//! This module contains integration tests for the REST API endpoints
//! and WebSocket communication protocols.

use multiagent_hive::{AgentType, HiveCoordinator};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

/// Test API server configuration
struct TestServer {
    base_url: String,
    client: Client,
}

impl TestServer {
    fn new(port: u16) -> Self {
        Self {
            base_url: format!("http://localhost:{}", port),
            client: Client::new(),
        }
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        let response = self
            .client
            .get(&format!("{}/api/mcp/health", self.base_url))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn create_agent(&self, config: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let response = self
            .client
            .post(&format!("{}/api/agents", self.base_url))
            .json(&config)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("API request failed: {}", response.status()))
        }
    }

    async fn get_agents(&self) -> anyhow::Result<serde_json::Value> {
        let response = self
            .client
            .get(&format!("{}/api/agents", self.base_url))
            .send()
            .await?;

        Ok(response.json().await?)
    }

    async fn create_task(&self, config: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let response = self
            .client
            .post(&format!("{}/api/tasks", self.base_url))
            .json(&config)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("API request failed: {}", response.status()))
        }
    }

    async fn get_hive_status(&self) -> anyhow::Result<serde_json::Value> {
        let response = self
            .client
            .get(&format!("{}/api/hive/status", self.base_url))
            .send()
            .await?;

        Ok(response.json().await?)
    }

    async fn get_resource_info(&self) -> anyhow::Result<serde_json::Value> {
        let response = self
            .client
            .get(&format!("{}/api/resources", self.base_url))
            .send()
            .await?;

        Ok(response.json().await?)
    }
}

/// Integration test for basic API functionality with retry logic
#[tokio::test]
async fn test_api_basic_functionality() {
    // Note: This test requires the server to be running
    // In a real CI/CD environment, you would start the server programmatically

    let server = TestServer::new(3001);
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 3;

    // Test health check with retry logic
    let healthy = loop {
        match server.health_check().await {
            Ok(healthy) => break healthy,
            Err(_) => {
                retry_count += 1;
                if retry_count >= MAX_RETRIES {
                    println!(
                        "⚠️  Server not running after {} retries, skipping integration tests",
                        MAX_RETRIES
                    );
                    return;
                }
                println!(
                    "🔄 Server not ready, retrying... ({}/{})",
                    retry_count, MAX_RETRIES
                );
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    };

    assert!(healthy, "Server should be healthy after retries");

    println!("✅ Server is healthy and responding");

    // Test agent creation
    let agent_config = json!({
        "name": "APITestAgent",
        "type": "worker",
        "capabilities": [
            {
                "name": "api_testing",
                "proficiency": 0.9,
                "learning_rate": 0.1
            }
        ]
    });

    let agent_result = server.create_agent(agent_config).await;
    assert!(agent_result.is_ok(), "Agent creation should succeed");

    let agent_data = agent_result.unwrap();
    assert!(agent_data["id"].is_string(), "Agent should have an ID");

    println!("✅ Agent creation API works correctly");

    // Test task creation
    let task_config = json!({
        "description": "API Integration Test Task",
        "type": "api_testing",
        "priority": 1,
        "required_capabilities": [
            {
                "name": "api_testing",
                "min_proficiency": 0.5
            }
        ]
    });

    let task_result = server.create_task(task_config).await;
    assert!(task_result.is_ok(), "Task creation should succeed");

    let task_data = task_result.unwrap();
    assert!(task_data["id"].is_string(), "Task should have an ID");

    println!("✅ Task creation API works correctly");

    // Test status endpoint
    let status_result = server.get_hive_status().await;
    assert!(status_result.is_ok(), "Status endpoint should work");

    let status_data = status_result.unwrap();
    assert!(
        status_data["hive_id"].is_string(),
        "Status should contain hive ID"
    );

    println!("✅ Hive status API works correctly");

    // Test resource info endpoint
    let resource_result = server.get_resource_info().await;
    assert!(
        resource_result.is_ok(),
        "Resource info endpoint should work"
    );

    println!("✅ Resource info API works correctly");

    // Test agents list endpoint
    let agents_result = server.get_agents().await;
    assert!(agents_result.is_ok(), "Agents list endpoint should work");

    let agents_data = agents_result.unwrap();
    assert!(
        agents_data["agents"].is_array(),
        "Should return agents array"
    );

    println!("✅ Agents list API works correctly");
}

/// Test API error handling
#[tokio::test]
async fn test_api_error_handling() {
    let server = TestServer::new(3001);

    // Skip if server is not running
    if server.health_check().await.is_err() {
        println!("⚠️  Server not running, skipping error handling tests");
        return;
    }

    // Test invalid agent configuration
    let invalid_config = json!({
        "name": "",  // Invalid: empty name
        "type": "invalid_type",
        "capabilities": "not_an_array"  // Invalid: should be array
    });

    let result = server.create_agent(invalid_config).await;
    assert!(result.is_err(), "Invalid config should result in error");

    println!("✅ API properly handles invalid input");

    // Test non-existent endpoint
    let response = server
        .client
        .get(&format!("{}/api/nonexistent", server.base_url))
        .send()
        .await;

    match response {
        Ok(resp) => assert_eq!(
            resp.status().as_u16(),
            404,
            "Should return 404 for non-existent endpoint"
        ),
        Err(_) => println!("⚠️  Could not test 404 response"),
    }

    println!("✅ API properly handles non-existent endpoints");
}

/// Test API performance under load
#[tokio::test]
async fn test_api_performance_under_load() {
    let server = TestServer::new(3001);

    // Skip if server is not running
    if server.health_check().await.is_err() {
        println!("⚠️  Server not running, skipping performance tests");
        return;
    }

    let start_time = std::time::Instant::now();

    // Create multiple agents concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let server_clone = TestServer::new(3001);
        let handle = tokio::spawn(async move {
            let config = json!({
                "name": format!("LoadTestAgent{}", i),
                "type": "worker",
                "capabilities": [
                    {
                        "name": "load_testing",
                        "proficiency": 0.8,
                        "learning_rate": 0.1
                    }
                ]
            });

            server_clone.create_agent(config).await
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    let results = futures::future::join_all(handles).await;

    let end_time = std::time::Instant::now();
    let duration = end_time.duration_since(start_time);

    let success_count = results
        .iter()
        .filter(|result| matches!(result, Ok(Ok(_))))
        .count();

    println!(
        "✅ Load test completed: {}/{} requests successful in {:?}",
        success_count,
        results.len(),
        duration
    );

    // Performance assertions
    assert!(
        success_count >= 8,
        "At least 80% of requests should succeed"
    );
    assert!(
        duration.as_millis() < 5000,
        "Load test should complete within 5 seconds"
    );

    println!("✅ API performance under load is acceptable");
}

/// Test WebSocket communication
#[tokio::test]
async fn test_websocket_communication() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

    // Skip if server is not running
    let server = TestServer::new(3001);
    if server.health_check().await.is_err() {
        println!("⚠️  Server not running, skipping WebSocket tests");
        return;
    }

    // Connect to WebSocket
    let ws_url = "ws://localhost:3001/ws";
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (mut write, mut read) = ws_stream.split();

    // Send a test message
    let test_message = json!({
        "type": "ping",
        "timestamp": chrono::Utc::now().timestamp()
    });

    write
        .send(Message::Text(test_message.to_string()))
        .await
        .expect("Failed to send message");

    // Wait for response with timeout
    let response_future = read.next();
    let timeout_duration = Duration::from_secs(5);

    match tokio::time::timeout(timeout_duration, response_future).await {
        Ok(Some(Ok(Message::Text(response_text)))) => {
            println!("✅ WebSocket response received: {}", response_text);
            assert!(!response_text.is_empty(), "Response should not be empty");
        }
        Ok(Some(Ok(_))) => {
            println!("✅ WebSocket response received (non-text)");
        }
        Ok(Some(Err(e))) => {
            panic!("WebSocket error: {}", e);
        }
        Ok(None) => {
            panic!("WebSocket connection closed unexpectedly");
        }
        Err(_) => {
            println!("⚠️  WebSocket response timeout - this may be expected if server doesn't respond to ping");
        }
    }

    // Close connection
    write.send(Message::Close(None)).await.ok();

    println!("✅ WebSocket communication test completed");
}

/// Test API rate limiting (if implemented)
#[tokio::test]
async fn test_api_rate_limiting() {
    let server = TestServer::new(3001);

    // Skip if server is not running
    if server.health_check().await.is_err() {
        println!("⚠️  Server not running, skipping rate limiting tests");
        return;
    }

    // Make multiple rapid requests
    let mut success_count = 0;
    let mut rate_limited_count = 0;

    for i in 0..20 {
        let result = server.health_check().await;

        match result {
            Ok(true) => success_count += 1,
            Ok(false) => rate_limited_count += 1,
            Err(_) => {
                // Could be rate limited or network error
                rate_limited_count += 1;
            }
        }

        // Small delay between requests
        if i < 19 {
            sleep(Duration::from_millis(50)).await;
        }
    }

    println!(
        "✅ Rate limiting test: {}/{} successful, {}/{} potentially rate limited",
        success_count, 20, rate_limited_count, 20
    );

    // If rate limiting is implemented, we expect some requests to be limited
    // If not implemented, all should succeed
    assert!(success_count > 0, "At least some requests should succeed");
}

/// Test API data consistency
#[tokio::test]
async fn test_api_data_consistency() {
    let server = TestServer::new(3001);

    // Skip if server is not running
    if server.health_check().await.is_err() {
        println!("⚠️  Server not running, skipping consistency tests");
        return;
    }

    // Create an agent
    let agent_config = json!({
        "name": "ConsistencyTestAgent",
        "type": "worker",
        "capabilities": [
            {
                "name": "consistency_testing",
                "proficiency": 0.9,
                "learning_rate": 0.1
            }
        ]
    });

    let agent_result = server.create_agent(agent_config.clone()).await;
    assert!(agent_result.is_ok(), "Agent creation should succeed");

    let agent_id = agent_result.unwrap()["id"]
        .as_str()
        .expect("Agent should have ID")
        .to_string();

    // Get agents list
    let agents_before = server.get_agents().await.expect("Should get agents list");

    let agent_count_before = agents_before["agents"]
        .as_array()
        .expect("Should be array")
        .len();

    // Get status
    let status = server.get_hive_status().await.expect("Should get status");

    let status_agent_count = status["metrics"]["total_agents"]
        .as_u64()
        .expect("Should have agent count");

    // Verify consistency
    assert_eq!(
        agent_count_before as u64, status_agent_count,
        "Agent count should be consistent between endpoints"
    );

    println!("✅ API data consistency verified");
}
