//! PHP Runtime Engine

use php_types::{PhpValue, PhpArrayKey, PhpArray};
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

/// Internal control flow signal for break/continue/return
enum ExecSignal {
    None,
    Break,
    Continue,
    Return(Option<PhpValue>),
}

impl Engine {
    /// Create new engine
    pub fn new() -> Self {
        let mut ctx = ExecutionContext::new();
        // Initialize superglobals minimal
        ctx.set_variable("_GET".to_string(), PhpValue::Array(PhpArray::new()));
        Self { context: ctx }
    }

    /// Execute a statement
    pub fn execute_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        let signal = self.exec(stmt)?;
        match signal {
            ExecSignal::None => Ok(()),
            ExecSignal::Break | ExecSignal::Continue => Ok(()),
            ExecSignal::Return(val_opt) => {
                if let Some(val) = val_opt { self.context.add_output(&val.to_string()); }
                Ok(())
            }
        }
    }

    fn exec(&mut self, stmt: &Stmt) -> Result<ExecSignal, String> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate_expr(expr)?;
                Ok(ExecSignal::None)
            }
            Stmt::Echo(expr) => {
                let value = self.evaluate_expr(expr)?;
                self.context.add_output(&value.to_string());
                Ok(ExecSignal::None)
            }
            Stmt::Print(expr) => {
                let value = self.evaluate_expr(expr)?;
                self.context.add_output(&value.to_string());
                Ok(ExecSignal::None)
            }
            Stmt::Assignment { variable, value } => {
                let val = self.evaluate_expr(value)?;
                self.context.set_variable(variable.clone(), val);
                Ok(ExecSignal::None)
            }
            Stmt::ConstantDefinition { name, value } => {
                let val = self.evaluate_expr(value)?;
                self.context.set_constant(name.clone(), val);
                Ok(ExecSignal::None)
            }
            Stmt::Block(statements) => {
                for stmt in statements {
                    match self.exec(stmt)? {
                        ExecSignal::None => {}
                        signal => return Ok(signal),
                    }
                }
                Ok(ExecSignal::None)
            }
            Stmt::If { condition, then_stmt, else_stmt } => {
                let condition_val = self.evaluate_expr(condition)?;
                let is_truthy = match condition_val {
                    PhpValue::Bool(b) => b,
                    PhpValue::Null => false,
                    PhpValue::Int(0) => false,
                    PhpValue::Float(f) => f != 0.0,
                    PhpValue::String(s) => !s.is_empty() && s != "0",
                    _ => true, // Most other values are truthy
                };

                if is_truthy {
                    return self.exec(then_stmt);
                } else if let Some(else_stmt) = else_stmt {
                    return self.exec(else_stmt);
                }
                Ok(ExecSignal::None)
            }
            Stmt::While { condition, body } => {
                loop {
                    let cond_val = self.evaluate_expr(condition)?;
                    if !cond_val.is_truthy() { break; }
                    match self.exec(body)? {
                        ExecSignal::None => {}
                        ExecSignal::Break => break,
                        ExecSignal::Continue => continue,
                        ExecSignal::Return(v) => return Ok(ExecSignal::Return(v)),
                    }
                }
                Ok(ExecSignal::None)
            }
            Stmt::For { init, condition, increment, body } => {
                // Execute initialization
                if let Some(init_stmt) = init {
                    self.exec(init_stmt)?;
                }
                
                // Loop while condition is true
                loop {
                    // Check condition
                    let should_continue = if let Some(cond_expr) = condition {
                        let cond_val = self.evaluate_expr(cond_expr)?;
                        cond_val.is_truthy()
                    } else {
                        true // No condition means infinite loop
                    };
                    
                    if !should_continue {
                        break;
                    }
                    
                    // Execute body
                    match self.exec(body)? {
                        ExecSignal::None => {}
                        ExecSignal::Break => break,
                        ExecSignal::Continue => {}
                        ExecSignal::Return(v) => return Ok(ExecSignal::Return(v)),
                    }
                    
                    // Execute increment
                    if let Some(inc_expr) = increment {
                        self.evaluate_expr(inc_expr)?;
                    }
                }
                Ok(ExecSignal::None)
            }
            Stmt::Foreach { array, value_var, key_var, body } => {
                let array_value = self.evaluate_expr(array)?;
                
                // For now, handle arrays as basic iteration
                // This is a simplified implementation - real PHP foreach is more complex
                match array_value {
                    PhpValue::Array(ref arr) => {
                        for (array_key, value) in &arr.data {
                            // Set the key variable if specified
                            if let Some(key_name) = key_var {
                                let key_value = match array_key {
                                    PhpArrayKey::Int(i) => PhpValue::Int(*i),
                                    PhpArrayKey::String(s) => PhpValue::String(s.clone()),
                                };
                                self.context.set_variable(key_name.clone(), key_value);
                            }
                            
                            // Set the value variable
                            self.context.set_variable(value_var.clone(), value.clone());
                            
                            // Execute the body
                            match self.exec(body)? {
                                ExecSignal::None => {}
                                ExecSignal::Break => break,
                                ExecSignal::Continue => continue,
                                ExecSignal::Return(v) => return Ok(ExecSignal::Return(v)),
                            }
                        }
                    }
                    _ => return Err(format!("Cannot iterate over non-array value in foreach")),
                }
                Ok(ExecSignal::None)
            }
            Stmt::Switch { expression, cases, default } => {
                let discr = self.evaluate_expr(expression)?;
                let mut matched = false;
                for case in cases {
                    let case_val = self.evaluate_expr(&case.value)?;
                    if php_types::php_equals(&discr, &case_val) {
                        matched = true;
                        for stmt in &case.statements {
                            match stmt {
                                Stmt::Break => return Ok(ExecSignal::None),
                                _ => match self.exec(stmt)? {
                                    ExecSignal::None => {}
                                    ExecSignal::Break => return Ok(ExecSignal::None),
                                    ExecSignal::Continue => return Ok(ExecSignal::Continue),
                                    ExecSignal::Return(v) => return Ok(ExecSignal::Return(v)),
                                }
                            }
                        }
                        if matched { break; }
                    }
                }
                if !matched {
                    if let Some(default_stmts) = default {
                        for stmt in default_stmts {
                            match stmt {
                                Stmt::Break => break,
                                _ => match self.exec(stmt)? {
                                    ExecSignal::None => {}
                                    ExecSignal::Break => break,
                                    ExecSignal::Continue => return Ok(ExecSignal::Continue),
                                    ExecSignal::Return(v) => return Ok(ExecSignal::Return(v)),
                                }
                            }
                        }
                    }
                }
                Ok(ExecSignal::None)
            }
            Stmt::Break => Ok(ExecSignal::Break),
            Stmt::Continue => Ok(ExecSignal::Continue),
            Stmt::Return(expr_opt) => {
                let val = if let Some(expr) = expr_opt { Some(self.evaluate_expr(expr)?) } else { None };
                Ok(ExecSignal::Return(val))
            }
            Stmt::FunctionDefinition { name, parameters, body } => {
                // Store function definition
                let func = Function { params: parameters.clone(), body: *body.clone() };
                self.context.functions.insert(name.clone(), func);
                Ok(ExecSignal::None)
            }
        }
    }

    /// Evaluate an expression
    pub fn evaluate_expr(&mut self, expr: &Expr) -> Result<PhpValue, String> {
        match expr {
            Expr::Variable(name) => {
                // Undefined variable returns null (PHP notice ignored)
                Ok(self.context.get_variable(name).cloned().unwrap_or(PhpValue::Null))
            }
            Expr::Constant(name) => {
                self.context
                    .get_constant(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined constant: {}", name))
            }
            Expr::Number(n) => Ok(PhpValue::Float(*n)),
            Expr::String(s) => {
                let interpolated = self.interpolate_string(s);
                Ok(PhpValue::String(interpolated))
            }
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
                    BinaryOp::Equal => Ok(PhpValue::Bool(php_types::php_equals(&left_val, &right_val))),
                    BinaryOp::NotEqual => Ok(PhpValue::Bool(!php_types::php_equals(&left_val, &right_val))),
                    BinaryOp::LessThan => Ok(PhpValue::Bool(php_types::php_less_than(&left_val, &right_val))),
                    BinaryOp::LessThanOrEqual => Ok(PhpValue::Bool(php_types::php_less_than_or_equal(&left_val, &right_val))),
                    BinaryOp::GreaterThan => Ok(PhpValue::Bool(php_types::php_greater_than(&left_val, &right_val))),
                    BinaryOp::GreaterThanOrEqual => Ok(PhpValue::Bool(php_types::php_greater_than_or_equal(&left_val, &right_val))),
                    _ => Err("Binary operator not implemented".to_string()),
                }
            }
            Expr::FunctionCall { name, args } => {
                self.call_function(name, args)
            }
            Expr::Unary { op, operand } => {
                use php_parser::ast::UnaryOp;
                match op {
                    UnaryOp::PostIncrement => {
                        // For postfix increment: return old value, then increment
                        if let Expr::Variable(var_name) = operand.as_ref() {
                            let current_val = self.context
                                .get_variable(var_name)
                                .cloned()
                                .unwrap_or(PhpValue::Int(0));
                            
                            // Increment the variable
                            let new_val = match current_val {
                                PhpValue::Int(i) => PhpValue::Int(i + 1),
                                PhpValue::Float(f) => PhpValue::Float(f + 1.0),
                                _ => PhpValue::Int(1), // Convert to int and increment
                            };
                            
                            self.context.set_variable(var_name.clone(), new_val);
                            Ok(current_val) // Return the old value
                        } else {
                            Err("Increment operator can only be applied to variables".to_string())
                        }
                    }
                    UnaryOp::PostDecrement => {
                        // For postfix decrement: return old value, then decrement
                        if let Expr::Variable(var_name) = operand.as_ref() {
                            let current_val = self.context
                                .get_variable(var_name)
                                .cloned()
                                .unwrap_or(PhpValue::Int(0));
                            
                            // Decrement the variable
                            let new_val = match current_val {
                                PhpValue::Int(i) => PhpValue::Int(i - 1),
                                PhpValue::Float(f) => PhpValue::Float(f - 1.0),
                                _ => PhpValue::Int(-1), // Convert to int and decrement
                            };
                            
                            self.context.set_variable(var_name.clone(), new_val);
                            Ok(current_val) // Return the old value
                        } else {
                            Err("Decrement operator can only be applied to variables".to_string())
                        }
                    }
                    _ => Err("Unary operator not implemented".to_string()),
                }
            }
            Expr::Array(elements) => {
                // Build PHP array value
                let mut arr = PhpArray::new();
                for element in elements.iter() {
                    // Evaluate value
                    let value = self.evaluate_expr(&element.value)?;
                    if let Some(ref key_expr) = element.key {
                        let key_val = self.evaluate_expr(key_expr)?;
                        match key_val {
                            PhpValue::Int(i_key) => arr.insert_int(i_key, value),
                            PhpValue::String(s_key) => arr.insert_string(s_key, value),
                            // Fallback: convert to string
                            other => arr.insert_string(other.to_string(), value),
                        }
                    } else {
                        // Auto index
                        arr.push(value);
                    }
                }
                Ok(PhpValue::Array(arr))
            }
            Expr::ArrayAccess { array, index } => {
                let array_val = self.evaluate_expr(array)?;
                let index_val = self.evaluate_expr(index)?;
                match array_val {
                    PhpValue::Array(arr) => {
                        let result = match index_val {
                            PhpValue::Int(i) => arr.get_int(i).cloned(),
                            PhpValue::String(s) => arr.get_string(&s).cloned(),
                            other => {
                                // Try numeric then string
                                let numeric = other.to_int();
                                arr.get_int(numeric).cloned().or_else(|| arr.get_string(&other.to_string()).cloned())
                            }
                        };
                        Ok(result.unwrap_or(PhpValue::Null))
                    }
                    _ => Err("Attempt to access index of non-array value".to_string()),
                }
            }
            Expr::NullCoalesce { left, right } => {
                let left_val = self.evaluate_expr(left)?;
                if left_val.is_null() {
                    self.evaluate_expr(right)
                } else {
                    Ok(left_val)
                }
            }
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
            _ => {
                // User-defined function?
                if let Some(func) = self.context.functions.get(name).cloned() {
                    // Evaluate args
                    if args.len() != func.params.len() {
                        return Err(format!("Function {} expects {} arguments, got {}", name, func.params.len(), args.len()));
                    }
                    // Save current variables (shallow)
                    let saved_vars = self.context.variables.clone();
                    // Bind parameters
                    for (param, expr) in func.params.iter().zip(args.iter()) {
                        let val = self.evaluate_expr(expr)?;
                        self.context.set_variable(param.clone(), val);
                    }
                    // Execute body
                    let result = match self.exec(&func.body)? {
                        ExecSignal::Return(v) => v.unwrap_or(PhpValue::Null),
                        _ => PhpValue::Null,
                    };
                    // Restore variables (simple approach - constants/functions persist)
                    self.context.variables = saved_vars;
                    Ok(result)
                } else {
                    Err(format!("Unknown function: {}", name))
                }
            }
        }
    }

    /// Get execution output
    pub fn get_output(&self) -> &str {
        self.context.get_output()
    }

    /// Perform simple variable interpolation in strings: replaces $var with its string value
    fn interpolate_string(&self, input: &str) -> String {
        // Simple state machine scan
        let mut result = String::with_capacity(input.len());
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '$' {
                // Attempt variable name
                let mut j = i + 1;
                if j < chars.len() && (chars[j].is_ascii_alphabetic() || chars[j] == '_') {
                    j += 1;
                    while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                        j += 1;
                    }
                    let var_name: String = chars[i+1..j].iter().collect();
                    if let Some(val) = self.context.get_variable(&var_name) {
                        result.push_str(&val.to_string());
                    } else {
                        // Undefined stays as is
                        result.push('$');
                        result.push_str(&var_name);
                    }
                    i = j;
                    continue;
                } else {
                    result.push('$');
                    i += 1;
                    continue;
                }
            }
            result.push(c);
            i += 1;
        }
        result
    }

}
