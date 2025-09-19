//! Main PHP lexer implementation
//!
//! This module contains the main Lexer struct that coordinates all the
//! specialized token recognition modules.

use std::iter::Peekable;
use std::vec::IntoIter;

use crate::error::{LexError, LexResult};
use crate::stream::CharStream;
use crate::token::Token;

use super::comments::CommentHandler;
use super::keywords::KeywordHandler;
use super::literals::LiteralHandler;
use super::operators::OperatorHandler;

/// Main PHP lexer
pub struct Lexer<'a> {
    stream: CharStream<'a>,
    keyword_handler: KeywordHandler,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input
    pub fn new(input: &'a str) -> Self {
        Self {
            stream: CharStream::new(input),
            keyword_handler: KeywordHandler::new(),
        }
    }
    
    /// Tokenize the entire input into a vector of tokens
    pub fn tokenize(&mut self) -> LexResult<Vec<Token>> {
        let mut tokens = Vec::new();
        
        while let Some(token) = self.next_token()? {
            if token != Token::EOF {
                tokens.push(token);
            } else {
                break;
            }
        }
        
        Ok(tokens)
    }
    
    /// Get next token from input
    pub fn next_token(&mut self) -> LexResult<Option<Token>> {
        // Skip whitespace
        self.skip_whitespace();
        
        loop {
            // Check for end of input
            if self.stream.is_at_end() {
                return Ok(Some(Token::EOF));
            }
            
            // Try to skip comments
            if CommentHandler::try_skip_comment(&mut self.stream)? {
                self.skip_whitespace();
                continue;
            }
            
            // Get next character and tokenize it
            if let Some(ch) = self.stream.peek().copied() {
                return Ok(Some(self.tokenize_char(ch)?));
            } else {
                return Ok(Some(Token::EOF));
            }
        }
    }
    
    /// Convert to iterator
    pub fn into_iter(mut self) -> LexResult<TokenIterator> {
        let tokens = self.tokenize()?;
        Ok(TokenIterator::new(tokens))
    }
    
    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.stream.peek() {
            if ch.is_whitespace() {
                self.stream.next();
            } else {
                break;
            }
        }
    }
    
    /// Tokenize a single character
    fn tokenize_char(&mut self, ch: char) -> LexResult<Token> {
        match ch {
            // PHP tags
            '<' => {
                // Check if it's <?php
                if self.stream.peek_ahead(5) == "<?php" {
                    OperatorHandler::try_php_open(&mut self.stream)
                } else {
                    OperatorHandler::tokenize_less_than(&mut self.stream)
                }
            }
            '?' => OperatorHandler::try_php_close(&mut self.stream),
            
            // Variables
            '$' => LiteralHandler::tokenize_variable(&mut self.stream),
            
            // String literals
            '"' | '\'' => LiteralHandler::tokenize_string(&mut self.stream),
            
            // Numbers
            '0'..='9' => LiteralHandler::tokenize_number(&mut self.stream),
            
            // Operators
            '=' => OperatorHandler::tokenize_equals(&mut self.stream),
            '>' => OperatorHandler::tokenize_greater_than(&mut self.stream),
            '!' => OperatorHandler::tokenize_not_equals(&mut self.stream),
            
            // Single character tokens
            '+' => { self.stream.next(); Ok(Token::Plus) }
            '-' => { self.stream.next(); Ok(Token::Minus) }
            '*' => { self.stream.next(); Ok(Token::Multiply) }
            '/' => { 
                // This should only be reached if it's not a comment
                self.stream.next(); 
                Ok(Token::Divide) 
            }
            '.' => { self.stream.next(); Ok(Token::Dot) }
            ';' => { self.stream.next(); Ok(Token::Semicolon) }
            ',' => { self.stream.next(); Ok(Token::Comma) }
            '(' => { self.stream.next(); Ok(Token::OpenParen) }
            ')' => { self.stream.next(); Ok(Token::CloseParen) }
            '{' => { self.stream.next(); Ok(Token::OpenBrace) }
            '}' => { self.stream.next(); Ok(Token::CloseBrace) }
            '[' => { self.stream.next(); Ok(Token::OpenBracket) }
            ']' => { self.stream.next(); Ok(Token::CloseBracket) }
            
            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                let word = LiteralHandler::tokenize_identifier(&mut self.stream);
                Ok(self.keyword_handler.lookup_keyword(&word))
            }
            
            // Unexpected character
            _ => {
                let pos = self.stream.position();
                self.stream.next();
                Err(LexError::UnexpectedChar {
                    char: ch,
                    line: pos.line,
                    column: pos.column,
                })
            }
        }
    }
}

/// Iterator wrapper for tokens
pub struct TokenIterator {
    tokens: Peekable<IntoIter<Token>>,
}

impl TokenIterator {
    /// Create new token iterator
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }
    
    /// Peek at next token
    pub fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }
}

impl Iterator for TokenIterator {
    type Item = Token;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.next()
    }
}
