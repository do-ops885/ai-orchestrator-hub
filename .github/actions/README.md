# GitHub Actions Logging System

This directory contains reusable composite actions for consistent structured logging across all GitHub Actions workflows.

## Overview

The logging system addresses the persistence issue where each GitHub Actions step runs in a separate shell environment. Instead of duplicating logging code across workflows, we use composite actions that recreate the logging helper script in each step that needs it.

## Available Actions

### `setup-logging`

Sets up structured logging helper script for GitHub Actions workflows.

**Usage:**
```yaml
- name: Setup structured logging
  uses: ./.github/actions/setup-logging
  with:
    correlation-id: ${{ github.run_id }}-${{ github.run_attempt }}
```

**Features:**
- Creates `log_helper.sh` script with structured JSON logging
- Includes `log_structured()` function for consistent log formatting
- Includes `retry_with_backoff()` function for resilient command execution
- Includes `monitor_resources()` function for system monitoring
- Automatic correlation ID tracking across workflow runs

### `cleanup-logging`

Cleans up the logging helper script and provides final status logging.

**Usage:**
```yaml
- name: Cleanup logging
  if: always()
  uses: ./.github/actions/cleanup-logging
  with:
    final-status: ${{ job.status }}
```

**Features:**
- Automatically cleans up `log_helper.sh` script
- Logs final job status with correlation ID
- Monitors final resource usage
- Runs on job completion (success or failure)

## Logging Functions

### `log_structured(level, message, extra)`

Logs a structured JSON message.

**Parameters:**
- `level`: Log level (INFO, WARN, ERROR, DEBUG)
- `message`: Log message string
- `extra`: Optional JSON string with additional context

**Example:**
```bash
log_structured "INFO" "Build started" '{"target": "x86_64-unknown-linux-gnu"}'
```

### `retry_with_backoff(command)`

Retries a command with exponential backoff on failure.

**Parameters:**
- `command`: Command to execute with retry logic

**Environment Variables:**
- `RETRY_MAX_ATTEMPTS`: Maximum retry attempts (default: 3)
- `RETRY_INITIAL_DELAY`: Initial delay in seconds (default: 5)

**Example:**
```bash
retry_with_backoff "cargo build --release"
```

### `monitor_resources(step)`

Logs current system resource usage.

**Parameters:**
- `step`: Step name for resource monitoring context

**Example:**
```bash
monitor_resources "build_started"
```

## Usage in Workflows

### Basic Setup

```yaml
jobs:
  my-job:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup structured logging
        uses: ./.github/actions/setup-logging
        with:
          correlation-id: ${{ github.run_id }}-${{ github.run_attempt }}

      - name: Run build
        run: |
          source log_helper.sh
          log_structured "INFO" "Starting build"
          retry_with_backoff "cargo build --release"
          monitor_resources "build_complete"

      - name: Cleanup logging
        if: always()
        uses: ./.github/actions/cleanup-logging
        with:
          final-status: ${{ job.status }}
```

### Advanced Usage with Resource Monitoring

```yaml
- name: Setup structured logging
  uses: ./.github/actions/setup-logging

- name: Build with monitoring
  run: |
    source log_helper.sh
    log_structured "INFO" "Build started"
    monitor_resources "build_start"

    # Build process with monitoring
    (
      while true; do
        monitor_resources "build_in_progress"
        sleep 30
      done
    ) &
    MONITOR_PID=$!

    retry_with_backoff "cargo build --release --all-features"

    # Stop monitoring
    kill $MONITOR_PID 2>/dev/null || true
    monitor_resources "build_complete"
```

## Log Output Format

All logs are output in structured JSON format:

```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "INFO",
  "message": "Build completed successfully",
  "workflow": "CI",
  "job": "build",
  "run_id": "123456789",
  "run_attempt": "1",
  "correlation_id": "123456789-1",
  "context": {
    "target": "x86_64-unknown-linux-gnu",
    "duration_seconds": 45
  }
}
```

## Benefits

1. **Consistency**: Uniform logging format across all workflows
2. **Persistence**: Script is recreated in each step, solving the shell environment issue
3. **Observability**: Structured JSON logs enable better monitoring and analysis
4. **Resilience**: Built-in retry logic for flaky operations
5. **Resource Tracking**: Automatic monitoring of system resources
6. **Correlation**: Consistent correlation IDs for tracking across workflow runs
7. **Cleanup**: Automatic cleanup prevents script accumulation

## Migration Guide

To migrate existing workflows:

1. Replace inline logging script creation with `setup-logging` action
2. Add `cleanup-logging` action at the end of jobs
3. Update existing `source log_helper.sh` calls to work with the new system
4. Remove manual script cleanup code

### Before
```yaml
- name: Setup logging
  run: |
    cat > log_helper.sh << 'EOF'
    # logging code...
    EOF
    chmod +x log_helper.sh
    source log_helper.sh

# ... workflow steps ...

- name: Cleanup
  run: rm -f log_helper.sh
```

### After
```yaml
- name: Setup structured logging
  uses: ./.github/actions/setup-logging

# ... workflow steps ...

- name: Cleanup logging
  if: always()
  uses: ./.github/actions/cleanup-logging
  with:
    final-status: ${{ job.status }}
```

## Troubleshooting

### Common Issues

1. **"log_helper.sh: command not found"**
   - Ensure `setup-logging` action runs before using logging functions
   - Check that `source log_helper.sh` is called in each step

2. **"jq: command not found"**
   - The setup action checks for jq availability
   - Falls back to simple logging if jq is not available

3. **Correlation ID issues**
   - Ensure correlation-id is passed to setup-logging action
   - Check that CORRELATION_ID environment variable is set

4. **Resource monitoring failures**
   - Some monitoring commands may fail on certain runners
   - Functions include error handling and fallbacks

### Debug Mode

Enable debug logging by setting the log level:

```bash
export LOG_LEVEL=DEBUG
```

This will include additional debug information in log output.
