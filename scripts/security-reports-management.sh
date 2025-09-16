#!/bin/bash
# Security Reports Management Script
# This script provides a unified interface for managing security reports

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

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

log_header() {
    echo -e "${MAGENTA}$1${NC}"
}

log_command() {
    echo -e "${CYAN}$1${NC}"
}

print_banner() {
    echo
    echo "================================================================="
    echo "🔒 SECURITY REPORTS MANAGEMENT SYSTEM"
    echo "================================================================="
    echo
    echo "This system ensures all security reports are properly organized,"
    echo "validated, and maintained according to project standards."
    echo
    echo "📁 Standard Location: security-reports/"
    echo "🏷️  Naming Convention: {type}-YYYYMMDD-HHMMSS.{extension}"
    echo
}

print_menu() {
    echo "Available Operations:"
    echo "===================="
    echo
    echo "1) 🔍 Validate Security Reports Compliance"
    echo "   Check that all security reports are in the correct location"
    echo "   and follow proper naming conventions."
    echo
    echo "2) 📋 Check for Misplaced Security Files"
    echo "   Find security-related files that are not in security-reports/."
    echo
    echo "3) 🏷️  Validate Naming Conventions"
    echo "   Ensure all security reports follow the naming standard."
    echo
    echo "4) 🧹 Cleanup Old Security Reports"
    echo "   Remove security reports older than a specified number of days."
    echo
    echo "5) 📊 Generate Security Reports Summary"
    echo "   Show statistics and information about current security reports."
    echo
    echo "6) 📝 Create New Security Report Script"
    echo "   Generate a new script template for creating security reports."
    echo
    echo "7) ℹ️  Show Help and Documentation"
    echo "   Display detailed help and usage information."
    echo
    echo "0) Exit"
    echo
}

validate_compliance() {
    log_header "🔍 VALIDATING SECURITY REPORTS COMPLIANCE"
    echo

    if [ ! -f "./validate-security-reports-compliance.sh" ]; then
        log_error "Validation script not found. Please ensure you're in the scripts directory."
        return 1
    fi

    log_command "./validate-security-reports-compliance.sh"
    ./validate-security-reports-compliance.sh
}

check_misplaced_files() {
    log_header "📋 CHECKING FOR MISPLACED SECURITY FILES"
    echo

    if [ ! -f "./validate-security-reports-location.sh" ]; then
        log_error "Location validation script not found."
        return 1
    fi

    log_command "./validate-security-reports-location.sh"
    ./validate-security-reports-location.sh
}

validate_naming() {
    log_header "🏷️  VALIDATING NAMING CONVENTIONS"
    echo

    if [ ! -f "./validate-security-reports-naming.sh" ]; then
        log_error "Naming validation script not found."
        return 1
    fi

    log_command "./validate-security-reports-naming.sh"
    ./validate-security-reports-naming.sh
}

cleanup_reports() {
    log_header "🧹 CLEANUP OLD SECURITY REPORTS"
    echo

    if [ ! -f "./cleanup-security-reports.sh" ]; then
        log_error "Cleanup script not found."
        return 1
    fi

    echo "This will help you clean up old security reports to manage disk space."
    echo
    read -p "Enter retention period in days (default: 30): " retention_days
    retention_days=${retention_days:-30}

    read -p "Perform dry run first? (y/n, default: y): " dry_run
    dry_run=${dry_run:-y}

    if [[ "$dry_run" =~ ^[Yy]$ ]]; then
        log_command "./cleanup-security-reports.sh --days $retention_days --dry-run"
        ./cleanup-security-reports.sh --days "$retention_days" --dry-run

        echo
        read -p "Proceed with actual cleanup? (y/n): " proceed
        if [[ "$proceed" =~ ^[Yy]$ ]]; then
            log_command "./cleanup-security-reports.sh --days $retention_days --force"
            ./cleanup-security-reports.sh --days "$retention_days" --force
        else
            log_info "Cleanup cancelled."
        fi
    else
        log_command "./cleanup-security-reports.sh --days $retention_days --force"
        ./cleanup-security-reports.sh --days "$retention_days" --force
    fi
}

show_summary() {
    log_header "📊 SECURITY REPORTS SUMMARY"
    echo

    SECURITY_REPORTS_DIR="$PROJECT_ROOT/security-reports"

    if [ ! -d "$SECURITY_REPORTS_DIR" ]; then
        log_error "Security reports directory does not exist: $SECURITY_REPORTS_DIR"
        return 1
    fi

    echo "📁 Directory: $SECURITY_REPORTS_DIR"
    echo

    # Count files by type
    total_files=$(find "$SECURITY_REPORTS_DIR" -type f | wc -l)
    json_files=$(find "$SECURITY_REPORTS_DIR" -name "*.json" | wc -l)
    txt_files=$(find "$SECURITY_REPORTS_DIR" -name "*.txt" | wc -l)
    sarif_files=$(find "$SECURITY_REPORTS_DIR" -name "*.sarif" | wc -l)

    echo "📈 File Counts:"
    echo "  Total: $total_files"
    echo "  JSON:  $json_files"
    echo "  Text:  $txt_files"
    echo "  SARIF: $sarif_files"
    echo

    # Show disk usage
    if command -v du &> /dev/null; then
        disk_usage=$(du -sh "$SECURITY_REPORTS_DIR" | cut -f1)
        echo "💾 Disk Usage: $disk_usage"
        echo
    fi

    # Show recent reports
    if [ $total_files -gt 0 ]; then
        echo "🕐 Recent Reports:"
        find "$SECURITY_REPORTS_DIR" -type f -printf '%T@ %p\n' | sort -n | tail -5 | while read -r line; do
            timestamp=$(echo "$line" | cut -d' ' -f1)
            file=$(echo "$line" | cut -d' ' -f2-)
            filename=$(basename "$file")
            date_str=$(date -d "@$timestamp" +"%Y-%m-%d %H:%M")
            echo "  $date_str - $filename"
        done
        echo
    fi

    # Check for naming convention compliance
    invalid_count=$(find "$SECURITY_REPORTS_DIR" -type f | while read -r file; do
        filename=$(basename "$file")
        if [[ ! "$filename" =~ ^[a-zA-Z0-9_-]+-[0-9]{8}-[0-9]{6}\.(json|txt|sarif)$ ]]; then
            echo "invalid"
        fi
    done | wc -l)

    if [ "$invalid_count" -gt 0 ]; then
        log_warn "Found $invalid_count files that don't follow naming conventions"
    else
        log_success "All files follow naming conventions"
    fi
}

