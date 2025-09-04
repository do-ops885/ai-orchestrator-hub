use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use tempfile::NamedTempFile;
use tokio::runtime::Runtime;

/// Benchmark file creation
fn file_creation_benchmark(c: &mut Criterion) {
    c.bench_function("file_creation", |b| {
        b.iter(|| {
            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path().to_path_buf();

            // Write some data
            fs::write(&path, b"Hello, World!").unwrap();

            black_box(path);
        });
    });
}

/// Benchmark file reading
fn file_reading_benchmark(c: &mut Criterion) {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Create a test file with known content
    let test_data = vec![0u8; 1024 * 1024]; // 1MB
    fs::write(path, &test_data).unwrap();

    c.bench_function("file_reading", |b| {
        b.iter(|| {
            let data = fs::read(path).unwrap();
            black_box(data);
        });
    });
}

/// Benchmark buffered file I/O
fn buffered_io_benchmark(c: &mut Criterion) {
    c.bench_function("buffered_file_write", |b| {
        b.iter(|| {
            let temp_file = NamedTempFile::new().unwrap();
            let mut file = fs::File::create(temp_file.path()).unwrap();

            let data = vec![0u8; 64 * 1024]; // 64KB chunks

            for _ in 0..16 {
                file.write_all(&data).unwrap();
            }

            file.flush().unwrap();
            black_box(file);
        });
    });

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Pre-populate file
    {
        let mut file = fs::File::create(path).unwrap();
        let data = vec![0u8; 64 * 1024];

        for _ in 0..16 {
            file.write_all(&data).unwrap();
        }
    }

    c.bench_function("buffered_file_read", |b| {
        b.iter(|| {
            let mut file = fs::File::open(path).unwrap();
            let mut buffer = vec![0u8; 64 * 1024];
            let mut total_read = 0;

            loop {
                let bytes_read = file.read(&mut buffer).unwrap();
                if bytes_read == 0 {
                    break;
                }
                total_read += bytes_read;
            }

            black_box(total_read);
        });
    });
}

/// Benchmark small file operations (common in databases)
fn small_file_operations_benchmark(c: &mut Criterion) {
    c.bench_function("small_file_operations", |b| {
        b.iter(|| {
            let mut results = Vec::new();

            for i in 0..100 {
                let temp_file = NamedTempFile::new().unwrap();
                let path = temp_file.path();

                // Write small record
                let data = format!("record_{}: {}", i, "some data content");
                fs::write(path, data.as_bytes()).unwrap();

                // Read it back
                let read_data = fs::read_to_string(path).unwrap();
                results.push(read_data);
            }

            black_box(results);
        });
    });
}

/// Benchmark directory operations
fn directory_operations_benchmark(c: &mut Criterion) {
    c.bench_function("directory_listing", |b| {
        b.iter(|| {
            let temp_dir = tempfile::tempdir().unwrap();
            let dir_path = temp_dir.path();

            // Create test files
            for i in 0..100 {
                let file_path = dir_path.join(format!("file_{}.txt", i));
                fs::write(file_path, format!("content {}", i)).unwrap();
            }

            // List directory
            let entries = fs::read_dir(dir_path).unwrap();
            let count = entries.count();

            black_box(count);
        });
    });
}

/// Benchmark concurrent file operations
fn concurrent_file_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("concurrent_file_operations", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = vec![];

                for i in 0..10 {
                    let handle = tokio::spawn(async move {
                        let temp_file = NamedTempFile::new().unwrap();
                        let path = temp_file.path();

                        // Write data
                        let data = format!("Concurrent data from task {}", i);
                        tokio::fs::write(path, data.as_bytes()).await.unwrap();

                        // Read it back
                        let read_data = tokio::fs::read_to_string(path).await.unwrap();

                        read_data
                    });

                    handles.push(handle);
                }

                let mut results = Vec::new();
                for handle in handles {
                    results.push(handle.await.unwrap());
                }

                black_box(results);
            });
        });
    });
}

/// Benchmark file metadata operations
fn file_metadata_benchmark(c: &mut Criterion) {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Create test file
    fs::write(path, "test content").unwrap();

    c.bench_function("file_metadata", |b| {
        b.iter(|| {
            let metadata = fs::metadata(path).unwrap();
            let size = metadata.len();
            let modified = metadata.modified().unwrap();

            black_box((size, modified));
        });
    });
}

/// Benchmark file copying
fn file_copy_benchmark(c: &mut Criterion) {
    let temp_file = NamedTempFile::new().unwrap();
    let source_path = temp_file.path();

    // Create source file
    let data = vec![0u8; 1024 * 1024]; // 1MB
    fs::write(source_path, &data).unwrap();

    c.bench_function("file_copy", |b| {
        b.iter(|| {
            let dest_file = NamedTempFile::new().unwrap();
            let dest_path = dest_file.path();

            fs::copy(source_path, dest_path).unwrap();

            black_box(dest_path);
        });
    });
}

/// Benchmark file seeking
fn file_seeking_benchmark(c: &mut Criterion) {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    // Create test file
    let data = (0..1024 * 1024)
        .map(|i| (i % 256) as u8)
        .collect::<Vec<_>>();
    fs::write(path, &data).unwrap();

    c.bench_function("file_seeking", |b| {
        b.iter(|| {
            let mut file = fs::File::open(path).unwrap();
            let mut buffer = [0u8; 1024];
            let mut sum = 0u64;

            // Random seeks and reads
            for i in 0..100 {
                let pos = (i * 1024) % (data.len() - 1024);
                file.seek(std::io::SeekFrom::Start(pos as u64)).unwrap();
                file.read_exact(&mut buffer).unwrap();

                for &byte in &buffer {
                    sum += byte as u64;
                }
            }

            black_box(sum);
        });
    });
}

criterion_group!(
    benches,
    file_creation_benchmark,
    file_reading_benchmark,
    buffered_io_benchmark,
    small_file_operations_benchmark,
    directory_operations_benchmark,
    concurrent_file_operations_benchmark,
    file_metadata_benchmark,
    file_copy_benchmark,
    file_seeking_benchmark
);
criterion_main!(benches);
