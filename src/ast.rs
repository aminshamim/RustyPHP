// ast.rs

use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub enum Node {
    Block(Vec<Node>),
    Echo(Box<Node>),
    Print(Box<Node>),
    VariableAssignment(String, Box<Node>),
    Variable(String),
    Number(f64),
    StringLiteral(String),
    BinaryOperation(Box<Node>, Operator, Box<Node>),
    If(Box<Node>, Box<Node>, Option<Box<Node>>),
    While(Box<Node>, Box<Node>),
    For(Box<Node>, Box<Node>, Box<Node>, Box<Node>),
}

#[derive(Debug)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Concatenate,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Echo(content) => write!(f, "Echo({})", content),
            Node::Print(content) => write!(f, "Print({})", content),
            Node::VariableAssignment(name, value) => write!(f, "VariableAssignment({}, {})", name, value),
            Node::Variable(name) => write!(f, "Variable({})", name),
            Node::Number(value) => write!(f, "Number({})", value),
            Node::StringLiteral(value) => write!(f, "StringLiteral({})", value),
            Node::BinaryOperation(left, op, right) => write!(f, "BinaryOperation({}, {:?}, {})", left, op, right),
            Node::If(condition, then_branch, else_branch) => write!(f, "If({}, {}, {:?})", condition, then_branch, else_branch),
            Node::While(condition, body) => write!(f, "While({}, {})", condition, body),
            Node::For(init, condition, increment, body) => write!(f, "For({}, {}, {}, {})", init, condition, increment, body),
            Node::Block(statements) => {
                let statements_str = statements.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ");
                write!(f, "Block([{}])", statements_str)
            }
        }
    }
}

impl Node {
    pub fn execute(&self, context: &mut HashMap<String, f64>, string_context: &mut HashMap<String, String>) -> String {
        match self {
            Node::Block(statements) => {
                let mut output = String::new();
                for statement in statements {
                    output.push_str(&statement.execute(context, string_context));
                    output.push('\n');
                }
                output
            }
            Node::Echo(expression) => expression.evaluate_as_string(context, string_context),
            Node::Print(expression) => expression.evaluate_as_string(context, string_context),
            Node::VariableAssignment(name, value) => {
                match **value {
                    Node::Number(num) => {
                        context.insert(name.clone(), num);
                        // format!("Assigned {} to variable ${}", num, name)
                        String::new()
                    }
                    Node::StringLiteral(ref s) => {
                        string_context.insert(name.clone(), s.clone());
                        // format!("Assigned '{}' to variable ${}", s, name)
                        String::new()
                    }
                    _ => "Invalid assignment".to_string(),
                }
            }
            Node::Variable(name) => {
                if let Some(value) = context.get(name) {
                    println!("Debug: Retrieved numeric value {} for variable ${}", value, name);
                    value.to_string()
                } else if let Some(value) = string_context.get(name) {
                    println!("Debug: Retrieved string value '{}' for variable ${}", value, name);
                    value.clone()
                } else {
                    "Undefined variable".to_string()
                }
            }
            Node::Number(value) => value.to_string(),
            Node::StringLiteral(value) => value.clone(),
            Node::BinaryOperation(left, op, right) => {
                match op {
                    Operator::Concatenate => {
                        format!(
                            "{}{}",
                            left.evaluate_as_string(context, string_context),
                            right.evaluate_as_string(context, string_context)
                        )
                    }
                    _ => {
                        let left_val = left.evaluate(context, string_context);
                        let right_val = right.evaluate(context, string_context);
                        println!("Debug: Performing {:?} operation between {} and {}", op, left_val, right_val);
                        match op {
                            Operator::Add => (left_val + right_val).to_string(),
                            Operator::Subtract => (left_val - right_val).to_string(),
                            Operator::Multiply => (left_val * right_val).to_string(),
                            Operator::Divide => {
                                if right_val != 0.0 {
                                    (left_val / right_val).to_string()
                                } else {
                                    "Error: Division by zero".to_string()
                                }
                            }
                            _ => "Invalid operation".to_string(),
                        }
                    }
                }
            }
            Node::If(condition, then_branch, else_branch) => {
                if condition.evaluate(context, string_context) != 0.0 {
                    then_branch.execute(context, string_context)
                } else if let Some(else_node) = else_branch {
                    else_node.execute(context, string_context)
                } else {
                    String::new()
                }
            }
            Node::While(condition, body) => {
                let mut output = String::new();
                while condition.evaluate(context, string_context) != 0.0 {
                    output.push_str(&body.execute(context, string_context));
                }
                output
            }
            Node::For(init, condition, increment, body) => {
                let mut output = String::new();
                init.execute(context, string_context);
                while condition.evaluate(context, string_context) != 0.0 {
                    output.push_str(&body.execute(context, string_context));
                    increment.execute(context, string_context);
                }
                output
            }
        }
    }

    pub fn evaluate(&self, context: &HashMap<String, f64>, string_context: &HashMap<String, String>) -> f64 {
        match self {
            Node::Number(value) => *value,
            Node::Variable(name) => {
                if let Some(value) = context.get(name) {
                    println!("Debug: Retrieved numeric value {} for variable ${}", value, name);
                    *value
                } else {
                    println!("Debug: Undefined variable ${}", name);
                    0.0
                }
            }
            Node::BinaryOperation(left, op, right) => {
                let left_val = left.evaluate(context, string_context);
                let right_val = right.evaluate(context, string_context);
                match op {
                    Operator::Add => left_val + right_val,
                    Operator::Subtract => left_val - right_val,
                    Operator::Multiply => left_val * right_val,
                    Operator::Divide => {
                        if right_val != 0.0 {
                            left_val / right_val
                        } else {
                            println!("Error: Division by zero in operation {} / {}", left_val, right_val);
                            0.0
                        }
                    }
                    _ => {
                        println!("Unsupported operator in BinaryOperation");
                        0.0
                    }
                }
            }
            _ => 0.0,
        }
    }

    pub fn evaluate_as_string(&self, context: &HashMap<String, f64>, string_context: &HashMap<String, String>) -> String {
        match self {
            Node::StringLiteral(value) => value.clone(),
            Node::Variable(name) => {
                if let Some(value) = string_context.get(name) {
                    println!("Debug: Retrieved string value '{}' for variable ${}", value, name);
                    value.clone()
                } else if let Some(value) = context.get(name) {
                    println!("Debug: Retrieved numeric value {} for variable ${}", value, name);
                    value.to_string()
                } else {
                    println!("Debug: Undefined variable ${}", name);
                    "Undefined variable".to_string()
                }
            }
            _ => self.evaluate(context, string_context).to_string(),
        }
    }
}
