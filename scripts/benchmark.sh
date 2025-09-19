#!/bin/bash
# Run performance benchmarks

echo "⚡ Running RustyPHP benchmarks..."

# Compilation benchmarks
cargo bench --package php-lexer
cargo bench --package php-parser
cargo bench --package php-runtime

# Execution benchmarks against PHP test files
echo "📊 Benchmarking against test PHP files..."
for file in tests/php_files/*.php; do
    echo "Testing: $file"
    # TODO: Add actual benchmarking logic
done

echo "✅ Benchmarks completed!"
