//! High-Performance Runtime Engine Extensions
//! Optimizations for 100k+ RPS PHP execution

use php_types::{PhpValue, PhpArrayKey, PhpArray};
use std::collections::HashMap;
use std::sync::Arc;

/// Optimized variable storage with faster lookups
pub struct OptimizedVariableStore {
    // Use FxHashMap for better performance than std HashMap
    variables: fxhash::FxHashMap<String, PhpValue>,
    // Pre-allocated commonly used variables
    common_vars: [Option<PhpValue>; 32],
    // String interner for variable names to reduce allocations
    name_interner: StringInterner,
}

/// String interner to reduce allocations for variable names
pub struct StringInterner {
    strings: Vec<String>,
    map: fxhash::FxHashMap<String, usize>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            map: fxhash::FxHashMap::default(),
        }
    }
    
    pub fn intern(&mut self, s: String) -> usize {
        if let Some(&id) = self.map.get(&s) {
            id
        } else {
            let id = self.strings.len();
            self.map.insert(s.clone(), id);
            self.strings.push(s);
            id
        }
    }
    
    pub fn get(&self, id: usize) -> Option<&str> {
        self.strings.get(id).map(|s| s.as_str())
    }
}

impl OptimizedVariableStore {
    pub fn new() -> Self {
        Self {
            variables: fxhash::FxHashMap::default(),
            common_vars: [None; 32],
            name_interner: StringInterner::new(),
        }
    }
    
    /// Fast variable lookup for common variable names
    pub fn get_optimized(&self, name: &str) -> Option<&PhpValue> {
        // Fast path for single-character variables ($a, $b, etc.)
        if name.len() == 1 {
            let idx = name.chars().next().unwrap() as usize;
            if idx < 256 && idx >= 97 && idx <= 122 { // a-z
                let common_idx = idx - 97;
                if common_idx < 32 {
                    return self.common_vars[common_idx].as_ref();
                }
            }
        }
        
        // Fall back to hash map
        self.variables.get(name)
    }
    
    pub fn set_optimized(&mut self, name: String, value: PhpValue) {
        // Fast path for single-character variables
        if name.len() == 1 {
            let idx = name.chars().next().unwrap() as usize;
            if idx < 256 && idx >= 97 && idx <= 122 { // a-z
                let common_idx = idx - 97;
                if common_idx < 32 {
                    self.common_vars[common_idx] = Some(value);
                    return;
                }
            }
        }
        
        self.variables.insert(name, value);
    }
    
    pub fn clear(&mut self) {
        self.variables.clear();
        for slot in &mut self.common_vars {
            *slot = None;
        }
    }
}

/// Memory pool for reusing PHP values to reduce allocations
pub struct PhpValuePool {
    strings: Vec<String>,
    arrays: Vec<PhpArray>,
    integers: Vec<i64>,
    floats: Vec<f64>,
}

impl PhpValuePool {
    pub fn new() -> Self {
        Self {
            strings: Vec::with_capacity(1000),
            arrays: Vec::with_capacity(100),
            integers: Vec::with_capacity(1000),
            floats: Vec::with_capacity(100),
        }
    }
    
    pub fn get_string(&mut self) -> String {
        self.strings.pop().unwrap_or_default()
    }
    
    pub fn return_string(&mut self, mut s: String) {
        s.clear();
        if self.strings.len() < 1000 {
            self.strings.push(s);
        }
    }
    
    pub fn get_array(&mut self) -> PhpArray {
        self.arrays.pop().unwrap_or_else(PhpArray::new)
    }
    
    pub fn return_array(&mut self, mut arr: PhpArray) {
        arr.data.clear();
        if self.arrays.len() < 100 {
            self.arrays.push(arr);
        }
    }
}

/// Optimized expression evaluator with reduced allocations
pub struct OptimizedExpressionEvaluator {
    value_pool: PhpValuePool,
    // Pre-compiled common expressions
    compiled_cache: fxhash::FxHashMap<String, CompiledExpression>,
}

#[derive(Clone)]
pub enum CompiledExpression {
    Constant(PhpValue),
    Variable(String),
    BinaryOp {
        left: Box<CompiledExpression>,
        op: php_parser::ast::BinaryOp,
        right: Box<CompiledExpression>,
    },
}

impl OptimizedExpressionEvaluator {
    pub fn new() -> Self {
        Self {
            value_pool: PhpValuePool::new(),
            compiled_cache: fxhash::FxHashMap::default(),
        }
    }
    
