use bracket_lib::{prelude::{BTerm, RGB, BLACK, to_cp437, Point, DistanceAlg::Pythagoras}, terminal::{TOMATO, WHITE, ALICE_BLUE, GREEN}, pathfinding::a_star_search };
use crate::player::{map::{Map, WIDTH}, Player};

#[derive(PartialEq, Clone, Copy)]
pub enum EntityType {
    Zombie, FrozenTroll, Human, Orc
}

#[derive(Clone)]
pub struct Entity {
    pub t: EntityType,
    pub x: i32,
    pub y: i32,
    is_angry: bool,
    symbol: char,
    color: (u8, u8, u8),
    damage: u16,
    hits: u8,
    hits_max: u8,
    attack_phrase: String,
    find_phrase: String,
    first_sight: bool,
}

impl Entity {
    pub fn new(x: i32, y: i32, t: EntityType) -> Self {
        let (s, c, angry, damage, max_hits, ats, fis): 
        (char, (u8, u8, u8), bool, u16, u8, &str, &str) = match t {
            EntityType::Zombie => ('Z', TOMATO, true, 5, 2, "The zombie hurt you!", "The zombie growls at you!"),
            EntityType::FrozenTroll => ('t', ALICE_BLUE, true, 5, 2, "The frozen troll hurt you!", "The frozen troll snorts at you!"),
            EntityType::Human => ('☺', WHITE, false, 0, 0, "", "The traveler greets you!"),
            EntityType::Orc => ('o', GREEN, true, 10, 1, "The orc hurt you!", "The orc yells at you!"),
        };
        Self { 
            t: t, 
            is_angry: angry,
            x: x, 
            y: y,
            symbol: s,
            color: c,
            damage: damage,
            hits: 0,
            hits_max: max_hits,
            attack_phrase: String::from(ats),
            find_phrase: String::from(fis),
            first_sight: true,
        }
    }

    pub fn draw_and_tick(&mut self, player: &mut Player, map: &mut Map, is_tick: bool, engine: &mut BTerm) -> String {
        let mut str: String = String::new();
        
        if is_tick {
            str = self.goto_player(player, map);
        }

        engine.set(self.x, self.y, 
            RGB::named(self.color), 
            RGB::named(BLACK), 
            to_cp437(self.symbol));
        
        str
    }

    fn goto_player(&mut self, player: &mut Player, map: &mut Map) -> String {
        let mut str: String = String::new();
        if player.is_visible((self.x, self.y)) {
            if self.first_sight {
                str = self.find_phrase.clone();
                self.first_sight = false;
            }

            if self.is_angry {
                let path = a_star_search(
                    Map::xy_idx(self.x, self.y), 
                    Map::xy_idx(player.pos.0, player.pos.1), 
                    &mut *map);

                if path.steps.len() > 1 {
                    let future_x = path.steps[1] as i32 % WIDTH;
                    let future_y = path.steps[1] as i32 / WIDTH;

                    if self.hits >= self.hits_max + 1 {
                        self.hits = 0;
                    } 
                    else if self.hits == self.hits_max {
                        self.hits += 1;
                    }

                    if (future_x, future_y) != player.pos && self.hits < self.hits_max {                     
                        self.x = future_x;
                        self.y = future_y;
                    }
                    if Pythagoras.distance2d(
                        Point::new(player.pos.0, player.pos.1), Point::new(future_x, future_y)) <= 1.5 {
                        if player.health > 0 {
                            player.health -= self.damage;
                        }

                        str = self.attack_phrase.clone();
                        self.hits += 1;
                    }    
                }
            }
        }
        else {
            self.first_sight = !self.first_sight;
        }

        str
    }
}