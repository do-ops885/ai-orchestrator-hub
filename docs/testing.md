# Testing Guide

This guide covers testing strategies, tools, and best practices for the AI Orchestrator Hub.

## Testing Overview

### Testing Pyramid

```
End-to-End Tests (E2E)
    ↕️
Integration Tests
    ↕️
Unit Tests
    ↕️
Static Analysis
```

### Test Types

- **Unit Tests**: Test individual functions, methods, and modules
- **Integration Tests**: Test component interactions and API endpoints
- **End-to-End Tests**: Test complete user workflows and real-time features
- **Performance Tests**: Test system performance under load and stress
- **Security Tests**: Test authentication, authorization, and input validation
- **Accessibility Tests**: Test UI accessibility and usability
- **Neural Network Tests**: Test AI/ML model accuracy and performance
- **MCP Protocol Tests**: Test Model Context Protocol integration

## Unit Testing

### Backend Unit Tests

```rust
// backend/src/agents/tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_agent_creation() {
        let agent = Agent::new("test_agent", AgentType::Worker).await;
        assert_eq!(agent.name, "test_agent");
        assert_eq!(agent.agent_type, AgentType::Worker);
        assert!(agent.energy > 0.0);
    }

    #[test]
    async fn test_adaptive_learning() {
        let mut agent = create_test_agent();
        let initial_proficiency = agent.get_capability_proficiency("data_processing");

        // Simulate learning experience
        agent.record_experience("data_processing", true).await;
        let updated_proficiency = agent.get_capability_proficiency("data_processing");

        assert!(updated_proficiency > initial_proficiency);
    }

    #[test]
    async fn test_agent_recovery() {
        let mut agent = create_test_agent();
        agent.mark_failed().await;

        let recovery_manager = AgentRecoveryManager::new();
        let recovered = recovery_manager.attempt_recovery(&mut agent).await;

        assert!(recovered);
        assert_eq!(agent.status, AgentStatus::Idle);
    }

    #[tokio::test]
    async fn test_concurrent_task_processing() {
        let agent = Arc::new(RwLock::new(create_test_agent()));
        let tasks = create_multiple_tasks(10);

        let results = join_all(
            tasks.into_iter().map(|task| {
                let agent_clone = agent.clone();
                async move {
                    let mut agent = agent_clone.write().await;
                    agent.process_task(task).await
                }
            })
        ).await;

        assert_eq!(results.len(), 10);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    async fn test_neural_processing() {
        let processor = AdaptiveLearningSystem::new(Default::default()).await.unwrap();
        let text = "Process this data for analysis";

        let result = processor.analyze_text(text).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.confidence > 0.0);
        assert!(!analysis.keywords.is_empty());
    }
}
```

### Frontend Unit Tests

```typescript
// __tests__/components/AgentCard.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { AgentCard } from '../../components/AgentCard';

const mockAgent = {
  id: '1',
  name: 'Test Agent',
  type: 'Worker',
  status: 'Idle',
  capabilities: [],
  position: [0, 0],
  energy: 100,
  created_at: new Date().toISOString(),
  last_active: new Date().toISOString(),
};

describe('AgentCard', () => {
  it('renders agent information correctly', () => {
    render(<AgentCard agent={mockAgent} />);

    expect(screen.getByText('Test Agent')).toBeInTheDocument();
    expect(screen.getByText('Worker')).toBeInTheDocument();
    expect(screen.getByText('Idle')).toBeInTheDocument();
  });

  it('calls onSelect when clicked', () => {
    const mockOnSelect = jest.fn();
    render(<AgentCard agent={mockAgent} onSelect={mockOnSelect} />);

    fireEvent.click(screen.getByRole('button'));
    expect(mockOnSelect).toHaveBeenCalledWith(mockAgent);
  });

  it('displays energy level correctly', () => {
    render(<AgentCard agent={mockAgent} />);

    const energyBar = screen.getByRole('progressbar');
    expect(energyBar).toHaveAttribute('aria-valuenow', '100');
  });
});
```

### Test Utilities

```rust
// tests/test_utils.rs
use crate::models::{Agent, Task, Hive};

pub fn create_test_agent() -> Agent {
    Agent::builder()
        .name("Test Agent")
        .agent_type(AgentType::Worker)
        .capabilities(vec![Capability::new("test", 1.0, 0.1)])
        .build()
}

pub fn create_test_task() -> Task {
    Task::builder()
        .description("Test task")
        .priority(Priority::Medium)
        .required_capabilities(vec![])
        .build()
}

pub fn create_test_hive() -> Hive {
    Hive::builder()
        .max_agents(10)
        .task_queue_size(100)
        .build()
}

pub async fn setup_test_database() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();

    // Run migrations
    Migrator::up(&db, None).await.unwrap();

    db
}
```

