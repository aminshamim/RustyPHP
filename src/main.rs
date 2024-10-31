mod lexer;
mod parser;
mod ast;
mod interpreter;
// mod stdlib;
// mod error;
mod playground;

use actix_web::{HttpServer, App, web};
use log::info;
use tera::Tera;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting PHP playground in Rust...");

    // Initialize Tera and handle any errors if the template is not found
    let tera = Tera::new("src/playground/templates/**/*").expect("Error initializing Tera templates");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone())) // Register Tera with Actix Web
            .configure(playground::init_routes) // Initialize playground routes
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
