use crate::{comms::websocket::start_websocket_server, server::GameServer};
use std::time::Instant;

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
pub mod server;
pub mod comms {
    pub mod playercomm;
    pub mod websocket;
}

fn main() {
    let handle = start_websocket_server();

    let mut game_server = GameServer::new();
    let mut last_frame = Instant::now();
    loop {
        let delta_time = last_frame.elapsed();
        last_frame = Instant::now();
        if !game_server.update(delta_time.as_secs_f64()) {
            break;
        }
    }
    handle.join().expect("Error joining websocket server");
}
