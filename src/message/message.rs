use actix::prelude::*;

use crate::server::web_socket::MyWs;

pub struct RegisterClient {
    pub addr: Addr<MyWs>,
}

impl Message for RegisterClient {
    type Result = ();
}

pub struct UnregisterClient {
    pub addr: Addr<MyWs>,
}

impl Message for UnregisterClient {
    type Result = ();
}

pub struct BroadcastMessage {
    pub message: String,
}

impl Message for BroadcastMessage {
    type Result = ();
}

pub struct GetState;

impl Message for GetState {
    type Result = String;
}

pub struct WsMessage(pub String);

impl Message for WsMessage {
    type Result = ();
}
