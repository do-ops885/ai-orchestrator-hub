use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Benchmark CPU-intensive computation
fn cpu_intensive_computation_benchmark(c: &mut Criterion) {
    c.bench_function("cpu_intensive_computation", |b| {
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

/// Benchmark matrix operations (common in neural networks)
fn matrix_operations_benchmark(c: &mut Criterion) {
    c.bench_function("matrix_operations", |b| {
        b.iter(|| {
            let size = 50;
            let mut matrix_a = vec![vec![0.0; size]; size];
            let mut matrix_b = vec![vec![0.0; size]; size];
            let mut result = vec![vec![0.0; size]; size];

            // Initialize matrices
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

/// Benchmark vector operations
fn vector_operations_benchmark(c: &mut Criterion) {
    c.bench_function("vector_operations", |b| {
        b.iter(|| {
            let size = 100_000;
            let mut vec_a = vec![0.0; size];
            let mut vec_b = vec![0.0; size];
            let mut result = vec![0.0; size];

            // Initialize vectors
            for i in 0..size {
                vec_a[i] = (i as f64).sin();
                vec_b[i] = (i as f64).cos();
            }

            // Vector operations
            for i in 0..size {
                result[i] = vec_a[i] * vec_b[i] + vec_a[i].sin() + vec_b[i].cos();
            }

            black_box(result);
        });
    });
}

/// Benchmark concurrent CPU operations
fn concurrent_cpu_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("concurrent_cpu_operations", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = vec![];

                for i in 0..num_cpus::get() {
                    let handle = tokio::spawn(async move {
                        let mut result = 0u64;
                        let start = i * 100_000;
                        let end = start + 100_000;

                        for j in start..end {
                            result = result.wrapping_add(j as u64);
                            result = result.wrapping_mul(17);
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

/// Benchmark CPU cache performance
fn cpu_cache_benchmark(c: &mut Criterion) {
    c.bench_function("cpu_cache_performance", |b| {
        b.iter(|| {
            let size = 10_000;
            let mut data = vec![0u64; size];

            // Sequential access (good cache performance)
            for i in 0..size {
                data[i] = i as u64 * 2;
            }

            // Random access (poor cache performance)
            let mut sum = 0u64;
            for _ in 0..size {
                let index = (sum % size as u64) as usize;
                sum = sum.wrapping_add(data[index]);
            }

            black_box(sum);
        });
    });
}

/// Benchmark branch prediction
fn branch_prediction_benchmark(c: &mut Criterion) {
    c.bench_function("branch_prediction_good", |b| {
        b.iter(|| {
            let data = vec![0i32; 10_000];
            let mut sum = 0;

            // Predictable branches
            for &value in &data {
                if value >= 0 {
                    sum += value;
                } else {
                    sum -= value;
                }
            }

            black_box(sum);
        });
    });
}

/// Benchmark memory bandwidth
fn memory_bandwidth_benchmark(c: &mut Criterion) {
    c.bench_function("memory_bandwidth", |b| {
        b.iter(|| {
            let size = 1_000_000;
            let mut data = vec![0u64; size];

            // Write bandwidth
            for i in 0..size {
                data[i] = i as u64;
            }

            // Read bandwidth
            let mut sum = 0u64;
            for &value in &data {
                sum = sum.wrapping_add(value);
            }

            black_box(sum);
        });
    });
}

/// Benchmark SIMD operations (if available)
fn simd_operations_benchmark(c: &mut Criterion) {
    c.bench_function("simd_operations", |b| {
        b.iter(|| {
            let size = 100_000;
            let mut vec_a = vec![0.0f32; size];
            let mut vec_b = vec![0.0f32; size];
            let mut result = vec![0.0f32; size];

            // Initialize
            for i in 0..size {
                vec_a[i] = i as f32;
                vec_b[i] = (i * 2) as f32;
            }

            // SIMD-friendly operations
            for i in 0..size {
                result[i] = vec_a[i] * vec_b[i] + vec_a[i] + vec_b[i];
            }

            black_box(result);
        });
    });
}

criterion_group!(
    benches,
    cpu_intensive_computation_benchmark,
    matrix_operations_benchmark,
    vector_operations_benchmark,
    concurrent_cpu_operations_benchmark,
    cpu_cache_benchmark,
    branch_prediction_benchmark,
    memory_bandwidth_benchmark,
    simd_operations_benchmark
);
criterion_main!(benches);
