use crate::{
    comms::playercomm::{PlayerConnectEvent, PlayerMessage},
    engine::{
        bomb::Bomb,
        config::GameConfig,
        explosion::Explosion,
        mob::Mob,
        player::{Player, PlayerFlags, PlayerId},
        position::{MapPosition, PixelPositionF64},
        types::{BombList, ExplosionList, MobList, PlayerList},
        world::World,
        worlddata::{InternalCellData, MobSpawner},
    },
    error::ZResult,
    traits::celltypes::CellType,
};
use futures::future::join_all;
use log::*;
use rand::{seq::SliceRandom, thread_rng, Rng};

use tokio::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

pub struct RustonatorGame {
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
        world.populate_initial(&[]);

        Self {
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
    ) -> ZResult<()> {
        let max_mobs = (self.width as f64 * self.height as f64 * 0.4) as usize;

        // Limit max FPS.
        let fps = 30.0;
        let min_timeslice: f64 = 1.0 / fps;

        let mut last_frame = Instant::now();
        let mut count: u64 = 0;
        let mut first_frame = Instant::now();

        let mut add_blocks_timer = Instant::now();
        let mut mob_spawn_timer = Instant::now();

        let mut next_mob_spawn_seconds = thread_rng().gen_range(1.0, 60.0);

        loop {
            let mut delta_time = last_frame.elapsed().as_secs_f64();
            if delta_time < min_timeslice {
                // Only allow new players if we have time.

                // NOTE: We need to use an async delay here just in case the server
                //       happens to be running on a single thread.
                //       If performance suffers, we could check for the number of
                //       CPU cores / tokio threads on startup and switch to using
                //       thread::sleep() here in the case where multiple threads
                //       are supported.

                delta_time = last_frame.elapsed().as_secs_f64();
                let sleep_time = (min_timeslice - delta_time) * 1_000f64;
                tokio::time::delay_for(Duration::from_millis(sleep_time as u64)).await;
                delta_time = last_frame.elapsed().as_secs_f64();
            }
            last_frame = Instant::now();

            self.player_connect_events(&mut player_join_rx).await;
            self.process_player_inputs(delta_time).await;
            self.game_process_explosions_and_bombs(delta_time);
            self.game_process_mobs(delta_time);
            self.game_process_players(delta_time).await;

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
                    .map(|p| p.position.to_map_position(&self.world))
                    .chain(
                        self.mobs
                            .iter()
                            .map(|m| m.position.to_map_position(&self.world)),
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

    pub async fn process_player_inputs(&mut self, delta_time: f64) {
        let mut quit = Vec::new();
        for p in self.players.values_mut() {
            if let Ok(false) | Err(_) = p.handle_player_input(&mut self.world, delta_time).await {
                quit.push(p.id);
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
            if !explosion.active {
                self.world.clear_explosion_cell(explosion);
            }
        }

        self.explosions.retain(|_, e| e.active);

        let mut explode_new = Vec::new();
        for bomb in self.bombs.iter_mut() {
            if bomb.tick(delta_time) {
                // Bomb exploded.
                explode_new.push(bomb.id);
            }
        }

        for bomb_id in explode_new.into_iter() {
            self.world.explode_bomb(
                bomb_id,
                &mut self.bombs,
                &mut self.explosions,
                &mut self.players,
            );
        }

        self.bombs.retain(|_, b| b.active);
    }

    pub fn create_bomb_for_player(&mut self, player: &mut Player) {
        if !player.has_bomb_remaining() {
            return;
        }

        let pos = player.position.to_map_position(&self.world);
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
            .map(|m| m.position.to_map_position(&self.world))
            .collect();
        let mut spawners = self.mob_spawners.clone();
        spawners.shuffle(&mut rand::thread_rng());
        for spawner in spawners {
            if !self
                .world
                .is_nearby_map_entity(spawner.position(), &mob_positions, 3)
            {
                let mut mob = Mob::new();
                mob.position = PixelPositionF64::from_map_position(spawner.position(), &self.world);
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
                .get_internal_cell(mob.position.to_map_position(&self.world))
            {
                if let Some(explosion) = self.explosions.get(*explosion_id) {
                    if explosion.harmful {
                        mob.terminate();

                        // Award points to the player that killed this mob.
                        if let Some(p) = self.players.get_mut(&explosion.pid) {
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

        // Remove dead mobs.
        self.mobs.retain(|_, m| m.active);
    }

    pub async fn game_process_players(&mut self, delta_time: f64) {
        // Update players.
        self.world.zones_mut().clear_players();
        let player_ids: Vec<PlayerId> = self.players.keys().copied().collect();
        for pid in player_ids {
            // This is why people move to an ECS :(
            // Rust needs to remove the player from the list while processing it.
            let mut player = match self.players.remove(&pid) {
                Some(p) => p,
                None => {
                    continue;
                }
            };
            if player.frame_list.is_empty() {
                player.action.clear();
            } else {
                let f = player.frame_list.remove(0);
                // TODO: validate frame.
                player.action = f.action;
            }

            if player.is_active() && player.action.fire() {
                self.create_bomb_for_player(&mut player);

                // Prevent more bombs until the player releases fire.
                player.action.cease_fire();
            }

            player.update(&self.world, delta_time);
            if let Err(e) = self.process_player_move(&mut player).await {
                error!(
                    "Error processing move for player: {:?} ({}): {:?}",
                    player.id, player.name, e
                );
                player.terminate();
            }

            // Reinsert player.
            self.players.insert(player.id, player);
        }

        // Remove dead players.
        let mut futs = Vec::new();
        for p in self.players.values_mut().filter(|p| p.is_dead()) {
            futs.push(Box::pin(p.ws.disconnect()));
        }
        join_all(futs).await;

        self.players.retain(|_, p| !p.is_dead());
    }

    async fn process_player_move(&mut self, player: &mut Player) -> ZResult<()> {
        let mut reason = String::new();
        let mut died = false;

        if player.is_active() {
            // Did we collect anything?
            let map_pos = player.position.to_map_position(&self.world);
            match self.world.get_cell(map_pos) {
                Some(CellType::Empty) | None => {}
                Some(CellType::MobSpawner) => {
                    if !player.has_flag(PlayerFlags::Invincible) {
                        // You ded.
                        died = true;
                        reason = String::from("You touched a robot spawner");

                        // Create explosion but don't add it to the world. It is for display only.
                        let explosion = Explosion::new(None, map_pos);
                        self.explosions.add(explosion);
                    }
                }
                Some(ct) => {
                    if player.got_item(ct).await? {
                        self.world.set_cell(map_pos, CellType::Empty);
                    }
                }
            }

            // Did we touch something we shouldn't have?
            if !player.has_flag(PlayerFlags::Invincible) {
                // Mob?
                let range = self.world.sizes().tile_size().width as f64 / 2.0;
                for mob in self.mobs.iter() {
                    if player.position.distance_to(mob.position) <= range {
                        // You ded.
                        died = true;
                        reason = if mob.is_smart() {
                            String::from("You were killed by a robot overlord")
                        } else {
                            String::from("You were killed by a robot")
                        };

                        // Create explosion but don't add it to the world. It is for display only.
                        let explosion = Explosion::new(None, map_pos);
                        self.explosions.add(explosion);
                    }
                }

                // Explosion?
                if let Some(InternalCellData::Explosion(explosion_id)) =
                    self.world.get_internal_cell(map_pos)
                {
                    if let Some(explosion) = self.explosions.get(*explosion_id) {
                        if explosion.harmful {
                            died = true;

                            // Award points to the player that killed this mob.
                            if explosion.pid == player.id {
                                reason = String::from("Oops! You were killed by your own bomb");
                            } else if let Some(p) = self.players.get_mut(&explosion.pid) {
                                if !p.is_dead() {
                                    reason = format!("You were killed by '{}'", p.name);
                                    p.increase_score(1000);
                                } else {
                                    let pname_str = if explosion.pname.is_empty() {
                                        String::from("an unknown player")
                                    } else {
                                        format!("'{}'", explosion.pname)
                                    };

                                    reason = format!(
                                        "You were killed by {}, who has already died since \
                                         placing that bomb",
                                        pname_str
                                    );
                                }
                            } else {
                                reason = String::from(
                                    "Hmm...you died from an explosion but we don't know whose it \
                                     was",
                                );
                            }
                        }
                    }
                }
            }
        }

        // Send frame update.
        self.send_data_to_player(player).await?;

        if died {
            debug!(
                "Player {:?} ({}, score {}) killed: {}",
                player.id, player.name, player.score, reason
            );
            player.terminate();
            player.ws.send(PlayerMessage::Dead(reason)).await?;
        }

        Ok(())
    }

    async fn send_data_to_player(&self, player: &mut Player) -> ZResult<()> {
        let map_pos = player.position.to_map_position(&self.world);
        let chunkwidth = self.world.sizes().chunk_size().width;
        let chunkheight = self.world.sizes().chunk_size().height;
        let local_players: Vec<&Player> = self
            .players
            .values()
            .filter(|p| {
                p.position.to_map_position(&self.world).is_within_grid(
                    map_pos,
                    chunkwidth,
                    chunkheight,
                )
            })
            .collect();

        let local_mobs: Vec<&Mob> = self
            .mobs
            .iter()
            .filter(|m| {
                m.position.to_map_position(&self.world).is_within_grid(
                    map_pos,
                    chunkwidth,
                    chunkheight,
                )
            })
            .collect();

        let local_bombs: Vec<&Bomb> = self
            .bombs
            .iter()
            .filter(|b| b.position.is_within_grid(map_pos, chunkwidth, chunkheight))
            .collect();

        let local_explosions: Vec<&Explosion> = self
            .explosions
            .iter()
            .filter(|e| e.position.is_within_grid(map_pos, chunkwidth, chunkheight))
            .collect();

        let ser_data = serde_json::json!({
            "player": player,
            "world": self.world.get_chunk_data(map_pos),
            "players": local_players,
            "mobs": local_mobs,
            "bombs": local_bombs,
            "explosions": local_explosions
        });

        player.ws.send(PlayerMessage::FrameData(ser_data)).await?;
        Ok(())
    }
}
