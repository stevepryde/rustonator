use crate::comms::playercomm::PlayerConnectEvent;
use crate::engine::bomb::BombId;
use crate::engine::explosion::ExplosionId;
use crate::engine::mob::MobId;
use crate::engine::player::PlayerId;
use crate::{
    engine::{
        bomb::Bomb, config::GameConfig, explosion::Explosion, mob::Mob, player::Player,
        world::World, worlddata::MobSpawner,
    },
    tools::itemstore::ItemStore,
};
use log::*;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

pub struct GameServer {
    config: GameConfig,
    world: World,
    mob_spawners: Vec<MobSpawner>,
    players_rx: Receiver<PlayerConnectEvent>,
    players: HashMap<PlayerId, Player>,
    mobs: HashMap<MobId, Mob>,
    bombs: ItemStore<BombId, Bomb>,
    explosions: ItemStore<ExplosionId, Explosion>,
}

impl GameServer {
    pub fn new(players_rx: Receiver<PlayerConnectEvent>) -> Self {
        let config = GameConfig::new();
        let mut world = World::new(50, 50, &config);
        let mob_spawners = world.add_mob_spawners();

        GameServer {
            config,
            world,
            mob_spawners,
            players_rx,
            players: HashMap::new(),
            mobs: HashMap::new(),
            bombs: ItemStore::new(),
            explosions: ItemStore::new(),
        }
    }

    pub async fn update(&mut self, delta_time: f64) -> bool {
        self.player_connect_events().await;
        self.player_inputs().await;

        // Update remaining time for all bombs and explosions.
        for explosion in self.explosions.iter_mut() {
            explosion.update(delta_time);
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
            self.add_explosion(explosion);
        }

        self.bombs.retain(|_, b| b.is_active());

        true
    }

    pub fn add_bomb(&mut self, bomb: Bomb) {
        let pos = bomb.position();
        let id = self.bombs.add(bomb);
        self.world.add_bomb(id, pos);
    }

    pub fn add_explosion(&mut self, explosion: Explosion) {
        let pos = explosion.position();
        let id = self.explosions.add(explosion);
        self.world.add_explosion(id, pos);
    }

    pub async fn player_connect_events(&mut self) {
        // Have any players joined?
        if let Ok(x) = self.players_rx.try_recv() {
            match x {
                PlayerConnectEvent::Connected(p) => {
                    info!("Player connected: {:?}", p);
                    self.players.insert(p.id(), Player::new(p.id(), p));
                }
                PlayerConnectEvent::Disconnected(pid) => {
                    info!("Player {:?} disconnected", pid);
                    self.players.retain(|player_id, _| player_id != &pid);
                }
                _ => {
                    info!("Unknown player event!");
                }
            }
        }
    }

    pub async fn player_inputs(&mut self) {
        let mut quit = Vec::new();
        for p in self.players.values_mut() {
            if !p.get_input().await {
                quit.push(p.id());
            }
        }

        for q in quit {
            self.players.retain(|player_id, _| player_id != &q);
        }
    }
}
