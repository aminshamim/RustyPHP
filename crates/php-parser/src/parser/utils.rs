//! Parser utilities for PHP parser
//!
//! This module contains common utility functions used across
//! different parser modules.

use php_lexer::Token;
use std::iter::Peekable;
use std::vec::IntoIter;

/// Parser utility functions
pub struct ParserUtils;

impl ParserUtils {
    /// Peek at next token
    pub fn peek_token(tokens: &mut Peekable<IntoIter<Token>>) -> Option<&Token> {
        tokens.peek()
    }

    /// Consume next token
    pub fn next_token(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> Option<Token> {
        *position += 1;
        tokens.next()
    }
}
