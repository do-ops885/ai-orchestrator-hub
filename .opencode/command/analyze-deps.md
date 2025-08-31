---
description: Analyze and optimize project dependencies for Rust and Node.js
agent: performance-optimizer
---

# Analyze Dependencies Command

Analyze project dependencies for security vulnerabilities, unused packages, license compliance, and optimization opportunities across Rust and Node.js ecosystems.

## Dependency Analysis Strategy

### 1. Environment Setup
Prepare analysis environment:

```bash
# Ensure clean state
git status

# Update package registries
cargo update
npm update

# Create analysis directory
mkdir -p analysis/$(date +%Y%m%d)
```

### 2. Security Analysis
Perform security vulnerability scanning:

```bash
# Rust security audit
cargo audit

# Generate security report
cargo audit --format json > analysis/security-audit.json

# Node.js security audit
npm audit

# Detailed security report
npm audit --audit-level moderate --json > analysis/npm-audit.json
```

### 3. Dependency Analysis
Analyze dependency usage and health:

```bash
# Rust dependency analysis
cargo tree > analysis/cargo-tree.txt
cargo outdated > analysis/cargo-outdated.txt

# Node.js dependency analysis
npm ls --depth=0 > analysis/npm-dependencies.txt
npx depcheck > analysis/unused-dependencies.txt
```

### 4. License Compliance
Check license compatibility:

```bash
# Rust license checking
cargo license > analysis/rust-licenses.txt

# Node.js license checking
npx license-checker --json > analysis/npm-licenses.json

# Check license compatibility
npx license-checker --failOn analysis/blacklisted-licenses.txt
```

### 5. Bundle Size Analysis
Analyze bundle sizes and optimization opportunities:

```bash
# Frontend bundle analysis
cd frontend
npm run build:analyze

# Rust binary size analysis
cd ../backend
cargo bloat --release > ../analysis/binary-bloat.txt
```

## Analysis Categories

### Security Analysis
- **Vulnerability Scanning**: Known security vulnerabilities
- **Dependency Updates**: Outdated packages with security fixes
- **Supply Chain Risks**: Third-party dependency risks
- **License Compliance**: License compatibility and restrictions

### Performance Analysis
- **Bundle Size**: JavaScript bundle size impact
- **Binary Size**: Rust binary size optimization
- **Load Times**: Dependency impact on application startup
- **Tree Shaking**: Effectiveness of dead code elimination

### Maintenance Analysis
- **Dependency Health**: Package maintenance status
- **Update Frequency**: How often packages are updated
- **Community Support**: Community size and activity
- **Documentation Quality**: Available documentation

## Detailed Analysis

### Rust Dependencies
Analyze Rust crate dependencies:

```bash
# Detailed dependency information
cargo tree --duplicates
cargo tree --edges all

# Check for unused dependencies
cargo +nightly udeps

# Analyze feature usage
cargo tree --features all
```

### Node.js Dependencies
Analyze npm package dependencies:

```bash
# Dependency size analysis
npx webpack-bundle-analyzer build/static/js/*.js

# Package size breakdown
npm ls --depth=0 --json | jq '.dependencies | to_entries | sort_by(.value.version)'

# Check for duplicates
npm ls --depth=10 | grep -E "deduped|DUPLICATE"
```

### Cross-Platform Analysis
Analyze dependencies across platforms:

```bash
# Check platform-specific dependencies
npm ls --depth=0 | grep -E "(darwin|linux|win32)"

# Rust target-specific dependencies
cargo tree --target x86_64-unknown-linux-gnu
cargo tree --target wasm32-unknown-unknown
```

## Optimization Recommendations

### Dependency Cleanup
Generate cleanup recommendations:

```bash
# Identify unused dependencies
npx depcheck --json | jq '.dependencies' > analysis/unused-deps.json

# Suggest dependency updates
npm outdated --json > analysis/outdated-packages.json

# Check for lighter alternatives
npx bundle-phobia-cli package-name
```

### Security Fixes
Address security vulnerabilities:

```bash
# Apply security fixes
npm audit fix

# Update vulnerable packages
npm update --depth=5

# Check for breaking changes
npm audit --audit-level high
```

### Performance Optimization
Optimize dependency impact:

```bash
# Bundle splitting analysis
npx webpack-bundle-analyzer --mode static

# Tree shaking verification
npm run build -- --analyze

# Compression analysis
gzip -c build/static/js/main.js | wc -c
```

## Reporting

### Generate Reports
Create comprehensive analysis reports:

```bash
# Generate HTML report
npx bundle-analyzer build/static/js/*.js --html analysis/bundle-report.html

# Create summary report
cat > analysis/summary.md << EOF
# Dependency Analysis Summary
Date: $(date)

## Security Issues
$(cat analysis/security-audit.json | jq '.vulnerabilities | length') vulnerabilities found

## Unused Dependencies
$(cat analysis/unused-dependencies.txt | wc -l) potentially unused packages

## Bundle Size
$(du -sh build/ | cut -f1) total bundle size

## Recommendations
- Update $(npm outdated | wc -l) outdated packages
- Remove $(npx depcheck | grep -c "unused") unused dependencies
- Review $(npm audit | grep -c "vulnerability") security issues
EOF
```

### Visualization
Create visual representations:

```bash
# Dependency graph visualization
npx dependency-cruiser --include-only "^src" --output-type dot src/ | dot -T svg > analysis/dependency-graph.svg

# Bundle size visualization
npx webpack-bundle-analyzer build/static/js/*.js --mode static --report analysis/bundle-analysis.html
```

## Automation

### CI/CD Integration
Integrate analysis into CI pipeline:

```yaml
# GitHub Actions dependency analysis
name: Dependency Analysis
on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  pull_request:
    paths:
      - '**/Cargo.toml'
      - '**/package.json'

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Rust audit
        run: cargo audit
      - name: NPM audit
        run: npm audit
      - name: Bundle analysis
        run: npm run build:analyze
```

### Automated Fixes
Set up automated dependency management:

```bash
# Auto-update dependencies
npx npm-check-updates -u

# Auto-fix security issues
npm audit fix --force

# Auto-remove unused deps
npx depcheck | xargs npm uninstall
```

## Best Practices

1. **Regular Audits**: Perform dependency analysis regularly
2. **Security First**: Prioritize security vulnerability fixes
3. **Minimal Dependencies**: Use only necessary dependencies
4. **Version Pinning**: Pin dependency versions for reproducibility
5. **License Compliance**: Ensure all licenses are compatible
6. **Performance Monitoring**: Track dependency impact on performance
7. **Documentation**: Document dependency decisions and constraints

## Common Issues

- **Circular Dependencies**: Dependencies that create circular references
- **Version Conflicts**: Incompatible dependency version requirements
- **License Incompatibilities**: Dependencies with incompatible licenses
- **Bundle Bloat**: Dependencies that significantly increase bundle size
- **Security Vulnerabilities**: Known security issues in dependencies
- **Maintenance Issues**: Dependencies with poor maintenance status
- **Platform Compatibility**: Dependencies not compatible with target platforms