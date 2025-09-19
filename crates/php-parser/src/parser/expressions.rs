//! Expression parsing for PHP parser
//!
//! This module handles parsing of PHP expressions:
//! - Binary operations with precedence
//! - Primary expressions (literals, variables, constants)
//! - Function calls
//! - Parenthesized expressions

use crate::ast::{BinaryOp, Expr};
use crate::error::{ParseError, ParseResult};
use php_lexer::Token;
use std::iter::Peekable;
use std::vec::IntoIter;

/// Expression parsing functionality
pub struct ExpressionParser;

impl ExpressionParser {
    /// Parse expression
    pub fn parse_expression(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Expr> {
        Self::parse_expression_precedence(tokens, position, 0)
    }

    /// Parse expression with precedence climbing
    fn parse_expression_precedence(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
        min_precedence: u8,
    ) -> ParseResult<Expr> {
        let mut left = Self::parse_primary(tokens, position)?;

        loop {
            let op = match tokens.peek() {
                Some(Token::Plus) => BinaryOp::Add,
                Some(Token::Minus) => BinaryOp::Subtract,
                Some(Token::Multiply) => BinaryOp::Multiply,
                Some(Token::Divide) => BinaryOp::Divide,
                Some(Token::Dot) => BinaryOp::Concatenate,
                Some(Token::DoubleEquals) => BinaryOp::Equal,
                Some(Token::NotEquals) => BinaryOp::NotEqual,
                Some(Token::LessThan) => BinaryOp::LessThan,
                Some(Token::GreaterThan) => BinaryOp::GreaterThan,
                Some(Token::LessOrEqual) => BinaryOp::LessThanOrEqual,
                Some(Token::GreaterOrEqual) => BinaryOp::GreaterThanOrEqual,
                _ => break,
            };

            let precedence = Self::get_precedence(&op);
            if precedence < min_precedence {
                break;
            }

            super::utils::ParserUtils::next_token(tokens, position);

            let right = Self::parse_expression_precedence(tokens, position, precedence + 1)?;

            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse primary expression
    fn parse_primary(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Expr> {
        match super::utils::ParserUtils::next_token(tokens, position) {
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::String(s)) => Ok(Expr::String(s)),
            Some(Token::Variable(name)) => Ok(Expr::Variable(name)),
            Some(Token::Identifier(name)) => {
                // Check if this is a function call (identifier followed by opening parenthesis)
                if let Some(&Token::OpenParen) = tokens.peek() {
                    super::utils::ParserUtils::next_token(tokens, position); // consume opening parenthesis
                    let args = Self::parse_function_args(tokens, position)?;
                    Self::consume_token(tokens, position, Token::CloseParen)?;
                    Ok(Expr::FunctionCall { name, args })
                } else {
                    // It's a constant reference
                    Ok(Expr::Constant(name))
                }
            }
            Some(Token::True) => Ok(Expr::Bool(true)),
            Some(Token::False) => Ok(Expr::Bool(false)),
            Some(Token::Null) => Ok(Expr::Null),
            Some(Token::OpenParen) => {
                let expr = Self::parse_expression(tokens, position)?;
                Self::consume_token(tokens, position, Token::CloseParen)?;
                Ok(expr)
            }
            Some(token) => Err(ParseError::UnexpectedToken {
                token: format!("{:?}", token),
                position: *position,
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    /// Parse function arguments
    fn parse_function_args(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Vec<Expr>> {
        let mut args = Vec::new();

        // Check for empty argument list
        if let Some(&Token::CloseParen) = tokens.peek() {
            return Ok(args);
        }

        // Parse first argument
        args.push(Self::parse_expression(tokens, position)?);

        // Parse remaining arguments
        while let Some(&Token::Comma) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // consume comma
            args.push(Self::parse_expression(tokens, position)?);
        }

        Ok(args)
    }

    /// Get operator precedence
    fn get_precedence(op: &BinaryOp) -> u8 {
        match op {
            BinaryOp::LogicalOr => 0,
            BinaryOp::LogicalAnd => 1,
            BinaryOp::Equal | BinaryOp::NotEqual => 2,
            BinaryOp::LessThan
            | BinaryOp::GreaterThan
            | BinaryOp::LessThanOrEqual
            | BinaryOp::GreaterThanOrEqual => 3,
            BinaryOp::Concatenate => 4,
            BinaryOp::Add | BinaryOp::Subtract => 5,
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 6,
        }
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
}
