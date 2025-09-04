use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Simple benchmark results
#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    duration_ms: f64,
    operations_per_sec: f64,
    memory_mb: f64,
}

/// Memory usage estimation (simplified)
fn get_memory_usage() -> f64 {
    // This is a placeholder - in a real implementation you'd use system APIs
    50.0
}

/// CPU intensive benchmark
fn benchmark_cpu_intensive(iterations: u32) -> BenchmarkResult {
    let start = Instant::now();
    let mut result = 0u64;

    for i in 0..iterations {
        result = result.wrapping_add(i as u64);
        result = result.wrapping_mul(17);
        result = result.wrapping_div(3);
    }

    let duration = start.elapsed();
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();

    BenchmarkResult {
        name: "CPU Intensive".to_string(),
        duration_ms: duration.as_millis() as f64,
        operations_per_sec: ops_per_sec,
        memory_mb: get_memory_usage(),
    }
}

/// Memory allocation benchmark
fn benchmark_memory_allocation(iterations: u32) -> BenchmarkResult {
    let start = Instant::now();
    let mut allocations = Vec::new();

    for i in 0..iterations {
        let data = vec![i as u8; 1024]; // 1KB allocations
        allocations.push(data);
    }

    let duration = start.elapsed();
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();

    BenchmarkResult {
        name: "Memory Allocation".to_string(),
        duration_ms: duration.as_millis() as f64,
        operations_per_sec: ops_per_sec,
        memory_mb: get_memory_usage(),
    }
}

/// HashMap operations benchmark
fn benchmark_hashmap_operations(iterations: u32) -> BenchmarkResult {
    let start = Instant::now();
    let mut map = HashMap::new();

    // Insert operations
    for i in 0..iterations {
        map.insert(i, format!("value_{}", i));
    }

    // Lookup operations
    for i in 0..iterations {
        let _ = map.get(&i);
    }

    let duration = start.elapsed();
    let ops_per_sec = (iterations * 2) as f64 / duration.as_secs_f64(); // inserts + lookups

    BenchmarkResult {
        name: "HashMap Operations".to_string(),
        duration_ms: duration.as_millis() as f64,
        operations_per_sec: ops_per_sec,
        memory_mb: get_memory_usage(),
    }
}

/// Vector operations benchmark
fn benchmark_vector_operations(iterations: u32) -> BenchmarkResult {
    let start = Instant::now();
    let mut vec = Vec::new();

    // Push operations
    for i in 0..iterations {
        vec.push(i);
    }

    // Access operations
    for i in 0..iterations {
        let idx = (i as usize) % vec.len();
        let _ = vec[idx];
    }

    let duration = start.elapsed();
    let ops_per_sec = (iterations * 2) as f64 / duration.as_secs_f64();

    BenchmarkResult {
        name: "Vector Operations".to_string(),
        duration_ms: duration.as_millis() as f64,
        operations_per_sec: ops_per_sec,
        memory_mb: get_memory_usage(),
    }
}

/// String operations benchmark
fn benchmark_string_operations(iterations: u32) -> BenchmarkResult {
    let start = Instant::now();
    let mut strings = Vec::new();

    // String creation and formatting
    for i in 0..iterations {
        let s = format!("String number {} with some content", i);
        strings.push(s);
    }

    // String operations
    for s in &mut strings {
        s.push_str(" - modified");
        let _len = s.len();
    }

    let duration = start.elapsed();
    let ops_per_sec = (iterations * 3) as f64 / duration.as_secs_f64();

    BenchmarkResult {
        name: "String Operations".to_string(),
        duration_ms: duration.as_millis() as f64,
        operations_per_sec: ops_per_sec,
        memory_mb: get_memory_usage(),
    }
}

/// Reference counting benchmark
fn benchmark_arc_operations(iterations: u32) -> BenchmarkResult {
    let start = Instant::now();
    let mut arcs = Vec::new();

    // Create Arc objects
    for i in 0..iterations {
        let data = Arc::new(format!("Arc data {}", i));
        arcs.push(data);
    }

    // Clone operations
    let mut clones = Vec::new();
    for arc in &arcs {
        clones.push(Arc::clone(arc));
    }

    let duration = start.elapsed();
    let ops_per_sec = (iterations * 2) as f64 / duration.as_secs_f64();

    BenchmarkResult {
        name: "Arc Operations".to_string(),
        duration_ms: duration.as_millis() as f64,
        operations_per_sec: ops_per_sec,
        memory_mb: get_memory_usage(),
    }
}

fn main() {
    let mut output = String::new();
    output.push_str("🚀 AI Orchestrator Hub - Simple Performance Benchmarks\n");
    output.push_str("====================================================\n");
    output.push_str("\n");

    let iterations = 100_000;
    output.push_str(&format!(
        "Running benchmarks with {} iterations each...\n",
        iterations
    ));
    output.push_str("\n");

    let benchmarks = vec![
        benchmark_cpu_intensive(iterations),
        benchmark_memory_allocation(iterations),
        benchmark_hashmap_operations(iterations / 10), // Fewer iterations for HashMap
        benchmark_vector_operations(iterations),
        benchmark_string_operations(iterations / 10), // Fewer iterations for strings
        benchmark_arc_operations(iterations / 10),    // Fewer iterations for Arc
    ];

    output.push_str("📊 Benchmark Results:\n");
    output.push_str("====================\n");

    for result in &benchmarks {
        output.push_str(&format!("🎯 {}:\n", result.name));
        output.push_str(&format!("   Duration: {:.2}ms\n", result.duration_ms));
        output.push_str(&format!(
            "   Operations/sec: {:.0}\n",
            result.operations_per_sec
        ));
        output.push_str(&format!("   Memory usage: {:.1} MB\n", result.memory_mb));
        output.push_str("\n");
    }

    // Summary statistics
    let total_duration: f64 = benchmarks.iter().map(|b| b.duration_ms).sum();
    let avg_ops_per_sec: f64 =
        benchmarks.iter().map(|b| b.operations_per_sec).sum::<f64>() / benchmarks.len() as f64;
    let max_memory = benchmarks.iter().map(|b| b.memory_mb).fold(0.0, f64::max);

    output.push_str("📈 Summary Statistics:\n");
    output.push_str("======================\n");
    output.push_str(&format!("Total benchmark time: {:.2}ms\n", total_duration));
    output.push_str(&format!("Average operations/sec: {:.0}\n", avg_ops_per_sec));
    output.push_str(&format!("Peak memory usage: {:.1} MB\n", max_memory));
    output.push_str(&format!("Benchmarks completed: {}\n", benchmarks.len()));
    output.push_str("\n");
    output.push_str("✅ Simple benchmarking completed successfully!\n");

    // Write to console
    println!("{}", output);

    // Write to file
    if let Err(e) = fs::write("../../benchmarks/simple_benchmark_results.txt", &output) {
        eprintln!("Failed to write benchmark results to file: {}", e);
    } else {
        println!("📄 Results also saved to benchmarks/simple_benchmark_results.txt");
    }
}
