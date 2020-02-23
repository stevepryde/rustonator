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

use crate::comms::playercomm::{PlayerConnectEvent, PlayerMessage};
use crate::error::ZResult;
use fern;
use futures::SinkExt;
use log::info;

fn main() {
    init_logging();

    let (player_join_tx, mut player_join_rx) = channel(30);
    let handle_ws = task::spawn(spawn_websocket_server(player_join_tx));

    let mut game_server = GameServer::new();

    let mut players = Vec::new();
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
            // TODO: create playermanager to manage player events.
            if let Ok(x) = player_join_rx.try_next() {
                match x {
                    Some(PlayerConnectEvent::Connected(p)) => {
                        info!("Player connected: {:?}", p);
                        players.push(p);
                    }
                    Some(PlayerConnectEvent::Disconnected(pid)) => {
                        info!("Player {:?} disconnected", pid);
                    }
                    None => {
                        info!("Unknown player event!");
                    }
                }
            }
            // TODO: feed player comms into server using playermanager.
            // for p in players.iter_mut() {
            //     if let Ok(x) = p.receiver.try_next() {
            //         info!("Player {:?} received {:?}", p.id, x);
            //         p.sender
            //             .send(PlayerMessage::new(
            //                 "HELLO",
            //                 serde_json::json!({"my": "data"}),
            //             ))
            //             .await;
            //     }
            // }
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
