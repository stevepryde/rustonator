use crate::comms::websocket::{WsError, WsResult};
use crate::engine::player::PlayerId;
use crate::error::{ZError, ZResult};
use futures::StreamExt;
use log::*;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, Sender};

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

    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub async fn recv_one(&mut self) -> ZResult<Option<PlayerMessage>> {
        match self.receiver.try_recv() {
            Ok(v) => Ok(Some(v.data)),
            Err(TryRecvError::Empty) => Ok(None),
            _ => Err(ZError::WebSocketError(WsError::Disconnected)),
        }
    }
}

pub enum PlayerConnectEvent {
    Connected(PlayerComm),
    Disconnected(PlayerId),
}
