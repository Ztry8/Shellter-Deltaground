use bracket_lib::{
    terminal::{BTerm, RGB, to_cp437, BLACK, GRAY, GREEN3,  GREENYELLOW, YELLOWGREEN, TOMATO, YELLOW3, GREY100, ORANGE_RED, SANDY_BROWN, ROSY_BROWN, GRAY100, LIGHT_BLUE, ALICE_BLUE, CYAN, PINK, GREEN4, WHITE, ORANGE, },
    random::RandomNumberGenerator,
    pathfinding::{DistanceAlg::Pythagoras, SmallVec, a_star_search},
    prelude::{Algorithm2D, Point, BaseMap}
};

use std::{cmp::{max, min}, vec};

use crate::entity::{Entity, EntityType};
use super::Player;

pub const WIDTH: i32 = 64;
pub const HEIGHT: i32 = 26;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, DamagedWall, WeakWall, Floor, Tree1, Tree2, Exit, Column, Scarecrow, HealthPotion
}

const SOLID_TILES: [TileType; 7] = [
    TileType::Tree1, 
    TileType::Tree2,
    TileType::Wall,
    TileType::DamagedWall,
    TileType::WeakWall,
    TileType::Column,
    TileType::Scarecrow,
];

const TRANSPARENT_TILES: [TileType; 7] = [
    TileType::Tree1,
    TileType::Tree2,
    TileType::Floor,
    TileType::Exit,
    TileType::Column,
    TileType::Scarecrow,
    TileType::HealthPotion
];
 
#[derive(PartialEq)]
pub enum MapType {
    Dungeon, Forest,
}

#[derive(Clone)]
pub struct Map {
    pub source: Vec<TileType>,
    pub entities: Vec<Entity>,
    pub colors: Vec<(u8, u8, u8)>,
    pub start_pos: (i32, i32),
}

impl Map {
    pub fn xy_idx(x: i32, y: i32) -> usize {
        (y as usize * WIDTH as usize) + x as usize
    }

