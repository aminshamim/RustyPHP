#[derive(Debug)]
pub enum Node {
    Echo(String),
    Print(String),
    VariableAssignment(String, Box<Node>), // e.g., $variable = expression
    Variable(String),                      // Represents a variable reference, e.g., $variable
    Number(f64),                           // Represents a number
    StringLiteral(String),                 // Represents a string literal
    BinaryOperation(Box<Node>, Operator, Box<Node>), // e.g., 1 + 2, $var * 3
    If(Box<Node>, Box<Node>, Option<Box<Node>>), // if condition, then, else (optional)
    While(Box<Node>, Box<Node>),           // while condition, body
    For(Box<Node>, Box<Node>, Box<Node>, Box<Node>), // for init, condition, increment, body
}

#[derive(Debug)]
pub enum Operator {
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
}

impl Node {
    pub fn execute(&self) -> String {
        match self {
            Node::Echo(content) => content.clone(),
            Node::Print(content) => content.clone(),
            Node::VariableAssignment(name, value) => {
                format!("Assign {} to variable ${}", value.execute(), name)
            }
            Node::Variable(name) => format!("Retrieve value of variable ${}", name),
            Node::Number(value) => value.to_string(),
            Node::StringLiteral(value) => value.clone(),
            Node::BinaryOperation(left, op, right) => {
                let left_val = left.execute().parse::<f64>().unwrap_or(0.0);
                let right_val = right.execute().parse::<f64>().unwrap_or(0.0);
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
                }
            }
            Node::If(condition, then_branch, else_branch) => {
                let condition_result = condition.execute().parse::<f64>().unwrap_or(0.0);
                if condition_result != 0.0 {
                    then_branch.execute()
                } else if let Some(else_node) = else_branch {
                    else_node.execute()
                } else {
                    String::new()
                }
            }
            Node::While(condition, body) => {
                let mut result = String::new();
                while condition.execute().parse::<f64>().unwrap_or(0.0) != 0.0 {
                    result.push_str(&body.execute());
                }
                result
            }
            Node::For(init, condition, increment, body) => {
                let mut result = String::new();
                init.execute(); // Perform initialization
                while condition.execute().parse::<f64>().unwrap_or(0.0) != 0.0 {
                    result.push_str(&body.execute());
                    increment.execute(); // Perform increment
                }
                result
            }
        }
    }
}
