use log::{error, info};

use crate::{
    comms::playercomm::{PlayerComm, PlayerConnectEvent, PlayerMessageExternal},
    engine::player::PlayerId,
};
use std::net::SocketAddr;

use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, WebSocketStream};
use tungstenite::Message;

pub type WsResult<T> = Result<T, WsError>;

#[derive(Debug, Clone)]
pub enum WsError {
    ConnectError(String),
    RecvError(String),
    SendError(String),
    JsonError(String),
    Disconnected,
}

impl From<tungstenite::error::Error> for WsError {
    fn from(e: tungstenite::error::Error) -> Self {
        WsError::ConnectError(e.to_string())
    }
}

impl From<serde_json::error::Error> for WsError {
    fn from(e: serde_json::error::Error) -> Self {
        WsError::JsonError(e.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for WsError {
    fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self {
        WsError::SendError(e.to_string())
    }
}

impl From<tokio::sync::mpsc::error::RecvError> for WsError {
    fn from(e: tokio::sync::mpsc::error::RecvError) -> Self {
        WsError::RecvError(e.to_string())
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for WsError {
    fn from(e: std::sync::mpsc::SendError<T>) -> Self {
        WsError::SendError(e.to_string())
    }
}

impl From<tokio::io::Error> for WsError {
    fn from(e: tokio::io::Error) -> Self {
        WsError::ConnectError(e.to_string())
    }
}

/// Start async websocket server.
/// NOTE: The caller can run this on a separate executor if needed.
pub async fn spawn_websocket_server(
    server_sender: std::sync::mpsc::Sender<PlayerConnectEvent>,
) -> WsResult<()> {
    let addr = "0.0.0.0:9002";
    let listener = TcpListener::bind(&addr).await?;
    info!("Websocket server listening on: {}", addr);
    let mut next_player_id: u64 = 1;

    while let Ok((stream, _)) = listener.accept().await {
        let peer = match stream.peer_addr() {
            Ok(x) => x,
            Err(e) => {
                error!("Unable to get peer address: {}", e);
                continue;
            }
        };
        info!("Socket connected: {}", peer);

        let player_id = PlayerId::from(next_player_id);
        next_player_id += 1;
        let sender = server_sender.clone();
        let fut = accept_connection(peer, stream, player_id, sender);

        tokio::spawn(fut);
    }

    Ok(())
}

async fn accept_connection(
    peer: SocketAddr,
    stream: TcpStream,
    player_id: PlayerId,
    server_sender: std::sync::mpsc::Sender<PlayerConnectEvent>,
) -> WsResult<()> {
    let sender = server_sender.clone();
    if let Err(e) = handle_connection(peer, stream, player_id, sender).await {
        error!("Error processing connection: {:?}", e);
    }

    // Disconnect player.
    server_sender
        .clone()
        .send(PlayerConnectEvent::Disconnected(player_id))?;
    Ok(())
}

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    player_id: PlayerId,
    server_sender: std::sync::mpsc::Sender<PlayerConnectEvent>,
) -> WsResult<()> {
    let ws_stream = accept_async(stream).await?;

    info!("New websocket connection: {}", peer);

    let (ws_tx, ws_rx) = ws_stream.split();

    // All ok, tell the server a new player has joined.

    // PlayerComm -> ws (here)
    let (pcomm_tx, wscomm_rx) = tokio::sync::mpsc::channel(30);

    // ws (here) -> PlayerComm. This uses a sync channel to provide access to the try_recv() method.
    let (wscomm_tx, pcomm_rx) = std::sync::mpsc::channel();

    let player_comm = PlayerComm::new(player_id, pcomm_tx, pcomm_rx);
    let server_sender_clone = server_sender.clone();
    server_sender_clone.send(PlayerConnectEvent::Connected(player_comm))?;

    // PlayerComm -> ws -> external
    let writer = process_websocket_write(wscomm_rx, ws_tx);

    // External -> ws -> PlayerComm
    let reader = process_websocket_read(ws_rx, wscomm_tx);
    futures::try_join!(writer, reader)?;
    Ok(())
}

/// Process websocket read events. We need to run this in a polling loop
/// in order to automatically process heartbeats. Messages are pushed into
/// a channel connected to the PlayerComm object.
async fn process_websocket_read(
    mut ws_rx: SplitStream<WebSocketStream<TcpStream>>,
    player_tx: std::sync::mpsc::Sender<PlayerMessageExternal>,
) -> WsResult<()> {
    while let Some(msg) = ws_rx.next().await {
        let msg = msg?;

        if msg.is_close() {
            break;
        }

        if msg.is_text() {
            // Put message on the input channel.
            // NOTE: this will terminate the connection if any message fails
            //       to deserialize. This is probably the desired behaviour
            //       to eliminate faulty clients.
            let player_msg: PlayerMessageExternal = serde_json::from_str(&msg.to_string())?;
            player_tx.send(player_msg)?
        } else if msg.is_binary() {
            // TODO: support bincode?
            // I'd like to someday explore binary message formats with the
            // client. The client will hopefully someday also be Rust.
            error!("Unexpected binary message received. Connection dropped.");
            break;
        }
    }

    Ok(())
}

async fn process_websocket_write(
    mut player_rx: tokio::sync::mpsc::Receiver<PlayerMessageExternal>,
    mut ws_tx: SplitSink<WebSocketStream<TcpStream>, Message>,
) -> WsResult<()> {
    while let Some(msg) = player_rx.recv().await {
        if msg.is_disconnect() {
            // TODO: trigger reader to drop as well.
            break;
        }

        ws_tx
            .send(Message::from(serde_json::to_value(&msg)?.to_string()))
            .await?;
    }

    Ok(())
}
