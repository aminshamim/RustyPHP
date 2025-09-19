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
                Some(Token::Spaceship) => BinaryOp::Spaceship,
                Some(Token::Ampersand) => BinaryOp::BitwiseAnd,
                Some(Token::Pipe) => BinaryOp::BitwiseOr,
                Some(Token::LogicalAnd) => BinaryOp::LogicalAnd,
                Some(Token::LogicalOr) => BinaryOp::LogicalOr,
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

        // Ternary operator: condition ? then : else  (with shorthand condition ?: else)
        if let Some(Token::QuestionMark) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // consume '?'
            let then_part = if let Some(Token::Colon) = tokens.peek() {
                // Shorthand form 'expr ?: else'
                None
            } else {
                Some(Box::new(Self::parse_expression_precedence(tokens, position, 0)?))
            };
            // Expect ':'
            Self::consume_token(tokens, position, Token::Colon)?;
            let else_expr = Self::parse_expression_precedence(tokens, position, 0)?;
            left = Expr::Ternary {
                condition: Box::new(left),
                then_expr: then_part,
                else_expr: Box::new(else_expr),
            };
        }

        // match expression: match (expr) { condList => result, default => result }
        if let Some(Token::Identifier(id)) = tokens.peek().cloned() {
            if id == "match" {
                super::utils::ParserUtils::next_token(tokens, position); // 'match'
                Self::consume_token(tokens, position, Token::OpenParen)?;
                let subject = Self::parse_expression(tokens, position)?;
                Self::consume_token(tokens, position, Token::CloseParen)?;
                Self::consume_token(tokens, position, Token::OpenBrace)?;
                let mut arms: Vec<(Vec<Expr>, Box<Expr>)> = Vec::new();
                let mut default_arm: Option<Box<Expr>> = None;
                while let Some(tok) = tokens.peek() {
                    if matches!(tok, Token::CloseBrace) { break; }
                    // default arm (Token::Default or identifier "default")
                    match tokens.peek().cloned() {
                        Some(Token::Default) => {
                            super::utils::ParserUtils::next_token(tokens, position); // default
                            Self::consume_token(tokens, position, Token::Arrow)?;
                            let result_expr = Self::parse_expression(tokens, position)?;
                            default_arm = Some(Box::new(result_expr));
                            if let Some(Token::Comma) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); }
                            continue;
                        }
                        Some(Token::Identifier(d)) if d == "default" => {
                            super::utils::ParserUtils::next_token(tokens, position);
                            Self::consume_token(tokens, position, Token::Arrow)?;
                            let result_expr = Self::parse_expression(tokens, position)?;
                            default_arm = Some(Box::new(result_expr));
                            if let Some(Token::Comma) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); }
                            continue;
                        }
                        _ => {}
                    }
                    // parse one or more conditions separated by commas until '=>'
                    let mut conds = Vec::new();
                    loop {
                        let cond_expr = Self::parse_expression(tokens, position)?;
                        conds.push(cond_expr);
                        if let Some(Token::Comma) = tokens.peek() { // could be separator between conditions or end of arm
                            // lookahead to see if Arrow follows next
                            let mut la = tokens.clone();
                            la.next(); // consume comma in lookahead
                            if let Some(Token::Arrow) = la.peek() {
                                super::utils::ParserUtils::next_token(tokens, position); // consume comma and break
                                break;
                            } else {
                                super::utils::ParserUtils::next_token(tokens, position); // consume comma continue
                                continue;
                            }
                        }
                        break;
                    }
                    Self::consume_token(tokens, position, Token::Arrow)?;
                    let result_expr = Self::parse_expression(tokens, position)?;
                    arms.push((conds, Box::new(result_expr)));
                    if let Some(Token::Comma) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); }
                }
                Self::consume_token(tokens, position, Token::CloseBrace)?;
                return Ok(Expr::Match { subject: Box::new(subject), arms, default_arm });
            }
        }
        Ok(left)
    }

    /// Parse primary expression
    fn parse_primary(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Expr> {
        // Transparent reference prefix '&' (ignored semantics for now)
        if let Some(Token::Ampersand) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // consume '&'
            // Parse the next primary and return directly (no reference semantics implemented)
            return Self::parse_primary(tokens, position);
        }
        // Prefix increment/decrement
        if let Some(Token::Increment) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // '++'
            let operand = Self::parse_primary(tokens, position)?;
            return Ok(Expr::Unary { op: crate::ast::UnaryOp::PreIncrement, operand: Box::new(operand) });
        }
        if let Some(Token::Decrement) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // '--'
            let operand = Self::parse_primary(tokens, position)?;
            return Ok(Expr::Unary { op: crate::ast::UnaryOp::PreDecrement, operand: Box::new(operand) });
        }
        // Anonymous function: function (...) { ... }
        if let Some(Token::Function) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // consume 'function'
            // Optional name (if present treat as normal function? we only allow anonymous here, so if Identifier next and then '(' treat as named fallback error)
            if let Some(Token::Identifier(_)) = tokens.peek() {
                // Fallback: not anonymous, push error to caller
            }
            // Parameter list
            Self::consume_token(tokens, position, Token::OpenParen)?;
            let mut params = Vec::new();
            if let Some(Token::CloseParen) = tokens.peek() {
                super::utils::ParserUtils::next_token(tokens, position); // empty param list
            } else {
                loop {
                    // Skip type hints (identifiers + pipes)
                    loop { match tokens.peek() { Some(Token::Identifier(_)) => { super::utils::ParserUtils::next_token(tokens, position); }, _ => break } if let Some(Token::Pipe) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); continue; } else { break; } }
                    if let Some(Token::Ellipsis) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); }
                    // Optional by-reference '&'
                    if let Some(Token::Ampersand) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); }
                    let var_name = match super::utils::ParserUtils::next_token(tokens, position) { Some(Token::Variable(v)) => v, other => return Err(ParseError::ExpectedToken { expected: "parameter variable".into(), found: format!("{:?}", other), position: *position }) };
                    if let Some(Token::Equals) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); let _ = Self::parse_expression(tokens, position)?; }
                    params.push(var_name);
                    match tokens.peek() { Some(Token::Comma) => { super::utils::ParserUtils::next_token(tokens, position); }, Some(Token::CloseParen) => { super::utils::ParserUtils::next_token(tokens, position); break; }, other => return Err(ParseError::ExpectedToken { expected: ", or )".into(), found: format!("{:?}", other), position: *position }) }
                }
            }
            // Optional return type: ':' identifiers and pipes
            if let Some(Token::Colon) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); while let Some(Token::Identifier(_)) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); if let Some(Token::Pipe) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); continue; } break; } }
            // Body block
            Self::consume_token(tokens, position, Token::OpenBrace)?;
            // Collect statements until closing brace, but we simplify: parse as block, then wrap in implicit return of last expression if it is an expression statement
            let mut body_stmts = Vec::new();
            while let Some(tk) = tokens.peek() { if matches!(tk, Token::CloseBrace) { break; } body_stmts.push(super::main::Parser::parse_statement_with_tokens(tokens, position)?); }
            Self::consume_token(tokens, position, Token::CloseBrace)?;
            // Heuristic: if last statement is Expression(e) produce closure body expression = e; else null
            let body_expr = if let Some(last) = body_stmts.last() { if let crate::ast::Stmt::Expression(e) = last { e.clone() } else { Expr::Null } } else { Expr::Null };
            return Ok(Expr::ArrowFunction { params, body: Box::new(body_expr) });
        }
        // Arrow function start: identifier 'fn'
        if let Some(Token::Identifier(name)) = tokens.peek().cloned() {
            if name == "fn" {
                super::utils::ParserUtils::next_token(tokens, position); // consume 'fn'
                // Expect '('
                Self::consume_token(tokens, position, Token::OpenParen)?;
                let mut params = Vec::new();
                // Parse param list (possibly empty) skipping type hints (identifiers and pipes) until variable appears
                if let Some(token) = tokens.peek() {
                    if let Token::CloseParen = token { super::utils::ParserUtils::next_token(tokens, position); } else {
                        loop {
                            // Skip simple type hints (Identifier ('|' Identifier)*)
                            loop {
                                match tokens.peek() {
                                    Some(Token::Identifier(_)) => { super::utils::ParserUtils::next_token(tokens, position); }
                                    _ => break,
                                }
                                if let Some(Token::Pipe) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); continue; } else { break; }
                            }
                            // Variadic/spread ellipsis (ignored semantics)
                            if let Some(Token::Ellipsis) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); }
                            // Optional by-reference '&'
                            if let Some(Token::Ampersand) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); }
                            // Expect variable name
                            let var_name = match super::utils::ParserUtils::next_token(tokens, position) {
                                Some(Token::Variable(v)) => v,
                                other => return Err(ParseError::ExpectedToken { expected: "parameter variable".into(), found: format!("{:?}", other), position: *position }),
                            };
                            // Optional default value assign skip: '=' expr
                            if let Some(Token::Equals) = tokens.peek() {
                                super::utils::ParserUtils::next_token(tokens, position);
                                let _ = Self::parse_expression(tokens, position)?; // discard
                            }
                            params.push(var_name);
                            match tokens.peek() {
                                Some(Token::Comma) => { super::utils::ParserUtils::next_token(tokens, position); continue; }
                                Some(Token::CloseParen) => { super::utils::ParserUtils::next_token(tokens, position); break; }
                                other => return Err(ParseError::ExpectedToken { expected: ", or )".into(), found: format!("{:?}", other), position: *position }),
                            }
                        }
                    }
                }
                // Expect => (represented as Arrow token? we currently have Token::Arrow for '=>')
                Self::consume_token(tokens, position, Token::Arrow)?;
                let body = Self::parse_expression(tokens, position)?;
                return Ok(Expr::ArrowFunction { params, body: Box::new(body) });
            }
        }
        // Match expression starting directly (e.g., = match (...){...};)
        if let Some(Token::Identifier(name)) = tokens.peek().cloned() {
            if name == "match" {
                super::utils::ParserUtils::next_token(tokens, position); // 'match'
                Self::consume_token(tokens, position, Token::OpenParen)?;
                let subject = Self::parse_expression(tokens, position)?;
                Self::consume_token(tokens, position, Token::CloseParen)?;
                Self::consume_token(tokens, position, Token::OpenBrace)?;
                let mut arms: Vec<(Vec<Expr>, Box<Expr>)> = Vec::new();
                let mut default_arm: Option<Box<Expr>> = None;
                while let Some(tok) = tokens.peek() {
                    if matches!(tok, Token::CloseBrace) { break; }
                    match tokens.peek().cloned() {
                        Some(Token::Default) => {
                            super::utils::ParserUtils::next_token(tokens, position);
                            Self::consume_token(tokens, position, Token::Arrow)?;
                            let res = Self::parse_expression(tokens, position)?;
                            default_arm = Some(Box::new(res));
                            if let Some(Token::Comma) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); }
                            continue;
                        }
                        Some(Token::Identifier(d)) if d == "default" => {
                            super::utils::ParserUtils::next_token(tokens, position);
                            Self::consume_token(tokens, position, Token::Arrow)?;
                            let res = Self::parse_expression(tokens, position)?;
                            default_arm = Some(Box::new(res));
                            if let Some(Token::Comma) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); }
                            continue;
                        }
                        _ => {}
                    }
                    let mut conds = Vec::new();
                    loop {
                        let cond_expr = Self::parse_expression(tokens, position)?;
                        conds.push(cond_expr);
                        if let Some(Token::Comma) = tokens.peek() { // lookahead for arrow after comma
                            let mut la = tokens.clone(); la.next(); if let Some(Token::Arrow) = la.peek() { super::utils::ParserUtils::next_token(tokens, position); break; } else { super::utils::ParserUtils::next_token(tokens, position); continue; } }
                        break;
                    }
                    Self::consume_token(tokens, position, Token::Arrow)?;
                    let result_expr = Self::parse_expression(tokens, position)?;
                    arms.push((conds, Box::new(result_expr)));
                    if let Some(Token::Comma) = tokens.peek() { super::utils::ParserUtils::next_token(tokens, position); }
                }
                Self::consume_token(tokens, position, Token::CloseBrace)?;
                return Ok(Expr::Match { subject: Box::new(subject), arms, default_arm });
            }
        }
        // Yield expression (identifier 'yield' not a keyword yet)
        if let Some(Token::Identifier(name)) = tokens.peek().cloned() {
            if name == "yield" {
                super::utils::ParserUtils::next_token(tokens, position); // 'yield'
                // Optional 'from'
                if let Some(Token::Identifier(n2)) = tokens.peek().cloned() { if n2 == "from" { super::utils::ParserUtils::next_token(tokens, position); } }
                let inner = Self::parse_expression(tokens, position)?; // value expression
                return Ok(Expr::Yield { value: Box::new(inner) });
            }
        }
        match super::utils::ParserUtils::next_token(tokens, position) {
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::String(s)) => Ok(Expr::String(s)),
            Some(Token::Variable(name)) => {
                // Pattern: $var(...)
                if let Some(Token::OpenParen) = tokens.peek() {
                    let mut clone_iter = tokens.clone();
                    let open = clone_iter.next();
                    let maybe_ellipsis = clone_iter.next();
                    let maybe_close = clone_iter.next();
                    if matches!(open, Some(Token::OpenParen)) && matches!(maybe_ellipsis, Some(Token::Ellipsis)) && matches!(maybe_close, Some(Token::CloseParen)) {
                        // consume actual tokens
                        super::utils::ParserUtils::next_token(tokens, position); // '('
                        super::utils::ParserUtils::next_token(tokens, position); // '...'
                        super::utils::ParserUtils::next_token(tokens, position); // ')'
                        return Ok(Expr::Variable(name));
                    }
                    // Dynamic call: $var(...args...)
                    if let Some(Token::OpenParen) = tokens.peek() {
                        super::utils::ParserUtils::next_token(tokens, position); // consume '('
                        let args = Self::parse_function_args(tokens, position)?;
                        Self::consume_token(tokens, position, Token::CloseParen)?;
                        let call_expr = Expr::DynamicCall { target: Box::new(Expr::Variable(name.clone())), args };
                        let call_expr = Self::parse_postfix_access(tokens, position, call_expr)?;
                        return Ok(call_expr);
                    }
                }
                Ok(Expr::Variable(name))
            }
            // Built-in function tokens: convert to identifier name for uniform handling
            Some(Token::ArrayMerge) => Self::parse_builtin_as_call("array_merge".to_string(), tokens, position),
            Some(Token::ArrayPush) => Self::parse_builtin_as_call("array_push".to_string(), tokens, position),
            Some(Token::ArrayPop) => Self::parse_builtin_as_call("array_pop".to_string(), tokens, position),
            Some(Token::Count) => Self::parse_builtin_as_call("count".to_string(), tokens, position),
            Some(Token::Explode) => Self::parse_builtin_as_call("explode".to_string(), tokens, position),
            Some(Token::Implode) => Self::parse_builtin_as_call("implode".to_string(), tokens, position),
            Some(Token::PrintR) => Self::parse_builtin_as_call("print_r".to_string(), tokens, position),
            Some(Token::Strlen) => Self::parse_builtin_as_call("strlen".to_string(), tokens, position),
            Some(Token::Strpos) => Self::parse_builtin_as_call("strpos".to_string(), tokens, position),
            Some(Token::Substr) => Self::parse_builtin_as_call("substr".to_string(), tokens, position),
            Some(Token::Isset) => Self::parse_builtin_as_call("isset".to_string(), tokens, position),
            Some(Token::Identifier(name)) => {
                if name == "array" {
                    // Legacy array() constructor
                    if let Some(Token::OpenParen) = tokens.peek() {
                        super::utils::ParserUtils::next_token(tokens, position); // '('
                        let mut elements = Vec::new();
                        if let Some(Token::CloseParen) = tokens.peek() {
                            super::utils::ParserUtils::next_token(tokens, position); // empty )
                            return Ok(Expr::Array(elements));
                        }
                        loop {
                            // Parse key or value expression
                            let first_expr = Self::parse_expression(tokens, position)?;
                            let element = if let Some(Token::Arrow) = tokens.peek() {
                                super::utils::ParserUtils::next_token(tokens, position); // '=>'
                                let val_expr = Self::parse_expression(tokens, position)?;
                                crate::ast::ArrayElement { key: Some(first_expr), value: val_expr }
                            } else {
                                crate::ast::ArrayElement { key: None, value: first_expr }
                            };
                            elements.push(element);
                            match tokens.peek() {
                                Some(Token::Comma) => { super::utils::ParserUtils::next_token(tokens, position); }
                                Some(Token::CloseParen) => { super::utils::ParserUtils::next_token(tokens, position); break; }
                                other => return Err(ParseError::ExpectedToken { expected: ", or )".into(), found: format!("{:?}", other), position: *position }),
                            }
                        }
                        return Ok(Expr::Array(elements));
                    }
                }
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
                // Look ahead for possible cast pattern: (Identifier) followed by expression
                if let Some(Token::Identifier(cast_name)) = tokens.peek().cloned() {
                    // Simple list of primitive casts we accept
                    let primitive = matches!(cast_name.as_str(), "int" | "float" | "string" | "bool" | "boolean");
                    if primitive {
                        // consume identifier and closing paren, then parse the target expression
                        super::utils::ParserUtils::next_token(tokens, position); // cast type
                        Self::consume_token(tokens, position, Token::CloseParen)?;
                        // After cast, allow immediate opening paren or primary expression
                        let inner = Self::parse_expression(tokens, position)?;
                        return Ok(inner); // ignore cast semantics for now
                    }
                }
                // Regular parenthesized expression
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

    /// Helper to parse a built-in function token as potential function call or constant
    fn parse_builtin_as_call(
        name: String,
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Expr> {
        if let Some(&Token::OpenParen) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // consume '('
            let args = Self::parse_function_args(tokens, position)?;
            Self::consume_token(tokens, position, Token::CloseParen)?;
            let call_expr = Expr::FunctionCall { name, args };
            let call_expr = Self::parse_postfix_access(tokens, position, call_expr)?;
            Ok(call_expr)
        } else {
            Ok(Expr::Constant(name))
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

        // Parse first argument (support named args: Identifier ':' expr)
    args.push(Self::parse_named_or_positional_arg(tokens, position)?);

        // Parse remaining arguments
        while let Some(&Token::Comma) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // consume comma
            args.push(Self::parse_named_or_positional_arg(tokens, position)?);
        }

        Ok(args)
    }

    /// Parse either a named argument (Identifier ':' expr) or a standard expression argument.
    fn parse_named_or_positional_arg(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Expr> {
        // Skip spread ellipsis (ignored semantics)
        if let Some(Token::Ellipsis) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position); // '...'
        }
        if let Some(Token::Identifier(_)) = tokens.peek() {
            // Clone iterator to inspect following token
            let mut clone_iter = tokens.clone();
            let first = clone_iter.next();
            let second = clone_iter.peek();
            // Named arg pattern: name ':' expr
            if matches!(first, Some(Token::Identifier(_))) && matches!(second, Some(Token::Colon)) {
                super::utils::ParserUtils::next_token(tokens, position); // identifier
                super::utils::ParserUtils::next_token(tokens, position); // colon
                return Self::parse_expression(tokens, position);
            }
            // declare-style pattern: name '=' expr (treat as expression after '=' for now)
            if matches!(first, Some(Token::Identifier(_))) && matches!(second, Some(Token::Equals)) {
                // consume identifier and equals, parse expression
                super::utils::ParserUtils::next_token(tokens, position); // identifier
                super::utils::ParserUtils::next_token(tokens, position); // '='
                return Self::parse_expression(tokens, position);
            }
        }
        Self::parse_expression(tokens, position)
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
            | BinaryOp::GreaterThanOrEqual
            | BinaryOp::Spaceship => 3,
            BinaryOp::BitwiseAnd => 4,
            BinaryOp::BitwiseOr => 4,
            BinaryOp::Concatenate => 5,
            BinaryOp::Add | BinaryOp::Subtract => 6,
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 7,
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
            // Support spread '...expr' (ignored flattening semantics; treat as normal expression)
            if let Some(Token::Ellipsis) = tokens.peek() {
                super::utils::ParserUtils::next_token(tokens, position); // consume '...'
            }
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
                Some(Token::ObjectOperator) => {
                    super::utils::ParserUtils::next_token(tokens, position); // '->'
                    // Expect identifier for method/property
                    let name = match super::utils::ParserUtils::next_token(tokens, position) {
                        Some(Token::Identifier(id)) => id,
                        other => return Err(ParseError::ExpectedToken { expected: "method name".into(), found: format!("{:?}", other), position: *position })
                    };
                    // Optional call
                    if let Some(Token::OpenParen) = tokens.peek() {
                        super::utils::ParserUtils::next_token(tokens, position); // '('
                        // parse args until ')'
                        let mut args = Vec::new();
                        if let Some(Token::CloseParen) = tokens.peek() {
                            super::utils::ParserUtils::next_token(tokens, position); // empty
                        } else {
                            loop {
                                let arg = Self::parse_expression(tokens, position)?;
                                args.push(arg);
                                match tokens.peek() {
                                    Some(Token::Comma) => { super::utils::ParserUtils::next_token(tokens, position); }
                                    Some(Token::CloseParen) => { super::utils::ParserUtils::next_token(tokens, position); break; }
                                    other => return Err(ParseError::ExpectedToken { expected: ", or )".into(), found: format!("{:?}", other), position: *position }),
                                }
                            }
                        }
                        expr = Expr::MethodCall { target: Box::new(expr), method: name, args };
                    } else {
                        // Property fetch fallback: treat as zero-arg method call
                        expr = Expr::MethodCall { target: Box::new(expr), method: name, args: Vec::new() };
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }
}
