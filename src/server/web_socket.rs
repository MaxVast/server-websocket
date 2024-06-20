use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::message::message::{
    BroadcastMessage, GetState, RegisterClient, UnregisterClient, WsMessage,
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
                ctx.text("Welcome! You are connected.");
                act.app_state
                    .send(GetState)
                    .into_actor(act)
                    .then(|res, _, ctx| {
                        if let Ok(state) = res {
                            ctx.text(state);
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

impl Handler<WsMessage> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let new_value = text.to_string();
                self.app_state
                    .do_send(BroadcastMessage { message: new_value });
            }
            Ok(ws::Message::Close(_)) => {
                self.app_state.do_send(UnregisterClient {
                    addr: ctx.address(),
                });
                ctx.stop();
            }
            _ => (),
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
