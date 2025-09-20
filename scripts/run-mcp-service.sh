#!/bin/bash

# AI Orchestrator Hub MCP Service Runner
# This script runs the standalone MCP server in HTTP mode as a persistent background service

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKEND_DIR="$REPO_ROOT/backend"
SERVICE_NAME="mcp-server"
LOG_DIR="$REPO_ROOT/logs"
PID_FILE="$REPO_ROOT/${SERVICE_NAME}.pid"

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

    # Check if we're in the right directory
    if [[ ! -f "$BACKEND_DIR/Cargo.toml" ]]; then
        log "ERROR" "Backend directory not found. Please run from the ai-orchestrator-hub directory."
        exit 1
    fi

    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        log "ERROR" "Cargo (Rust) is required but not installed"
        exit 1
    fi

    # Check if port 3002 is available (MCP server default)
    if lsof -Pi :3002 -sTCP:LISTEN -t >/dev/null 2>&1; then
        log "WARN" "Port 3002 is already in use. The MCP service may fail to start."
    fi

    log "SUCCESS" "Prerequisites check passed"
}

# Function to build the project
build_project() {
    log "INFO" "Building the project..."

    cd "$BACKEND_DIR"

    # Build in release mode for production
    if [[ "${1:-release}" == "release" ]]; then
        log "INFO" "Building in release mode..."
        cargo build --release
    else
        log "INFO" "Building in debug mode..."
        cargo build
    fi

    log "SUCCESS" "Project built successfully"
}

# Function to start the service
start_service() {
    log "INFO" "Starting AI Orchestrator Hub MCP service..."

    # Create log directory if it doesn't exist
    mkdir -p "$LOG_DIR"

    # Check if service is already running
    if [[ -f "$PID_FILE" ]]; then
        if kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
            log "WARN" "Service is already running (PID: $(cat "$PID_FILE"))"
            return 0
        else
            log "WARN" "Removing stale PID file"
            rm -f "$PID_FILE"
        fi
    fi

    cd "$BACKEND_DIR"

    # Start the service in background
    local build_mode="${1:-release}"
    local log_file="$LOG_DIR/${SERVICE_NAME}.log"

    if [[ "$build_mode" == "release" ]]; then
        nohup ./target/release/mcp_server --http > "$log_file" 2>&1 &
    else
        nohup ./target/debug/mcp_server --http > "$log_file" 2>&1 &
    fi

    local pid=$!
    echo $pid > "$PID_FILE"

    # Wait a moment for the service to start
    sleep 3

    # Check if the service is still running
    if kill -0 $pid 2>/dev/null; then
        log "SUCCESS" "MCP service started successfully (PID: $pid)"
        log "INFO" "Logs available at: $log_file"
        log "INFO" "MCP HTTP endpoint: http://localhost:3002/"
        log "INFO" "MCP Health check: http://localhost:3002/health"
    else
        log "ERROR" "Service failed to start. Check logs at: $log_file"
        rm -f "$PID_FILE"
        exit 1
    fi
}

# Function to stop the service
stop_service() {
    log "INFO" "Stopping AI Orchestrator Hub MCP service..."

    if [[ ! -f "$PID_FILE" ]]; then
        log "WARN" "PID file not found. Service may not be running."
        return 0
    fi

    local pid=$(cat "$PID_FILE")

    if kill -0 $pid 2>/dev/null; then
        log "INFO" "Sending SIGTERM to process $pid..."
        kill $pid

        # Wait for graceful shutdown
        local count=0
        while kill -0 $pid 2>/dev/null && [[ $count -lt 10 ]]; do
            sleep 1
            ((count++))
        done

        if kill -0 $pid 2>/dev/null; then
            log "WARN" "Process didn't stop gracefully, sending SIGKILL..."
            kill -9 $pid
        fi
    else
        log "WARN" "Process $pid is not running"
    fi

    rm -f "$PID_FILE"
    log "SUCCESS" "Service stopped"
}

# Function to check service status
check_status() {
    if [[ -f "$PID_FILE" ]]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 $pid 2>/dev/null; then
            log "SUCCESS" "MCP service is running (PID: $pid)"
            log "INFO" "MCP HTTP endpoint: http://localhost:3002/"
            return 0
        else
            log "ERROR" "Service is not running (stale PID file)"
            rm -f "$PID_FILE"
            return 1
        fi
    else
        log "INFO" "Service is not running"
        return 1
    fi
}

# Function to show service logs
show_logs() {
    local log_file="$LOG_DIR/${SERVICE_NAME}.log"

    if [[ -f "$log_file" ]]; then
        log "INFO" "Showing recent logs (last 50 lines):"
        echo "----------------------------------------"
        tail -50 "$log_file"
        echo "----------------------------------------"
        log "INFO" "Full logs available at: $log_file"
    else
        log "WARN" "Log file not found: $log_file"
    fi
}

# Function to restart the service
restart_service() {
    log "INFO" "Restarting service..."
    stop_service
    sleep 2
    start_service "$@"
}

# Function to show usage
show_usage() {
    echo "AI Orchestrator Hub Standalone MCP Service Manager"
    echo
    echo "Usage: $0 <command> [options]"
    echo
    echo "Commands:"
    echo "  start [debug|release]    - Start the service (default: release)"
    echo "  stop                     - Stop the service"
    echo "  restart [debug|release]  - Restart the service"
    echo "  status                   - Check service status"
    echo "  logs                     - Show service logs"
    echo "  build [debug|release]    - Build the project only"
    echo
    echo "Examples:"
    echo "  $0 start                 # Start in release mode"
    echo "  $0 start debug           # Start in debug mode"
    echo "  $0 stop                  # Stop the service"
    echo "  $0 restart               # Restart in release mode"
    echo "  $0 status                # Check if running"
    echo "  $0 logs                  # Show recent logs"
    echo
    echo "Service Details:"
    echo "  - PID file: $PID_FILE"
    echo "  - Log file: $LOG_DIR/${SERVICE_NAME}.log"
    echo "  - Default port: 3002"
    echo "  - MCP endpoint: http://localhost:3002/"
    echo "  - Mode: HTTP (supports MCP protocol over HTTP)"
}

# Main function
main() {
    local command="${1:-help}"

    case $command in
        "start")
            check_prerequisites
            build_project "${2:-release}"
            start_service "${2:-release}"
            ;;
        "stop")
            stop_service
            ;;
        "restart")
            check_prerequisites
            build_project "${2:-release}"
            restart_service "${2:-release}"
            ;;
        "status")
            check_status
            ;;
        "logs")
            show_logs
            ;;
        "build")
            check_prerequisites
            build_project "${2:-release}"
            ;;
        "help"|*)
            show_usage
            ;;
    esac
}

# Run main function with all arguments
main "$@"
