use actix::Addr;
use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{web, Error, HttpResponse};
use chrono::prelude::*;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::env;
use std::{fs::File, io::Write};
use uuid::Uuid;

use crate::server::message::BroadcastMessage;
use crate::state::app_state::AppState;
// Deserialize incoming message payload
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
#[serde(rename_all = "lowercase")] // pour que les valeurs soient sérialisées en minuscules
pub enum MessageType {
    Text,
    Image,
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

async fn upload_image(
    data: web::Data<Addr<AppState>>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    // Get the upload folder
    let upload_dir = env::current_dir().unwrap().join("uploads/img");

    // Create the upload folder if doesnt exist
    if !upload_dir.exists() {
        std::fs::create_dir(&upload_dir).unwrap();
    }

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
                if filename.len() == 0 {
                    let _response_json = GenericResponse {
                        status: "Bad request".to_string(),
                        message: "No file found...".to_string(),
                        value: vec![{}].into(),
                    };
                    return Ok(HttpResponse::BadRequest().json(_response_json));
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
    let file_path_str = file_path.to_string();

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

// fallback route
async fn handler_404() -> HttpResponse {
    HttpResponse::NotFound().body("404 : Nothing here..")
}

//Config server
pub fn config(conf: &mut web::ServiceConfig) {
    conf.route("/send-message", web::post().to(send_message))
        .route("/upload-image", web::post().to(upload_image))
        .service(Files::new("/uploads/img", "uploads/img").show_files_listing());
        //.default_service(web::to(handler_404));
}
