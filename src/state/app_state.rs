use actix::prelude::*;
use std::sync::{Arc, Mutex};

use crate::message::message::{
    BroadcastMessage, GetState, RegisterClient, UnregisterClient, WsMessage,
};
use crate::server::web_socket::MyWs;

pub struct AppState {
    pub clients: Vec<Addr<MyWs>>,
    pub state: Arc<Mutex<String>>,
}

impl AppState {
    pub fn new(state: Arc<Mutex<String>>) -> Self {
        AppState {
            clients: vec![],
            state,
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
            client.do_send(WsMessage(msg.message.clone()));
        }

        let mut state = self.state.lock().unwrap();
        *state = msg.message;
    }
}

impl Handler<GetState> for AppState {
    type Result = String;

    fn handle(&mut self, _: GetState, _: &mut Context<Self>) -> Self::Result {
        let state = self.state.lock().unwrap();
        state.clone()
    }
}
