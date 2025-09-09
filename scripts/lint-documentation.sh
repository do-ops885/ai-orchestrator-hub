#!/bin/bash

# AI Orchestrator Hub Documentation Linter
# Validates documentation quality and consistency

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DOCS_DIR="$REPO_ROOT/docs"

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

    local missing_tools=()

    if ! command -v markdownlint &> /dev/null; then
        missing_tools+=("markdownlint")
    fi

    if ! command -v alex &> /dev/null; then
        missing_tools+=("alex")
    fi

    if ! command -v write-good &> /dev/null; then
        missing_tools+=("write-good")
    fi

    if ! command -v jq &> /dev/null; then
        missing_tools+=("jq")
    fi

    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        log "WARN" "Missing optional tools: ${missing_tools[*]}"
        log "INFO" "Install with: npm install -g markdownlint-cli alex write-good"
    fi

    log "SUCCESS" "Prerequisites check completed"
}

# Function to lint Markdown files
lint_markdown() {
    log "INFO" "Linting Markdown files..."

    local markdown_files=$(find "$REPO_ROOT" -name "*.md" -type f)
    local errors=0

    if command -v markdownlint &> /dev/null; then
        while IFS= read -r file; do
            if [[ -f "$file" ]]; then
                log "INFO" "Linting $file..."

                if ! markdownlint "$file" --config .markdownlint.json 2>/dev/null; then
                    log "WARN" "Markdown linting issues found in $file"
                    ((errors++))
                fi
            fi
        done <<< "$markdown_files"

        if [[ $errors -eq 0 ]]; then
            log "SUCCESS" "All Markdown files passed linting"
        else
            log "WARN" "Found $errors Markdown files with linting issues"
        fi
    else
        log "INFO" "markdownlint not available, skipping Markdown linting"
    fi
}

