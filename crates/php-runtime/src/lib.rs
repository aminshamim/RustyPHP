//! PHP Runtime Engine
//! 
//! This crate provides the execution engine for PHP code.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod engine;

pub use engine::{Engine, ExecutionContext, Function};
