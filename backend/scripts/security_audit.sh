#!/bin/bash

# AI Orchestrator Hub - Security Audit Script
# This script performs comprehensive security validation and testing

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="${PROJECT_ROOT}/security-reports"
LOG_FILE="${REPORT_DIR}/security-audit-$(date +%Y%m%d_%H%M%S).log"

# Create report directory
mkdir -p "$REPORT_DIR"

# Logging function
log() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${timestamp} [${level}] ${message}" | tee -a "$LOG_FILE"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" | tee -a "$LOG_FILE"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*" | tee -a "$LOG_FILE"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check Rust installation
check_rust_installation() {
    log_info "Checking Rust installation..."
    
    if ! command_exists cargo; then
        log_error "Cargo not found. Please install Rust toolchain."
        exit 1
    fi
    
    local rust_version=$(cargo --version | cut -d' ' -f2)
    log_info "Rust version: $rust_version"
    
    if ! command_exists rustc; then
        log_error "Rust compiler not found."
        exit 1
    fi
    
    log_success "Rust installation verified"
}

# Function to install cargo-audit if not present
install_cargo_audit() {
    log_info "Checking cargo-audit installation..."
    
    if ! command_exists cargo-audit; then
        log_warn "cargo-audit not found. Installing..."
        cargo install cargo-audit
        log_success "cargo-audit installed successfully"
    else
        log_info "cargo-audit is already installed"
    fi
}

# Function to run cargo audit
run_cargo_audit() {
    log_info "Running cargo audit for vulnerability scanning..."
    
    local audit_report="${REPORT_DIR}/cargo-audit-report-$(date +%Y%m%d_%H%M%S).json"
    
    if cargo audit --json > "$audit_report" 2>&1; then
        log_success "Cargo audit completed successfully"
        
        # Parse results
        local vulnerabilities=$(jq '.vulnerabilities_found // 0' "$audit_report")
        local warnings=$(jq '.warnings | length' "$audit_report")
        
        if [[ "$vulnerabilities" -gt 0 ]]; then
            log_error "Found $vulnerabilities vulnerabilities"
            return 1
        else
            log_success "No vulnerabilities found"
        fi
        
        if [[ "$warnings" -gt 0 ]]; then
            log_warn "Found $warnings warnings"
        fi
        
    else
        log_error "Cargo audit failed"
        return 1
    fi
}

# Function to check for unwrap() calls
check_unwrap_calls() {
    log_info "Checking for unwrap() calls in source code..."
    
    local unwrap_report="${REPORT_DIR}/unwrap-calls-$(date +%Y%m%d_%H%M%S).txt"
    
    # Find all .rs files in src directory
    find "$PROJECT_ROOT/src" -name "*.rs" -exec grep -H -n "\.unwrap()" {} \; > "$unwrap_report" 2>&1 || true
    
    local unwrap_count=$(wc -l < "$unwrap_report" | tr -d ' ')
    
    if [[ "$unwrap_count" -gt 0 ]]; then
        log_error "Found $unwrap_count unwrap() calls"
        log_warn "Unwrap calls found in:"
        head -10 "$unwrap_report" | while read line; do
            log_warn "  $line"
        done
        
        if [[ "$unwrap_count" -gt 10 ]]; then
            log_error "Too many unwrap calls ($unwrap_count). This is a critical security issue."
            return 1
        fi
    else
        log_success "No unwrap() calls found"
    fi
}

# Function to check for expect() calls
check_expect_calls() {
    log_info "Checking for expect() calls in source code..."
    
    local expect_report="${REPORT_DIR}/expect-calls-$(date +%Y%m%d_%H%M%S).txt"
    
    # Find all .rs files in src directory
    find "$PROJECT_ROOT/src" -name "*.rs" -exec grep -H -n "\.expect(" {} \; > "$expect_report" 2>&1 || true
    
    local expect_count=$(wc -l < "$expect_report" | tr -d ' ')
    
    if [[ "$expect_count" -gt 0 ]]; then
        log_error "Found $expect_count expect() calls"
        log_warn "Expect calls found in:"
        head -10 "$expect_report" | while read line; do
            log_warn "  $line"
        done
        
        if [[ "$expect_count" -gt 10 ]]; then
            log_error "Too many expect calls ($expect_count). This is a critical security issue."
            return 1
        fi
    else
        log_success "No expect() calls found"
    fi
}

# Function to run clippy with security lints
run_clippy_security() {
    log_info "Running Clippy with security lints..."
    
    local clippy_report="${REPORT_DIR}/clippy-report-$(date +%Y%m%d_%H%M%S).txt"
    
    if cargo clippy --all-targets --all-features -- -D warnings > "$clippy_report" 2>&1; then
        log_success "Clippy security checks passed"
    else
        log_error "Clippy found security issues"
        log_warn "Clippy output:"
        tail -20 "$clippy_report" | while read line; do
            log_warn "  $line"
        done
        return 1
    fi
}

