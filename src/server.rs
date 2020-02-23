use crate::engine::bomb::BombId;
use crate::engine::explosion::ExplosionId;
use crate::{
    engine::{
        bomb::Bomb, config::GameConfig, explosion::Explosion, mob::Mob, player::Player,
        world::World, worlddata::MobSpawner,
    },
    tools::itemstore::ItemStore,
};
use std::collections::HashMap;

pub struct GameServer {
    config: GameConfig,
    world: World,
    mob_spawners: Vec<MobSpawner>,
    players: HashMap<String, Player>,
    mobs: HashMap<u32, Mob>,
    bombs: ItemStore<BombId, Bomb>,
    explosions: ItemStore<ExplosionId, Explosion>,
}

impl Default for GameServer {
    fn default() -> Self {
        let config = GameConfig::new();
        let mut world = World::new(50, 50, &config);
        let mob_spawners = world.add_mob_spawners();

        GameServer {
            config,
            world,
            mob_spawners,
            players: HashMap::new(),
            mobs: HashMap::new(),
            bombs: ItemStore::new(),
            explosions: ItemStore::new(),
        }
    }
}

impl GameServer {
    pub fn new() -> Self {
        GameServer::default()
    }

    pub fn update(&mut self, delta_time: f64) -> bool {
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
}
