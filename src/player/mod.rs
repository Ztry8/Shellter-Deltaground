use std::{cmp::max, fmt};

use bracket_lib::{
    color::{HOTPINK, ORANGE_RED, RED, WHITESMOKE}, prelude::field_of_view, random::RandomNumberGenerator, terminal::{to_cp437, BTerm, Point, VirtualKeyCode, BLACK, RGB, WHITE}
};

pub mod gui;
use crate::{location::{xy_idx, TileType, WorldLocation}, REAL_HEIGHT, REAL_WIDTH};

use self::gui::{BOTTOM_TABLE_HEIGHT, RIGHT_TABLE_WIDTH};

#[derive(Clone, PartialEq)]
pub struct Player {
    pub pos: (i32, i32),
    pub health: i32,
    pub max_health: i32,
    pub coins: i32,
    pub exp: i32,
    pub visible_tiles: Vec<Point>,
    pub visible_tiles_far: Vec<Point>,
    pub damage: i32,
    pub weapon: Option<ItemType>,
    pub inventory: Vec<ItemType>,
    pub kchance: i32, 
    pub lockpick: i32,
    pub luck: i32, 
    pub strength: i32, 
    pub intelligence: i32, 
    pub agility: i32,
    pub fov_range: i32,
    pub weight: f32,
    pub max_weight: f32,
    blood: i32,
}

const SMALL_POTION_WEIGHT: f32 = 0.7;

