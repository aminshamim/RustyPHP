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

    // Determine port (env override) default 1010
    let port: u16 = std::env::var("RUSTYPHP_PORT").ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10101);

    let bind_addr = format!("127.0.0.1:{}", port);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .configure(php_web::playground::init_routes)
    })
    .bind(&bind_addr).map_err(|e| {
        eprintln!("Failed to bind {}: {}", bind_addr, e);
        if bind_addr.ends_with(":1010") { eprintln!("Port 1010 may require elevated privileges on some systems. Try an unprivileged port like 10101."); }
        e
    })?;

    info!("Server starting on http://{}", bind_addr);
    server.run()
    .await
}
