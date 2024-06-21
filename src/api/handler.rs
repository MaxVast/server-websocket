use actix::Addr;
use actix_web::{web, Error, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::message::message::BroadcastMessage;
use crate::state::app_state::AppState;
// Deserialize incoming message payload
#[derive(Deserialize)]
struct MessagePayload {
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
    pub value: String,
}

// Endpoint to send message
async fn send_message(
    data: web::Data<Addr<AppState>>,
    msg: web::Json<MessagePayload>,
) -> Result<HttpResponse, Error> {
    data.do_send(BroadcastMessage {
        message: msg.message.clone(),
    });
    let response_json = GenericResponse {
        status: "success".to_string(),
        message: "Message sent and broadcasted".to_string(),
        value: msg.message.to_string(),
    };
    Ok(HttpResponse::Ok().json(response_json))
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
