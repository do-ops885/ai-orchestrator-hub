---
description: Run comprehensive tests across Rust backend and frontend
agent: rust-developer
---

# Test All Command

Run comprehensive test suite across the entire AI Orchestrator Hub project, including Rust backend, frontend, and integration tests.

## Test Execution Strategy

### 1. Environment Preparation
Set up testing environment:

```bash
# Ensure clean state
git status

# Start required services (if any)
docker-compose up -d database redis

# Set test environment
export RUST_TEST_THREADS=4
export NODE_ENV=test
```

### 2. Backend Testing
Execute Rust backend tests:

```bash
# Run all Rust tests
cargo test --all-features

# Run with coverage
cargo tarpaulin --out Html

# Run benchmarks as tests
cargo bench --all
```

### 3. Frontend Testing
Execute frontend tests:

```bash
# Install test dependencies
npm ci

# Run unit tests
npm run test:unit

# Run integration tests
npm run test:integration

# Run E2E tests
npm run test:e2e
```

### 4. Integration Testing
Run cross-system integration tests:

```bash
# Start backend in test mode
cargo run --bin test-server &

# Run integration tests
npm run test:integration

# API contract tests
npm run test:contract
```

### 5. Performance Testing
Execute performance benchmarks:

```bash
# Backend benchmarks
cargo bench

# Frontend performance tests
npm run test:performance

# Load testing
npm run test:load
```

## Test Categories

### Unit Tests
- **Backend Unit Tests**: Individual function and module testing
- **Frontend Unit Tests**: Component and utility function testing
- **Algorithm Tests**: Core algorithm correctness verification

### Integration Tests
- **API Integration**: Backend-frontend communication testing
- **Database Integration**: Data persistence and retrieval testing
- **External Service Integration**: Third-party service interaction testing

### End-to-End Tests
- **User Workflows**: Complete user journey testing
- **System Workflows**: End-to-end system process testing
- **Error Scenarios**: Failure mode and recovery testing

### Performance Tests
- **Load Testing**: System capacity and scalability testing
- **Stress Testing**: System limits and failure point identification
- **Spike Testing**: Sudden load increase handling

## Test Configuration

### Backend Test Configuration
Configure `Cargo.toml` for testing:

```toml
[features]
default = []
test = ["mock", "integration"]

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.11"
criterion = "0.4"
```

### Frontend Test Configuration
Configure test setup in `jest.config.js`:

```javascript
module.exports = {
  testEnvironment: 'jsdom',
  setupFilesAfterEnv: ['<rootDir>/jest.setup.js'],
  moduleNameMapping: {
    '^@/(.*)$': '<rootDir>/src/$1',
  },
  collectCoverageFrom: [
    'src/**/*.{js,jsx,ts,tsx}',
    '!src/**/*.d.ts',
  ],
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80,
    },
  },
};
```

## Test Data Management

### Test Fixtures
- **Mock Data**: Simulated data for isolated testing
- **Test Databases**: Isolated database instances for testing
- **API Mocks**: Simulated external service responses

### Data Setup
```bash
# Setup test database
npm run db:test:setup

# Load test fixtures
npm run fixtures:load

# Clean test data
npm run test:cleanup
```

## Test Reporting

### Coverage Reports
Generate comprehensive coverage reports:

```bash
# Backend coverage
cargo tarpaulin --out Html --output-dir ./target/coverage

# Frontend coverage
npm run test:coverage

# Combined coverage
npm run coverage:merge
```

### Test Results
Collect and analyze test results:

```bash
# JUnit XML output
npm run test:junit

# Allure reports
npm run test:allure

# Performance reports
npm run test:performance:report
```

## Continuous Integration

### CI Pipeline Integration
- **Automated Testing**: Run tests on every commit
- **Parallel Execution**: Run tests in parallel for speed
- **Artifact Storage**: Store test results and coverage reports
- **Quality Gates**: Block merges on test failures

### Test Environments
- **Development**: Fast feedback with subset of tests
- **Staging**: Full test suite with integration tests
- **Production**: Critical path and performance tests

## Error Handling and Debugging

### Test Failures
Handle and debug test failures:

```bash
# Run failed tests only
npm run test:failed

# Debug specific test
npm run test:debug -- --testNamePattern="specific test"

# Verbose output
npm run test:verbose
```

### Flaky Tests
Identify and fix unreliable tests:

```bash
# Run tests multiple times
npm run test:flaky

# Analyze test stability
npm run test:stability
```

## Test Maintenance

### Test Organization
- **Test Structure**: Logical grouping and naming conventions
- **Test Documentation**: Clear test descriptions and purposes
- **Test Dependencies**: Minimal and clear test dependencies

### Regular Maintenance
- **Test Updates**: Update tests for code changes
- **Dead Test Removal**: Remove obsolete tests
- **Performance Monitoring**: Monitor test execution time
- **Coverage Analysis**: Ensure adequate test coverage

## Best Practices

1. **Fast Feedback**: Quick test execution for development
2. **Reliable Tests**: Deterministic and stable test results
3. **Comprehensive Coverage**: Test all critical paths and edge cases
4. **Maintainable Tests**: Clean, readable, and well-documented tests
5. **Performance Aware**: Efficient test execution and resource usage
6. **CI/CD Integration**: Seamless integration with deployment pipeline

## Common Issues

- **Flaky Tests**: Intermittent test failures
- **Slow Tests**: Performance issues in test execution
- **Test Dependencies**: Complex setup and teardown requirements
- **Coverage Gaps**: Missing test coverage for critical code
- **Environment Issues**: Test environment inconsistencies
- **Race Conditions**: Timing issues in concurrent tests