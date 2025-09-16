use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::sync::Arc;

/// Benchmark CPU-intensive computation
fn cpu_intensive_benchmark(c: &mut Criterion) {
    c.bench_function("cpu_intensive", |b| {
        b.iter(|| {
            let mut result = 0u64;
            for i in 0..1_000_000 {
                result = result.wrapping_add(i);
                result = result.wrapping_mul(17);
                result = result.wrapping_div(3);
            }
            black_box(result);
        });
    });
}

/// Benchmark memory allocation patterns
fn memory_allocation_benchmark(c: &mut Criterion) {
    c.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let mut allocations = Vec::new();
            for _ in 0..1000 {
                allocations.push(vec![0u8; 1024]); // 1KB allocations
            }
            black_box(allocations);
        });
    });
}

/// Benchmark HashMap operations
fn hashmap_benchmark(c: &mut Criterion) {
    c.bench_function("hashmap_operations", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..10_000 {
                map.insert(i, i * 2);
            }
            for i in 0..10_000 {
                let _ = map.get(&i);
            }
            black_box(map);
        });
    });
}

/// Benchmark vector operations
fn vector_benchmark(c: &mut Criterion) {
    c.bench_function("vector_operations", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..10_000 {
                vec.push(i);
            }
            for i in 0..10_000 {
                let _ = vec[i % vec.len()];
            }
            black_box(vec);
        });
    });
}

/// Benchmark string operations
fn string_benchmark(c: &mut Criterion) {
    c.bench_function("string_operations", |b| {
        b.iter(|| {
            let mut strings = Vec::new();
            for i in 0..1000 {
                strings.push(format!("String number {} with content", i));
            }
            for s in &mut strings {
                s.push_str(" - modified");
            }
            black_box(strings);
        });
    });
}

/// Benchmark Arc operations
fn arc_benchmark(c: &mut Criterion) {
    c.bench_function("arc_operations", |b| {
        b.iter(|| {
            let mut arcs = Vec::new();
            for i in 0..1000 {
                arcs.push(Arc::new(format!("Arc data {}", i)));
            }
            let mut clones = Vec::new();
            for arc in &arcs {
                clones.push(Arc::clone(arc));
            }
            black_box((arcs, clones));
        });
    });
}

/// Benchmark matrix operations (simulating neural network computations)
fn matrix_benchmark(c: &mut Criterion) {
    c.bench_function("matrix_operations", |b| {
        b.iter(|| {
            let size = 50;
            let mut matrix_a = vec![vec![0.0; size]; size];
            let mut matrix_b = vec![vec![0.0; size]; size];
            let mut result = vec![vec![0.0; size]; size];

            // Initialize
            for i in 0..size {
                for j in 0..size {
                    matrix_a[i][j] = (i * j) as f64;
                    matrix_b[i][j] = ((i + j) * 2) as f64;
                }
            }

            // Matrix multiplication
            for i in 0..size {
                for j in 0..size {
                    for k in 0..size {
                        result[i][j] += matrix_a[i][k] * matrix_b[k][j];
                    }
                }
            }

            black_box(result);
        });
    });
}

/// Benchmark concurrent operations
fn concurrent_benchmark(c: &mut Criterion) {
    c.bench_function("concurrent_operations", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut handles = vec![];

                for i in 0..4 {
                    let handle = tokio::spawn(async move {
                        let mut result = 0u64;
                        for j in 0..25_000 {
                            result = result.wrapping_add((i * j) as u64);
                        }
                        result
                    });
                    handles.push(handle);
                }

                let mut total = 0u64;
                for handle in handles {
                    total = total.wrapping_add(handle.await.unwrap());
                }

                black_box(total);
            });
        });
    });
}

/// Benchmark async file I/O operations (simulating optimized_agent.rs improvements)
fn async_file_io_benchmark(c: &mut Criterion) {
    c.bench_function("async_file_io_optimization", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Simulate the optimized file I/O pattern from optimized_agent.rs
                let result = tokio::task::spawn_blocking(|| {
                    // Simulate synchronous file reading (like /proc/meminfo)
                    let mut total = 0u64;
                    for i in 0..1000 {
                        total = total.wrapping_add(i);
                    }
                    total
                })
                .await
                .unwrap_or(0);

                black_box(result);
            });
        });
    });
}

/// Benchmark CPU-intensive computation moved to blocking threads
fn blocking_computation_benchmark(c: &mut Criterion) {
    c.bench_function("blocking_computation_optimization", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Simulate the pattern from adaptive_verification.rs
                // where heavy computation is moved to spawn_blocking
                let computations = vec![100, 200, 300, 400, 500];

                let mut handles = vec![];
                for &size in &computations {
                    let handle = tokio::task::spawn_blocking(move || {
                        let mut result = 0u64;
                        for i in 0..size {
                            result = result.wrapping_add(i);
                            result = result.wrapping_mul(17);
                        }
                        result
                    });
                    handles.push(handle);
                }

                let mut total = 0u64;
                for handle in handles {
                    total = total.wrapping_add(handle.await.unwrap_or(0));
                }

                black_box(total);
            });
        });
    });
}

criterion_group!(
    benches,
    cpu_intensive_benchmark,
    memory_allocation_benchmark,
    hashmap_benchmark,
    vector_benchmark,
    string_benchmark,
    arc_benchmark,
    matrix_benchmark,
    concurrent_benchmark,
    async_file_io_benchmark,
    blocking_computation_benchmark
);

fn main() {
    let mut criterion = Criterion::default().output_directory("../../benchmarks");

    benches(&mut criterion);
}
