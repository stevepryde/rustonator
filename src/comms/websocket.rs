use log::{error, info};

use crate::comms::playercomm::PlayerComm;
use crate::comms::playercomm::{PlayerConnectEvent, PlayerMessage};
use crate::engine::player::PlayerId;
use crate::error::ZResult;
use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use async_tungstenite::{accept_async, WebSocketStream};
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;

use tungstenite::Message;

/// Start async websocket server.
/// NOTE: The caller can run this on a separate executor if needed.
pub async fn spawn_websocket_server(server_sender: Sender<PlayerConnectEvent>) -> ZResult<()> {
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

        task::spawn(accept_connection(
            peer,
            stream,
            player_id,
            server_sender.clone(),
        ));
    }

    Ok(())
}

async fn accept_connection(
    peer: SocketAddr,
    stream: TcpStream,
    player_id: PlayerId,
    mut server_sender: Sender<PlayerConnectEvent>,
) -> ZResult<()> {
    if let Err(e) = handle_connection(peer, stream, player_id, server_sender.clone()).await {
        error!("Error processing connection: {:?}", e);
    }

    // Disconnect player.
    server_sender
        .send(PlayerConnectEvent::Disconnected(player_id))
        .await?;
    Ok(())
}

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    player_id: PlayerId,
    server_sender: Sender<PlayerConnectEvent>,
) -> ZResult<()> {
    let ws_stream = accept_async(stream).await?;

    info!("New websocket connection: {}", peer);

    let (ws_tx, ws_rx) = ws_stream.split();

    // All ok, tell the server a new player has joined.
    let (comms_out_engine_tx, comms_out_ws_rx) = channel(30);
    let (comms_in_ws_tx, comms_in_engine_rx) = channel(30);
    let player_comm = PlayerComm::new(player_id, comms_out_engine_tx, comms_in_engine_rx);
    let mut server_sender_clone = server_sender.clone();
    server_sender_clone
        .send(PlayerConnectEvent::Connected(player_comm))
        .await?;

    let writer = process_websocket_write(comms_out_ws_rx, ws_tx);
    let reader = process_websocket_read(ws_rx, comms_in_ws_tx);
    futures::try_join!(writer, reader)?;
    Ok(())
}

async fn process_websocket_read(
    mut ws_rx: SplitStream<WebSocketStream<TcpStream>>,
    mut player_sender: Sender<PlayerMessage>,
) -> ZResult<()> {
    while let Some(msg) = ws_rx.next().await {
        let msg = msg?;
        if msg.is_binary() {
            // TODO: support MessagePack?
            error!("Unexpected binary message received. Connection dropped.");
            break;
        }
        if msg.is_text() {
            // Put message on the input channel.
            // NOTE: this will terminate the connection if any message fails
            //       to deserialize. This is probably the desired behaviour
            //       to eliminate faulty clients.
            let player_msg: PlayerMessage = serde_json::from_str(&msg.to_string())?;
            player_sender.send(player_msg).await?
        }
    }

    Ok(())
}

async fn process_websocket_write(
    mut engine_rx: Receiver<PlayerMessage>,
    mut ws_tx: SplitSink<WebSocketStream<TcpStream>, Message>,
) -> ZResult<()> {
    while let Some(msg) = engine_rx.next().await {
        ws_tx
            .send(Message::from(serde_json::to_value(&msg)?.to_string()))
            .await?;
    }

    Ok(())
}
