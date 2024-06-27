use crate::api::handler::MessageType;
use actix::prelude::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

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
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BroadcastMessage {
    pub message: String,
    pub type_message: MessageType,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl Message for BroadcastMessage {
    type Result = ();
}

pub struct GetBroadcastMessage;

impl Message for GetBroadcastMessage {
    type Result = BroadcastMessage;
}