## Integration Testing

### API Integration Tests

```rust
// backend/tests/api_integration_tests.rs
use reqwest::Client;
use serde_json::json;

#[tokio::test]
async fn test_complete_agent_lifecycle() {
    let client = Client::new();
    let base_url = "http://localhost:3001";

    // Health check
    let response = client
        .get(&format!("{}/health", base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // Create agent
    let response = client
        .post(&format!("{}/api/agents", base_url))
        .json(&json!({
            "name": "Integration Test Agent",
            "agent_type": "Worker",
            "capabilities": [{
                "name": "data_processing",
                "proficiency": 0.8,
                "learning_rate": 0.1
            }]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 201);
    let agent: serde_json::Value = response.json().await.unwrap();
    let agent_id = agent["id"].as_str().unwrap();

    // Get agent details
    let response = client
        .get(&format!("{}/api/agents/{}", base_url, agent_id))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // Create task
    let response = client
        .post(&format!("{}/api/tasks", base_url))
        .json(&json!({
            "description": "Process integration test data",
            "priority": "High",
            "required_capabilities": [{
                "name": "data_processing",
                "min_proficiency": 0.7
            }]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 201);

    // Get hive status
    let response = client
        .get(&format!("{}/api/hive/status", base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // Get resource information
    let response = client
        .get(&format!("{}/api/resources", base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // Get metrics
    let response = client
        .get(&format!("{}/metrics", base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}
```

### WebSocket Integration Tests

```typescript
// frontend/__tests__/websocket.integration.test.ts
import WebSocket from 'ws';

describe('WebSocket Integration', () => {
  let ws: WebSocket;
  const wsUrl = 'ws://localhost:3001/ws';

  beforeEach((done) => {
    ws = new WebSocket(wsUrl);
    ws.on('open', () => done());
  });

  afterEach(() => {
    if (ws.readyState === WebSocket.OPEN) {
      ws.close();
    }
  });

  it('receives comprehensive hive status updates', (done) => {
    ws.on('message', (data) => {
      const message = JSON.parse(data.toString());
      if (message.type === 'hive_status') {
        expect(message.data).toHaveProperty('metrics');
        expect(message.data).toHaveProperty('swarm_center');
        expect(message.data).toHaveProperty('total_energy');
        expect(message.data.metrics).toHaveProperty('total_agents');
        expect(message.data.metrics).toHaveProperty('active_agents');
        done();
      }
    });
  });

  it('handles agent lifecycle notifications', (done) => {
    let messageCount = 0;

    ws.on('message', (data) => {
      const message = JSON.parse(data.toString());
      messageCount++;

      if (message.type === 'agent_created') {
        expect(message.data.agent).toHaveProperty('name');
        expect(message.data.agent).toHaveProperty('capabilities');
      }

      if (messageCount >= 2) done(); // Wait for creation + status update
    });

    // Create agent via REST API
    fetch('http://localhost:3001/api/agents', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        name: 'WebSocket Test Agent',
        agent_type: 'Worker',
        capabilities: [{
          name: 'data_processing',
          proficiency: 0.8
        }]
      })
    });
  });

  it('receives intelligent alerts', (done) => {
    ws.on('message', (data) => {
      const message = JSON.parse(data.toString());
      if (message.type === 'alert_triggered') {
        expect(message.data).toHaveProperty('level');
        expect(message.data).toHaveProperty('title');
        expect(message.data).toHaveProperty('confidence');
        done();
      }
    });

    // Trigger an alert by creating high load
    // This would be done through the test setup
  });

  it('handles resource update notifications', (done) => {
    ws.on('message', (data) => {
      const message = JSON.parse(data.toString());
      if (message.type === 'resource_update') {
        expect(message.data).toHaveProperty('cpu_usage');
        expect(message.data).toHaveProperty('memory_usage');
        done();
      }
    });
  });
});
```

### Database Integration Tests

