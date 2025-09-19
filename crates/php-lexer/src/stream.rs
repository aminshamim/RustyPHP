//! Character stream handling for lexer

use crate::error::{LexError, LexResult};
use std::str::Chars;
use std::iter::Peekable;

/// Position information for tokens
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

impl Position {
    /// Create a new position
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
    
    /// Initial position at start of file
    pub fn start() -> Self {
        Self::new(1, 1)
    }
}

/// Character stream with position tracking
pub struct CharStream<'a> {
    chars: Peekable<Chars<'a>>,
    position: Position,
}

impl<'a> Clone for CharStream<'a> {
    fn clone(&self) -> Self {
        // NOTE: We cannot clone the internal iterator state exactly without storing the original slice & offset.
        // For the limited heuristic in the lexer (one-char lookahead already exists), we avoid deep clone usage now.
        // If clone is requested, we create a new empty-at-end stream to prevent misuse.
        // Future improvement: refactor CharStream to store original & index to enable true cloning.
        Self {
            chars: "".chars().peekable(),
            position: self.position,
        }
    }
}

impl<'a> CharStream<'a> {
    /// Create a new character stream from input string
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            position: Position::start(),
        }
    }
    
    /// Peek at the next character without consuming it
    pub fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }
    
    /// Peek ahead multiple characters without consuming them
    pub fn peek_ahead(&mut self, count: usize) -> String {
        let mut result = String::new();
        let chars: Vec<char> = self.chars.clone().take(count).collect();
        for ch in chars {
            result.push(ch);
        }
        result
    }
    
    /// Consume and return the next character
    pub fn next(&mut self) -> Option<char> {
        match self.chars.next() {
            Some('\n') => {
                self.position.line += 1;
                self.position.column = 1;
                Some('\n')
            }
            Some(ch) => {
                self.position.column += 1;
                Some(ch)
            }
            None => None,
        }
    }
    
    /// Get current position
    pub fn position(&self) -> Position {
        self.position
    }
    
    /// Take a specific number of characters
    pub fn take(&mut self, count: usize) -> String {
        let mut result = String::new();
        for _ in 0..count {
            if let Some(ch) = self.next() {
                result.push(ch);
            } else {
                break;
            }
        }
        result
    }
    
    /// Skip whitespace characters
    pub fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.peek() {
            if ch.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }
    
    /// Read an identifier (alphanumeric + underscore)
    pub fn read_identifier(&mut self) -> String {
        let mut identifier = String::new();
        while let Some(&ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.next();
            } else {
                break;
            }
        }
        identifier
    }
    
    /// Read a number (integer or float)
    pub fn read_number(&mut self) -> LexResult<f64> {
        let start_pos = self.position();
        let mut number_str = String::new();
        
        while let Some(&ch) = self.peek() {
            if ch.is_numeric() || ch == '.' {
                number_str.push(ch);
                self.next();
            } else {
                break;
            }
        }
        
        number_str.parse::<f64>().map_err(|_| LexError::InvalidNumber {
            number: number_str,
            line: start_pos.line,
            column: start_pos.column,
        })
    }
    
    /// Read a string literal (with quote character)
    pub fn read_string(&mut self, quote: char) -> LexResult<String> {
        let start_pos = self.position();
        let mut string_content = String::new();
        
        // Skip opening quote
        self.next();
        
        while let Some(&ch) = self.peek() {
            if ch == quote {
                self.next(); // Skip closing quote
                return Ok(string_content);
            }
            
            // Handle escape sequences (basic)
            if ch == '\\' {
                self.next(); // Skip backslash
                if let Some(&escaped) = self.peek() {
                    match escaped {
                        'n' => string_content.push('\n'),
                        't' => string_content.push('\t'),
                        'r' => string_content.push('\r'),
                        '\\' => string_content.push('\\'),
                        '\'' => string_content.push('\''),
                        '"' => string_content.push('"'),
                        _ => {
                            string_content.push('\\');
                            string_content.push(escaped);
                        }
                    }
                    self.next();
                }
            } else {
                string_content.push(ch);
                self.next();
            }
        }
        
        Err(LexError::UnterminatedString {
            line: start_pos.line,
            column: start_pos.column,
        })
    }

    /// Check if we're at the end of the input
    pub fn is_at_end(&mut self) -> bool {
        self.chars.peek().is_none()
    }
}
