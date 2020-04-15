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
use fern;

#[tokio::main]
async fn main() {
    init_logging();

    let (player_join_tx, player_join_rx) = channel(30);

    tokio::spawn(async {
        if let Err(e) = spawn_websocket_server(player_join_tx).await {
            eprintln!("Websocket error: {:?}", e);
        }
    });
    let mut game = RustonatorGame::new(51, 51);
    if let Err(e) = game.game_loop(player_join_rx).await {
        eprintln!("Error: {:?}", e);
    }
}

fn init_logging() {
    // let mut log_file = path.clone();
    // log_file.push("test.log");

    fern::Dispatch::new()
        .level(log::LevelFilter::Off)
        .level_for("rustonator", log::LevelFilter::Debug)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(std::io::stdout())
        .apply()
        .expect("Error setting up logging");
}
