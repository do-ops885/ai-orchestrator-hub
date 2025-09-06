#!/bin/bash

# GitHub Actions Workflows Deployment Script
# This script safely deploys the improved workflows with backup and rollback capabilities

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
WORKFLOWS_DIR="$REPO_ROOT/.github/workflows"
BACKUP_DIR="$REPO_ROOT/.github/workflows-backup-$(date +%Y%m%d-%H%M%S)"

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
    if [[ ! -f "$REPO_ROOT/.github/workflows/build.yml" ]]; then
        log "ERROR" "Not in the correct repository root. Please run from the ai-orchestrator-hub directory."
        exit 1
    fi

    # Check if improved workflows exist
    local improved_workflows=(
        "build-improved.yml"
        "maintenance-improved.yml"
        "lint-improved.yml"
        "pr-validation-improved.yml"
    )

    for workflow in "${improved_workflows[@]}"; do
        if [[ ! -f "$WORKFLOWS_DIR/$workflow" ]]; then
            log "ERROR" "Improved workflow not found: $workflow"
            exit 1
        fi
    done

    # Check if git is available and repo is clean
    if ! command -v git &> /dev/null; then
        log "ERROR" "Git is required but not installed"
        exit 1
    fi

    # Check for uncommitted changes
    if [[ -n $(git status --porcelain) ]]; then
        log "WARN" "You have uncommitted changes. Consider committing them before deployment."
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log "INFO" "Deployment cancelled by user"
            exit 0
        fi
    fi

    log "SUCCESS" "Prerequisites check passed"
}

