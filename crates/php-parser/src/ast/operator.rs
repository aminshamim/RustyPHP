//! Operator definitions

use std::fmt;

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    /// Addition: +
    Add,
    /// Subtraction: -
    Subtract,
    /// Multiplication: *
    Multiply,
    /// Division: /
    Divide,
    /// Modulo: %
    Modulo,
    /// String concatenation: .
    Concatenate,
    /// Equality: ==
    Equal,
    /// Inequality: !=
    NotEqual,
    /// Less than: <
    LessThan,
    /// Less than or equal: <=
    LessThanOrEqual,
    /// Greater than: >
    GreaterThan,
    /// Greater than or equal: >=
    GreaterThanOrEqual,
    /// Logical AND: &&
    LogicalAnd,
    /// Logical OR: ||
    LogicalOr,
    /// Spaceship: <=>
    Spaceship,
    /// Bitwise AND: &
    BitwiseAnd,
    /// Bitwise OR: |
    BitwiseOr,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    /// Arithmetic negation: -
    Minus,
    /// Logical negation: !
    Not,
    /// Pre-increment: ++
    PreIncrement,
    /// Post-increment: ++
    PostIncrement,
    /// Pre-decrement: --
    PreDecrement,
    /// Post-decrement: --
    PostDecrement,
}

impl BinaryOp {
    /// Get operator precedence (higher number = higher precedence)
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::LogicalOr => 1,
            BinaryOp::LogicalAnd => 2,
            BinaryOp::Equal | BinaryOp::NotEqual => 3,
            BinaryOp::LessThan | BinaryOp::LessThanOrEqual | 
            BinaryOp::GreaterThan | BinaryOp::GreaterThanOrEqual | BinaryOp::Spaceship => 4,
            BinaryOp::BitwiseAnd => 5,
            BinaryOp::BitwiseOr => 5, // treat same precedence for simplified implementation
            BinaryOp::Concatenate => 6,
            BinaryOp::Add | BinaryOp::Subtract => 7,
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 8,
        }
    }
    
    /// Check if operator is left-associative
    pub fn is_left_associative(&self) -> bool {
        true // All PHP binary operators are left-associative
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::Concatenate => ".",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::LessThan => "<",
            BinaryOp::LessThanOrEqual => "<=",
            BinaryOp::GreaterThan => ">",
            BinaryOp::GreaterThanOrEqual => ">=",
            BinaryOp::LogicalAnd => "&&",
            BinaryOp::LogicalOr => "||",
            BinaryOp::Spaceship => "<=>",
            BinaryOp::BitwiseAnd => "&",
            BinaryOp::BitwiseOr => "|",
        };
        write!(f, "{}", op)
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            UnaryOp::Minus => "-",
            UnaryOp::Not => "!",
            UnaryOp::PreIncrement | UnaryOp::PostIncrement => "++",
            UnaryOp::PreDecrement | UnaryOp::PostDecrement => "--",
        };
        write!(f, "{}", op)
    }
}
