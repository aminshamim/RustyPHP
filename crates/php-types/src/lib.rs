//! PHP type system and value representations
//! 
//! This crate provides the core PHP type system including values, arrays, objects,
//! and type conversions that match PHP's semantics.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod value;
pub mod array;
pub mod object;
pub mod conversion;

pub use value::*;
pub use array::*;
pub use object::*;
pub use conversion::*;
