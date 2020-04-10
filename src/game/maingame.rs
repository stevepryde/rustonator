use crate::{
    comms::playercomm::{PlayerConnectEvent, PlayerMessage},
    engine::{
        bomb::{Bomb, BombId},
        config::GameConfig,
        explosion::ExplosionId,
        mob::MobId,
        player::{Player, PlayerId},
        position::MapPosition,
        world::World,
        worlddata::InternalCellData,
    },
    error::ZResult,
    game::server::{
        game_process_explosions_and_bombs,
        game_process_mobs,
        spawn_mob,
        BombList,
        ExplosionList,
        MobList,
        PlayerList,
    },
};
use log::*;
use rand::{thread_rng, Rng};
use tokio::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};



pub async fn game_loop(mut player_join_rx: Receiver<PlayerConnectEvent>) -> ZResult<()> {
    let config = GameConfig::new();

    let world_width = 50;
    let world_height = 50;
    let mut world = World::new(world_width, world_height, &config);
    let mob_spawners = world.add_mob_spawners();

    let mut players = PlayerList::new();
    let mut mobs = MobList::new();
    let mut bombs = BombList::new();
    let mut explosions = ExplosionList::new();

    let max_mobs = (world_width as f64 * world_height as f64 * 0.4) as usize;

    // Limit max FPS.
    let min_timeslice: f64 = 1.0 / 60.0;

    let mut last_frame = Instant::now();
    let mut count: u64 = 0;
    let mut first_frame = Instant::now();

    let mut add_blocks_timer = Instant::now();
    let mut mob_spawn_timer = Instant::now();
    let mut next_mob_spawn_seconds = thread_rng().gen_range(1.0, 60.0);

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
        game_process_explosions_and_bombs(delta_time, &mut explosions, &mut bombs, &mut world);
        game_process_mobs(delta_time, &mut mobs, &mut players, &mut explosions, &world);

        // Remove dead mobs.
        mobs.retain(|_, m| m.is_active());

        game_process_players(delta_time, &mut players)

        // Update players.
        world.zones_mut().clear_players();
        for p in players.values_mut() {
            if p.is_dead() {
                continue;
            }

            p.update(delta_time);
        }

        // Remove dead players.
        players.retain(|_, p| !p.is_dead());

        // Spawn new mob ?
        if mob_spawn_timer.elapsed().as_secs_f64() > next_mob_spawn_seconds {
            if mobs.len() < max_mobs {
                spawn_mob(&mut mobs, mob_spawners.clone(), &players, &world);
            }

            mob_spawn_timer = Instant::now();
            next_mob_spawn_seconds = thread_rng().gen_range(1.0, 60.0);
        }

        // Add blocks?
        if add_blocks_timer.elapsed().as_secs() > 10 {
            let entities: Vec<MapPosition> = players
                .values()
                .map(|p| p.position().to_map_position(&world))
                .chain(mobs.iter().map(|m| m.position().to_map_position(&world)))
                .collect();
            world.populate_blocks(&entities);
            add_blocks_timer = Instant::now();
        }

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
)
{
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
