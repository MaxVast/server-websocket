use actix::prelude::*;
use std::sync::{Arc, Mutex};

use crate::server::message::{
    BroadcastMessage, GetBroadcastMessage, RegisterClient, UnregisterClient,
};
use crate::server::web_socket::MyWs;

pub struct AppState {
    pub clients: Vec<Addr<MyWs>>,
    pub broadcast_message: Arc<Mutex<BroadcastMessage>>,
}

impl AppState {
    pub fn new(broadcast_message: Arc<Mutex<BroadcastMessage>>) -> Self {
        AppState {
            clients: vec![],
            broadcast_message,
        }
    }
}

impl Actor for AppState {
    type Context = Context<Self>;
}

impl Handler<RegisterClient> for AppState {
    type Result = ();

    fn handle(&mut self, msg: RegisterClient, _: &mut Context<Self>) {
        self.clients.push(msg.addr);
    }
}

impl Handler<UnregisterClient> for AppState {
    type Result = ();

    fn handle(&mut self, msg: UnregisterClient, _: &mut Context<Self>) {
        self.clients.retain(|client| client != &msg.addr);
    }
}

impl Handler<BroadcastMessage> for AppState {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _: &mut Context<Self>) {
        for client in &self.clients {
            client.do_send(msg.clone());
        }

        let mut message = self.broadcast_message.lock().unwrap();
        *message = msg;
    }
}

impl Handler<GetBroadcastMessage> for AppState {
    type Result = MessageResult<GetBroadcastMessage>;

    fn handle(&mut self, _msg: GetBroadcastMessage, _ctx: &mut Self::Context) -> Self::Result {
        let message = self.broadcast_message.lock().unwrap().clone();
        MessageResult(message)
    }
}
