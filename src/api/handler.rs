use actix::Addr;
use actix_files::{Files, NamedFile};
use actix_multipart::Multipart;
use actix_web::http::StatusCode;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use chrono::prelude::*;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fs::File, io::Write, path::Path};
use uuid::Uuid;

use crate::server::message::BroadcastMessage;
use crate::state::app_state::AppState;

#[derive(Deserialize)]
struct MessagePayload {
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct GenericResponse<T> {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Vec<T>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Text,
    Image,
    Video,
}

// Endpoint to send message
async fn send_message(
    data: web::Data<Addr<AppState>>,
    msg: web::Json<MessagePayload>,
) -> Result<HttpResponse, Error> {
    let broadcast_message = BroadcastMessage {
        message: msg.message.to_string(),
        type_message: MessageType::Text,
        created_at: Utc::now(),
    };
    data.do_send(broadcast_message.clone());
    let response_json = GenericResponse {
        status: "created".to_string(),
        message: "Message sent and broadcasted".to_string(),
        value: vec![broadcast_message].into(),
    };

    Ok(HttpResponse::Created().json(response_json))
}

fn check_file_is_uploaded(filename: &str) -> Result<(), Error> {
    //Check if file is uploaded
    if filename.is_empty() {
        let response_json = GenericResponse {
            status: "Bad request".to_string(),
            message: "No file found...".to_string(),
            value: vec![{}].into(),
        };
        return Err(actix_web::error::InternalError::from_response(
            StatusCode::BAD_REQUEST,
            HttpResponse::BadRequest().json(response_json),
        )
        .into());
    }
    Ok(())
}

async fn upload_image(
    data: web::Data<Addr<AppState>>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    // Generate an Uuid before each loop
    let id = Uuid::new_v4();

    // Declare the variable before entering the loop
    let mut file_path = String::new();

    while let Some(item) = payload.next().await {
        match item {
            Ok(mut field) => {
                let content_disposition = field.content_disposition();
                let filename = content_disposition.get_filename().unwrap_or("unknown");

                //Check if file is uploaded
                check_file_is_uploaded(filename)?;

                // Check if the file has .jpg, .png, or .webp extension
                let file_extension = Path::new(filename)
                    .extension()
                    .and_then(std::ffi::OsStr::to_str);
                if file_extension != Some("jpg")
                    && file_extension != Some("png")
                    && file_extension != Some("webp")
                {
                    let response_json = GenericResponse {
                        status: "Unsupported media type".to_string(),
                        message: "Only .jpg, .png, or .webp files are allowed.".to_string(),
                        value: vec![{}].into(),
                    };
                    return Ok(HttpResponse::UnsupportedMediaType().json(response_json));
                }

                // Add id with the file name original
                let filename_with_id = format!("{}_{}", id, filename);

                file_path = format!("uploads/img/{}", filename_with_id);
                //Upload the file into folder uploads/
                let mut file = File::create(file_path.clone()).unwrap();

                // Copy the content of the field to the file
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    file.write_all(&data).unwrap();
                }
            }
            //If a error return error 500
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().body(format!("Error: {:?}", e)));
            }
        }
    }
    // Send the file path to WebSocket clients
    let file_path_str = format!("/{}", file_path);

    let broadcast_message = BroadcastMessage {
        message: file_path_str.to_string(),
        type_message: MessageType::Image,
        created_at: Utc::now(),
    };
    data.do_send(broadcast_message.clone());

    let response_json = GenericResponse {
        status: "created".to_string(),
        message: "Image uploaded and broadcasted".to_string(),
        value: vec![broadcast_message].into(),
    };

    Ok(HttpResponse::Created().json(response_json))
}

async fn upload_video(
    data: web::Data<Addr<AppState>>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    // Generate an Uuid before each loop
    let id = Uuid::new_v4();

    // Declare the variable before entering the loop
    let mut file_path = String::new();

    while let Some(item) = payload.next().await {
        match item {
            Ok(mut field) => {
                let content_disposition = field.content_disposition();
                let filename = content_disposition.get_filename().unwrap_or("unknown");

                //Check if file is uploaded
                check_file_is_uploaded(filename)?;

                // Check if the file has .mp4 extension
                let file_extension = Path::new(filename)
                    .extension()
                    .and_then(std::ffi::OsStr::to_str);
                if file_extension != Some("mp4") {
                    let response_json = GenericResponse {
                        status: "Unsupported media type".to_string(),
                        message: "Only .mp4 files are allowed.".to_string(),
                        value: vec![{}].into(),
                    };
                    return Ok(HttpResponse::UnsupportedMediaType().json(response_json));
                }

                // Add id with the file name original
                let filename_with_id = format!("{}_{}", id, filename);

                file_path = format!("uploads/video/{}", filename_with_id);
                //Upload the file into folder uploads/
                let mut file = File::create(file_path.clone()).unwrap();

                // Copy the content of the field to the file
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    file.write_all(&data).unwrap();
                }
            }
            //If a error return error 500
            Err(e) => {
                return Ok(HttpResponse::InternalServerError().body(format!("Error: {:?}", e)));
            }
        }
    }
    // Send the file path to WebSocket clients
    let file_path_str = format!("/{}", file_path);

    let broadcast_message = BroadcastMessage {
        message: file_path_str.to_string(),
        type_message: MessageType::Video,
        created_at: Utc::now(),
    };
    data.do_send(broadcast_message.clone());

    let response_json = GenericResponse {
        status: "created".to_string(),
        message: "Video uploaded and broadcasted".to_string(),
        value: vec![broadcast_message].into(),
    };

    Ok(HttpResponse::Created().json(response_json))
}

async fn serve_video(req: HttpRequest) -> Result<HttpResponse, Error> {
    let filename: String = req.match_info().query("filename").to_string();
    let path: PathBuf = format!("./uploads/video/{}", filename).parse().unwrap();
    let file = NamedFile::open(path)?;
    Ok(file.into_response(&req).map_into_boxed_body())
}

// fallback route
async fn handler_404() -> HttpResponse {
    HttpResponse::NotFound().body("404 : Nothing here..")
}

//Config server
pub fn config(conf: &mut web::ServiceConfig) {
    conf.route("/send-message", web::post().to(send_message))
        .route("/upload-image", web::post().to(upload_image))
        .route("/upload-video", web::post().to(upload_video))
        .route("/uploads/video/{filename}", web::get().to(serve_video))
        .service(Files::new("/uploads/img", "uploads/img").show_files_listing())
        .default_service(web::to(handler_404));
}
