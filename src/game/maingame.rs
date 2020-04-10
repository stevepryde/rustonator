use crate::{
    comms::playercomm::PlayerConnectEvent,
    engine::{
        bomb::Bomb,
        config::GameConfig,
        mob::Mob,
        player::Player,
        position::{MapPosition, PixelPositionF64},
        types::{BombList, ExplosionList, MobList, PlayerList},
        world::World,
        worlddata::{InternalCellData, MobSpawner},
    },
    error::ZResult,
    traits::celltypes::CellType,
};
use log::*;
use rand::{seq::SliceRandom, thread_rng, Rng};
use tokio::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

pub struct RustonatorGame {
    config: GameConfig,
    width: u32,
    height: u32,
    world: World,
    players: PlayerList,
    mobs: MobList,
    mob_spawners: Vec<MobSpawner>,
    bombs: BombList,
    explosions: ExplosionList,
}

impl RustonatorGame {
    pub fn new(width: u32, height: u32) -> Self {
        let config = GameConfig::new();
        let mut world = World::new(width as i32, height as i32, &config);
        let mob_spawners = world.add_mob_spawners();
        Self {
            config,
            width,
            height,
            world,
            players: PlayerList::new(),
            mobs: MobList::new(),
            mob_spawners,
            bombs: BombList::new(),
            explosions: ExplosionList::new(),
        }
    }

    pub async fn game_loop(
        &mut self,
        mut player_join_rx: Receiver<PlayerConnectEvent>,
    ) -> ZResult<()>
    {
        let max_mobs = (self.width as f64 * self.height as f64 * 0.4) as usize;

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

            self.player_connect_events(&mut player_join_rx).await;
            self.process_player_inputs().await;
            self.game_process_explosions_and_bombs(delta_time);
            self.game_process_mobs(delta_time);

            // Remove dead mobs.
            self.mobs.retain(|_, m| m.is_active());

            // self.game_process_players(delta_time, &mut players)

            // Update players.
            self.world.zones_mut().clear_players();
            for p in self.players.values_mut() {
                if p.is_dead() {
                    continue;
                }

                p.update(delta_time);
            }

            // Remove dead players.
            self.players.retain(|_, p| !p.is_dead());

            // Spawn new mob ?
            if mob_spawn_timer.elapsed().as_secs_f64() > next_mob_spawn_seconds {
                if self.mobs.len() < max_mobs {
                    self.spawn_mob();
                }

                mob_spawn_timer = Instant::now();
                next_mob_spawn_seconds = thread_rng().gen_range(1.0, 60.0);
            }

            // Add blocks?
            if add_blocks_timer.elapsed().as_secs() > 10 {
                let entities: Vec<MapPosition> = self
                    .players
                    .values()
                    .map(|p| p.position().to_map_position(&self.world))
                    .chain(
                        self.mobs
                            .iter()
                            .map(|m| m.position().to_map_position(&self.world)),
                    )
                    .collect();
                self.world.populate_blocks(&entities);
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

    pub async fn player_connect_events(&mut self, players_rx: &mut Receiver<PlayerConnectEvent>) {
        // Have any players joined?
        if let Ok(x) = players_rx.try_recv() {
            match x {
                PlayerConnectEvent::Connected(p) => {
                    info!("Player connected: {:?}", p);
                    self.players.insert(p.id(), Player::new(p.id(), p));
                }
                PlayerConnectEvent::Disconnected(pid) => {
                    info!("Player {:?} disconnected", pid);
                    self.players.retain(|player_id, _| player_id != &pid);
                }
            }
        }
    }

    pub async fn process_player_inputs(&mut self) {
        let mut quit = Vec::new();
        for p in self.players.values_mut() {
            if let Ok(false) | Err(_) = p.handle_player_input(&mut self.world).await {
                quit.push(p.id());
            }
        }

        for q in quit {
            self.players.retain(|player_id, _| player_id != &q);
        }
    }

    pub fn game_process_explosions_and_bombs(&mut self, delta_time: f64) {
        // Update remaining time for all bombs and explosions.
        for explosion in self.explosions.iter_mut() {
            explosion.update(delta_time);
            if !explosion.is_active() {
                self.world.clear_explosion_cell(explosion);
            }
        }

        self.explosions.retain(|_, e| e.is_active());

        let mut explode_new = Vec::new();
        for bomb in self.bombs.iter_mut() {
            if let Some(x) = bomb.tick(delta_time) {
                // Bomb exploded.
                explode_new.push(x);
            }
        }

        for explosion in explode_new.into_iter() {
            self.world.add_explosion(explosion, &mut self.explosions);
        }

        self.bombs.retain(|_, b| b.is_active());
    }

    pub fn create_bomb_for_player(&mut self, player: &mut Player) {
        if !player.has_bomb_remaining() {
            return;
        }

        let pos = player.position().to_map_position(&self.world);
        if let Some(CellType::Empty) = self.world.get_cell(pos) {
            let bomb = Bomb::new(player, pos);
            player.bomb_placed();
            self.world.add_bomb(bomb, &mut self.bombs);
        }
    }

    /// Spawn mob at a random mob spawner, and assign it a new target.
    pub fn spawn_mob(&mut self) {
        let mob_positions: Vec<MapPosition> = self
            .mobs
            .iter()
            .map(|m| m.position().to_map_position(&self.world))
            .collect();
        let mut spawners = self.mob_spawners.clone();
        spawners.shuffle(&mut rand::thread_rng());
        for spawner in spawners {
            if !self
                .world
                .is_nearby_map_entity(spawner.position(), &mob_positions, 3)
            {
                let mut mob = Mob::new();
                mob.set_position(PixelPositionF64::from_map_position(
                    spawner.position(),
                    &self.world,
                ));
                mob.choose_new_target(&self.world, &self.players);
                self.mobs.add(mob);
                break;
            }
        }
    }

    pub fn game_process_mobs(&mut self, delta_time: f64) {
        for mob in self.mobs.iter_mut() {
            mob.update(delta_time, &self.players, &self.world);

            // Check if mob is dead.
            if let Some(InternalCellData::Explosion(explosion_id)) = self
                .world
                .get_internal_cell(mob.position().to_map_position(&self.world))
            {
                if let Some(explosion) = self.explosions.get(*explosion_id) {
                    if explosion.is_active() {
                        mob.terminate();

                        // Award points to the player that killed this mob.
                        if let Some(p) = self.players.get_mut(&explosion.pid()) {
                            if !p.is_dead() {
                                if mob.is_smart() {
                                    p.increase_score(2000);
                                } else {
                                    p.increase_score(500);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
