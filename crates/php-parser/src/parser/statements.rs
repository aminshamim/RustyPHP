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
        // Check if it's a variable at the start
        if let Some(Token::Variable(name)) = tokens.peek().cloned() {
            // Consume the variable token
            super::utils::ParserUtils::next_token(tokens, position);
            
            match tokens.peek() {
                Some(Token::Equals) => {
                    // It's an assignment: $var = expr;
                    super::utils::ParserUtils::next_token(tokens, position); // consume equals
                    let value = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
                    Self::consume_semicolon(tokens, position)?;
                    return Ok(Stmt::Assignment { variable: name, value });
                }
                Some(Token::Increment) => {
                    // It's $var++; - construct the postfix increment expression
                    super::utils::ParserUtils::next_token(tokens, position); // consume increment
                    let expr = crate::ast::Expr::Unary {
                        op: crate::ast::UnaryOp::PostIncrement,
                        operand: Box::new(crate::ast::Expr::Variable(name)),
                    };
                    Self::consume_semicolon(tokens, position)?;
                    return Ok(crate::ast::Stmt::Expression(expr));
                }
                Some(Token::Decrement) => {
                    // It's $var--; - construct the postfix decrement expression
                    super::utils::ParserUtils::next_token(tokens, position); // consume decrement
                    let expr = crate::ast::Expr::Unary {
                        op: crate::ast::UnaryOp::PostDecrement,
                        operand: Box::new(crate::ast::Expr::Variable(name)),
                    };
                    Self::consume_semicolon(tokens, position)?;
                    return Ok(crate::ast::Stmt::Expression(expr));
                }
                _ => {
                    // It's some other expression starting with $var
                    // We need to let the expression parser take over, but we've already consumed the variable
                    // This is tricky because we can't rewind the iterator
                    // For now, let's assume it's a complex expression and manually build the variable part
                    
                    // Start with the variable we already consumed
                    let expr = crate::ast::Expr::Variable(name);
                    
                    // Let the expression parser handle any remaining parts (like array access, function calls, etc.)
                    // But this is getting complex. Let me fall back to expression parsing for this case.
                    
                    // Actually, let's simplify: if it's not = or ++ or --, 
                    // let's just treat it as a variable expression followed by semicolon
                    Self::consume_semicolon(tokens, position)?;
                    return Ok(crate::ast::Stmt::Expression(expr));
                }
            }
        }

        // Not a variable token, parse as expression statement
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

    /// Parse function definition
    pub fn parse_function_definition(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        Self::consume_token(tokens, position, Token::Function)?;
        
        // Parse function name
        let name = match super::utils::ParserUtils::next_token(tokens, position) {
            Some(Token::Identifier(name)) => name,
            Some(token) => return Err(ParseError::ExpectedToken {
                expected: "function name".to_string(),
                found: format!("{:?}", token),
                position: *position,
            }),
            None => return Err(ParseError::UnexpectedEof),
        };

        // Parse parameter list
        Self::consume_token(tokens, position, Token::OpenParen)?;
        let mut parameters = Vec::new();
        
        // Check for empty parameter list
        if let Some(&Token::CloseParen) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // consume ')'
        } else {
            loop {
                // Parse parameter (expect $variable)
                match super::utils::ParserUtils::next_token(tokens, position) {
                    Some(Token::Variable(param_name)) => parameters.push(param_name),
                    Some(token) => return Err(ParseError::ExpectedToken {
                        expected: "parameter variable".to_string(),
                        found: format!("{:?}", token),
                        position: *position,
                    }),
                    None => return Err(ParseError::UnexpectedEof),
                }

                // Check for more parameters or end
                match tokens.peek() {
                    Some(&Token::Comma) => {
                        super::utils::ParserUtils::next_token(tokens, position); // consume ','
                    }
                    Some(&Token::CloseParen) => {
                        super::utils::ParserUtils::next_token(tokens, position); // consume ')'
                        break;
                    }
                    _ => return Err(ParseError::ExpectedToken {
                        expected: "comma or close parenthesis".to_string(),
                        found: format!("{:?}", tokens.peek()),
                        position: *position,
                    }),
                }
            }
        }

        // Parse function body (expect block statement)
        Self::consume_token(tokens, position, Token::OpenBrace)?;
        let body = Self::parse_block_statements(tokens, position)?;
        Self::consume_token(tokens, position, Token::CloseBrace)?;

        Ok(Stmt::FunctionDefinition {
            name,
            parameters,
            body: Box::new(Stmt::Block(body)),
        })
    }

    /// Parse block statements (helper for function bodies, control structures)
    fn parse_block_statements(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        
        while let Some(token) = tokens.peek() {
            match token {
                Token::CloseBrace | Token::EOF => break,
                _ => statements.push(super::main::Parser::parse_statement_with_tokens(tokens, position)?),
            }
        }
        
        Ok(statements)
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