    pub fn new(engine: &mut BTerm, t: MapType, mut seed: u64, floor: u16) -> Self {
        let mut map = vec![TileType::Floor; (WIDTH * HEIGHT) as usize];
        let mut color = vec![GRAY; (WIDTH * HEIGHT) as usize];
        let mut entities: Vec<Entity> = vec![];

        let mut rng = RandomNumberGenerator::new();
        if seed == 0 {
            seed = rng.range(1, u64::MAX)
        }

        println!("World generated with seed: {}", seed);
        rng = RandomNumberGenerator::seeded(seed);
                
        match t {
            MapType::Dungeon => {
                for i in 0..map.len() {
                    let chance = rng.roll_dice(2, 6);  
                    
                    if chance <= 7 {
                        map[i] = TileType::WeakWall; 
                    }
                    else if chance == 8 || chance == 9 {
                        map[i] = TileType::DamagedWall; 
                    }
                    else {
                        map[i] = TileType::Wall; 
                    }
                } 

                let main_color ;
                let second_color ;
                let third_color ;
                let fourth_color;
                
                if floor <= 2 {
                    main_color = SANDY_BROWN;
                    second_color = ROSY_BROWN;
                    third_color = GRAY100;
                    fourth_color = third_color;
                }
                else if floor <= 4  {
                    main_color = GREY100;
                    second_color = GRAY;
                    third_color = ORANGE_RED;
                    fourth_color = TOMATO;
                }
                else {
                    if rng.range(1, 3) == 1 {
                        main_color = LIGHT_BLUE;
                        second_color = ALICE_BLUE;
                        third_color = CYAN;
                        fourth_color = GREY100;
                    }
                    else {
                        main_color = YELLOWGREEN;
                        second_color = YELLOW3;
                        third_color = GREEN3;
                        fourth_color = PINK;
                    }
                }
                engine.screen_burn_color(RGB::named(main_color));

                for i in 0..color.len() {
                    let chance = rng.roll_dice(2, 6);  
                            
                    if chance <= 6 {
                        color[i] = main_color; }
                    else if chance >= 7 && chance <= 9 {
                        color[i] = second_color; }
                    else if chance > 9 && chance < 11 { 
                        color[i] = third_color; 
                    }
                    else {
                        color[i] = fourth_color;
                    }
                }

                let mut rooms : Vec<Rectangle> = Vec::new();
                
                let max_rooms = rng.range(60, 101);
                let min_size = max_rooms / 10;
                let max_size = max_rooms / 6;

                let mut i = 0;
                for _ in 0..max_rooms {
                    let w = rng.range(min_size, max_size);
                    let h = rng.range(min_size, max_size);
                    let x = rng.roll_dice(1, WIDTH - w - 1) - 1;
                    let y = rng.roll_dice(1, HEIGHT - h - 1) - 1;
                    let new_room = Rectangle::new(x, y, w, h);
                    let mut ok = true;
                    for other_room in rooms.iter() {
                        if new_room.intersect(other_room) { ok = false }
                    }
                    if ok {
                        Map::apply_room_to_map(&new_room, &mut map);

                        if !rooms.is_empty() {
                            let (new_x, new_y) = new_room.center();
                            let (prev_x, prev_y) = rooms[rooms.len()-1].center();
                            if rng.range(1, 3) == 1 {
                                Map::apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y, floor, rng.clone());
                                Map::apply_vertical_tunnel(&mut map, prev_y, new_y, new_x, floor, rng.clone());
                            } 
                            else {
                                Map::apply_vertical_tunnel(&mut map, prev_y, new_y, prev_x, floor, rng.clone());
                                Map::apply_horizontal_tunnel(&mut map, prev_x, new_x, new_y, floor, rng.clone());
                            }
                        }
                        rooms.push(new_room); 
                         
                        let t = match main_color {
                            SANDY_BROWN => EntityType::Human,
                            GRAY100 => EntityType::Orc,
                            YELLOWGREEN => EntityType::Zombie,
                            LIGHT_BLUE => EntityType::FrozenTroll, 
                            _ => EntityType::Human,
                        };

                        if i != 0 && i != 1 {
                            if rng.range(1, 101) <= 35 + (5 * (floor - 1)) {
                                entities.push(Entity::new(
                                    new_room.center().0, 
                                    new_room.center().1, 
                                    t));  
                            }   
                        }

                        i += 1;
                    }
                }

                for i in &entities {
                    map[Map::xy_idx(i.x, i.y)] = TileType::Floor;
                }

                map[Map::xy_idx(rooms[1].center().0, rooms[1].center().1)] = TileType::Exit;
                map[Map::xy_idx(rooms[0].center().0, rooms[0].center().1)] = TileType::Floor;

                let mut result = Self {
                    source: map,
                    start_pos: rooms[0].center(),
                    colors: color,
                    entities: entities,
                };

                let path = a_star_search(
                    Map::xy_idx(rooms[0].center().0, rooms[0].center().1), 
                    Map::xy_idx(rooms[1].center().0, rooms[1].center().1), 
                    &mut result);
                
                if path.steps.len() <= 1 {
                    result = Map::new(engine, t, RandomNumberGenerator::new().range(1, u64::MAX), floor);
                }

                result
            }
            MapType::Forest => {    
                // 1200 max, 0 min
                let amount_green: u32 = rng.range(350, 1001);

                for x in 0..WIDTH {
                    map[Map::xy_idx(x, 0)] = TileType::Tree1;
                    map[Map::xy_idx(x, HEIGHT-1)] = TileType::Tree1;
                }
                for y in 0..HEIGHT {
                    map[Map::xy_idx(0, y)] = TileType::Tree1;
                    map[Map::xy_idx(WIDTH-1, y)] = TileType::Tree1;
                }

                for _i in 0..amount_green {
                    let x = rng.roll_dice(1, WIDTH-1);
                    let y = rng.roll_dice(1, HEIGHT-1);
                    let idx = Map::xy_idx(x, y);
                    if idx != Map::xy_idx(WIDTH/2, HEIGHT/2) {
                        if rng.range(1, 3) == 1 {
                            map[idx] = TileType::Tree1; 
                        }
                        else {
                            map[idx] = TileType::Tree2; 
                        }
                    }
                }

                for i in 0..color.len() {
                    let chance = rng.roll_dice(2, 6);  
                            
                    if map[i] != TileType::Floor {
                        if chance <= 7 {
                            color[i] = GREEN4; 
                        }
                        else if chance == 8 || chance == 9 {
                            color[i] = GREEN3 
                        }
                        else {
                            color[i] = GREENYELLOW; 
                        }
                    }
                }

                map[Map::xy_idx(WIDTH/2, HEIGHT/2)] = TileType::Floor;
                return Self {
                    source: map, 
                    start_pos: (WIDTH/2, HEIGHT/2),
                    colors: color,
                    entities: vec![],
                }
            }
        }
    }

    pub fn draw(&mut self, player: &mut Player, is_tick: bool, engine: &mut BTerm) -> String {
        let mut y = 0;
        let mut x = 0;
        let mut symbol ;

        for tile in self.source.iter() {
            if player.is_visible((x, y)) {        
                symbol = match tile {
                    TileType::Floor => '.',                
                    TileType::Wall => '#',
                    TileType::DamagedWall => 'M',
                    TileType::WeakWall => 'W',
                    TileType::Tree1 => '♣',
                    TileType::Tree2 => '♠',
                    TileType::HealthPotion => { self.colors[Map::xy_idx(x, y)] = CYAN; '¡' },
                    TileType::Exit => { self.colors[Map::xy_idx(x, y)] = WHITE; 'H' },
                    TileType::Scarecrow => { self.colors[Map::xy_idx(x, y)] = ORANGE; '♀' },
                    TileType::Column => { self.colors[Map::xy_idx(x, y)] = GRAY; 'I' },
                };

                engine.set(x, y, 
                    RGB::named(self.colors[Map::xy_idx(x, y)]), 
                    RGB::named(BLACK), 
                    to_cp437(symbol));
            }

            x += 1;
            if x > WIDTH-1 {
                x = 0;
                y += 1;
            }
        }

        let m =  &mut Map {
            source: self.source.clone(),
            entities: self.entities.clone(),
            colors: self.colors.clone(),
            start_pos: self.start_pos,
        };
        let mut str = String::new();
        for entity in &mut self.entities {
            if player.is_visible((entity.x, entity.y)) {      
                str = entity.draw_and_tick(player, m, is_tick, engine);
            }
        }
        str
    }

    pub fn is_solid(&self, (x, y): (i32, i32)) -> bool {
        for i in 0..self.entities.len() {
            if self.entities[i].x == x && self.entities[i].y == y {
                return true;
            }
        }
        SOLID_TILES.contains(&self.source[Map::xy_idx(x, y)]) 
    }

    fn apply_room_to_map(room: &Rectangle, map: &mut [TileType]) {
        for y in room.y1 +1 ..= room.y2 {
            for x in room.x1 + 1 ..= room.x2 {
                map[Map::xy_idx(x, y)] = TileType::Floor;
            }
        }
    }

    fn generate(map: &mut [TileType], idx: usize, floor: u16, rng: &mut RandomNumberGenerator) {
        let chance = rng.range(1, 101);
        if chance <= 40 {
            if rng.range(1, 6) <= 3 {
                map[idx as usize] = TileType::Column;
            }
            else {
                map[idx as usize] = TileType::Scarecrow;
            }
        }
        else if chance <= 60 - floor + 1 {
            map[idx as usize] = TileType::HealthPotion;
        }
    }

    fn apply_horizontal_tunnel(map: &mut [TileType], x1: i32, x2: i32, y: i32, floor: u16, 
        mut rng: RandomNumberGenerator) {
        let mut no_exists: bool = true;
        for x in min(x1, x2) ..= max(x1, x2) {
            let idx = Map::xy_idx(x, y);
            if idx > 0 && idx < (WIDTH*HEIGHT) as usize {
                if rng.range(1, 101) <= 40  && no_exists {
                    Map::generate(map, idx, floor, &mut rng);
                    no_exists = false;
                }   
                else {
                    map[idx as usize] = TileType::Floor;
                }
            }
        }
    }
    
    fn apply_vertical_tunnel(map: &mut [TileType], y1: i32, y2: i32, x: i32, floor: u16,
        mut rng: RandomNumberGenerator) {
        let mut no_exists: bool = true;
        for y in min(y1,y2) ..= max(y1,y2) {
            let idx = Map::xy_idx(x, y);
            if idx > 0 && idx < (WIDTH*HEIGHT) as usize {
                if rng.range(1, 101) <= 40  && no_exists {
                    Map::generate(map, idx, floor, &mut rng);
                    no_exists = false;
                }
                else {
                    map[idx as usize] = TileType::Floor;
                }
            }
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > WIDTH - 1 || y < 1 || y > HEIGHT - 1 { 
            return false; 
        }
        !self.is_solid((x, y))
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:usize) -> bool {
        !TRANSPARENT_TILES.contains(&self.source[idx as usize])
    }
    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = WIDTH as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        Pythagoras.distance2d(p1, p2)
    }
    fn get_available_exits(&self, idx:usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = idx as i32 % WIDTH;
        let y = idx as i32 / WIDTH;
        let w = WIDTH as usize;
    
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0)) };
    
        exits
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(WIDTH, HEIGHT)
    }   
}

#[derive(Clone, Copy)]
pub struct Rectangle {
    pub x1: i32,
    pub x2: i32,
    pub y1: i32,
    pub y2: i32,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {x1: x, y1: y, x2: x + w, y2: y + h}
    }

    pub fn intersect(&self, other: &Self) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2)/2, (self.y1 + self.y2)/2)
    }
}