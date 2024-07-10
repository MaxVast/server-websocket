use actix::Actor;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{http::header, web, App, HttpServer};
use chrono::Utc;
use std::fs::File;
use std::io::BufReader;
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

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let mut certs_file = BufReader::new(File::open("cert.pem").unwrap());
    let mut key_file = BufReader::new(File::open("key.pem").unwrap());

    // load TLS certs and key
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .next()
        .unwrap()
        .unwrap();

    // set up TLS config options
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"]) // Ajout de POST si nécessaire
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
        .bind_rustls_0_23(("127.0.0.1", 8443), tls_config)?
        .run()
        .await
}
