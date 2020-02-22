use crate::component::action::Action;
use futures::channel::mpsc::{Receiver, Sender};

// TODO: Create PlayerMessage struct with enum for different message types.
// Generic JSON should have one field for message type then a generic Value
// field for data. The data will be deserialized depending on the message type.

pub type PlayerMessage = serde_json::Value;

#[derive(Debug)]
pub struct PlayerComm {
    id: u64,
    sender: Sender<PlayerMessage>,
    receiver: Receiver<PlayerMessage>,
}

impl PlayerComm {
    pub fn new(id: u64, sender: Sender<PlayerMessage>, receiver: Receiver<PlayerMessage>) -> Self {
        PlayerComm {
            id,
            sender,
            receiver,
        }
    }
}
