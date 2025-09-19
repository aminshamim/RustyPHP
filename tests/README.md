# Testing Documentation

This document describes the testing structure and organization for RustyPHP.

## Directory Structure

```
RustyPHP/
├── tests/                          # Main test directory
│   ├── debug/                      # Debug PHP files for troubleshooting
│   │   ├── debug_comment_test.php
│   │   ├── debug_string_test.php
│   │   └── debug_string_with_colon.php
│   ├── php_files/                  # PHP test files organized by component
│   │   ├── lexer/                  # Lexer-specific test files
│   │   │   ├── minimal_comment_test.php
│   │   │   ├── multiline_comment_test.php
│   │   │   └── simple_comment_test.php
│   │   ├── parser/                 # Parser-specific test files
│   │   │   ├── test_constants.php
│   │   │   ├── test_modular.php
│   │   │   └── simple_modular_test.php
│   │   ├── arithmetic.php          # General test files
│   │   ├── basic.php
│   │   ├── progress_test.php
│   │   ├── test.php
│   │   └── test_simple.php
│   ├── benchmarks/                 # Performance benchmarks
│   └── compatibility/              # PHP compatibility tests
├── crates/*/tests/                 # Crate-specific unit tests
│   └── integration_tests.rs
└── scripts/
    ├── test_all.sh                # Comprehensive test runner
    └── test_runner.sh             # Legacy test runner
```

## Test Types

### 1. Unit Tests
Located in each crate's `tests/` directory:
- **Lexer Tests** (`php-lexer/tests/`): Token recognition, comment handling, operator parsing
- **Parser Tests** (`php-parser/tests/`): AST generation, statement parsing, expression parsing
- **Runtime Tests** (`php-runtime/tests/`): Execution engine, variable management
- **Other Crates**: Type system, standard library, CLI, FFI, web interface

### 2. Integration Tests
- **PHP File Tests**: Real PHP files that test end-to-end functionality
- **Component Integration**: Tests that verify modules work together
- **Full Workflow Tests**: Lexing → Parsing → Runtime execution

### 3. Module-Specific Tests
The parser now has modular unit tests for each component:
- **Statement Parser**: Echo, print, assignments, constants
- **Expression Parser**: Binary operations, precedence, primary expressions
- **Control Flow Parser**: If/else, while loops, for loops, return/break/continue
- **Utilities**: Token navigation, error handling

### 4. Debug Tests
Files in `tests/debug/` are used for troubleshooting specific issues:
- Comment parsing edge cases
- String handling with special characters
- Lexer debugging scenarios

## Running Tests

### Quick Test
```bash
cargo test --workspace
```

### Comprehensive Test Suite
```bash
./scripts/test_all.sh
```

### Individual Component Tests
```bash
cargo test --package php-lexer
cargo test --package php-parser
cargo test --package php-runtime
```

### PHP File Tests
```bash
./target/release/php tests/php_files/basic.php
./target/release/php tests/php_files/arithmetic.php
```

## Test Categories

### ✅ Passing Tests
- Basic token recognition
- Simple PHP statements
- Variable assignments
- Arithmetic expressions
- Comment handling
- String literals
- Modular parser functionality

### 🧪 Integration Tests  
- Multi-statement programs
- Complex expressions with precedence
- Control flow statements
- Constant definitions
- Mixed content files

### 🐛 Debug Tests
- Edge cases that previously failed
- Specific bug reproductions
- Performance edge cases
- Compatibility issues

## Adding New Tests

### For New Features
1. Add unit tests in the appropriate crate's `tests/` directory
2. Create PHP test files in `tests/php_files/` 
3. Add integration tests if the feature crosses module boundaries
4. Update the test runner script if needed

### For Bug Fixes
1. Create a minimal reproduction case in `tests/debug/`
2. Add a regression test in the appropriate unit test file
3. Verify the fix with the comprehensive test suite

### For Performance
1. Add benchmark tests in `tests/benchmarks/`
2. Use `cargo bench` for micro-benchmarks
3. Test with larger PHP files for macro-performance

## Test Naming Conventions

- **Unit tests**: `test_feature_name()`
- **Integration tests**: `test_integration_feature()`
- **PHP files**: `test_feature.php` or `feature_test.php`
- **Debug files**: `debug_issue_description.php`

## Continuous Integration

The test suite is designed to be run in CI/CD pipelines:
- All tests must pass for PR acceptance
- Debug tests are informational and may fail
- Benchmark tests track performance regressions
- Integration tests verify real-world usage

## Test Philosophy

1. **Modularity**: Each component has focused, isolated tests
2. **Real-world Usage**: PHP files test actual use cases
3. **Regression Prevention**: Every bug gets a test
4. **Performance Awareness**: Benchmarks track performance impact
5. **Debugging Support**: Debug tests help troubleshoot issues

## Metrics and Coverage

Run with coverage to ensure comprehensive testing:
```bash
cargo tarpaulin --workspace --out Html
```

The goal is 80%+ test coverage across all core modules with 100% coverage for critical parsing logic.
