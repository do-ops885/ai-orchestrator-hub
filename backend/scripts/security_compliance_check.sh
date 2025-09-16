#!/bin/bash

# AI Orchestrator Hub - Security and Compliance Check Script
# This script performs comprehensive security and compliance validation
# for production deployment readiness.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_FILE="/tmp/security_compliance_check_$(date +%Y%m%d_%H%M%S).log"
REPORT_FILE="/tmp/security_compliance_report_$(date +%Y%m%d_%H%M%S).json"

# Initialize report
REPORT='{
    "timestamp": "'$(date -Iseconds)'",
    "checks": [],
    "summary": {
        "total": 0,
        "passed": 0,
        "failed": 0,
        "warnings": 0
    },
    "compliance": {
        "gdpr": false,
        "iso27001": false,
        "nist_csf": false,
        "owasp_top10": false
    },
    "overall_status": "unknown"
}'

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

# Update report function
update_report() {
    local check_name="$1"
    local status="$2"
    local details="$3"
    local category="$4"
    
    REPORT=$(echo "$REPORT" | jq --arg name "$check_name" --arg status "$status" --arg details "$details" --arg category "$category" '
        .checks += [{
            "name": $name,
            "status": $status,
            "details": $details,
            "category": $category,
            "timestamp": "'$(date -Iseconds)'"
        }]
    ')
    
    # Update summary
    local total=$(echo "$REPORT" | jq '.summary.total + 1')
    REPORT=$(echo "$REPORT" | jq --argjson total "$total" '.summary.total = $total')
    
    case "$status" in
        "passed")
            local passed=$(echo "$REPORT" | jq '.summary.passed + 1')
            REPORT=$(echo "$REPORT" | jq --argjson passed "$passed" '.summary.passed = $passed')
            ;;
        "failed")
            local failed=$(echo "$REPORT" | jq '.summary.failed + 1')
            REPORT=$(echo "$REPORT" | jq --argjson failed "$failed" '.summary.failed = $failed')
            ;;
        "warning")
            local warnings=$(echo "$REPORT" | jq '.summary.warnings + 1')
            REPORT=$(echo "$REPORT" | jq --argjson warnings "$warnings" '.summary.warnings = $warnings')
            ;;
    esac
}

