//! Control flow parsing for PHP parser
//!
//! This module handles parsing of PHP control flow statements:
//! - If/else statements
//! - While loops
//! - For loops
//! - Break and continue statements
//! - Return statements

use crate::ast::{Stmt};
use crate::error::{ParseError, ParseResult};
use php_lexer::Token;
use std::iter::Peekable;
use std::vec::IntoIter;

/// Control flow parsing functionality
pub struct ControlFlowParser;

impl ControlFlowParser {
    /// Parse if statement
    pub fn parse_if(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        Self::consume_token(tokens, position, Token::If)?;
        Self::consume_token(tokens, position, Token::OpenParen)?;
        let condition = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
        Self::consume_token(tokens, position, Token::CloseParen)?;
        let then_stmt = Box::new(super::main::Parser::parse_statement_with_tokens(tokens, position)?);

        let else_stmt = if let Some(&Token::Else) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position);
            Some(Box::new(super::main::Parser::parse_statement_with_tokens(tokens, position)?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_stmt,
            else_stmt,
        })
    }

    /// Parse while loop
    pub fn parse_while(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        Self::consume_token(tokens, position, Token::While)?;
        Self::consume_token(tokens, position, Token::OpenParen)?;
        let condition = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
        Self::consume_token(tokens, position, Token::CloseParen)?;
        let body = Box::new(super::main::Parser::parse_statement_with_tokens(tokens, position)?);

        Ok(Stmt::While { condition, body })
    }

    /// Parse for loop
    pub fn parse_for(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        Self::consume_token(tokens, position, Token::For)?;
        Self::consume_token(tokens, position, Token::OpenParen)?;

        let init = if let Some(&Token::Semicolon) = tokens.peek() {
            None
        } else {
            Some(Box::new(super::main::Parser::parse_statement_with_tokens(tokens, position)?))
        };

        if init.is_none() {
            Self::consume_token(tokens, position, Token::Semicolon)?;
        }

        let condition = if let Some(&Token::Semicolon) = tokens.peek() {
            None
        } else {
            Some(super::expressions::ExpressionParser::parse_expression(tokens, position)?)
        };

        Self::consume_token(tokens, position, Token::Semicolon)?;

        let increment = if let Some(&Token::CloseParen) = tokens.peek() {
            None
        } else {
            Some(super::expressions::ExpressionParser::parse_expression(tokens, position)?)
        };

        Self::consume_token(tokens, position, Token::CloseParen)?;

        let body = Box::new(super::main::Parser::parse_statement_with_tokens(tokens, position)?);

        Ok(Stmt::For {
            init,
            condition,
            increment,
            body,
        })
    }

    /// Parse return statement
    pub fn parse_return(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        Self::consume_token(tokens, position, Token::Return)?;

        let value = if let Some(&Token::Semicolon) = tokens.peek() {
            None
        } else {
            Some(super::expressions::ExpressionParser::parse_expression(tokens, position)?)
        };

        Self::consume_semicolon(tokens, position)?;

        Ok(Stmt::Return(value))
    }

    /// Parse break statement
    pub fn parse_break(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        Self::consume_token(tokens, position, Token::Break)?;
        Self::consume_semicolon(tokens, position)?;
        Ok(Stmt::Break)
    }

    /// Parse continue statement
    pub fn parse_continue(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        Self::consume_token(tokens, position, Token::Continue)?;
        Self::consume_semicolon(tokens, position)?;
        Ok(Stmt::Continue)
    }

    /// Consume specific token or return error
    fn consume_token(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
        expected: Token,
    ) -> ParseResult<()> {
        match super::utils::ParserUtils::next_token(tokens, position) {
            Some(token) if std::mem::discriminant(&token) == std::mem::discriminant(&expected) => {
                Ok(())
            }
            Some(token) => Err(ParseError::ExpectedToken {
                expected: format!("{:?}", expected),
                found: format!("{:?}", token),
                position: *position,
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    /// Consume semicolon
    fn consume_semicolon(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<()> {
        Self::consume_token(tokens, position, Token::Semicolon)
    }
}
