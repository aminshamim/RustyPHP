//! Playground request handlers

use actix_web::{HttpResponse, web};
use serde_json::json;
use std::time::Instant;
use tera::Tera;
use std::collections::HashMap;

/// Serve the playground HTML page
pub async fn playground(tera: web::Data<Tera>) -> HttpResponse {
    let ctx = tera::Context::new();
    let rendered = tera.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

/// Execute PHP code and return results
pub async fn execute_code(code: web::Json<String>) -> HttpResponse {
    let start = Instant::now();

    // Use the new lexer and parser
    let tokens = match php_lexer::lex(&code) {
        Ok(tokens) => tokens,
        Err(e) => {
            let elapsed = start.elapsed().as_micros();
            let response = json!({ 
                "output": format!("Lexer Error: {}", e), 
                "response_time": elapsed 
            });
            return HttpResponse::Ok().json(response);
        }
    };

    // Parse with legacy compatibility
    let result = match php_parser::parse_legacy(tokens) {
        Ok(ast) => {
            // For now, we'll use the legacy execution until we migrate the runtime
            let mut context = HashMap::new();
            let mut string_context = HashMap::new();
            execute_legacy_ast(ast, &mut context, &mut string_context)
        }
        Err(e) => Err(e),
    };

    let elapsed = start.elapsed().as_micros();
    let response = match result {
        Ok(output) => json!({ "output": output, "response_time": elapsed }),
        Err(error) => json!({ "output": format!("Error: {}", error), "response_time": elapsed }),
    };

    HttpResponse::Ok().json(response)
}

/// Execute legacy AST (temporary compatibility layer)
fn execute_legacy_ast(
    node: php_parser::LegacyNode,
    context: &mut HashMap<String, f64>,
    string_context: &mut HashMap<String, String>,
) -> Result<String, String> {
    use php_parser::{LegacyNode, LegacyOperator};
    
    match node {
        LegacyNode::Block(statements) => {
            let mut output = String::new();
            for statement in statements {
                let result = execute_legacy_ast(statement, context, string_context)?;
                if !result.is_empty() {
                    output.push_str(&result);
                    output.push('\n');
                }
            }
            Ok(output)
        }
        LegacyNode::Echo(expr) => {
            let value = evaluate_legacy_expr(*expr, context, string_context)?;
            Ok(value)
        }
        LegacyNode::Print(expr) => {
            let value = evaluate_legacy_expr(*expr, context, string_context)?;
            Ok(value)
        }
        LegacyNode::VariableAssignment(name, value) => {
            match value.as_ref() {
                LegacyNode::Number(num) => {
                    context.insert(name, *num);
                    Ok(String::new())
                }
                LegacyNode::StringLiteral(s) => {
                    string_context.insert(name, s.clone());
                    Ok(String::new())
                }
                _ => {
                    let val = evaluate_legacy_expr(*value, context, string_context)?;
                    string_context.insert(name, val);
                    Ok(String::new())
                }
            }
        }
        LegacyNode::BinaryOperation(left, op, right) => {
            let left_val = evaluate_legacy_expr(*left, context, string_context)?;
            let right_val = evaluate_legacy_expr(*right, context, string_context)?;
            
            match op {
                LegacyOperator::Concatenate => Ok(format!("{}{}", left_val, right_val)),
                _ => {
                    // Try to parse as numbers for arithmetic
                    let left_num: f64 = left_val.parse().unwrap_or(0.0);
                    let right_num: f64 = right_val.parse().unwrap_or(0.0);
                    
                    let result = match op {
                        LegacyOperator::Add => left_num + right_num,
                        LegacyOperator::Subtract => left_num - right_num,
                        LegacyOperator::Multiply => left_num * right_num,
                        LegacyOperator::Divide => {
                            if right_num != 0.0 {
                                left_num / right_num
                            } else {
                                return Err("Division by zero".to_string());
                            }
                        }
                        LegacyOperator::Concatenate => unreachable!(),
                    };
                    Ok(result.to_string())
                }
            }
        }
        _ => Ok("Unsupported operation".to_string()),
    }
}

/// Evaluate legacy expression
fn evaluate_legacy_expr(
    expr: php_parser::LegacyNode,
    context: &HashMap<String, f64>,
    string_context: &HashMap<String, String>,
) -> Result<String, String> {
    use php_parser::LegacyNode;
    
    match expr {
        LegacyNode::Number(n) => Ok(n.to_string()),
        LegacyNode::StringLiteral(s) => Ok(s),
        LegacyNode::Variable(name) => {
            if let Some(value) = string_context.get(&name) {
                Ok(value.clone())
            } else if let Some(value) = context.get(&name) {
                Ok(value.to_string())
            } else {
                Ok(String::new()) // PHP returns empty string for undefined variables in string context
            }
        }
        LegacyNode::BinaryOperation(left, op, right) => {
            execute_legacy_ast(LegacyNode::BinaryOperation(left, op, right), 
                             &mut context.clone(), &mut string_context.clone())
        }
        _ => Ok("unsupported".to_string()),
    }
}
