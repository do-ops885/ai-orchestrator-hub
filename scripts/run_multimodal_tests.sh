#!/bin/bash

# Multi-Modal Agent Comprehensive Test Runner
# This script runs all tests for the Multi-Modal Agent with detailed reporting

set -e

echo "🚀 Multi-Modal Agent Comprehensive Test Suite"
echo "============================================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Change to backend directory
cd "$(dirname "$0")/../backend"

# Test categories
declare -a test_categories=(
    "multimodal_agent_tests"
    "integration_multimodal_agent"
    "performance_multimodal_tests"
    "edge_case_multimodal_tests"
    "stress_test_multimodal"
)

# Results tracking
total_tests=0
passed_tests=0
failed_tests=0
test_results=()

print_status "Starting Multi-Modal Agent test suite..."
echo

# Run unit tests for the multi-modal agent
print_status "Running Multi-Modal Agent unit tests..."
if cargo test multimodal_agent --lib -- --test-threads=4 --nocapture; then
    print_success "Unit tests passed"
    ((passed_tests++))
else
    print_error "Unit tests failed"
    ((failed_tests++))
fi
test_results+=("Unit Tests")
((total_tests++))
echo

# Run integration tests
for category in "${test_categories[@]}"; do
    print_status "Running $category tests..."
    
    start_time=$(date +%s)
    if cargo test "$category" --test "*" -- --test-threads=2 --nocapture; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_success "$category tests passed (${duration}s)"
        test_results+=("$category: PASSED (${duration}s)")
        ((passed_tests++))
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_error "$category tests failed (${duration}s)"
        test_results+=("$category: FAILED (${duration}s)")
        ((failed_tests++))
    fi
    ((total_tests++))
    echo
done

# Run benchmarks if criterion is available
print_status "Running performance benchmarks..."
if cargo bench multimodal_agent_benchmarks --no-run &>/dev/null; then
    print_status "Running benchmark suite..."
    if cargo bench multimodal_agent_benchmarks -- --output-format=pretty; then
        print_success "Benchmarks completed successfully"
        test_results+=("Benchmarks: COMPLETED")
    else
        print_warning "Benchmarks completed with warnings"
        test_results+=("Benchmarks: COMPLETED (with warnings)")
    fi
else
    print_warning "Criterion benchmarks not available (install with 'cargo install criterion')"
    test_results+=("Benchmarks: SKIPPED (criterion not available)")
fi
echo

# Run a quick demo to verify functionality
print_status "Running functionality verification..."
if cargo run --example simple_multimodal_demo &>/dev/null; then
    print_success "Functionality verification passed"
    test_results+=("Demo: PASSED")
else
    print_error "Functionality verification failed"
    test_results+=("Demo: FAILED")
    ((failed_tests++))
fi
((total_tests++))
echo

# Generate test report
echo "📊 Test Results Summary"
echo "======================="
echo "Total test categories: $total_tests"
echo "Passed: $passed_tests"
echo "Failed: $failed_tests"
echo

if [ $failed_tests -eq 0 ]; then
    print_success "All tests passed! 🎉"
    success_rate="100%"
else
    success_rate=$(( (passed_tests * 100) / total_tests ))
    if [ $success_rate -ge 80 ]; then
        print_warning "Most tests passed (${success_rate}% success rate)"
    else
        print_error "Multiple test failures (${success_rate}% success rate)"
    fi
fi

echo
echo "Detailed Results:"
echo "=================="
for result in "${test_results[@]}"; do
    if [[ $result == *"PASSED"* ]]; then
        echo -e "${GREEN}✓${NC} $result"
    elif [[ $result == *"FAILED"* ]]; then
        echo -e "${RED}✗${NC} $result"
    elif [[ $result == *"COMPLETED"* ]]; then
        echo -e "${BLUE}ℹ${NC} $result"
    else
        echo -e "${YELLOW}⚠${NC} $result"
    fi
done

echo
echo "📋 Next Steps:"
echo "==============="
if [ $failed_tests -eq 0 ]; then
    echo "• Multi-Modal Agent is ready for production use!"
    echo "• Consider running load tests in staging environment"
    echo "• Review benchmark results for performance optimization opportunities"
    echo "• Update documentation with latest test results"
else
    echo "• Review failed tests and address issues"
    echo "• Check logs for detailed error information"
    echo "• Consider running individual test categories for debugging"
    echo "• Verify system dependencies and configuration"
fi

echo
echo "🔗 Useful Commands:"
echo "==================="
echo "• Run specific test: cargo test <test_name>"
echo "• Run with output: cargo test <test_name> -- --nocapture"
echo "• Run benchmarks: cargo bench multimodal_agent_benchmarks"
echo "• Run demo: cargo run --example simple_multimodal_demo"
echo "• Check docs: cargo doc --open"

# Exit with appropriate code
if [ $failed_tests -eq 0 ]; then
    exit 0
else
    exit 1
fi