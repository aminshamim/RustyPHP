//! PHP syntax parsing and AST generation
//! 
//! This crate provides parsing of PHP tokens into an Abstract Syntax Tree (AST)
//! for execution by the runtime.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod ast;
pub mod parser;
pub mod error;

pub use ast::*;
pub use parser::*;
pub use error::*;
