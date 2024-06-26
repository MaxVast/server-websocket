use actix::Actor;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use chrono::Utc;
use std::sync::{Arc, Mutex};

mod api;
mod server;
mod state;

use crate::server::message::BroadcastMessage;
use api::handler::config;
use server::web_socket::ws_index;
use state::app_state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    }

    println!("✅ Server started successfully");
    let broadcast_message = BroadcastMessage {
        message: "Hello world, welcome to Syneido !".to_string(),
        created_at: Utc::now(),
    };
    let state = Arc::new(Mutex::new(broadcast_message));
    let app_state = AppState::new(state).start();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(app_state.clone()))
            .route("/ws/", web::get().to(ws_index))
            .configure(config)
    })
    .workers(2)
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
