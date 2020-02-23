use crate::engine::player::PlayerId;
use futures::channel::mpsc::{Receiver, Sender};
use serde::{Deserialize, Serialize};

// TODO: change to enum that will auto-deserialize to the correct code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMessage {
    code: String,
    data: serde_json::Value,
}

impl PlayerMessage {
    pub fn new(code: &str, data: serde_json::Value) -> Self {
        PlayerMessage {
            code: code.to_owned(),
            data,
        }
    }
}

#[derive(Debug)]
pub struct PlayerComm {
    pub id: PlayerId,
    pub sender: Sender<PlayerMessage>,
    pub receiver: Receiver<PlayerMessage>,
}

impl PlayerComm {
    pub fn new(
        id: PlayerId,
        sender: Sender<PlayerMessage>,
        receiver: Receiver<PlayerMessage>,
    ) -> Self {
        PlayerComm {
            id,
            sender,
            receiver,
        }
    }
}

pub enum PlayerConnectEvent {
    Connected(PlayerComm),
    Disconnected(PlayerId),
}
