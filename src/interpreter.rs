use crate::ast::{Node, Operator};

pub fn execute(node: Node) -> Result<String, String> {
    let mut output = String::new();

    match node {
        Node::Echo(content) => {
            output.push_str(&content);
        }
        Node::Print(content) => {
            output.push_str(&content);
        }
        Node::VariableAssignment(name, value) => {
            output.push_str(&format!("Assign {} to variable ${}", value.execute(), name));
        }
        Node::Variable(name) => {
            output.push_str(&format!("Retrieve value of variable ${}", name));
        }
        Node::Number(value) => {
            output.push_str(&value.to_string());
        }
        Node::StringLiteral(content) => {
            output.push_str(&content);
        }
        Node::BinaryOperation(left, op, right) => {
            let left_val = left.execute().parse::<f64>().unwrap_or(0.0);
            let right_val = right.execute().parse::<f64>().unwrap_or(0.0);
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
            };
            output.push_str(&result.to_string());
        }
        Node::If(condition, then_branch, else_branch) => {
            let condition_result = condition.execute().parse::<f64>().unwrap_or(0.0);
            if condition_result != 0.0 {
                output.push_str(&then_branch.execute());
            } else if let Some(else_node) = else_branch {
                output.push_str(&else_node.execute());
            }
        }
        Node::While(condition, body) => {
            while condition.execute().parse::<f64>().unwrap_or(0.0) != 0.0 {
                output.push_str(&body.execute());
            }
        }
        Node::For(init, condition, increment, body) => {
            init.execute();
            while condition.execute().parse::<f64>().unwrap_or(0.0) != 0.0 {
                output.push_str(&body.execute());
                increment.execute();
            }
        }
    }

    Ok(output)
}
