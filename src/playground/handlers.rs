use actix_web::{HttpResponse, web};
use serde_json::json;
use std::time::Instant;
use tera::Tera;
use std::collections::HashMap;
use crate::{lexer, parser, interpreter};

pub async fn playground(tera: web::Data<Tera>) -> HttpResponse {
    let ctx = tera::Context::new();
    let rendered = tera.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub async fn execute_code(code: web::Json<String>) -> HttpResponse {
    let start = Instant::now();

    let tokens = lexer::lex(&code).unwrap_or_default();
    // for token in &tokens {
    //     // Convert the token to a JSON string
    //     let json_token = serde_json::to_string(token)
    //         .unwrap_or_else(|_| "Error serializing token".to_string());
    //
    //     // Print the JSON string representation of each token
    //     println!("{}", json_token);
    // }
    let mut context = HashMap::new();
    let mut string_context = HashMap::new();
    let result = match parser::parse(tokens) {
        Ok(ast) => interpreter::execute(ast, &mut context, &mut string_context),
        Err(e) => Err(e),
    };

    let elapsed = start.elapsed().as_micros();
    let response = match result {
        Ok(output) => json!({ "output": output, "response_time": elapsed }),
        Err(error) => json!({ "output": format!("Error: {}", error), "response_time": elapsed }),
    };

    HttpResponse::Ok().json(response)
}
