#!/bin/bash
# Security Report Generation Template
# This template ensures all security reports are generated in the correct location
#
# Usage: ./security-report-template.sh [report-type] [additional-args...]
#
# This script serves as a template for creating security report generation scripts.
# It ensures proper directory structure, naming conventions, and error handling.

set -euo pipefail

# Configuration
SCRIPT_NAME="$(basename "$0")"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SECURITY_REPORTS_DIR="$PROJECT_ROOT/security-reports"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" >&2
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" >&2
}

# Function to generate timestamp
generate_timestamp() {
    date +%Y%m%d-%H%M%S
}

# Function to ensure security-reports directory exists
ensure_security_reports_dir() {
    if [ ! -d "$SECURITY_REPORTS_DIR" ]; then
        log_info "Creating security-reports directory: $SECURITY_REPORTS_DIR"
        mkdir -p "$SECURITY_REPORTS_DIR"
        # Set appropriate permissions (readable by owner and group, no world access)
        chmod 755 "$SECURITY_REPORTS_DIR"
    fi
}

# Function to validate report filename
validate_filename() {
    local filename="$1"
    local expected_pattern="^[a-zA-Z0-9_-]+-[0-9]{8}-[0-9]{6}\.(json|txt|sarif)$"

    if [[ ! "$filename" =~ $expected_pattern ]]; then
        log_error "Invalid filename format: $filename"
        log_error "Expected format: {type}-YYYYMMDD-HHMMSS.{extension}"
        return 1
    fi
    return 0
}

# Function to generate report filename
generate_report_filename() {
    local report_type="$1"
    local extension="${2:-json}"
    local timestamp
    timestamp=$(generate_timestamp)

    echo "${report_type}-${timestamp}.${extension}"
}

# Function to set appropriate file permissions
set_report_permissions() {
    local file_path="$1"

    # Set permissions to be readable/writable by owner, readable by group
    # This prevents accidental exposure of sensitive security information
    chmod 644 "$file_path"
}

# Function to validate security-reports directory
validate_security_reports_dir() {
    # Check if directory exists
    if [ ! -d "$SECURITY_REPORTS_DIR" ]; then
        log_error "Security reports directory does not exist: $SECURITY_REPORTS_DIR"
        return 1
    fi

    # Check if directory is writable
    if [ ! -w "$SECURITY_REPORTS_DIR" ]; then
        log_error "Security reports directory is not writable: $SECURITY_REPORTS_DIR"
        return 1
    fi

    # Check if we're in a git repository (optional but recommended)
    if [ -d "$PROJECT_ROOT/.git" ]; then
        # Check if security-reports directory is in .gitignore
        if ! grep -q "^security-reports/" "$PROJECT_ROOT/.gitignore" 2>/dev/null; then
            log_warn "Consider adding 'security-reports/' to .gitignore to prevent accidental commits"
        fi
    fi

    return 0
}

# Function to clean up old reports (optional)
cleanup_old_reports() {
    local report_type="$1"
    local max_age_days="${2:-30}"
    local count=0

    log_info "Cleaning up $report_type reports older than $max_age_days days"

    # Find and remove old reports
    while IFS= read -r -d '' file; do
        if [ -f "$file" ]; then
            log_info "Removing old report: $(basename "$file")"
            rm -f "$file"
            ((count++))
        fi
    done < <(find "$SECURITY_REPORTS_DIR" -name "${report_type}-*.json" -mtime +"$max_age_days" -print0 2>/dev/null)

    if [ $count -gt 0 ]; then
        log_success "Cleaned up $count old $report_type reports"
    else
        log_info "No old $report_type reports to clean up"
    fi
}

# Function to generate report summary
generate_report_summary() {
    local report_file="$1"
    local report_type="$2"

    if [ -f "$report_file" ]; then
        local file_size
        file_size=$(du -h "$report_file" | cut -f1)
        local line_count
        line_count=$(wc -l < "$report_file")

        log_success "Generated $report_type report:"
        log_success "  Location: $report_file"
        log_success "  Size: $file_size"
        log_success "  Lines: $line_count"

        # Print summary to stdout for potential use by other tools
        cat <<EOF
{
  "report_type": "$report_type",
  "file_path": "$report_file",
  "file_size": "$file_size",
  "line_count": $line_count,
  "generated_at": "$(date -Iseconds)",
  "generator": "$SCRIPT_NAME"
}
EOF
    else
        log_error "Report file was not created: $report_file"
        return 1
    fi
}

# Main function - override this in your specific script
main() {
    local report_type="${1:-unknown}"
    shift

    log_info "Starting security report generation: $report_type"

    # Validate environment
    ensure_security_reports_dir
    validate_security_reports_dir || exit 1

    # Generate report filename
    local report_filename
    report_filename=$(generate_report_filename "$report_type")
    local report_path="$SECURITY_REPORTS_DIR/$report_filename"

    # Validate filename format
    validate_filename "$report_filename" || exit 1

    log_info "Generating report: $report_path"

    # Call the actual report generation function (must be implemented by child scripts)
    if ! generate_security_report "$report_path" "$@"; then
        log_error "Failed to generate security report"
        exit 1
    fi

    # Set appropriate permissions
    set_report_permissions "$report_path"

    # Generate summary
    generate_report_summary "$report_path" "$report_type"

    # Optional cleanup
    if [ "${CLEANUP_OLD_REPORTS:-false}" = "true" ]; then
        cleanup_old_reports "$report_type"
    fi

    log_success "Security report generation completed successfully"
}

# Function that must be implemented by scripts using this template
generate_security_report() {
    local output_file="$1"
    shift

    log_error "generate_security_report() function must be implemented by the specific script"
    log_error "This is just a template - create your own implementation"
    return 1
}

# Show usage if requested
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    cat <<EOF
Security Report Generation Template

This template provides a standardized way to generate security reports
with proper directory structure, naming conventions, and error handling.

USAGE:
    $SCRIPT_NAME [report-type] [additional-args...]

ARGUMENTS:
    report-type    Type of security report (e.g., cargo-audit, npm-audit, secrets-scan)

OPTIONS:
    --help, -h     Show this help message

ENVIRONMENT VARIABLES:
    CLEANUP_OLD_REPORTS    Set to 'true' to automatically clean up old reports (default: false)

EXAMPLES:
    # Generate a cargo audit report
    $SCRIPT_NAME cargo-audit

    # Generate with cleanup enabled
    CLEANUP_OLD_REPORTS=true $SCRIPT_NAME npm-audit

OUTPUT:
    Reports are generated in: $SECURITY_REPORTS_DIR
    Format: {type}-YYYYMMDD-HHMMSS.{extension}

This is a template script. To use it:
1. Copy this file to create your specific security report script
2. Implement the generate_security_report() function
3. Customize the report_type and any additional logic as needed
EOF
    exit 0
fi

# Run main function if script is executed directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi