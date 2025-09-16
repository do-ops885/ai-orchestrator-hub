#!/bin/bash
# Quality Metrics Processing Script
# This script processes and analyzes quality metrics from CI/CD workflows

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Load configuration
load_config() {
    CONFIG_FILE="$PROJECT_ROOT/.github/quality-config.yml"
    if [ ! -f "$CONFIG_FILE" ]; then
        log_error "Configuration file not found: $CONFIG_FILE"
        exit 1
    fi

    # Use yq if available, otherwise fall back to grep/sed
    if command -v yq &> /dev/null; then
        COVERAGE_TS_MIN=$(yq '.coverage.typescript.minimum' "$CONFIG_FILE")
        COVERAGE_RS_MIN=$(yq '.coverage.rust.minimum' "$CONFIG_FILE")
        COMPLEXITY_TS_MAX=$(yq '.complexity.typescript.maximum_per_function' "$CONFIG_FILE")
        SECURITY_MAX_VULNS=$(yq '.security.maximum_vulnerabilities' "$CONFIG_FILE")
    else
        log_warn "yq not found, using fallback parsing"
        COVERAGE_TS_MIN=80
        COVERAGE_RS_MIN=75
        COMPLEXITY_TS_MAX=15
        SECURITY_MAX_VULNS=0
    fi
}

# Process coverage metrics
process_coverage() {
    log_info "Processing coverage metrics..."

    local coverage_dir="$1"
    local output_file="$2"

    # Initialize coverage data
    local ts_coverage=0
    local rs_coverage=0
    local total_lines=0

    # Process TypeScript coverage
    if [ -d "$coverage_dir/frontend/coverage" ]; then
        ts_coverage_file=$(find "$coverage_dir/frontend/coverage" -name "*.json" | head -1)
        if [ -f "$ts_coverage_file" ]; then
            ts_coverage=$(jq -r '.total.lines.pct // 0' "$ts_coverage_file" 2>/dev/null || echo "0")
            log_info "TypeScript coverage: ${ts_coverage}%"
        fi
    fi

    # Process Rust coverage
    if [ -f "$coverage_dir/rust-lcov.info" ]; then
        # Simple LCOV parsing - in production, use lcov tools
        total_lines=$(grep -c "DA:" "$coverage_dir/rust-lcov.info" 2>/dev/null || echo "0")
        covered_lines=$(grep "DA:" "$coverage_dir/rust-lcov.info" | awk -F: '{if($3>0) sum++} END{print sum+0}' 2>/dev/null || echo "0")

        if [ "$total_lines" -gt 0 ]; then
            rs_coverage=$((covered_lines * 100 / total_lines))
        fi
        log_info "Rust coverage: ${rs_coverage}% (${covered_lines}/${total_lines} lines)"
    fi

    # Generate coverage report
    cat > "$output_file" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "coverage": {
    "typescript": {
      "percentage": $ts_coverage,
      "threshold": $COVERAGE_TS_MIN,
      "status": "$([ $(echo "$ts_coverage >= $COVERAGE_TS_MIN" | bc -l 2>/dev/null || echo "0") -eq 1 ] && echo "pass" || echo "fail")"
    },
    "rust": {
      "percentage": $rs_coverage,
      "lines_covered": $covered_lines,
      "lines_total": $total_lines,
      "threshold": $COVERAGE_RS_MIN,
      "status": "$([ $(echo "$rs_coverage >= $COVERAGE_RS_MIN" | bc -l 2>/dev/null || echo "0") -eq 1 ] && echo "pass" || echo "fail")"
    }
  }
}
EOF

    log_success "Coverage metrics processed and saved to $output_file"
}

# Process complexity metrics
process_complexity() {
    log_info "Processing complexity metrics..."

    local complexity_dir="$1"
    local output_file="$2"

    # Initialize complexity data
    local ts_high_complexity=0
    local rs_high_complexity=0
    local ts_avg_complexity=0
    local rs_avg_complexity=0

    # Process TypeScript complexity
    if [ -f "$complexity_dir/frontend-complexity.json" ]; then
        ts_high_complexity=$(jq "[.[] | select(.complexity > $COMPLEXITY_TS_MAX)] | length" "$complexity_dir/frontend-complexity.json" 2>/dev/null || echo "0")
        ts_avg_complexity=$(jq "[.[] | .complexity] | if length > 0 then add / length else 0 end" "$complexity_dir/frontend-complexity.json" 2>/dev/null || echo "0")
        log_info "TypeScript complexity: avg ${ts_avg_complexity}, high complexity files: $ts_high_complexity"
    fi

    # Process Rust complexity
    if [ -f "$complexity_dir/rust-complexity.json" ]; then
        rs_high_complexity=$(jq ".files | map(select(.complexity > 20)) | length" "$complexity_dir/rust-complexity.json" 2>/dev/null || echo "0")
        rs_avg_complexity=$(jq ".files | map(.complexity) | if length > 0 then add / length else 0 end" "$complexity_dir/rust-complexity.json" 2>/dev/null || echo "0")
        log_info "Rust complexity: avg ${rs_avg_complexity}, high complexity files: $rs_high_complexity"
    fi

    # Generate complexity report
    cat > "$output_file" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "complexity": {
    "typescript": {
      "average": $ts_avg_complexity,
      "high_complexity_files": $ts_high_complexity,
      "threshold": $COMPLEXITY_TS_MAX,
      "status": "$([ $ts_high_complexity -le 5 ] && echo "pass" || echo "fail")"
    },
    "rust": {
      "average": $rs_avg_complexity,
      "high_complexity_files": $rs_high_complexity,
      "threshold": 20,
      "status": "$([ $rs_high_complexity -le 3 ] && echo "pass" || echo "fail")"
    }
  }
}
EOF

    log_success "Complexity metrics processed and saved to $output_file"
}

# Process security metrics
process_security() {
    log_info "Processing security metrics..."

    local security_dir="$1"
    local output_file="$2"

    # Initialize security data
    local rust_vulnerabilities=0
    local npm_vulnerabilities=0
    local secrets_found=0
    local container_vulns=0

    # Process cargo audit results
    if [ -f "$security_dir/security-reports/backend/audit.json" ]; then
        rust_vulnerabilities=$(jq '.vulnerabilities.count // 0' "$security_dir/security-reports/backend/audit.json" 2>/dev/null || echo "0")
    fi

    # Process npm audit results
    if [ -f "$security_dir/frontend-security-reports/frontend/audit-results.json" ]; then
        npm_vulnerabilities=$(jq '.metadata.vulnerabilities.total // 0' "$security_dir/frontend-security-reports/frontend/audit-results.json" 2>/dev/null || echo "0")
    fi

    # Process secrets scan results
    if [ -f "$security_dir/security-scan-results/gitleaks-report.json" ]; then
        secrets_found=$(jq 'length' "$security_dir/security-scan-results/gitleaks-report.json" 2>/dev/null || echo "0")
    fi

    # Process container scan results
    if [ -f "trivy-results.sarif" ]; then
        container_vulns=$(jq '.runs[0].results | length' trivy-results.sarif 2>/dev/null || echo "0")
    fi

    local total_vulnerabilities=$((rust_vulnerabilities + npm_vulnerabilities + container_vulns))

    # Generate security report
    cat > "$output_file" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "security": {
    "vulnerabilities": {
      "total": $total_vulnerabilities,
      "rust": $rust_vulnerabilities,
      "npm": $npm_vulnerabilities,
      "container": $container_vulns
    },
    "secrets": {
      "found": $secrets_found
    },
    "thresholds": {
      "max_vulnerabilities": $SECURITY_MAX_VULNS,
      "max_secrets": 0
    },
    "status": {
      "vulnerabilities": "$([ $total_vulnerabilities -le $SECURITY_MAX_VULNS ] && echo "pass" || echo "fail")",
      "secrets": "$([ $secrets_found -eq 0 ] && echo "pass" || echo "fail")"
    }
  }
}
EOF

    log_success "Security metrics processed and saved to $output_file"
}

