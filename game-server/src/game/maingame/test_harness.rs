use super::RustonatorGame;
use crate::{
    comms::playercomm::{JoinGameData, PlayerComm, PlayerMessage, PlayerMessageExternal},
    component::action::Action,
    engine::{
        mob::Mob,
        player::{Player, PlayerId},
        position::{MapPosition, PixelPositionF64},
        worlddata::InternalCellData,
    },
    traits::celltypes::CellType,
};
use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::time::{Duration, Instant};

const DEFAULT_TICK_SECONDS: f64 = 1.0 / 30.0;
const TEST_CHANNEL_CAPACITY: usize = 256;
const SPAWN_INVINCIBILITY_SECONDS: f64 = 5.0;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn components(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSnapshot {
    pub id: PlayerId,
    pub position: MapPosition,
    pub active: bool,
    pub cur_bombs: u32,
    pub score: u32,
    pub score_multiplier: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BombSnapshot {
    pub owner_id: PlayerId,
    pub position: MapPosition,
    pub remote: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExplosionSnapshot {
    pub position: MapPosition,
    pub harmful: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioSnapshot {
    pub players: Vec<PlayerSnapshot>,
    pub bombs: Vec<BombSnapshot>,
    pub explosions: Vec<ExplosionSnapshot>,
}

pub struct ScenarioHarness {
    game: RustonatorGame,
    inbound: HashMap<PlayerId, Sender<PlayerMessageExternal>>,
    outbound: HashMap<PlayerId, Receiver<PlayerMessageExternal>>,
    outbound_log: HashMap<PlayerId, Vec<PlayerMessage>>,
    next_message_uid: u64,
}

impl ScenarioHarness {
    pub fn new_empty_arena(width: u32, height: u32) -> Self {
        let mut harness = Self {
            game: RustonatorGame::new(width, height),
            inbound: HashMap::new(),
            outbound: HashMap::new(),
            outbound_log: HashMap::new(),
            next_message_uid: 1,
        };
        harness.clear_to_empty_arena();
        harness
    }

    pub fn new_horizontal_corridor(width: u32, height: u32, corridor_y: i32) -> Self {
        let mut harness = Self::new_empty_arena(width, height);
        let map_size = *harness.game.world.sizes().map_size();
        for y in 1..map_size.height - 1 {
            if y == corridor_y {
                continue;
            }
            for x in 1..map_size.width - 1 {
                harness.game.world.set_cell(MapPosition::new(x, y), CellType::Wall);
            }
        }
        harness
    }

    pub async fn add_joined_player_at(
        &mut self,
        id: u64,
        position: MapPosition,
        name: &str,
    ) -> PlayerId {
        let player_id = PlayerId::from(id);
        let (server_tx, server_rx) = channel(TEST_CHANNEL_CAPACITY);
        let (client_tx, client_rx) = channel(TEST_CHANNEL_CAPACITY);
        let player = Player::new(player_id, PlayerComm::new(player_id, server_tx, client_rx));

        self.game.players.insert(player_id, player);
        self.inbound.insert(player_id, client_tx);
        self.outbound.insert(player_id, server_rx);

        self.send_message(
            player_id,
            PlayerMessage::JoinGame(JoinGameData {
                name: name.to_owned(),
                character: None,
            }),
        );
        self.tick_default().await;
        self.set_player_position(player_id, position);
        self.assert_invariants();
        player_id
    }

    pub fn set_player_position(&mut self, player_id: PlayerId, position: MapPosition) {
        if let Some(player) = self.game.players.get_mut(&player_id) {
            player.set_position(PixelPositionF64::from_map_position(position, &self.game.world));
        }
    }

    pub fn grant_remote_bomb_charges(&mut self, player_id: PlayerId, amount: u32) {
        if let Some(player) = self.game.players.get_mut(&player_id) {
            player.grant_remote_bomb_charges(amount);
        }
    }

    pub fn decrease_bomb_time(&mut self, player_id: PlayerId, amount: u32) {
        if let Some(player) = self.game.players.get_mut(&player_id) {
            for _ in 0..amount {
                player.decrease_bomb_time();
            }
        }
    }

    pub fn increase_range(&mut self, player_id: PlayerId, amount: u32) {
        if let Some(player) = self.game.players.get_mut(&player_id) {
            for _ in 0..amount {
                player.increase_range();
            }
        }
    }

    pub fn add_mob_at(&mut self, position: MapPosition) {
        let mut mob = Mob::new();
        mob.set_position(PixelPositionF64::from_map_position(position, &self.game.world));
        self.game.mobs.add(mob);
    }

    #[allow(dead_code)]
    pub fn increase_max_bombs(&mut self, player_id: PlayerId, amount: u32) {
        if let Some(player) = self.game.players.get_mut(&player_id) {
            for _ in 0..amount {
                player.increase_max_bombs();
            }
        }
    }

    pub fn queue_action(
        &mut self,
        player_id: PlayerId,
        x: i32,
        y: i32,
        fire: bool,
        prefer_y: bool,
    ) {
        let mut action = Action::new();
        action.set(x, y, fire);
        action.set_prefer_y(prefer_y);
        self.send_message(player_id, PlayerMessage::Action(action));
    }

    pub fn queue_fire(&mut self, player_id: PlayerId) {
        self.queue_action(player_id, 0, 0, true, false);
    }

    pub fn queue_move(&mut self, player_id: PlayerId, direction: Direction) {
        let (x, y) = direction.components();
        self.queue_action(player_id, x, y, false, false);
    }

    pub fn disconnect_event(&mut self, player_id: PlayerId) {
        self.game
            .handle_connect_event(crate::comms::playercomm::PlayerConnectEvent::Disconnected(
                player_id,
            ));
        self.inbound.remove(&player_id);
        self.outbound.remove(&player_id);
        self.assert_invariants();
    }

    pub fn drop_input(&mut self, player_id: PlayerId) {
        self.inbound.remove(&player_id);
    }

    pub async fn tick_default(&mut self) {
        self.tick(DEFAULT_TICK_SECONDS).await;
    }

    pub async fn tick_n(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.tick_default().await;
        }
    }

    pub async fn tick_for_seconds(&mut self, seconds: f64) {
        let ticks = (seconds / DEFAULT_TICK_SECONDS).ceil().max(1.0) as usize;
        self.tick_n(ticks).await;
    }

    pub async fn move_for_ticks(&mut self, player_id: PlayerId, direction: Direction, ticks: usize) {
        for _ in 0..ticks {
            self.queue_move(player_id, direction);
            self.tick_default().await;
        }
    }

    pub async fn wait_for_spawn_invincibility_to_expire(&mut self) {
        self.tick_for_seconds(SPAWN_INVINCIBILITY_SECONDS + DEFAULT_TICK_SECONDS).await;
    }

    pub fn force_block_regen(&mut self) {
        let interval = Duration::from_secs(6);
        let mut timer = Instant::now() - interval;
        let _ = self.game.regenerate_blocks_if_due(&mut timer, interval);
        self.assert_invariants();
    }

    pub async fn tick(&mut self, delta_time: f64) {
        self.game.process_player_inputs(delta_time).await;
        self.game.game_process_explosions_and_bombs(delta_time);
        self.game.game_process_mobs(delta_time);
        self.game.game_process_players(delta_time).await;
        self.drain_outbound_messages();
        self.inbound.retain(|player_id, _| self.game.players.contains_key(player_id));
        self.outbound
            .retain(|player_id, _| self.game.players.contains_key(player_id));
        self.assert_invariants();
    }

    pub fn snapshot(&self) -> ScenarioSnapshot {
        let mut players: Vec<_> = self
            .game
            .players
            .values()
            .map(|player| PlayerSnapshot {
                id: player.id(),
                position: player.position().to_map_position(&self.game.world),
                active: player.is_active(),
                cur_bombs: player.cur_bombs(),
                score: player.score(),
                score_multiplier: player.score_multiplier(),
            })
            .collect();
        players.sort_by_key(|player| (player.position.y, player.position.x));

        let mut bombs: Vec<_> = self
            .game
            .bombs
            .iter()
            .map(|bomb| BombSnapshot {
                owner_id: bomb.pid(),
                position: bomb.position(),
                remote: bomb.is_remote(),
            })
            .collect();
        bombs.sort_by_key(|bomb| (bomb.position.y, bomb.position.x, bomb.remote));

        let mut explosions: Vec<_> = self
            .game
            .explosions
            .iter()
            .map(|explosion| ExplosionSnapshot {
                position: explosion.position(),
                harmful: explosion.is_harmful(),
            })
            .collect();
        explosions.sort_by_key(|explosion| (explosion.position.y, explosion.position.x));

        ScenarioSnapshot {
            players,
            bombs,
            explosions,
        }
    }

    pub fn player_cur_bombs(&self, player_id: PlayerId) -> Option<u32> {
        self.game.players.get(&player_id).map(Player::cur_bombs)
    }

    pub fn player_score(&self, player_id: PlayerId) -> Option<u32> {
        self.game.players.get(&player_id).map(Player::score)
    }

    pub fn player_score_multiplier(&self, player_id: PlayerId) -> Option<u32> {
        self.game.players.get(&player_id).map(Player::score_multiplier)
    }

    pub fn player_is_active(&self, player_id: PlayerId) -> Option<bool> {
        self.game.players.get(&player_id).map(Player::is_active)
    }

    pub fn last_dead_reason(&self, player_id: PlayerId) -> Option<&str> {
        self.outbound_log.get(&player_id).and_then(|messages| {
            messages.iter().rev().find_map(|message| match message {
                PlayerMessage::Dead(reason) => Some(reason.as_str()),
                _ => None,
            })
        })
    }

    pub fn last_frame_data(&self, player_id: PlayerId) -> Option<&serde_json::Value> {
        self.outbound_log.get(&player_id).and_then(|messages| {
            messages.iter().rev().find_map(|message| match message {
                PlayerMessage::FrameData(data) => Some(data),
                _ => None,
            })
        })
    }

    pub fn leaderboard_player_count(&self) -> usize {
        self.game.leaderboard_players().len()
    }

    pub fn visible_player_count_for(&self, player_id: PlayerId) -> usize {
        let player = self
            .game
            .players
            .get(&player_id)
            .expect("player missing from scenario");
        self.game
            .local_players_for_map_pos(player.position().to_map_position(&self.game.world))
            .len()
    }

    pub fn cell(&self, position: MapPosition) -> Option<CellType> {
        self.game.world.get_cell(position)
    }

    pub fn mob_count(&self) -> usize {
        self.game.mobs.len()
    }

    #[allow(dead_code)]
    pub fn set_cell(&mut self, position: MapPosition, cell_type: CellType) {
        self.game.world.set_cell(position, cell_type);
    }

    #[allow(dead_code)]
    pub fn set_wall(&mut self, position: MapPosition) {
        self.set_cell(position, CellType::Wall);
    }

    #[allow(dead_code)]
    pub fn set_mystery(&mut self, position: MapPosition) {
        self.set_cell(position, CellType::Mystery);
    }

    #[allow(dead_code)]
    pub fn clear_cell(&mut self, position: MapPosition) {
        self.set_cell(position, CellType::Empty);
    }

    fn send_message(&mut self, player_id: PlayerId, message: PlayerMessage) {
        let uid = self.next_message_uid;
        self.next_message_uid += 1;
        self.inbound
            .get(&player_id)
            .expect("player input sender missing")
            .try_send(PlayerMessageExternal::new(uid, message))
            .expect("failed to queue test message");
    }

    fn drain_outbound_messages(&mut self) {
        for (player_id, receiver) in &mut self.outbound {
            while let Ok(message) = receiver.try_recv() {
                self.outbound_log
                    .entry(*player_id)
                    .or_default()
                    .push(message.message().clone());
            }
        }
    }

    fn clear_to_empty_arena(&mut self) {
        let map_size = *self.game.world.sizes().map_size();
        for y in 0..map_size.height {
            for x in 0..map_size.width {
                let pos = MapPosition::new(x, y);
                if !matches!(self.game.world.get_cell(pos), Some(CellType::Wall)) {
                    self.game.world.set_cell(pos, CellType::Empty);
                }
            }
        }
        self.game.mob_spawners.clear();
    }

    fn assert_invariants(&self) {
        let mut bomb_counts: HashMap<PlayerId, u32> = HashMap::new();

        for bomb in self.game.bombs.iter() {
            assert!(bomb.is_active(), "inactive bomb left in bomb list at {:?}", bomb.position());
            assert!(
                matches!(self.game.world.get_cell(bomb.position()), Some(CellType::Bomb)),
                "bomb cell missing for bomb at {:?}: {:?}",
                bomb.position(),
                self.game.world.get_cell(bomb.position())
            );
            match self.game.world.get_internal_cell(bomb.position()) {
                Some(InternalCellData::Bomb(bomb_id)) => {
                    assert_eq!(*bomb_id, bomb.id(), "internal bomb id mismatch at {:?}", bomb.position());
                }
                other => {
                    panic!(
                        "missing internal bomb reference at {:?}: {:?}",
                        bomb.position(),
                        other
                    );
                }
            }

            *bomb_counts.entry(bomb.pid()).or_insert(0) += 1;
        }

        let map_size = self.game.world.sizes().map_size();
        for y in 0..map_size.height {
            for x in 0..map_size.width {
                let pos = MapPosition::new(x, y);
                if matches!(self.game.world.get_cell(pos), Some(CellType::Bomb)) {
                    match self.game.world.get_internal_cell(pos) {
                        Some(InternalCellData::Bomb(bomb_id)) => {
                            let bomb = self
                                .game
                                .bombs
                                .get(*bomb_id)
                                .unwrap_or_else(|| panic!("bomb cell at {:?} points to missing bomb", pos));
                            assert!(bomb.is_active(), "bomb cell at {:?} points to inactive bomb", pos);
                        }
                        other => {
                            panic!("bomb cell at {:?} missing internal reference: {:?}", pos, other);
                        }
                    }
                }
            }
        }

        for player in self.game.players.values() {
            let active_bombs = bomb_counts.get(&player.id()).copied().unwrap_or(0);
            assert_eq!(
                player.cur_bombs(),
                active_bombs,
                "player {:?} bomb count out of sync",
                player.id()
            );
        }
    }
}
