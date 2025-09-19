//! Expression AST nodes

use std::fmt;

/// Represents PHP expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Variable reference: $var
    Variable(String),
    /// Constant reference: CONSTANT_NAME
    Constant(String),
    /// Numeric literal: 42, 3.14
    Number(f64),
    /// String literal: "hello"
    String(String),
    /// Boolean literal: true, false
    Bool(bool),
    /// Null literal
    Null,
    /// Binary operation: $a + $b
    Binary {
        /// Left operand
        left: Box<Expr>,
        /// Operator
        op: super::BinaryOp,
        /// Right operand
        right: Box<Expr>,
    },
    /// Unary operation: -$a, !$b
    Unary {
        /// Operator
        op: super::UnaryOp,
        /// Operand
        operand: Box<Expr>,
    },
    /// Function call: func($arg1, $arg2)
    FunctionCall {
        /// Function name
        name: String,
        /// Arguments
        args: Vec<Expr>,
    },
    /// Array literal: [1, 2, 3] or array(1, 2, 3)
    Array(Vec<ArrayElement>),
    /// Array access: $arr[0] or $arr['key']
    ArrayAccess {
        /// Array expression
        array: Box<Expr>,
        /// Index expression
        index: Box<Expr>,
    },
    /// Null coalescing expression: left ?? right
    NullCoalesce {
        /// Left expression
        left: Box<Expr>,
        /// Right expression
        right: Box<Expr>,
    },
}

/// Array element in array literal
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayElement {
    /// Optional key (for associative arrays)
    pub key: Option<Expr>,
    /// Value
    pub value: Expr,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Variable(name) => write!(f, "${}", name),
            Expr::Constant(name) => write!(f, "{}", name),
            Expr::Number(n) => write!(f, "{}", n),
            Expr::String(s) => write!(f, "\"{}\"", s),
            Expr::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Expr::Null => write!(f, "null"),
            Expr::Binary { left, op, right } => write!(f, "({} {} {})", left, op, right),
            Expr::Unary { op, operand } => write!(f, "({}{})", op, operand),
            Expr::FunctionCall { name, args } => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expr::Array(elements) => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    if let Some(key) = &elem.key {
                        write!(f, "{} => {}", key, elem.value)?;
                    } else {
                        write!(f, "{}", elem.value)?;
                    }
                }
                write!(f, "]")
            }
            Expr::ArrayAccess { array, index } => write!(f, "{}[{}]", array, index),
            Expr::NullCoalesce { left, right } => write!(f, "({} ?? {})", left, right),
        }
    }
}
