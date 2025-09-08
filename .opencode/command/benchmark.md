---
description: Run performance benchmarks for the entire system
agent: performance-optimizer
---

# Benchmark Command

Run comprehensive performance benchmarks for the AI Orchestrator Hub system, measuring throughput, latency, and resource utilization across all components.

## Benchmark Strategy

### 1. Environment Setup
Prepare benchmarking environment:

```bash
# Ensure performance mode
export RUSTFLAGS="-C target-cpu=native -C opt-level=3"
export NODE_ENV=production

# Disable debug features
unset RUST_BACKTRACE
unset RUST_LOG

# Configure system for benchmarking
echo "performance" | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```

### 2. System Benchmarks
Execute system-level performance tests:

```bash
# CPU benchmarks
cargo bench --bench cpu_benchmarks

# Memory benchmarks
cargo bench --bench memory_benchmarks

# I/O benchmarks
cargo bench --bench io_benchmarks
```

### 3. Application Benchmarks
Run application-specific benchmarks:

```bash
# Agent performance benchmarks
cargo bench --bench agent_benchmarks

# Neural network benchmarks
cargo bench --bench neural_benchmarks

# Swarm intelligence benchmarks
cargo bench --bench swarm_benchmarks
```

### 4. Integration Benchmarks
Test system integration performance:

```bash
# API benchmarks
npm run bench:api

# Database benchmarks
npm run bench:database

# End-to-end benchmarks
npm run bench:e2e
```

### 5. Load Testing
Execute load and stress tests:

```bash
# Load testing
npm run loadtest -- --concurrency 100 --duration 300

# Stress testing
npm run stresstest -- --max-connections 1000

# Spike testing
npm run spiketest -- --spike-duration 60
```

## Benchmark Categories

### Performance Metrics
- **Throughput**: Operations per second
- **Latency**: Response time percentiles (P50, P95, P99)
- **Resource Usage**: CPU, memory, network, disk utilization
- **Scalability**: Performance under increasing load
- **Efficiency**: Resource usage per operation

### Component Benchmarks
- **Agent Performance**: Individual agent processing speed
- **Neural Networks**: Model inference and training performance
- **Swarm Coordination**: Multi-agent communication efficiency
- **Task Processing**: Task scheduling and execution performance
- **Data Processing**: Data ingestion and processing throughput

### System Benchmarks
- **API Performance**: REST API response times and throughput
- **Database Performance**: Query performance and connection pooling
- **Network Performance**: Inter-service communication efficiency
- **Frontend Performance**: Page load times and interaction performance

## Benchmark Configuration

### Criterion Configuration
Configure `Cargo.toml` for benchmarking:

```toml
[dev-dependencies]
criterion = { version = "0.4", features = ["html_reports"] }

[[bench]]
name = "agent_benchmarks"
harness = false

[[bench]]
name = "neural_benchmarks"
harness = false
```

### Benchmark Setup
Create benchmark harness:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn agent_creation_benchmark(c: &mut Criterion) {
    c.bench_function("agent_creation", |b| {
        b.iter(|| {
            black_box(create_agent());
        });
    });
}

criterion_group!(benches, agent_creation_benchmark);
criterion_main!(benches);
```

### Load Testing Configuration
Configure load testing tools:

```javascript
// k6 load test configuration
export const options = {
  scenarios: {
    constant_load: {
      executor: 'constant-vus',
      vus: 50,
      duration: '5m',
    },
    spike_test: {
      executor: 'ramping-vus',
      stages: [
        { duration: '1m', target: 10 },
        { duration: '1m', target: 1000 },
        { duration: '1m', target: 10 },
      ],
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<500'],
    http_req_failed: ['rate<0.1'],
  },
};
```

## Performance Profiling

### Profiling Tools
Use profiling tools for detailed analysis:

```bash
# Flame graphs
cargo flamegraph --bench agent_benchmarks

# Memory profiling
cargo heaptrack --bench memory_benchmarks

# CPU profiling
cargo perf --bench cpu_benchmarks
```

### Performance Analysis
Analyze benchmark results:

```bash
# Generate performance reports
cargo bench --bench all -- --save-baseline

# Compare with previous runs
cargo bench --bench all -- --baseline master

# Export results
cargo bench --bench all -- --output-format json
```

## Benchmark Results

### Result Collection
Collect and store benchmark results:

```bash
# Save benchmark results
mkdir -p benchmarks/$(date +%Y%m%d_%H%M%S)
cargo bench --bench all -- --save-baseline current

# Export to JSON
cargo bench --bench all -- --output-format json > benchmarks/results.json
```

### Result Analysis
Analyze benchmark data:

```bash
# Generate comparison reports
npm run bench:compare -- --baseline master --current current

# Create performance dashboard
npm run bench:dashboard

# Alert on regressions
npm run bench:alert -- --threshold 5%
```

## Continuous Benchmarking

### CI/CD Integration
Integrate benchmarks into CI pipeline:

```yaml
# GitHub Actions benchmark workflow
name: Benchmarks
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run benchmarks
        run: cargo bench --bench all
      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          name: Rust Benchmarks
          tool: 'cargo'
          output-file-path: output.json
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
```

### Regression Detection
Detect performance regressions:

```bash
# Compare with baseline
cargo bench --bench all -- --baseline master --threshold 5%

# Fail on regression
if [ $? -ne 0 ]; then
  echo "Performance regression detected!"
  exit 1
fi
```

## Optimization Recommendations

### Based on Results
Generate optimization recommendations:

```bash
# Analyze bottlenecks
npm run bench:analyze

# Suggest optimizations
npm run bench:optimize

# Create optimization plan
npm run bench:plan
```

## Best Practices

1. **Consistent Environment**: Run benchmarks in identical environments
2. **Statistical Significance**: Run benchmarks multiple times for reliable results
3. **Realistic Scenarios**: Use realistic test data and scenarios
4. **Resource Monitoring**: Monitor system resources during benchmarks
5. **Baseline Comparison**: Compare results against established baselines
6. **Documentation**: Document benchmark methodology and results
7. **Automation**: Automate benchmark execution and analysis

## Common Issues

- **Environment Variability**: Inconsistent benchmark environments
- **Measurement Noise**: External factors affecting results
- **Benchmark Validity**: Benchmarks not reflecting real-world usage
- **Resource Contention**: Other processes affecting benchmark results
- **Configuration Drift**: Changing system configuration between runs
- **Result Interpretation**: Difficulty interpreting complex benchmark data