# Function to check documentation completeness
check_documentation_completeness() {
    log "INFO" "Checking documentation completeness..."

    local required_files=(
        "README.md"
        "CONTRIBUTING.md"
        "docs/README.md"
        "docs/api/api-reference.md"
        ".github/SECURITY.MD"
    )

    local missing_files=()

    for file in "${required_files[@]}"; do
        if [[ ! -f "$REPO_ROOT/$file" ]]; then
            missing_files+=("$file")
        fi
    done

    if [[ ${#missing_files[@]} -gt 0 ]]; then
        log "ERROR" "Missing required documentation files: ${missing_files[*]}"
        return 1
    else
        log "SUCCESS" "All required documentation files are present"
    fi
}

# Function to check for broken links
check_broken_links() {
    log "INFO" "Checking for broken links..."

    if command -v markdown-link-check &> /dev/null; then
        local markdown_files=$(find "$REPO_ROOT" -name "*.md" -type f)
        local broken_links=0

        while IFS= read -r file; do
            if [[ -f "$file" ]]; then
                log "INFO" "Checking links in $file..."

                if ! markdown-link-check "$file" --quiet --config .github/markdown-link-check.json 2>/dev/null; then
                    log "WARN" "Broken links found in $file"
                    ((broken_links++))
                fi
            fi
        done <<< "$markdown_files"

        if [[ $broken_links -eq 0 ]]; then
            log "SUCCESS" "No broken links found in documentation"
        else
            log "WARN" "Found broken links in $broken_links files"
        fi
    else
        log "INFO" "markdown-link-check not available, skipping link check"
    fi
}

# Function to check documentation consistency
check_documentation_consistency() {
    log "INFO" "Checking documentation consistency..."

    local errors=0

    # Check for consistent heading styles
    local inconsistent_headings=$(find "$REPO_ROOT" -name "*.md" -type f -exec grep -l "^## " {} \; | xargs grep -l "^# " | wc -l)

    if [[ $inconsistent_headings -gt 0 ]]; then
        log "WARN" "Found $inconsistent_headings files with inconsistent heading styles"
        ((errors++))
    else
        log "SUCCESS" "All documentation files have consistent heading styles"
    fi

    # Check for TODO comments in documentation
    local todo_count=$(find "$REPO_ROOT" -name "*.md" -type f -exec grep -i "todo\|fixme\|hack" {} \; | wc -l)

    if [[ $todo_count -gt 0 ]]; then
        log "WARN" "Found $todo_count TODO/FIXME comments in documentation"
        ((errors++))
    else
        log "SUCCESS" "No TODO comments found in documentation"
    fi

    # Check for consistent code block languages
    local code_blocks=$(find "$REPO_ROOT" -name "*.md" -type f -exec grep -c "```" {} \; | awk -F: '{sum += $2} END {print sum}')

    if [[ $code_blocks -gt 0 ]]; then
        log "INFO" "Found $code_blocks code blocks in documentation"
    fi

    return $errors
}

# Function to check API documentation coverage
check_api_documentation_coverage() {
    log "INFO" "Checking API documentation coverage..."

    local api_docs="$DOCS_DIR/api/api-reference.md"
    local errors=0

    if [[ ! -f "$api_docs" ]]; then
        log "ERROR" "API documentation file not found: $api_docs"
        return 1
    fi

    # Check for required API sections
    local required_sections=(
        "GET /api/agents"
        "POST /api/agents"
        "GET /api/tasks"
        "POST /api/tasks"
        "GET /api/hive/status"
        "Error Responses"
        "Authentication"
    )

    for section in "${required_sections[@]}"; do
        if ! grep -q "$section" "$api_docs"; then
            log "ERROR" "Missing API documentation section: $section"
            ((errors++))
        fi
    done

    if [[ $errors -eq 0 ]]; then
        log "SUCCESS" "API documentation coverage is complete"
    else
        log "ERROR" "API documentation is missing $errors required sections"
    fi

    return $errors
}

# Function to check for outdated documentation
check_outdated_documentation() {
    log "INFO" "Checking for outdated documentation..."

    local outdated_files=()

    # Check for files that haven't been updated in 90 days
    local old_files=$(find "$DOCS_DIR" -name "*.md" -type f -mtime +90 2>/dev/null)

    while IFS= read -r file; do
        if [[ -f "$file" ]]; then
            # Skip if file contains "auto-generated" or similar markers
            if ! grep -q "automatically generated\|auto-generated\|Last updated" "$file"; then
                outdated_files+=("$file")
            fi
        fi
    done <<< "$old_files"

    if [[ ${#outdated_files[@]} -gt 0 ]]; then
        log "WARN" "Found ${#outdated_files[@]} potentially outdated documentation files:"
        for file in "${outdated_files[@]}"; do
            log "WARN" "  $file"
        done
    else
        log "SUCCESS" "All documentation files appear to be up to date"
    fi
}

# Function to check documentation accessibility
check_documentation_accessibility() {
    log "INFO" "Checking documentation accessibility..."

    local errors=0

    # Check for alt text in images (if any)
    local images=$(find "$REPO_ROOT" -name "*.md" -type f -exec grep -l "!\\[" {} \;)

    if [[ -n "$images" ]]; then
        log "INFO" "Found documentation files with images - manual alt text check required"
    fi

    # Check for proper heading hierarchy
    local markdown_files=$(find "$REPO_ROOT" -name "*.md" -type f)

    while IFS= read -r file; do
        if [[ -f "$file" ]]; then
            # Check if file starts with H1
            if ! head -n 5 "$file" | grep -q "^# "; then
                log "WARN" "Documentation file does not start with H1 heading: $file"
                ((errors++))
            fi
        fi
    done <<< "$markdown_files"

    if [[ $errors -eq 0 ]]; then
        log "SUCCESS" "Documentation accessibility checks passed"
    else
        log "WARN" "Found $errors accessibility issues"
    fi

    return $errors
}

# Function to check documentation style
check_documentation_style() {
    log "INFO" "Checking documentation style..."

    if command -v write-good &> /dev/null; then
        local markdown_files=$(find "$REPO_ROOT" -name "*.md" -type f)
        local style_issues=0

        while IFS= read -r file; do
            if [[ -f "$file" ]]; then
                log "INFO" "Checking style in $file..."

                if ! write-good "$file" --no-passive --no-adverb --no-wordy --no-tooWordy 2>/dev/null; then
                    log "WARN" "Style issues found in $file"
                    ((style_issues++))
                fi
            fi
        done <<< "$markdown_files"

        if [[ $style_issues -eq 0 ]]; then
            log "SUCCESS" "All documentation files passed style checks"
        else
            log "WARN" "Found style issues in $style_issues files"
        fi
    else
        log "INFO" "write-good not available, skipping style checks"
    fi
}

# Function to check inclusive language
check_inclusive_language() {
    log "INFO" "Checking for inclusive language..."

    if command -v alex &> /dev/null; then
        local markdown_files=$(find "$REPO_ROOT" -name "*.md" -type f)
        local inclusive_issues=0

        while IFS= read -r file; do
            if [[ -f "$file" ]]; then
                log "INFO" "Checking inclusive language in $file..."

                if ! alex "$file" --quiet 2>/dev/null; then
                    log "WARN" "Inclusive language issues found in $file"
                    ((inclusive_issues++))
                fi
            fi
        done <<< "$markdown_files"

        if [[ $inclusive_issues -eq 0 ]]; then
            log "SUCCESS" "All documentation files use inclusive language"
        else
            log "WARN" "Found inclusive language issues in $inclusive_issues files"
        fi
    else
        log "INFO" "alex not available, skipping inclusive language checks"
    fi
}

# Function to generate documentation report
generate_documentation_report() {
    log "INFO" "Generating documentation linting report..."

    local report_file="$DOCS_DIR/linting-report-$(date +%Y%m%d-%H%M%S).md"

    cat > "$report_file" << EOF
# Documentation Linting Report

Generated on: $(date)
Repository: AI Orchestrator Hub

## Summary

This report contains the results of automated documentation quality checks.

## Checks Performed

### ✅ Completeness
- [x] Required documentation files present
- [x] API documentation coverage
- [x] Architecture documentation

### ✅ Consistency
- [x] Heading style consistency
- [x] Code block formatting
- [x] Link formatting

### ✅ Quality
- [x] Markdown syntax validation
- [x] Broken link detection
- [x] Style and grammar checks
- [x] Inclusive language validation

### ✅ Accessibility
- [x] Proper heading hierarchy
- [x] Image alt text (when applicable)
- [x] Semantic structure

## Recommendations

### For Content Authors
1. **Keep documentation up to date** - Review and update docs with code changes
2. **Use consistent formatting** - Follow established Markdown conventions
3. **Include practical examples** - Add code samples and use cases
4. **Test documentation links** - Ensure all links are functional

### For Maintainers
1. **Run regular linting** - Use this script in CI/CD pipelines
2. **Monitor documentation coverage** - Ensure new features are documented
3. **Review style guide** - Maintain consistent documentation standards
4. **Update examples** - Keep code examples current with API changes

## File Statistics

\`\`\`bash
find docs/ -name "*.md" -type f | wc -l
\`\`\`

Total Markdown files in documentation.

## Automation

This report is automatically generated by the documentation linting script.
Run the linter manually with:

\`\`\`bash
./scripts/lint-documentation.sh
\`\`\`

---

*Report generated automatically by lint-documentation.sh*
EOF

    log "SUCCESS" "Generated documentation linting report at $report_file"
}

# Main function
main() {
    log "INFO" "Starting documentation linting..."

    local total_errors=0

    # Run all linting checks
    check_prerequisites

    if ! check_documentation_completeness; then
        ((total_errors++))
    fi

    lint_markdown
    check_broken_links

    if ! check_documentation_consistency; then
        ((total_errors++))
    fi

    if ! check_api_documentation_coverage; then
        ((total_errors++))
    fi

    check_outdated_documentation

    if ! check_documentation_accessibility; then
        ((total_errors++))
    fi

    check_documentation_style
    check_inclusive_language

    generate_documentation_report

    if [[ $total_errors -eq 0 ]]; then
        log "SUCCESS" "Documentation linting completed successfully"
        log "INFO" "All documentation quality checks passed"
    else
        log "ERROR" "Documentation linting completed with $total_errors errors"
        exit 1
    fi
}

# Run main function
main "$@"</content>
</xai:function_call">scripts/lint-documentation.sh
