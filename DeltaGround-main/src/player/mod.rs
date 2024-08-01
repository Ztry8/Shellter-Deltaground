use bracket_lib::{terminal::{BTerm, RGB, to_cp437, BLACK, Point, VirtualKeyCode, YELLOW}, prelude::field_of_view};
pub mod map;
use map::Map;

pub mod gui;
use self::map::TileType;

const POTION_POWER: u16 = 10;

#[derive(Clone)]
pub struct Player {
    pub pos: (i32, i32),
    pub health: u16,
    pub max_health: u16,
    visible_tiles: Vec<Point>,
    fov_range: i32,
    dbg: bool,
}

impl Player {
    pub fn new(x: i32, y: i32, fov_range: i32) -> Self {
        Self { 
            pos: (x, y),
            visible_tiles: vec![],
            fov_range: fov_range,
            dbg: false,
            health: 50,
            max_health: 50,
        }
    }

    pub fn action(&mut self, key: VirtualKeyCode, map: &mut Map) -> (u8, String) {
        let i: u8  = match key {
            VirtualKeyCode::Left | VirtualKeyCode::A => 
                self.try_move(map, -1, 0),

            VirtualKeyCode::Right | VirtualKeyCode::D => 
                self.try_move(map, 1, 0),

            VirtualKeyCode::Up | VirtualKeyCode::W => 
                self.try_move(map, 0, -1),

            VirtualKeyCode::Down | VirtualKeyCode::S => 
                self.try_move(map, 0, 1),
                
            VirtualKeyCode::F1 => { self.dbg = !self.dbg; 3 },
            VirtualKeyCode::F3 =>  self.try_move(map, 0, 0),
            _ => 0,
        };
        (i, String::new())
    }

    pub fn draw(&self, engine: &mut BTerm) {
        engine.set(self.pos.0, self.pos.1, 
            RGB::named(YELLOW), 
            RGB::named(BLACK), 
            to_cp437('@'));
    }

    pub fn is_visible(&self, (x, y): (i32, i32)) -> bool {
        self.visible_tiles.contains(&Point::new(x, y)) || self.dbg
    }

    fn try_move(&mut self, map: &mut Map, delta_x: i32, delta_y: i32) -> u8 {
        let x = self.pos.0 + delta_x;
        let y = self.pos.1 + delta_y;
        
        if map.source[Map::xy_idx(x, y)] == TileType::Exit {
            return 2
        }

        if !map.is_solid((x, y)) {
            if map.source[Map::xy_idx(x, y)] == TileType::HealthPotion {
                if self.health < self.max_health {
                    self.health += POTION_POWER;
                }
                if self.health > self.max_health {
                    self.health = self.max_health;
                }
                map.source[Map::xy_idx(x, y)] = TileType::Floor;
            }
            self.pos.0 = x;
            self.pos.1 = y;

            self.visible_tiles = field_of_view(Point::new(self.pos.0, self.pos.1), self.fov_range, map);
        }
        1
    }
}
