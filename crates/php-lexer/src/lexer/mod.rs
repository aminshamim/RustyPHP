//! Lexer module organization
//!
//! This module contains the main lexer and specialized sub-modules for different
//! types of token recognition.

pub mod comments;
pub mod keywords;
pub mod literals;
pub mod operators;
pub mod main;

pub use main::Lexer;
