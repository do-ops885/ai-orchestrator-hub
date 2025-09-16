#!/bin/bash
# Comprehensive Security Reports Compliance Validation Script
# This script validates all aspects of security report organization and compliance

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SECURITY_REPORTS_DIR="$PROJECT_ROOT/security-reports"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Global variables
ISSUES_FOUND=0
WARNINGS_FOUND=0
TOTAL_CHECKS=0
PASSED_CHECKS=0

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    ((WARNINGS_FOUND++))
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    ((ISSUES_FOUND++))
}

print_header() {
    echo
    echo "================================================================="
    echo "🔒 SECURITY REPORTS COMPLIANCE VALIDATION"
    echo "================================================================="
    echo
}

print_summary() {
    echo
    echo "================================================================="
    echo "📊 VALIDATION SUMMARY"
    echo "================================================================="
    echo
    echo "Checks Performed: $TOTAL_CHECKS"
    echo "Passed: $PASSED_CHECKS"
    echo "Warnings: $WARNINGS_FOUND"
    echo "Errors: $ISSUES_FOUND"
    echo

    if [ $ISSUES_FOUND -gt 0 ]; then
        echo -e "${RED}❌ VALIDATION FAILED${NC} - $ISSUES_FOUND issues found"
        echo
        echo "🔧 To fix these issues:"
        echo "   1. Review the error messages above"
        echo "   2. Move misplaced security files to security-reports/"
        echo "   3. Rename files to follow naming conventions"
        echo "   4. Update scripts to use correct output directories"
        echo "   5. See CONTRIBUTING.md#security-report-organization for details"
        echo
        return 1
    elif [ $WARNINGS_FOUND -gt 0 ]; then
        echo -e "${YELLOW}⚠️  VALIDATION PASSED WITH WARNINGS${NC} - $WARNINGS_FOUND warnings"
        echo
        echo "Consider addressing the warnings for better compliance."
        return 0
    else
        echo -e "${GREEN}✅ VALIDATION PASSED${NC} - All checks successful"
        return 0
    fi
}

# Check 1: Validate security-reports directory exists
check_security_reports_directory() {
    ((TOTAL_CHECKS++))
    log_info "Checking security-reports directory..."

    if [ ! -d "$SECURITY_REPORTS_DIR" ]; then
        log_error "security-reports directory does not exist"
        log_error "Expected location: $SECURITY_REPORTS_DIR"
        return 1
    fi

    log_success "security-reports directory exists"
    ((PASSED_CHECKS++))
}

# Check 2: Validate directory permissions
check_directory_permissions() {
    ((TOTAL_CHECKS++))
    log_info "Checking security-reports directory permissions..."

    if [ ! -d "$SECURITY_REPORTS_DIR" ]; then
        log_error "Cannot check permissions - directory does not exist"
        return 1
    fi

    permissions=$(stat -c "%a" "$SECURITY_REPORTS_DIR")
    if [ "$permissions" -gt "755" ]; then
        log_error "security-reports directory has overly permissive permissions: $permissions"
        log_error "Recommended: 755 or less"
        return 1
    fi

    log_success "Directory permissions are appropriate: $permissions"
    ((PASSED_CHECKS++))
}

