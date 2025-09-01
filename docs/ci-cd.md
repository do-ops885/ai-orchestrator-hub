# CI/CD Guide

This guide covers continuous integration and continuous deployment pipelines for the AI Orchestrator Hub.

## CI/CD Overview

### Pipeline Stages

```
Source Code
    ↓
Build & Test
    ↓
Security Scan
    ↓
Package & Deploy
    ↓
Integration Test
    ↓
Production
```

### Key Principles

- **Automated**: All processes are automated and triggered by events
- **Fast Feedback**: Quick feedback on code quality and functionality
- **Consistent**: Same process for all environments
- **Reliable**: Robust error handling and rollback capabilities
- **Secure**: Security checks integrated into pipeline

## GitHub Actions Workflows

### Main CI Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # Backend CI
  backend:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      redis:
        image: redis:7-alpine
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache Rust dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            backend/target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-

      - name: Check formatting
        working-directory: ./backend
        run: cargo fmt --all -- --check

      - name: Run Clippy
        working-directory: ./backend
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run tests
        working-directory: ./backend
        run: cargo test --all-features
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/hive_test
          REDIS_URL: redis://localhost:6379

      - name: Generate coverage report
        working-directory: ./backend
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --output-dir coverage

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          file: ./backend/coverage/cobertura.xml
          flags: backend
          name: Backend Coverage

      - name: Build release binary
        working-directory: ./backend
        run: cargo build --release

       - name: Upload build artifacts
         uses: actions/upload-artifact@v4
         with:
           name: backend-binary
           path: backend/target/release/ai-orchestrator-hub

  # Frontend CI
  frontend:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Install dependencies
        working-directory: ./frontend
        run: npm ci

      - name: Run linting
        working-directory: ./frontend
        run: npm run lint:check

      - name: Run type checking
        working-directory: ./frontend
        run: npx tsc --noEmit

      - name: Run tests
        working-directory: ./frontend
        run: npm test -- --coverage --watchAll=false

      - name: Build application
        working-directory: ./frontend
        run: npm run build

      - name: Upload build artifacts
        uses: actions/upload-artifact@v4
        with:
          name: frontend-build
          path: frontend/.next/

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          file: ./frontend/coverage/lcov.info
          flags: frontend
          name: Frontend Coverage

  # Security scanning
  security:
    runs-on: ubuntu-latest
    needs: [backend, frontend]

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          scan-ref: '.'
          format: 'sarif'
          output: 'trivy-results.sarif'

      - name: Upload Trivy scan results
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: 'trivy-results.sarif'

      - name: Run Snyk security scan
        uses: snyk/actions/rust@master
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
        with:
          args: --file=backend/Cargo.toml

      - name: Run npm audit
        working-directory: ./frontend
        run: npm audit --audit-level=moderate

  # Integration tests
  integration:
    runs-on: ubuntu-latest
    needs: [backend, frontend]

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Download backend binary
        uses: actions/download-artifact@v4
        with:
          name: backend-binary
          path: ./backend-binary

      - name: Download frontend build
        uses: actions/download-artifact@v4
        with:
          name: frontend-build
          path: ./frontend-build

       - name: Setup test environment
         run: |
           chmod +x ./backend-binary/ai-orchestrator-hub
           ./backend-binary/ai-orchestrator-hub &
           sleep 10

      - name: Run integration tests
        run: npm run test:integration

      - name: Run E2E tests
        uses: cypress-io/github-action@v6
        with:
          working-directory: frontend
          start: npm run dev
          wait-on: 'http://localhost:3000'
```

### Release Pipeline

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  release:
    runs-on: ubuntu-latest

    permissions:
      contents: read
      packages: write

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=semver,pattern={{major}}
            type=sha

      - name: Build and push backend image
        uses: docker/build-push-action@v5
        with:
          context: ./backend
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Build and push frontend image
        uses: docker/build-push-action@v5
        with:
          context: ./frontend
          push: true
          tags: ${{ steps.meta.outputs.tags }}-frontend
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Generate release notes
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ github.ref }}
          body: |
            ## Changes
            - Backend improvements
            - Frontend enhancements
            - Bug fixes
            - Security updates

            ## Docker Images
            - Backend: `${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }}`
            - Frontend: `${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }}-frontend`

      - name: Deploy to staging
        if: github.ref == 'refs/heads/main'
        run: |
          echo "Deploying to staging environment"
          # Add deployment commands here

      - name: Deploy to production
        if: github.ref == 'refs/heads/main'
        run: |
          echo "Deploying to production environment"
          # Add production deployment commands here
```

### Documentation Pipeline