# Generate quality report
generate_report() {
    local metrics_dir="$1"
    local output_file="$2"

    log_info "Generating comprehensive quality report..."

    # Read all metrics files
    local coverage_data=""
    local complexity_data=""
    local security_data=""

    if [ -f "$metrics_dir/coverage-metrics.json" ]; then
        coverage_data=$(cat "$metrics_dir/coverage-metrics.json")
    fi

    if [ -f "$metrics_dir/complexity-metrics.json" ]; then
        complexity_data=$(cat "$metrics_dir/complexity-metrics.json")
    fi

    if [ -f "$metrics_dir/security-metrics.json" ]; then
        security_data=$(cat "$metrics_dir/security-metrics.json")
    fi

    # Generate comprehensive report
    cat > "$output_file" << EOF
{
  "generated_at": "$(date -Iseconds)",
  "quality_report": {
    "coverage": $coverage_data,
    "complexity": $complexity_data,
    "security": $security_data,
    "summary": {
      "overall_status": "unknown",
      "critical_issues": 0,
      "recommendations": []
    }
  }
}
EOF

    # Calculate overall status
    local overall_status="pass"
    local critical_issues=0
    local recommendations=()

    # Check coverage status
    if [ -n "$coverage_data" ]; then
        ts_status=$(echo "$coverage_data" | jq -r '.coverage.typescript.status' 2>/dev/null || echo "unknown")
        rs_status=$(echo "$coverage_data" | jq -r '.coverage.rust.status' 2>/dev/null || echo "unknown")

        if [ "$ts_status" = "fail" ] || [ "$rs_status" = "fail" ]; then
            overall_status="fail"
            critical_issues=$((critical_issues + 1))
            recommendations+=("Improve code coverage to meet minimum thresholds")
        fi
    fi

    # Check complexity status
    if [ -n "$complexity_data" ]; then
        ts_complexity_status=$(echo "$complexity_data" | jq -r '.complexity.typescript.status' 2>/dev/null || echo "unknown")
        rs_complexity_status=$(echo "$complexity_data" | jq -r '.complexity.rust.status' 2>/dev/null || echo "unknown")

        if [ "$ts_complexity_status" = "fail" ] || [ "$rs_complexity_status" = "fail" ]; then
            overall_status="fail"
            critical_issues=$((critical_issues + 1))
            recommendations+=("Reduce code complexity by refactoring complex functions")
        fi
    fi

    # Check security status
    if [ -n "$security_data" ]; then
        vuln_status=$(echo "$security_data" | jq -r '.security.status.vulnerabilities' 2>/dev/null || echo "unknown")
        secrets_status=$(echo "$security_data" | jq -r '.security.status.secrets' 2>/dev/null || echo "unknown")

        if [ "$vuln_status" = "fail" ] || [ "$secrets_status" = "fail" ]; then
            overall_status="fail"
            critical_issues=$((critical_issues + 1))
            recommendations+=("Address security vulnerabilities and remove secrets")
        fi
    fi

    # Update report with calculated values
    jq --arg status "$overall_status" \
       --argjson issues $critical_issues \
       --args recommendations \
       '.quality_report.summary.overall_status = $status | .quality_report.summary.critical_issues = $issues | .quality_report.summary.recommendations = $recommendations' \
       "$output_file" > "${output_file}.tmp" && mv "${output_file}.tmp" "$output_file"

    log_success "Comprehensive quality report generated: $output_file"

    # Print summary
    echo "=== Quality Report Summary ==="
    echo "Overall Status: $overall_status"
    echo "Critical Issues: $critical_issues"
    if [ ${#recommendations[@]} -gt 0 ]; then
        echo "Recommendations:"
        for rec in "${recommendations[@]}"; do
            echo "  - $rec"
        done
    fi
}

# Main function
main() {
    local input_dir="${1:-.}"
    local output_dir="${2:-quality-reports}"

    log_info "Starting quality metrics processing..."
    log_info "Input directory: $input_dir"
    log_info "Output directory: $output_dir"

    # Load configuration
    load_config

    # Create output directory
    mkdir -p "$output_dir"

    # Process different types of metrics
    process_coverage "$input_dir" "$output_dir/coverage-metrics.json"
    process_complexity "$input_dir" "$output_dir/complexity-metrics.json"
    process_security "$input_dir" "$output_dir/security-metrics.json"

    # Generate comprehensive report
    generate_report "$output_dir" "$output_dir/quality-report.json"

    log_success "Quality metrics processing completed successfully"
}

# Show usage if requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: $0 [input_directory] [output_directory]"
    echo ""
    echo "Process quality metrics from CI/CD workflow artifacts."
    echo ""
    echo "Arguments:"
    echo "  input_directory   Directory containing metrics artifacts (default: .)"
    echo "  output_directory  Directory to save processed metrics (default: quality-reports)"
    echo ""
    echo "Examples:"
    echo "  $0 . quality-reports"
    echo "  $0 /tmp/artifacts /tmp/processed"
    exit 0
fi

# Run main function
main "$@"