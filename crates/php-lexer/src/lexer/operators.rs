//! Operator token recognition for PHP lexer
//!
//! This module handles recognition of PHP operators:
//! - Arithmetic operators (+, -, *, /, %)
//! - Comparison operators (==, !=, <, >, <=, >=)
//! - Assignment operators (=)
//! - Logical operators (&&, ||, !)

use crate::error::LexResult;
use crate::stream::CharStream;
use crate::token::Token;

/// Operator token recognition functionality
pub struct OperatorHandler;

impl OperatorHandler {
    /// Tokenize equals or double equals
    pub fn tokenize_equals(stream: &mut CharStream) -> LexResult<Token> {
        stream.next(); // consume '='
        
        if let Some(&'=') = stream.peek() {
            stream.next(); // consume second '='
            Ok(Token::DoubleEquals)
        } else {
            Ok(Token::Equals)
        }
    }

    /// Tokenize less than or less or equal
    pub fn tokenize_less_than(stream: &mut CharStream) -> LexResult<Token> {
        stream.next(); // consume '<'
        
        if let Some(&'=') = stream.peek() {
            stream.next(); // consume '='
            Ok(Token::LessOrEqual)
        } else {
            Ok(Token::LessThan)
        }
    }

    /// Tokenize greater than or greater or equal
    pub fn tokenize_greater_than(stream: &mut CharStream) -> LexResult<Token> {
        stream.next(); // consume '>'
        
        if let Some(&'=') = stream.peek() {
            stream.next(); // consume '='
            Ok(Token::GreaterOrEqual)
        } else {
            Ok(Token::GreaterThan)
        }
    }

    /// Tokenize not equals
    pub fn tokenize_not_equals(stream: &mut CharStream) -> LexResult<Token> {
        stream.next(); // consume '!'
        
        if let Some(&'=') = stream.peek() {
            stream.next(); // consume '='
            Ok(Token::NotEquals)
        } else {
            // Just '!' - could be logical NOT in the future
            Ok(Token::NotEquals) // For now, treat as incomplete !=
        }
    }

    /// Tokenize PHP opening tag
    pub fn try_php_open(stream: &mut CharStream) -> LexResult<Token> {
        // Expect "<?php"
        if stream.peek_ahead(5) == "<?php" {
            // Consume the entire tag
            stream.next(); // '<'
            stream.next(); // '?'
            stream.next(); // 'p'
            stream.next(); // 'h'
            stream.next(); // 'p'
            Ok(Token::PhpOpen)
        } else {
            Self::tokenize_less_than(stream)
        }
    }

    /// Tokenize PHP closing tag
    pub fn try_php_close(stream: &mut CharStream) -> LexResult<Token> {
        // Check for ?>
        if stream.peek_ahead(1).chars().next() == Some('>') {
            stream.next(); // '?'
            stream.next(); // '>'
            Ok(Token::PhpClose)
        } else {
            // Just '?' - not implemented as operator yet
            stream.next();
            Ok(Token::PhpClose) // Placeholder for now
        }
    }

    /// Tokenize ampersand operators (& and &&)
    pub fn tokenize_ampersand(stream: &mut CharStream) -> LexResult<Token> {
        stream.next(); // consume '&'
        
        if let Some(&'&') = stream.peek() {
            stream.next(); // consume second '&'
            Ok(Token::LogicalAnd)
        } else {
            Ok(Token::Ampersand)
        }
    }

    /// Tokenize pipe operators (| and ||)
    pub fn tokenize_pipe(stream: &mut CharStream) -> LexResult<Token> {
        stream.next(); // consume '|'
        
        if let Some(&'|') = stream.peek() {
            stream.next(); // consume second '|'
            Ok(Token::LogicalOr)
        } else {
            // Single pipe would be bitwise OR, but for now just treat as logical OR
            Ok(Token::LogicalOr)
        }
    }
}