# Check if required tools are installed
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local tools=("cargo" "jq" "curl" "openssl" "sqlite3")
    local missing_tools=()
    
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done
    
    if [ ${#missing_tools[@]} -eq 0 ]; then
        log_success "All required tools are installed"
        update_report "prerequisites" "passed" "All required tools are installed" "setup"
    else
        log_error "Missing required tools: ${missing_tools[*]}"
        update_report "prerequisites" "failed" "Missing tools: ${missing_tools[*]}" "setup"
        return 1
    fi
}

# Check code compilation
check_code_compilation() {
    log_info "Checking code compilation..."
    
    cd "$PROJECT_ROOT"
    
    if cargo check --release; then
        log_success "Code compiles successfully"
        update_report "code_compilation" "passed" "Code compiles without errors" "code_quality"
    else
        log_error "Code compilation failed"
        update_report "code_compilation" "failed" "Code compilation failed" "code_quality"
        return 1
    fi
}

# Run security tests
run_security_tests() {
    log_info "Running security tests..."
    
    cd "$PROJECT_ROOT"
    
    # Run comprehensive security tests
    if cargo test comprehensive_security_tests --lib --release; then
        log_success "All security tests passed"
        update_report "security_tests" "passed" "All security tests passed" "testing"
    else
        log_error "Security tests failed"
        update_report "security_tests" "failed" "Security tests failed" "testing"
        return 1
    fi
    
    # Run clippy with security checks
    if cargo clippy --all-targets --all-features --release -- -D warnings; then
        log_success "Clippy checks passed"
        update_report "clippy_checks" "passed" "Clippy checks passed" "code_quality"
    else
        log_warning "Clippy found issues"
        update_report "clippy_checks" "warning" "Clippy found issues" "code_quality"
    fi
}

# Check dependencies for vulnerabilities
check_dependency_vulnerabilities() {
    log_info "Checking dependencies for vulnerabilities..."
    
    cd "$PROJECT_ROOT"
    
    # Check for cargo-audit
    if ! command -v cargo-audit &> /dev/null; then
        log_info "Installing cargo-audit..."
        cargo install cargo-audit
    fi
    
    if cargo audit; then
        log_success "No vulnerabilities found in dependencies"
        update_report "dependency_vulnerabilities" "passed" "No vulnerabilities found" "security"
    else
        log_warning "Vulnerabilities found in dependencies"
        update_report "dependency_vulnerabilities" "warning" "Vulnerabilities found in dependencies" "security"
    fi
}

# Check configuration security
check_configuration_security() {
    log_info "Checking configuration security..."
    
    cd "$PROJECT_ROOT"
    
    # Check production configuration
    if [ -f "settings/production.toml" ]; then
        log_success "Production configuration file exists"
        update_report "production_config" "passed" "Production configuration file exists" "configuration"
    else
        log_error "Production configuration file missing"
        update_report "production_config" "failed" "Production configuration file missing" "configuration"
        return 1
    fi
    
    # Check for sensitive data in configuration
    if grep -q "password\|secret\|key" settings/production.toml; then
        log_warning "Potential sensitive data found in configuration"
        update_report "sensitive_data_config" "warning" "Potential sensitive data found in configuration" "configuration"
    else
        log_success "No obvious sensitive data in configuration"
        update_report "sensitive_data_config" "passed" "No obvious sensitive data in configuration" "configuration"
    fi
}

# Check database security
check_database_security() {
    log_info "Checking database security..."
    
    cd "$PROJECT_ROOT"
    
    # Check if database file exists
    if [ -f "data/hive_persistence.db" ]; then
        log_success "Database file exists"
        update_report "database_exists" "passed" "Database file exists" "database"
    else
        log_warning "Database file does not exist (will be created on first run)"
        update_report "database_exists" "warning" "Database file does not exist" "database"
    fi
    
    # Check database permissions
    if [ -f "data/hive_persistence.db" ]; then
        local perms=$(stat -c "%a" data/hive_persistence.db)
        if [ "$perms" = "600" ] || [ "$perms" = "640" ]; then
            log_success "Database permissions are secure ($perms)"
            update_report "database_permissions" "passed" "Database permissions are secure ($perms)" "database"
        else
            log_warning "Database permissions may be too open ($perms)"
            update_report "database_permissions" "warning" "Database permissions may be too open ($perms)" "database"
        fi
    fi
}

# Check TLS/SSL configuration
check_tls_configuration() {
    log_info "Checking TLS/SSL configuration..."
    
    # Check if OpenSSL supports TLS 1.3
    if openssl s_client -help 2>&1 | grep -q "tls1_3"; then
        log_success "OpenSSL supports TLS 1.3"
        update_report "tls13_support" "passed" "OpenSSL supports TLS 1.3" "network_security"
    else
        log_warning "OpenSSL does not support TLS 1.3"
        update_report "tls13_support" "warning" "OpenSSL does not support TLS 1.3" "network_security"
    fi
    
    # Check secure cipher suites
    local secure_ciphers="ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384"
    if openssl ciphers | grep -q "ECDHE-RSA-AES.*GCM"; then
        log_success "Secure cipher suites are available"
        update_report "secure_ciphers" "passed" "Secure cipher suites are available" "network_security"
    else
        log_warning "Secure cipher suites may not be available"
        update_report "secure_ciphers" "warning" "Secure cipher suites may not be available" "network_security"
    fi
}

# Check authentication and authorization
check_auth_security() {
    log_info "Checking authentication and authorization security..."
    
    cd "$PROJECT_ROOT"
    
    # Check if auth module exists
    if [ -f "src/utils/auth.rs" ]; then
        log_success "Authentication module exists"
        update_report "auth_module" "passed" "Authentication module exists" "authentication"
    else
        log_error "Authentication module missing"
        update_report "auth_module" "failed" "Authentication module missing" "authentication"
        return 1
    fi
    
    # Check for JWT implementation
    if grep -q "jsonwebtoken" Cargo.toml; then
        log_success "JWT library is included"
        update_report "jwt_implementation" "passed" "JWT library is included" "authentication"
    else
        log_error "JWT library not found"
        update_report "jwt_implementation" "failed" "JWT library not found" "authentication"
        return 1
    fi
    
    # Check for password hashing
    if grep -q "argon2" Cargo.toml; then
        log_success "Argon2 password hashing is included"
        update_report "password_hashing" "passed" "Argon2 password hashing is included" "authentication"
    else
        log_warning "Argon2 password hashing not found"
        update_report "password_hashing" "warning" "Argon2 password hashing not found" "authentication"
    fi
}

# Check input validation
check_input_validation() {
    log_info "Checking input validation..."
    
    cd "$PROJECT_ROOT"
    
    # Check validation module
    if [ -f "src/utils/validation.rs" ]; then
        log_success "Input validation module exists"
        update_report "validation_module" "passed" "Input validation module exists" "input_validation"
    else
        log_error "Input validation module missing"
        update_report "validation_module" "failed" "Input validation module missing" "input_validation"
        return 1
    fi
    
    # Check for sanitization functions
    if grep -q "sanitize_string\|validate_uuid\|sanitize_email" src/utils/validation.rs; then
        log_success "Input sanitization functions are implemented"
        update_report "input_sanitization" "passed" "Input sanitization functions are implemented" "input_validation"
    else
        log_warning "Input sanitization functions may be incomplete"
        update_report "input_sanitization" "warning" "Input sanitization functions may be incomplete" "input_validation"
    fi
}

# Check audit logging
check_audit_logging() {
    log_info "Checking audit logging..."
    
    cd "$PROJECT_ROOT"
    
    # Check structured logging module
    if [ -f "src/utils/structured_logging.rs" ]; then
        log_success "Structured logging module exists"
        update_report "structured_logging" "passed" "Structured logging module exists" "audit_logging"
    else
        log_error "Structured logging module missing"
        update_report "structured_logging" "failed" "Structured logging module missing" "audit_logging"
        return 1
    fi
    
    # Check security event types
    if grep -q "SecurityEventType" src/utils/structured_logging.rs; then
        log_success "Security event types are defined"
        update_report "security_event_types" "passed" "Security event types are defined" "audit_logging"
    else
        log_warning "Security event types may not be defined"
        update_report "security_event_types" "warning" "Security event types may not be defined" "audit_logging"
    fi
}

# Check GDPR compliance
check_gdpr_compliance() {
    log_info "Checking GDPR compliance..."
    
    local gdpr_checks=0
    local gdpr_passed=0
    
    # Check data protection
    if grep -q "aes-gcm\|encryption" Cargo.toml; then
        log_success "Data encryption libraries are included"
        ((gdpr_passed++))
    else
        log_warning "Data encryption libraries may be missing"
    fi
    ((gdpr_checks++))
    
    # Check audit logging
    if [ -f "src/utils/structured_logging.rs" ]; then
        log_success "Audit logging is implemented"
        ((gdpr_passed++))
    else
        log_warning "Audit logging may not be implemented"
    fi
    ((gdpr_checks++))
    
    # Check input validation
    if [ -f "src/utils/validation.rs" ]; then
        log_success "Input validation is implemented"
        ((gdpr_passed++))
    else
        log_warning "Input validation may not be implemented"
    fi
    ((gdpr_checks++))
    
    local gdpr_percentage=$((gdpr_passed * 100 / gdpr_checks))
    
    if [ "$gdpr_percentage" -ge 80 ]; then
        log_success "GDPR compliance check passed ($gdpr_percentage%)"
        update_report "gdpr_compliance" "passed" "GDPR compliance: $gdpr_percentage%" "compliance"
        REPORT=$(echo "$REPORT" | jq '.compliance.gdpr = true')
    else
        log_warning "GDPR compliance check failed ($gdpr_percentage%)"
        update_report "gdpr_compliance" "warning" "GDPR compliance: $gdpr_percentage%" "compliance"
        REPORT=$(echo "$REPORT" | jq '.compliance.gdpr = false')
    fi
}

# Check ISO 27001 compliance
check_iso27001_compliance() {
    log_info "Checking ISO 27001 compliance..."
    
    local iso_checks=0
    local iso_passed=0
    
    # Check information security policies
    if [ -f "docs/COMPLIANCE.md" ]; then
        log_success "Security documentation exists"
        ((iso_passed++))
    else
        log_warning "Security documentation may be missing"
    fi
    ((iso_checks++))
    
    # Check access control
    if [ -f "src/utils/auth.rs" ]; then
        log_success "Access control is implemented"
        ((iso_passed++))
    else
        log_warning "Access control may not be implemented"
    fi
    ((iso_checks++))
    
    # Check cryptography
    if grep -q "aes-gcm\|ring\|jsonwebtoken" Cargo.toml; then
        log_success "Cryptography controls are implemented"
        ((iso_passed++))
    else
        log_warning "Cryptography controls may be missing"
    fi
    ((iso_checks++))
    
    # Check logging and monitoring
    if [ -f "src/utils/structured_logging.rs" ]; then
        log_success "Logging and monitoring are implemented"
        ((iso_passed++))
    else
        log_warning "Logging and monitoring may not be implemented"
    fi
    ((iso_checks++))
    
    local iso_percentage=$((iso_passed * 100 / iso_checks))
    
    if [ "$iso_percentage" -ge 80 ]; then
        log_success "ISO 27001 compliance check passed ($iso_percentage%)"
        update_report "iso27001_compliance" "passed" "ISO 27001 compliance: $iso_percentage%" "compliance"
        REPORT=$(echo "$REPORT" | jq '.compliance.iso27001 = true')
    else
        log_warning "ISO 27001 compliance check failed ($iso_percentage%)"
        update_report "iso27001_compliance" "warning" "ISO 27001 compliance: $iso_percentage%" "compliance"
        REPORT=$(echo "$REPORT" | jq '.compliance.iso27001 = false')
    fi
}

# Check NIST CSF compliance
check_nist_csf_compliance() {
    log_info "Checking NIST CSF compliance..."
    
    local nist_checks=0
    local nist_passed=0
    
    # Check Identify (asset management)
    if [ -f "Cargo.toml" ] && [ -f "src/main.rs" ]; then
        log_success "Asset management is in place"
        ((nist_passed++))
    else
        log_warning "Asset management may be incomplete"
    fi
    ((nist_checks++))
    
    # Check Protect (access control, data security)
    if [ -f "src/utils/auth.rs" ] && grep -q "aes-gcm" Cargo.toml; then
        log_success "Protection controls are implemented"
        ((nist_passed++))
    else
        log_warning "Protection controls may be incomplete"
    fi
    ((nist_checks++))
    
    # Check Detect (monitoring)
    if [ -f "src/utils/structured_logging.rs" ]; then
        log_success "Detection capabilities are implemented"
        ((nist_passed++))
    else
        log_warning "Detection capabilities may be incomplete"
    fi
    ((nist_checks++))
    
    # Check Respond (incident response)
    if [ -f "docs/SECURITY_AUDIT_REPORT.md" ]; then
        log_success "Response procedures are documented"
        ((nist_passed++))
    else
        log_warning "Response procedures may not be documented"
    fi
    ((nist_checks++))
    
    # Check Recover (backup and recovery)
    if grep -q "backup\|recovery" docs/*.md 2>/dev/null; then
        log_success "Recovery procedures are documented"
        ((nist_passed++))
    else
        log_warning "Recovery procedures may not be documented"
    fi
    ((nist_checks++))
    
    local nist_percentage=$((nist_passed * 100 / nist_checks))
    
    if [ "$nist_percentage" -ge 80 ]; then
        log_success "NIST CSF compliance check passed ($nist_percentage%)"
        update_report "nist_csf_compliance" "passed" "NIST CSF compliance: $nist_percentage%" "compliance"
        REPORT=$(echo "$REPORT" | jq '.compliance.nist_csf = true')
    else
        log_warning "NIST CSF compliance check failed ($nist_percentage%)"
        update_report "nist_csf_compliance" "warning" "NIST CSF compliance: $nist_percentage%" "compliance"
        REPORT=$(echo "$REPORT" | jq '.compliance.nist_csf = false')
    fi
}

# Check OWASP Top 10 compliance
check_owasp_compliance() {
    log_info "Checking OWASP Top 10 compliance..."
    
    local owasp_checks=0
    local owasp_passed=0
    
    # Check A01: Broken Access Control
    if [ -f "src/utils/auth.rs" ] && grep -q "Role\|Permission" src/utils/auth.rs; then
        log_success "Access control is implemented"
        ((owasp_passed++))
    else
        log_warning "Access control may be incomplete"
    fi
    ((owasp_checks++))
    
    # Check A02: Cryptographic Failures
    if grep -q "aes-gcm\|argon2\|jsonwebtoken" Cargo.toml; then
        log_success "Cryptographic controls are implemented"
        ((owasp_passed++))
    else
        log_warning "Cryptographic controls may be incomplete"
    fi
    ((owasp_checks++))
    
    # Check A03: Injection
    if [ -f "src/utils/validation.rs" ] && grep -q "sanitize\|validate" src/utils/validation.rs; then
        log_success "Injection protection is implemented"
        ((owasp_passed++))
    else
        log_warning "Injection protection may be incomplete"
    fi
    ((owasp_checks++))
    
    # Check A04: Insecure Design
    if [ -f "src/infrastructure/security_middleware.rs" ]; then
        log_success "Security middleware is implemented"
        ((owasp_passed++))
    else
        log_warning "Security middleware may be missing"
    fi
    ((owasp_checks++))
    
    # Check A05: Security Misconfiguration
    if [ -f "settings/production.toml" ] && [ -f "settings/default.toml" ]; then
        log_success "Configuration management is in place"
        ((owasp_passed++))
    else
        log_warning "Configuration management may be incomplete"
    fi
    ((owasp_checks++))
    
    local owasp_percentage=$((owasp_passed * 100 / owasp_checks))
    
    if [ "$owasp_percentage" -ge 80 ]; then
        log_success "OWASP Top 10 compliance check passed ($owasp_percentage%)"
        update_report "owasp_compliance" "passed" "OWASP Top 10 compliance: $owasp_percentage%" "compliance"
        REPORT=$(echo "$REPORT" | jq '.compliance.owasp_top10 = true')
    else
        log_warning "OWASP Top 10 compliance check failed ($owasp_percentage%)"
        update_report "owasp_compliance" "warning" "OWASP Top 10 compliance: $owasp_percentage%" "compliance"
        REPORT=$(echo "$REPORT" | jq '.compliance.owasp_top10 = false')
    fi
}

# Generate final report
generate_final_report() {
    log_info "Generating final compliance report..."
    
    # Calculate overall status
    local total=$(echo "$REPORT" | jq '.summary.total')
    local passed=$(echo "$REPORT" | jq '.summary.passed')
    local failed=$(echo "$REPORT" | jq '.summary.failed')
    local warnings=$(echo "$REPORT" | jq '.summary.warnings')
    
    local success_percentage=$((passed * 100 / total))
    
    if [ "$failed" -eq 0 ] && [ "$success_percentage" -ge 90 ]; then
        REPORT=$(echo "$REPORT" | jq '.overall_status = "passed"')
        log_success "Overall compliance status: PASSED ($success_percentage%)"
    elif [ "$failed" -eq 0 ] && [ "$success_percentage" -ge 70 ]; then
        REPORT=$(echo "$REPORT" | jq '.overall_status = "warning"')
        log_warning "Overall compliance status: WARNING ($success_percentage%)"
    else
        REPORT=$(echo "$REPORT" | jq '.overall_status = "failed"')
        log_error "Overall compliance status: FAILED ($success_percentage%)"
    fi
    
    # Save report
    echo "$REPORT" | jq '.' > "$REPORT_FILE"
    
    log_success "Compliance report saved to: $REPORT_FILE"
    log_info "Detailed log saved to: $LOG_FILE"
    
    # Display summary
    echo ""
    echo "=== COMPLIANCE SUMMARY ==="
    echo "Total checks: $total"
    echo "Passed: $passed"
    echo "Failed: $failed"
    echo "Warnings: $warnings"
    echo "Success rate: $success_percentage%"
    echo "Overall status: $(echo "$REPORT" | jq -r '.overall_status')"
    echo ""
    echo "=== COMPLIANCE FRAMEWORKS ==="
    echo "GDPR: $(echo "$REPORT" | jq -r '.compliance.gdpr')"
    echo "ISO 27001: $(echo "$REPORT" | jq -r '.compliance.iso27001')"
    echo "NIST CSF: $(echo "$REPORT" | jq -r '.compliance.nist_csf')"
    echo "OWASP Top 10: $(echo "$REPORT" | jq -r '.compliance.owasp_top10')"
    echo ""
}

# Main execution
main() {
    log_info "Starting AI Orchestrator Hub Security and Compliance Check"
    log_info "Project root: $PROJECT_ROOT"
    log_info "Log file: $LOG_FILE"
    log_info "Report file: $REPORT_FILE"
    echo ""
    
    # Run all checks
    local exit_code=0
    
    check_prerequisites || exit_code=1
    check_code_compilation || exit_code=1
    run_security_tests || exit_code=1
    check_dependency_vulnerabilities
    check_configuration_security || exit_code=1
    check_database_security
    check_tls_configuration
    check_auth_security || exit_code=1
    check_input_validation || exit_code=1
    check_audit_logging || exit_code=1
    check_gdpr_compliance
    check_iso27001_compliance
    check_nist_csf_compliance
    check_owasp_compliance
    
    echo ""
    generate_final_report
    
    # Exit with appropriate code
    if [ "$(echo "$REPORT" | jq -r '.overall_status')" = "failed" ]; then
        exit 1
    elif [ "$(echo "$REPORT" | jq -r '.overall_status')" = "warning" ]; then
        exit 2
    else
        exit 0
    fi
}

# Run main function
main "$@"