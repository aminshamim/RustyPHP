//! Modular PHP parser
//!
//! This module provides the main parser interface and organizes
//! all specialized parsing modules.

mod control_flow;
mod expressions;
mod main;
mod statements;
mod utils;

#[cfg(test)]
mod tests;

// Re-export the main parser struct and functions
pub use main::{Parser, parse_legacy, LegacyNode, LegacyOperator};

use crate::ast::Stmt;
use crate::error::ParseResult;
use php_lexer::Token;

/// Convenience function to parse tokens
pub fn parse(tokens: Vec<Token>) -> ParseResult<Stmt> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
