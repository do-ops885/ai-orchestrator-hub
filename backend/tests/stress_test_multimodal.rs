//! # Multi-Modal Agent Stress Tests
//!
//! High-load tests to validate system behavior under extreme conditions.

use multiagent_hive::agents::{DataModality, MultiModalAgent};
use multiagent_hive::neural::NLPProcessor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;

#[tokio::test]
async fn test_massive_concurrent_load() {
    let concurrent_agents = 50;
    let analyses_per_agent = 20;
    let total_analyses = concurrent_agents * analyses_per_agent;

    let completed_count = Arc::new(AtomicUsize::new(0));
    let failed_count = Arc::new(AtomicUsize::new(0));

    let start_time = Instant::now();
    let mut handles = Vec::new();

    for agent_id in 0..concurrent_agents {
        let completed = completed_count.clone();
        let failed = failed_count.clone();

        let handle = tokio::spawn(async move {
            match timeout(Duration::from_secs(60), async {
                let nlp_processor = NLPProcessor::new().await?;
                let mut agent = MultiModalAgent::new(
                    format!("StressAgent{}", agent_id),
                    Some(nlp_processor)
                ).await?;

                for analysis_id in 0..analyses_per_agent {
                    let content = format!(
                        "Stress test content for agent {} analysis {}. This is a comprehensive test of system performance under high load conditions.",
                        agent_id, analysis_id
                    );
                    match agent.analyze_multimodal_data(&content).await {
                        Ok(_) => {
                            completed.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(_) => {
                            failed.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            }).await {
                Ok(Ok(())) => {},
                Ok(Err(e)) => {
                    eprintln!("Agent {} failed: {}", agent_id, e);
                    failed.fetch_add(analyses_per_agent, Ordering::Relaxed);
                }
                Err(_) => {
                    eprintln!("Agent {} timed out", agent_id);
                    failed.fetch_add(analyses_per_agent, Ordering::Relaxed);
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all agents to complete
    futures::future::join_all(handles).await;

    let total_time = start_time.elapsed();
    let completed = completed_count.load(Ordering::Relaxed);
    let failed = failed_count.load(Ordering::Relaxed);

    println!("Stress test completed in {:?}", total_time);
    println!("Total analyses: {}", total_analyses);
    println!("Completed: {}", completed);
    println!("Failed: {}", failed);
    println!(
        "Success rate: {:.2}%",
        (completed as f64 / total_analyses as f64) * 100.0
    );
    println!(
        "Throughput: {:.2} analyses/second",
        completed as f64 / total_time.as_secs_f64()
    );

    // Assertions for stress test
    assert!(completed > 0, "At least some analyses should complete");
    assert!(
        (completed as f64 / total_analyses as f64) > 0.8,
        "Success rate should be > 80%"
    );
    assert!(
        total_time < Duration::from_secs(120),
        "Should complete within 2 minutes"
    );
}

#[tokio::test]
async fn test_memory_stress() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("MemoryStressTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    // Generate increasingly large content
    for size_multiplier in 1..=20 {
        let base_content = "This is a memory stress test with increasingly large content. "
            .repeat(size_multiplier * 100);

        let mixed_content = format!(
            r#"
# Memory Stress Test - Iteration {}

This document contains large amounts of text to test memory usage.

{}

```python
def memory_intensive_function():
    # Large data structure
    data = {}
    
    for i in range(10000):
        data[f"key_{{i}}"] = "value_" * 100
    
    return data

# Process the data
result = memory_intensive_function()
print(f"Processed {{len(result)}} items")
```

{{
    "large_array": {},
    "metadata": {{
        "size": {},
        "generated_for": "memory_stress_test"
    }}
}}
            "#,
            size_multiplier,
            base_content,
            (0..size_multiplier * 10)
                .map(|i| format!(r#"{{"id": {}, "data": "{}"}}"#, i, "x".repeat(50)))
                .collect::<Vec<_>>()
                .join(","),
            size_multiplier * 1000
        );

        let start = Instant::now();
        let result = timeout(
            Duration::from_secs(30),
            agent.analyze_multimodal_data(&mixed_content),
        )
        .await;

        match result {
            Ok(Ok(analysis)) => {
                let duration = start.elapsed();
                println!(
                    "Size multiplier {}: completed in {:?}, quality: {:.2}",
                    size_multiplier, duration, analysis.overall_quality
                );

                assert!(analysis.overall_quality > 0.0);
                assert!(duration < Duration::from_secs(20)); // Should handle reasonably
            }
            Ok(Err(e)) => {
                panic!(
                    "Analysis failed at size multiplier {}: {}",
                    size_multiplier, e
                );
            }
            Err(_) => {
                panic!("Analysis timed out at size multiplier {}", size_multiplier);
            }
        }
    }

    // Verify agent is still functional after stress
    let simple_result = agent
        .analyze_multimodal_data("Simple test after stress")
        .await
        .unwrap();
    assert!(simple_result.overall_quality > 0.0);
}

#[tokio::test]
async fn test_rapid_sequential_analysis() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("RapidTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    let iterations = 1000;
    let start_time = Instant::now();

    for i in 0..iterations {
        let content = format!("Rapid analysis test iteration {} with unique content.", i);

        let result = timeout(
            Duration::from_secs(5),
            agent.analyze_multimodal_data(&content),
        )
        .await;

        match result {
            Ok(Ok(analysis)) => {
                assert!(analysis.overall_quality > 0.0);
            }
            Ok(Err(e)) => {
                panic!("Analysis failed at iteration {}: {}", i, e);
            }
            Err(_) => {
                panic!("Analysis timed out at iteration {}", i);
            }
        }

        // Log progress every 100 iterations
        if i % 100 == 0 && i > 0 {
            println!("Completed {} iterations in {:?}", i, start_time.elapsed());
        }
    }

    let total_time = start_time.elapsed();
    let avg_time_per_analysis = total_time / iterations;

    println!(
        "Rapid test completed: {} iterations in {:?}",
        iterations, total_time
    );
    println!("Average time per analysis: {:?}", avg_time_per_analysis);

    assert!(
        avg_time_per_analysis < Duration::from_millis(100),
        "Average analysis time should be under 100ms for simple content"
    );
}

#[tokio::test]
async fn test_error_recovery_stress() {
    let nlp_processor = NLPProcessor::new().await.unwrap();
    let mut agent = MultiModalAgent::new("ErrorRecoveryTest".to_string(), Some(nlp_processor))
        .await
        .unwrap();

    // Mix of valid and potentially problematic inputs
    let test_inputs = vec![
        "Valid simple text",
        "",                                // Empty
        "x".repeat(100000),                // Very long
        "🚀".repeat(1000),                 // Many unicode characters
        r#"{"malformed": json"#,           // Malformed JSON
        "function test() { /* incomplete", // Incomplete code
        "\0\0\0null bytes\0\0\0",          // Null bytes
        "Valid text after problems",
        "More normal content",
        "Final validation text",
    ];

    let mut success_count = 0;
    let mut error_count = 0;

    for (i, input) in test_inputs.iter().enumerate() {
        match agent.analyze_multimodal_data(input).await {
            Ok(result) => {
                success_count += 1;
                assert!(result.overall_quality >= 0.0);
                println!(
                    "Input {} succeeded with quality {:.2}",
                    i, result.overall_quality
                );
            }
            Err(e) => {
                error_count += 1;
                println!("Input {} failed: {}", i, e);
            }
        }
    }

    println!(
        "Error recovery test: {} successes, {} errors",
        success_count, error_count
    );

    // Should handle most inputs gracefully
    assert!(
        success_count >= test_inputs.len() / 2,
        "Should handle at least 50% of inputs"
    );

    // Verify agent is still functional after error stress
    let final_result = agent
        .analyze_multimodal_data("Final functionality test")
        .await
        .unwrap();
    assert!(final_result.overall_quality > 0.0);
}
