use crate::comms::websocket::{WsError, WsResult};
use crate::engine::player::PlayerId;
use crate::error::ZResult;
use futures::channel::mpsc::{Receiver, Sender};
use futures::StreamExt;
use log::*;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub type PlayerSender = Sender<PlayerMessageExternal>;
pub type PlayerReceiver = Receiver<PlayerMessageExternal>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageId(u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerMessage {
    InitPlayer,
    Disconnect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMessageExternal {
    uid: u64,
    data: PlayerMessage,
}

impl PlayerMessageExternal {
    pub fn new(uid: u64, data: PlayerMessage) -> Self {
        PlayerMessageExternal { uid, data }
    }
}

#[derive(Debug)]
pub struct PlayerComm {
    id: PlayerId,
    uid: MessageId,
    sender: PlayerSender,
    receiver: PlayerReceiver,
}

impl PlayerComm {
    pub fn new(id: PlayerId, sender: PlayerSender, receiver: PlayerReceiver) -> Self {
        PlayerComm {
            id,
            uid: MessageId(0),
            sender,
            receiver,
        }
    }

    pub async fn recv_one(&self) -> ZResult<Option<PlayerMessage>> {
        let v: Option<PlayerMessageExternal> = self.receiver.await;
        if let Some(msg) = v {
            Ok(Some(msg.data))
        } else {
            Ok(None)
        }
    }
}

pub enum PlayerConnectEvent {
    Connected(PlayerComm),
    Disconnected(PlayerId),
}
