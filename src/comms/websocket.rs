use log::{debug, error};

use crate::{comms::playercomm::PlayerComm, component::action::Action};
use std::{
    convert::TryFrom,
    io,
    io::ErrorKind,
    net::{TcpListener, TcpStream},
    sync::mpsc::{channel, Receiver, Sender},
    thread::{spawn, JoinHandle},
};
use tungstenite::{accept_hdr, handshake::server::Request, Error, Message, WebSocket};

/// Start the websocket server on a new thread.
/// Each connection will spawn its own thread on top of that.
/// TODO: Implement connection limit.
pub fn start_websocket_server(server_sender: Sender<PlayerComm>) -> JoinHandle<()> {
    spawn(|| {
        let mut next_player_id: u64 = 0;
        let server = TcpListener::bind("0.0.0.0:3002").unwrap();
        debug!("Websocket server listening on port 3002");
        for r_stream in server.incoming() {
            // TODO: set up channels and initialize player.
            // This needs to assign a unique ID to the connection (sequential id would work
            // ok) This would create the input buffer and the server would
            // associate this id with the new player object.
            // Server keeps its own list of write channels for each id.
            //
            // Can the server write whenever or does it need to send via channel here
            // and then have this loop poll the channel and do the writes here?
            //
            // NOTE: the server does not handle read channels.
            // Write directly to cache of player inputs here.
            // Then the game server can just process a list of inputs each frame.
            // Needs a limit of 1 input per frame - anything beyond
            // this will be dropped or processed next frame. If the buffer
            // fills up, inputs will be dropped (this could result from the
            // server being too slow / overloaded, or clients attempting to cheat)

            let pid = next_player_id;
            next_player_id += 1;

            spawn(move || {
                let callback = |_req: &Request| {
                    // Let's add an additional header to our response to the client.
                    Ok(Some(vec![(
                        String::from("x-detonator-version"),
                        String::from("0.1"),
                    )]))
                };
                let stream = match r_stream {
                    Ok(x) => x,
                    Err(e) => {
                        debug!("Unable to connect websocket stream: {}", e);
                        return;
                    }
                };
                if let Err(e) = stream.set_nonblocking(true) {
                    error!("Error setting stream to non-blocking mode: {}", e);
                    return;
                }

                let websocket = match accept_hdr(stream, callback) {
                    Ok(x) => x,
                    Err(e) => {
                        debug!("Error accepting websocket connection: {}", e);
                        return;
                    }
                };

                // All ok, tell the server a new player has joined.
                let (action_sender_to_server, player_receiver) = channel();
                let (player_sender, framedata_receiver_from_server) = channel();
                let player_comm = PlayerComm::new(pid, player_sender, player_receiver);
                let server_sender_clone = server_sender.clone();

                server_sender_clone.send(player_comm);

                if let Err(e) = process_websocket(
                    websocket,
                    pid,
                    action_sender_to_server,
                    framedata_receiver_from_server,
                ) {
                    debug!("Websocket disconnected: {}", e);
                }
            });
        }
    })
}

/// Process the entire lifecycle of a single player's websocket connection.
fn process_websocket(
    mut websocket: WebSocket<TcpStream>,
    pid: u64,
    ws_sender: Sender<Action>,
    ws_receiver: Receiver<serde_json::Value>,
) -> tungstenite::Result<()>
{
    loop {
        // TODO: This will burn up the CPU. Need to think more about using blocking IO
        // instead. Or maybe I need async after all?
        // Wish I could split the sender and receiver...

        // Options:
        // 1. Separate websocket server - still leaves the problem of communicating over
        //    unix/tcp socket using sync/threads
        // 2. Convert to async - this seems more plausible but how do I solve the
        // problem    of bad actors? If someone floods the network with
        // websocket messages, the system    will try to handle them all. How do
        // I rate-limit these?
        // a. Disconnect clients if too many messages received
        // b. Too many connection attempts? I can rate-limit join requests.
        //    The websocket server should run on a separate thread anyway.
        // 3. Best of both worlds. Both async and separate processes.
        //    Separating the IO that talks to the outside world from the internal
        // machinery    has its own benefits in terms of scaling. For now I'll
        // attempt to write it    such that it's separated across a thread
        // boundary anyway and can easily    be moved to a seprate process
        // later.

        // Did we receive any action from the player to send to the server?
        match websocket.read_message() {
            Ok(msg) => {
                if msg.is_text() {
                    // Put message on the input channel.
                    // Serialize into Action.
                    if let Ok(value) = serde_json::to_value(msg.to_string()) {
                        if let Ok(action) = Action::try_from(value) {
                            ws_sender.send(action);
                        }
                    }
                }
            }
            Err(Error::Io(e)) => match e.kind() {
                ErrorKind::WouldBlock => {}
                _ => return Err(Error::Io(e)),
            },
            Err(e) => {
                return Err(e);
            }
        }

        // Did we get anything from the server to send to the player?
        if let Ok(x) = ws_receiver.try_recv() {
            websocket.write_message(Message::from(x.to_string()))?;
        }
    }

    Ok(())
}