create_new_script() {
    log_header "📝 CREATE NEW SECURITY REPORT SCRIPT"
    echo

    if [ ! -f "./security-report-template.sh" ]; then
        log_error "Template script not found."
        return 1
    fi

    echo "This will create a new security report script based on the template."
    echo
    read -p "Enter script name (without .sh extension): " script_name

    if [ -z "$script_name" ]; then
        log_error "Script name cannot be empty."
        return 1
    fi

    if [ -f "${script_name}.sh" ]; then
        log_error "Script ${script_name}.sh already exists."
        return 1
    fi

    read -p "Enter report type (e.g., cargo-audit, npm-audit): " report_type
    read -p "Enter file extension (json, txt, sarif) [json]: " extension
    extension=${extension:-json}

    # Copy template
    cp "./security-report-template.sh" "${script_name}.sh"

    # Update the script with specific values
    sed -i.bak "s/generate_security_report() {/generate_security_report() {\n    # TODO: Implement $report_type report generation\n    log_info \"Generating $report_type report...\"\n    \n    # Example implementation:\n    # $report_type command --output \"\$output_file\" \n    \n    echo \"{\\\\"report_type\\\": \\\"$report_type\\\", \\\"generated_at\\\": \\\"\$(date -Iseconds)\\\"}\" > \"\$output_file\"\n/" "${script_name}.sh"

    sed -i.bak "s/local report_type=\"\${1:-unknown}\"/local report_type=\"\${1:-$report_type}\"/" "${script_name}.sh"
    sed -i.bak "s/local extension=\"\${2:-json}\"/local extension=\"\${2:-$extension}\"/" "${script_name}.sh"

    # Make executable
    chmod +x "${script_name}.sh"

    # Remove backup file
    rm -f "${script_name}.sh.bak"

    log_success "Created new script: ${script_name}.sh"
    log_info "Please edit the script to implement the actual report generation logic."
    log_info "See the generate_security_report() function for implementation details."
}

show_help() {
    log_header "ℹ️  SECURITY REPORTS MANAGEMENT HELP"
    echo

    echo "OVERVIEW:"
    echo "========="
    echo "This system provides comprehensive management of security reports to ensure"
    echo "consistency, organization, and compliance across the project."
    echo

    echo "STANDARDS:"
    echo "=========="
    echo "📁 Location: All security reports must be in 'security-reports/' directory"
    echo "🏷️  Naming: {type}-YYYYMMDD-HHMMSS.{extension} (e.g., cargo-audit-20231201-143022.json)"
    echo "🔒 Permissions: Files should be 644 (readable by owner/group, no world access)"
    echo "🧹 Cleanup: Old reports should be cleaned up regularly to manage disk space"
    echo

    echo "AVAILABLE SCRIPTS:"
    echo "=================="
    echo "• validate-security-reports-compliance.sh - Comprehensive compliance check"
    echo "• validate-security-reports-location.sh   - Check for misplaced files"
    echo "• validate-security-reports-naming.sh    - Validate naming conventions"
    echo "• cleanup-security-reports.sh            - Remove old reports"
    echo "• security-report-template.sh            - Template for new report scripts"
    echo

    echo "USAGE EXAMPLES:"
    echo "==============="
    echo "# Quick compliance check"
    echo "./security-reports-management.sh"
    echo "Select option 1"
    echo
    echo "# Cleanup reports older than 7 days"
    echo "./cleanup-security-reports.sh --days 7 --force"
    echo
    echo "# Create new audit script"
    echo "./security-reports-management.sh"
    echo "Select option 6, enter script name and details"
    echo

    echo "INTEGRATION:"
    echo "============"
    echo "• Pre-commit hooks automatically validate security reports"
    echo "• CI/CD pipelines include compliance checks"
    echo "• GitHub Actions workflow validates report organization"
    echo "• Automated cleanup can be scheduled via cron"
    echo

    echo "For more information, see CONTRIBUTING.md#security-report-organization"
    echo
}

main() {
    # Change to scripts directory if not already there
    if [[ ! "$PWD" == *"/scripts" ]]; then
        log_info "Changing to scripts directory..."
        cd "$SCRIPT_DIR"
    fi

    while true; do
        print_banner
        print_menu

        read -p "Select operation (0-7): " choice
        echo

        case $choice in
            1)
                validate_compliance
                ;;
            2)
                check_misplaced_files
                ;;
            3)
                validate_naming
                ;;
            4)
                cleanup_reports
                ;;
            5)
                show_summary
                ;;
            6)
                create_new_script
                ;;
            7)
                show_help
                ;;
            0)
                log_success "Goodbye!"
                exit 0
                ;;
            *)
                log_error "Invalid option. Please select 0-7."
                ;;
        esac

        echo
        read -p "Press Enter to continue..."
        clear
    done
}

# Check if running interactively
if [ -t 0 ]; then
    main "$@"
else
    # Non-interactive mode - show help
    print_banner
    show_help
fi