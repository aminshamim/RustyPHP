//! Literal token recognition for PHP lexer
//!
//! This module handles recognition of PHP literals:
//! - String literals (single and double quoted)
//! - Number literals (integers and floats)
//! - Variables ($variable)

use crate::error::{LexError, LexResult};
use crate::stream::CharStream;
use crate::token::Token;

/// Literal token recognition functionality
pub struct LiteralHandler;

impl LiteralHandler {
    /// Tokenize a variable ($variable)
    pub fn tokenize_variable(stream: &mut CharStream) -> LexResult<Token> {
        stream.next(); // consume '$'
        
        let mut name = String::new();
        while let Some(&ch) = stream.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                name.push(stream.next().unwrap());
            } else {
                break;
            }
        }
        
        if name.is_empty() {
            let pos = stream.position();
            return Err(LexError::UnexpectedChar {
                char: '$',
                line: pos.line,
                column: pos.column,
            });
        }
        
        Ok(Token::Variable(name))
    }

    /// Tokenize a string literal
    pub fn tokenize_string(stream: &mut CharStream) -> LexResult<Token> {
        let quote_char = stream.peek().copied().unwrap();
        let content = stream.read_string(quote_char)?;
        Ok(Token::String(content))
    }

    /// Tokenize a number literal
    pub fn tokenize_number(stream: &mut CharStream) -> LexResult<Token> {
        let number = stream.read_number()?;
        Ok(Token::Number(number))
    }

    /// Tokenize an identifier
    pub fn tokenize_identifier(stream: &mut CharStream) -> String {
        stream.read_identifier()
    }

    /// Tokenize a heredoc or nowdoc string literal beginning with <<<
    /// Basic implementation: captures content until a line that exactly matches the identifier
    /// (optionally followed by a semicolon). Nowdoc (with single quotes) is treated the same as
    /// heredoc for now (no special interpolation differences at lexing stage in this simplified engine).
    pub fn tokenize_heredoc(stream: &mut CharStream) -> LexResult<Token> {
        // We are positioned at first '<' of the sequence '<<<'
        // Consume the three '<'
        stream.next();
        stream.next();
        stream.next();

        // Detect nowdoc vs heredoc (nowdoc starts with single quote)
        let mut identifier = String::new();
        let mut nowdoc = false;
        if let Some(&ch) = stream.peek() {
            if ch == '\'' { // nowdoc
                nowdoc = true;
                stream.next(); // consume opening quote
                while let Some(&c2) = stream.peek() {
                    if c2 == '\'' { stream.next(); break; }
                    if c2 == '\n' || c2 == '\r' { break; }
                    identifier.push(stream.next().unwrap());
                }
            } else {
                while let Some(&c2) = stream.peek() {
                    if c2 == '\n' || c2 == '\r' { break; }
                    if c2.is_whitespace() { break; }
                    identifier.push(stream.next().unwrap());
                }
            }
        }

        // Consume remainder of the line (up to and including first newline)
        while let Some(ch) = stream.next() {
            if ch == '\n' { break; }
            if ch == '\r' { // handle Windows newlines \r\n
                if let Some(&'\n') = stream.peek() { stream.next(); }
                break;
            }
        }

        if identifier.is_empty() {
            let pos = stream.position();
            return Err(LexError::UnterminatedString { line: pos.line, column: pos.column });
        }

        // Accumulate lines until a line that is exactly identifier or identifier; (with optional trailing whitespace)
        let mut content = String::new();
        let mut current_line = String::new();
    // Track if terminator had trailing semicolon (currently unused; parser tolerates missing semicolon after heredoc assignment)
    let mut _had_trailing_semicolon = false;
        loop {
            let ch_opt = stream.next();
            match ch_opt {
                Some('\n') => {
                    // Check termination condition on current_line without trailing whitespace
                    let trimmed = current_line.trim_end();
                    if trimmed == identifier || trimmed == format!("{};", identifier) {
                        if trimmed.ends_with(';') { _had_trailing_semicolon = true; }
                        // Heredoc terminator found - do not include terminator line
                        break;
                    } else {
                        content.push_str(&current_line);
                        content.push('\n');
                        current_line.clear();
                    }
                }
                Some('\r') => {
                    // Normalize CR or CRLF as newline
                    if let Some(&'\n') = stream.peek() { stream.next(); }
                    let trimmed = current_line.trim_end();
                    if trimmed == identifier || trimmed == format!("{};", identifier) {
                        if trimmed.ends_with(';') { _had_trailing_semicolon = true; }
                        break;
                    } else {
                        content.push_str(&current_line);
                        content.push('\n');
                        current_line.clear();
                    }
                }
                Some(ch) => {
                    current_line.push(ch);
                }
                None => {
                    // EOF without terminator
                    let pos = stream.position();
                    return Err(LexError::UnterminatedString { line: pos.line, column: pos.column });
                }
            }
        }

        // For nowdoc we don't perform interpolation at lexer stage; runtime will treat raw string
        let _ = nowdoc; // suppress unused warning for now
        // If a semicolon followed the terminator, emit a semicolon token next by pushing it back conceptually.
        // Since we don't have a pushback mechanism, we'll store the fact in a thread-local? Simpler: append ';' to content? Not correct.
        // Instead: we will include a trailing semicolon token by hacking: place a sentinel in stream that the main lexer will detect.
        // Minimal approach: if had_trailing_semicolon, we append a ';' char to stream at current position by manipulating internal buffer.
        // Simpler: return String token; main lexer after calling this will check last consumed line? For now, ignore emitting semicolon and rely on parser not requiring it.
        // However parser currently expects semicolon after assignment. We'll fake by appending an actual semicolon token via a global flag (out of scope). So fallback: ensure upstream code doesn't require semicolon by adding one at end of content? This changes string value; unacceptable.
        // Revised simpler solution: Do nothing extra; adjust parser to accept missing semicolon after heredoc assignment.
        Ok(Token::String(content))
    }
}
