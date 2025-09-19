# Migration Guide: Current Code → Multi-Crate Structure

This guide helps migrate the existing RustyPHP code to the new multi-crate workspace structure.

## Current State Analysis

The existing codebase has:
- `src/lexer.rs` - Tokenization logic
- `src/parser.rs` - Basic parsing with limited operators
- `src/ast.rs` - AST definitions with embedded execution
- `src/interpreter.rs` - Alternative execution logic
- `src/playground/` - Web interface
- `src/main.rs` - Web server setup

## Migration Steps

### Step 1: Run Setup Script
```bash
./scripts/setup_workspace.sh
```

### Step 2: Migrate Lexer
```bash
# Move lexer code
cp src/lexer.rs crates/php-lexer/src/lexer.rs

# Update crates/php-lexer/src/lib.rs:
```
```rust
//! PHP lexical analysis and tokenization
//! 
//! This crate provides tokenization of PHP source code into a stream of tokens
//! for consumption by the parser.

pub mod token;
pub mod lexer; 
pub mod error;
pub mod stream;

pub use lexer::*;
pub use token::*;
```

### Step 3: Migrate Parser & AST
```bash
# Move parser code
cp src/parser.rs crates/php-parser/src/parser.rs
cp src/ast.rs crates/php-parser/src/ast/

# Split ast.rs into logical modules:
# - ast/expr.rs (expressions)
# - ast/stmt.rs (statements)  
# - ast/literal.rs (literals)
# - ast/visitor.rs (visitor pattern)
```

### Step 4: Create Type System
```bash
# Extract type handling from ast.rs to php-types
# Focus on PHP value representations, not execution
```

### Step 5: Migrate Runtime
```bash
# Move interpreter logic to php-runtime
cp src/interpreter.rs crates/php-runtime/src/engine/interpreter.rs

# Separate execution from AST definitions
# Create proper execution context
```

### Step 6: Migrate Web Interface  
```bash
# Move playground to php-web
cp -r src/playground/ crates/php-web/src/playground/
cp src/main.rs crates/php-web/src/bin/server.rs
```

### Step 7: Update Dependencies

Each crate will have focused dependencies:

**php-lexer/Cargo.toml:**
```toml
[dependencies]
serde.workspace = true
thiserror.workspace = true
```

**php-parser/Cargo.toml:**
```toml
[dependencies]
php-lexer = { path = "../php-lexer" }
php-types = { path = "../php-types" }
serde.workspace = true
thiserror.workspace = true
```

**php-runtime/Cargo.toml:**
```toml
[dependencies]
php-parser = { path = "../php-parser" }
php-types = { path = "../php-types" }
php-stdlib = { path = "../php-stdlib" }
serde.workspace = true
thiserror.workspace = true
tokio.workspace = true
```

**php-web/Cargo.toml:**
```toml
[dependencies]
php-runtime = { path = "../php-runtime" }
actix-web.workspace = true
tera.workspace = true
serde.workspace = true
serde_json.workspace = true
tokio.workspace = true
```

### Step 8: Clean Separation of Concerns

**Before (current):**
- AST nodes contain execution logic
- Lexer and parser tightly coupled
- Single HashMap-based context
- Web interface mixed with core logic

**After (target):**
- Pure AST without execution logic
- Clean lexer → parser → runtime pipeline
- Proper execution context with scoping
- Separated web SAPI from core runtime

### Step 9: Update Import Paths

**Old:**
```rust
use crate::lexer;
use crate::parser;
use crate::ast::{Node, Operator};
```

**New:**
```rust
use php_lexer::{lex, Token};
use php_parser::{parse, ast::Node};
use php_runtime::{Engine, ExecutionContext};
```

### Step 10: Testing Strategy

Create comprehensive tests for each crate:

```bash
# Test lexer independently
cargo test --package php-lexer

# Test parser with mock tokens
cargo test --package php-parser

# Test runtime with mock AST
cargo test --package php-runtime

# Integration tests
cargo test --workspace
```

## Key Architectural Changes

### 1. Separation of AST and Execution
**Current:** AST nodes have `execute()` methods
**New:** Pure AST + separate interpreter

### 2. Error Handling
**Current:** String-based errors
**New:** Typed error enums with position info

### 3. Context Management  
**Current:** Two separate HashMaps
**New:** Unified ExecutionContext with proper scoping

### 4. Type System
**Current:** Mixed f64/String handling
**New:** Proper PHP value type hierarchy

## Benefits of New Structure

1. **Modularity**: Each crate has single responsibility
2. **Testing**: Easier to test components in isolation  
3. **Performance**: Optimized data structures per component
4. **Maintainability**: Clear dependency boundaries
5. **Extensibility**: Easy to add new crates for features
6. **Reusability**: Components can be used independently

## Backward Compatibility

The migration maintains the existing playground functionality while providing a cleaner foundation for future development. The web interface should work identically after migration.

## Validation

After migration, verify:
1. `cargo check --workspace` passes
2. `cargo test --workspace` passes  
3. Web playground still works
4. Performance hasn't regressed
5. All existing functionality preserved

This migration sets the foundation for implementing the full roadmap efficiently.
