//! Literal token recognition for PHP lexer
//!
//! This module handles recognition of PHP literals:
//! - String literals (single and double quoted)
//! - Number literals (integers and floats)
//! - Variables ($variable)

use crate::error::{LexError, LexResult};
use crate::stream::CharStream;
use crate::token::Token;

/// Literal token recognition functionality
pub struct LiteralHandler;

impl LiteralHandler {
    /// Tokenize a variable ($variable)
    pub fn tokenize_variable(stream: &mut CharStream) -> LexResult<Token> {
        stream.next(); // consume '$'
        
        let mut name = String::new();
        while let Some(&ch) = stream.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                name.push(stream.next().unwrap());
            } else {
                break;
            }
        }
        
        if name.is_empty() {
            let pos = stream.position();
            return Err(LexError::UnexpectedChar {
                char: '$',
                line: pos.line,
                column: pos.column,
            });
        }
        
        Ok(Token::Variable(name))
    }

    /// Tokenize a string literal
    pub fn tokenize_string(stream: &mut CharStream) -> LexResult<Token> {
        let quote_char = stream.peek().copied().unwrap();
        let content = stream.read_string(quote_char)?;
        Ok(Token::String(content))
    }

    /// Tokenize a number literal
    pub fn tokenize_number(stream: &mut CharStream) -> LexResult<Token> {
        let number = stream.read_number()?;
        Ok(Token::Number(number))
    }

    /// Tokenize an identifier
    pub fn tokenize_identifier(stream: &mut CharStream) -> String {
        stream.read_identifier()
    }
}
