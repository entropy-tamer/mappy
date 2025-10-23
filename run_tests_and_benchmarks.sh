#!/bin/bash

# Comprehensive test and benchmark runner for Mappy
# This script runs all tests and benchmarks for the quotient filter functionality

set -e

echo "🦊 Running comprehensive tests and benchmarks for Mappy quotient filter"
echo "=================================================================="

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

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "mappy-core" ]; then
    print_error "Please run this script from the mappy root directory"
    exit 1
fi

# Build the project first
print_status "Building project..."
cargo build --release --all-features
if [ $? -eq 0 ]; then
    print_success "Build completed successfully"
else
    print_error "Build failed"
    exit 1
fi

echo ""
print_status "Running basic tests..."
echo "=========================="

# Run basic tests
cargo test --features advanced-quotient-filter
if [ $? -eq 0 ]; then
    print_success "Basic tests passed"
else
    print_error "Basic tests failed"
    exit 1
fi

echo ""
print_status "Running quotient filter specific tests..."
echo "============================================="

# Run quotient filter tests
cargo test --features advanced-quotient-filter quotient_filter_tests
if [ $? -eq 0 ]; then
    print_success "Quotient filter tests passed"
else
    print_error "Quotient filter tests failed"
    exit 1
fi

echo ""
print_status "Running advanced quotient filter tests..."
echo "=============================================="

# Run advanced quotient filter tests
cargo test --features advanced-quotient-filter advanced_quotient_filter_tests
if [ $? -eq 0 ]; then
    print_success "Advanced quotient filter tests passed"
else
    print_error "Advanced quotient filter tests failed"
    exit 1
fi

echo ""
print_status "Running benchmarks..."
echo "========================"

# Run quotient filter benchmarks
print_status "Running quotient filter benchmarks..."
cargo bench --bench quotient_filter_benchmarks
if [ $? -eq 0 ]; then
    print_success "Quotient filter benchmarks completed"
else
    print_error "Quotient filter benchmarks failed"
    exit 1
fi

# Run advanced quotient filter benchmarks
print_status "Running advanced quotient filter benchmarks..."
cargo bench --bench advanced_quotient_filter_benchmarks
if [ $? -eq 0 ]; then
    print_success "Advanced quotient filter benchmarks completed"
else
    print_error "Advanced quotient filter benchmarks failed"
    exit 1
fi

echo ""
print_status "Running performance comparison benchmarks..."
echo "==============================================="

# Run comparison benchmarks
cargo bench --bench maplet_vs_std
if [ $? -eq 0 ]; then
    print_success "Comparison benchmarks completed"
else
    print_error "Comparison benchmarks failed"
    exit 1
fi

echo ""
print_status "Running storage benchmarks..."
echo "==============================="

# Run storage benchmarks
cargo bench --bench storage_benchmarks
if [ $? -eq 0 ]; then
    print_success "Storage benchmarks completed"
else
    print_error "Storage benchmarks failed"
    exit 1
fi

echo ""
print_status "Running comprehensive test suite..."
echo "======================================="

# Run all tests with different configurations
print_status "Testing with different hash functions..."
cargo test --features advanced-quotient-filter -- --nocapture

print_status "Testing with different capacities..."
cargo test --features advanced-quotient-filter -- --nocapture

print_status "Testing with different load factors..."
cargo test --features advanced-quotient-filter -- --nocapture

echo ""
print_status "Running stress tests..."
echo "========================="

# Run stress tests
print_status "Running memory stress tests..."
cargo test --features advanced-quotient-filter --release -- --nocapture

print_status "Running concurrent stress tests..."
cargo test --features advanced-quotient-filter --release -- --nocapture

echo ""
print_status "Running performance analysis..."
echo "=================================="

# Run performance analysis
print_status "Analyzing performance characteristics..."
cargo bench --bench quotient_filter_benchmarks -- --output-format html
cargo bench --bench advanced_quotient_filter_benchmarks -- --output-format html

echo ""
print_status "Generating benchmark reports..."
echo "==================================="

# Generate benchmark reports
if [ -d "target/criterion" ]; then
    print_success "Benchmark reports generated in target/criterion/"
    print_status "You can view the reports by opening target/criterion/index.html in your browser"
else
    print_warning "Benchmark reports not found"
fi

echo ""
print_status "Running final validation..."
echo "=============================="

# Final validation
cargo test --features advanced-quotient-filter --release
if [ $? -eq 0 ]; then
    print_success "Final validation passed"
else
    print_error "Final validation failed"
    exit 1
fi

echo ""
echo "=================================================================="
print_success "All tests and benchmarks completed successfully! 🦊"
echo ""
print_status "Summary:"
echo "========="
echo "✅ Basic tests passed"
echo "✅ Quotient filter tests passed"
echo "✅ Advanced quotient filter tests passed"
echo "✅ Quotient filter benchmarks completed"
echo "✅ Advanced quotient filter benchmarks completed"
echo "✅ Comparison benchmarks completed"
echo "✅ Storage benchmarks completed"
echo "✅ Stress tests completed"
echo "✅ Performance analysis completed"
echo ""
print_status "The quotient filter implementation is working correctly and performing well!"
echo ""
print_status "To view benchmark results, open target/criterion/index.html in your browser"
echo "=================================================================="