```yaml
# .github/workflows/docs.yml
name: Documentation

on:
  push:
    branches: [ main ]
    paths:
      - 'docs/**'
      - 'README.md'
      - '.github/workflows/docs.yml'

jobs:
  docs:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install markdownlint
        run: npm install -g markdownlint-cli

      - name: Lint markdown files
        run: markdownlint '**/*.md' --config .markdownlint.json

      - name: Check links
        uses: gaurav-nelson/github-action-markdown-link-check@v1
        with:
          use-quiet-mode: 'yes'
          use-verbose-mode: 'yes'
          check-modified-files-only: 'yes'

      - name: Setup Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      - name: Install Sphinx
        run: |
          python -m pip install --upgrade pip
          pip install sphinx sphinx-rtd-theme

      - name: Build documentation
        run: |
          cd docs
          sphinx-build -b html . _build/html

      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v4
        if: github.ref == 'refs/heads/main'
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: docs/_build/html
```

## Advanced CI/CD Patterns

### Matrix Builds

```yaml
# .github/workflows/matrix-build.yml
name: Matrix Build

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
        node: [18, 20]
        exclude:
          - os: windows-latest
            rust: beta
          - os: macos-latest
            node: 18

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ matrix.rust }}

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node }}

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Run tests
        run: |
          cargo test --all-features
          cd frontend && npm test
```

### Parallel Testing

```yaml
# .github/workflows/parallel-tests.yml
name: Parallel Tests

on:
  push:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install nextest
        run: cargo install cargo-nextest

      - name: Run backend tests in parallel
        working-directory: ./backend
        run: cargo nextest run --all-features --profile ci

      - name: Run frontend tests
        working-directory: ./frontend
        run: npm run test:ci

  performance:
    runs-on: ubuntu-latest
    needs: test

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run performance benchmarks
        working-directory: ./backend
        run: cargo bench

      - name: Store benchmark results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          name: Rust Benchmark
          tool: 'cargo'
          output-file-path: output.txt
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
```

### Multi-Environment Deployment

```yaml
# .github/workflows/deploy.yml
name: Deploy

on:
  push:
    branches: [ main ]
  workflow_dispatch:
    inputs:
      environment:
        description: 'Environment to deploy to'
        required: true
        default: 'staging'
        type: choice
        options:
          - staging
          - production

env:
  ENVIRONMENT: ${{ github.event.inputs.environment || 'staging' }}

jobs:
  deploy:
    runs-on: ubuntu-latest
    environment: ${{ env.ENVIRONMENT }}

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup deployment environment
        run: |
          echo "Deploying to ${{ env.ENVIRONMENT }}"
          # Configure environment-specific settings

      - name: Deploy backend
        run: |
          # Backend deployment commands
          echo "Deploying backend to ${{ env.ENVIRONMENT }}"

      - name: Deploy frontend
        run: |
          # Frontend deployment commands
          echo "Deploying frontend to ${{ env.ENVIRONMENT }}"

      - name: Run smoke tests
        run: |
          # Quick validation tests
          curl -f https://api-${{ env.ENVIRONMENT }}.example.com/health
          curl -f https://app-${{ env.ENVIRONMENT }}.example.com

      - name: Notify deployment
        if: success()
        run: |
          # Send notification
          echo "Deployment to ${{ env.ENVIRONMENT }} successful"
```

## Quality Gates

### Code Quality Checks

```yaml
# .github/workflows/quality-gates.yml
name: Quality Gates

on:
  pull_request:
    branches: [ main ]

jobs:
  quality:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Check code formatting
        run: |
          cargo fmt --all -- --check
          cd frontend && npm run lint:check

      - name: Run security scan
        uses: github/super-linter/slim@v5
        env:
          DEFAULT_BRANCH: main
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Check test coverage
        run: |
          cargo tarpaulin --all-features --out Lcov --output-dir coverage
          # Check if coverage meets threshold
          # Add your coverage threshold check here

      - name: Check documentation
        run: |
          # Check if all public APIs are documented
          cargo doc --no-deps
          # Check if README links are valid
          npx markdown-link-check README.md

      - name: Performance regression check
        run: |
          # Compare current benchmarks with baseline
          cargo bench
          # Add performance regression detection
```

### Automated Reviews

```yaml
# .github/workflows/auto-review.yml
name: Auto Review

on:
  pull_request:
    types: [opened, synchronize, reopened]

jobs:
  auto-review:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install review tools
        run: npm install -g @microsoft/eslint-formatter-sarif

      - name: Run auto-review
        uses: dorny/test-reporter@v1
        if: success() || failure()
        with:
          name: Auto Review Results
          path: 'results.sarif'
          reporter: sarif
          fail-on-error: false

      - name: Comment PR with results
        uses: dorny/github-action-pr-comment@v1
        if: always()
        with:
          path: results.sarif
```

## Artifact Management

### Build Artifacts

```yaml
# .github/workflows/artifacts.yml
name: Build Artifacts

on:
  push:
    branches: [ main ]
  release:
    types: [published]

jobs:
  artifacts:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build release binary
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package binary
        run: |
          cd target/${{ matrix.target }}/release
          tar -czf multiagent-hive-${{ matrix.target }}.tar.gz multiagent-hive

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: multiagent-hive-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/multiagent-hive-${{ matrix.target }}.tar.gz

      - name: Upload to release
        if: github.event_name == 'release'
        uses: softprops/action-gh-release@v2
        with:
          files: target/${{ matrix.target }}/release/multiagent-hive-${{ matrix.target }}.tar.gz
```

