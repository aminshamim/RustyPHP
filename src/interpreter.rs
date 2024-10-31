// interpreter.rs

use crate::ast::{Node, Operator};
use std::collections::HashMap;

pub fn execute(node: Node, context: &mut HashMap<String, f64>, string_context: &mut HashMap<String, String>) -> Result<String, String> {
    let mut output = String::new();
    match node {
        Node::Block(statements) => {
            for statement in statements {
                output.push_str(&statement.execute(context, string_context));
                output.push('\n');
            }
        }
        Node::Echo(content) => {
            output.push_str(&content.evaluate_as_string(context, string_context));
        }
        Node::Print(content) => {
            output.push_str(&content.evaluate_as_string(context, string_context));
        }
        Node::VariableAssignment(name, value) => {
            match *value {
                Node::Number(num) => {
                    context.insert(name.clone(), num);
                    println!("Debug: Assigned numeric value {} to variable ${}", num, name);
                    output.push_str(&format!("Assigned {} to variable ${}", num, name));
                }
                Node::StringLiteral(ref s) => {
                    string_context.insert(name.clone(), s.clone());
                    println!("Debug: Assigned string value '{}' to variable ${}", s, name);
                    output.push_str(&format!("Assigned '{}' to variable ${}", s, name));
                }
                _ => output.push_str("Invalid assignment"),
            }
        }
        Node::BinaryOperation(left, op, right) => {
            let left_val = left.evaluate(context, string_context );
            let right_val = right.evaluate(context, string_context);
            let result = match op {
                Operator::Add => left_val + right_val,
                Operator::Subtract => left_val - right_val,
                Operator::Multiply => left_val * right_val,
                Operator::Divide => {
                    if right_val != 0.0 {
                        left_val / right_val
                    } else {
                        return Err("Error: Division by zero".to_string());
                    }
                }
                _ => return Err("Unsupported operator".to_string()),
            };
            output.push_str(&result.to_string());
        }
        _ => output.push_str("Unknown command"),
    }

    Ok(output)
}
