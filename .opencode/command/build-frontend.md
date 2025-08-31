---
description: Build the Next.js frontend with dashboard and visualization components
agent: react-developer
---

# Build Frontend Command

Build the complete Next.js frontend for the AI Orchestrator Hub, including all dashboard components, visualizations, and user interfaces.

## Build Process

### 1. Environment Setup
Verify Node.js environment and dependencies:

```bash
# Check Node.js version
node --version
npm --version

# Install dependencies
npm ci

# Verify TypeScript
npx tsc --version
```

### 2. Code Quality Checks
Run linting and type checking:

```bash
# Run ESLint
npm run lint

# Type checking
npm run type-check

# Format code
npm run format:check
```

### 3. Build Process
Execute the Next.js build:

```bash
# Clean previous build
rm -rf .next out

# Build application
npm run build

# Export static files (if needed)
npm run export
```

### 4. Optimization Steps
Apply performance optimizations:

```bash
# Analyze bundle size
npm run analyze

# Generate service worker
npm run build:sw

# Optimize images
npm run build:images
```

### 5. Testing
Run frontend tests:

```bash
# Unit tests
npm run test:unit

# Integration tests
npm run test:integration

# E2E tests
npm run test:e2e
```

## Build Verification

### 1. Build Artifacts
Verify generated build artifacts:

```bash
# Check build output
ls -la .next/

# Verify static assets
ls -la .next/static/

# Check build manifest
cat .next/build-manifest.json
```

### 2. Performance Metrics
Analyze build performance:

```bash
# Bundle size analysis
npx webpack-bundle-analyzer .next/static/chunks/*.js

# Lighthouse performance
npx lighthouse http://localhost:3000 --output=json --output-path=./lighthouse-report.json
```

### 3. Static Analysis
Run security and quality checks:

```bash
# Security audit
npm audit

# Bundle analysis
npm run analyze:bundle

# Accessibility check
npm run test:a11y
```

## Build Configuration

### Next.js Configuration
Ensure proper `next.config.js`:

```javascript
/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  experimental: {
    optimizeCss: true,
    scrollRestoration: true,
  },
  images: {
    formats: ['image/webp', 'image/avif'],
  },
  webpack: (config, { isServer }) => {
    if (!isServer) {
      config.resolve.fallback.fs = false;
    }
    return config;
  },
};

module.exports = nextConfig;
```

### Environment Variables
Set required environment variables:

```bash
# API endpoints
NEXT_PUBLIC_API_URL=http://localhost:8000
NEXT_PUBLIC_WS_URL=ws://localhost:8000

# Feature flags
NEXT_PUBLIC_ENABLE_ANALYTICS=true
NEXT_PUBLIC_ENABLE_PWA=true

# Build settings
NODE_ENV=production
```

## Optimization Features

### Bundle Optimization
- **Code Splitting**: Automatic route-based splitting
- **Tree Shaking**: Remove unused code
- **Minification**: JavaScript and CSS minification
- **Compression**: Gzip and Brotli compression

### Performance Features
- **Image Optimization**: Next.js automatic image optimization
- **Font Optimization**: Automatic font loading optimization
- **CSS Optimization**: Critical CSS extraction
- **Caching**: Aggressive caching strategies

### PWA Features
- **Service Worker**: Offline functionality
- **App Manifest**: Installable PWA
- **Caching**: Runtime caching strategies
- **Background Sync**: Offline data synchronization

## Deployment Preparation

### Static Export
Prepare for static hosting:

```bash
# Export static files
npm run export

# Verify export
ls -la out/

# Test static serving
npx serve out/
```

### Docker Integration
Build Docker image:

```bash
# Build Docker image
docker build -t ai-orchestrator-hub-frontend .

# Run container
docker run -p 3000:3000 ai-orchestrator-hub-frontend
```

## Error Handling

### Common Build Issues
1. **Type Errors**: Check TypeScript configuration and type definitions
2. **Module Resolution**: Verify import paths and module installation
3. **Build Failures**: Check build logs for specific error details
4. **Performance Issues**: Analyze bundle size and optimize imports
5. **Environment Issues**: Verify Node.js version and environment variables

### Troubleshooting Steps
1. Clear cache: `rm -rf .next node_modules/.cache`
2. Reinstall dependencies: `rm -rf node_modules && npm ci`
3. Check TypeScript: `npx tsc --noEmit`
4. Verify configuration: Check `next.config.js` and `tsconfig.json`
5. Review build logs: Check `.next/build.log`

## Continuous Integration

### CI/CD Integration
- **GitHub Actions**: Automated build and deployment
- **Vercel**: Optimized Next.js deployment
- **Netlify**: Static site deployment
- **Docker**: Containerized deployment
- **CDN**: Global content distribution

### Build Metrics
Track important metrics:

- Build time
- Bundle size
- First Contentful Paint
- Largest Contentful Paint
- Cumulative Layout Shift
- Test coverage