```rust
// tests/database_integration_tests.rs
use sqlx::PgPool;

#[sqlx::test]
async fn test_task_persistence(pool: PgPool) {
    // Create task
    let task_id = sqlx::query!(
        "INSERT INTO tasks (description, priority, status) VALUES ($1, $2, $3) RETURNING id",
        "Integration test task",
        "medium",
        "pending"
    )
    .fetch_one(&pool)
    .await
    .unwrap()
    .id;

    // Retrieve task
    let task = sqlx::query!("SELECT * FROM tasks WHERE id = $1", task_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(task.description, "Integration test task");
    assert_eq!(task.status, "pending");

    // Update task
    sqlx::query!(
        "UPDATE tasks SET status = $1 WHERE id = $2",
        "completed",
        task_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Verify update
    let updated_task = sqlx::query!("SELECT status FROM tasks WHERE id = $1", task_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(updated_task.status, "completed");
}
```

## End-to-End Testing

### E2E Test Setup

```typescript
// e2e/tests/agent-management.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Agent Management', () => {
  test('should create and manage agents', async ({ page }) => {
    // Navigate to application
    await page.goto('http://localhost:3000');

    // Click create agent button
    await page.click('[data-testid="create-agent-button"]');

    // Fill agent form
    await page.fill('[data-testid="agent-name-input"]', 'E2E Test Agent');
    await page.selectOption('[data-testid="agent-type-select"]', 'Worker');

    // Add capability
    await page.click('[data-testid="add-capability-button"]');
    await page.fill('[data-testid="capability-name-input"]', 'data_processing');
    await page.fill('[data-testid="capability-proficiency-input"]', '0.8');

    // Submit form
    await page.click('[data-testid="submit-agent-button"]');

    // Verify agent appears in list
    await expect(page.locator('[data-testid="agent-list"]')).toContainText('E2E Test Agent');

    // Verify agent details
    await page.click('[data-testid="agent-card-E2E-Test-Agent"]');
    await expect(page.locator('[data-testid="agent-status"]')).toHaveText('Idle');
    await expect(page.locator('[data-testid="agent-energy"]')).toBeVisible();
  });

  test('should assign tasks to agents', async ({ page }) => {
    // Create agent first
    await page.goto('http://localhost:3000');
    await createTestAgent(page);

    // Create task
    await page.click('[data-testid="create-task-button"]');
    await page.fill('[data-testid="task-description-input"]', 'E2E test task');
    await page.selectOption('[data-testid="task-priority-select"]', 'High');

    // Add capability requirement
    await page.click('[data-testid="add-requirement-button"]');
    await page.fill('[data-testid="requirement-name-input"]', 'data_processing');
    await page.fill('[data-testid="requirement-proficiency-input"]', '0.7');

    // Submit task
    await page.click('[data-testid="submit-task-button"]');

    // Verify task assignment
    await expect(page.locator('[data-testid="task-status"]')).toHaveText('Assigned');
    await expect(page.locator('[data-testid="assigned-agent"]')).toBeVisible();
  });
});
```

### E2E Test Configuration

```javascript
// playwright.config.js
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',

  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],

  webServer: [
    {
      command: 'npm run dev',
      port: 3000,
      reuseExistingServer: !process.env.CI,
    },
    {
      command: 'cargo run',
      port: 3001,
      reuseExistingServer: !process.env.CI,
    },
  ],
});
```

## Performance Testing

### Load Testing

```bash
# Using k6 for load testing
# tests/performance/load-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
  stages: [
    { duration: '2m', target: 100 }, // Ramp up to 100 users
    { duration: '5m', target: 100 }, // Stay at 100 users
    { duration: '2m', target: 200 }, // Ramp up to 200 users
    { duration: '5m', target: 200 }, // Stay at 200 users
    { duration: '2m', target: 0 },   // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(99)<1500'], // 99% of requests should be below 1.5s
    http_req_failed: ['rate<0.1'],     // Error rate should be below 10%
  },
};

export default function () {
  let response = http.get('http://localhost:3001/api/hive/status');

  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });

  sleep(1);
}
```

### Stress Testing

```bash
# tests/performance/stress-test.js
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  stages: [
    { duration: '1m', target: 10 },
    { duration: '2m', target: 50 },
    { duration: '2m', target: 100 },
    { duration: '2m', target: 200 },
    { duration: '2m', target: 500 },
    { duration: '2m', target: 1000 },
  ],
};

export default function () {
  // Create agent
  let createResponse = http.post('http://localhost:3001/api/agents', {
    name: `Stress Test Agent ${__VU}`,
    agent_type: 'Worker',
  });

  check(createResponse, {
    'agent creation successful': (r) => r.status === 201,
  });

  // Create task
  let taskResponse = http.post('http://localhost:3001/api/tasks', {
    description: `Stress test task ${__VU}`,
    priority: 'Medium',
  });

  check(taskResponse, {
    'task creation successful': (r) => r.status === 201,
  });
}
```

