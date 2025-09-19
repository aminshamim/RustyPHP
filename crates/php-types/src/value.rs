//! PHP value types and representations

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;

/// Core PHP value type that can represent any PHP value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PhpValue {
    /// PHP null value
    Null,
    /// PHP boolean value
    Bool(bool),
    /// PHP integer value (using i64 for compatibility)
    Int(i64),
    /// PHP float value
    Float(f64),
    /// PHP string value
    String(String),
    /// PHP array value (ordered map)
    Array(PhpArray),
    /// PHP object value
    Object(PhpObject),
    /// PHP resource (placeholder for now)
    Resource(u64),
}

/// PHP array type (ordered associative array)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhpArray {
    /// Internal storage as ordered map
    pub data: HashMap<PhpArrayKey, PhpValue>,
    /// Next integer key for auto-indexing
    pub next_index: i64,
}

/// PHP array key type (string or integer)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhpArrayKey {
    /// Integer key
    Int(i64),
    /// String key
    String(String),
}

/// PHP object representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhpObject {
    /// Class name
    pub class_name: String,
    /// Object properties
    pub properties: HashMap<String, PhpValue>,
}

impl PhpValue {
    /// Create a new null value
    pub fn null() -> Self {
        PhpValue::Null
    }
    
    /// Create a new boolean value
    pub fn bool(value: bool) -> Self {
        PhpValue::Bool(value)
    }
    
    /// Create a new integer value
    pub fn int(value: i64) -> Self {
        PhpValue::Int(value)
    }
    
    /// Create a new float value
    pub fn float(value: f64) -> Self {
        PhpValue::Float(value)
    }
    
    /// Create a new string value
    pub fn string<S: Into<String>>(value: S) -> Self {
        PhpValue::String(value.into())
    }
    
    /// Create a new empty array
    pub fn array() -> Self {
        PhpValue::Array(PhpArray::new())
    }
    
    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, PhpValue::Null)
    }
    
    /// Check if the value is truthy (PHP semantics)
    pub fn is_truthy(&self) -> bool {
        match self {
            PhpValue::Null => false,
            PhpValue::Bool(b) => *b,
            PhpValue::Int(i) => *i != 0,
            PhpValue::Float(f) => *f != 0.0 && !f.is_nan(),
            PhpValue::String(s) => !s.is_empty() && s != "0",
            PhpValue::Array(arr) => !arr.is_empty(),
            PhpValue::Object(_) => true,
            PhpValue::Resource(_) => true,
        }
    }
    
    /// Convert to boolean (PHP semantics)
    pub fn to_bool(&self) -> bool {
        self.is_truthy()
    }
    
    /// Convert to integer (PHP semantics)
    pub fn to_int(&self) -> i64 {
        match self {
            PhpValue::Null => 0,
            PhpValue::Bool(b) => if *b { 1 } else { 0 },
            PhpValue::Int(i) => *i,
            PhpValue::Float(f) => *f as i64,
            PhpValue::String(s) => {
                // PHP string to int conversion is complex, simplified here
                s.parse::<i64>().unwrap_or(0)
            }
            PhpValue::Array(arr) => {
                if arr.is_empty() { 0 } else { 1 }
            }
            PhpValue::Object(_) => 1,
            PhpValue::Resource(r) => *r as i64,
        }
    }
    
    /// Convert to float (PHP semantics)
    pub fn to_float(&self) -> f64 {
        match self {
            PhpValue::Null => 0.0,
            PhpValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            PhpValue::Int(i) => *i as f64,
            PhpValue::Float(f) => *f,
            PhpValue::String(s) => {
                s.parse::<f64>().unwrap_or(0.0)
            }
            PhpValue::Array(arr) => {
                if arr.is_empty() { 0.0 } else { 1.0 }
            }
            PhpValue::Object(_) => 1.0,
            PhpValue::Resource(r) => *r as f64,
        }
    }
    
    /// Convert to string (PHP semantics)
    pub fn to_string(&self) -> String {
        match self {
            PhpValue::Null => String::new(),
            PhpValue::Bool(b) => if *b { "1".to_string() } else { String::new() },
            PhpValue::Int(i) => i.to_string(),
            PhpValue::Float(f) => f.to_string(),
            PhpValue::String(s) => s.clone(),
            PhpValue::Array(_) => "Array".to_string(),
            PhpValue::Object(_) => "Object".to_string(),
            PhpValue::Resource(r) => format!("Resource id #{}", r),
        }
    }
    
    /// Get the PHP type name
    pub fn type_name(&self) -> &'static str {
        match self {
            PhpValue::Null => "NULL",
            PhpValue::Bool(_) => "boolean",
            PhpValue::Int(_) => "integer", 
            PhpValue::Float(_) => "double",
            PhpValue::String(_) => "string",
            PhpValue::Array(_) => "array",
            PhpValue::Object(_) => "object",
            PhpValue::Resource(_) => "resource",
        }
    }
}

impl PhpArray {
    /// Create a new empty array
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            next_index: 0,
        }
    }
    
    /// Check if array is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    /// Get array length
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Insert value with integer key
    pub fn insert_int(&mut self, key: i64, value: PhpValue) {
        self.data.insert(PhpArrayKey::Int(key), value);
        if key >= self.next_index {
            self.next_index = key + 1;
        }
    }
    
    /// Insert value with string key
    pub fn insert_string<S: Into<String>>(&mut self, key: S, value: PhpValue) {
        self.data.insert(PhpArrayKey::String(key.into()), value);
    }
    
    /// Push value to end of array (auto-index)
    pub fn push(&mut self, value: PhpValue) {
        self.insert_int(self.next_index, value);
    }
    
    /// Get value by integer key
    pub fn get_int(&self, key: i64) -> Option<&PhpValue> {
        self.data.get(&PhpArrayKey::Int(key))
    }
    
    /// Get value by string key
    pub fn get_string(&self, key: &str) -> Option<&PhpValue> {
        self.data.get(&PhpArrayKey::String(key.to_string()))
    }
}

impl Default for PhpArray {
    fn default() -> Self {
        Self::new()
    }
}

impl PhpObject {
    /// Create a new object
    pub fn new<S: Into<String>>(class_name: S) -> Self {
        Self {
            class_name: class_name.into(),
            properties: HashMap::new(),
        }
    }
    
    /// Set property value
    pub fn set_property<S: Into<String>>(&mut self, name: S, value: PhpValue) {
        self.properties.insert(name.into(), value);
    }
    
    /// Get property value
    pub fn get_property(&self, name: &str) -> Option<&PhpValue> {
        self.properties.get(name)
    }
}

impl fmt::Display for PhpValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Display for PhpArrayKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PhpArrayKey::Int(i) => write!(f, "{}", i),
            PhpArrayKey::String(s) => write!(f, "{}", s),
        }
    }
}
