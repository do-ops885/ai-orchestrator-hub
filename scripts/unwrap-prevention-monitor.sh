#!/bin/bash
# Unwrap() Prevention Monitoring Script
# Prevents unwrap() calls from entering production code

set -euo pipefail

# Configuration
REPO_ROOT="$(git rev-parse --show-toplevel)"
LOG_FILE="${REPO_ROOT}/logs/unwrap-monitor.log"
ALERT_WEBHOOK="${UNWRAP_ALERT_WEBHOOK:-}"
PRODUCTION_PATHS=(
    "backend/src"
    "!backend/src/tests"
    "!backend/src/**/*test*.rs"
    "!backend/src/**/tests.rs"
    "!backend/examples"
    "!backend/benches"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Alert function
send_alert() {
    local severity="$1"
    local message="$2"
    local file_count="$3"
    
    log "🚨 ALERT [$severity]: $message"
    
    if [[ -n "$ALERT_WEBHOOK" ]]; then
        curl -X POST "$ALERT_WEBHOOK" \
            -H "Content-Type: application/json" \
            -d "{
                \"text\": \"🚨 AI Orchestrator Hub - Unwrap() Alert\",
                \"attachments\": [{
                    \"color\": \"danger\",
                    \"fields\": [
                        {\"title\": \"Severity\", \"value\": \"$severity\", \"short\": true},
                        {\"title\": \"Files Affected\", \"value\": \"$file_count\", \"short\": true},
                        {\"title\": \"Message\", \"value\": \"$message\", \"short\": false},
                        {\"title\": \"Repository\", \"value\": \"ai-orchestrator-hub\", \"short\": true},
                        {\"title\": \"Timestamp\", \"value\": \"$(date -u)\", \"short\": true}
                    ]
                }]
            }" 2>/dev/null || log "Failed to send webhook alert"
    fi
}

# Check for unwrap() calls in production code
check_unwrap_calls() {
    local scan_type="${1:-full}"
    local file_pattern="${2:-}"
    
    log "Starting unwrap() scan (type: $scan_type)"
    
    # Build find command with exclusions
    local find_cmd="find ${REPO_ROOT}/backend/src -name '*.rs' -type f"
    
    # Exclude test files and directories
    find_cmd+=" ! -path '*/tests/*'"
    find_cmd+=" ! -path '*/test_*'"
    find_cmd+=" ! -name '*test*.rs'"
    find_cmd+=" ! -name 'tests.rs'"
    find_cmd+=" ! -path '*/examples/*'"
    find_cmd+=" ! -path '*/benches/*'"
    
    # Add specific file pattern if provided
    if [[ -n "$file_pattern" ]]; then
        find_cmd+=" -path '*$file_pattern*'"
    fi
    
    # Find unwrap() and expect() calls
    local unwrap_files=$(eval "$find_cmd" | xargs grep -l "\.unwrap()\|\.expect(" 2>/dev/null || true)
    local unwrap_count=0
    if [[ -n "$unwrap_files" ]]; then
        unwrap_count=$(echo "$unwrap_files" | wc -l)
    fi
    
    if [[ $unwrap_count -gt 0 ]]; then
        echo -e "${RED}❌ CRITICAL ALERT: Found unwrap() calls in production code!${NC}"
        echo -e "${RED}Files with unwrap() calls:${NC}"
        
        while IFS= read -r file; do
            if [[ -n "$file" ]]; then
                echo -e "${RED}  📁 $file${NC}"
                # Show specific lines with unwrap() calls
                grep -n "\.unwrap()\|\.expect(" "$file" | head -5 | while IFS= read -r line; do
                    echo -e "${YELLOW}    $line${NC}"
                done
            fi
        done <<< "$unwrap_files"
        
        send_alert "CRITICAL" "Found unwrap() calls in production code" "$unwrap_count"
        return 1
    else
        echo -e "${GREEN}✅ SUCCESS: No unwrap() calls found in production code${NC}"
        log "✅ Unwrap() scan completed successfully - no issues found"
        return 0
    fi
}

# Check recent commits for unwrap() additions
check_recent_commits() {
    local since="${1:-1 hour ago}"
    
    log "Checking commits since: $since"
    
    # Get recent commits
    local recent_commits=$(git log --since="$since" --pretty=format:"%H" 2>/dev/null || true)
    
    if [[ -z "$recent_commits" ]]; then
        log "No recent commits found"
        return 0
    fi
    
    # Check each commit for unwrap() additions
    while IFS= read -r commit; do
        if [[ -n "$commit" ]]; then
            local added_unwraps=$(git show "$commit" --name-only --pretty="" | \
                grep "backend/src.*\.rs$" | \
                grep -v test | \
                xargs git show "$commit" -- 2>/dev/null | \
                grep "^+" | \
                grep "\.unwrap()\|\.expect(" || true)
            
            if [[ -n "$added_unwraps" ]]; then
                local commit_info=$(git show --oneline -s "$commit")
                send_alert "HIGH" "Commit $commit added unwrap() calls: $commit_info" "1"
                echo -e "${RED}⚠️  Commit $commit added unwrap() calls${NC}"
                echo "$added_unwraps"
                return 1
            fi
        fi
    done <<< "$recent_commits"
    
    return 0
}

# Pre-commit hook integration
install_pre_commit_hook() {
    local hook_file="${REPO_ROOT}/.git/hooks/pre-commit"
    
    log "Installing pre-commit hook for unwrap() prevention"
    
    cat > "$hook_file" << 'EOF'
#!/bin/bash
# Unwrap() Prevention Pre-commit Hook

echo "🔍 Checking for unwrap() calls in staged files..."

# Get staged .rs files in production paths
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep "backend/src.*\.rs$" | grep -v test || true)

if [[ -n "$STAGED_FILES" ]]; then
    # Check staged content for unwrap() calls
    UNWRAP_FOUND=false
    
    while IFS= read -r file; do
        if [[ -n "$file" ]]; then
            # Check staged content (not working directory)
            STAGED_CONTENT=$(git show ":$file" 2>/dev/null || true)
            if echo "$STAGED_CONTENT" | grep -q "\.unwrap()\|\.expect("; then
                echo "❌ ERROR: Found unwrap() call in staged file: $file"
                echo "   Lines with unwrap():"
                echo "$STAGED_CONTENT" | grep -n "\.unwrap()\|\.expect(" | head -3
                UNWRAP_FOUND=true
            fi
        fi
    done <<< "$STAGED_FILES"
    
    if [[ "$UNWRAP_FOUND" == "true" ]]; then
        echo ""
        echo "🚫 COMMIT REJECTED: unwrap() calls found in production code"
        echo "   Please replace unwrap() calls with proper error handling:"
        echo "   - Use unwrap_or(default_value) for safe defaults"
        echo "   - Use match or if-let for explicit error handling"
        echo "   - Use ? operator for error propagation"
        echo ""
        exit 1
    fi
fi

echo "✅ No unwrap() calls found in staged files"
EOF

    chmod +x "$hook_file"
    log "✅ Pre-commit hook installed successfully"
}

# GitHub Actions workflow for CI/CD
create_github_workflow() {
    local workflow_dir="${REPO_ROOT}/.github/workflows"
    local workflow_file="$workflow_dir/unwrap-prevention.yml"
    
    mkdir -p "$workflow_dir"
    
    log "Creating GitHub Actions workflow for unwrap() prevention"
    
    cat > "$workflow_file" << 'EOF'
name: Unwrap() Prevention Check

on:
  push:
    branches: [ main, develop ]
    paths: 
      - 'backend/src/**/*.rs'
      - '!backend/src/tests/**'
      - '!backend/src/**/*test*.rs'
  pull_request:
    branches: [ main, develop ]
    paths:
      - 'backend/src/**/*.rs'
      - '!backend/src/tests/**'
      - '!backend/src/**/*test*.rs'

jobs:
  unwrap-check:
    name: Check for unwrap() calls
    runs-on: ubuntu-latest
    
    steps:
    - name: Checkout code
      uses: actions/checkout@v4
      
    - name: Check for unwrap() calls in production code
      run: |
        echo "🔍 Scanning for unwrap() calls in production code..."
        
        # Find production Rust files (exclude tests)
        PRODUCTION_FILES=$(find backend/src -name "*.rs" -type f \
          ! -path "*/tests/*" \
          ! -path "*/test_*" \
          ! -name "*test*.rs" \
          ! -name "tests.rs" \
          ! -path "*/examples/*" \
          ! -path "*/benches/*")
        
        UNWRAP_FOUND=false
        UNWRAP_COUNT=0
        
        for file in $PRODUCTION_FILES; do
          if grep -q "\.unwrap()\|\.expect(" "$file" 2>/dev/null; then
            echo "❌ Found unwrap() call in: $file"
            grep -n "\.unwrap()\|\.expect(" "$file" | head -3
            UNWRAP_FOUND=true
            UNWRAP_COUNT=$((UNWRAP_COUNT + 1))
          fi
        done
        
        if [[ "$UNWRAP_FOUND" == "true" ]]; then
          echo ""
          echo "🚫 CRITICAL: Found unwrap() calls in $UNWRAP_COUNT production files"
          echo "This violates our security policy and could cause runtime panics."
          echo ""
          echo "Please replace unwrap() calls with proper error handling:"
          echo "- Use unwrap_or(default) for safe defaults"
          echo "- Use match or if-let for explicit handling"
          echo "- Use ? operator for error propagation"
          echo "- Use Result<T, E> return types"
          exit 1
        else
          echo "✅ SUCCESS: No unwrap() calls found in production code"
        fi
        
    - name: Notify on failure
      if: failure()
      run: |
        echo "::error::Unwrap() calls detected in production code. See job output for details."
EOF

    log "✅ GitHub Actions workflow created: $workflow_file"
}

# Create monitoring dashboard configuration
create_monitoring_config() {
    local config_file="${REPO_ROOT}/monitoring/unwrap-prevention.yml"
    
    mkdir -p "${REPO_ROOT}/monitoring"
    
    cat > "$config_file" << 'EOF'
# Unwrap() Prevention Monitoring Configuration

alerts:
  - name: unwrap_calls_detected
    description: "Unwrap() calls detected in production code"
    severity: critical
    conditions:
      - metric: "code_quality.unwrap_calls"
        operator: ">"
        threshold: 0
    actions:
      - type: webhook
        url: "${UNWRAP_ALERT_WEBHOOK}"
      - type: email
        recipients: ["security@company.com", "dev-team@company.com"]
      - type: slack
        channel: "#alerts-critical"

  - name: unwrap_commit_added
    description: "Commit added unwrap() calls to production code"
    severity: high
    conditions:
      - metric: "git.unwrap_additions"
        operator: ">"
        threshold: 0
    actions:
      - type: block_merge
        message: "Cannot merge: unwrap() calls detected"
      - type: slack
        channel: "#dev-alerts"

monitoring:
  scan_interval: "5m"
  retention_period: "30d"
  
  checks:
    - name: "production_unwrap_scan"
      type: "file_pattern"
      paths: ["backend/src/**/*.rs"]
      exclude: ["*/tests/*", "*test*.rs", "tests.rs"]
      pattern: "\.unwrap\(\)|\.expect\("
      
    - name: "commit_unwrap_scan"
      type: "git_diff"
      since: "1 hour ago"
      paths: ["backend/src/**/*.rs"]
      exclude: ["*/tests/*", "*test*.rs"]
      pattern: "^\+.*\.unwrap\(\)|^\+.*\.expect\("

reporting:
  daily_summary: true
  weekly_trends: true
  metrics_export: "prometheus"
EOF

    log "✅ Monitoring configuration created: $config_file"
}

# Create Rust-specific linting rules
create_clippy_config() {
    local clippy_file="${REPO_ROOT}/backend/clippy.toml"
    
    log "Updating Clippy configuration for unwrap() prevention"
    
    # Check if clippy.toml exists and add unwrap rules
    if [[ -f "$clippy_file" ]]; then
        # Add unwrap prevention rules if not already present
        if ! grep -q "disallowed-methods" "$clippy_file"; then
            cat >> "$clippy_file" << 'EOF'

# Unwrap() Prevention Rules
disallowed-methods = [
    "std::result::Result::unwrap",
    "std::option::Option::unwrap",
    "std::result::Result::expect",
    "std::option::Option::expect",
]

# Additional safety rules
too-many-arguments-threshold = 5
type-complexity-threshold = 100
EOF
        fi
    else
        cat > "$clippy_file" << 'EOF'
# Clippy configuration for AI Orchestrator Hub

# Unwrap() Prevention Rules (CRITICAL)
disallowed-methods = [
    "std::result::Result::unwrap",
    "std::option::Option::unwrap", 
    "std::result::Result::expect",
    "std::option::Option::expect",
]

# Code quality rules
too-many-arguments-threshold = 5
type-complexity-threshold = 100
cognitive-complexity-threshold = 30

# Performance rules
single-char-pattern = "deny"
unnecessary-cast = "deny"
redundant-clone = "deny"
EOF
    fi
    
    log "✅ Clippy configuration updated with unwrap() prevention rules"
}

# Create automated fix suggestions
create_fix_suggestions() {
    local suggestions_file="${REPO_ROOT}/docs/UNWRAP_ALTERNATIVES.md"
    
    cat > "$suggestions_file" << 'EOF'
# Unwrap() Alternatives Guide

## 🚫 Forbidden Patterns
```rust
// ❌ NEVER use these in production code:
value.unwrap()              // Can panic!
value.expect("message")     // Can panic!
```

## ✅ Safe Alternatives

### 1. Use `unwrap_or()` for Default Values
```rust
// ❌ Dangerous
let config = parse_config().unwrap();

// ✅ Safe with default
let config = parse_config().unwrap_or_default();
let config = parse_config().unwrap_or(Config::new());
```

### 2. Use `unwrap_or_else()` for Computed Defaults
```rust
// ✅ Safe with computed default
let config = parse_config().unwrap_or_else(|| {
    log::warn!("Using default config");
    Config::default()
});
```

### 3. Use Pattern Matching
```rust
// ✅ Explicit error handling
match parse_config() {
    Ok(config) => process_config(config),
    Err(e) => {
        log::error!("Config parse failed: {}", e);
        return Err(HiveError::ConfigurationError { 
            reason: e.to_string() 
        });
    }
}
```

### 4. Use `if let` for Optional Values
```rust
// ✅ Safe optional handling
if let Some(value) = optional_value {
    process_value(value);
} else {
    log::debug!("No value provided, using default behavior");
}
```

### 5. Use `?` Operator for Error Propagation
```rust
// ✅ Propagate errors up the call stack
fn process_data() -> HiveResult<ProcessedData> {
    let config = parse_config()?;  // Propagates error
    let data = load_data(config)?; // Propagates error
    Ok(process(data))
}
```

### 6. Use `map_or()` for Transformations
```rust
// ✅ Safe transformation with default
let result = maybe_value.map_or(String::new(), |v| v.to_string());
let result = maybe_value.map_or_else(|| default_value(), |v| transform(v));
```

### 7. For Partial Comparison (floating point)
```rust
// ✅ Safe comparison with default ordering
values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
```

## 🛠️ Quick Fix Patterns

### File I/O Operations
```rust
// ❌ Can panic
let content = std::fs::read_to_string("file.txt").unwrap();

// ✅ Proper error handling
let content = std::fs::read_to_string("file.txt")
    .map_err(|e| HiveError::IoError { 
        operation: "read config file".to_string(),
        source: e 
    })?;
```

### JSON Parsing
```rust
// ❌ Can panic
let data: Config = serde_json::from_str(&json).unwrap();

// ✅ Proper error handling
let data: Config = serde_json::from_str(&json)
    .map_err(|e| HiveError::SerializationError { 
        message: format!("Failed to parse config: {}", e) 
    })?;
```

### Thread Operations
```rust
// ❌ Can panic
let result = handle.join().unwrap();

// ✅ Proper error handling
let result = handle.join().map_err(|e| HiveError::ThreadError {
    message: format!("Thread join failed: {:?}", e)
})?;
```

## 🔧 IDE Integration

### VS Code Settings
Add to `.vscode/settings.json`:
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.extraArgs": ["--", "-W", "clippy::unwrap_used"]
}
```

### Rust Analyzer Configuration
Add to `.vscode/settings.json`:
```json
{
    "rust-analyzer.diagnostics.disabled": [],
    "rust-analyzer.diagnostics.remapPrefix": {},
    "rust-analyzer.check.overrideCommand": [
        "cargo", "clippy", "--workspace", "--message-format=json",
        "--", "-W", "clippy::unwrap_used", "-W", "clippy::expect_used"
    ]
}
```

## 📋 Review Checklist

Before committing code, verify:
- [ ] No `.unwrap()` calls in production code
- [ ] No `.expect()` calls in production code  
- [ ] All errors are properly handled or propagated
- [ ] Default values are provided where appropriate
- [ ] Error messages are descriptive and actionable
- [ ] Tests cover error scenarios

## 🚨 Emergency Fix Script

If unwrap() calls are detected in production:

```bash
# Run the emergency fix script
./scripts/unwrap-prevention-monitor.sh check_unwrap_calls

# Install pre-commit hooks to prevent future issues
./scripts/unwrap-prevention-monitor.sh install_pre_commit_hook
```
EOF

    log "✅ Unwrap alternatives guide created: $suggestions_file"
}

# Main execution
main() {
    local action="${1:-full_setup}"
    
    # Ensure log directory exists
    mkdir -p "$(dirname "$LOG_FILE")"
    
    case "$action" in
        "check_unwrap_calls")
            check_unwrap_calls "${2:-full}" "${3:-}"
            ;;
        "check_recent_commits")
            check_recent_commits "${2:-1 hour ago}"
            ;;
        "install_pre_commit_hook")
            install_pre_commit_hook
            ;;
        "create_github_workflow")
            create_github_workflow
            ;;
        "create_monitoring_config")
            create_monitoring_config
            ;;
        "create_clippy_config")
            create_clippy_config
            ;;
        "create_fix_suggestions")
            create_fix_suggestions
            ;;
        "full_setup")
            echo "🔧 Setting up complete unwrap() prevention monitoring..."
            check_unwrap_calls
            install_pre_commit_hook
            create_github_workflow
            create_monitoring_config
            create_clippy_config
            create_fix_suggestions
            echo -e "${GREEN}✅ Unwrap() prevention monitoring setup complete!${NC}"
            ;;
        *)
            echo "Usage: $0 {check_unwrap_calls|check_recent_commits|install_pre_commit_hook|create_github_workflow|create_monitoring_config|create_clippy_config|create_fix_suggestions|full_setup}"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"