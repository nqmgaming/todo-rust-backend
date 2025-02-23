pub mod pizza;
pub mod user;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    pizza::pizza_routes(cfg);
    user::user_routes(cfg);
}
