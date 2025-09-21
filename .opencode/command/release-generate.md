---
description: Generate a new release with version bumping, comprehensive validation, changelog generation, and CI/CD integration
agent: coordinator
---

# Release Generate Command

Generate a production-ready release for the AI Orchestrator Hub, including version management, validation, documentation updates, and seamless CI/CD integration.

## Environment Setup

Prepare the release environment with all necessary tools and configurations:

```bash
# Verify git repository state
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Git repository has uncommitted changes"
    exit 1
fi

# Ensure on main branch
current_branch=$(git branch --show-current)
if [ "$current_branch" != "main" ]; then
    echo "Error: Must be on main branch for release"
    exit 1
fi

# Check for required tools
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo not found"; exit 1; }
command -v npm >/dev/null 2>&1 || { echo "Error: npm not found"; exit 1; }
command -v gh >/dev/null 2>&1 || { echo "Error: GitHub CLI not found"; exit 1; }

# Set environment variables
export RELEASE_ENV=true
export RUST_BACKTRACE=1
export NODE_ENV=production

# Pull latest changes
git pull origin main --rebase
```

## Version Bumping Logic

Determine and apply version increments for both backend and frontend:

### Version Analysis
```bash
# Get current versions
current_backend_version=$(grep '^version =' backend/Cargo.toml | sed 's/version = "\(.*\)"/\1/')
current_frontend_version=$(grep '"version"' frontend/package.json | sed 's/.*"version": "\([^"]*\)".*/\1/')

# Validate versions
if [ -z "$current_backend_version" ]; then
    echo "Error: Could not determine current backend version"
    exit 1
fi

if [ -z "$current_frontend_version" ]; then
    echo "Error: Could not determine current frontend version"
    exit 1
fi

echo "Current versions:"
echo "Backend: $current_backend_version"
echo "Frontend: $current_frontend_version"

# Determine bump type (patch, minor, major)
read -p "Enter version bump type (patch/minor/major): " bump_type

case $bump_type in
    patch)
        new_version=$(echo $current_backend_version | awk -F. '{$3++; print $1"."$2"."$3}')
        ;;
    minor)
        new_version=$(echo $current_backend_version | awk -F. '{$2++; $3=0; print $1"."$2"."$3}')
        ;;
    major)
        new_version=$(echo $current_backend_version | awk -F. '{$1++; $2=0; $3=0; print $1"."$2"."$3}')
        ;;
    *)
        echo "Invalid bump type"
        exit 1
        ;;
esac

if [ -z "$new_version" ]; then
    echo "Error: Failed to determine new version"
    exit 1
fi

echo "New version: $new_version"
```

### Backend Version Update
```bash
# Update Cargo.toml
sed -i "s/^version = \".*\"/version = \"$new_version\"/" backend/Cargo.toml

# Verify backend version update
if ! grep -q "version = \"$new_version\"" backend/Cargo.toml; then
    echo "Error: Failed to update backend version"
    exit 1
fi

# Update Cargo.lock
cd backend
cargo check
cd ..
```

### Frontend Version Update
```bash
# Update package.json
sed -i "s/\"version\": \".*\"/\"version\": \"$new_version\"/" frontend/package.json

# Verify frontend version update
if ! grep -q "\"version\": \"$new_version\"" frontend/package.json; then
    echo "Error: Failed to update frontend version"
    exit 1
fi

# Update package-lock.json
cd frontend
npm install --package-lock-only
cd ..
```

## Pre-release Validation

Execute comprehensive validation suite before release:

### Code Quality Checks
```bash
# Backend linting
cd backend
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check

# Frontend linting
cd ../frontend
npm run lint
npm run format:check
cd ..
```

### Build Validation
```bash
# Backend build
cd backend
cargo build --release
cargo test --release

# Frontend build
cd ../frontend
npm ci
npm run build
npm run test
cd ..
```

### Security Validation
```bash
# Backend security audit
cd backend
cargo audit

# Frontend security audit
cd ../frontend
npm audit --audit-level high
cd ..

# Run custom security checks
./scripts/security_audit.sh
./scripts/unwrap-prevention-monitor.sh check_unwrap_calls
```

### Integration Testing
```bash
# Start services for integration tests
docker compose up -d database redis

# Run integration tests
cd backend
cargo test --test integration --release
cd ../frontend
npm run test:integration
cd ..

# Stop services
docker compose down
```

### Performance Validation
```bash
# Backend benchmarks
cd backend
cargo bench

# Frontend performance tests
cd ../frontend
npm run test:performance
cd ..
```

## Changelog Generation and Updates

Generate and update changelog with release notes:

