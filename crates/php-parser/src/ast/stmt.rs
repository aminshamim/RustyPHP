//! Statement AST nodes

use super::Expr;
use std::fmt;

/// Represents PHP statements  
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Expression statement: $a + 1;
    Expression(Expr),
    /// Echo statement: echo $var;
    Echo(Expr),
    /// Print statement: print $var;
    Print(Expr),
    /// Variable assignment: $var = value;
    Assignment {
        /// Variable name
        variable: String,
        /// Value expression
        value: Expr,
    },
    /// Constant definition: const NAME = value; or define('NAME', value);
    ConstantDefinition {
        /// Constant name
        name: String,
        /// Value expression
        value: Expr,
    },
    /// Block statement: { stmt1; stmt2; }
    Block(Vec<Stmt>),
    /// If statement: if (condition) { ... } else { ... }
    If {
        /// Condition expression
        condition: Expr,
        /// Then branch
        then_stmt: Box<Stmt>,
        /// Optional else branch
        else_stmt: Option<Box<Stmt>>,
    },
    /// While loop: while (condition) { ... }
    While {
        /// Condition expression
        condition: Expr,
        /// Loop body
        body: Box<Stmt>,
    },
    /// For loop: for (init; condition; increment) { ... }
    For {
        /// Initialization statement
        init: Option<Box<Stmt>>,
        /// Condition expression
        condition: Option<Expr>,
        /// Increment expression
        increment: Option<Expr>,
        /// Loop body
        body: Box<Stmt>,
    },
    /// Return statement: return $value;
    Return(Option<Expr>),
    /// Break statement: break;
    Break,
    /// Continue statement: continue;
    Continue,
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expression(expr) => write!(f, "{};", expr),
            Stmt::Echo(expr) => write!(f, "echo {};", expr),
            Stmt::Print(expr) => write!(f, "print {};", expr),
            Stmt::Assignment { variable, value } => write!(f, "${} = {};", variable, value),
            Stmt::ConstantDefinition { name, value } => write!(f, "const {} = {};", name, value),
            Stmt::Block(stmts) => {
                writeln!(f, "{{")?;
                for stmt in stmts {
                    writeln!(f, "  {}", stmt)?;
                }
                write!(f, "}}")
            }
            Stmt::If { condition, then_stmt, else_stmt } => {
                write!(f, "if ({}) {}", condition, then_stmt)?;
                if let Some(else_stmt) = else_stmt {
                    write!(f, " else {}", else_stmt)?;
                }
                Ok(())
            }
            Stmt::While { condition, body } => write!(f, "while ({}) {}", condition, body),
            Stmt::For { init, condition, increment, body } => {
                write!(f, "for (")?;
                if let Some(init) = init { write!(f, "{}", init)?; } else { write!(f, ";")?; }
                write!(f, " ")?;
                if let Some(condition) = condition { write!(f, "{}", condition)?; }
                write!(f, "; ")?;
                if let Some(increment) = increment { write!(f, "{}", increment)?; }
                write!(f, ") {}", body)
            }
            Stmt::Return(expr) => {
                write!(f, "return")?;
                if let Some(expr) = expr {
                    write!(f, " {}", expr)?;
                }
                write!(f, ";")
            }
            Stmt::Break => write!(f, "break;"),
            Stmt::Continue => write!(f, "continue;"),
        }
    }
}
