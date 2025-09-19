//! Expression parsing for PHP parser
//!
//! This module handles parsing of PHP expressions:
//! - Binary operations with precedence
//! - Primary expressions (literals, variables, constants)
//! - Function calls
//! - Parenthesized expressions

use crate::ast::{ArrayElement, BinaryOp, Expr};
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

    // Handle postfix-style array access chains or function calls followed by array access
    left = Self::parse_postfix_access(tokens, position, left)?;

        // Handle postfix operators (like $i++, $i--)
        left = Self::parse_postfix(tokens, position, left)?;

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
                Some(Token::NullCoalescing) => BinaryOp::Concatenate, // placeholder not real; null coalescing handled separately
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

        // Null coalescing (??) is right associative, low precedence. Handle here after other binary ops.
        while let Some(Token::NullCoalescing) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // consume '??'
            let rhs = Self::parse_expression_precedence(tokens, position, 0)?;
            // Represent null coalescing as a BinaryOp::Concatenate for now? Better: introduce dedicated Expr variant.
            // Add dedicated variant to AST (skipped for minimal change) -> use Binary with special op? We'll add new Expr::NullCoalesce
            // But AST currently lacks; implement new variant.
            left = Expr::NullCoalesce {
                left: Box::new(left),
                right: Box::new(rhs),
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
                    let call_expr = Expr::FunctionCall { name, args };
                    // Allow immediate array access after function call
                    let call_expr = Self::parse_postfix_access(tokens, position, call_expr)?;
                    Ok(call_expr)
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
            Some(Token::OpenBracket) => {
                // Parse array literal: [element1, element2, ...]
                Self::parse_array_literal(tokens, position)
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

    /// Parse array literal: [element1, element2, key => value, ...]
    fn parse_array_literal(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Expr> {
        let mut elements = Vec::new();

        // Check for empty array
        if let Some(&Token::CloseBracket) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // consume ']'
            return Ok(Expr::Array(elements));
        }

        loop {
            // Parse the value expression
            let value = Self::parse_expression(tokens, position)?;

            // Check if this is a key-value pair (key => value)
            let element = if let Some(&Token::Arrow) = tokens.peek() {
                super::utils::ParserUtils::next_token(tokens, position); // consume '=>'
                let key_value = Self::parse_expression(tokens, position)?;
                ArrayElement { key: Some(value), value: key_value }
            } else {
                ArrayElement { key: None, value }
            };

            elements.push(element);

            // Check for next element or end of array
            match tokens.peek() {
                Some(&Token::Comma) => {
                    super::utils::ParserUtils::next_token(tokens, position); // consume ','
                    // Allow trailing comma
                    if let Some(&Token::CloseBracket) = tokens.peek() {
                        break;
                    }
                }
                Some(&Token::CloseBracket) => break,
                _ => {
                    return Err(ParseError::ExpectedToken {
                        expected: "comma or close bracket".to_string(),
                        found: format!("{:?}", tokens.peek()),
                        position: *position,
                    });
                }
            }
        }

        // Consume closing bracket
        Self::consume_token(tokens, position, Token::CloseBracket)?;
        Ok(Expr::Array(elements))
    }

    /// Parse postfix operators (like $i++, $i--)
    fn parse_postfix(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
        expr: Expr,
    ) -> ParseResult<Expr> {
        match tokens.peek() {
            Some(Token::Increment) => {
                super::utils::ParserUtils::next_token(tokens, position);
                Ok(Expr::Unary {
                    op: crate::ast::UnaryOp::PostIncrement,
                    operand: Box::new(expr),
                })
            }
            Some(Token::Decrement) => {
                super::utils::ParserUtils::next_token(tokens, position);
                Ok(Expr::Unary {
                    op: crate::ast::UnaryOp::PostDecrement,
                    operand: Box::new(expr),
                })
            }
            _ => Ok(expr),
        }
    }

    /// Parse chained array access: expr[ index ] ...
    fn parse_postfix_access(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
        mut expr: Expr,
    ) -> ParseResult<Expr> {
        loop {
            match tokens.peek() {
                Some(Token::OpenBracket) => {
                    super::utils::ParserUtils::next_token(tokens, position); // consume '['
                    let index_expr = Self::parse_expression(tokens, position)?;
                    Self::consume_token(tokens, position, Token::CloseBracket)?;
                    expr = Expr::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index_expr),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }
}