# Function to create backup
create_backup() {
    log "INFO" "Creating backup of existing workflows..."

    mkdir -p "$BACKUP_DIR"

    # Backup existing workflows
    local original_workflows=(
        "build.yml"
        "maintenance.yml"
        "lint.yml"
        "pr-validation.yml"
        "release.yml"
        "security.yml"
    )

    for workflow in "${original_workflows[@]}"; do
        if [[ -f "$WORKFLOWS_DIR/$workflow" ]]; then
            cp "$WORKFLOWS_DIR/$workflow" "$BACKUP_DIR/"
            log "INFO" "Backed up: $workflow"
        fi
    done

    # Create rollback script
    cat > "$BACKUP_DIR/rollback.sh" << 'EOF'
#!/bin/bash

# Rollback script for GitHub Actions workflows
# This script restores the original workflows from backup

set -e

BACKUP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKFLOWS_DIR="$(cd "$BACKUP_DIR/../workflows" && pwd)"

echo "Rolling back workflows from backup..."

# Restore original workflows
for file in "$BACKUP_DIR"/*.yml; do
    if [[ -f "$file" ]]; then
        filename=$(basename "$file")
        cp "$file" "$WORKFLOWS_DIR/$filename"
        echo "Restored: $filename"
    fi
done

echo "Rollback completed successfully!"
echo "You may need to commit these changes to git."
EOF

    chmod +x "$BACKUP_DIR/rollback.sh"

    log "SUCCESS" "Backup created at: $BACKUP_DIR"
    log "INFO" "Rollback script available at: $BACKUP_DIR/rollback.sh"
}

# Function to deploy improved workflows
deploy_workflows() {
    log "INFO" "Deploying improved workflows..."

    # Deployment mappings
    declare -A workflow_mappings=(
        ["build-improved.yml"]="build.yml"
        ["maintenance-improved.yml"]="maintenance.yml"
        ["lint-improved.yml"]="lint.yml"
        ["pr-validation-improved.yml"]="pr-validation.yml"
    )

    for improved_workflow in "${!workflow_mappings[@]}"; do
        local target_workflow="${workflow_mappings[$improved_workflow]}"

        log "INFO" "Deploying $improved_workflow -> $target_workflow"

        # Copy improved workflow to target location
        cp "$WORKFLOWS_DIR/$improved_workflow" "$WORKFLOWS_DIR/$target_workflow"

        log "SUCCESS" "Deployed: $target_workflow"
    done

    log "SUCCESS" "All improved workflows deployed successfully!"
}

# Function to validate deployment
validate_deployment() {
    log "INFO" "Validating deployment..."

    # Check that all target workflows exist and are not empty
    local target_workflows=(
        "build.yml"
        "maintenance.yml"
        "lint.yml"
        "pr-validation.yml"
    )

    for workflow in "${target_workflows[@]}"; do
        if [[ ! -f "$WORKFLOWS_DIR/$workflow" ]]; then
            log "ERROR" "Deployment validation failed: $workflow not found"
            return 1
        fi

        if [[ ! -s "$WORKFLOWS_DIR/$workflow" ]]; then
            log "ERROR" "Deployment validation failed: $workflow is empty"
            return 1
        fi

        # Check for improved workflow markers
        if ! grep -q "Improved" "$WORKFLOWS_DIR/$workflow"; then
            log "WARN" "Warning: $workflow may not be the improved version"
        fi
    done

    log "SUCCESS" "Deployment validation passed"
}

# Function to create post-deployment checklist
create_checklist() {
    local checklist_file="$REPO_ROOT/post-deployment-checklist.md"

    cat > "$checklist_file" << 'EOF'
# Post-Deployment Checklist for Improved GitHub Actions Workflows

## Immediate Actions (Next 24 hours)

- [ ] **Monitor first workflow runs** - Check that workflows execute without errors
- [ ] **Review structured logs** - Verify JSON logging is working correctly
- [ ] **Test PR validation** - Create a test PR to validate the new PR workflow
- [ ] **Check artifact uploads** - Ensure reports and logs are being uploaded

## Short-term Actions (Next week)

- [ ] **Monitor performance** - Compare execution times with baseline
- [ ] **Review false positive rates** - Track any reduction in flaky tests
- [ ] **Test maintenance workflow** - Manually trigger to verify automation
- [ ] **Validate security scanning** - Ensure vulnerability detection is working
- [ ] **Check dependency updates** - Verify automated PR creation

## Configuration Adjustments

- [ ] **Adjust performance thresholds** - Fine-tune regression detection limits
- [ ] **Configure notification channels** - Set up Slack/Discord webhooks if needed
- [ ] **Review quality gates** - Adjust lint and test coverage thresholds
- [ ] **Customize test strategies** - Modify edge case test scenarios

## Team Communication

- [ ] **Update team documentation** - Document new workflow features
- [ ] **Conduct team training** - Explain new structured logging and reports
- [ ] **Gather feedback** - Collect developer experience feedback
- [ ] **Plan iterations** - Schedule workflow improvement reviews

## Monitoring and Metrics

- [ ] **Set up dashboards** - Create monitoring for key metrics
- [ ] **Track success rates** - Monitor workflow success/failure rates
- [ ] **Measure cost impact** - Track GitHub Actions usage costs
- [ ] **Performance baselines** - Establish new performance benchmarks

## Rollback Plan (if needed)

If issues arise, you can rollback using:
```bash
# Navigate to backup directory
cd .github/workflows-backup-*

# Run rollback script
./rollback.sh

# Commit changes
git add ../.github/workflows/
git commit -m "Rollback to original workflows"
```

## Success Criteria

- ✅ All workflows execute successfully
- ✅ Structured logging provides clear insights
- ✅ No increase in false positive rates
- ✅ Performance improvements are measurable
- ✅ Team feedback is positive

## Notes

- Backup location: `.github/workflows-backup-*`
- Rollback script: `.github/workflows-backup-*/rollback.sh`
- Implementation summary: `tmp_rovodev_implementation_summary.md`

EOF

    log "SUCCESS" "Post-deployment checklist created: $checklist_file"
}

# Function to show deployment summary
show_summary() {
    echo
    echo "=============================================="
    echo "  GitHub Actions Workflows Deployment Summary"
    echo "=============================================="
    echo
    echo "✅ Deployed Workflows:"
    echo "   • build.yml (Enhanced with structured logging, memory leak detection)"
    echo "   • maintenance.yml (Scheduled automation, dependency updates)"
    echo "   • lint.yml (Documentation linting, performance rules)"
    echo "   • pr-validation.yml (Edge case testing, chaos engineering)"
    echo
    echo "📁 Backup Location:"
    echo "   $BACKUP_DIR"
    echo
    echo "🔄 Rollback Command:"
    echo "   $BACKUP_DIR/rollback.sh"
    echo
    echo "📋 Next Steps:"
    echo "   1. Review post-deployment-checklist.md"
    echo "   2. Commit changes to git"
    echo "   3. Monitor first workflow runs"
    echo "   4. Test with a sample PR"
    echo
    echo "🚀 Expected Benefits:"
    echo "   • 30-50% reduction in false positives"
    echo "   • Enhanced security scanning"
    echo "   • Automated maintenance"
    echo "   • Better performance monitoring"
    echo
    echo "=============================================="
}

# Main deployment function
main() {
    local mode="${1:-deploy}"

    case $mode in
        "deploy")
            log "INFO" "Starting GitHub Actions workflows deployment..."
            check_prerequisites
            create_backup
            deploy_workflows
            validate_deployment
            create_checklist
            show_summary
            log "SUCCESS" "Deployment completed successfully!"
            ;;
        "validate")
            log "INFO" "Validating current deployment..."
            validate_deployment
            ;;
        "backup")
            log "INFO" "Creating backup only..."
            check_prerequisites
            create_backup
            ;;
        *)
            echo "Usage: $0 [deploy|validate|backup]"
            echo
            echo "Commands:"
            echo "  deploy   - Deploy improved workflows (default)"
            echo "  validate - Validate current deployment"
            echo "  backup   - Create backup only"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
