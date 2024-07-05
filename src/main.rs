use actix_cors::Cors;
use actix::Actor;
use actix_web::middleware::Logger;
use actix_web::{http::header, web, App, HttpServer};
use chrono::Utc;

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    sync::{Arc, Mutex},
};

mod api;
mod middleware;
mod server;
mod state;

use crate::api::handler::MessageType;
use crate::middleware::api_key::ApiKey;
use crate::server::message::BroadcastMessage;
use api::handler::config;
use server::web_socket::ws_index;
use state::app_state::AppState;

fn create_directory_if_not_exists(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o775))?;
        println!(
            "✅ Folder path : './{}' created with permission 0775 successfully",
            path.display()
        );
    }
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    }

    let img_dir = Path::new("uploads/img");
    let video_dir = Path::new("uploads/video");

    create_directory_if_not_exists(img_dir)?;
    create_directory_if_not_exists(video_dir)?;

    println!("✅ Server started successfully");
    let broadcast_message = BroadcastMessage {
        message: "Hello world, welcome to Syneido !".to_string(),
        type_message: MessageType::Text,
        created_at: Utc::now(),
    };
    let shared_state = Arc::new(Mutex::new(broadcast_message));
    let app_state = AppState::new(shared_state.clone()).start();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:3000/")
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials();
        App::new()
            .wrap(ApiKey)
            .wrap(cors)
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
