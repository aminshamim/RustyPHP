//! Main PHP parser implementation
//!
//! This module contains the main Parser struct that coordinates all the
//! specialized parsing modules.

use crate::ast::{Stmt};
use crate::error::{ParseResult};
use php_lexer::Token;
use std::iter::Peekable;
use std::vec::IntoIter;

use super::control_flow::ControlFlowParser;
use super::expressions::ExpressionParser;
use super::statements::StatementParser;
use super::utils::ParserUtils;

/// PHP parser
pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    position: usize,
}

impl Parser {
    /// Create a new parser with tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
            position: 0,
        }
    }

    /// Parse tokens into a statement (program)
    pub fn parse(&mut self) -> ParseResult<Stmt> {
        self.parse_block()
    }

    /// Parse a block of statements
    fn parse_block(&mut self) -> ParseResult<Stmt> {
        let mut statements = Vec::new();

        // Skip PHP open tag if present
        if let Some(Token::PhpOpen) = self.tokens.peek() {
            ParserUtils::next_token(&mut self.tokens, &mut self.position);
        }

        while let Some(token) = self.tokens.peek() {
            match token {
                Token::EOF => break,
                Token::PhpClose => {
                    ParserUtils::next_token(&mut self.tokens, &mut self.position); // consume ?>
                    break;
                }
                _ => statements.push(self.parse_statement()?),
            }
        }

        Ok(Stmt::Block(statements))
    }

    /// Parse a single statement
    fn parse_statement(&mut self) -> ParseResult<Stmt> {
        Self::parse_statement_with_tokens(&mut self.tokens, &mut self.position)
    }

    /// Parse a single statement with token access (for use by other modules)
    pub fn parse_statement_with_tokens(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        match tokens.peek() {
            Some(Token::Echo) => StatementParser::parse_echo(tokens, position),
            Some(Token::Print) => StatementParser::parse_print(tokens, position),
            Some(Token::Variable(_)) => StatementParser::parse_assignment_or_expression(tokens, position),
            Some(Token::Const) => StatementParser::parse_const(tokens, position),
            Some(Token::Function) => StatementParser::parse_function_definition(tokens, position),
            Some(Token::If) => ControlFlowParser::parse_if(tokens, position),
            Some(Token::While) => ControlFlowParser::parse_while(tokens, position),
            Some(Token::For) => ControlFlowParser::parse_for(tokens, position),
            Some(Token::Foreach) => ControlFlowParser::parse_foreach(tokens, position),
            Some(Token::Return) => ControlFlowParser::parse_return(tokens, position),
            Some(Token::Break) => ControlFlowParser::parse_break(tokens, position),
            Some(Token::Continue) => ControlFlowParser::parse_continue(tokens, position),
            Some(Token::Switch) => ControlFlowParser::parse_switch(tokens, position),
            Some(Token::OpenBrace) => Self::parse_block_statement(tokens, position),
            _ => StatementParser::parse_expression_statement(tokens, position),
        }
    }

    /// Parse a block statement: { stmt1; stmt2; ... }
    pub fn parse_block_statement(
        tokens: &mut Peekable<IntoIter<Token>>,
        position: &mut usize,
    ) -> ParseResult<Stmt> {
        // Consume opening brace
        if let Some(Token::OpenBrace) = tokens.peek() {
            super::utils::ParserUtils::next_token(tokens, position);
        }

        let mut statements = Vec::new();
        
        while let Some(token) = tokens.peek() {
            match token {
                Token::CloseBrace => {
                    super::utils::ParserUtils::next_token(tokens, position); // consume '}'
                    break;
                }
                Token::EOF => break,
                _ => statements.push(Self::parse_statement_with_tokens(tokens, position)?),
            }
        }

        Ok(Stmt::Block(statements))
    }
}

// Legacy parser support - this will be removed in the future

/// Legacy AST node for backwards compatibility
#[derive(Debug, Clone)]
pub enum LegacyNode {
    /// Block of statements
    Block(Vec<LegacyNode>),
    /// Echo statement
    Echo(Box<LegacyNode>),
    /// Print statement
    Print(Box<LegacyNode>),
    /// Variable assignment
    VariableAssignment(String, Box<LegacyNode>),
    /// Variable reference
    Variable(String),
    /// Number literal
    Number(f64),
    /// String literal
    StringLiteral(String),
    /// Binary operation
    BinaryOperation(Box<LegacyNode>, LegacyOperator, Box<LegacyNode>),
    /// If statement
    If(Box<LegacyNode>, Box<LegacyNode>, Option<Box<LegacyNode>>),
    /// While loop
    While(Box<LegacyNode>, Box<LegacyNode>),
    /// For loop
    For(Box<LegacyNode>, Box<LegacyNode>, Box<LegacyNode>, Box<LegacyNode>),
}

/// Legacy operator enum
#[derive(Debug, Clone)]
pub enum LegacyOperator {
    /// Addition
    Add,
    /// Subtraction
    Subtract,
    /// Multiplication
    Multiply,
    /// Division
    Divide,
    /// String concatenation
    Concatenate,
}

/// Legacy parser function
pub fn parse_legacy(_tokens: Vec<Token>) -> Result<LegacyNode, String> {
    // Legacy implementation - to be removed
    Err("Legacy parser not implemented in modular version".to_string())
}
