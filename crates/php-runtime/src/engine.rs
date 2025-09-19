//! PHP Runtime Engine

use php_types::{PhpValue, PhpArrayKey, PhpArray};
use php_parser::ast::{Stmt, Expr, DestructTarget};
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
    /// Persistent storage for static variables per function
    static_storage: std::collections::HashMap<String, std::collections::HashMap<String, PhpValue>>,
    /// Stack tracking static vars declared in current function frame
    static_var_stack: Vec<(String, Vec<String>)>,
    /// Current function name if inside call
    current_function: Option<String>,
    /// Output buffering stack (top-of-stack is active buffer)
    output_buffers: Vec<String>,
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
        // Initialize commonly used JSON / filter constants (simplified integer values)
        ctx.set_constant("JSON_UNESCAPED_SLASHES".to_string(), PhpValue::Int(1));
        ctx.set_constant("JSON_UNESCAPED_UNICODE".to_string(), PhpValue::Int(2));
        ctx.set_constant("JSON_THROW_ON_ERROR".to_string(), PhpValue::Int(4));
        ctx.set_constant("FILTER_VALIDATE_INT".to_string(), PhpValue::Int(257));
        Self { context: ctx, static_storage: std::collections::HashMap::new(), static_var_stack: Vec::new(), current_function: None, output_buffers: Vec::new() }
    }

    /// Execute a statement
    pub fn execute_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        let signal = self.exec(stmt)?;
        match signal {
            ExecSignal::None => Ok(()),
            ExecSignal::Break | ExecSignal::Continue => Ok(()),
            ExecSignal::Return(val_opt) => {
                if let Some(val) = val_opt { self.write_output(&val.to_string()); }
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
                self.write_output(&value.to_string());
                Ok(ExecSignal::None)
            }
            Stmt::Print(expr) => {
                let value = self.evaluate_expr(expr)?;
                self.write_output(&value.to_string());
                Ok(ExecSignal::None)
            }
            Stmt::Assignment { variable, value } => {
                let val = self.evaluate_expr(value)?;
                self.context.set_variable(variable.clone(), val);
                Ok(ExecSignal::None)
            }
            Stmt::NullCoalesceAssign { variable, value } => {
                let current = self.context.get_variable(variable).cloned().unwrap_or(PhpValue::Null);
                if current.is_null() {
                    let new_val = self.evaluate_expr(value)?;
                    self.context.set_variable(variable.clone(), new_val);
                }
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
            Stmt::StaticVar { name, initial } => {
                if let Some(current_fn_name) = self.current_function.clone() {
                    // Evaluate initial expression (no borrow of static_storage yet)
                    let init_eval = if let Some(init_expr) = initial { 
                        let cloned = init_expr.clone();
                        Some(self.evaluate_expr(&cloned)?)
                    } else { None };
                    // Now borrow static_storage
                    let entry = self.static_storage.entry(current_fn_name.clone()).or_insert_with(std::collections::HashMap::new);
                    if !entry.contains_key(name) {
                        entry.insert(name.clone(), init_eval.clone().unwrap_or(PhpValue::Null));
                    }
                    if let Some(val) = entry.get(name).cloned() { self.context.set_variable(name.clone(), val); }
                    if let Some((fn_name, list)) = self.static_var_stack.last_mut() {
                        if *fn_name == current_fn_name && !list.contains(name) { list.push(name.clone()); }
                    }
                }
                Ok(ExecSignal::None)
            }
            Stmt::DestructuringAssignment { targets, value } => {
                let array_val = self.evaluate_expr(value)?;
                // Only handle array values; others ignored
                if let PhpValue::Array(arr) = array_val {
                    // Sequential index counter for plain vars
                    let mut auto_index: i64 = 0;
                    for target in targets {
                        match target {
                            DestructTarget::Var(var) => {
                                let val = arr.get_int(auto_index).cloned().unwrap_or(PhpValue::Null);
                                self.context.set_variable(var.clone(), val);
                                auto_index += 1;
                            }
                            DestructTarget::KeyVar(key, var) => {
                                let val = arr.get_string(&key).cloned().unwrap_or(PhpValue::Null);
                                self.context.set_variable(var.clone(), val);
                            }
                        }
                    }
                }
                Ok(ExecSignal::None)
            }
        }
    }

    /// Write output respecting active output buffer
    fn write_output(&mut self, text: &str) {
        if let Some(last) = self.output_buffers.last_mut() {
            last.push_str(text);
        } else {
            self.context.add_output(text);
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
                    BinaryOp::Spaceship => {
                        // Basic comparison: convert to numeric if both numeric else string cmp
                        use std::cmp::Ordering;
                        let ordering = match (&left_val, &right_val) {
                            (PhpValue::Int(a), PhpValue::Int(b)) => a.cmp(b),
                            (PhpValue::Float(a), PhpValue::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
                            (PhpValue::Int(a), PhpValue::Float(b)) => (*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal),
                            (PhpValue::Float(a), PhpValue::Int(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal),
                            _ => left_val.to_string().cmp(&right_val.to_string()),
                        };
                        let res = match ordering { Ordering::Less => -1, Ordering::Equal => 0, Ordering::Greater => 1 };
                        Ok(PhpValue::Int(res))
                    }
                    BinaryOp::BitwiseAnd => {
                        let l = left_val.to_int();
                        let r = right_val.to_int();
                        Ok(PhpValue::Int(l & r))
                    }
                    BinaryOp::BitwiseOr => {
                        let l = left_val.to_int();
                        let r = right_val.to_int();
                        Ok(PhpValue::Int(l | r))
                    }
                    BinaryOp::LogicalAnd => {
                        Ok(PhpValue::Bool(left_val.is_truthy() && right_val.is_truthy()))
                    }
                    BinaryOp::LogicalOr => {
                        Ok(PhpValue::Bool(left_val.is_truthy() || right_val.is_truthy()))
                    }
                    _ => Err("Binary operator not implemented".to_string()),
                }
            }
            Expr::FunctionCall { name, args } => {
                self.call_function(name, args)
            }
            Expr::ArrowFunction { params, body } => {
                // Represent closure as stored function with generated id
                let id = format!("__closure_{}", self.context.functions.len());
                let func = Function { params: params.clone(), body: Stmt::Return(Some(*body.clone())) }; // wrap expression in implicit return
                self.context.functions.insert(id.clone(), func);
                Ok(PhpValue::String(id)) // Temporary representation (string id). TODO: dedicated closure value type.
            }
            Expr::DynamicCall { target, args } => {
                // Evaluate target to string id referencing stored closure
                let target_val = self.evaluate_expr(target)?;
                if let PhpValue::String(id) = target_val {
                    // Look up stored function by id
                    if let Some(func) = self.context.functions.get(&id).cloned() {
                        if func.params.len() != args.len() { return Err(format!("Closure expects {} args, got {}", func.params.len(), args.len())); }
                        let saved_vars = self.context.variables.clone();
                        for (p, arg_expr) in func.params.iter().zip(args.iter()) {
                            let val = self.evaluate_expr(arg_expr)?;
                            self.context.set_variable(p.clone(), val);
                        }
                        let result = match self.exec(&func.body)? {
                            ExecSignal::Return(v) => v.unwrap_or(PhpValue::Null),
                            _ => PhpValue::Null,
                        };
                        self.context.variables = saved_vars;
                        Ok(result)
                    } else {
                        Err("Undefined closure id".into())
                    }
                } else {
                    Err("Attempted to call non-closure value".into())
                }
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
                    UnaryOp::PreIncrement => {
                        if let Expr::Variable(var_name) = operand.as_ref() {
                            let current_val = self.context.get_variable(var_name).cloned().unwrap_or(PhpValue::Int(0));
                            let new_val = match current_val {
                                PhpValue::Int(i) => PhpValue::Int(i + 1),
                                PhpValue::Float(f) => PhpValue::Float(f + 1.0),
                                _ => PhpValue::Int(1),
                            };
                            self.context.set_variable(var_name.clone(), new_val.clone());
                            Ok(new_val)
                        } else { Err("Increment operator can only be applied to variables".to_string()) }
                    }
                    UnaryOp::PreDecrement => {
                        if let Expr::Variable(var_name) = operand.as_ref() {
                            let current_val = self.context.get_variable(var_name).cloned().unwrap_or(PhpValue::Int(0));
                            let new_val = match current_val {
                                PhpValue::Int(i) => PhpValue::Int(i - 1),
                                PhpValue::Float(f) => PhpValue::Float(f - 1.0),
                                _ => PhpValue::Int(-1),
                            };
                            self.context.set_variable(var_name.clone(), new_val.clone());
                            Ok(new_val)
                        } else { Err("Decrement operator can only be applied to variables".to_string()) }
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
                    // PHP would emit a notice and return null; we silently return null for now
                    _ => Ok(PhpValue::Null),
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
            Expr::Ternary { condition, then_expr, else_expr } => {
                let cond_val = self.evaluate_expr(condition)?;
                let is_truthy = cond_val.is_truthy();
                if is_truthy {
                    if let Some(then_e) = then_expr { self.evaluate_expr(then_e) } else { Ok(cond_val) }
                } else {
                    self.evaluate_expr(else_expr)
                }
            }
            Expr::Match { subject, arms, default_arm } => {
                let subj_val = self.evaluate_expr(subject)?;
                for (conds, result) in arms {
                    for cond in conds {
                        let cval = self.evaluate_expr(cond)?;
                        if php_types::php_equals(&subj_val, &cval) {
                            return self.evaluate_expr(result);
                        }
                    }
                }
                if let Some(def) = default_arm { return self.evaluate_expr(def); }
                Ok(PhpValue::Null)
            }
            Expr::Yield { value } => {
                // Evaluate yielded value but ignore (no generator semantics yet)
                let _ = self.evaluate_expr(value)?;
                Ok(PhpValue::Null)
            }
            Expr::MethodCall { target: _target, method: _method, args } => {
                // Evaluate args for side effects
                for a in args { let _ = self.evaluate_expr(a)?; }
                Ok(PhpValue::Null) // placeholder
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
            "isset" => {
                // isset can take one or more variables/expressions. We'll evaluate each; if any is undefined or null -> false.
                if args.is_empty() { return Ok(PhpValue::Bool(false)); }
                for expr in args {
                    // Only treat simple variable references as per minimal implementation; other expressions fallback to evaluated value
                    let val = self.evaluate_expr(expr)?;
                    if val.is_null() { return Ok(PhpValue::Bool(false)); }
                }
                Ok(PhpValue::Bool(true))
            }
            "parse_str" => {
                // Expect 2 arguments: query string, target array variable (passed as variable expression in source)
                if args.len() != 2 {
                    return Err("parse_str() expects exactly 2 arguments".into());
                }
                // Evaluate first argument to string
                let query_val = self.evaluate_expr(&args[0])?;
                let query_str = query_val.to_string();
                // Determine variable name from second arg expression (must be variable)
                let target_var_name = match &args[1] {
                    Expr::Variable(name) => name.clone(),
                    _ => return Err("parse_str() second argument must be a variable".into()),
                };
                // Parse query string into PhpArray
                let mut arr = PhpArray::new();
                for pair in query_str.split('&') {
                    if pair.is_empty() { continue; }
                    let mut kv = pair.splitn(2, '=');
                    let raw_key = kv.next().unwrap_or("");
                    let raw_val = kv.next().unwrap_or("");
                    let key = Self::percent_decode(raw_key);
                    let val_str = Self::percent_decode(raw_val);
                    arr.insert_string(key, PhpValue::String(val_str));
                }
                self.context.set_variable(target_var_name, PhpValue::Array(arr));
                Ok(PhpValue::Null)
            }
            "array_merge" => {
                if args.is_empty() { return Ok(PhpValue::Array(PhpArray::new())); }
                let mut result = PhpArray::new();
                for expr in args {
                    let val = self.evaluate_expr(expr)?;
                    if let PhpValue::Array(arr) = val {
                        // For simplicity: numeric keys appended (preserving insertion order), string keys overwrite
                        for (k, v) in arr.data.iter() {
                            match k {
                                PhpArrayKey::Int(_) => { result.push(v.clone()); }
                                PhpArrayKey::String(s) => { result.insert_string(s.clone(), v.clone()); }
                            }
                        }
                    }
                }
                Ok(PhpValue::Array(result))
            }
            "getenv" => {
                if args.len() != 1 { return Err("getenv() expects exactly 1 argument".into()); }
                let name_val = self.evaluate_expr(&args[0])?;
                let key = name_val.to_string();
                match std::env::var(&key) {
                    Ok(v) => Ok(PhpValue::String(v)),
                    Err(_) => Ok(PhpValue::Bool(false)),
                }
            }
            "array_sum" => {
                if args.len() != 1 { return Err("array_sum() expects exactly 1 argument".into()); }
                let arr_val = self.evaluate_expr(&args[0])?;
                match arr_val {
                    PhpValue::Array(arr) => {
                        let mut sum_f: f64 = 0.0;
                        for (_, v) in arr.data.iter() {
                            sum_f += match v {
                                PhpValue::Int(i) => *i as f64,
                                PhpValue::Float(f) => *f,
                                PhpValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
                                PhpValue::Bool(b) => if *b { 1.0 } else { 0.0 },
                                _ => 0.0,
                            };
                        }
                        // If sum is integer representable, return Int else Float
                        if (sum_f.fract() - 0.0).abs() < std::f64::EPSILON { Ok(PhpValue::Int(sum_f as i64)) } else { Ok(PhpValue::Float(sum_f)) }
                    }
                    _ => Ok(PhpValue::Int(0))
                }
            }
            "str_repeat" => {
                if args.len() != 2 { return Err("str_repeat() expects exactly 2 arguments".into()); }
                let input_val = self.evaluate_expr(&args[0])?;
                let times_val = self.evaluate_expr(&args[1])?;
                let s = input_val.to_string();
                let times: i64 = match times_val {
                    PhpValue::Int(i) => i,
                    PhpValue::Float(f) => f as i64,
                    PhpValue::String(ref st) => st.parse::<i64>().unwrap_or(0),
                    _ => 0,
                };
                if times <= 0 { return Ok(PhpValue::String(String::new())); }
                // Basic guard to avoid huge allocations; mimic simplified behavior
                if times as usize > 100_000 { return Err("str_repeat(): Second argument too large".into()); }
                let repeated = s.repeat(times as usize);
                Ok(PhpValue::String(repeated))
            }
            "usort" => {
                if args.len() != 2 { return Err("usort() expects exactly 2 arguments".into()); }
                use php_parser::ast::Expr as AstExpr;
                let arr_expr = &args[0];
                // Evaluate array
                let arr_value = self.evaluate_expr(arr_expr)?;
                if let PhpValue::Array(arr) = arr_value {
                    // Extract values
                    let mut values: Vec<PhpValue> = arr.data.iter().map(|(_, v)| v.clone()).collect();
                    // Very naive sort: compare string representations (mimics comparator returning strcmp semantics)
                    let len = values.len();
                    for i in 0..len {
                        for j in 0..len - 1 - i {
                            let a_s = values[j].to_string();
                            let b_s = values[j + 1].to_string();
                            if a_s > b_s { values.swap(j, j + 1); }
                        }
                    }
                    // Rebuild numeric array
                    let mut new_arr = PhpArray::new();
                    for v in values { new_arr.push(v); }
                    if let AstExpr::Variable(var_name) = arr_expr { self.context.set_variable(var_name.clone(), PhpValue::Array(new_arr)); }
                    Ok(PhpValue::Bool(true))
                } else { Ok(PhpValue::Bool(false)) }
            }
            "iterator_to_array" => {
                if args.len() < 1 { return Err("iterator_to_array() expects at least 1 argument".into()); }
                let val = self.evaluate_expr(&args[0])?;
                match val {
                    PhpValue::Array(a) => Ok(PhpValue::Array(a)),
                    _ => Ok(PhpValue::Array(PhpArray::new()))
                }
            }
            "json_encode" => {
                if args.is_empty() { return Err("json_encode() expects at least 1 argument".into()); }
                let value = self.evaluate_expr(&args[0])?;
                let mut flags: i64 = 0;
                if args.len() >= 2 { flags = match self.evaluate_expr(&args[1])? { PhpValue::Int(i) => i, PhpValue::Float(f) => f as i64, _ => 0 }; }
                let unescaped_slashes = (flags & 1) != 0; // using placeholder bit positions (not exact PHP mapping)
                let unescaped_unicode = (flags & 2) != 0;
                fn escape_str(s: &str, unesc_slash: bool, unesc_unicode: bool) -> String {
                    let mut out = String::new();
                    for ch in s.chars() {
                        match ch {
                            '"' => out.push_str("\\\""),
                            '\\' => out.push_str("\\\\"),
                            '/' => { if unesc_slash { out.push('/'); } else { out.push_str("\\/"); } },
                            '\n' => out.push_str("\\n"),
                            '\r' => out.push_str("\\r"),
                            '\t' => out.push_str("\\t"),
                            c if c < ' ' => {
                                out.push_str(&format!("\\u{:04x}", c as u32));
                            }
                            c => {
                                if !unesc_unicode && (c as u32) > 0x7F { out.push_str(&format!("\\u{:04x}", c as u32)); } else { out.push(c); }
                            }
                        }
                    }
                    out
                }
                fn encode(value: &PhpValue, unesc_slash: bool, unesc_unicode: bool) -> String {
                    match value {
                        PhpValue::Null => "null".to_string(),
                        PhpValue::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
                        PhpValue::Int(i) => i.to_string(),
                        PhpValue::Float(f) => {
                            if f.is_finite() { f.to_string() } else { "null".to_string() }
                        }
                        PhpValue::String(s) => format!("\"{}\"", escape_str(s, unesc_slash, unesc_unicode)),
                        PhpValue::Array(arr) => {
                            // Detect list: keys 0..n-1 all int sequential
                            let mut is_list = true;
                            let mut expected_index: i64 = 0;
                            for (k, _) in arr.data.iter() {
                                match k {
                                    PhpArrayKey::Int(i) => { if *i != expected_index { is_list = false; break; } expected_index += 1; }
                                    PhpArrayKey::String(_) => { is_list = false; break; }
                                }
                            }
                            if is_list {
                                let mut parts = Vec::new();
                                for (_, v) in arr.data.iter() { parts.push(encode(v, unesc_slash, unesc_unicode)); }
                                format!("[{}]", parts.join(","))
                            } else {
                                let mut parts = Vec::new();
                                for (k, v) in arr.data.iter() {
                                    let key_str = match k { PhpArrayKey::Int(i) => i.to_string(), PhpArrayKey::String(s) => s.clone() };
                                    parts.push(format!("\"{}\":{}", escape_str(&key_str, unesc_slash, unesc_unicode), encode(v, unesc_slash, unesc_unicode)));
                                }
                                format!("{{{}}}", parts.join(","))
                            }
                        }
                        _ => "null".to_string(),
                    }
                }
                let json = encode(&value, unescaped_slashes, unescaped_unicode);
                Ok(PhpValue::String(json))
            }
            "json_decode" => {
                if args.is_empty() { return Err("json_decode() expects at least 1 argument".into()); }
                let json_val = self.evaluate_expr(&args[0])?;
                let json_str = json_val.to_string();
                // second param assoc = bool (default true for us for simpler mapping)
                let mut assoc = true;
                if args.len() >= 2 {
                    assoc = match self.evaluate_expr(&args[1])? { PhpValue::Bool(b) => b, PhpValue::Int(i) => i != 0, _ => true };
                }
                match serde_json::from_str::<serde_json::Value>(&json_str) {
                    Ok(v) => {
                        fn to_php(v: &serde_json::Value, assoc: bool) -> PhpValue {
                            match v {
                                serde_json::Value::Null => PhpValue::Null,
                                serde_json::Value::Bool(b) => PhpValue::Bool(*b),
                                serde_json::Value::Number(n) => {
                                    if let Some(i) = n.as_i64() { PhpValue::Int(i) } else if let Some(f) = n.as_f64() { PhpValue::Float(f) } else { PhpValue::Null }
                                }
                                serde_json::Value::String(s) => PhpValue::String(s.clone()),
                                serde_json::Value::Array(arr) => {
                                    let mut a = PhpArray::new();
                                    for item in arr { a.push(to_php(item, assoc)); }
                                    PhpValue::Array(a)
                                }
                                serde_json::Value::Object(map) => {
                                    let mut a = PhpArray::new();
                                    for (k, val) in map.iter() {
                                        if assoc {
                                            a.insert_string(k.clone(), to_php(val, assoc));
                                        }
                                    }
                                    PhpValue::Array(a)
                                }
                            }
                        }
                        Ok(to_php(&v, assoc))
                    }
                    Err(_) => Ok(PhpValue::Null)
                }
            }
            "set_error_handler" => {
                // Accept any callable, ignore for now, return null (previous handler)
                Ok(PhpValue::Null)
            }
            "preg_match" => {
                // preg_match(pattern, subject, matches?)
                if args.len() < 2 { return Err("preg_match() expects at least 2 parameters".into()); }
                use php_parser::ast::Expr as AstExpr;
                let pattern_raw = self.evaluate_expr(&args[0])?.to_string();
                let subject = self.evaluate_expr(&args[1])?.to_string();
                // Strip delimiters if pattern like /.../
                let pattern = if pattern_raw.len() >= 2 && pattern_raw.starts_with('/') {
                    if let Some(last) = pattern_raw.rfind('/') { pattern_raw[1..last].to_string() } else { pattern_raw.clone() }
                } else { pattern_raw.clone() };
                match regex::Regex::new(&pattern) {
                    Ok(re) => {
                        if let Some(caps) = re.captures(&subject) {
                            // If third argument variable provided populate
                            if args.len() >= 3 {
                                if let AstExpr::Variable(var_name) = &args[2] {
                                    let mut arr = PhpArray::new();
                                    for (i, cap) in caps.iter().enumerate() {
                                        if let Some(m) = cap { arr.insert_int(i as i64, PhpValue::String(m.as_str().to_string())); }
                                    }
                                    self.context.set_variable(var_name.clone(), PhpValue::Array(arr));
                                }
                            }
                            Ok(PhpValue::Int(1))
                        } else { Ok(PhpValue::Int(0)) }
                    }
                    Err(_) => Ok(PhpValue::Int(0))
                }
            }
            "filter_var" => {
                // filter_var(value, filter) minimal: only FILTER_VALIDATE_INT
                if args.len() < 2 { return Err("filter_var() expects at least 2 arguments".into()); }
                let val = self.evaluate_expr(&args[0])?;
                let filter = self.evaluate_expr(&args[1])?;
                let filter_id = match filter { PhpValue::Int(i) => i, _ => 0 };
                // We defined FILTER_VALIDATE_INT constant as 257
                if filter_id == 257 {
                    let s = val.to_string();
                    if let Ok(i) = s.parse::<i64>() { Ok(PhpValue::Int(i)) } else { Ok(PhpValue::Bool(false)) }
                } else {
                    Ok(val) // fallback returns original
                }
            }
            "ob_start" => {
                self.output_buffers.push(String::new());
                Ok(PhpValue::Bool(true))
            }
            "ob_get_clean" => {
                if let Some(buf) = self.output_buffers.pop() { Ok(PhpValue::String(buf)) } else { Ok(PhpValue::Bool(false)) }
            }
            "printf" => {
                if args.is_empty() { return Ok(PhpValue::Int(0)); }
                let fmt = self.evaluate_expr(&args[0])?.to_string();
                let mut arg_index = 1usize;
                let mut out = String::new();
                let chars: Vec<char> = fmt.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    if chars[i] == '%' {
                        i += 1;
                        if i >= chars.len() { break; }
                        let spec = chars[i];
                        match spec {
                            '%' => { out.push('%'); }
                            's' | 'd' | 'f' => {
                                if arg_index < args.len() {
                                    let val = self.evaluate_expr(&args[arg_index])?;
                                    let formatted = match spec {
                                        'd' => val.to_int().to_string(),
                                        'f' => {
                                            let f = val.to_float();
                                            format!("{}", f)
                                        }
                                        _ => val.to_string(),
                                    };
                                    out.push_str(&formatted);
                                    arg_index += 1;
                                }
                            }
                            _ => { out.push('%'); out.push(spec); }
                        }
                    } else {
                        out.push(chars[i]);
                    }
                    i += 1;
                }
                let len = out.len() as i64;
                self.write_output(&out);
                Ok(PhpValue::Int(len))
            }
            "implode" => {
                if args.is_empty() { return Err("implode() expects at least 1 argument".into()); }
                let (glue, pieces_expr_index) = if args.len() == 1 { ("".to_string(), 0usize) } else { (self.evaluate_expr(&args[0])?.to_string(), 1usize) };
                let pieces_val = self.evaluate_expr(&args[pieces_expr_index])?;
                match pieces_val {
                    PhpValue::Array(arr) => {
                        let mut parts = Vec::new();
                        for (_, v) in arr.data.iter() { parts.push(v.to_string()); }
                        Ok(PhpValue::String(parts.join(&glue)))
                    }
                    other => Ok(PhpValue::String(other.to_string()))
                }
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
                    let prev_function = self.current_function.clone();
                    self.current_function = Some(name.to_string());
                    self.static_var_stack.push((name.to_string(), Vec::new()));
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
                    // Persist static vars back
                    if let Some((fn_name, vars)) = self.static_var_stack.pop() {
                        if let Some(store) = self.static_storage.get_mut(&fn_name) {
                            for var in vars {
                                if let Some(val) = self.context.get_variable(&var).cloned() {
                                    store.insert(var, val);
                                }
                            }
                        }
                    }
                    self.current_function = prev_function;
                    // Restore variables (simple approach - constants/functions persist)
                    self.context.variables = saved_vars;
                    Ok(result)
                } else {
                    Err(format!("Unknown function: {}", name))
                }
            }
        }
    }

    /// Simple percent-decoding helper (handles + -> space and %XX hex sequences)
    fn percent_decode(input: &str) -> String {
        let mut bytes = Vec::with_capacity(input.len());
        let mut chars = input.as_bytes().iter().cloned().peekable();
        while let Some(b) = chars.next() {
            match b {
                b'+' => bytes.push(b' '),
                b'%' => {
                    let h1 = chars.next();
                    let h2 = chars.next();
                    if let (Some(c1), Some(c2)) = (h1, h2) {
                        let hex = [c1, c2];
                        if let Ok(s) = std::str::from_utf8(&hex) {
                            if let Ok(v) = u8::from_str_radix(s, 16) { bytes.push(v); continue; }
                        }
                        // Fallback: push literal
                        bytes.push(b'%'); bytes.push(c1); bytes.push(c2);
                    } else {
                        bytes.push(b'%');
                        if let Some(c1) = h1 { bytes.push(c1); }
                        if let Some(c2) = h2 { bytes.push(c2); }
                    }
                }
                _ => bytes.push(b),
            }
        }
        String::from_utf8_lossy(&bytes).into_owned()
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
