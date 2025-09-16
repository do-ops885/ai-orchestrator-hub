#!/bin/bash
# Security Reports Cleanup Script
# This script cleans up old security reports to manage disk space and maintain organization

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

# Default configuration
DEFAULT_RETENTION_DAYS=30
DEFAULT_DRY_RUN=true

# Global variables
DRY_RUN="$DEFAULT_DRY_RUN"
RETENTION_DAYS="$DEFAULT_RETENTION_DAYS"
TOTAL_FILES_FOUND=0
FILES_TO_DELETE=0
SPACE_TO_SAVE=0

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_usage() {
    cat << EOF
Security Reports Cleanup Script

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -d, --days DAYS       Number of days to retain reports (default: $DEFAULT_RETENTION_DAYS)
    -n, --dry-run         Show what would be deleted without actually deleting (default)
    -f, --force           Actually delete files (use with caution)
    -t, --type TYPE       Clean only specific report type (e.g., cargo-audit, npm-audit)
    -h, --help           Show this help message

EXAMPLES:
    # Dry run with default 30-day retention
    $0

    # Dry run with 7-day retention
    $0 --days 7

    # Actually delete old reports older than 30 days
    $0 --force

    # Clean only cargo-audit reports older than 14 days
    $0 --type cargo-audit --days 14 --force

DESCRIPTION:
    This script helps manage disk space by cleaning up old security reports.
    By default, it performs a dry run to show what would be deleted.

    Security reports are identified by their naming pattern:
    {type}-YYYYMMDD-HHMMSS.{extension}

    The script preserves the most recent reports of each type to ensure
    you always have recent security information available.

EOF
}

parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -d|--days)
                RETENTION_DAYS="$2"
                if ! [[ "$RETENTION_DAYS" =~ ^[0-9]+$ ]] || [ "$RETENTION_DAYS" -lt 1 ]; then
                    log_error "Retention days must be a positive integer"
                    exit 1
                fi
                shift 2
                ;;
            -n|--dry-run)
                DRY_RUN=true
                shift
                ;;
            -f|--force)
                DRY_RUN=false
                shift
                ;;
            -t|--type)
                REPORT_TYPE="$2"
                shift 2
                ;;
            -h|--help)
                print_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
        esac
    done
}

check_security_reports_directory() {
    if [ ! -d "$SECURITY_REPORTS_DIR" ]; then
        log_error "Security reports directory does not exist: $SECURITY_REPORTS_DIR"
        exit 1
    fi
}

analyze_reports() {
    log_info "Analyzing security reports in: $SECURITY_REPORTS_DIR"
    echo

    # Count total files
    TOTAL_FILES_FOUND=$(find "$SECURITY_REPORTS_DIR" -type f | wc -l)

    if [ "$TOTAL_FILES_FOUND" -eq 0 ]; then
        log_success "No security reports found to clean up"
        exit 0
    fi

    log_info "Found $TOTAL_FILES_FOUND security report files"

    # Analyze by type
    echo
    echo "📊 REPORT ANALYSIS:"
    echo "=================="

    declare -A type_counts
    declare -A type_sizes

    while IFS= read -r file; do
        filename=$(basename "$file")

        # Extract report type
        if [[ "$filename" =~ ^([a-zA-Z0-9_-]+)- ]]; then
            report_type="${BASH_REMATCH[1]}"
            ((type_counts["$report_type"]++))
            size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo "0")
            type_sizes["$report_type"]=$(( ${type_sizes["$report_type"]:-0} + size ))
        fi
    done < <(find "$SECURITY_REPORTS_DIR" -type f)

    # Display analysis
    for report_type in "${!type_counts[@]}"; do
        count=${type_counts[$report_type]}
        size_bytes=${type_sizes[$report_type]}
        size_human=$(numfmt --to=iec-i --suffix=B "$size_bytes" 2>/dev/null || echo "${size_bytes}B")
        echo "  $report_type: $count files (${size_human})"
    done

    echo
}

identify_files_to_delete() {
    log_info "Identifying files older than $RETENTION_DAYS days..."

    # Find files to delete
    if [ -n "${REPORT_TYPE:-}" ]; then
        # Specific report type
        files_to_check=$(find "$SECURITY_REPORTS_DIR" -name "${REPORT_TYPE}-*.json" -o -name "${REPORT_TYPE}-*.txt" -o -name "${REPORT_TYPE}-*.sarif" 2>/dev/null)
    else
        # All report types
        files_to_check=$(find "$SECURITY_REPORTS_DIR" -type f \( -name "*.json" -o -name "*.txt" -o -name "*.sarif" \) 2>/dev/null)
    fi

    FILES_TO_DELETE=0
    SPACE_TO_SAVE=0

    echo "$files_to_check" | while read -r file; do
        if [ -z "$file" ] || [ ! -f "$file" ]; then
            continue
        fi

        # Check if file is older than retention period
        if [ $(find "$file" -mtime +"$RETENTION_DAYS" | wc -l) -gt 0 ]; then
            size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo "0")
            SPACE_TO_SAVE=$((SPACE_TO_SAVE + size))
            FILES_TO_DELETE=$((FILES_TO_DELETE + 1))

            if [ "$DRY_RUN" = true ]; then
                echo "  🗑️  Would delete: $(basename "$file") ($(date -r "$file" +%Y-%m-%d))"
            else
                echo "  🗑️  Deleting: $(basename "$file") ($(date -r "$file" +%Y-%m-%d))"
            fi
        fi
    done
}

