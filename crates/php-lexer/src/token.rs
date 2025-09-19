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
    ElseIf,
    While,
    For,
    Foreach,
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
    As,
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
    Declare,
    Try,
    Catch,
    
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
    Arrow, // =>
    LessThan,
    GreaterThan,
    LessOrEqual,
    GreaterOrEqual,
    /// Spaceship operator <=>
    Spaceship,
    Plus,
    Minus,
    Multiply,
    Divide,
    Dot,
    Colon,
    QuestionMark,
    NullCoalescing, // ??
    Increment, // ++
    Decrement, // --
    /// Error suppression operator '@' (currently ignored by runtime)
    At,
    /// Ampersand '&' (reference / bitwise AND placeholder)
    Ampersand,
    /// Object operator '->'
    ObjectOperator,
    /// Pipe '|' for union types (currently skipped by parser)
    Pipe,
    /// Logical AND '&&'
    LogicalAnd,
    /// Logical OR '||'
    LogicalOr,
    /// Ellipsis '...' for variadics/spread (currently skipped by parser)
    Ellipsis,
    
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
            Token::Echo | Token::Print | Token::If | Token::Else | Token::ElseIf |
            Token::While | Token::For | Token::Function | Token::Return |
            Token::Class | Token::Extends | Token::Implements | Token::New |
            Token::Public | Token::Private | Token::Protected | Token::Static |
            Token::Var | Token::Const | Token::True | Token::False | Token::Null |
            Token::Isset | Token::Empty | Token::Switch | Token::Case |
            Token::Default | Token::Break | Token::Continue | Token::Do |
            Token::Try | Token::Catch
        )
    }
    
    /// Returns true if this token represents an operator
    pub fn is_operator(&self) -> bool {
        matches!(self,
            Token::Equals | Token::Plus | Token::Minus | Token::Multiply |
            Token::Divide | Token::Dot | Token::Colon | Token::QuestionMark |
            Token::NullCoalescing | Token::Arrow | Token::Increment | Token::Decrement |
            Token::LogicalAnd | Token::LogicalOr | Token::Ampersand | Token::Pipe
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
            Token::Arrow => write!(f, "=>"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Dot => write!(f, "."),
            Token::Colon => write!(f, ":"),
            Token::QuestionMark => write!(f, "?"),
            Token::NullCoalescing => write!(f, "??"),
            Token::Increment => write!(f, "++"),
            Token::Decrement => write!(f, "--"),
            Token::At => write!(f, "@"),
            Token::Ampersand => write!(f, "&"),
            Token::Pipe => write!(f, "|"),
            Token::LogicalAnd => write!(f, "&&"),
            Token::LogicalOr => write!(f, "||"),
            Token::Ellipsis => write!(f, "..."),
            Token::Declare => write!(f, "declare"),
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
