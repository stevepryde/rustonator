use crate::server::GameServer;
use std::time::{Duration, Instant};

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
pub mod error;
use crate::comms::websocket::spawn_websocket_server;
use async_std::task;
use futures::channel::mpsc::channel;

use crate::error::ZResult;
use log::info;

fn main() {
    let (player_join_tx, mut player_join_rx) = channel(30);
    let handle_ws = task::spawn(spawn_websocket_server(player_join_tx));

    let mut game_server = GameServer::new();

    // Limit max FPS.
    let min_timeslice: f64 = 1.0 / 60.0;
    let handle_main = task::spawn(async move {
        let mut last_frame = Instant::now();
        let mut count = 0;
        let first_frame = Instant::now();
        loop {
            let delta_time = last_frame.elapsed().as_secs_f64();
            if delta_time < min_timeslice {
                let sleep_time = (min_timeslice - delta_time) * 1_000f64;
                task::sleep(Duration::from_millis(sleep_time as u64)).await;
            }
            last_frame = Instant::now();
            // Have any players joined?
            if let Ok(x) = player_join_rx.try_next() {
                info!("Player joined: {:?}", x);
            }
            if !game_server.update(delta_time) {
                break;
            }

            // count += 1;
            // println!("COUNT: {}", count);
            // println!(
            //     "FPS: {:.2}",
            //     count as f64 / first_frame.elapsed().as_secs_f64()
            // )
        }

        ZResult::Ok(())
    });
    task::block_on(async move {
        if let Err(e) = handle_main.await {
            eprintln!("Error: {:?}", e);
        }
        if let Err(e) = handle_ws.await {
            eprintln!("Websocket Error: {:?}", e);
        }
    });
}
