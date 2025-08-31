---
description: Set up complete development environment for AI Orchestrator Hub
agent: rust-developer
---

# Setup Development Environment

Set up a complete development environment for the AI Orchestrator Hub project, including all required tools, dependencies, and configurations.

## System Requirements

### Hardware Requirements
- **CPU**: 4+ cores recommended
- **RAM**: 8GB minimum, 16GB recommended
- **Storage**: 20GB free space
- **Network**: Stable internet connection

### Software Prerequisites
- **Operating System**: Linux, macOS, or Windows (WSL2)
- **Package Manager**: apt, brew, or chocolatey

## Development Tools Setup

### 1. Rust Toolchain
Install and configure Rust:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install additional components
rustup component add rustfmt
rustup component add clippy
rustup component add rust-src

# Verify installation
rustc --version
cargo --version
```

### 2. Node.js Environment
Install Node.js and npm:

```bash
# Install Node.js (using nvm recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# Install latest LTS
nvm install --lts
nvm use --lts

# Install yarn
npm install -g yarn

# Verify installation
node --version
npm --version
yarn --version
```

### 3. System Dependencies
Install system-level dependencies:

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev cmake protobuf-compiler

# macOS
brew install cmake protobuf

# Windows (using chocolatey)
choco install cmake protoc
```

### 4. Development Tools
Install additional development tools:

```bash
# Install development tools
cargo install cargo-watch
cargo install cargo-edit
cargo install cargo-audit
cargo install cargo-tarpaulin
cargo install flamegraph

# Install Node.js development tools
npm install -g typescript
npm install -g eslint
npm install -g prettier
```

## Project Setup

### 1. Clone Repository
Clone the project repository:

```bash
# Clone the repository
git clone https://github.com/your-org/ai-orchestrator-hub.git
cd ai-orchestrator-hub

# Initialize submodules if any
git submodule update --init --recursive
```

### 2. Backend Setup
Set up the Rust backend:

```bash
# Navigate to backend
cd backend

# Install dependencies
cargo fetch

# Check code
cargo check

# Run initial build
cargo build

# Verify tests pass
cargo test
```

### 3. Frontend Setup
Set up the Next.js frontend:

```bash
# Navigate to frontend
cd ../frontend

# Install dependencies
npm ci

# Verify setup
npm run build
```

### 4. Database Setup
Set up development database:

```bash
# Install PostgreSQL (if using)
# Ubuntu
sudo apt install postgresql postgresql-contrib

# macOS
brew install postgresql

# Start PostgreSQL service
sudo systemctl start postgresql  # Linux
brew services start postgresql   # macOS

# Create development database
createdb ai_orchestrator_dev
```

## Configuration

### 1. Environment Variables
Set up environment configuration:

```bash
# Create .env files
cp backend/.env.example backend/.env
cp frontend/.env.example frontend/.env

# Edit configuration files
# backend/.env
DATABASE_URL=postgresql://localhost/ai_orchestrator_dev
RUST_LOG=debug
PORT=8000

# frontend/.env.local
NEXT_PUBLIC_API_URL=http://localhost:8000
NEXT_PUBLIC_WS_URL=ws://localhost:8000
```

### 2. IDE Configuration
Configure development environment:

```bash
# Install VS Code extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension ms-vscode.vscode-typescript-next
code --install-extension esbenp.prettier-vscode
code --install-extension ms-vscode.vscode-eslint

# Configure Rust analyzer
# Add to .vscode/settings.json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all"
}
```

## Development Workflow

### 1. Running the Application
Start development servers:

```bash
# Terminal 1: Backend
cd backend
cargo watch -x run

# Terminal 2: Frontend
cd frontend
npm run dev

# Terminal 3: Database (if needed)
# Start database service
```

### 2. Development Commands
Useful development commands:

```bash
# Backend development
cargo watch -x check          # Continuous checking
cargo watch -x test           # Continuous testing
cargo watch -x 'run -- --dev' # Development mode

# Frontend development
npm run dev                   # Development server
npm run type-check           # Type checking
npm run lint:watch           # Continuous linting
```

### 3. Testing Setup
Configure testing environment:

```bash
# Backend testing
cargo install cargo-nextest   # Faster test runner
cargo nextest run

# Frontend testing
npm run test:watch           # Watch mode testing
```

## Docker Development

### 1. Docker Setup
Set up Docker for development:

```bash
# Install Docker
# Ubuntu
sudo apt install docker.io docker-compose

# macOS
brew install --cask docker

# Start Docker service
sudo systemctl start docker  # Linux
# macOS: Start Docker Desktop
```

### 2. Development Containers
Use development containers:

```bash
# Build development container
docker build -f Dockerfile.dev -t ai-orchestrator-dev .

# Run development container
docker run -it -v $(pwd):/app -p 8000:8000 -p 3000:3000 ai-orchestrator-dev
```

## Troubleshooting

### Common Setup Issues
1. **Rust Installation Issues**: Ensure PATH includes Cargo bin directory
2. **Node.js Version Conflicts**: Use nvm to manage Node.js versions
3. **Permission Issues**: Use sudo for system package installation
4. **Port Conflicts**: Check if ports 8000 and 3000 are available
5. **Dependency Issues**: Clear caches and reinstall dependencies

### Verification Steps
Verify setup completeness:

```bash
# Check Rust setup
rustc --version && cargo --version && rustup show

# Check Node.js setup
node --version && npm --version && yarn --version

# Check project setup
cd backend && cargo check
cd ../frontend && npm run build

# Check database connection
psql -d ai_orchestrator_dev -c "SELECT version();"
```

## Additional Tools

### 1. Performance Monitoring
Install performance monitoring tools:

```bash
# System monitoring
sudo apt install htop iotop

# Rust profiling
cargo install cargo-flamegraph
cargo install heaptrack

# Node.js profiling
npm install -g clinic
```

### 2. Code Quality Tools
Install code quality tools:

```bash
# Rust
cargo install cargo-audit    # Security auditing
cargo install cargo-outdated # Dependency updates

# Node.js
npm install -g sonarjs      # Code quality
npm install -g depcheck     # Dependency checking
```

### 3. Documentation Tools
Install documentation tools:

```bash
# Rust documentation
cargo install cargo-doc

# API documentation
npm install -g redoc-cli
npm install -g swagger-cli
```

## Next Steps

After setup completion:

1. **Read Documentation**: Review README.md and CONTRIBUTING.md
2. **Run Tests**: Execute the full test suite
3. **Explore Codebase**: Familiarize yourself with the project structure
4. **Make Changes**: Start with small changes and create pull requests
5. **Join Community**: Participate in discussions and ask questions

## Getting Help

If you encounter issues:

1. Check the troubleshooting section above
2. Review project documentation
3. Search existing issues on GitHub
4. Ask questions in project discussions
5. Contact maintainers for assistance