use crate::api::handler::GenericResponse;
use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::server::message::{
    BroadcastMessage, GetBroadcastMessage, RegisterClient, UnregisterClient,
};
use crate::state::app_state::AppState;

pub struct MyWs {
    pub app_state: Addr<AppState>,
}

impl MyWs {
    pub fn new(app_state: Addr<AppState>) -> Self {
        MyWs { app_state }
    }
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.app_state
            .send(RegisterClient { addr: addr.clone() })
            .into_actor(self)
            .then(|_, act, ctx| {
                act.app_state
                    .send(GetBroadcastMessage)
                    .into_actor(act)
                    .then(|res, _, ctx| {
                        if let Ok(state) = res {
                            let response_json = GenericResponse {
                                status: "success".to_string(),
                                message: "Message broadcasted".to_string(),
                                value: vec![state].into(),
                            };
                            let json_msg = serde_json::to_string(&response_json).unwrap();

                            ctx.text(json_msg);
                        }
                        fut::ready(())
                    })
                    .wait(ctx);

                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        self.app_state.do_send(UnregisterClient {
            addr: ctx.address(),
        });
        Running::Stop
    }
}

impl Handler<BroadcastMessage> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        // Create the broadcast message
        let broadcast_msg = BroadcastMessage {
            message: msg.message.clone(),
            type_message: msg.type_message.clone(),
            created_at: msg.created_at,
        };

        // Create the response struct
        let response_json = GenericResponse {
            status: "success".to_string(),
            message: "Message broadcasted".to_string(),
            value: vec![broadcast_msg].into(),
        };

        // Serialize the response to JSON
        if let Ok(json_msg) = serde_json::to_string(&response_json) {
            // Send the JSON response back to the client
            ctx.text(json_msg);
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Close(_)) = msg {
            self.app_state.do_send(UnregisterClient {
             addr: ctx.address(),
            });
            ctx.stop();
        }

    }
}

pub async fn ws_index(
    r: HttpRequest,
    stream: web::Payload,
    data: web::Data<Addr<AppState>>,
) -> Result<HttpResponse, Error> {
    let ws = MyWs::new(data.get_ref().clone());
    ws::start(ws, &r, stream)
}
