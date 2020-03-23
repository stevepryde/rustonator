use crate::{
    comms::websocket::{WsError, WsResult},
    component::action::Action,
    engine::{
        player::{Player, PlayerId, SerPlayer},
        world::World,
        worlddata::SerWorldData,
    },
    error::{ZError, ZResult},
};
use futures::StreamExt;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ops::Deref;
use tokio::sync::mpsc::{error::TryRecvError, Receiver, Sender};

pub type PlayerSender = Sender<PlayerMessageExternal>;
pub type PlayerReceiver = Receiver<PlayerMessageExternal>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageId(u64);

impl Deref for MessageId {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerMessage {
    JoinGame(String),
    Action(Action),
    SpawnPlayer(SerPlayer, SerWorldData),
    PowerUp(String),
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

    pub async fn send(&mut self, message: PlayerMessage) -> ZResult<()> {
        self.sender
            .send(PlayerMessageExternal::new(*self.uid, message))
            .await
            .map_err(|e| ZError::from(WsError::from(e)))?;
        Ok(())
    }

    pub async fn send_powerup(&mut self, powerup: &str) -> ZResult<()> {
        self.send(PlayerMessage::PowerUp(powerup.to_string())).await
    }
}

pub enum PlayerConnectEvent {
    Connected(PlayerComm),
    Disconnected(PlayerId),
}
