//! RustyPHP Web Server
//! 
//! A web server for running PHP code through the RustyPHP interpreter

use actix_web::{HttpServer, App, web};
use log::info;
use tera::Tera;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting RustyPHP playground server...");

    // Initialize Tera templates from the new location
    let template_path = "crates/php-web/src/playground/templates/*.html";
    info!("Loading templates from: {}", template_path);
    let tera = match Tera::new(template_path) {
        Ok(t) => {
            info!("Templates loaded successfully");
            t
        }
        Err(e) => {
            eprintln!("Error initializing Tera templates: {}", e);
            eprintln!("Current working directory: {:?}", std::env::current_dir());
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Template error: {}", e)));
        }
    };

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .configure(php_web::playground::init_routes)
    })
    .bind("127.0.0.1:8080")?;
    
    info!("Server starting on http://127.0.0.1:8080");
    server.run()
    .await
}