# Function to run security tests
run_security_tests() {
    log_info "Running security tests..."
    
    if cargo test security_audit_tests --lib 2>&1; then
        log_success "Security tests passed"
    else
        log_error "Security tests failed"
        return 1
    fi
}

# Function to check for unsafe code
check_unsafe_code() {
    log_info "Checking for unsafe code blocks..."
    
    local unsafe_report="${REPORT_DIR}/unsafe-code-$(date +%Y%m%d_%H%M%S).txt"
    
    find "$PROJECT_ROOT/src" -name "*.rs" -exec grep -H -n "unsafe" {} \; > "$unsafe_report" 2>&1 || true
    
    local unsafe_count=$(wc -l < "$unsafe_report" | tr -d ' ')
    
    if [[ "$unsafe_count" -gt 0 ]]; then
        log_warn "Found $unsafe_count unsafe code blocks"
        log_warn "Unsafe code found in:"
        cat "$unsafe_report" | while read line; do
            log_warn "  $line"
        done
    else
        log_success "No unsafe code found"
    fi
}

# Function to check dependency versions
check_dependency_versions() {
    log_info "Checking dependency versions for known issues..."
    
    local cargo_toml="$PROJECT_ROOT/Cargo.toml"
    
    if [[ ! -f "$cargo_toml" ]]; then
        log_error "Cargo.toml not found"
        return 1
    fi
    
    # Check for specific versions that might have security issues
    local critical_deps=("tokio" "jsonwebtoken" "argon2" "axum" "rusqlite")
    
    for dep in "${critical_deps[@]}"; do
        local version=$(grep "^$dep = " "$cargo_toml" | sed 's/.*"\([^"]*\)".*/\1/')
        if [[ -n "$version" ]]; then
            log_info "$dep: $version"
            
            # Check for known vulnerable versions (simplified check)
            case "$dep" in
                "tokio")
                    if [[ "$version" < "1.20.0" ]]; then
                        log_warn "Tokio version $version may have security issues"
                    fi
                    ;;
                "jsonwebtoken")
                    if [[ "$version" < "8.0.0" ]]; then
                        log_warn "jsonwebtoken version $version may have security issues"
                    fi
                    ;;
                "argon2")
                    if [[ "$version" < "0.4.0" ]]; then
                        log_warn "argon2 version $version may have security issues"
                    fi
                    ;;
            esac
        fi
    done
}

# Function to check file permissions
check_file_permissions() {
    log_info "Checking file permissions..."
    
    local permission_issues=0
    
    # Check for world-writable files
    while IFS= read -r -d '' file; do
        if [[ -w "$file" && -O "$file" ]]; then
            log_warn "World-writable file: $file"
            ((permission_issues++))
        fi
    done < <(find "$PROJECT_ROOT" -type f -perm -o=w -print0 2>/dev/null)
    
    # Check for sensitive files with loose permissions
    local sensitive_files=("$PROJECT_ROOT/Cargo.toml" "$PROJECT_ROOT/settings/")
    for file in "${sensitive_files[@]}"; do
        if [[ -e "$file" ]]; then
            local perms=$(stat -f "%A" "$file" 2>/dev/null || stat -c "%a" "$file" 2>/dev/null)
            if [[ "$perms" =~ ^[67][0-9]{2}$ ]]; then
                log_warn "Sensitive file with loose permissions: $file ($perms)"
                ((permission_issues++))
            fi
        fi
    done
    
    if [[ "$permission_issues" -eq 0 ]]; then
        log_success "File permissions are secure"
    else
        log_error "Found $permission_issues file permission issues"
        return 1
    fi
}

# Function to check for hardcoded secrets
check_hardcoded_secrets() {
    log_info "Checking for hardcoded secrets..."
    
    local secrets_report="${REPORT_DIR}/hardcoded-secrets-$(date +%Y%m%d_%H%M%S).txt"
    
    # Common secret patterns
    local patterns=(
        "password\s*=\s*[\"'][^\"']{8,}[\"']"
        "secret\s*=\s*[\"'][^\"']{16,}[\"']"
        "api_key\s*=\s*[\"'][^\"']{16,}[\"']"
        "token\s*=\s*[\"'][^\"']{16,}[\"']"
        "private_key\s*=\s*[\"'][^\"']{20,}[\"']"
    )
    
    local found_secrets=0
    
    for pattern in "${patterns[@]}"; do
        if grep -r -i -E "$pattern" "$PROJECT_ROOT/src" --include="*.rs" >> "$secrets_report" 2>/dev/null; then
            ((found_secrets++))
        fi
    done
    
    if [[ "$found_secrets" -gt 0 ]]; then
        log_error "Found potential hardcoded secrets"
        log_warn "Secret patterns found in:"
        head -10 "$secrets_report" | while read line; do
            log_warn "  $line"
        done
        return 1
    else
        log_success "No hardcoded secrets found"
    fi
}