### Changelog Analysis
```bash
# Get commits since last release
last_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
if [ -z "$last_tag" ]; then
    commits=$(git log --oneline --pretty=format:"%s")
else
    commits=$(git log --oneline --pretty=format:"%s" $last_tag..HEAD)
fi

if [ -z "$commits" ]; then
    echo "Warning: No commits found since last release"
fi

# Categorize commits
features=$(echo "$commits" | grep -i "^feat:" | sed 's/^feat: /- /')
fixes=$(echo "$commits" | grep -i "^fix:" | sed 's/^fix: /- /')
chores=$(echo "$commits" | grep -i "^chore:" | sed 's/^chore: /- /')
docs=$(echo "$commits" | grep -i "^docs:" | sed 's/^docs: /- /')
```

### Changelog Update
```bash
# Create changelog entry
changelog_entry="## [$new_version] - $(date +%Y-%m-%d)

### Features
$features

### Fixes
$fixes

### Documentation
$docs

### Maintenance
$chores
"

# Update CHANGELOG.md
if [ -f CHANGELOG.md ]; then
    # Insert after header
    sed -i "1a\\
$changelog_entry
" CHANGELOG.md
else
    echo "# Changelog

$changelog_entry" > CHANGELOG.md
fi

# Update RELEASE_NOTES.md
echo "# Release Notes v$new_version

$changelog_entry" > RELEASE_NOTES.md
```

## GitHub Actions Workflow Monitoring

Monitor CI/CD pipeline execution with error detection:

### Workflow Trigger
```bash
# Create and push release branch
release_branch="release/v$new_version"
git checkout -b $release_branch

# Commit version changes
git add backend/Cargo.toml backend/Cargo.lock frontend/package.json frontend/package-lock.json CHANGELOG.md RELEASE_NOTES.md
git commit -m "chore: bump version to $new_version"

# Push release branch
git push origin $release_branch

# Create pull request
if ! gh pr create --title "Release v$new_version" --body "Automated release preparation for version $new_version" --base main --head $release_branch; then
    echo "Error: Failed to create pull request"
    exit 1
fi
```

### Workflow Monitoring
```bash
# Monitor CI status
pr_number=$(gh pr list --head $release_branch --json number --jq '.[0].number')

if [ -z "$pr_number" ] || [ "$pr_number" = "null" ]; then
    echo "Error: Could not find PR number for branch $release_branch"
    exit 1
fi

echo "Monitoring PR #$pr_number workflows..."

# Wait for CI completion
timeout=3600  # 1 hour timeout
elapsed=0
while [ $elapsed -lt $timeout ]; do
    status=$(gh pr checks $pr_number --json name,status,conclusion)
    
    # Check for failures
    failures=$(echo $status | jq '.[] | select(.conclusion == "failure") | .name')
    if [ -n "$failures" ]; then
        echo "CI failures detected: $failures"
        break
    fi
    
    # Check if all completed
    pending=$(echo $status | jq '.[] | select(.status != "completed") | .name')
    if [ -z "$pending" ]; then
        echo "All CI checks completed successfully"
        break
    fi
    
    sleep 30
    elapsed=$((elapsed + 30))
done

if [ $elapsed -ge $timeout ]; then
    echo "CI timeout reached"
    exit 1
fi
```

### Error Detection and Handling
```bash
# Analyze CI failures
if [ -n "$failures" ]; then
    echo "Analyzing CI failures..."
    
    # Get detailed logs
    for failure in $failures; do
        echo "Failure in: $failure"
        gh run list --workflow="$failure" --limit 1 --json url --jq '.[0].url'
    done
    
    # Attempt auto-fix for common issues
    ./scripts/ci-error-fixer.sh "$failures"
    
    # Re-run failed workflows
    gh pr checks $pr_number --rerun-failed
fi
```

## Error Fixing Mechanisms

Implement automated error correction and rollback capabilities:

### Common Error Fixes
```bash
# Function to fix common issues
fix_common_errors() {
    # Fix formatting issues
    cd backend && cargo fmt && cd ..
    cd frontend && npm run format && cd ..
    
    # Fix linting issues
    cd backend && cargo clippy --fix --allow-dirty && cd ..
    cd frontend && npm run lint:fix && cd ..
    
    # Rebuild dependencies
    cd backend && cargo clean && cargo build && cd ..
    cd frontend && rm -rf node_modules && npm ci && cd ..
}

# Function to rollback changes
rollback_release() {
    echo "Rolling back release changes..."
    
    # Delete release branch
    git checkout main
    git branch -D $release_branch
    git push origin --delete $release_branch
    
    # Reset versions
    git checkout HEAD~1 -- backend/Cargo.toml backend/Cargo.lock frontend/package.json frontend/package-lock.json CHANGELOG.md RELEASE_NOTES.md
    
    echo "Rollback completed"
}
```

