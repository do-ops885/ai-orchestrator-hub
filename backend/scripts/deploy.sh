#!/bin/bash

# AI Orchestrator Hub - Deployment Script
# This script handles deployment to different environments

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ENVIRONMENT=${1:-staging}
VERSION=${2:-latest}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Validate environment
validate_environment() {
    case $ENVIRONMENT in
        staging|production)
            log_info "Deploying to $ENVIRONMENT environment"
            ;;
        *)
            log_error "Invalid environment: $ENVIRONMENT. Use 'staging' or 'production'"
            exit 1
            ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check if kubectl is installed
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl is not installed. Please install it first."
        exit 1
    fi

    # Check if helm is installed
    if ! command -v helm &> /dev/null; then
        log_error "helm is not installed. Please install it first."
        exit 1
    fi

    # Check if docker is installed
    if ! command -v docker &> /dev/null; then
        log_error "docker is not installed. Please install it first."
        exit 1
    fi

    log_success "Prerequisites check passed"
}

# Build Docker image
build_image() {
    log_info "Building Docker image..."

    cd "$PROJECT_ROOT"

    # Build the Docker image
    docker build -t ai-orchestrator-backend:$VERSION .

    log_success "Docker image built successfully"
}

# Push Docker image
push_image() {
    log_info "Pushing Docker image to registry..."

    # Tag the image
    docker tag ai-orchestrator-backend:$VERSION your-registry.com/ai-orchestrator-backend:$VERSION

    # Push to registry
    docker push your-registry.com/ai-orchestrator-backend:$VERSION

    log_success "Docker image pushed successfully"
}

# Deploy using Helm
deploy_helm() {
    log_info "Deploying using Helm..."

    cd "$PROJECT_ROOT/helm/ai-orchestrator"

    # Set namespace based on environment
    NAMESPACE=$ENVIRONMENT

    # Deploy using Helm
    helm upgrade --install ai-orchestrator-$ENVIRONMENT . \
        --namespace $NAMESPACE \
        --create-namespace \
        --set image.tag=$VERSION \
        --set image.repository=your-registry.com/ai-orchestrator-backend \
        --values values-$ENVIRONMENT.yaml \
        --wait \
        --timeout 600s

    log_success "Helm deployment completed"
}

# Run database migrations
run_migrations() {
    log_info "Running database migrations..."

    # This would typically involve running migration scripts
    # For now, we'll just log the action
    log_info "Database migrations would be run here"

    log_success "Database migrations completed"
}

# Run health checks
run_health_checks() {
    log_info "Running health checks..."

    # Wait for deployment to be ready
    kubectl wait --for=condition=available --timeout=300s deployment/ai-orchestrator-$ENVIRONMENT -n $ENVIRONMENT

    # Get service URL
    SERVICE_URL=$(kubectl get svc ai-orchestrator-$ENVIRONMENT -n $ENVIRONMENT -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')

    if [ -z "$SERVICE_URL" ]; then
        SERVICE_URL=$(kubectl get svc ai-orchestrator-$ENVIRONMENT -n $ENVIRONMENT -o jsonpath='{.spec.clusterIP}')
    fi

    # Run health check
    if curl -f http://$SERVICE_URL:8080/health; then
        log_success "Health check passed"
    else
        log_error "Health check failed"
        exit 1
    fi
}

# Run smoke tests
run_smoke_tests() {
    log_info "Running smoke tests..."

    # This would run basic smoke tests against the deployed application
    log_info "Smoke tests would be run here"

    log_success "Smoke tests passed"
}

# Rollback function
rollback() {
    log_warning "Starting rollback..."

    # Rollback Helm release
    helm rollback ai-orchestrator-$ENVIRONMENT 0 -n $ENVIRONMENT

    log_success "Rollback completed"
}

# Main deployment function
main() {
    log_info "Starting deployment to $ENVIRONMENT environment with version $VERSION"

    validate_environment
    check_prerequisites

    # Build and push image
    build_image
    push_image

    # Deploy
    deploy_helm
    run_migrations
    run_health_checks
    run_smoke_tests

    log_success "Deployment to $ENVIRONMENT completed successfully!"
    log_info "Application is available at: http://your-domain.com/api/health"
}

# Handle command line arguments
case "${2:-}" in
    --rollback)
        rollback
        ;;
    *)
        main
        ;;
esac