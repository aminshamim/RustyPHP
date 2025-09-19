//! PHP type conversion utilities

use crate::value::PhpValue;

/// Convert between PHP values with type juggling
pub trait PhpConvert {
    /// Convert to PhpValue
    fn to_php_value(self) -> PhpValue;
}

impl PhpConvert for bool {
    fn to_php_value(self) -> PhpValue {
        PhpValue::Bool(self)
    }
}

impl PhpConvert for i64 {
    fn to_php_value(self) -> PhpValue {
        PhpValue::Int(self)
    }
}

impl PhpConvert for f64 {
    fn to_php_value(self) -> PhpValue {
        PhpValue::Float(self)
    }
}

impl PhpConvert for String {
    fn to_php_value(self) -> PhpValue {
        PhpValue::String(self)
    }
}

impl PhpConvert for &str {
    fn to_php_value(self) -> PhpValue {
        PhpValue::String(self.to_string())
    }
}

/// Perform PHP-style arithmetic operations
pub fn php_add(left: &PhpValue, right: &PhpValue) -> PhpValue {
    match (left, right) {
        // If both are numbers, do numeric addition
        (PhpValue::Int(a), PhpValue::Int(b)) => PhpValue::Int(a + b),
        (PhpValue::Float(a), PhpValue::Float(b)) => PhpValue::Float(a + b),
        (PhpValue::Int(a), PhpValue::Float(b)) => PhpValue::Float(*a as f64 + b),
        (PhpValue::Float(a), PhpValue::Int(b)) => PhpValue::Float(a + *b as f64),
        
        // Convert to numbers and add
        _ => {
            let a = left.to_float();
            let b = right.to_float();
            PhpValue::Float(a + b)
        }
    }
}

/// Perform PHP-style subtraction
pub fn php_subtract(left: &PhpValue, right: &PhpValue) -> PhpValue {
    match (left, right) {
        (PhpValue::Int(a), PhpValue::Int(b)) => PhpValue::Int(a - b),
        (PhpValue::Float(a), PhpValue::Float(b)) => PhpValue::Float(a - b),
        (PhpValue::Int(a), PhpValue::Float(b)) => PhpValue::Float(*a as f64 - b),
        (PhpValue::Float(a), PhpValue::Int(b)) => PhpValue::Float(a - *b as f64),
        _ => {
            let a = left.to_float();
            let b = right.to_float();
            PhpValue::Float(a - b)
        }
    }
}

/// Perform PHP-style multiplication
pub fn php_multiply(left: &PhpValue, right: &PhpValue) -> PhpValue {
    match (left, right) {
        (PhpValue::Int(a), PhpValue::Int(b)) => PhpValue::Int(a * b),
        (PhpValue::Float(a), PhpValue::Float(b)) => PhpValue::Float(a * b),
        (PhpValue::Int(a), PhpValue::Float(b)) => PhpValue::Float(*a as f64 * b),
        (PhpValue::Float(a), PhpValue::Int(b)) => PhpValue::Float(a * *b as f64),
        _ => {
            let a = left.to_float();
            let b = right.to_float();
            PhpValue::Float(a * b)
        }
    }
}

/// Perform PHP-style division
pub fn php_divide(left: &PhpValue, right: &PhpValue) -> Result<PhpValue, String> {
    let b = right.to_float();
    if b == 0.0 {
        return Err("Division by zero".to_string());
    }
    
    let a = left.to_float();
    Ok(PhpValue::Float(a / b))
}

/// Perform PHP-style string concatenation
pub fn php_concatenate(left: &PhpValue, right: &PhpValue) -> PhpValue {
    let left_str = left.to_string();
    let right_str = right.to_string();
    PhpValue::String(format!("{}{}", left_str, right_str))
}
