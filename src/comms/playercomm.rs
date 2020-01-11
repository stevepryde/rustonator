use crate::component::action::Action;
use std::sync::mpsc::{Receiver, Sender};

pub struct PlayerComm {
    id: u64,
    sender: Sender<serde_json::Value>,
    receiver: Receiver<Action>,
}

impl PlayerComm {
    pub fn new(id: u64, sender: Sender<serde_json::Value>, receiver: Receiver<Action>) -> Self {
        PlayerComm {
            id,
            sender,
            receiver,
        }
    }
}
