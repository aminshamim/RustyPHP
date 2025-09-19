//! Playground request handlers

use actix_web::{HttpResponse, web};
use serde_json::json;
use std::time::Instant;
use tera::Tera;
use php_runtime::Engine;
use serde::Deserialize;

/// Serve the playground HTML page
pub async fn playground(tera: web::Data<Tera>) -> HttpResponse {
    let ctx = tera::Context::new();
    let rendered = tera.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

/// Execute PHP code and return results
/// Incoming playground execution payload. Accepts either a raw JSON string ("<?php echo 1; ?>")
/// or an object form `{ "code": "<?php echo 1; ?>" }` for future extensibility.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum ExecutePayload {
    /// Raw string payload
    Raw(String),
    /// Object form with code field
    Object { 
        /// PHP source code
        code: String 
    },
}

/// Execute a snippet of PHP code provided in the request body and return JSON
pub async fn execute_code(body: web::Json<ExecutePayload>) -> HttpResponse {
    let start = Instant::now();
    let mut errors: Vec<String> = Vec::new();

    // Extract code from flexible payload
    let code = match &*body {
        ExecutePayload::Raw(s) => s.clone(),
        ExecutePayload::Object { code } => code.clone(),
    };

    // 1. Lexing
    let tokens = match php_lexer::lex(&code) {
        Ok(tokens) => tokens,
        Err(e) => {
            errors.push(format!("Lexer: {}", e));
            let elapsed = start.elapsed().as_micros();
            return HttpResponse::Ok().json(json!({
                "output": "",
                "time_us": elapsed,
                "errors": errors
            }));
        }
    };

    // 2. Parsing
    let ast = match php_parser::parse(tokens) {
        Ok(ast) => ast,
        Err(e) => {
            errors.push(format!("Parser: {}", e));
            let elapsed = start.elapsed().as_micros();
            return HttpResponse::Ok().json(json!({
                "output": "",
                "time_us": elapsed,
                "errors": errors
            }));
        }
    };

    // 3. Execution
    let mut engine = Engine::new();
    if let Err(e) = engine.execute_stmt(&ast) {
        errors.push(format!("Runtime: {}", e));
    }

    let output = engine.get_output().to_string();
    let elapsed = start.elapsed().as_micros();
    HttpResponse::Ok().json(json!({
        "output": output,
        "time_us": elapsed,
        "errors": errors
    }))
}
// Legacy compatibility code removed: web playground now uses the modular parser + runtime engine.
