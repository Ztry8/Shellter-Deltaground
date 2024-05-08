use std::fmt;

use crate::{location::{xy_idx, WorldLocation}, player::Player, State};
use bracket_lib::{
    color::WHITE, pathfinding::a_star_search, prelude::{BTerm, DistanceAlg::Pythagoras, Point}, random::RandomNumberGenerator, terminal::GREEN,
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum EntityType {
    Zombie,
    Skeleton,
    Orc,
    Cyclops,
    Goblin,
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub x: i32,
    pub y: i32,
    pub health: i32,
    pub damage: i32,
    entity_type: EntityType,
    hits: u8,
    hits_max: u8,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Entity {
    pub fn new(x: i32, y: i32, t: EntityType, exp: i32) -> Self {
        let (damage, max_hits, mut health) = match t {
            EntityType::Skeleton => (1, 3, 12),
            EntityType::Zombie => (1, 5, 15),
            EntityType::Orc => (3, 1, 20),
            EntityType::Cyclops => (4, 1, 25),
            EntityType::Goblin => (2, 2, 15),
        };

        health *= 1+exp/100;
        Self {
            x,
            y,
            entity_type: t,
            damage: damage,
            hits: 0,
            hits_max: max_hits,
            health: health,
        }
    }

    fn draw(&self, engine: &mut BTerm, player_pos: (i32, i32)) {
        let data = match self.entity_type {
            EntityType::Zombie => ('Z', GREEN),
            EntityType::Skeleton => ('s', WHITE),
            EntityType::Orc => ('o', GREEN),
            EntityType::Cyclops => ('c', GREEN),
            EntityType::Goblin => ('g', GREEN),
        };
        
        if self.health > 0 {
            State::put(engine, player_pos, self.x, self.y, data.1, data.0);
        } else {
            State::put_red(engine, player_pos, self.x, self.y, data.1, data.0);
        }
    }

    pub fn name(&self) -> String {
        self.entity_type.to_string()
    }
    
    pub fn update(&mut self, player: &mut Player, map: &mut WorldLocation, engine: &mut BTerm) -> String {
        let luck;
        if player.exp >= 100 {
            luck = RandomNumberGenerator::new().range(1, 4); 
        } else {
            luck = 1;
        }
        let damage = self.damage * luck*(1+player.exp/50);

        let mut ret = String::from("");
        if self.health > 0 {
            let s: String = self.name();
            let path = a_star_search(
                xy_idx(self.x, self.y, map.width),
                xy_idx(player.pos.0, player.pos.1, map.width),
                &*map,
            );
            if path.steps.len() > 1 {
                let future_x = path.steps[1] as i32 % map.width;
                let future_y = path.steps[1] as i32 / map.width;

                if !map.is_solid((future_x, future_y)) && (future_x, future_y) != player.pos 
                    && self.hits < self.hits_max {
                    self.x = future_x;
                    self.y = future_y;
                }

                if self.hits >= self.hits_max {
                    self.hits = 0;
                } 
            }

            if Pythagoras.distance2d(
                Point::new(player.pos.0, player.pos.1),
                Point::new(self.x, self.y),
            ) <= 1.0 {  
                // if player.coins > 0 && self.entity_type == EntityType::Bandit {
                //     if player.coins < damage as i32 {
                //         let msg;
                //         if player.coins == 1 {
                //             msg = "coin";
                //         } else {
                //             msg = "coins";
                //         }

                //         let str = format!("The {} has stoled from you {} {}!", s.clone(), player.coins,
                //             msg);
                //         player.coins = 0;
                //         ret = str;
                //     } else {
                //         player.coins -= damage as i32;
                //         ret = format!("The {} has stoled from you {} coins!", s.clone(), damage);
                //     }
                // } else {
                self.hits += 1;

                if RandomNumberGenerator::new().range(1, 101) >= player.luck*player.agility*4-5 {
                    if RandomNumberGenerator::new().range(0, 11) <= 2 {
                        player.health -= damage*2;
                        ret = format!("{} critically hits player, for {}HP", s.clone(), damage*2);
                    } else {
                        player.health -= damage;
                        ret = format!("{} hits player, for {}HP", s.clone(), damage);
                    }
                } else if player.weight <= player.max_weight/2.0 {
                    ret = format!("Player evades from {} hit", s.clone());
                }
            }
        }
        self.draw(engine, player.pos);
        ret
    }
}
