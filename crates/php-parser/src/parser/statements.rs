//! Statement parsing for PHP parser
//!
//! This module handles parsing of PHP statements:
//! - Echo and print statements
//! - Variable assignments
//! - Constant definitions
//! - Expression statements

use crate::ast::{Expr, Stmt};
use crate::ast::DestructTarget;
use crate::error::{ParseError, ParseResult};
use php_lexer::Token;
use std::iter::Peekable;
use std::vec::IntoIter;

/// Statement parsing functionality
pub struct StatementParser;

impl StatementParser {
    /// Parse a declare statement (limited support: declare(strict_types=1); ignored)
    pub fn parse_declare(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        Self::consume_token(tokens, position, Token::Declare)?;
        // Expect '('
        Self::consume_token(tokens, position, Token::OpenParen)?;
        // Consume identifier = expression pairs separated by commas until ')'
        loop {
            // key identifier
            match super::utils::ParserUtils::next_token(tokens, position) {
                Some(Token::Identifier(_)) => {},
                Some(tok) => return Err(ParseError::ExpectedToken { expected: "identifier".to_string(), found: format!("{:?}", tok), position: *position }),
                None => return Err(ParseError::UnexpectedEof),
            }
            // '='
            Self::consume_token(tokens, position, Token::Equals)?;
            // value expression (reuse existing expression parser)
            let _ = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
            // comma or ')'
            match tokens.peek() {
                Some(Token::Comma) => { super::utils::ParserUtils::next_token(tokens, position); },
                Some(Token::CloseParen) => { super::utils::ParserUtils::next_token(tokens, position); break; },
                other => return Err(ParseError::ExpectedToken { expected: ", or )".to_string(), found: format!("{:?}", other), position: *position }),
            }
        }
        // semicolon
        Self::consume_semicolon(tokens, position)?;
        // Represent as empty statement (no dedicated AST variant yet)
        Ok(Stmt::Block(vec![]))
    }
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
        // Detect destructuring assignment starting with '[' ... '] ='
        if let Some(Token::OpenBracket) = tokens.peek() {
            if let Ok(stmt) = Self::try_parse_destructuring(tokens, position) {
                return Ok(stmt);
            }
        }
        // Peek for variable-led assignment or increment/decrement without consuming unless confirmed
        if let Some(Token::Variable(var_name)) = tokens.peek().cloned() {
            // Clone iterator to inspect following token
            let mut la = tokens.clone();
            let _ = la.next(); // consume variable in lookahead
            match la.peek() {
                Some(Token::Equals) => {
                    // Commit: real consume variable and '=' path
                    super::utils::ParserUtils::next_token(tokens, position); // variable
                    super::utils::ParserUtils::next_token(tokens, position); // '='
                    if let Some(Token::Ampersand) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); }
                    let value = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
                    // Semicolon or tolerant heuristic
                    match tokens.peek() {
                        Some(Token::Semicolon) => { super::utils::ParserUtils::next_token(tokens, position); }
                        Some(Token::Variable(_)) | Some(Token::Function) | Some(Token::If) | Some(Token::Echo) | Some(Token::Return) | Some(Token::Const) | Some(Token::OpenBrace) => {}
                        _ => { Self::consume_semicolon(tokens, position)?; }
                    }
                    return Ok(Stmt::Assignment { variable: var_name, value });
                }
                Some(Token::Increment) => {
                    super::utils::ParserUtils::next_token(tokens, position); // variable
                    super::utils::ParserUtils::next_token(tokens, position); // ++
                    let expr = crate::ast::Expr::Unary { op: crate::ast::UnaryOp::PostIncrement, operand: Box::new(crate::ast::Expr::Variable(var_name)) };
                    Self::consume_semicolon(tokens, position)?;
                    return Ok(Stmt::Expression(expr));
                }
                Some(Token::Decrement) => {
                    super::utils::ParserUtils::next_token(tokens, position); // variable
                    super::utils::ParserUtils::next_token(tokens, position); // --
                    let expr = crate::ast::Expr::Unary { op: crate::ast::UnaryOp::PostDecrement, operand: Box::new(crate::ast::Expr::Variable(var_name)) };
                    Self::consume_semicolon(tokens, position)?;
                    return Ok(Stmt::Expression(expr));
                }
                // Compound assignment += (currently only supporting '+=' pattern as Plus followed by Equals)
                Some(Token::Plus) => {
                    // Look ahead one more to see if '=' follows
                    let mut la2 = la.clone();
                    la2.next(); // consume Plus in lookahead
                    if let Some(Token::Equals) = la2.peek() {
                        // consume real variable, plus, equals
                        super::utils::ParserUtils::next_token(tokens, position); // variable
                        super::utils::ParserUtils::next_token(tokens, position); // plus
                        super::utils::ParserUtils::next_token(tokens, position); // equals
                        let rhs = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
                        // Build value = var + rhs
                        let value_expr = Expr::Binary { left: Box::new(Expr::Variable(var_name.clone())), op: crate::ast::BinaryOp::Add, right: Box::new(rhs) };
                        match tokens.peek() {
                            Some(Token::Semicolon) => { super::utils::ParserUtils::next_token(tokens, position); }
                            _ => { Self::consume_semicolon(tokens, position)?; }
                        }
                        return Ok(Stmt::Assignment { variable: var_name, value: value_expr });
                    }
                }
                Some(Token::Dot) => {
                    // Look ahead for '=' to form '.='
                    let mut la2 = la.clone();
                    la2.next();
                    if let Some(Token::Equals) = la2.peek() {
                        super::utils::ParserUtils::next_token(tokens, position); // variable
                        super::utils::ParserUtils::next_token(tokens, position); // dot
                        super::utils::ParserUtils::next_token(tokens, position); // equals
                        let rhs = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
                        let value_expr = Expr::Binary { left: Box::new(Expr::Variable(var_name.clone())), op: crate::ast::BinaryOp::Concatenate, right: Box::new(rhs) };
                        match tokens.peek() {
                            Some(Token::Semicolon) => { super::utils::ParserUtils::next_token(tokens, position); }
                            _ => { Self::consume_semicolon(tokens, position)?; }
                        }
                        return Ok(Stmt::Assignment { variable: var_name, value: value_expr });
                    }
                }
                Some(Token::NullCoalescing) => {
                    // Look ahead for '=' to detect '??='
                    let mut la2 = la.clone();
                    la2.next(); // consume NullCoalescing in lookahead
                    if let Some(Token::Equals) = la2.peek() {
                        // consume actual tokens
                        super::utils::ParserUtils::next_token(tokens, position); // variable
                        super::utils::ParserUtils::next_token(tokens, position); // '??'
                        super::utils::ParserUtils::next_token(tokens, position); // '='
                        let rhs = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
                        match tokens.peek() {
                            Some(Token::Semicolon) => { super::utils::ParserUtils::next_token(tokens, position); }
                            _ => { Self::consume_semicolon(tokens, position)?; }
                        }
                        return Ok(Stmt::NullCoalesceAssign { variable: var_name, value: rhs });
                    }
                }
                _ => { /* fall through to generic expression parsing */ }
            }
        }

        // Not a variable token, parse as expression statement
        Self::parse_expression_statement(tokens, position)
    }

    /// Parse static variable declaration inside function: static $var = expr;
    pub fn parse_static(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        Self::consume_token(tokens, position, Token::Static)?;
        let var_name = match super::utils::ParserUtils::next_token(tokens, position) {
            Some(Token::Variable(n)) => n,
            other => return Err(ParseError::ExpectedToken { expected: "variable".into(), found: format!("{:?}", other), position: *position })
        };
        let mut initial: Option<Expr> = None;
        if let Some(Token::Equals) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // '='
            initial = Some(super::expressions::ExpressionParser::parse_expression(tokens, position)?);
        }
        Self::consume_semicolon(tokens, position)?;
        Ok(Stmt::StaticVar { name: var_name, initial })
    }

    /// Attempt to parse a destructuring assignment; on failure, restore iterator state by returning error
    fn try_parse_destructuring(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        let start_pos = *position;
        let mut clone = tokens.clone();
        let mut clone_pos = start_pos;
        // Consume '['
        match super::utils::ParserUtils::next_token(&mut clone, &mut clone_pos) {
            Some(Token::OpenBracket) => {}
            _ => return Err(ParseError::InvalidStatement { message: "not destructuring".into() }),
        }
        let mut targets = Vec::new();
    let expect_comma_or_close = false; // placeholder for future validation logic
        loop {
            match clone.peek() {
                Some(Token::CloseBracket) => {
                    super::utils::ParserUtils::next_token(&mut clone, &mut clone_pos); // consume ]
                    break;
                }
                None => return Err(ParseError::UnexpectedEof),
                _ => {
                    if expect_comma_or_close {
                        return Err(ParseError::InvalidStatement { message: "expected comma or ]".into() });
                    }
                    // Optional keyed form: String '=>' Variable
                    let mut key: Option<String> = None;
                    // Peek for string key
                    if let Some(Token::String(s)) = clone.peek().cloned() {
                        // Look ahead for =>
                        let mut la = clone.clone();
                        let _ = la.next(); // consume string in lookahead
                        if let Some(Token::Arrow) = la.peek() {
                            // consume string and => in real clone
                            super::utils::ParserUtils::next_token(&mut clone, &mut clone_pos); // string
                            super::utils::ParserUtils::next_token(&mut clone, &mut clone_pos); // =>
                            key = Some(s);
                        }
                    }
                    // Expect variable
                    match super::utils::ParserUtils::next_token(&mut clone, &mut clone_pos) {
                        Some(Token::Variable(var_name)) => {
                            if let Some(k) = key { targets.push(DestructTarget::KeyVar(k, var_name)); } else { targets.push(DestructTarget::Var(var_name)); }
                        }
                        other => return Err(ParseError::ExpectedToken { expected: "variable".into(), found: format!("{:?}", other), position: clone_pos }),
                    }
                    // Comma or close
                    match clone.peek() {
                        Some(Token::Comma) => { super::utils::ParserUtils::next_token(&mut clone, &mut clone_pos); }
                        Some(Token::CloseBracket) => {}
                        _ => {}
                    }
                }
            }
        }
        // Expect equals
        match clone.peek() {
            Some(Token::Equals) => { super::utils::ParserUtils::next_token(&mut clone, &mut clone_pos); }
            _ => return Err(ParseError::InvalidStatement { message: "missing = after destructuring pattern".into() }),
        }
        // Commit: replace original iterator by consuming the same tokens
        // Consume from original tokens now that we know it's destructuring
        // Simpler: re-parse pattern on original tokens
        // Consume '['
        super::utils::ParserUtils::next_token(tokens, position);
        let mut committed_targets = Vec::new();
        loop {
            if let Some(Token::CloseBracket) = tokens.peek() {
                super::utils::ParserUtils::next_token(tokens, position); // ]
                break;
            }
            let mut key: Option<String> = None;
            if let Some(Token::String(s)) = tokens.peek().cloned() {
                // lookahead for =>
                let mut la = tokens.clone();
                let _ = la.next();
                if let Some(Token::Arrow) = la.peek() {
                    super::utils::ParserUtils::next_token(tokens, position); // string
                    super::utils::ParserUtils::next_token(tokens, position); // =>
                    key = Some(s);
                }
            }
            let var_name = match super::utils::ParserUtils::next_token(tokens, position) {
                Some(Token::Variable(v)) => v,
                other => return Err(ParseError::ExpectedToken { expected: "variable".into(), found: format!("{:?}", other), position: *position }),
            };
            committed_targets.push(if let Some(k) = key { DestructTarget::KeyVar(k, var_name) } else { DestructTarget::Var(var_name) });
            if let Some(Token::Comma) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); }
        }
        // '='
        Self::consume_token(tokens, position, Token::Equals)?;
        let value_expr = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
        Self::consume_semicolon(tokens, position)?;
        Ok(Stmt::DestructuringAssignment { targets: committed_targets, value: value_expr })
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
                // Skip optional simple type hints (Identifier '|' Identifier ...)
                loop {
                    match tokens.peek() {
                        Some(Token::Identifier(_)) => { super::utils::ParserUtils::next_token(tokens, position); }
                        _ => break,
                    }
                    // Support union types: continue if next is pipe
                    if let Some(Token::Pipe) = tokens.peek() {
                        super::utils::ParserUtils::next_token(tokens, position); // consume pipe and loop
                        continue;
                    } else {
                        break;
                    }
                }
                // Variadic ellipsis '...'
                if let Some(Token::Ellipsis) = tokens.peek() {
                    super::utils::ParserUtils::next_token(tokens, position); // consume ellipsis (ignored semantics)
                }
                // Optional by-reference '&'
                if let Some(Token::Ampersand) = tokens.peek() {
                    super::utils::ParserUtils::next_token(tokens, position); // consume '&'
                }
                // Now expect parameter variable
                let param_name = match super::utils::ParserUtils::next_token(tokens, position) {
                    Some(Token::Variable(name)) => name,
                    Some(other) => return Err(ParseError::ExpectedToken {
                        expected: "parameter variable".to_string(),
                        found: format!("{:?}", other),
                        position: *position,
                    }),
                    None => return Err(ParseError::UnexpectedEof),
                };
                // Optional default value assignment: = expr (ignored for now; just consume tokens)
                if let Some(Token::Equals) = tokens.peek() {
                    super::utils::ParserUtils::next_token(tokens, position); // consume '='
                    // Parse and discard expression
                    let _default_expr = super::expressions::ExpressionParser::parse_expression(tokens, position)?;
                }
                parameters.push(param_name);

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
        // Optional return type hint: ':' type1 '|' type2 ... (skip for now)
        if let Some(Token::Colon) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // consume ':'
            // Consume one or more identifiers separated by pipes
            loop {
                match tokens.peek() {
                    Some(Token::Identifier(_)) => { super::utils::ParserUtils::next_token(tokens, position); }
                    _ => break,
                }
                if let Some(Token::Pipe) = tokens.peek() {
                    super::utils::ParserUtils::next_token(tokens, position); // consume '|'
                    continue;
                } else {
                    break;
                }
            }
        }

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
