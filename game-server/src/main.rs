pub mod component {
    pub mod action;
    pub mod effect;
}
pub mod engine {
    pub mod bomb;
    pub mod config;
    pub mod explosion;
    pub mod mob;
    pub mod player;
    pub mod position;
    pub mod types;
    pub mod world;
    pub mod worlddata;
    pub mod worldzone;
}
pub mod tools {
    pub mod itemstore;
}
pub mod traits {
    pub mod celltypes;
    pub mod randenum;
    pub mod worldobject;
}
pub mod utils {
    pub mod misc;
}

pub mod comms {
    pub mod playercomm;
    pub mod websocket;
}
pub mod error;
pub mod game {
    pub mod maingame;
}

use crate::comms::websocket::spawn_websocket_server;
use tokio::sync::mpsc::channel;

use crate::game::maingame::RustonatorGame;

#[tokio::main]
async fn main() {
    init_logging();

    let (player_join_tx, player_join_rx) = channel(30);

    tokio::spawn(async {
        if let Err(e) = spawn_websocket_server(player_join_tx).await {
            eprintln!("Websocket error: {:?}", e);
        }
    });
    let mut game = RustonatorGame::new(47, 47);
    if let Err(e) = game.game_loop(player_join_rx).await {
        eprintln!("Error: {:?}", e);
    }
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive("game_server=debug".parse().unwrap())
                .from_env_lossy(),
        )
        .init();
}
