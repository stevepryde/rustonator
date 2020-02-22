use log::{debug, error, info};

use crate::error::ZResult;
use crate::{comms::playercomm::PlayerComm, component::action::Action};
use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use async_tungstenite::{accept_async, accept_hdr_async, WebSocketStream};
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt};
use std::{convert::TryFrom, io, io::ErrorKind, net::SocketAddr};
use tungstenite::handshake::server::{Request, Response};

/// Start async websocket server.
/// NOTE: The caller can run this on a separate executor if needed.
pub async fn spawn_websocket_server(server_sender: Sender<PlayerComm>) -> ZResult<()> {
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

        let player_id = next_player_id;
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
    player_id: u64,
    server_sender: Sender<PlayerComm>,
) {
    if let Err(e) = handle_connection(peer, stream, player_id, server_sender).await {
        error!("Error processing connection: {:?}", e);
    }
}

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    player_id: u64,
    server_sender: Sender<PlayerComm>,
) -> ZResult<()> {
    let mut ws_stream = accept_async(stream).await?;

    info!("New websocket connection: {}", peer);

    let (ws_tx, ws_rx) = ws_stream.split();

    // All ok, tell the server a new player has joined.
    let (comms_out_engine_tx, comms_out_ws_rx) = channel(30);
    let (comms_in_ws_tx, comms_in_engine_rx) = channel(30);
    let player_comm = PlayerComm::new(player_id, comms_out_engine_tx, comms_in_engine_rx);
    let mut server_sender_clone = server_sender.clone();
    server_sender_clone.send(player_comm).await?;

    // TODO: spawn separate tasks for read/write, then return.
    process_websocket_read(ws_rx, comms_in_ws_tx).await?;
    Ok(())
}

async fn process_websocket_read(
    mut ws_rx: SplitStream<WebSocketStream<TcpStream>>,
    mut player_sender: Sender<serde_json::Value>,
) -> ZResult<()> {
    while let Some(msg) = ws_rx.next().await {
        let msg = msg?;
        if msg.is_binary() {
            error!("Unexpected binary message received. Connection dropped.");
            break;
        }
        if msg.is_text() {
            // Put message on the input channel.
            match serde_json::to_value(msg.to_string()) {
                Ok(value) => player_sender.send(value).await?,
                Err(e) => {
                    error!("Unexpected message received: {}", e);
                    break;
                }
            }
        }
    }

    Ok(())
}
