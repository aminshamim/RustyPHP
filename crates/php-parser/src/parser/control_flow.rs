//! Control flow parsing for PHP parser
//!
//! This module handles parsing of PHP control flow statements:
//! - If/else statements
//! - While loops
//! - For loops
//! - Break and continue statements
//! - Return statements

use crate::ast::{Stmt};
use crate::ast::SwitchCase;
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

        let else_stmt = match tokens.peek() {
            Some(Token::ElseIf) => {
                // Parse elseif as a nested if statement
                super::utils::ParserUtils::next_token(tokens, position); // consume elseif
                Self::consume_token(tokens, position, Token::OpenParen)?;
                let condition = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
                Self::consume_token(tokens, position, Token::CloseParen)?;
                let then_stmt = Box::new(super::main::Parser::parse_statement_with_tokens(tokens, position)?);

                // Recursively handle more elseif/else clauses
                let else_stmt = match tokens.peek() {
                    Some(Token::ElseIf) => {
                        Some(Box::new(Self::parse_if(tokens, position)?))
                    }
                    Some(Token::Else) => {
                        super::utils::ParserUtils::next_token(tokens, position);
                        Some(Box::new(super::main::Parser::parse_statement_with_tokens(tokens, position)?))
                    }
                    _ => None
                };

                Some(Box::new(Stmt::If {
                    condition,
                    then_stmt,
                    else_stmt,
                }))
            }
            Some(Token::Else) => {
                super::utils::ParserUtils::next_token(tokens, position);
                Some(Box::new(super::main::Parser::parse_statement_with_tokens(tokens, position)?))
            }
            _ => None
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

    /// Parse foreach statement: foreach ($array as $item) { ... } or foreach ($array as $key => $value) { ... }
    pub fn parse_foreach(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        // Consume 'foreach'
        Self::consume_token(tokens, position, Token::Foreach)?;

        // Consume '('
        Self::consume_token(tokens, position, Token::OpenParen)?;

        // Parse the array expression
        let array = super::expressions::ExpressionParser::parse_expression(tokens, position)?;

        // Consume 'as'
        Self::consume_token(tokens, position, Token::As)?;

        // Parse the variable(s)
        let mut key_var = None;
        let value_var;

        // First variable
        let first_var = match super::utils::ParserUtils::next_token(tokens, position) {
            Some(Token::Variable(name)) => name,
            Some(token) => {
                return Err(ParseError::ExpectedToken {
                    expected: "variable".to_string(),
                    found: format!("{:?}", token),
                    position: *position,
                })
            }
            None => return Err(ParseError::UnexpectedEof),
        };

        // Check if there's an arrow (key => value syntax)
        if let Some(Token::Arrow) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // consume '=>'
            key_var = Some(first_var);
            
            // Parse the value variable
            value_var = match super::utils::ParserUtils::next_token(tokens, position) {
                Some(Token::Variable(name)) => name,
                Some(token) => {
                    return Err(ParseError::ExpectedToken {
                        expected: "variable".to_string(),
                        found: format!("{:?}", token),
                        position: *position,
                    })
                }
                None => return Err(ParseError::UnexpectedEof),
            };
        } else {
            // Just value variable
            value_var = first_var;
        }

        // Consume ')'
        Self::consume_token(tokens, position, Token::CloseParen)?;

        // Parse the body
        let body = Box::new(super::main::Parser::parse_statement_with_tokens(tokens, position)?);

        Ok(Stmt::Foreach {
            array,
            value_var,
            key_var,
            body,
        })
    }

    /// Parse switch statement
    pub fn parse_switch(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        Self::consume_token(tokens, position, Token::Switch)?;
        Self::consume_token(tokens, position, Token::OpenParen)?;
        let expr = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
        Self::consume_token(tokens, position, Token::CloseParen)?;
        Self::consume_token(tokens, position, Token::OpenBrace)?;

        let mut cases: Vec<SwitchCase> = Vec::new();
        let mut default_block: Option<Vec<Stmt>> = None;

        while let Some(token) = tokens.peek() {
            match token {
                Token::Case => {
                    super::utils::ParserUtils::next_token(tokens, position); // consume 'case'
                    let value_expr = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
                    Self::consume_token(tokens, position, Token::Colon)?;
                    let mut stmts = Vec::new();
                    // Collect statements until break / next case / default / close brace
                    loop {
                        match tokens.peek() {
                            Some(Token::Break) => {
                                // consume break and its semicolon
                                Self::consume_token(tokens, position, Token::Break)?;
                                if let Err(e) = Self::consume_semicolon(tokens, position) { return Err(e); }
                                break;
                            }
                            Some(Token::Case) | Some(Token::Default) | Some(Token::CloseBrace) => break,
                            Some(_) => {
                                let stmt = super::main::Parser::parse_statement_with_tokens(tokens, position)?;
                                stmts.push(stmt);
                            }
                            None => return Err(ParseError::UnexpectedEof),
                        }
                    }
                    cases.push(SwitchCase { value: value_expr, statements: stmts });
                }
                Token::Default => {
                    super::utils::ParserUtils::next_token(tokens, position); // consume 'default'
                    Self::consume_token(tokens, position, Token::Colon)?;
                    let mut stmts = Vec::new();
                    loop {
                        match tokens.peek() {
                            Some(Token::Break) => {
                                Self::consume_token(tokens, position, Token::Break)?;
                                if let Err(e) = Self::consume_semicolon(tokens, position) { return Err(e); }
                                break;
                            }
                            Some(Token::CloseBrace) => break,
                            Some(_) => {
                                let stmt = super::main::Parser::parse_statement_with_tokens(tokens, position)?;
                                stmts.push(stmt);
                            }
                            None => return Err(ParseError::UnexpectedEof),
                        }
                    }
                    default_block = Some(stmts);
                }
                Token::CloseBrace => {
                    break;
                }
                _ => return Err(ParseError::UnexpectedToken { token: format!("{:?}", token), position: *position }),
            }
        }

        Self::consume_token(tokens, position, Token::CloseBrace)?;

        Ok(Stmt::Switch { expression: expr, cases, default: default_block })
    }
}