### Automated Recovery
```bash
# Monitor for recoverable errors
if [ -n "$failures" ]; then
    echo "Attempting automated fixes..."
    
    # Backup current state
    git stash push -m "release-backup-$new_version"
    
    # Apply fixes
    fix_common_errors
    
    # Test fixes
    cd backend && cargo test && cd ..
    cd frontend && npm test && cd ..
    
    # Commit fixes
    git add .
    git commit -m "fix: automated fixes for release v$new_version"
    git push origin $release_branch
    
    # Re-run CI
    gh pr checks $pr_number --rerun-failed
else
    # If unrecoverable, rollback
    rollback_release
    exit 1
fi
```

## Atomic Commit Creation

Create structured commits for the release:

### Commit Strategy
```bash
# Analyze changes for atomic commits
changed_files=$(git diff --name-only HEAD~1)

# Group by type
backend_files=$(echo "$changed_files" | grep "^backend/")
frontend_files=$(echo "$changed_files" | grep "^frontend/")
docs_files=$(echo "$changed_files" | grep -E "\.(md|txt)$")
config_files=$(echo "$changed_files" | grep -E "\.(toml|json|yaml|yml)$" | grep -v "package-lock.json")

# Create atomic commits
if [ -n "$config_files" ]; then
    git add $config_files
    git commit -m "chore: update configuration files for v$new_version"
fi

if [ -n "$backend_files" ]; then
    git add $backend_files
    git commit -m "chore: update backend version to $new_version"
fi

if [ -n "$frontend_files" ]; then
    git add $frontend_files
    git commit -m "chore: update frontend version to $new_version"
fi

if [ -n "$docs_files" ]; then
    git add $docs_files
    git commit -m "docs: update changelog and release notes for v$new_version"
fi

# Push all commits
git push origin $release_branch
```

### Release Finalization
```bash
# Merge PR when all checks pass
if ! gh pr merge $pr_number --merge --delete-branch; then
    echo "Error: Failed to merge pull request"
    exit 1
fi

# Create git tag
git tag -a "v$new_version" -m "Release version $new_version"
git push origin "v$new_version"

# Create GitHub release
if ! gh release create "v$new_version" --title "Release v$new_version" --notes-file RELEASE_NOTES.md; then
    echo "Error: Failed to create GitHub release"
    exit 1
fi

echo "Release v$new_version completed successfully!"
```

## Release Validation

Final validation steps to ensure release quality:

### Post-release Checks
```bash
# Verify deployment
curl -f https://api.github.com/repos/your-org/ai-orchestrator-hub/releases/latest

# Check package registries
cargo search ai-orchestrator-hub
npm view ai-orchestrator-hub version

# Monitor for issues
gh issue list --label bug --state open --limit 5
```

### Rollback Procedures
```bash
# Emergency rollback function
emergency_rollback() {
    echo "Performing emergency rollback..."
    
    # Delete release tag
    git push origin --delete "v$new_version"
    git tag -d "v$new_version"
    
    # Revert to previous version
    git revert HEAD --no-edit
    git push origin main
    
    echo "Emergency rollback completed"
}
```

## Best Practices

1. **Version Consistency**: Ensure backend and frontend versions match
2. **Comprehensive Testing**: Never skip validation steps
3. **Atomic Operations**: Use transactions for database changes if applicable
4. **Monitoring**: Keep close watch on CI/CD pipelines
5. **Rollback Ready**: Always have rollback procedures ready
6. **Documentation**: Keep changelog and release notes up-to-date
7. **Security**: Run security audits before every release
8. **Performance**: Validate performance benchmarks pass

## Common Issues and Solutions

- **Version Conflicts**: Manually resolve version mismatches
- **CI Timeouts**: Increase timeout or optimize workflows
- **Dependency Issues**: Update lockfiles and audit dependencies
- **Merge Conflicts**: Resolve conflicts before merging release PR
- **Security Vulnerabilities**: Address all high/critical vulnerabilities
- **Performance Regressions**: Investigate and fix performance issues

## Integration with CI/CD

This release process integrates seamlessly with GitHub Actions:

- **Automated Triggers**: Release branches trigger CI pipelines
- **Quality Gates**: PR checks prevent merging broken releases
- **Artifact Management**: Build artifacts are stored and versioned
- **Deployment**: Automatic deployment to staging/production environments
- **Notifications**: Team notifications for release events