## Security Testing

### Authentication Testing

```typescript
// tests/security/auth.test.ts
describe('Authentication', () => {
  it('should reject requests without authentication', async () => {
    const response = await fetch('/api/agents');
    expect(response.status).toBe(401);
  });

  it('should accept valid JWT tokens', async () => {
    const token = await login('test@example.com', 'password');
    const response = await fetch('/api/agents', {
      headers: { Authorization: `Bearer ${token}` }
    });
    expect(response.status).toBe(200);
  });

  it('should reject expired tokens', async () => {
    const expiredToken = createExpiredToken();
    const response = await fetch('/api/agents', {
      headers: { Authorization: `Bearer ${expiredToken}` }
    });
    expect(response.status).toBe(401);
  });
});
```

### Authorization Testing

```typescript
// tests/security/authorization.test.ts
describe('Authorization', () => {
  it('should allow admin users to create agents', async () => {
    const adminToken = await loginAsAdmin();
    const response = await fetch('/api/agents', {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${adminToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ name: 'Admin Agent', agent_type: 'Worker' })
    });
    expect(response.status).toBe(201);
  });

  it('should deny regular users from admin operations', async () => {
    const userToken = await loginAsUser();
    const response = await fetch('/api/admin/reset-hive', {
      method: 'POST',
      headers: { Authorization: `Bearer ${userToken}` }
    });
    expect(response.status).toBe(403);
  });
});
```

### Input Validation Testing

```typescript
// tests/security/input-validation.test.ts
describe('Input Validation', () => {
  it('should reject malformed JSON', async () => {
    const response = await fetch('/api/agents', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: '{ invalid json }'
    });
    expect(response.status).toBe(400);
  });

  it('should sanitize XSS attempts', async () => {
    const maliciousInput = '<script>alert("xss")</script>';
    const response = await fetch('/api/agents', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: maliciousInput })
    });
    expect(response.status).toBe(201);

    const agent = await response.json();
    expect(agent.name).not.toContain('<script>');
  });

  it('should validate agent type', async () => {
    const response = await fetch('/api/agents', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: 'Test', agent_type: 'InvalidType' })
    });
    expect(response.status).toBe(400);
  });
});
```

## Test Automation

### CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Cache dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            backend/target
            frontend/node_modules
          key: ${{ runner.os }}-deps-${{ hashFiles('**/Cargo.lock', '**/package-lock.json') }}

      - name: Run backend linting
        working-directory: ./backend
        run: |
          cargo clippy --all-features -- -D warnings
          cargo fmt --all -- --check

      - name: Run backend tests
        working-directory: ./backend
        run: |
          cargo test --all-features
          cargo test --doc

      - name: Run frontend linting
        working-directory: ./frontend
        run: npm run lint:check

      - name: Run frontend tests
        working-directory: ./frontend
        run: npm test -- --coverage --watchAll=false

      - name: Build frontend
        working-directory: ./frontend
        run: npm run build

      - name: Run integration tests
        run: |
          cd backend && cargo build --release
          cd ../frontend && npm run build
          # Start services and run integration tests
          npm run test:integration

      - name: Upload coverage reports
        uses: actions/upload-artifact@v4
        with:
          name: coverage-reports
          path: |
            frontend/coverage/
            backend/target/tarpaulin/
```

### Test Coverage

```rust
// backend/tests/coverage.rs
use std::process::Command;

#[test]
fn test_coverage_report() {
    let output = Command::new("cargo")
        .args(&["tarpaulin", "--out", "Xml"])
        .output()
        .expect("Failed to run tarpaulin");

    assert!(output.status.success(), "Coverage report generation failed");

    // Parse coverage XML and assert minimum coverage
    let coverage = parse_coverage_xml(&output.stdout);
    assert!(coverage.total_coverage > 80.0, "Total coverage below 80%: {:.2}%", coverage.total_coverage);
}
```

### Test Data Management

```rust
// tests/test_data.rs
use once_cell::sync::Lazy;

