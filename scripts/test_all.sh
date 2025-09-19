#!/bin/bash

# RustyPHP Test Runner
# Organizes and runs all tests in the project

set -e

echo "🧪 RustyPHP Test Suite"
echo "====================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${BLUE}Running: ${test_name}${NC}"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASSED: ${test_name}${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗ FAILED: ${test_name}${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        # Show the actual error
        echo -e "${YELLOW}Error details:${NC}"
        eval "$test_command"
    fi
    echo
}

echo -e "${BLUE}1. Unit Tests${NC}"
echo "-------------"

run_test "Lexer Unit Tests" "cargo test --package php-lexer"
run_test "Parser Unit Tests" "cargo test --package php-parser"
run_test "Runtime Unit Tests" "cargo test --package php-runtime"
run_test "Types Unit Tests" "cargo test --package php-types"
run_test "Stdlib Unit Tests" "cargo test --package php-stdlib"
run_test "CLI Unit Tests" "cargo test --package php-cli"

echo -e "${BLUE}2. Integration Tests${NC}"
echo "--------------------"

# Test with actual PHP files
echo -e "${YELLOW}Testing with PHP files:${NC}"

# Basic parsing tests
if [ -f "tests/php_files/basic.php" ]; then
    run_test "Basic PHP File" "./target/release/php tests/php_files/basic.php"
fi

if [ -f "tests/php_files/arithmetic.php" ]; then
    run_test "Arithmetic PHP File" "./target/release/php tests/php_files/arithmetic.php"
fi

# Parser-specific tests
if [ -f "tests/php_files/parser/test_modular.php" ]; then
    run_test "Modular Parser Test" "./target/release/php tests/php_files/parser/test_modular.php"
fi

if [ -f "tests/php_files/parser/test_constants.php" ]; then
    run_test "Constants Test" "./target/release/php tests/php_files/parser/test_constants.php"
fi

# Lexer-specific tests
echo -e "${YELLOW}Lexer Tests:${NC}"
for test_file in tests/php_files/lexer/*.php; do
    if [ -f "$test_file" ]; then
        test_name=$(basename "$test_file" .php)
        run_test "Lexer: $test_name" "./target/release/php '$test_file'"
    fi
done

echo -e "${BLUE}3. Benchmark Tests${NC}"
echo "------------------"

if [ -d "tests/benchmarks" ]; then
    run_test "Benchmarks" "cargo bench --package php-parser"
fi

echo -e "${BLUE}4. Debug Tests${NC}"
echo "--------------"

echo -e "${YELLOW}Running debug tests (these may fail and that's OK):${NC}"
for debug_file in tests/debug/*.php; do
    if [ -f "$debug_file" ]; then
        debug_name=$(basename "$debug_file" .php)
        echo -e "${BLUE}Debug: ${debug_name}${NC}"
        if ./target/release/php "$debug_file" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ Debug test passed${NC}"
        else
            echo -e "${YELLOW}⚠ Debug test failed (expected)${NC}"
        fi
    fi
done

echo -e "${BLUE}5. Workspace Tests${NC}"
echo "------------------"

run_test "Full Workspace Test" "cargo test --workspace"
run_test "Workspace Check" "cargo check --workspace"

echo
echo "=============================="
echo -e "${BLUE}Test Summary${NC}"
echo "=============================="
echo -e "Total Tests: ${TOTAL_TESTS}"
echo -e "${GREEN}Passed: ${PASSED_TESTS}${NC}"
echo -e "${RED}Failed: ${FAILED_TESTS}${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi
