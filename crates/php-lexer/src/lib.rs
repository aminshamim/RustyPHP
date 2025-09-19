//! PHP lexer for tokenizing PHP source code

mod error;
mod lexer;
mod stream;
mod token;

#[cfg(test)]
mod debug_test;

pub use error::{LexError, LexResult};
pub use token::Token;

// Export the main lexer for internal use
use lexer::Lexer;

/// Convenience function to tokenize PHP source code
/// 
/// # Examples
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
