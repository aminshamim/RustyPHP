//! PHP lexical analysis and tokenization
//! 
//! This crate provides tokenization of PHP source code into a stream of tokens
//! for consumption by the parser.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod token;
pub mod lexer;
pub mod error;
pub mod stream;

pub use lexer::Lexer;
pub use token::*;
pub use error::*;

/// Convenience function to tokenize PHP source code
/// 
/// # Arguments
/// * `input` - The PHP source code to tokenize
/// 
/// # Returns
/// * `LexResult<Vec<Token>>` - Vector of tokens or lexer error
/// 
/// # Example
/// ```
/// use php_lexer::lex;
/// let tokens = lex("<?php echo 'Hello World'; ?>").unwrap();
/// ```
pub fn lex(input: &str) -> LexResult<Vec<Token>> {
    let mut lexer = Lexer::new(input);
    let mut tokens = lexer.tokenize()?;
    tokens.push(Token::EOF);
    Ok(tokens)
}
