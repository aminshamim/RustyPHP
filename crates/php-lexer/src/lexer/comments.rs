//! Comment handling for PHP lexer
//!
//! This module handles all types of PHP comments:
//! - Single line comments (//)
//! - Hash comments (#)
//! - Multi-line comments (/* */)

use crate::error::{LexError, LexResult};
use crate::stream::CharStream;

/// Comment detection and skipping functionality
pub struct CommentHandler;

impl CommentHandler {
    /// Try to detect and skip a comment starting at current position
    pub fn try_skip_comment(stream: &mut CharStream) -> LexResult<bool> {
        match stream.peek() {
            Some(&'/') => {
                let next_chars = stream.peek_ahead(2);
                if next_chars.starts_with("//") {
                    Self::skip_single_line_comment(stream);
                    Ok(true)
                } else if next_chars.starts_with("/*") {
                    Self::skip_multi_line_comment(stream)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Some(&'#') => {
                Self::skip_single_line_comment(stream);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Skip single-line comment (// or #)
    fn skip_single_line_comment(stream: &mut CharStream) {
        while let Some(ch) = stream.next() {
            if ch == '\n' {
                break;
            }
        }
    }

    /// Skip multi-line comment (/* ... */)
    fn skip_multi_line_comment(stream: &mut CharStream) -> LexResult<()> {
        let start_pos = stream.position();
        
        // Skip the opening /*
        stream.next(); // '/'
        stream.next(); // '*'
        
        while let Some(ch) = stream.next() {
            if ch == '*' {
                if let Some(&'/') = stream.peek() {
                    stream.next(); // consume '/'
                    return Ok(());
                }
            }
        }
        
        // Unterminated comment
        Err(LexError::UnterminatedString {
            line: start_pos.line,
            column: start_pos.column,
        })
    }
}
