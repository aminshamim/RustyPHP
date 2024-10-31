use crate::lexer::Token;
use crate::ast::{Node, Operator};
use std::iter::Peekable;

// Main function to parse a list of tokens into a single node
pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let mut iter = tokens.into_iter().peekable();
    parse_statement(&mut iter)
}

// Parse an individual statement
fn parse_statement(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    match iter.next() {
        Some(Token::Echo) => parse_echo(iter),
        Some(Token::If) => parse_if_statement(iter),
        Some(Token::While) => parse_while_statement(iter),
        Some(Token::For) => parse_for_statement(iter),
        Some(Token::Variable(name)) => parse_variable_assignment(name, iter),
        Some(token) => Err(format!("Unexpected token: {:?}", token)),
        None => Err("Unexpected end of input".to_string()),
    }
}

// Parse an `echo` statement
fn parse_echo(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    if let Some(Token::String(content)) = iter.next() {
        if let Some(Token::Semicolon) = iter.next() {
            return Ok(Node::Echo(content));
        }
    }
    Err("Syntax error in echo statement".to_string())
}

// Parse an `if` statement with an optional `else` branch
fn parse_if_statement(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    let condition = parse_expression(iter)?;
    let then_branch = parse_statement(iter)?;
    let else_branch = if let Some(Token::Else) = iter.peek() {
        iter.next(); // Consume `else`
        Some(Box::new(parse_statement(iter)?))
    } else {
        None
    };
    Ok(Node::If(Box::new(condition), Box::new(then_branch), else_branch))
}

// Parse a `while` loop
fn parse_while_statement(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    let condition = parse_expression(iter)?;
    let body = parse_statement(iter)?;
    Ok(Node::While(Box::new(condition), Box::new(body)))
}

// Parse a `for` loop
fn parse_for_statement(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    let init = parse_statement(iter)?;
    let condition = parse_expression(iter)?;
    let increment = parse_statement(iter)?;
    let body = parse_statement(iter)?;
    Ok(Node::For(Box::new(init), Box::new(condition), Box::new(increment), Box::new(body)))
}

// Parse a variable assignment statement
fn parse_variable_assignment(name: String, iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    if let Some(Token::Equals) = iter.next() {
        let value = parse_expression(iter)?;
        if let Some(Token::Semicolon) = iter.next() {
            return Ok(Node::VariableAssignment(name, Box::new(value)));
        }
    }
    Err("Syntax error in variable assignment".to_string())
}

// Parse an expression, supporting binary operations and literals
fn parse_expression(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    let left = parse_primary(iter)?;

    if let Some(op_token) = iter.peek() {
        if let Some(op) = match op_token {
            Token::Plus => Some(Operator::Add),
            Token::Minus => Some(Operator::Subtract),
            Token::Multiply => Some(Operator::Multiply),
            Token::Divide => Some(Operator::Divide),
            _ => None,
        } {
            iter.next(); // Consume operator
            let right = parse_primary(iter)?;
            return Ok(Node::BinaryOperation(Box::new(left), op, Box::new(right)));
        }
    }
    Ok(left)
}

// Parse primary expressions like numbers, strings, and variables
fn parse_primary(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Node, String> {
    match iter.next() {
        Some(Token::Number(value)) => Ok(Node::Number(value)),
        Some(Token::String(content)) => Ok(Node::StringLiteral(content)),
        Some(Token::Variable(name)) => Ok(Node::Variable(name)),
        _ => Err("Expected a primary expression".to_string()),
    }
}
