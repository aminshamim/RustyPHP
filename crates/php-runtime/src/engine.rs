//! PHP Runtime Engine

use php_types::PhpValue;
use php_parser::ast::{Stmt, Expr};
use std::collections::HashMap;

/// PHP execution context with variable scoping
#[derive(Debug)]
pub struct ExecutionContext {
    /// Variable storage
    variables: HashMap<String, PhpValue>,
    /// Constant storage
    constants: HashMap<String, PhpValue>,
    /// Function definitions
    functions: HashMap<String, Function>,
    /// Output buffer
    output: String,
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    /// Function parameters
    pub params: Vec<String>,
    /// Function body
    pub body: Stmt,
}

impl ExecutionContext {
    /// Create new execution context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            constants: HashMap::new(),
            functions: HashMap::new(),
            output: String::new(),
        }
    }

    /// Get variable value
    pub fn get_variable(&self, name: &str) -> Option<&PhpValue> {
        self.variables.get(name)
    }

    /// Set variable value
    pub fn set_variable(&mut self, name: String, value: PhpValue) {
        self.variables.insert(name, value);
    }

    /// Get constant value
    pub fn get_constant(&self, name: &str) -> Option<&PhpValue> {
        self.constants.get(name)
    }

    /// Set constant value
    pub fn set_constant(&mut self, name: String, value: PhpValue) {
        self.constants.insert(name, value);
    }

    /// Get output
    pub fn get_output(&self) -> &str {
        &self.output
    }

    /// Add to output
    pub fn add_output(&mut self, text: &str) {
        self.output.push_str(text);
    }
}

/// PHP Runtime Engine
pub struct Engine {
    /// Current execution context
    context: ExecutionContext,
}

impl Engine {
    /// Create new engine
    pub fn new() -> Self {
        Self {
            context: ExecutionContext::new(),
        }
    }

    /// Execute a statement
    pub fn execute_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate_expr(expr)?;
                Ok(())
            }
            Stmt::Echo(expr) => {
                let value = self.evaluate_expr(expr)?;
                self.context.add_output(&value.to_string());
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate_expr(expr)?;
                self.context.add_output(&value.to_string());
                Ok(())
            }
            Stmt::Assignment { variable, value } => {
                let val = self.evaluate_expr(value)?;
                self.context.set_variable(variable.clone(), val);
                Ok(())
            }
            Stmt::ConstantDefinition { name, value } => {
                let val = self.evaluate_expr(value)?;
                self.context.set_constant(name.clone(), val);
                Ok(())
            }
            Stmt::Block(statements) => {
                for stmt in statements {
                    self.execute_stmt(stmt)?;
                }
                Ok(())
            }
            _ => Err("Statement not implemented yet".to_string()),
        }
    }

    /// Evaluate an expression
    pub fn evaluate_expr(&mut self, expr: &Expr) -> Result<PhpValue, String> {
        match expr {
            Expr::Variable(name) => {
                self.context
                    .get_variable(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable: ${}", name))
            }
            Expr::Constant(name) => {
                self.context
                    .get_constant(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined constant: {}", name))
            }
            Expr::Number(n) => Ok(PhpValue::Float(*n)),
            Expr::String(s) => Ok(PhpValue::String(s.clone())),
            Expr::Bool(b) => Ok(PhpValue::Bool(*b)),
            Expr::Null => Ok(PhpValue::Null),
            Expr::Binary { left, op, right } => {
                let left_val = self.evaluate_expr(left)?;
                let right_val = self.evaluate_expr(right)?;
                
                use php_parser::ast::BinaryOp;
                match op {
                    BinaryOp::Add => Ok(php_types::php_add(&left_val, &right_val)),
                    BinaryOp::Subtract => Ok(php_types::php_subtract(&left_val, &right_val)),
                    BinaryOp::Multiply => Ok(php_types::php_multiply(&left_val, &right_val)),
                    BinaryOp::Divide => php_types::php_divide(&left_val, &right_val),
                    BinaryOp::Concatenate => Ok(php_types::php_concatenate(&left_val, &right_val)),
                    _ => Err("Binary operator not implemented".to_string()),
                }
            }
            Expr::FunctionCall { name, args } => {
                self.call_function(name, args)
            }
            _ => Err("Expression not implemented yet".to_string()),
        }
    }

    /// Call a function
    fn call_function(&mut self, name: &str, args: &[Expr]) -> Result<PhpValue, String> {
        match name {
            "define" => {
                if args.len() != 2 {
                    return Err("define() expects exactly 2 arguments".to_string());
                }
                
                // First argument should be the constant name (string)
                let const_name = match self.evaluate_expr(&args[0])? {
                    PhpValue::String(s) => s,
                    _ => return Err("define() first argument must be a string".to_string()),
                };
                
                // Second argument is the constant value
                let const_value = self.evaluate_expr(&args[1])?;
                
                // Define the constant
                self.context.set_constant(const_name, const_value);
                
                // define() returns true on success
                Ok(PhpValue::Bool(true))
            }
            _ => Err(format!("Unknown function: {}", name)),
        }
    }

    /// Get execution output
    pub fn get_output(&self) -> &str {
        self.context.get_output()
    }
}
