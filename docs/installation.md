# Installation Guide

This guide provides detailed installation instructions for the Multiagent Hive System.

## System Requirements

### Minimum Requirements
- **CPU**: 2 cores, 2.5 GHz
- **RAM**: 2 GB
- **Storage**: 1 GB free space
- **Network**: 10 Mbps connection
- **OS**: Linux, macOS 10.15+, Windows 10+ (WSL recommended)

### Recommended Requirements
- **CPU**: 4+ cores, 3.0 GHz
- **RAM**: 4 GB
- **Storage**: 2 GB free space
- **Network**: 100 Mbps connection
- **OS**: Linux (Ubuntu 20.04+), macOS 12+

### Optimal Requirements
- **CPU**: 8+ cores with SIMD support
- **RAM**: 8 GB+
- **Storage**: 5 GB SSD
- **Network**: 1 Gbps connection
- **GPU**: NVIDIA GPU with CUDA support (optional)

## Prerequisites

### Rust Installation

The backend requires Rust 1.70 or later.

#### Linux/macOS
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Windows
Download and run the installer from [rustup.rs](https://rustup.rs/)

#### Verify Installation
```bash
rustc --version  # Should show 1.70+
cargo --version  # Should show 1.70+
```

### Node.js Installation

The frontend requires Node.js 18 or later.

#### Using Node Version Manager (Recommended)
```bash
# Install nvm (Linux/macOS)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc

# Install Node.js
nvm install 20
nvm use 20
nvm alias default 20
```

#### Direct Download
Download from [nodejs.org](https://nodejs.org/) (LTS version)

#### Verify Installation
```bash
node --version   # Should show 18+
npm --version    # Should show 8+
```

### Git Installation

```bash
# Linux (Ubuntu/Debian)
sudo apt update
sudo apt install git

# macOS
brew install git

# Windows
# Download from https://git-scm.com/downloads
```

## Source Installation

### 1. Clone Repository

```bash
git clone https://github.com/your-org/multiagent-hive.git
cd multiagent-hive
```

### 2. Backend Installation

```bash
cd backend

# Install dependencies
cargo build

# Optional: Build with advanced features
cargo build --features advanced-neural

# Optional: Build optimized release
cargo build --release
```

#### Build Features

- `default`: Basic NLP processing (recommended)
- `advanced-neural`: FANN neural networks
- `gpu-acceleration`: GPU support (requires CUDA)

### 3. Frontend Installation

```bash
cd frontend

# Install dependencies
npm install

# Optional: Install with exact versions
npm ci
```

## Docker Installation

### Using Docker Compose (Recommended)

```yaml
# docker-compose.yml
version: '3.8'

services:
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    ports:
      - "3001:3001"
    environment:
      - HIVE_PORT=3001
      - LOG_LEVEL=info
    volumes:
      - ./backend/data:/app/data

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      - NEXT_PUBLIC_API_URL=http://backend:3001
```

```bash
# Build and run
docker-compose up --build
```

### Manual Docker Build

#### Backend Dockerfile

```dockerfile
FROM rust:1.70-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/multiagent-hive /usr/local/bin/

EXPOSE 3001
CMD ["multiagent-hive"]
```

#### Frontend Dockerfile

```dockerfile
FROM node:20-alpine as builder

WORKDIR /app
COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM node:20-alpine as runner

WORKDIR /app
COPY --from=builder /app/package*.json ./
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/public ./public

EXPOSE 3000
CMD ["npm", "start"]
```

## Package Manager Installation

### Cargo (Rust Package Manager)

```bash
# Install from crates.io (when published)
cargo install multiagent-hive

# Run
multiagent-hive
```

### npm (Node.js Package Manager)

```bash
# Install globally (when published)
npm install -g @multiagent-hive/cli

# Run
multiagent-hive
```

## Platform-Specific Instructions

### Linux (Ubuntu/Debian)

```bash
# Update system
sudo apt update && sudo apt upgrade

# Install build dependencies
sudo apt install build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Clone and build
git clone https://github.com/your-org/multiagent-hive.git
cd multiagent-hive

# Backend
cd backend && cargo build --release

# Frontend
cd ../frontend && npm install && npm run build
```

### macOS

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install rust node git

# Clone and build
git clone https://github.com/your-org/multiagent-hive.git
cd multiagent-hive

# Backend
cd backend && cargo build --release

# Frontend
cd ../frontend && npm install && npm run build
```

### Windows

```powershell
# Install Chocolatey
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))

# Install dependencies
choco install rust node git

# Clone and build
git clone https://github.com/your-org/multiagent-hive.git
cd multiagent-hive

# Backend
cd backend
cargo build --release

# Frontend
cd ../frontend
npm install
npm run build
```

## Post-Installation Configuration

### Environment Variables

Create `.env` files in respective directories:

#### Backend (.env)
```env
HIVE_PORT=3001
HIVE_HOST=0.0.0.0
LOG_LEVEL=info
NEURAL_MODE=basic
MAX_AGENTS=1000
DATABASE_URL=hive.db
```

#### Frontend (.env.local)
```env
NEXT_PUBLIC_API_URL=http://localhost:3001
NEXT_PUBLIC_WS_URL=ws://localhost:3001/ws
```

### Directory Structure

After installation, your directory should look like:

```
multiagent-hive/
├── backend/
│   ├── target/release/multiagent-hive  # Compiled binary
│   ├── Cargo.toml
│   └── src/
├── frontend/
│   ├── .next/                         # Built frontend
│   ├── package.json
│   └── src/
├── docs/
├── docker-compose.yml
└── README.md
```

## Verification

### Backend Verification

```bash
cd backend

# Check version
./target/release/multiagent-hive --version

# Test basic functionality
cargo test

# Start server
./target/release/multiagent-hive
```

### Frontend Verification

```bash
cd frontend

# Check build
npm run build

# Start development server
npm run dev
```

### Full System Test

```bash
# Terminal 1: Backend
cd backend && cargo run

# Terminal 2: Frontend
cd frontend && npm run dev

# Test API
curl http://localhost:3001/api/hive/status
```

## Troubleshooting Installation

### Common Issues

#### Rust Compilation Errors
```bash
# Update Rust
rustup update

# Clean and rebuild
cd backend
cargo clean
cargo build
```

#### Node.js Permission Errors
```bash
# Fix npm permissions
sudo chown -R $(whoami) ~/.npm
```

#### Port Conflicts
```bash
# Check port usage
lsof -i :3001
lsof -i :3000

# Change ports in configuration
# Backend: HIVE_PORT=3002
# Frontend: Update .env.local
```

#### Memory Issues
```bash
# Check available memory
free -h  # Linux
vm_stat   # macOS

# Reduce agent count in config
MAX_AGENTS=100
```

## Next Steps

After successful installation:

1. **Read the Getting Started Guide**: [docs/getting-started.md](getting-started.md)
2. **Configure the System**: [docs/configuration.md](configuration.md)
3. **Explore Examples**: Check the `backend/examples/` directory
4. **Join the Community**: See [SUPPORT.md](../SUPPORT.md) for help resources

## Uninstalling

### Source Installation
```bash
# Remove directory
rm -rf multiagent-hive

# Remove Rust (optional)
rustup self uninstall

# Remove Node.js (optional)
# Follow platform-specific removal instructions
```

### Docker Installation
```bash
# Stop containers
docker-compose down

# Remove images
docker-compose down --rmi all

# Remove volumes
docker volume prune
```