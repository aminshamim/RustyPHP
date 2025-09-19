//! Playground route configuration

use actix_web::web;
use super::handlers;

/// Initialize playground routes
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(handlers::playground));
    cfg.route("/execute", web::post().to(handlers::execute_code));
    cfg.route("/api/execute", web::post().to(handlers::execute_code)); // alias for API style usage
}