# Check 3: Find misplaced security files
check_misplaced_security_files() {
    ((TOTAL_CHECKS++))
    log_info "Checking for misplaced security files..."

    local misplaced_files=()

    # Define directories to check
    local dirs_to_check=(
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

    # Security file patterns
    local patterns=(
        "*audit*.json"
        "*security*.json"
        "*secrets*.txt"
        "*scan*.txt"
        "*vulnerability*.json"
        "*codeql*.sarif"
        "*trivy*.sarif"
        "*dependency*.json"
        "*gitleaks*.json"
    )

    for dir in "${dirs_to_check[@]}"; do
        if [ -d "$PROJECT_ROOT/$dir" ]; then
            for pattern in "${patterns[@]}"; do
                while IFS= read -r -d '' file; do
                    # Skip if it's in security-reports
                    if [[ "$file" != "$SECURITY_REPORTS_DIR"* ]]; then
                        misplaced_files+=("$file")
                    fi
                done < <(find "$PROJECT_ROOT/$dir" -name "$pattern" -type f -print0 2>/dev/null)
            done
        fi
    done

    # Check root directory
    for pattern in "${patterns[@]}"; do
        while IFS= read -r -d '' file; do
            if [[ "$file" != "$SECURITY_REPORTS_DIR"* ]] && [[ "$file" != "./.git"* ]]; then
                misplaced_files+=("$file")
            fi
        done < <(find "$PROJECT_ROOT" -maxdepth 1 -name "$pattern" -type f -print0 2>/dev/null)
    done

    if [ ${#misplaced_files[@]} -gt 0 ]; then
        log_error "Found ${#misplaced_files[@]} misplaced security files:"
        for file in "${misplaced_files[@]}"; do
            echo "  ❌ $file"
        done
        return 1
    fi

    log_success "No misplaced security files found"
    ((PASSED_CHECKS++))
}

# Check 4: Validate naming conventions
check_naming_conventions() {
    ((TOTAL_CHECKS++))
    log_info "Checking security reports naming conventions..."

    if [ ! -d "$SECURITY_REPORTS_DIR" ]; then
        log_warn "Skipping naming check - security-reports directory does not exist"
        return 0
    fi

    local invalid_files=()
    local expected_pattern="^[a-zA-Z0-9_-]+-[0-9]{8}-[0-9]{6}\.(json|txt|sarif)$"

    while IFS= read -r -d '' file; do
        filename=$(basename "$file")

        # Skip directories and hidden files
        if [ -d "$file" ] || [[ "$filename" =~ ^\. ]]; then
            continue
        fi

        if [[ ! "$filename" =~ $expected_pattern ]]; then
            invalid_files+=("$filename")
        fi
    done < <(find "$SECURITY_REPORTS_DIR" -type f -print0)

    if [ ${#invalid_files[@]} -gt 0 ]; then
        log_error "Found ${#invalid_files[@]} files with invalid naming:"
        for filename in "${invalid_files[@]}"; do
            echo "  ❌ $filename (expected: {type}-YYYYMMDD-HHMMSS.{extension})"
        done
        return 1
    fi

    log_success "All security reports follow naming conventions"
    ((PASSED_CHECKS++))
}

# Check 5: Validate file permissions
check_file_permissions() {
    ((TOTAL_CHECKS++))
    log_info "Checking security report file permissions..."

    if [ ! -d "$SECURITY_REPORTS_DIR" ]; then
        log_warn "Skipping file permissions check - directory does not exist"
        return 0
    fi

    local bad_permissions=()

    while IFS= read -r -d '' file; do
        permissions=$(stat -c "%a" "$file")
        # Check if file is world-readable or world-writable
        if [[ "$permissions" =~ [2367]$ ]]; then
            bad_permissions+=("$file:$permissions")
        fi
    done < <(find "$SECURITY_REPORTS_DIR" -type f -print0)

    if [ ${#bad_permissions[@]} -gt 0 ]; then
        log_error "Found ${#bad_permissions[@]} files with overly permissive permissions:"
        for item in "${bad_permissions[@]}"; do
            file=$(echo "$item" | cut -d: -f1)
            perms=$(echo "$item" | cut -d: -f2)
            echo "  ❌ $(basename "$file") (permissions: $perms, recommended: 644)"
        done
        return 1
    fi

    log_success "All security report files have appropriate permissions"
    ((PASSED_CHECKS++))
}

# Check 6: Check for excessive duplicates
check_duplicate_reports() {
    ((TOTAL_CHECKS++))
    log_info "Checking for excessive duplicate reports..."

    if [ ! -d "$SECURITY_REPORTS_DIR" ]; then
        log_warn "Skipping duplicate check - directory does not exist"
        return 0
    fi

    declare -A report_counts

    while IFS= read -r -d '' file; do
        filename=$(basename "$file")

        # Extract report type
        if [[ "$filename" =~ ^([a-zA-Z0-9_-]+)- ]]; then
            report_type="${BASH_REMATCH[1]}"
            ((report_counts["$report_type"]++))
        fi
    done < <(find "$SECURITY_REPORTS_DIR" -type f -print0)

    local excessive_duplicates=()

    for report_type in "${!report_counts[@]}"; do
        count=${report_counts[$report_type]}
        if [ $count -gt 20 ]; then
            excessive_duplicates+=("$report_type:$count")
        fi
    done

    if [ ${#excessive_duplicates[@]} -gt 0 ]; then
        log_warn "Found excessive duplicate reports:"
        for item in "${excessive_duplicates[@]}"; do
            report_type=$(echo "$item" | cut -d: -f1)
            count=$(echo "$item" | cut -d: -f2)
            echo "  ⚠️  $report_type: $count files (consider cleanup)"
        done
    else
        log_success "No excessive duplicate reports found"
    fi

    ((PASSED_CHECKS++))
}

# Check 7: Validate scripts reference security-reports
check_script_compliance() {
    ((TOTAL_CHECKS++))
    log_info "Checking scripts for security-reports compliance..."

    local non_compliant_scripts=()

    # Find scripts that might generate security reports
    while IFS= read -r -d '' script; do
        # Skip this validation script itself
        if [[ "$script" == *"/validate-security-reports-compliance.sh" ]]; then
            continue
        fi

        # Check if script contains security-related keywords but not security-reports
        if grep -q -i "audit\|security\|scan\|vulnerability\|secrets" "$script" 2>/dev/null; then
            if ! grep -q "security-reports" "$script" 2>/dev/null; then
                non_compliant_scripts+=("$script")
            fi
        fi
    done < <(find "$PROJECT_ROOT/scripts" -name "*.sh" -type f -print0 2>/dev/null)

    if [ ${#non_compliant_scripts[@]} -gt 0 ]; then
        log_warn "Found ${#non_compliant_scripts[@]} scripts that may need updates:"
        for script in "${non_compliant_scripts[@]}"; do
            echo "  ⚠️  $script (contains security keywords but no security-reports reference)"
        done
    else
        log_success "All security-related scripts reference security-reports directory"
    fi

    ((PASSED_CHECKS++))
}

# Check 8: Check .gitignore includes security-reports
check_gitignore() {
    ((TOTAL_CHECKS++))
    log_info "Checking .gitignore for security-reports..."

    local gitignore_file="$PROJECT_ROOT/.gitignore"

    if [ ! -f "$gitignore_file" ]; then
        log_warn ".gitignore file not found"
        return 0
    fi

    if ! grep -q "^security-reports" "$gitignore_file"; then
        log_warn "security-reports/ not found in .gitignore"
        log_warn "Consider adding it to prevent accidental commits of security reports"
    else
        log_success "security-reports properly excluded in .gitignore"
    fi

    ((PASSED_CHECKS++))
}

# Check 9: Generate compliance report
generate_compliance_report() {
    ((TOTAL_CHECKS++))
    log_info "Generating compliance report..."

    local report_file="$SECURITY_REPORTS_DIR/compliance-report-$(date +%Y%m%d-%H%M%S).json"

    # Ensure directory exists
    mkdir -p "$SECURITY_REPORTS_DIR"

    cat > "$report_file" << EOF
{
  "compliance_check": {
    "timestamp": "$(date -Iseconds)",
    "project_root": "$PROJECT_ROOT",
    "security_reports_dir": "$SECURITY_REPORTS_DIR",
    "checks_performed": $TOTAL_CHECKS,
    "passed_checks": $PASSED_CHECKS,
    "warnings": $WARNINGS_FOUND,
    "errors": $ISSUES_FOUND
  },
  "directory_info": {
    "exists": $([ -d "$SECURITY_REPORTS_DIR" ] && echo "true" || echo "false"),
    "permissions": "$( [ -d "$SECURITY_REPORTS_DIR" ] && stat -c "%a" "$SECURITY_REPORTS_DIR" || echo "N/A" )",
    "file_count": $(find "$SECURITY_REPORTS_DIR" -type f 2>/dev/null | wc -l)
  },
  "recommendations": [
    "Move all security-related files to security-reports/ directory",
    "Use timestamped filenames: {type}-YYYYMMDD-HHMMSS.{extension}",
    "Set file permissions to 644 (readable by owner and group only)",
    "Update scripts to output to security-reports/ directory",
    "Add security-reports/ to .gitignore if not already present",
    "Regularly clean up old security reports to manage disk space"
  ]
}
EOF

    log_success "Compliance report generated: $report_file"
    ((PASSED_CHECKS++))
}

# Main execution
main() {
    print_header

    # Run all checks
    check_security_reports_directory
    check_directory_permissions
    check_misplaced_security_files
    check_naming_conventions
    check_file_permissions
    check_duplicate_reports
    check_script_compliance
    check_gitignore
    generate_compliance_report

    # Print summary and exit with appropriate code
    print_summary
}

# Run main function
main "$@"