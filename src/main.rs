use actix::Actor;
use actix_web::{web, App, HttpServer};
use std::sync::{Arc, Mutex};

mod message;
mod server;
mod state;

use server::web_socket::ws_index;
use state::app_state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    }

    println!("✅ Server started successfully");
    let state = Arc::new(Mutex::new(String::from(
        "Hello world, welcome to Syneido !",
    )));
    let app_state = AppState::new(state).start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/ws/", web::get().to(ws_index))
    })
    .workers(2)
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
