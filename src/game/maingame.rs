use crate::comms::playercomm::{PlayerConnectEvent, PlayerMessage};
use crate::engine::bomb::{Bomb, BombId};
use crate::engine::config::GameConfig;
use crate::engine::explosion::ExplosionId;
use crate::engine::mob::MobId;
use crate::engine::player::Player;
use crate::engine::world::World;
use crate::error::ZResult;
use crate::game::server::{
    add_explosion_to_world, process_explosions, BombList, ExplosionList, MobList, PlayerList,
};
use log::*;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Duration, Instant};

pub async fn game_loop(mut player_join_rx: Receiver<PlayerConnectEvent>) -> ZResult<()> {
    let config = GameConfig::new();
    let mut world = World::new(50, 50, &config);
    let mob_spawners = world.add_mob_spawners();

    let mut players = PlayerList::new();
    let mut mobs = MobList::new();
    let mut bombs = BombList::new();
    let mut explosions = ExplosionList::new();

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

        player_connect_events(&mut player_join_rx, &mut players).await;
        process_player_inputs(&mut players, &mut world).await;
        process_explosions(delta_time, &mut explosions, &mut bombs, &mut world);

        // Remove dead players.
        players.retain(|_, p| !p.is_dead());

        count += 1;

        let elapsed = first_frame.elapsed().as_secs_f64();
        if elapsed > 5.0 {
            info!("FPS: {:.2}", count as f64 / elapsed);
            count = 0;
            first_frame = Instant::now();
        }
    }
}

pub async fn player_connect_events(
    players_rx: &mut Receiver<PlayerConnectEvent>,
    players: &mut PlayerList,
) {
    // Have any players joined?
    if let Ok(x) = players_rx.try_recv() {
        match x {
            PlayerConnectEvent::Connected(p) => {
                info!("Player connected: {:?}", p);
                players.insert(p.id(), Player::new(p.id(), p));
            }
            PlayerConnectEvent::Disconnected(pid) => {
                info!("Player {:?} disconnected", pid);
                players.retain(|player_id, _| player_id != &pid);
            }
        }
    }
}

pub async fn process_player_inputs(players: &mut PlayerList, world: &mut World) {
    let mut quit = Vec::new();
    for p in players.values_mut() {
        if let Ok(false) | Err(_) = p.handle_player_input(world).await {
            quit.push(p.id());
        }
    }

    for q in quit {
        players.retain(|player_id, _| player_id != &q);
    }
}
