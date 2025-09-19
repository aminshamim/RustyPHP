#!/bin/bash
# Run all tests across the workspace

echo "🧪 Running RustyPHP test suite..."

# Unit tests
cargo test --workspace

# Integration tests  
cargo test --workspace --test '*'

# Benchmarks
cargo bench --workspace

echo "✅ All tests completed!"
