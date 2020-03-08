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
use tokio::sync::mpsc::{channel, Receiver};

use crate::comms::playercomm::PlayerConnectEvent;
use crate::error::ZResult;
use fern;
use log::info;
use std::thread;
use tokio::runtime;
use tokio::runtime::Runtime;
use tokio::stream::StreamExt;

#[tokio::main]
async fn main() {
    init_logging();

    let (player_join_tx, player_join_rx) = channel(30);

    tokio::spawn(spawn_websocket_server(player_join_tx));
    if let Err(e) = game_loop(player_join_rx).await {
        eprintln!("Error: {:?}", e);
    }
}

async fn game_loop(mut player_join_rx: Receiver<PlayerConnectEvent>) -> ZResult<()> {
    let mut game_server = GameServer::new();

    let mut players = Vec::new();
    // Limit max FPS.
    let min_timeslice: f64 = 1.0 / 60.0;

    let mut last_frame = Instant::now();
    let mut count = 0;
    let first_frame = Instant::now();
    loop {
        let delta_time = last_frame.elapsed().as_secs_f64();
        if delta_time < min_timeslice {
            let sleep_time = (min_timeslice - delta_time) * 1_000f64;

            // NOTE: We need to use an async delay here just in case the server
            //       happens to be running on a single thread.
            //       If performance suffers, we could check for the number of
            //       CPU cores / tokio threads on startup and switch to using
            //       thread::sleep() here in the case where multiple threads
            //       are supported.
            tokio::time::delay_for(Duration::from_millis(sleep_time as u64)).await;
        }
        last_frame = Instant::now();
        // Have any players joined?
        // TODO: create playermanager to manage player events.
        if let Ok(x) = player_join_rx.try_recv() {
            match x {
                PlayerConnectEvent::Connected(p) => {
                    info!("Player connected: {:?}", p);
                    players.push(p);
                }
                PlayerConnectEvent::Disconnected(pid) => {
                    info!("Player {:?} disconnected", pid);
                    players.retain(|p| p.id() != pid);
                }
                _ => {
                    info!("Unknown player event!");
                }
            }
        }
        // TODO: feed player comms into server using playermanager.
        let mut quit = Vec::new();
        for p in players.iter_mut() {
            if let Ok(x) = p.recv_one().await {
                if x.is_none() {
                    quit.push(p.id());
                }

                info!("Player {:?} received {:?}", p.id(), x);
            }
        }

        for q in quit {
            players.retain(|v| v.id() != q);
        }

        if !game_server.update(delta_time) {
            break;
        }

        count += 1;
        println!("COUNT: {}", count);
        println!(
            "FPS: {:.2}",
            count as f64 / first_frame.elapsed().as_secs_f64()
        )
    }

    Ok(())
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
