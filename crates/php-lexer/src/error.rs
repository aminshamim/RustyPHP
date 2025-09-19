//! Error types for PHP lexer

use thiserror::Error;

/// Lexing errors that can occur during tokenization
#[derive(Error, Debug, Clone, PartialEq)]
pub enum LexError {
    /// Unexpected character encountered
    #[error("Unexpected character '{char}' at line {line}, column {column}")]
    UnexpectedChar { char: char, line: usize, column: usize },
    
    /// Unterminated string literal
    #[error("Unterminated string literal starting at line {line}, column {column}")]
    UnterminatedString { line: usize, column: usize },
    
    /// Invalid number format
    #[error("Invalid number format '{number}' at line {line}, column {column}")]
    InvalidNumber { number: String, line: usize, column: usize },
    
    /// Unexpected end of file
    #[error("Unexpected end of file")]
    UnexpectedEof,
}

/// Result type for lexer operations
pub type LexResult<T> = Result<T, LexError>;
