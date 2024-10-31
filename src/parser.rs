// parser.rs

use std::fmt::Debug;
use crate::lexer::Token;
use crate::ast::{Node, Operator};
use std::iter::Peekable;

pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    // for token in &tokens {
    //     // Convert the token to a JSON string
    //     let json_token = serde_json::to_string(token)
    //         .unwrap_or_else(|_| "Error serializing token".to_string());
    //
    //     // Print the JSON string representation of each token
    //     println!("{}", json_token);
    // }
    let mut iter = tokens.into_iter().peekable();
    parse_block(&mut iter)
}

fn parse_block(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    let mut statements = Vec::new();

    while let Some(token) = iter.peek() {
        if *token == Token::EOF {
            break;
        }
        statements.push(parse_statement(iter)?);
    }

    Ok(Node::Block(statements))
}

fn parse_statement(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    match iter.next() {
        Some(Token::Echo) => parse_echo(iter),
        Some(Token::Print) => parse_print(iter),
        Some(Token::Plus) => parse_expression(iter),
        Some(Token::Variable(name)) => parse_variable_assignment(name, iter),
        Some(token) => Err(format!("Unexpected token: {:?}", token)),
        None => Err("Unexpected end of input".to_string()),
    }
}

fn parse_echo(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    let expr = parse_expression(iter)?;
    if let Some(Token::Semicolon) | Some(Token::EOF) = iter.peek() {
        iter.next();
        return Ok(Node::Echo(Box::new(expr)));
    }
    Err("Syntax error in echo statement: expected ';' or end of input".to_string())
}

fn parse_print(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    let expr = parse_expression(iter)?;
    if let Some(Token::Semicolon) | Some(Token::EOF) = iter.peek() {
        iter.next();
        return Ok(Node::Print(Box::new(expr)));
    }
    Err("Syntax error in echo statement: expected ';' or end of input".to_string())
}

fn parse_variable_assignment(name: String, iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    if let Some(Token::Equals) = iter.next() {
        let value = parse_expression(iter)?;
        if let Some(Token::Semicolon) | Some(Token::EOF) = iter.peek() {
            iter.next();
            return Ok(Node::VariableAssignment(name, Box::new(value)));
        }
    }
    Err("Syntax error in variable assignment".to_string())
}

fn parse_expression(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    let left = parse_primary(iter)?;

    if let Some(op_token) = iter.peek() {
        if let Some(op) = match op_token {
            Token::Plus => Some(Operator::Add),
            Token::Minus => Some(Operator::Subtract),
            Token::Multiply => Some(Operator::Multiply),
            Token::Divide => Some(Operator::Divide),
            Token::Dot => Some(Operator::Concatenate), // Add concatenation support
            _ => None,
        } {
            iter.next();
            let right = parse_primary(iter)?;

            return Ok(Node::BinaryOperation(Box::new(left), op, Box::new(right)));
        }
    }
    Ok(left)
}

fn parse_primary(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    match iter.next() {
        Some(Token::Number(value)) => Ok(Node::Number(value)),
        Some(Token::String(content)) => Ok(Node::StringLiteral(content)),
        Some(Token::Variable(name)) => Ok(Node::Variable(name)),
        _ => Err("Expected a primary expression".to_string()),
    }
}
