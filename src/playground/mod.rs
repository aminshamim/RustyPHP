
use actix_web::web;
mod handlers;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(handlers::playground));
    cfg.route("/execute", web::post().to(handlers::execute_code));
}