static TEST_AGENTS: Lazy<Vec<Agent>> = Lazy::new(|| {
    vec![
        create_agent("Alice", AgentType::Worker),
        create_agent("Bob", AgentType::Coordinator),
        create_agent("Charlie", AgentType::Specialist),
    ]
});

static TEST_TASKS: Lazy<Vec<Task>> = Lazy::new(|| {
    vec![
        create_task("Process data", Priority::High),
        create_task("Analyze results", Priority::Medium),
        create_task("Generate report", Priority::Low),
    ]
});

pub fn get_test_agents() -> &'static [Agent] {
    &TEST_AGENTS
}

pub fn get_test_tasks() -> &'static [Task] {
    &TEST_TASKS
}
```

## Mocking and Stubbing

### Backend Mocking

```rust
// tests/mocks.rs
use mockall::mock;

#[cfg(test)]
mod mocks {
    use super::*;

    mock! {
        pub Database {
            fn get_agent(&self, id: &str) -> Result<Option<Agent>>;
            fn save_agent(&self, agent: &Agent) -> Result<()>;
            fn get_all_agents(&self) -> Result<Vec<Agent>>;
        }
    }

    mock! {
        pub NeuralProcessor {
            fn process_text(&self, text: &str) -> Result<NeuralResult>;
            fn analyze_sentiment(&self, text: &str) -> Result<f32>;
        }
    }
}
```

### Frontend Mocking

```typescript
// __mocks__/api.ts
export const mockApi = {
  getAgents: jest.fn(),
  createAgent: jest.fn(),
  updateAgent: jest.fn(),
  deleteAgent: jest.fn(),
};

export const mockWebSocket = {
  send: jest.fn(),
  onmessage: jest.fn(),
  onopen: jest.fn(),
  onclose: jest.fn(),
  onerror: jest.fn(),
};
```

## Test Reporting

### JUnit XML Output

```xml
<!-- test-results.xml -->
<testsuites>
  <testsuite name="unit_tests" tests="15" failures="0" errors="0" time="0.5">
    <testcase name="test_agent_creation" time="0.1" />
    <testcase name="test_task_assignment" time="0.2" />
  </testsuite>
  <testsuite name="integration_tests" tests="8" failures="1" errors="0" time="2.3">
    <testcase name="test_agent_lifecycle" time="1.2" />
    <testcase name="test_task_workflow">
      <failure message="Expected task to be completed">...</failure>
    </testcase>
  </testsuite>
</testsuites>
```

### Coverage Reports

```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Upload to codecov
bash <(curl -s https://codecov.io/bash) -f coverage/cobertura.xml
```

### Test Dashboards

```typescript
// components/TestDashboard.tsx
const TestDashboard = () => {
  const [testResults, setTestResults] = useState<TestResults | null>(null);

  useEffect(() => {
    fetchTestResults().then(setTestResults);
  }, []);

  if (!testResults) return <div>Loading...</div>;

  return (
    <div className="test-dashboard">
      <h2>Test Results</h2>
      <div className="metrics">
        <div>Unit Tests: {testResults.unit.passed}/{testResults.unit.total}</div>
        <div>Integration Tests: {testResults.integration.passed}/{testResults.integration.total}</div>
        <div>E2E Tests: {testResults.e2e.passed}/{testResults.e2e.total}</div>
        <div>Coverage: {testResults.coverage.percentage}%</div>
      </div>
      <TestChart data={testResults.history} />
    </div>
  );
};
```

## Best Practices

### Test Organization

- **Group related tests** in modules
- **Use descriptive test names** that explain what is being tested
- **Follow AAA pattern**: Arrange, Act, Assert
- **Keep tests independent** and isolated
- **Use fixtures** for common test data

### Test Quality

- **Test edge cases** and error conditions
- **Verify error messages** are helpful
- **Test async operations** properly
- **Mock external dependencies**
- **Use property-based testing** for complex logic

### CI/CD Integration

- **Run tests on every commit**
- **Fail builds on test failures**
- **Generate coverage reports**
- **Archive test artifacts**
- **Set up test environments**

### Maintenance

- **Keep tests up to date** with code changes
- **Remove obsolete tests**
- **Refactor tests** when code is refactored
- **Document test setup** and requirements
- **Review test coverage** regularly

## Next Steps

- **Configuration**: See [docs/configuration.md](configuration.md)
- **Development**: See [docs/getting-started.md](getting-started.md)
- **CI/CD**: See [docs/ci-cd.md](ci-cd.md)
- **Performance**: See [docs/performance.md](performance.md)