    /// Compile expression to optimized form
    pub fn compile(&mut self, expr: &php_parser::ast::Expr) -> CompiledExpression {
        match expr {
            php_parser::ast::Expr::Number(n) => {
                CompiledExpression::Constant(PhpValue::Float(*n))
            }
            php_parser::ast::Expr::String(s) => {
                CompiledExpression::Constant(PhpValue::String(s.clone()))
            }
            php_parser::ast::Expr::Variable(name) => {
                CompiledExpression::Variable(name.clone())
            }
            php_parser::ast::Expr::Binary { left, op, right } => {
                CompiledExpression::BinaryOp {
                    left: Box::new(self.compile(left)),
                    op: *op,
                    right: Box::new(self.compile(right)),
                }
            }
            _ => {
                // Fallback for complex expressions
                CompiledExpression::Constant(PhpValue::Null)
            }
        }
    }
    
    /// Evaluate compiled expression with optimizations
    pub fn evaluate_compiled(
        &mut self,
        expr: &CompiledExpression,
        vars: &OptimizedVariableStore,
    ) -> Result<PhpValue, String> {
        match expr {
            CompiledExpression::Constant(val) => Ok(val.clone()),
            CompiledExpression::Variable(name) => {
                Ok(vars.get_optimized(name).cloned().unwrap_or(PhpValue::Null))
            }
            CompiledExpression::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_compiled(left, vars)?;
                let right_val = self.evaluate_compiled(right, vars)?;
                
                use php_parser::ast::BinaryOp;
                match op {
                    BinaryOp::Add => Ok(php_types::php_add(&left_val, &right_val)),
                    BinaryOp::Subtract => Ok(php_types::php_subtract(&left_val, &right_val)),
                    BinaryOp::Multiply => Ok(php_types::php_multiply(&left_val, &right_val)),
                    BinaryOp::Concatenate => Ok(php_types::php_concatenate(&left_val, &right_val)),
                    _ => Err("Unsupported binary operation".to_string()),
                }
            }
        }
    }
}

/// Extensions to the main engine for high performance
impl php_runtime::Engine {
    /// Reset engine state for reuse in pool
    pub fn reset(&mut self) {
        // Implementation would reset all internal state
        // This is a placeholder - actual implementation would be in the engine module
    }
    
    /// Execute with performance optimizations enabled
    pub fn execute_optimized(&mut self, statements: &[php_parser::ast::Stmt]) -> Result<String, String> {
        // Fast path optimizations for common patterns:
        
        // 1. Single echo statement
        if statements.len() == 1 {
            if let php_parser::ast::Stmt::Echo(expr) = &statements[0] {
                if let php_parser::ast::Expr::String(s) = expr {
                    return Ok(s.clone());
                }
            }
        }
        
        // 2. Variable assignment + echo
        if statements.len() == 2 {
            // Pattern: $var = "value"; echo $var;
            // Could be optimized to direct output
        }
        
        // Fall back to normal execution
        for stmt in statements {
            self.execute_stmt(stmt)?;
        }
        
        Ok(self.get_output().to_string())
    }
}

/// JIT compilation hints for hot code paths
pub struct JitCompiler {
    hot_functions: fxhash::FxHashMap<String, usize>, // Function name -> call count
    compilation_threshold: usize,
}

impl JitCompiler {
    pub fn new() -> Self {
        Self {
            hot_functions: fxhash::FxHashMap::default(),
            compilation_threshold: 100, // Compile after 100 calls
        }
    }
    
    /// Track function calls and trigger compilation for hot functions
    pub fn track_function_call(&mut self, function_name: &str) -> bool {
        let count = self.hot_functions.entry(function_name.to_string()).or_insert(0);
        *count += 1;
        
        *count >= self.compilation_threshold
    }
    
    /// Compile function to optimized bytecode
    pub fn compile_function(&mut self, _function_name: &str, _body: &php_parser::ast::Stmt) {
        // TODO: Implement bytecode compilation for hot functions
        // This would generate optimized instruction sequences
    }
}

/// Benchmark and profiling utilities
pub mod profiling {
    use std::time::Instant;
    
    pub struct ExecutionProfiler {
        start_time: Instant,
        checkpoints: Vec<(String, Instant)>,
    }
    
    impl ExecutionProfiler {
        pub fn new() -> Self {
            Self {
                start_time: Instant::now(),
                checkpoints: Vec::new(),
            }
        }
        
        pub fn checkpoint(&mut self, name: &str) {
            self.checkpoints.push((name.to_string(), Instant::now()));
        }
        
        pub fn report(&self) -> String {
            let mut report = String::new();
            let mut last_time = self.start_time;
            
            for (name, time) in &self.checkpoints {
                let elapsed = time.duration_since(last_time).as_micros();
                report.push_str(&format!("{}: {}μs\n", name, elapsed));
                last_time = *time;
            }
            
            let total = self.start_time.elapsed().as_micros();
            report.push_str(&format!("Total: {}μs", total));
            
            report
        }
    }
}