# RustyPHP - AI Coding Assistant Instructions

## Project Overview

RustyPHP is a **High Performance, Multi Threaded, complete PHP interpreter and compiler implemented in Rust**, targeting production-level compatibility with PHP 8.x while leveraging Rust's memory safety and performance. The project is organized as a **multi-crate workspace** with clear separation of concerns, all codes are properly organized, following best practices and cover all unit tests. The php debug and feature tests are organized in the `tests/` directory. All functions and modules are documented with Rustdoc comments. All php functions are written in separate files inside the crates. same goes for the tests. all tests are written in separate files inside the `tests/` directory. 

**Current Status (Phase 1 - 75% Complete):**
- ✅ **Lexer**: Complete tokenization with comment/string handling (`php-lexer`)
- ✅ **Parser**: Full AST generation with operator precedence (`php-parser`) 
- 🚧 **Runtime**: Basic execution engine in development (`php-runtime`)
- 📅 **Standard Library & Extensions**: Planned phases

## Architecture & Crate Structure

The project follows a **layered dependency model**:

```
php-cli ───┐
           ├─── php-runtime ───┬─── php-parser ───── php-lexer
php-web ───┘                  ├─── php-types
                               └─── php-stdlib ───── php-ffi
```

### Key Crates & Responsibilities

- **`php-lexer`**: Tokenization only - handles PHP tags, comments, strings, operators
- **`php-parser`**: Pure AST generation - no execution logic, modular statement/expression parsers
- **`php-runtime`**: Execution engine - variable storage, expression evaluation
- **`php-types`**: PHP's dynamic type system with Rust safety
- **`php-stdlib`**: Built-in functions (echo, print, array functions)
- **`php-cli`/`php-web`**: Different execution environments

## Development Patterns & Conventions

### Error Handling Pattern
All crates use **typed error enums with `thiserror`**:
```rust
// Consistent across crates
pub type LexResult<T> = Result<T, LexError>;
pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LexError {
    #[error("Unexpected character '{char}' at line {line}, column {column}")]
    UnexpectedChar { char: char, line: usize, column: usize },
    // Position info is critical for debugging
}
```

### Modular Parser Architecture
The parser is **deliberately modular** with separate handlers:
- `parser/statements.rs` - Echo, assignments, constants
- `parser/expressions.rs` - Binary ops with precedence climbing
- `parser/utils.rs` - Token navigation utilities

### Testing Philosophy
- **Component isolation**: Each crate tests independently
- **Real PHP files**: `tests/php_files/` contains actual PHP code for integration
- **Debug infrastructure**: `tests/debug/` for troubleshooting edge cases
- **Comprehensive coverage**: Run `./scripts/test_all.sh` (16+ tests across components)

## Build & Test Workflows

### Essential Commands
```bash
# Core development loop
cargo check --workspace                    # Fast compilation check
cargo test --package php-lexer            # Test specific component
./scripts/test_all.sh                     # Comprehensive test suite (16 tests)
cargo test --workspace                    # All unit tests

# Build artifacts
cargo build --workspace --release         # Optimized build
cargo run tests/php_files/basic.php       # Test with actual PHP
```

### Test File Organization
```
tests/
├── php_files/
│   ├── lexer/          # Comment parsing, tokenization edge cases
│   ├── parser/         # AST generation, precedence validation
│   ├── basic.php       # Simple variable assignments & echo
│   └── arithmetic.php  # Expression evaluation with precedence
├── debug/              # Reproduction cases for specific bugs
└── README.md           # Detailed testing documentation
```

## Critical Implementation Details

### Character Stream Handling
The lexer uses a **position-aware character stream** with proper PHP tag context:
```rust
// php-lexer/src/stream.rs - tracks line/column for error reporting
stream.peek()           // Look ahead without consuming
stream.peek_ahead(2)    // Multi-character lookahead for "//" vs "/"
```

### Operator Precedence Parsing
Parser uses **precedence climbing** (not recursive descent) for expressions:
```rust
// php-parser/src/parser/expressions.rs
parse_expression_precedence(tokens, position, min_precedence)
// Handles: 2 + 3 * 4 correctly as 2 + (3 * 4)
```

### AST Design Philosophy
**Pure AST nodes** - no execution methods:
```rust
// php-parser/src/ast/
pub enum Expr {
    Variable(String),
    BinaryOp { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
    // No execute() methods - that's runtime's job
}
```

## Common Development Tasks

### Adding New PHP Features
1. **Lexer**: Add tokens in `php-lexer/src/token.rs`
2. **Parser**: Extend appropriate parser module (`statements.rs` vs `expressions.rs`)
3. **AST**: Add AST nodes in `php-parser/src/ast/`
4. **Tests**: Create PHP test file in `tests/php_files/`

### Debugging Parse Failures
1. Add minimal reproduction to `tests/debug/debug_new_issue.php`
2. Run with lexer output: Check token stream first
3. Use modular tests: Test statement vs expression parsing separately

### Performance Considerations
- **Avoid string allocations** in hot paths (lexer/parser)
- **Use workspace dependencies** - shared `serde`, `thiserror`
- **Test with realistic PHP files** - not just minimal examples

## Integration Points & Dependencies

### Cross-Crate Communication
- **Lexer → Parser**: `Token` stream via `php-lexer::lex()`
- **Parser → Runtime**: `Stmt`/`Expr` AST via `php-parser::parse()`
- **Types throughout**: `php-types::Value` for runtime values

### External Dependencies Strategy
- **Core parsing**: Minimal deps (no `nom`, custom char stream)
- **Web interface**: `actix-web` + `tera` templates in `php-web`
- **CLI tools**: `clap` for argument parsing in `php-cli`

### Future Migration Notes
Current legacy `src/` playground will be **deprecated** once multi-crate architecture is fully implemented. Key migration patterns documented in `MIGRATION.md`.

## Project-Specific Context

- **Target compatibility**: PHP 8.3+ (not legacy PHP)
- **Performance goal**: 20-30% faster than Zend PHP
- **Memory safety**: Leverage Rust ownership vs manual memory management
- **Extension model**: `php-ffi` for C extension compatibility
- **Development phase**: Currently implementing runtime execution engine

When working on this codebase, always consider **which crate** a change belongs in and maintain the **clean dependency boundaries**. The modular architecture is fundamental to the project's maintainability and testing strategy.