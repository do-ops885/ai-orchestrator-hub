# Dependency Analysis Summary
Date: 2025-09-20

## Security Issues
- **Rust Dependencies**: Cargo audit tool not installed in environment; unable to scan for vulnerabilities. Recommend installing `cargo-audit` for future analyses.
- **Node.js Dependencies**: 0 vulnerabilities found (info: 0, low: 0, moderate: 0, high: 0, critical: 0)
- **Total Dependencies**: 701 (prod: 30, dev: 636, optional: 108, peer: 9)

## Unused Dependencies
- **Unused Production Dependencies**: @ai-sdk/anthropic, @ai-sdk/openai, @vitest/coverage-v8, eslint-plugin-react-hooks
- **Unused Dev Dependencies**: @eslint/js, @testing-library/user-event, @vitest/coverage-v8, autoprefixer, eslint-plugin-prettier, eslint-plugin-react-hooks, postcss, tailwindcss
- **Recommendation**: Review and remove unused dependencies to reduce bundle size and maintenance overhead.

## Bundle Size
- **Frontend Bundle**: ~130 kB first load JS (Next.js optimized build)
- **Rust Binary**: Not analyzed (cargo-bloat not installed); recommend installing for binary size optimization insights.
- **Optimization Opportunity**: Unused dependencies contribute to bundle bloat; removing them could reduce size.

## Dependency Health
- **Rust Dependencies**: Tree generated; duplicates checked (see cargo-duplicates.txt)
- **Node.js Dependencies**: No duplicates found in shallow analysis
- **License Compliance**: Node.js licenses checked (see npm-licenses.json); Rust licenses require cargo-license tool.

## Recommendations
1. **Install Missing Tools**: cargo-audit, cargo-outdated, cargo-license, cargo-bloat for comprehensive Rust analysis
2. **Remove Unused Packages**: Eliminate 4 unused prod and 8 unused dev dependencies
3. **Security Monitoring**: Set up automated security scanning in CI/CD
4. **Bundle Optimization**: Use webpack-bundle-analyzer for detailed frontend bundle analysis
5. **License Audit**: Ensure all licenses are compatible and compliant

## Next Steps
- Automate this analysis in CI pipeline
- Set up alerts for new vulnerabilities
- Regular dependency updates and cleanup
- Monitor bundle size trends