perform_cleanup() {
    if [ "$FILES_TO_DELETE" -eq 0 ]; then
        log_success "No files to clean up"
        return 0
    fi

    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN - No files will be deleted"
        return 0
    fi

    log_warn "Starting cleanup of $FILES_TO_DELETE files..."

    local deleted_count=0
    local deleted_size=0

    # Find and delete old files
    if [ -n "${REPORT_TYPE:-}" ]; then
        # Specific report type
        while IFS= read -r file; do
            if [ -f "$file" ] && [ $(find "$file" -mtime +"$RETENTION_DAYS" | wc -l) -gt 0 ]; then
                size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo "0")
                rm -f "$file"
                deleted_size=$((deleted_size + size))
                ((deleted_count++))
                log_info "Deleted: $(basename "$file")"
            fi
        done < <(find "$SECURITY_REPORTS_DIR" -name "${REPORT_TYPE}-*.json" -o -name "${REPORT_TYPE}-*.txt" -o -name "${REPORT_TYPE}-*.sarif" 2>/dev/null)
    else
        # All report types
        while IFS= read -r file; do
            if [ -f "$file" ] && [ $(find "$file" -mtime +"$RETENTION_DAYS" | wc -l) -gt 0 ]; then
                size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo "0")
                rm -f "$file"
                deleted_size=$((deleted_size + size))
                ((deleted_count++))
                log_info "Deleted: $(basename "$file")"
            fi
        done < <(find "$SECURITY_REPORTS_DIR" -type f \( -name "*.json" -o -name "*.txt" -o -name "*.sarif" \) 2>/dev/null)
    fi

    # Report results
    space_human=$(numfmt --to=iec-i --suffix=B "$deleted_size" 2>/dev/null || echo "${deleted_size}B")
    log_success "Cleanup completed: $deleted_count files deleted, $space_human freed"
}

generate_cleanup_report() {
    local report_file="$SECURITY_REPORTS_DIR/cleanup-report-$(date +%Y%m%d-%H%M%S).json"

    cat > "$report_file" << EOF
{
  "cleanup_operation": {
    "timestamp": "$(date -Iseconds)",
    "retention_days": $RETENTION_DAYS,
    "dry_run": $DRY_RUN,
    "report_type_filter": "${REPORT_TYPE:-all}",
    "total_files_found": $TOTAL_FILES_FOUND,
    "files_deleted": $FILES_TO_DELETE,
    "space_saved_bytes": $SPACE_TO_SAVE
  },
  "directory_info": {
    "path": "$SECURITY_REPORTS_DIR",
    "remaining_files": $(find "$SECURITY_REPORTS_DIR" -type f 2>/dev/null | wc -l),
    "remaining_size_bytes": $(du -sb "$SECURITY_REPORTS_DIR" 2>/dev/null | cut -f1)
  }
}
EOF

    log_success "Cleanup report generated: $report_file"
}

main() {
    echo "🧹 Security Reports Cleanup"
    echo "=========================="
    echo

    parse_arguments "$@"
    check_security_reports_directory
    analyze_reports
    identify_files_to_delete

    echo
    if [ "$FILES_TO_DELETE" -gt 0 ]; then
        space_human=$(numfmt --to=iec-i --suffix=B "$SPACE_TO_SAVE" 2>/dev/null || echo "${SPACE_TO_SAVE}B")
        echo "📋 CLEANUP SUMMARY:"
        echo "=================="
        echo "  Files to delete: $FILES_TO_DELETE"
        echo "  Space to save: $space_human"
        echo "  Retention period: $RETENTION_DAYS days"
        if [ -n "${REPORT_TYPE:-}" ]; then
            echo "  Report type filter: $REPORT_TYPE"
        fi
        echo

        perform_cleanup
    else
        echo "✅ No files older than $RETENTION_DAYS days found"
    fi

    generate_cleanup_report

    echo
    if [ "$DRY_RUN" = true ]; then
        log_info "This was a dry run. Use --force to actually delete files."
    fi
}

# Run main function
main "$@"