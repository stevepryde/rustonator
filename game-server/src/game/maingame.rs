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
use rand::prelude::*;
use tracing::{debug, error, info, warn};

use tokio::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

const MOB_CAP_DENSITY: f64 = 40.0 / (47.0 * 47.0);

pub struct RustonatorGame {
    world: World,
    players: PlayerList,
    mobs: MobList,
    mob_spawners: Vec<MobSpawner>,
    bombs: BombList,
    explosions: ExplosionList,
    http_client: reqwest::Client,
    scores_url: String,
    scores_secret: String,
}

impl RustonatorGame {
    pub fn new(width: u32, height: u32) -> Self {
        let config = GameConfig::new();
        let mut world = World::new(width as i32, height as i32, &config);
        let mob_spawners = world.add_mob_spawners();
        world.populate_initial(&[]);

        let scores_url = std::env::var("SCORES_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:9003".to_string());
        let scores_secret = std::env::var("SCORES_SECRET")
            .unwrap_or_else(|_| "dev-secret".to_string());

        Self {
            world,
            players: PlayerList::new(),
            mobs: MobList::new(),
            mob_spawners,
            bombs: BombList::new(),
            explosions: ExplosionList::new(),
            http_client: reqwest::Client::new(),
            scores_url,
            scores_secret,
        }
    }

    fn max_mobs(&self) -> usize {
        let map_size = self.world.sizes().map_size();
        ((map_size.width as f64 * map_size.height as f64) * MOB_CAP_DENSITY)
            .round()
            .max(1.0) as usize
    }

