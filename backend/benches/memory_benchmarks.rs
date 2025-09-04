use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::sync::Arc;

/// Benchmark memory allocation performance
fn memory_allocation_benchmark(c: &mut Criterion) {
    c.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let mut allocations = Vec::new();

            // Allocate various sizes
            for size in [64, 256, 1024, 4096, 16384] {
                let data = vec![0u8; size];
                allocations.push(data);
            }

            black_box(allocations);
        });
    });
}

/// Benchmark heap vs stack allocation
fn heap_vs_stack_benchmark(c: &mut Criterion) {
    c.bench_function("heap_allocation", |b| {
        b.iter(|| {
            let mut heap_data = Vec::new();

            for _ in 0..1000 {
                heap_data.push(vec![0u8; 100]);
            }

            black_box(heap_data);
        });
    });

    c.bench_function("stack_allocation", |b| {
        b.iter(|| {
            let mut stack_data = [0u8; 100_000];

            for i in 0..100_000 {
                stack_data[i] = (i % 256) as u8;
            }

            black_box(stack_data);
        });
    });
}

/// Benchmark memory access patterns
fn memory_access_patterns_benchmark(c: &mut Criterion) {
    c.bench_function("sequential_memory_access", |b| {
        b.iter(|| {
            let size = 100_000;
            let mut data = vec![0u64; size];

            // Sequential access
            for i in 0..size {
                data[i] = i as u64 * 2;
            }

            let mut sum = 0u64;
            for &value in &data {
                sum = sum.wrapping_add(value);
            }

            black_box(sum);
        });
    });

    c.bench_function("random_memory_access", |b| {
        b.iter(|| {
            let size = 100_000;
            let mut data = vec![0u64; size];

            // Initialize
            for i in 0..size {
                data[i] = i as u64;
            }

            // Random access pattern
            let mut sum = 0u64;
            let mut index = 0usize;

            for _ in 0..size {
                sum = sum.wrapping_add(data[index]);
                index = ((index as u64)
                    .wrapping_mul(1_664_525)
                    .wrapping_add(1_013_904_223)) as usize
                    % size;
            }

            black_box(sum);
        });
    });
}

/// Benchmark reference counting performance
fn reference_counting_benchmark(c: &mut Criterion) {
    c.bench_function("arc_clone_performance", |b| {
        b.iter(|| {
            let original = Arc::new(vec![0u8; 1000]);
            let mut clones = Vec::new();

            for _ in 0..100 {
                clones.push(Arc::clone(&original));
            }

            black_box(clones);
        });
    });
}

/// Benchmark HashMap operations
fn hashmap_operations_benchmark(c: &mut Criterion) {
    c.bench_function("hashmap_insert", |b| {
        b.iter(|| {
            let mut map = HashMap::new();

            for i in 0..10_000 {
                map.insert(i, i * 2);
            }

            black_box(map);
        });
    });

    c.bench_function("hashmap_lookup", |b| {
        b.iter(|| {
            let mut map = HashMap::new();

            // Pre-populate
            for i in 0..10_000 {
                map.insert(i, i * 2);
            }

            let mut sum = 0;
            for i in 0..10_000 {
                if let Some(&value) = map.get(&i) {
                    sum += value;
                }
            }

            black_box(sum);
        });
    });
}

/// Benchmark string operations
fn string_operations_benchmark(c: &mut Criterion) {
    c.bench_function("string_concatenation", |b| {
        b.iter(|| {
            let mut result = String::new();

            for i in 0..1000 {
                result.push_str(&format!("item_{}", i));
            }

            black_box(result);
        });
    });

    c.bench_function("string_formatting", |b| {
        b.iter(|| {
            let mut results = Vec::new();

            for i in 0..1000 {
                let formatted = format!(
                    "Agent {} processing task {} at {}",
                    i,
                    i * 2,
                    chrono::Utc::now()
                );
                results.push(formatted);
            }

            black_box(results);
        });
    });
}

/// Benchmark vector operations
fn vector_operations_benchmark(c: &mut Criterion) {
    c.bench_function("vector_push", |b| {
        b.iter(|| {
            let mut vec = Vec::new();

            for i in 0..10_000 {
                vec.push(i);
            }

            black_box(vec);
        });
    });

    c.bench_function("vector_resize", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            vec.resize(10_000, 0);

            for i in 0..10_000 {
                vec[i] = i * 2;
            }

            black_box(vec);
        });
    });
}

/// Benchmark memory fragmentation
fn memory_fragmentation_benchmark(c: &mut Criterion) {
    c.bench_function("memory_fragmentation", |b| {
        b.iter(|| {
            let mut allocations = Vec::new();

            // Allocate and deallocate in a pattern that causes fragmentation
            for _ in 0..100 {
                let mut temp = Vec::new();

                for size in [100, 200, 50, 300, 25] {
                    temp.push(vec![0u8; size]);
                }

                // Keep some, drop others
                allocations.push(temp.remove(0));
                allocations.push(temp.remove(1));
                // temp drops here, causing fragmentation
            }

            black_box(allocations);
        });
    });
}

/// Benchmark garbage collection simulation
fn gc_simulation_benchmark(c: &mut Criterion) {
    c.bench_function("gc_simulation", |b| {
        b.iter(|| {
            let mut objects = Vec::new();

            // Simulate object creation and cleanup
            for i in 0..1000 {
                let obj = Arc::new(format!("object_{}", i));
                objects.push(obj);
            }

            // Simulate cleanup (dropping references)
            objects.clear();

            // Force some allocations to trigger cleanup
            for _ in 0..100 {
                let _temp = vec![0u8; 1000];
            }

            black_box(objects);
        });
    });
}

criterion_group!(
    benches,
    memory_allocation_benchmark,
    heap_vs_stack_benchmark,
    memory_access_patterns_benchmark,
    reference_counting_benchmark,
    hashmap_operations_benchmark,
    string_operations_benchmark,
    vector_operations_benchmark,
    memory_fragmentation_benchmark,
    gc_simulation_benchmark
);
criterion_main!(benches);
