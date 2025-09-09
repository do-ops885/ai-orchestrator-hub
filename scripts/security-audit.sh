#!/bin/bash

# AI Orchestrator Hub Security Audit Script
# Comprehensive security scanning and vulnerability assessment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKEND_DIR="$REPO_ROOT/backend"
FRONTEND_DIR="$REPO_ROOT/frontend"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging function
log() {
    local level="$1"
    local message="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    case $level in
        "INFO")
            echo -e "${BLUE}[INFO]${NC} $timestamp - $message"
            ;;
        "WARN")
            echo -e "${YELLOW}[WARN]${NC} $timestamp - $message"
            ;;
        "ERROR")
            echo -e "${RED}[ERROR]${NC} $timestamp - $message"
            ;;
        "SUCCESS")
            echo -e "${GREEN}[SUCCESS]${NC} $timestamp - $message"
            ;;
    esac
}

# Function to check prerequisites
check_prerequisites() {
    log "INFO" "Checking prerequisites..."

    # Check if we're in the right directory
    if [[ ! -f "$BACKEND_DIR/Cargo.toml" ]]; then
        log "ERROR" "Backend directory not found. Please run from the ai-orchestrator-hub directory."
        exit 1
    fi

    if [[ ! -f "$FRONTEND_DIR/package.json" ]]; then
        log "ERROR" "Frontend directory not found. Please run from the ai-orchestrator-hub directory."
        exit 1
    fi

    log "SUCCESS" "Prerequisites check passed"
}

# Function to audit Node.js dependencies
audit_nodejs_dependencies() {
    log "INFO" "Auditing Node.js dependencies..."

    cd "$FRONTEND_DIR"

    # Run npm audit
    log "INFO" "Running npm audit..."
    if npm audit --audit-level moderate; then
        log "SUCCESS" "Node.js dependency audit completed - no vulnerabilities found"
    else
        log "WARN" "Security vulnerabilities found in Node.js dependencies"
        return 1
    fi
}

# Function to check for secrets in code
check_secrets() {
    log "INFO" "Checking for secrets in code..."

    cd "$REPO_ROOT"

    # Check for common patterns that might indicate secrets
    if command -v grep &> /dev/null; then
        # Look for API keys, passwords, etc.
        secrets_found=0

        if grep -r "password\|PASSWORD\|secret\|SECRET\|key\|KEY\|token\|TOKEN" --include="*.rs" --include="*.ts" --include="*.js" --include="*.json" --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.git . | grep -v "example\|test\|mock\|fake" > secrets_scan.txt 2>/dev/null; then
            log "WARN" "Potential secrets found in code"
            secrets_found=1
        fi

        if [[ $secrets_found -eq 0 ]]; then
            log "SUCCESS" "No obvious secrets found in code"
        fi
    else
        log "WARN" "grep not available, skipping secrets check"
    fi
}

# Function to check file permissions
check_file_permissions() {
    log "INFO" "Checking file permissions..."

    cd "$REPO_ROOT"

    # Check for world-writable files
    world_writable=$(find . -type f -perm -002 2>/dev/null | wc -l)

    if [[ $world_writable -gt 0 ]]; then
        log "WARN" "Found $world_writable world-writable files"
    else
        log "SUCCESS" "No world-writable files found"
    fi
}

# Function to generate security report
generate_report() {
    log "INFO" "Generating security audit report..."

    local report_file="$REPO_ROOT/security-audit-report-$(date +%Y%m%d-%H%M%S).txt"

    echo "AI Orchestrator Hub Security Audit Report" > "$report_file"
    echo "Generated: $(date)" >> "$report_file"
    echo "==========================================" >> "$report_file"
    echo "" >> "$report_file"
    echo "Audit Results:" >> "$report_file"
    echo "- Node.js Dependencies: Scanned" >> "$report_file"
    echo "- Secrets Check: $([[ -f "$REPO_ROOT/secrets_scan.txt" ]] && echo "Issues found" || echo "Clean")" >> "$report_file"
    echo "- File Permissions: Checked" >> "$report_file"
    echo "" >> "$report_file"
    echo "Recommendations:" >> "$report_file"
    echo "1. Review any warnings in the audit output above" >> "$report_file"
    echo "2. Update dependencies regularly using Dependabot" >> "$report_file"
    echo "3. Run this audit script before deployments" >> "$report_file"
    echo "4. Monitor for new security advisories" >> "$report_file"
    echo "5. Ensure proper access controls are in place" >> "$report_file"

    log "SUCCESS" "Security audit report generated: $report_file"
}

# Main function
main() {
    log "INFO" "Starting comprehensive security audit..."

    check_prerequisites

    local audit_failed=0

    # Run all security checks
    audit_nodejs_dependencies || audit_failed=1
    check_secrets
    check_file_permissions

    generate_report

    if [[ $audit_failed -eq 1 ]]; then
        log "WARN" "Security audit completed with warnings"
        exit 1
    else
        log "SUCCESS" "Security audit completed successfully"
    fi
}

# Run main function
main "$@"