### Docker Images

```yaml
# .github/workflows/docker.yml
name: Docker

on:
  push:
    branches: [ main ]
    tags: [ 'v*' ]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  docker:
    runs-on: ubuntu-latest

    permissions:
      contents: read
      packages: write

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
          platforms: linux/amd64,linux/arm64
```

## Monitoring and Alerting

### Pipeline Monitoring

```yaml
# .github/workflows/monitoring.yml
name: Pipeline Monitoring

on:
  workflow_run:
    workflows: ["CI", "Release"]
    types:
      - completed

jobs:
  monitor:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Check workflow status
        run: |
          if [ "${{ github.event.workflow_run.conclusion }}" == "failure" ]; then
            echo "Workflow failed: ${{ github.event.workflow_run.name }}"
            # Send alert
          fi

      - name: Collect metrics
        run: |
          # Collect pipeline metrics
          echo "Workflow: ${{ github.event.workflow_run.name }}"
          echo "Status: ${{ github.event.workflow_run.conclusion }}"
          echo "Duration: ${{ github.event.workflow_run.updated_at - github.event.workflow_run.created_at }}"

      - name: Send notifications
        if: failure()
        run: |
          # Send Slack notification
          curl -X POST -H 'Content-type: application/json' \
            --data '{"text":"CI/CD Pipeline Failed"}' \
            ${{ secrets.SLACK_WEBHOOK_URL }}
```

### Performance Monitoring

```yaml
# .github/workflows/performance.yml
name: Performance Monitoring

on:
  schedule:
    - cron: '0 */6 * * *'  # Every 6 hours
  workflow_dispatch:

jobs:
  performance:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: |
          cargo bench
          # Store results for comparison

      - name: Compare with baseline
        run: |
          # Compare current results with stored baseline
          # Alert if performance regression detected

      - name: Update baseline
        if: github.event_name == 'schedule'
        run: |
          # Update baseline with current results
```

## Security in CI/CD

### Secret Management

```yaml
# .github/workflows/secrets.yml
name: Secret Management

on:
  push:
    branches: [ main ]
    paths:
      - '.github/workflows/**'
      - 'secrets/**'

jobs:
  secrets:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Check for exposed secrets
        uses: gitleaks/gitleaks-action@v2

      - name: Validate secret references
        run: |
          # Check that all secret references in workflows exist
          grep -r "\${{ secrets\." .github/workflows/
          # Validate against repository secrets

      - name: Rotate secrets
        if: github.event_name == 'schedule'
        run: |
          # Rotate encryption keys
          # Update secret values
```

### Dependency Scanning

```yaml
# .github/workflows/dependency-scan.yml
name: Dependency Scan

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  push:
    branches: [ main ]
    paths:
      - '**/Cargo.toml'
      - '**/package.json'
      - '**/package-lock.json'

jobs:
  scan:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Audit Rust dependencies
        run: cargo audit

      - name: Audit Node.js dependencies
        working-directory: ./frontend
        run: npm audit --audit-level=moderate

      - name: Check for outdated dependencies
        run: |
          cargo outdated
          cd frontend && npm outdated

      - name: Generate SBOM
        run: |
          # Generate Software Bill of Materials
          syft . -o spdx-json > sbom.json

      - name: Upload SBOM
        uses: actions/upload-artifact@v4
        with:
          name: sbom
          path: sbom.json
```

## Best Practices

### Pipeline Design

- **Keep pipelines fast**: Parallel jobs, caching, incremental builds
- **Fail fast**: Stop pipeline on critical failures
- **Provide clear feedback**: Detailed logs, test reports, coverage reports
- **Handle failures gracefully**: Proper error handling, cleanup, notifications
- **Secure by default**: Least privilege, secret management, vulnerability scanning

### Maintenance

- **Regular updates**: Keep actions and tools updated
- **Monitor performance**: Track pipeline duration and resource usage
- **Review and optimize**: Regularly review and improve pipeline efficiency
- **Document processes**: Keep pipeline documentation up to date
- **Backup configurations**: Version control all pipeline configurations

### Troubleshooting

- **Debug mode**: Enable debug logging for troubleshooting
- **Manual triggers**: Allow manual pipeline triggers for testing
- **Rollback procedures**: Have rollback plans for failed deployments
- **Incident response**: Document incident response procedures
- **Post-mortem**: Conduct post-mortems for major incidents

## Next Steps

- **Configuration**: See [docs/configuration.md](configuration.md)
- **Deployment**: See [docs/deployment.md](deployment.md)
- **Security**: See [docs/security-hardening.md](security-hardening.md)
- **Monitoring**: See [docs/observability.md](observability.md)