# Function to generate security report
generate_security_report() {
    log_info "Generating comprehensive security report..."
    
    local report_file="${REPORT_DIR}/security-audit-report-$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# Security Audit Report

**Date:** $(date '+%Y-%m-%d %H:%M:%S')  
**Project:** AI Orchestrator Hub  
**Audit Type:** Automated Security Scan  

## Executive Summary

This automated security audit was performed to identify potential security vulnerabilities and compliance issues in the AI Orchestrator Hub codebase.

## Scan Results

### Vulnerability Scanning
- **Cargo Audit:** $(grep -q "No vulnerabilities found" "$LOG_FILE" && echo "✅ PASSED" || echo "❌ FAILED")

### Code Quality
- **Unwrap() Calls:** $(grep -q "No unwrap() calls found" "$LOG_FILE" && echo "✅ PASSED" || echo "❌ FAILED")
- **Expect() Calls:** $(grep -q "No expect() calls found" "$LOG_FILE" && echo "✅ PASSED" || echo "❌ FAILED")
- **Unsafe Code:** $(grep -q "No unsafe code found" "$LOG_FILE" && echo "✅ PASSED" || echo "⚠️ WARNING")
- **Clippy Lints:** $(grep -q "Clippy security checks passed" "$LOG_FILE" && echo "✅ PASSED" || echo "❌ FAILED")

### Security Tests
- **Security Tests:** $(grep -q "Security tests passed" "$LOG_FILE" && echo "✅ PASSED" || echo "❌ FAILED")

### Configuration
- **Dependency Versions:** ✅ CHECKED
- **File Permissions:** $(grep -q "File permissions are secure" "$LOG_FILE" && echo "✅ PASSED" || echo "❌ FAILED")
- **Hardcoded Secrets:** $(grep -q "No hardcoded secrets found" "$LOG_FILE" && echo "✅ PASSED" || echo "❌ FAILED")

## Detailed Findings

### Critical Issues
$(grep -i "critical\|error" "$LOG_FILE" | grep -v "grep" | sed 's/^/- /')

### Warnings
$(grep -i "warn" "$LOG_FILE" | grep -v "grep" | sed 's/^/- /')

### Recommendations

1. **Immediate Actions:**
   - Address all unwrap() and expect() calls
   - Fix any identified vulnerabilities
   - Review and resolve clippy warnings

2. **Short-term Actions:**
   - Implement comprehensive input validation
   - Enhance error handling patterns
   - Add security monitoring

3. **Long-term Actions:**
   - Regular security audits
   - Continuous security testing
   - Security training for developers

## Files Generated

This audit generated the following detailed reports:
- $(basename "$LOG_FILE")
- $(basename "$audit_report" 2>/dev/null || echo "cargo-audit-report-*.json")
- $(basename "$unwrap_report" 2>/dev/null || echo "unwrap-calls-*.txt")
- $(basename "$expect_report" 2>/dev/null || echo "expect-calls-*.txt")
- $(basename "$clippy_report" 2>/dev/null || echo "clippy-report-*.txt")
- $(basename "$unsafe_report" 2>/dev/null || echo "unsafe-code-*.txt")
- $(basename "$secrets_report" 2>/dev/null || echo "hardcoded-secrets-*.txt")

## Next Steps

1. Review all failed checks and warnings
2. Implement necessary fixes
3. Re-run security audit after fixes
4. Schedule regular security audits

---

*Generated by automated security audit script*
EOF

    log_success "Security report generated: $report_file"
}

# Main execution function
main() {
    log_info "Starting comprehensive security audit..."
    log_info "Project root: $PROJECT_ROOT"
    log_info "Report directory: $REPORT_DIR"
    
    local overall_success=0
    
    # Check prerequisites
    check_rust_installation
    install_cargo_audit
    
    # Run security checks
    log_info "=== Phase 1: Vulnerability Scanning ==="
    run_cargo_audit || overall_success=1
    
    log_info "=== Phase 2: Code Quality Checks ==="
    check_unwrap_calls || overall_success=1
    check_expect_calls || overall_success=1
    check_unsafe_code
    run_clippy_security || overall_success=1
    
    log_info "=== Phase 3: Security Testing ==="
    run_security_tests || overall_success=1
    
    log_info "=== Phase 4: Configuration Checks ==="
    check_dependency_versions
    check_file_permissions || overall_success=1
    check_hardcoded_secrets || overall_success=1
    
    # Generate report
    generate_security_report
    
    # Final summary
    log_info "=== Security Audit Complete ==="
    
    if [[ "$overall_success" -eq 0 ]]; then
        log_success "🎉 All security checks passed!"
        exit 0
    else
        log_error "❌ Security issues found. Please review the report."
        exit 1
    fi
}

# Run main function
main "$@"