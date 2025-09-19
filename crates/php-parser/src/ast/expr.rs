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
    /// Yield expression (simplified, no generator state): yield expr; or yield from expr;
    Yield {
        /// Inner expression value (ignored for now)
        value: Box<Expr>,
    },
    /// Method call: target->method(args)
    MethodCall {
        /// Target expression
        target: Box<Expr>,
        /// Method name
        method: String,
        /// Arguments
        args: Vec<Expr>,
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
    /// Arrow function: fn(params) => expr
    ArrowFunction {
        /// Parameter variable names
        params: Vec<String>,
        /// Body expression
        body: Box<Expr>,
    },
    /// Dynamic function/closure call: $var(...)
    DynamicCall {
        /// Variable holding closure
        target: Box<Expr>,
        /// Arguments
        args: Vec<Expr>,
    },
    /// Ternary conditional: condition ? then : else
    Ternary {
        /// Condition expression
        condition: Box<Expr>,
        /// Expression if condition is truthy (may be None for shorthand ?: operator to reuse condition)
        then_expr: Option<Box<Expr>>,
        /// Else expression
        else_expr: Box<Expr>,
    },
    /// Match expression: match (subject) { conditions => result, default => result }
    Match {
        /// Subject expression evaluated once
        subject: Box<Expr>,
        /// Arms list: each arm has a list of condition expressions and a boxed result expression
        arms: Vec<(Vec<Expr>, Box<Expr>)>,
        /// Optional default arm expression (boxed)
        default_arm: Option<Box<Expr>>,
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
            Expr::ArrowFunction { params, body } => {
                write!(f, "fn(")?;
                for (i,p) in params.iter().enumerate() { if i>0 { write!(f, ", ")?; } write!(f, "${}", p)?; }
                write!(f, ") => {}", body)
            }
            Expr::DynamicCall { target, args } => {
                write!(f, "{}(", target)?;
                for (i,a) in args.iter().enumerate() { if i>0 { write!(f, ", ")?; } write!(f, "{}", a)?; }
                write!(f, ")")
            }
            Expr::Ternary { condition, then_expr, else_expr } => {
                if let Some(t) = then_expr { write!(f, "({} ? {} : {})", condition, t, else_expr) } else { write!(f, "({} ?: {})", condition, else_expr) }
            }
            Expr::Match { subject, arms, default_arm } => {
                write!(f, "match ({}) {{ ", subject)?;
                for (i, (conds, result)) in arms.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    for (j, c) in conds.iter().enumerate() { if j>0 { write!(f, ", ")?; } write!(f, "{}", c)?; }
                    write!(f, " => {}", result)?;
                }
                if let Some(def) = default_arm { if !arms.is_empty() { write!(f, ", ")?; } write!(f, "default => {}", def)?; }
                write!(f, " }}")
            }
            Expr::Yield { value } => write!(f, "yield {}", value),
            Expr::MethodCall { target, method, args } => {
                write!(f, "{}->{}(", target, method)?;
                for (i,a) in args.iter().enumerate() { if i>0 { write!(f, ", ")?; } write!(f, "{}", a)?; }
                write!(f, ")")
            }
        }
    }
}
