//! Statement parsing for PHP parser
//!
//! This module handles parsing of PHP statements:
//! - Echo and print statements
//! - Variable assignments
//! - Constant definitions
//! - Expression statements

use crate::ast::{Expr, Stmt};
use crate::error::{ParseError, ParseResult};
use php_lexer::Token;
use std::iter::Peekable;
use std::vec::IntoIter;

/// Statement parsing functionality
pub struct StatementParser;

impl StatementParser {
    /// Parse echo statement
    pub fn parse_echo(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        Self::consume_token(tokens, position, Token::Echo)?;
        let expr = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
        Self::consume_semicolon(tokens, position)?;
        Ok(Stmt::Echo(expr))
    }

    /// Parse print statement
    pub fn parse_print(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        Self::consume_token(tokens, position, Token::Print)?;
        let expr = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
        Self::consume_semicolon(tokens, position)?;
        Ok(Stmt::Print(expr))
    }

    /// Parse assignment or expression statement
    pub fn parse_assignment_or_expression(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        let checkpoint = *position;

        // Try to parse as assignment
        if let Some(Token::Variable(name)) = tokens.peek().cloned() {
            super::utils::ParserUtils::next_token(tokens, position);
            if let Some(Token::Equals) = tokens.peek() {
                super::utils::ParserUtils::next_token(tokens, position);
                let value = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
                Self::consume_semicolon(tokens, position)?;
                return Ok(Stmt::Assignment { variable: name, value });
            }
        }

        // Reset position and parse as expression
        *position = checkpoint;
        Self::parse_expression_statement(tokens, position)
    }

    /// Parse const statement
    pub fn parse_const(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        Self::consume_token(tokens, position, Token::Const)?;

        // Expect identifier for constant name
        let const_name = match super::utils::ParserUtils::next_token(tokens, position) {
            Some(Token::Identifier(name)) => name,
            Some(token) => {
                return Err(ParseError::ExpectedToken {
                    expected: "identifier".to_string(),
                    found: format!("{:?}", token),
                    position: *position,
                })
            }
            None => return Err(ParseError::UnexpectedEof),
        };

        // Expect equals sign
        Self::consume_token(tokens, position, Token::Equals)?;

        // Parse the value expression
        let value = super::expressions::ExpressionParser::parse_expression(tokens, position)?;

        // Expect semicolon
        Self::consume_semicolon(tokens, position)?;

        Ok(Stmt::ConstantDefinition {
            name: const_name,
            value,
        })
    }

    /// Parse expression statement
    pub fn parse_expression_statement(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        let expr = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
        Self::consume_semicolon(tokens, position)?;
        Ok(Stmt::Expression(expr))
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
