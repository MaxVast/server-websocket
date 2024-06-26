use actix::Addr;
use actix_web::{web, Error, HttpResponse};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub type_message: MessageType,
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
        created_at: Utc::now(),
    };
    data.do_send(broadcast_message.clone());
    let response_json = GenericResponse {
        status: "created".to_string(),
        message: "Message sent and broadcasted".to_string(),
        type_message: MessageType::Text,
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
        .default_service(web::to(handler_404));
}
