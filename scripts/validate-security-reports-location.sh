#!/bin/bash
# Security Reports Location Validation Script
# This script checks for misplaced security files and ensures they are in the correct location

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SECURITY_REPORTS_DIR="$PROJECT_ROOT/security-reports"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

MISPLACED_FILES=()
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

# Check if a file contains security-related content
is_security_file() {
    local file="$1"

    # Check filename patterns
    local filename
    filename=$(basename "$file")

    if [[ "$filename" =~ (audit|security|secrets|scan|vulnerability|codeql|trivy|dependency|gitleaks) ]]; then
        return 0
    fi

    # Check file content for security-related keywords
    if [ -f "$file" ] && file "$file" | grep -q "text"; then
        if grep -q -i "vulnerability\|security\|audit\|secrets\|scan\|codeql\|trivy\|dependency" "$file" 2>/dev/null; then
            return 0
        fi
    fi

    return 1
}

# Main validation function
validate_security_files_location() {
    log_success "Validating security reports are in correct location..."

    # Define directories to check for misplaced security files
    DIRECTORIES_TO_CHECK=(
        "backend"
        "frontend"
        "docs"
        "scripts"
        ".github"
        "tests"
        "monitoring"
        "benchmarks"
        "training"
        "data"
        "helm"
        "k8s"
        "examples"
        "src"
    )

    # Check for security files in wrong locations
    for dir in "${DIRECTORIES_TO_CHECK[@]}"; do
        if [ -d "$PROJECT_ROOT/$dir" ]; then
            # Find potential security files
            while IFS= read -r -d '' file; do
                # Skip if it's already in security-reports
                if [[ "$file" == "$SECURITY_REPORTS_DIR"* ]]; then
                    continue
                fi

                # Check if it's a security-related file
                if is_security_file "$file"; then
                    MISPLACED_FILES+=("$file")
                fi
            done < <(find "$PROJECT_ROOT/$dir" -type f \( -name "*audit*" -o -name "*security*" -o -name "*secrets*" -o -name "*scan*" -o -name "*vulnerability*" -o -name "*codeql*" -o -name "*trivy*" -o -name "*dependency*" \) -print0 2>/dev/null)
        fi
    done

    # Check root directory for security files not in security-reports
    if [ -d "$PROJECT_ROOT" ]; then
        while IFS= read -r -d '' file; do
            # Skip if it's in security-reports or other expected locations
            if [[ "$file" == "$SECURITY_REPORTS_DIR"* ]] || [[ "$file" == "./.git"* ]] || [[ "$file" == "./.github"* ]]; then
                continue
            fi

            if is_security_file "$file"; then
                MISPLACED_FILES+=("$file")
            fi
        done < <(find "$PROJECT_ROOT" -maxdepth 1 -type f \( -name "*audit*" -o -name "*security*" -o -name "*secrets*" -o -name "*scan*" -o -name "*vulnerability*" -o -name "*codeql*" -o -name "*trivy*" -o -name "*dependency*" \) -print0 2>/dev/null)
    fi

    # Report findings
    if [ ${#MISPLACED_FILES[@]} -gt 0 ]; then
        log_error "Found ${#MISPLACED_FILES[@]} misplaced security files:"
        echo
        echo "The following security-related files are not in the security-reports/ directory:"
        echo

        for file in "${MISPLACED_FILES[@]}"; do
            echo "  ❌ $file"
        done

        echo
        echo "🔧 To fix this issue:"
        echo "   1. Move the files to the security-reports/ directory"
        echo "   2. Rename them to follow the naming convention: {type}-YYYYMMDD-HHMMSS.{extension}"
        echo "   3. Update any scripts that reference the old locations"
        echo
        echo "📖 See CONTRIBUTING.md#security-report-organization for details"

        EXIT_CODE=1
    else
        log_success "No misplaced security files found"
    fi
}

# Check if security-reports directory exists
check_security_reports_directory() {
    if [ ! -d "$SECURITY_REPORTS_DIR" ]; then
        log_warn "security-reports directory does not exist"
        log_warn "Creating security-reports directory..."
        mkdir -p "$SECURITY_REPORTS_DIR"
        log_success "Created security-reports directory"
    else
        log_success "security-reports directory exists"
    fi
}

# Main execution
main() {
    echo "🔍 Checking security reports location compliance..."
    echo

    check_security_reports_directory
    validate_security_files_location

    echo
    if [ $EXIT_CODE -eq 0 ]; then
        log_success "Security reports location validation passed"
    else
        log_error "Security reports location validation failed"
        echo
        echo "This check ensures all security-related files are properly organized"
        echo "in the security-reports/ directory to maintain consistency and prevent"
        echo "accidental exposure of sensitive security information."
    fi

    exit $EXIT_CODE
}

# Run main function
main "$@"