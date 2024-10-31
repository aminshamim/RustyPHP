use actix_web::{HttpResponse, web};
use serde_json::json;
use std::time::Instant;
use tera::Tera;
use crate::{lexer, parser, interpreter};

pub async fn playground(tera: web::Data<Tera>) -> HttpResponse {
    let ctx = tera::Context::new();
    let rendered = tera.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub async fn execute_code(code: web::Json<String>) -> HttpResponse {
    // Start the timer
    let start = Instant::now();

    // Process the PHP code
    let tokens = lexer::lex(&code);
    let result = match parser::parse(tokens) {
        Ok(ast) => interpreter::execute(ast),
        Err(e) => Err(e),
    };

    // Calculate elapsed time in microseconds
    let elapsed = start.elapsed().as_micros();

    // Format the response with output and server response time in µs
    let response = match result {
        Ok(output) => json!({ "output": output, "response_time": elapsed }),
        Err(error) => json!({ "output": format!("Error: {}", error), "response_time": elapsed }),
    };

    HttpResponse::Ok().json(response)
}