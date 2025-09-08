#!/bin/bash

# AI Orchestrator Hub Service Installation Script
# This script installs the MCP server as a systemd service

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SERVICE_NAME="ai-orchestrator-hub"
SERVICE_FILE="$SCRIPT_DIR/${SERVICE_NAME}.service"
INSTALL_DIR="/opt/${SERVICE_NAME}"
USER_NAME="ai-orchestrator"
GROUP_NAME="ai-orchestrator"

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

# Function to check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        log "ERROR" "This script must be run as root (sudo)"
        exit 1
    fi
}

# Function to check prerequisites
check_prerequisites() {
    log "INFO" "Checking prerequisites..."

    # Check if systemd is available
    if ! command -v systemctl &> /dev/null; then
        log "ERROR" "systemd is required but not available"
        exit 1
    fi

    # Check if user exists, create if not
    if ! id "$USER_NAME" &>/dev/null; then
        log "INFO" "Creating user: $USER_NAME"
        useradd --system --shell /bin/false --home "$INSTALL_DIR" --create-home "$USER_NAME"
    fi

    # Check if group exists
    if ! getent group "$GROUP_NAME" &>/dev/null; then
        log "INFO" "Creating group: $GROUP_NAME"
        groupadd "$GROUP_NAME"
    fi

    # Add user to group if not already
    if ! groups "$USER_NAME" | grep -q "$GROUP_NAME"; then
        usermod -a -G "$GROUP_NAME" "$USER_NAME"
    fi

    log "SUCCESS" "Prerequisites check passed"
}

# Function to install the application
install_application() {
    log "INFO" "Installing application to $INSTALL_DIR..."

    # Create installation directory
    mkdir -p "$INSTALL_DIR"

    # Copy application files
    cp -r "$REPO_ROOT/backend" "$INSTALL_DIR/"
    cp -r "$REPO_ROOT/frontend" "$INSTALL_DIR/" 2>/dev/null || true
    cp -r "$REPO_ROOT/data" "$INSTALL_DIR/" 2>/dev/null || true

    # Create necessary directories
    mkdir -p "$INSTALL_DIR/logs"
    mkdir -p "$INSTALL_DIR/data"

    # Set ownership
    chown -R "$USER_NAME:$GROUP_NAME" "$INSTALL_DIR"

    # Build the application
    log "INFO" "Building application..."
    cd "$INSTALL_DIR/backend"
    sudo -u "$USER_NAME" cargo build --release

    log "SUCCESS" "Application installed successfully"
}

# Function to install systemd service
install_service() {
    log "INFO" "Installing systemd service..."

    # Copy service file
    cp "$SERVICE_FILE" "/etc/systemd/system/"

    # Reload systemd
    systemctl daemon-reload

    # Enable service
    systemctl enable "$SERVICE_NAME"

    log "SUCCESS" "Systemd service installed"
}

# Function to configure firewall (optional)
configure_firewall() {
    log "INFO" "Configuring firewall..."

    if command -v ufw &> /dev/null; then
        log "INFO" "Configuring UFW firewall..."
        ufw allow 3000/tcp comment "AI Orchestrator Hub MCP Server"
        log "SUCCESS" "Firewall configured (port 3000 opened)"
    elif command -v firewall-cmd &> /dev/null; then
        log "INFO" "Configuring firewalld..."
        firewall-cmd --permanent --add-port=3000/tcp
        firewall-cmd --reload
        log "SUCCESS" "Firewall configured (port 3000 opened)"
    else
        log "WARN" "No supported firewall detected. Please manually open port 3000"
    fi
}

# Function to start service
start_service() {
    log "INFO" "Starting service..."

    systemctl start "$SERVICE_NAME"

    # Wait for service to start
    sleep 5

    # Check status
    if systemctl is-active --quiet "$SERVICE_NAME"; then
        log "SUCCESS" "Service started successfully"
    else
        log "ERROR" "Service failed to start"
        log "INFO" "Check logs with: journalctl -u $SERVICE_NAME -f"
        exit 1
    fi
}

# Function to show status
show_status() {
    echo
    echo "=============================================="
    echo "  AI Orchestrator Hub Installation Complete"
    echo "=============================================="
    echo
    echo "Service Status:"
    systemctl status "$SERVICE_NAME" --no-pager -l
    echo
    echo "Service Details:"
    echo "  - Service name: $SERVICE_NAME"
    echo "  - Installation directory: $INSTALL_DIR"
    echo "  - User: $USER_NAME"
    echo "  - Port: 3000"
    echo
    echo "Endpoints:"
    echo "  - MCP HTTP: http://localhost:3000/api/mcp"
    echo "  - Health check: http://localhost:3000/api/mcp/health"
    echo "  - API docs: http://localhost:3000/api/hive/status"
    echo
    echo "Management Commands:"
    echo "  - Start: systemctl start $SERVICE_NAME"
    echo "  - Stop: systemctl stop $SERVICE_NAME"
    echo "  - Restart: systemctl restart $SERVICE_NAME"
    echo "  - Status: systemctl status $SERVICE_NAME"
    echo "  - Logs: journalctl -u $SERVICE_NAME -f"
    echo
    echo "Manual Control (alternative):"
    echo "  - $SCRIPT_DIR/run-mcp-service.sh start"
    echo "  - $SCRIPT_DIR/run-mcp-service.sh stop"
    echo "  - $SCRIPT_DIR/run-mcp-service.sh status"
    echo
    echo "=============================================="
}

# Function to uninstall service
uninstall_service() {
    log "INFO" "Uninstalling service..."

    # Stop service
    systemctl stop "$SERVICE_NAME" 2>/dev/null || true

    # Disable service
    systemctl disable "$SERVICE_NAME" 2>/dev/null || true

    # Remove service file
    rm -f "/etc/systemd/system/${SERVICE_NAME}.service"

    # Remove application directory
    rm -rf "$INSTALL_DIR"

    # Remove user (optional)
    if [[ "${1:-keep-user}" != "keep-user" ]]; then
        userdel "$USER_NAME" 2>/dev/null || true
        groupdel "$GROUP_NAME" 2>/dev/null || true
    fi

    # Reload systemd
    systemctl daemon-reload

    log "SUCCESS" "Service uninstalled"
}

# Function to show usage
show_usage() {
    echo "AI Orchestrator Hub Service Installer"
    echo
    echo "Usage: $0 <command>"
    echo
    echo "Commands:"
    echo "  install     - Install and start the service"
    echo "  uninstall   - Stop and uninstall the service"
    echo "  reinstall   - Reinstall the service"
    echo
    echo "Options for uninstall:"
    echo "  --keep-user - Keep the service user account"
    echo
    echo "Examples:"
    echo "  $0 install                    # Install and start"
    echo "  sudo $0 install               # Install as root"
    echo "  $0 uninstall                  # Remove everything"
    echo "  $0 uninstall --keep-user      # Remove but keep user"
    echo
    echo "Note: This script requires root privileges for installation"
}

# Main function
main() {
    local command="${1:-help}"

    case $command in
        "install")
            check_root
            check_prerequisites
            install_application
            install_service
            configure_firewall
            start_service
            show_status
            ;;
        "uninstall")
            check_root
            local keep_user=""
            if [[ "${2:-}" == "--keep-user" ]]; then
                keep_user="keep-user"
            fi
            uninstall_service "$keep_user"
            ;;
        "reinstall")
            check_root
            uninstall_service
            check_prerequisites
            install_application
            install_service
            start_service
            show_status
            ;;
        "help"|*)
            show_usage
            ;;
    esac
}

# Run main function with all arguments
main "$@"
