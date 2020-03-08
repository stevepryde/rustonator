use crate::comms::playercomm::PlayerConnectEvent;
use crate::error::ZResult;
use crate::game::server::GameServer;
use log::*;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Duration, Instant};

pub async fn game_loop(player_join_rx: Receiver<PlayerConnectEvent>) -> ZResult<()> {
    let mut game_server = GameServer::new(player_join_rx);

    // Limit max FPS.
    let min_timeslice: f64 = 1.0 / 60.0;

    let mut last_frame = Instant::now();
    let mut count: u64 = 0;
    let mut first_frame = Instant::now();

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

        if !game_server.update(delta_time).await {
            break;
        }

        count += 1;

        let elapsed = first_frame.elapsed().as_secs_f64();
        if elapsed > 5.0 {
            info!("FPS: {:.2}", count as f64 / elapsed);
            count = 0;
            first_frame = Instant::now();
        }
    }

    Ok(())
}
