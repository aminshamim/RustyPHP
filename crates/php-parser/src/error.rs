//! Parser error types

use thiserror::Error;

/// Parsing errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Unexpected token
    #[error("Unexpected token: {token:?} at position {position}")]
    UnexpectedToken { token: String, position: usize },
    
    /// Expected specific token
    #[error("Expected {expected}, found {found} at position {position}")]
    ExpectedToken { expected: String, found: String, position: usize },
    
    /// Unexpected end of input
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    /// Invalid expression
    #[error("Invalid expression: {message}")]
    InvalidExpression { message: String },
    
    /// Invalid statement
    #[error("Invalid statement: {message}")]
    InvalidStatement { message: String },
}

/// Result type for parser operations
pub type ParseResult<T> = Result<T, ParseError>;