    pub async fn game_loop(
        &mut self,
        mut player_join_rx: Receiver<PlayerConnectEvent>,
    ) -> ZResult<()>
    {
        let block_regen_interval = Duration::from_secs(6);

        // Limit max FPS.
        let fps = 30.0;
        let min_timeslice: f64 = 1.0 / fps;

        let mut last_frame = Instant::now();
        let mut count: u64 = 0;
        let mut first_frame = Instant::now();

        let mut add_blocks_timer = Instant::now();
        let mut mob_spawn_timer = Instant::now();

        let mut next_mob_spawn_seconds = rand::rng().random_range(1.0..60.0);

        let idle_cooldown = Duration::from_secs(30);
        let mut idle_since: Option<Instant> = None;
        let mut paused = true;
        info!("Game loop starting paused, waiting for players");

        loop {
            // Pause when no players for 30 seconds.
            if self.players.is_empty() {
                if let Some(since) = idle_since {
                    if since.elapsed() >= idle_cooldown && !paused {
                        info!("No players for 30s, pausing game loop");
                        paused = true;
                    }
                } else {
                    idle_since = Some(Instant::now());
                }
            } else {
                idle_since = None;
                if paused {
                    info!("Player connected, resuming game loop");
                    paused = false;
                    last_frame = Instant::now();
                    first_frame = Instant::now();
                    count = 0;
                }
            }

            if paused {
                let wait_for_regen =
                    block_regen_interval.saturating_sub(add_blocks_timer.elapsed());

                tokio::select! {
                    event = player_join_rx.recv() => match event {
                        Some(event) => self.handle_connect_event(event),
                        None => break,
                    },
                    _ = tokio::time::sleep(wait_for_regen) => {
                        self.regenerate_blocks_if_due(&mut add_blocks_timer, block_regen_interval);
                    }
                }
                continue;
            }

            let mut delta_time = last_frame.elapsed().as_secs_f64();
            if delta_time < min_timeslice {
                delta_time = last_frame.elapsed().as_secs_f64();
                let sleep_time = (min_timeslice - delta_time) * 1_000f64;
                tokio::time::sleep(Duration::from_millis(sleep_time as u64)).await;
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
                if self.mobs.len() < self.max_mobs() {
                    self.spawn_mob();
                }

                mob_spawn_timer = Instant::now();
                next_mob_spawn_seconds = rand::rng().random_range(1.0..60.0);
            }

            // Add blocks?
            self.regenerate_blocks_if_due(&mut add_blocks_timer, block_regen_interval);

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

    fn block_regen_entities(&self) -> Vec<MapPosition> {
        self.players
            .values()
            .filter(|p| p.is_active())
            .map(|p| p.position().to_map_position(&self.world))
            .chain(
                self.mobs
                    .iter()
                    .map(|m| m.position().to_map_position(&self.world)),
            )
            .collect()
    }

    fn regenerate_blocks_if_due(
        &mut self,
        add_blocks_timer: &mut Instant,
        block_regen_interval: Duration,
    ) -> bool
    {
        if add_blocks_timer.elapsed() < block_regen_interval {
            return false;
        }

        self.world.populate_blocks(&self.block_regen_entities());
        *add_blocks_timer = Instant::now();
        true
    }

    fn handle_connect_event(&mut self, event: PlayerConnectEvent) {
        match event {
            PlayerConnectEvent::Connected(p) => {
                info!("Player connected: {:?}", p);
                self.players.insert(p.id(), Player::new(p.id(), p));
            }
            PlayerConnectEvent::Disconnected(pid) => {
                info!("Player {:?} disconnected", pid);
                self.convert_remote_bombs_for_player(pid);
                if let Some(player) = self.players.get(&pid) {
                    self.submit_score(player);
                }
                self.players.retain(|player_id, _| player_id != &pid);
            }
        }
    }

    pub async fn player_connect_events(&mut self, players_rx: &mut Receiver<PlayerConnectEvent>) {
        if let Ok(event) = players_rx.try_recv() {
            self.handle_connect_event(event);
        }
    }

    pub async fn process_player_inputs(&mut self, delta_time: f64) {
        let mut quit = Vec::new();
        for p in self.players.values_mut() {
            if let Ok(false) | Err(_) = p.handle_player_input(&mut self.world, delta_time).await {
                quit.push(p.id());
            }
        }

        for q in quit {
            self.convert_remote_bombs_for_player(q);
            if let Some(player) = self.players.get(&q) {
                self.submit_score(player);
            }
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
            if bomb.tick(delta_time) {
                // Bomb exploded.
                explode_new.push(bomb.id());
            }
        }

        for bomb_id in explode_new.into_iter() {
            let _ = self.world.explode_bomb(
                bomb_id,
                &mut self.bombs,
                &mut self.explosions,
                &mut self.players,
            );
        }

        self.bombs.retain(|_, b| b.is_active());
    }

    pub fn create_bomb_for_player(&mut self, player: &mut Player) {
        if !player.has_bomb_remaining() {
            return;
        }

        let pos = player.position().to_map_position(&self.world);
        if let Some(CellType::Empty) = self.world.get_cell(pos) {
            let remote = player.consume_remote_bomb_charge();
            let bomb = Bomb::new(player, pos, remote);
            player.bomb_placed();
            self.world.add_bomb(bomb, &mut self.bombs);
        }
    }

    pub fn detonate_remote_bomb_for_player(&mut self, player: &mut Player) -> bool {
        let remote_bomb_id = self
            .bombs
            .iter()
            .filter(|bomb| bomb.pid() == player.id() && bomb.is_remote() && bomb.is_active())
            .map(|bomb| bomb.id())
            .min();

        if let Some(bomb_id) = remote_bomb_id {
            let exploded_pids = self.world.explode_bomb(
                bomb_id,
                &mut self.bombs,
                &mut self.explosions,
                &mut self.players,
            );
            for pid in exploded_pids {
                if pid == player.id() {
                    player.bomb_exploded();
                }
            }
            return true;
        }

        false
    }

    fn convert_remote_bombs_for_player(&mut self, player_id: PlayerId) {
        let remote_bomb_ids: Vec<_> = self
            .bombs
            .iter()
            .filter(|bomb| bomb.pid() == player_id && bomb.is_remote() && bomb.is_active())
            .map(|bomb| bomb.id())
            .collect();

        for bomb_id in remote_bomb_ids {
            let mut converted = false;
            if let Some(bomb) = self.bombs.get_mut(bomb_id) {
                bomb.convert_to_timed();
                converted = true;
            }

            if converted {
                self.world.update_bomb_path(bomb_id, &self.bombs);
            }
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
        spawners.shuffle(&mut rand::rng());
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
        let mob_ids: Vec<_> = self.mobs.iter().map(Mob::id).collect();
        for mob_id in mob_ids {
            if self.resolve_mob_explosion_hit(mob_id) {
                continue;
            }

            if let Some(mob) = self.mobs.get_mut(mob_id) {
                mob.update(delta_time, &self.players, &self.world);
            }

            self.resolve_mob_explosion_hit(mob_id);
        }

        // Remove dead mobs.
        self.mobs.retain(|_, m| m.is_active());
    }

    fn resolve_mob_explosion_hit(&mut self, mob_id: crate::engine::mob::MobId) -> bool {
        let Some((killer_id, smart)) = ({
            let Some(mob) = self.mobs.get(mob_id) else {
                return false;
            };
            let pos = mob.position().to_map_position(&self.world);
            self.harmful_explosion_owner_at(pos).map(|killer_id| (killer_id, mob.is_smart()))
        }) else {
            return false;
        };

        if let Some(mob) = self.mobs.get_mut(mob_id) {
            mob.terminate();
        }

        if let Some(p) = self.players.get_mut(&killer_id) && !p.is_dead() {
            p.increase_kill_streak();
            if smart {
                p.increase_score(2000);
            } else {
                p.increase_score(500);
            }
        }

        true
    }

    fn harmful_explosion_owner_at(
        &self,
        map_pos: MapPosition,
    ) -> Option<crate::engine::player::PlayerId> {
        let Some(InternalCellData::Explosion(explosion_id)) = self.world.get_internal_cell(map_pos)
        else {
            return None;
        };

        let explosion = self.explosions.get(*explosion_id)?;
        if !explosion.is_harmful() {
            return None;
        }

        Some(explosion.pid())
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

            if player.is_active() {
                if player.action().fire() {
                    if !self.detonate_remote_bomb_for_player(&mut player) {
                        self.create_bomb_for_player(&mut player);
                    }

                    // Prevent more bombs until the player releases fire.
                    player.action_mut().cease_fire();
                }

                player.update(&self.world, delta_time);
                if let Err(e) = self.process_player_move(&mut player).await {
                    error!(
                        "Error processing move for player: {:?} ({}): {:?}",
                        player.id(),
                        player.name(),
                        e
                    );
                    player.terminate();
                }
            } else {
                player.update(&self.world, delta_time);
            }

            // Reinsert player.
            self.players.insert(player.id(), player);
        }

        // Remove dead players.
        let mut futs = Vec::new();
        for p in self.players.values_mut().filter(|p| p.is_dead()) {
            futs.push(Box::pin(p.ws().disconnect()));
        }
        join_all(futs).await;

        self.players.retain(|_, p| !p.is_dead());
    }

    async fn process_player_move(&mut self, player: &mut Player) -> ZResult<()> {
        let mut reason = String::new();
        let mut died = false;

        if player.is_active() {
            // Did we collect anything?
            let map_pos = player.position().to_map_position(&self.world);
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
                    if player.position().distance_to(mob.position()) <= range {
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
                        break;
                    }
                }

                // Explosion?
                if let Some(InternalCellData::Explosion(explosion_id)) =
                    self.world.get_internal_cell(map_pos)
                    && let Some(explosion) = self.explosions.get(*explosion_id)
                        && explosion.is_harmful() {
                            died = true;

                            // Award points to the player that killed this mob.
                            if explosion.pid() == player.id() {
                                reason = String::from("Oops! You were killed by your own bomb");
                            } else if let Some(p) = self.players.get_mut(&explosion.pid()) {
                                if !p.is_dead() {
                                    reason = format!("You were killed by '{}'", p.name());
                                    p.increase_kill_streak();
                                    p.increase_score(1000);
                                } else {
                                    let pname = explosion.pname();
                                    let pname_str = if pname.is_empty() {
                                        String::from("an unknown player")
                                    } else {
                                        format!("'{}'", pname)
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

        // Send frame update.
        self.send_data_to_player(player).await?;

        if died {
            debug!(
                "Player {:?} ({}, score {}) killed: {}",
                player.id(),
                player.name(),
                player.score(),
                reason
            );
            player.terminate();
            self.convert_remote_bombs_for_player(player.id());
            player.ws().send(PlayerMessage::Dead(reason)).await?;

            self.submit_score(player);
        }

        Ok(())
    }

    fn local_players_for_map_pos(&self, map_pos: MapPosition) -> Vec<&Player> {
        let chunkwidth = self.world.sizes().chunk_size().width;
        let chunkheight = self.world.sizes().chunk_size().height;
        self.players
            .values()
            .filter(|p| {
                p.position().to_map_position(&self.world).is_within_grid(
                    map_pos,
                    chunkwidth,
                    chunkheight,
                )
            })
            .collect()
    }

    fn leaderboard_players(&self) -> Vec<&Player> {
        self.players
            .values()
            .filter(|p| p.has_joined() && !p.is_dead())
            .collect()
    }

    fn leaderboard_players_for<'a>(&'a self, player: &'a Player) -> Vec<&'a Player> {
        let mut leaderboard_players = self.leaderboard_players();
        if player.has_joined() && !player.is_dead() {
            leaderboard_players.push(player);
        }
        leaderboard_players
    }

    async fn send_data_to_player(&self, player: &mut Player) -> ZResult<()> {
        let map_pos = player.position().to_map_position(&self.world);
        let chunkwidth = self.world.sizes().chunk_size().width;
        let chunkheight = self.world.sizes().chunk_size().height;
        let local_players = self.local_players_for_map_pos(map_pos);
        let leaderboard_players = self.leaderboard_players_for(player);

        let local_mobs: Vec<&Mob> = self
            .mobs
            .iter()
            .filter(|m| {
                m.position().to_map_position(&self.world).is_within_grid(
                    map_pos,
                    chunkwidth,
                    chunkheight,
                )
            })
            .collect();

        let local_bombs: Vec<&Bomb> = self
            .bombs
            .iter()
            .filter(|b| {
                b.position()
                    .is_within_grid(map_pos, chunkwidth, chunkheight)
            })
            .collect();

        let local_explosions: Vec<&Explosion> = self
            .explosions
            .iter()
            .filter(|e| {
                e.position()
                    .is_within_grid(map_pos, chunkwidth, chunkheight)
            })
            .collect();

        let ser_data = serde_json::json!({
            "player": player,
            "world": self.world.get_chunk_data(map_pos),
            "players": local_players,
            "leaderboardPlayers": leaderboard_players,
            "mobs": local_mobs,
            "bombs": local_bombs,
            "explosions": local_explosions
        });

        player.ws().send(PlayerMessage::FrameData(ser_data)).await?;
        Ok(())
    }

    fn submit_score(&self, player: &Player) {
        if player.score() == 0 || player.name().is_empty() {
            return;
        }

        let client = self.http_client.clone();
        let url = format!("{}/api/scores", self.scores_url);
        let secret = self.scores_secret.clone();
        let name = player.name().to_string();
        let score = player.score();
        tokio::spawn(async move {
            let res = client
                .post(&url)
                .bearer_auth(&secret)
                .json(&serde_json::json!({ "name": name, "score": score }))
                .send()
                .await;
            if let Err(e) = res {
                warn!("Failed to submit score: {e}");
            }
        });
    }
}

#[cfg(test)]
mod test_harness;

#[cfg(test)]
mod tests {
    use super::*;
    use self::test_harness::{Direction, ScenarioHarness};
    use crate::{comms::playercomm::PlayerComm, engine::position::PixelPositionF64};
    use tokio::sync::mpsc;

    fn count_mystery_blocks(game: &RustonatorGame) -> usize {
        let map_size = game.world.sizes().map_size();
        let mut count = 0;

        for y in 0..map_size.height {
            for x in 0..map_size.width {
                if matches!(game.world.get_cell(MapPosition::new(x, y)), Some(CellType::Mystery)) {
                    count += 1;
                }
            }
        }

        count
    }

    fn clear_mystery_blocks(game: &mut RustonatorGame) {
        let map_size = *game.world.sizes().map_size();

        for y in 0..map_size.height {
            for x in 0..map_size.width {
                let pos = MapPosition::new(x, y);
                if matches!(game.world.get_cell(pos), Some(CellType::Mystery)) {
                    game.world.set_cell(pos, CellType::Empty);
                }
            }
        }
    }

    fn test_player(id: u64) -> Player {
        let (tx, _rx_external) = mpsc::channel(1);
        let (_tx_external, rx) = mpsc::channel(1);
        let mut player = Player::new(
            PlayerId::from(id),
            PlayerComm::new(PlayerId::from(id), tx, rx),
        );
        player.set_name("test");
        player
    }

    #[test]
    fn block_regen_repopulates_world_when_timer_is_due() {
        let mut game = RustonatorGame::new(47, 47);
        clear_mystery_blocks(&mut game);
        assert_eq!(count_mystery_blocks(&game), 0);

        let interval = Duration::from_secs(10);
        let mut timer = Instant::now() - interval;

        assert!(game.regenerate_blocks_if_due(&mut timer, interval));
        assert!(count_mystery_blocks(&game) > 0);
    }

    #[test]
    fn block_regen_does_not_run_before_timer_is_due() {
        let mut game = RustonatorGame::new(47, 47);
        clear_mystery_blocks(&mut game);
        assert_eq!(count_mystery_blocks(&game), 0);

        let interval = Duration::from_secs(10);
        let mut timer = Instant::now();

        assert!(!game.regenerate_blocks_if_due(&mut timer, interval));
        assert_eq!(count_mystery_blocks(&game), 0);
    }

    #[test]
    fn mob_cap_scales_to_forty_for_current_map_size() {
        let game = RustonatorGame::new(47, 47);
        assert_eq!(game.max_mobs(), 40);
    }

    #[test]
    fn mob_cap_scales_with_map_size() {
        let small = RustonatorGame::new(23, 23);
        let large = RustonatorGame::new(71, 71);

        assert!(small.max_mobs() < 40);
        assert!(large.max_mobs() > 40);
    }

    #[test]
    fn player_death_converts_remote_bombs_into_timed_bombs() {
        let mut game = RustonatorGame::new(47, 47);
        clear_mystery_blocks(&mut game);

        let bomb_pos = MapPosition::new(1, 1);
        game.world.set_cell(bomb_pos, CellType::Empty);

        let mut player = test_player(1);
        player.set_position(PixelPositionF64::from_map_position(bomb_pos, &game.world));
        player.grant_remote_bomb_charges(1);
        game.create_bomb_for_player(&mut player);

        assert_eq!(game.bombs.len(), 1);
        let bomb_id = game.bombs.iter().next().unwrap().id();
        assert!(game.bombs.get(bomb_id).unwrap().is_remote());

        game.convert_remote_bombs_for_player(player.id());

        let bomb = game.bombs.get(bomb_id).unwrap();
        assert!(!bomb.is_remote());
        assert!(!bomb.timestamp().is_zero());
    }

    #[test]
    fn disconnect_event_converts_remote_bombs_before_removing_player() {
        let mut game = RustonatorGame::new(47, 47);
        clear_mystery_blocks(&mut game);

        let bomb_pos = MapPosition::new(1, 1);
        game.world.set_cell(bomb_pos, CellType::Empty);

        let mut player = test_player(7);
        player.set_position(PixelPositionF64::from_map_position(bomb_pos, &game.world));
        player.grant_remote_bomb_charges(1);
        let player_id = player.id();

        game.create_bomb_for_player(&mut player);
        game.players.insert(player_id, player);

        let bomb_id = game.bombs.iter().next().unwrap().id();
        assert!(game.bombs.get(bomb_id).unwrap().is_remote());

        game.handle_connect_event(PlayerConnectEvent::Disconnected(player_id));

        let bomb = game.bombs.get(bomb_id).unwrap();
        assert!(!bomb.is_remote());
        assert!(!bomb.timestamp().is_zero());
        assert!(!game.players.contains_key(&player_id));
    }

    #[tokio::test]
    async fn input_disconnect_converts_remote_bombs_before_removing_player() {
        let mut harness = ScenarioHarness::new_empty_arena(47, 47);
        let player_id = harness.add_joined_player_at(9, MapPosition::new(1, 1), "carol").await;
        harness.grant_remote_bomb_charges(player_id, 1);

        harness.queue_fire(player_id);
        harness.tick_default().await;
        assert_eq!(harness.snapshot().bombs.len(), 1);
        assert!(harness.snapshot().bombs[0].remote);

        harness.drop_input(player_id);
        harness.tick_default().await;

        let snapshot = harness.snapshot();
        assert!(snapshot.players.is_empty());
        assert_eq!(snapshot.bombs.len(), 1);
        assert!(!snapshot.bombs[0].remote);
    }

    #[tokio::test]
    async fn scenario_timed_bomb_explodes_and_restores_player_bomb_capacity() {
        let mut harness = ScenarioHarness::new_empty_arena(47, 47);
        let player_id = harness.add_joined_player_at(1, MapPosition::new(1, 1), "alice").await;

        harness.queue_fire(player_id);
        harness.tick_default().await;

        let snapshot = harness.snapshot();
        assert_eq!(snapshot.bombs.len(), 1);
        assert!(!snapshot.bombs[0].remote);
        assert_eq!(harness.player_cur_bombs(player_id), Some(1));

        harness.tick_for_seconds(3.2).await;

        let snapshot = harness.snapshot();
        assert!(snapshot.bombs.is_empty());
        assert_eq!(harness.player_cur_bombs(player_id), Some(0));
        assert!(matches!(
            harness.cell(MapPosition::new(1, 1)),
            Some(CellType::Empty)
        ));
    }

    #[tokio::test]
    async fn scenario_remote_bomb_reverts_and_then_explodes_after_disconnect() {
        let mut harness = ScenarioHarness::new_empty_arena(47, 47);
        let player_id = harness.add_joined_player_at(7, MapPosition::new(1, 1), "bob").await;
        harness.grant_remote_bomb_charges(player_id, 1);

        harness.queue_fire(player_id);
        harness.tick_default().await;

        let snapshot = harness.snapshot();
        assert_eq!(snapshot.bombs.len(), 1);
        assert!(snapshot.bombs[0].remote);

        harness.disconnect_event(player_id);

        let snapshot = harness.snapshot();
        assert!(snapshot.players.is_empty());
        assert_eq!(snapshot.bombs.len(), 1);
        assert!(!snapshot.bombs[0].remote);

        harness.tick_for_seconds(3.2).await;

        let snapshot = harness.snapshot();
        assert!(snapshot.bombs.is_empty());
        assert!(matches!(
            harness.cell(MapPosition::new(1, 1)),
            Some(CellType::Empty)
        ));
    }

    #[tokio::test]
    async fn leaderboard_keeps_far_away_players_even_when_not_locally_visible() {
        let mut harness = ScenarioHarness::new_empty_arena(47, 47);
        let player_one = harness
            .add_joined_player_at(1, MapPosition::new(1, 1), "alice")
            .await;
        let _player_two = harness
            .add_joined_player_at(2, MapPosition::new(41, 41), "bob")
            .await;

        assert_eq!(harness.leaderboard_player_count(), 2);
        assert_eq!(harness.visible_player_count_for(player_one), 1);
    }

    #[tokio::test]
    async fn frame_payload_leaderboard_includes_current_player_even_when_alone() {
        let mut harness = ScenarioHarness::new_empty_arena(47, 47);
        let player = harness
            .add_joined_player_at(1, MapPosition::new(1, 1), "alice")
            .await;

        let frame = harness
            .last_frame_data(player)
            .expect("expected a frame payload for the joined player");
        let leaderboard = frame["leaderboardPlayers"]
            .as_array()
            .expect("leaderboardPlayers should be an array");

        assert_eq!(leaderboard.len(), 1);
        assert_eq!(leaderboard[0]["name"].as_str(), Some("alice"));
    }

    #[tokio::test]
    async fn scenario_later_bomb_is_triggered_early_by_chain_reaction() {
        let mut harness = ScenarioHarness::new_horizontal_corridor(47, 47, 3);
        let player_one = harness
            .add_joined_player_at(1, MapPosition::new(3, 3), "alice")
            .await;
        let player_two = harness
            .add_joined_player_at(2, MapPosition::new(4, 3), "bob")
            .await;

        harness.queue_fire(player_one);
        harness.tick_default().await;
        assert_eq!(harness.snapshot().bombs.len(), 1);

        harness.tick_for_seconds(1.0).await;

        harness.queue_fire(player_two);
        harness.tick_default().await;
        assert_eq!(harness.snapshot().bombs.len(), 2);

        harness.tick_for_seconds(2.2).await;

        let snapshot = harness.snapshot();
        assert!(snapshot.bombs.is_empty());
        assert_eq!(harness.player_cur_bombs(player_one), Some(0));
        assert_eq!(harness.player_cur_bombs(player_two), Some(0));
    }

    #[tokio::test]
    async fn scenario_enemy_bomb_kill_awards_score_and_multiplier() {
        let mut harness = ScenarioHarness::new_horizontal_corridor(47, 47, 3);
        let killer = harness
            .add_joined_player_at(1, MapPosition::new(5, 3), "alice")
            .await;
        let victim = harness
            .add_joined_player_at(2, MapPosition::new(6, 3), "bob")
            .await;

        harness.wait_for_spawn_invincibility_to_expire().await;

        harness.queue_action(killer, -1, 0, true, false);
        harness.tick_default().await;
        harness.move_for_ticks(killer, Direction::Left, 10).await;
        harness.tick_for_seconds(2.8).await;

        assert_eq!(harness.player_score(killer), Some(2000));
        assert_eq!(harness.player_score_multiplier(killer), Some(2));
        assert_eq!(harness.player_is_active(killer), Some(true));
        assert_eq!(harness.player_is_active(victim), Some(false));
        assert_eq!(harness.last_dead_reason(victim), Some("You were killed by 'alice'"));
    }

    #[tokio::test]
    async fn scenario_self_bomb_kill_reports_own_bomb_reason_without_score_gain() {
        let mut harness = ScenarioHarness::new_horizontal_corridor(47, 47, 3);
        let player = harness
            .add_joined_player_at(1, MapPosition::new(5, 3), "alice")
            .await;

        harness.wait_for_spawn_invincibility_to_expire().await;

        harness.queue_fire(player);
        harness.tick_default().await;
        harness.tick_for_seconds(3.2).await;

        assert_eq!(harness.player_score(player), Some(0));
        assert_eq!(harness.player_is_active(player), Some(false));
        assert_eq!(
            harness.last_dead_reason(player),
            Some("Oops! You were killed by your own bomb")
        );
    }

    #[tokio::test]
    async fn scenario_block_regen_does_not_spawn_on_players_or_bombs() {
        let mut harness = ScenarioHarness::new_empty_arena(47, 47);
        let _player = harness
            .add_joined_player_at(1, MapPosition::new(11, 11), "alice")
            .await;
        let bomber = harness
            .add_joined_player_at(2, MapPosition::new(21, 21), "bob")
            .await;

        harness.queue_fire(bomber);
        harness.tick_default().await;

        let player_pos = MapPosition::new(11, 11);
        let bomb_pos = harness.snapshot().bombs[0].position;

        for _ in 0..10 {
            harness.force_block_regen();
            assert!(matches!(harness.cell(player_pos), Some(CellType::Empty)));
            assert!(matches!(harness.cell(bomb_pos), Some(CellType::Bomb)));
        }
    }

    #[tokio::test]
    async fn scenario_reverted_remote_bomb_chain_reacts_from_shorter_timed_bomb() {
        let mut harness = ScenarioHarness::new_horizontal_corridor(47, 47, 3);
        let remote_owner = harness
            .add_joined_player_at(1, MapPosition::new(5, 3), "alice")
            .await;
        let timed_owner = harness
            .add_joined_player_at(2, MapPosition::new(6, 3), "bob")
            .await;

        harness.grant_remote_bomb_charges(remote_owner, 1);
        harness.decrease_bomb_time(timed_owner, 1);

        harness.queue_fire(remote_owner);
        harness.tick_default().await;
        harness.disconnect_event(remote_owner);

        harness.queue_fire(timed_owner);
        harness.tick_default().await;

        let snapshot = harness.snapshot();
        assert_eq!(snapshot.bombs.len(), 2);
        assert_eq!(harness.player_cur_bombs(timed_owner), Some(1));

        harness.tick_for_seconds(2.3).await;

        let snapshot = harness.snapshot();
        assert!(snapshot.bombs.is_empty());
        assert_eq!(harness.player_cur_bombs(timed_owner), Some(0));
    }

    #[tokio::test]
    async fn chained_bomb_does_not_blast_through_block_destroyed_in_same_tick() {
        let mut harness = ScenarioHarness::new_horizontal_corridor(47, 47, 3);
        let bomber_one = harness
            .add_joined_player_at(1, MapPosition::new(5, 3), "alice")
            .await;
        let bomber_two = harness
            .add_joined_player_at(2, MapPosition::new(6, 3), "bob")
            .await;
        let victim = harness
            .add_joined_player_at(3, MapPosition::new(8, 3), "charlie")
            .await;

        harness.increase_range(bomber_one, 1);
        harness.increase_range(bomber_two, 1);
        harness.set_mystery(MapPosition::new(7, 3));
        harness.wait_for_spawn_invincibility_to_expire().await;

        harness.queue_fire(bomber_one);
        harness.tick_default().await;
        harness.queue_fire(bomber_two);
        harness.tick_default().await;

        harness.tick_for_seconds(3.2).await;

        assert_eq!(harness.player_is_active(victim), Some(true));
        assert!(!matches!(harness.cell(MapPosition::new(7, 3)), Some(CellType::Mystery)));
    }

    #[tokio::test]
    async fn player_can_move_into_block_tile_after_explosion_tick_has_passed() {
        let mut harness = ScenarioHarness::new_horizontal_corridor(47, 47, 3);
        let bomber = harness
            .add_joined_player_at(1, MapPosition::new(5, 3), "alice")
            .await;
        let runner = harness
            .add_joined_player_at(2, MapPosition::new(7, 3), "bob")
            .await;

        harness.set_mystery(MapPosition::new(6, 3));
        harness.wait_for_spawn_invincibility_to_expire().await;

        harness.queue_fire(bomber);
        harness.tick_default().await;
        harness.tick_for_seconds(3.2).await;

        harness.move_for_ticks(runner, Direction::Left, 6).await;

        assert_eq!(harness.player_is_active(runner), Some(true));
        assert_eq!(
            harness
                .snapshot()
                .players
                .iter()
                .find(|player| player.id == runner)
                .map(|player| player.position),
            Some(MapPosition::new(6, 3))
        );
    }

    #[tokio::test]
    async fn mob_standing_in_fresh_explosion_dies_before_it_can_move_away() {
        let mut harness = ScenarioHarness::new_horizontal_corridor(47, 47, 3);
        let bomber = harness
            .add_joined_player_at(1, MapPosition::new(5, 3), "alice")
            .await;

        harness.wait_for_spawn_invincibility_to_expire().await;

        harness.queue_fire(bomber);
        harness.tick_default().await;
        harness.tick_for_seconds(2.9).await;
        harness.add_mob_at(MapPosition::new(6, 3));
        harness.tick_for_seconds(0.2).await;

        assert_eq!(harness.mob_count(), 0);
        assert!(harness.player_score(bomber).is_some_and(|score| score >= 500));
    }
}
