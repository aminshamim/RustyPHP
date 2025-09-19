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
    /// Foreach loop: foreach ($array as $item) { ... } or foreach ($array as $key => $value) { ... }
    Foreach {
        /// Array expression to iterate over
        array: Expr,
        /// Variable name for the value (always present)
        value_var: String,
        /// Optional variable name for the key
        key_var: Option<String>,
        /// Loop body
        body: Box<Stmt>,
    },
    /// Return statement: return $value;
    Return(Option<Expr>),
    /// Break statement: break;
    Break,
    /// Continue statement: continue;
    Continue,
    /// Function definition: function name($param1, $param2) { ... }
    FunctionDefinition {
        /// Function name
        name: String,
        /// Parameters
        parameters: Vec<String>,
        /// Function body
        body: Box<Stmt>,
    },
    /// Switch statement: switch(expr) { case v: ... break; default: ... }
    Switch {
        /// Discriminant expression
        expression: Expr,
        /// Cases
        cases: Vec<SwitchCase>,
        /// Optional default block
        default: Option<Vec<Stmt>>,
    },
}

/// Single switch case
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    /// Case match expression
    pub value: Expr,
    /// Statements in this case until break/next case
    pub statements: Vec<Stmt>,
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
            Stmt::Foreach { array, value_var, key_var, body } => {
                write!(f, "foreach ({} as ", array)?;
                if let Some(key_var) = key_var {
                    write!(f, "${} => ", key_var)?;
                }
                write!(f, "${}) {}", value_var, body)
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
            Stmt::FunctionDefinition { name, parameters, body } => {
                write!(f, "function {}(", name)?;
                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "${}", param)?;
                }
                write!(f, ") {}", body)
            }
            Stmt::Switch { expression, cases, default } => {
                writeln!(f, "switch ({}) {{", expression)?;
                for case in cases {
                    writeln!(f, "  case {}:", case.value)?;
                    for stmt in &case.statements {
                        writeln!(f, "    {}", stmt)?;
                    }
                }
                if let Some(default_stmts) = default {
                    writeln!(f, "  default:")?;
                    for stmt in default_stmts {
                        writeln!(f, "    {}", stmt)?;
                    }
                }
                write!(f, "}}")
            }
        }
    }
}
