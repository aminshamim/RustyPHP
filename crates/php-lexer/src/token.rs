//! Token definitions for PHP lexer

use serde::Serialize;

/// Represents a PHP token with its type and potential value
#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Token {
    // PHP Tags
    PhpOpen,
    PhpClose,
    
    // Language constructs
    Echo,
    Print,
    If,
    Else,
    While,
    For,
    Function,
    Return,
    Class,
    Extends,
    Implements,
    New,
    Public,
    Private,
    Protected,
    Static,
    Var,
    Const,
    True,
    False,
    Null,
    Isset,
    Empty,
    Switch,
    Case,
    Default,
    Break,
    Continue,
    Do,
    
    // Built-in functions (will move to stdlib later)
    PrintR,
    Strlen,
    Strpos,
    Substr,
    ArrayPush,
    ArrayPop,
    ArrayMerge,
    InArray,
    Explode,
    Implode,
    Count,
    
    // Literals and identifiers
    Variable(String),
    Number(f64),
    String(String),
    Identifier(String),
    
    // Operators
    Equals,
    DoubleEquals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessOrEqual,
    GreaterOrEqual,
    Plus,
    Minus,
    Multiply,
    Divide,
    Dot,
    
    // Punctuation
    Semicolon,
    Comma,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    
    // Special
    EOF,
}

impl Token {
    /// Returns true if this token represents a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(self, 
            Token::Echo | Token::Print | Token::If | Token::Else |
            Token::While | Token::For | Token::Function | Token::Return |
            Token::Class | Token::Extends | Token::Implements | Token::New |
            Token::Public | Token::Private | Token::Protected | Token::Static |
            Token::Var | Token::Const | Token::True | Token::False | Token::Null |
            Token::Isset | Token::Empty | Token::Switch | Token::Case |
            Token::Default | Token::Break | Token::Continue | Token::Do
        )
    }
    
    /// Returns true if this token represents an operator
    pub fn is_operator(&self) -> bool {
        matches!(self,
            Token::Equals | Token::Plus | Token::Minus | Token::Multiply |
            Token::Divide | Token::Dot
        )
    }
    
    /// Returns true if this token represents a literal value
    pub fn is_literal(&self) -> bool {
        matches!(self,
            Token::Number(_) | Token::String(_) | Token::True | Token::False | Token::Null
        )
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::PhpOpen => write!(f, "<?php"),
            Token::PhpClose => write!(f, "?>"),
            Token::Echo => write!(f, "echo"),
            Token::Print => write!(f, "print"),
            Token::Variable(name) => write!(f, "${}", name),
            Token::Number(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Equals => write!(f, "="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Dot => write!(f, "."),
            Token::Semicolon => write!(f, ";"),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::OpenBrace => write!(f, "{{"),
            Token::CloseBrace => write!(f, "}}"),
            Token::EOF => write!(f, "EOF"),
            _ => write!(f, "{:?}", self),
        }
    }
}
