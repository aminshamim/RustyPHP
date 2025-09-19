//! Web server and SAPI implementations
//! 
//! This crate provides web server functionality and various Server APIs (SAPI)
//! for running PHP code in web environments.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod playground;

pub use playground::*;