#[derive(PartialEq, Clone, Debug)]
pub enum WeaponType {
    Sword,
    Axe, 
    Dagger,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Size {
    Small, 
    Average,
    Large,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Material {
    Bronze,
    Silver,
}

#[derive(PartialEq, Clone, Debug)]
pub enum ItemType {
    Potion(Size, (u8, u8, u8)),
    Weapon(WeaponType, Material, i32),
}

impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for WeaponType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Material {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Player {
    pub fn new(x: i32, y: i32, fov_range: i32, 
        luck: i32, strength: i32, intelligence: i32, agility: i32) -> Self {
        Self {
            pos: (x, y),
            visible_tiles: vec![],
            visible_tiles_far: vec![],
            fov_range,
            coins: 2+luck/2,
            exp: 0,
            health: 20+strength*2+agility/2,
            max_health: 20+strength*2+agility/2,
            damage: 5+strength/2+agility/2,
            kchance: 1+luck/2,
            inventory: vec![],
            lockpick: 2+agility*2+intelligence/2, 
            luck: luck, 
            strength: strength, 
            intelligence: intelligence,
            agility: agility,
            weight: 0.0,
            weapon: None,
            max_weight: (20+strength*2+luck/2) as f32,
            blood: 0,
        }
    }

    fn fight(&mut self, xy: (i32, i32), map: &mut WorldLocation) -> String {
        for entity in &mut map.entities {
            if entity.x == xy.0 && entity.y == xy.1 {
                let mut ss = String::new();
                if entity.health > 0 {
                    if RandomNumberGenerator::new().range(1, 101) <= self.kchance {
                        entity.health -= self.damage*2;
                        ss = format!("Player critically hits {}, for {}HP", entity.name(), self.damage*2)
                    } else {
                        entity.health -= self.damage;
                        ss = format!("Player hits {}, for {}HP", entity.name(), self.damage)
                    }
                }
                if entity.health <= 0 {
                    self.exp += 5;
                    return format!("{} and kills {} +5EXP", ss, entity.name())
                }
                return ss
            }
        }
        if map.source[xy_idx(xy.0, xy.1, map.width)] == TileType::Door {
            if RandomNumberGenerator::new().range(1, 101) <= self.lockpick &&
                map.colors[xy_idx(xy.0, xy.1, map.width)] == WHITESMOKE {
                    map.source[xy_idx(xy.0, xy.1, map.width)] = TileType::Floor;
                    return format!("You broke down the door")
            } else {
                map.colors[xy_idx(xy.0, xy.1, map.width)] = WHITE;
            }
        }
            
        String::new()
    }

    pub fn use_inventory(&mut self, i: i32) -> String {
        let mut s = String::new();
        if i < self.inventory.len() as i32 && i >= 0  {
            s = match &mut self.inventory[i as usize].clone() {
                ItemType::Potion(size, color) => {
                    let k = match size {
                        Size::Small => 10,
                        Size::Average => 4,
                        Size::Large => 2,
                    };

                    match color {
                        &mut ORANGE_RED => {
                            let health = self.max_health/k;
                            if self.health + health >= self.max_health {
                                self.health = self.max_health;
                            } else {
                                self.health += health; 
                            }

                            self.weight -= Player::weight_by_size(size.clone());
                            format!("You drink {} {} potion, +{}HP!", size.to_string().to_lowercase(),
                                Player::color_to_str(color.clone()), health)
                        }
                        _ => format!(""),
                    }
                },
                ItemType::Weapon(weapon_type, material, damage) => {
                    if let Some(ItemType::Weapon(typ, mat, dam)) = &self.weapon.clone() {
                        self.damage += -dam + damage.clone();
                        self.inventory.push(self.weapon.clone().unwrap());
                        self.weapon = Some(ItemType::Weapon(weapon_type.clone(), material.clone(), damage.clone()));

                        let mut ss = format!("{} damage", -dam + damage.clone());
                        if -dam + damage.clone() > 0 {
                            ss = format!("+{} damage", -dam + damage.clone());
                        }

                        format!("You equip {} {} instead of {} {} {}", 
                            material.to_string().to_lowercase(), 
                            weapon_type.to_string().to_lowercase(), 
                            mat.to_string().to_lowercase(), 
                            typ.to_string().to_lowercase(), ss)
                    } else {
                        self.damage += damage.clone();
                        self.weapon = Some(ItemType::Weapon(weapon_type.clone(), material.clone(), damage.clone()));
                        format!("You equip {} {} +{} damage", material.to_string().to_lowercase(), 
                            weapon_type.to_string().to_lowercase(), damage)
                    }
                }
            };
            self.inventory.remove(i as usize);        
        }
        s
    }

    pub fn action(&mut self, key: VirtualKeyCode, map: &mut WorldLocation) -> String {
        match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::A => self.try_move(map, -1, 0),

            VirtualKeyCode::Right |
            VirtualKeyCode::D => self.try_move(map, 1, 0),

            VirtualKeyCode::Up |
            VirtualKeyCode::W => self.try_move(map, 0, -1),

            VirtualKeyCode::Down |
            VirtualKeyCode::S => self.try_move(map, 0, 1),

            VirtualKeyCode::F3 => self.try_move(map, 0, 0),
            _ => String::new(),
        }
    }

    pub fn draw(&self, engine: &mut BTerm) {
        let mut color = WHITE;
        if self.blood > 0 {
            color = RED;
        }

        engine.set(
            (REAL_WIDTH - RIGHT_TABLE_WIDTH) / 2,
            (REAL_HEIGHT - BOTTOM_TABLE_HEIGHT) / 2,
            RGB::named(color),
            RGB::named(BLACK),
            to_cp437('@'),
        );
    }

    pub fn color_to_str(color: (u8, u8, u8)) -> String {
        let s = match color {
            ORANGE_RED => "red",
            _ => "",
        };
        String::from(s)
    }

    fn weight_by_size(size: Size) -> f32 {
        match size {
            Size::Small => SMALL_POTION_WEIGHT,
            Size::Average => SMALL_POTION_WEIGHT*2.0,
            Size::Large => SMALL_POTION_WEIGHT*3.0,
        }
    }

    pub fn weight_by_weapon(weapon_type: WeaponType, material: Material) -> f32 {
        let weight = 0.5 + match material {
            Material::Bronze => 0.2,
            _ => 0.0,
        };

        weight + match weapon_type {
            WeaponType::Sword => 0.5,
            WeaponType::Axe => 1.5,
            _ => 0.0,
        }
    }

    fn event(&mut self, map: &mut WorldLocation) -> String {
        let message = match map.source[xy_idx(self.pos.0, self.pos.1, map.width)] {
            TileType::Coin => { 
                let coins = RandomNumberGenerator::new().range(1, 4);
                self.coins += coins; 
                format!("You pick up {} coins", coins)
            },
            TileType::Potion => {
                match map.colors[xy_idx(self.pos.0, self.pos.1, map.width)] {
                    HOTPINK => {
                        self.max_health += self.max_health/50+1; 
                        format!("You drink pink potion, +{}HP!", self.max_health/50+1)
                    }
                    _ => {
                        if self.weight + SMALL_POTION_WEIGHT <= self.max_weight && self.inventory.len()+1 <= 25 {
                            self.weight += SMALL_POTION_WEIGHT;
                            self.inventory.push(ItemType::Potion(Size::Small, 
                                map.colors[xy_idx(self.pos.0, self.pos.1, map.width)]));
                            format!("You pick up small {} potion +{}kg", 
                                Player::color_to_str(map.colors[xy_idx(self.pos.0, self.pos.1, map.width)]), 
                                SMALL_POTION_WEIGHT)
                        } else {
                            format!("You can't carry more than {}kg and more than 25 items", self.weight)
                        }
                    }
                }
            },
            TileType::Exit => {
                format!("You go down to the floor below...")
            }
            TileType::BearTrap => {
                map.source[xy_idx(self.pos.0, self.pos.1, map.width)] = TileType::BearTrapActived;
                if RandomNumberGenerator::new().range(1, 101) <= 
                    50-self.agility*2+(self.weight/2.0) as i32 {
                    self.health -= self.max_health/4;
                    self.blood += 5;
                    format!("You fall into a trap and you bleed out for 5 moves, -{}HP", self.max_health/4)
                } else {
                    format!("Trap doesn't work so you don't fall into a trap")
                }
            }
            TileType::Chest => {
                let mut damage = 1;
                let material = match RandomNumberGenerator::new().range(1, 11) {
                    0..=8 => Material::Bronze,
                    _ => {
                        damage *= 2;
                        Material::Silver
                    },
                };
                let weapon_type = match RandomNumberGenerator::new().range(1, 11) {
                    0..=5 => {
                        damage *= 3;
                        WeaponType::Sword
                    },
                    6..=7 => {
                        damage *= 4;
                        WeaponType::Axe
                    },
                    _ => WeaponType::Dagger,
                };
                
                let size = match RandomNumberGenerator::new().range(1, 11) {
                    0..=6 => Size::Small,
                    7..=8 => Size::Average,
                    _ => Size::Large,
                };

                if self.weight + Player::weight_by_weapon(weapon_type.clone(), material.clone()) + 
                Player::weight_by_size(size.clone()) <= self.max_weight 
                    && self.inventory.len()+2 <= 25 {
                        self.inventory.push(ItemType::Weapon(weapon_type.clone(), material.clone(), damage));
                        self.inventory.push(ItemType::Potion(size.clone(), ORANGE_RED));
                        self.weight += Player::weight_by_weapon(
                                weapon_type.clone(), 
                                material.clone()) + Player::weight_by_size(size.clone());
                        map.source[xy_idx(self.pos.0, self.pos.1, map.width)] = TileType::Floor;
                        format!("You pick up {} {} and {} {} potion from chest", 
                            material.to_string().to_lowercase(), 
                            weapon_type.to_string().to_lowercase(), 
                            size.to_string().to_lowercase(), 
                            Player::color_to_str(ORANGE_RED))
                } else {
                    format!("You can't carry more than {}kg and more than 25 items", self.max_weight)
                }
            }
            _ => String::new(),
        };
        if !matches!(map.source[xy_idx(self.pos.0, self.pos.1, map.width)], 
            TileType::BloodStain | TileType::BearTrapActived | TileType::Chest) {
            map.source[xy_idx(self.pos.0, self.pos.1, map.width)] = TileType::Floor;
        }
        message
    }

    fn try_move(&mut self, map: &mut WorldLocation, delta_x: i32, delta_y: i32) -> String {
        if self.blood > 0 {
            self.health -= 1+self.max_health/50;

            if map.source[xy_idx(self.pos.0, self.pos.1, map.width)] == TileType::Floor {
                map.source[xy_idx(self.pos.0, self.pos.1, map.width)] = TileType::BloodStain;
                map.colors[xy_idx(self.pos.0, self.pos.1, map.width)] = RED;
            }
            self.blood -= 1;
        }

        let x = self.pos.0 + delta_x;
        let y = self.pos.1 + delta_y;
        if !map.is_solid((x, y)) {
            self.pos.0 = x;
            self.pos.1 = y;
            self.visible_tiles =
                field_of_view(Point::new(self.pos.0, self.pos.1), self.fov_range, map);
            map.far = true;
            self.visible_tiles_far = field_of_view(
                Point::new(self.pos.0, self.pos.1), 
                    max(REAL_WIDTH-RIGHT_TABLE_WIDTH, REAL_HEIGHT-BOTTOM_TABLE_HEIGHT), 
                    map);
            map.far = false;

            return self.event(map);
        } else {
            return self.fight((x, y), map);
        }
    }
}
