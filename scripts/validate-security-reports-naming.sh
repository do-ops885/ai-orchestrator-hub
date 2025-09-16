#!/bin/bash
# Security Reports Naming Convention Validation Script
# This script validates that security reports follow proper naming conventions

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SECURITY_REPORTS_DIR="$PROJECT_ROOT/security-reports"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

INVALID_FILES=()
EXIT_CODE=0

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" >&2
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" >&2
}

# Validate filename against naming convention
validate_filename() {
    local filename="$1"
    local expected_pattern="^[a-zA-Z0-9_-]+-[0-9]{8}-[0-9]{6}\.(json|txt|sarif)$"

    if [[ "$filename" =~ $expected_pattern ]]; then
        return 0
    else
        return 1
    fi
}

# Suggest proper filename
suggest_filename() {
    local filename="$1"
    local timestamp
    timestamp=$(date +%Y%m%d-%H%M%S)

    # Try to determine the report type from the filename
    if [[ "$filename" =~ cargo ]]; then
        echo "cargo-audit-${timestamp}.json"
    elif [[ "$filename" =~ npm ]]; then
        echo "npm-audit-${timestamp}.json"
    elif [[ "$filename" =~ secrets ]]; then
        echo "secrets-scan-${timestamp}.txt"
    elif [[ "$filename" =~ codeql ]]; then
        echo "codeql-${timestamp}.sarif"
    elif [[ "$filename" =~ (container|trivy) ]]; then
        echo "container-scan-${timestamp}.sarif"
    elif [[ "$filename" =~ dependency ]]; then
        echo "dependency-review-${timestamp}.json"
    elif [[ "$filename" =~ security ]]; then
        echo "security-metrics-${timestamp}.json"
    else
        echo "security-report-${timestamp}.json"
    fi
}

# Main validation function
validate_naming_conventions() {
    log_success "Validating security reports naming conventions..."

    if [ ! -d "$SECURITY_REPORTS_DIR" ]; then
        log_warn "security-reports directory does not exist, skipping naming validation"
        return 0
    fi

    # Find all files in security-reports directory
    while IFS= read -r -d '' file; do
        filename=$(basename "$file")

        # Skip directories and hidden files
        if [ -d "$file" ] || [[ "$filename" =~ ^\. ]]; then
            continue
        fi

        # Check if filename follows convention
        if ! validate_filename "$filename"; then
            INVALID_FILES+=("$file")
        fi
    done < <(find "$SECURITY_REPORTS_DIR" -type f -print0)

    # Report findings
    if [ ${#INVALID_FILES[@]} -gt 0 ]; then
        log_error "Found ${#INVALID_FILES[@]} files with invalid naming:"
        echo
        echo "The following files do not follow the naming convention:"
        echo "Expected format: {type}-YYYYMMDD-HHMMSS.{extension}"
        echo

        for file in "${INVALID_FILES[@]}"; do
            filename=$(basename "$file")
            suggestion=$(suggest_filename "$filename")
            echo "  ❌ $filename"
            echo "      Suggested: $suggestion"
            echo
        done

        echo "🔧 To fix this issue:"
        echo "   1. Rename the files to follow the naming convention"
        echo "   2. Use the suggested names above as a guide"
        echo "   3. Update any scripts that reference the old filenames"
        echo
        echo "📖 See CONTRIBUTING.md#security-report-organization for details"

        EXIT_CODE=1
    else
        log_success "All security reports follow naming conventions"
    fi
}

# Check for duplicate report types (same type, different timestamps)
check_for_duplicates() {
    log_success "Checking for duplicate security report types..."

    if [ ! -d "$SECURITY_REPORTS_DIR" ]; then
        return 0
    fi

    # Group files by type and check for reasonable number
    declare -A report_counts

    while IFS= read -r -d '' file; do
        filename=$(basename "$file")

        # Extract report type (everything before first dash)
        if [[ "$filename" =~ ^([a-zA-Z0-9_-]+)- ]]; then
            report_type="${BASH_REMATCH[1]}"
            ((report_counts["$report_type"]++))
        fi
    done < <(find "$SECURITY_REPORTS_DIR" -type f -print0)

    # Check for excessive duplicates
    for report_type in "${!report_counts[@]}"; do
        count=${report_counts[$report_type]}
        if [ $count -gt 10 ]; then
            log_warn "Found $count $report_type reports (consider cleanup)"
            log_warn "Recommendation: Keep only recent reports and archive older ones"
        fi
    done
}

# Main execution
main() {
    echo "🏷️  Checking security reports naming convention compliance..."
    echo

    validate_naming_conventions
    check_for_duplicates

    echo
    if [ $EXIT_CODE -eq 0 ]; then
        log_success "Security reports naming validation passed"
    else
        log_error "Security reports naming validation failed"
        echo
        echo "This check ensures all security reports follow consistent naming"
        echo "conventions to improve organization and automation."
    fi

    exit $EXIT_CODE
}

# Run main function
main "$@"