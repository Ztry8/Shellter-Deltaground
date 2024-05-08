#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    DamagedWall,
    WeakWall,
    Floor,
    Exit,
    Potion,
    Coin,
    BearTrap,
    BearTrapActived,
    BloodStain,
    Door,
    Chest,
}

const SOLID_TILES: [TileType; 4] = [
    TileType::Wall,
    TileType::DamagedWall,
    TileType::WeakWall,
    TileType::Door,
];

const TRANSPARENT_TILES: [TileType; 8] = [
    TileType::Floor,
    TileType::BearTrap,
    TileType::BearTrapActived,
    TileType::BloodStain,
    TileType::Potion,
    TileType::Exit,
    TileType::Coin,
    TileType::Chest,
];

use std::{cmp::{max, min},vec};

use bracket_lib::{color::{ALICE_BLUE, CYAN, GOLD, GREY100, HOT_PINK, LIGHT_BLUE, ORANGE_RED, RGB, ROSY_BROWN, TOMATO, WHITE, WHITESMOKE}, pathfinding::{a_star_search, Algorithm2D, BaseMap, SmallVec}, random::RandomNumberGenerator, terminal::{BTerm, Point, GRAY, GRAY100, GREEN3, PINK, SANDY_BROWN, YELLOW3, YELLOWGREEN}};
use bracket_lib::terminal::DistanceAlg::Pythagoras;

use crate::{entity::EntityType, player::Player, State};
use super::entity::Entity;

pub fn xy_idx(x: i32, y: i32, width: i32) -> usize {
    (y as usize * width as usize) + x as usize
}

#[derive(Clone)]
pub struct WorldLocation {
    pub source: Vec<TileType>,
    pub entities: Vec<Entity>,
    pub colors: Vec<(u8, u8, u8)>,
    pub width: i32,
    pub height: i32,
    pub start_x: i32,
    pub start_y: i32,
    pub far: bool,
}

const MAX_ROOMS: i32 = 100000;
const MIN_SIZE: i32 = 6;
const MAX_SIZE: i32 = 15;

impl WorldLocation {
    pub fn new(engine: &mut BTerm, width: i32, height: i32, exp: i32, floor: i32) -> Self { 
        let mut map = vec![TileType::Floor; (width * height) as usize];
        let mut color = vec![GRAY; (width * height) as usize];
        let mut entities: Vec<Entity> = vec![];

        let mut rng = RandomNumberGenerator::new();
        for i in 0..map.len() {
            map[i] = match rng.roll_dice(2, 6) {
                (1..=7) => TileType::Wall,
                8 | 9 => TileType::DamagedWall,
                _ => TileType::WeakWall,
            };
        }

        let colors: ((u8, u8, u8), (u8, u8, u8), (u8, u8, u8), (u8, u8, u8)) = match floor {
            0..=5 => (SANDY_BROWN, ROSY_BROWN, GRAY100, GRAY100),
            6..=8 => (YELLOWGREEN, YELLOW3, GREEN3, PINK),
            _ => {
                if rng.range(1, 101) <= 50 {
                    (GREY100, GRAY, ORANGE_RED, TOMATO)
                } else {
                    (LIGHT_BLUE, ALICE_BLUE, CYAN, GREY100)
                }
            } 
        };
        engine.screen_burn_color(RGB::from_u8(colors.0.0, colors.0.1, 
            colors.0.1));

        for i in 0..color.len() {
            color[i] = match rng.roll_dice(2, 6) {
                (1..=6) => colors.0,
                (7..=9) => colors.1,
                10 => colors.2,
                _ => colors.3,
            };
        }


        let mut rooms: Vec<Room> = Vec::new();
        for i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, width - w - 1) - 1;
            let y = rng.roll_dice(1, height - h - 1) - 1;
            let new_room = Room::new(x, y, w, h);
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false;
                }
            }
            if ok {
                WorldLocation::apply_room_to_map(&new_room, &mut map, width);

                if !rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                    if rng.range(1, 3) == 1 {
                        WorldLocation::apply_horizontal_tunnel(
                            &mut map,
                            &mut color,
                            prev_x,
                            new_x,
                            prev_y,
                            width,
                            height,
                            &mut rng,
                        );
                        WorldLocation::apply_vertical_tunnel(
                            &mut map,
                            &mut color,
                            prev_y,
                            new_y,
                            new_x,
                            width,
                            height,
                            &mut rng,
                        );
                    } else {
                        WorldLocation::apply_vertical_tunnel(
                            &mut map,
                            &mut color,
                            prev_y,
                            new_y,
                            prev_x,
                            width,
                            height,
                            &mut rng,
                        );
                        WorldLocation::apply_horizontal_tunnel(
                            &mut map,
                            &mut color,
                            prev_x,
                            new_x,
                            new_y,
                            width,
                            height,
                            &mut rng,
                        );
                    }
                }
                WorldLocation::generate_at_room(&mut map, &mut color, 
                    (new_room.x1, new_room.y1),
                    (new_room.x2, new_room.y2),
                    width,
                    &mut rng, floor);
                rooms.push(new_room);

                let t = match colors.0 {
                    GRAY100 => {
                        if rng.range(1, 101) <= 50 {
                            EntityType::Cyclops
                        } else {
                            EntityType::Orc
                        }
                    }
                    SANDY_BROWN => {
                        if rng.range(1, 101) <= 50 {
                            EntityType::Skeleton
                        } else {
                            EntityType::Zombie
                        }
                    }
                    _ => EntityType::Goblin,
                };

                if rng.range(1, 101) <= floor+35 {
                    if i > 1 {
                        entities.push(Entity::new(new_room.center().0, new_room.center().1, t, exp));
                    }
                }
            }
        }
        for i in &entities {
            map[xy_idx(i.x, i.y, width)] = TileType::Floor;
        }

        
        for i in 0..map.len() {
            let x = i as i32 % width;
            let y = i as i32 / width;

            if map[i] == TileType::Door {
                if !((SOLID_TILES.contains(&map[xy_idx(x+1, y, width)]) && 
                SOLID_TILES.contains(&map[xy_idx(x-1, y, width)]))
                || (SOLID_TILES.contains(&map[xy_idx(x, y+1, width)]) && 
                    SOLID_TILES.contains(&map[xy_idx(x, y-1, width)]))) {
                        map[i] = TileType::Floor;
                }
            }
        }
        
        map[xy_idx(rooms[0].center().0, rooms[0].center().1, width)] = TileType::Floor;
        color[xy_idx(rooms[1].center().0, rooms[1].center().1, width)] = WHITE;
        map[xy_idx(rooms[1].center().0, rooms[1].center().1, width)] = TileType::Exit;
        let mut result = Self {
            source: map.clone(),
            entities: entities.clone(),
            colors: color.clone(),
            width,
            height,
            start_x: rooms[0].center().0, 
            start_y: rooms[0].center().1,
            far: false,
        };

        let path = a_star_search(
            xy_idx(rooms[0].center().0, rooms[0].center().1, width),
            xy_idx(rooms[1].center().0, rooms[1].center().1, width),
            &result,
        );


        if path.steps.len() <= 50  {
            result = WorldLocation::new(engine, width, height, exp, floor);
        }
        result
    }

    pub fn is_solid(&self, (x, y): (i32, i32)) -> bool {
        for i in 0..self.entities.len() {
            if self.entities[i].x == x && self.entities[i].y == y && self.entities[i].health > 0 {
                return true;
            }
        }
        SOLID_TILES.contains(&self.source[xy_idx(x, y, self.width)])
    }

    fn apply_room_to_map(room: &Room, map: &mut [TileType], width: i32) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                map[xy_idx(x, y, width)] = TileType::Floor;
            }
        }
    }

    fn generate_at_room(map: &mut [TileType], colors: &mut [(u8, u8, u8)], first: (i32, i32), second: (i32, i32), 
        width: i32, rng: &mut RandomNumberGenerator, floor: i32) {
        let mut potion = false;

        for _i in 0..(((second.1-first.1)*(second.0-first.0))/30+5) {
            let (x, y) = (rng.range(first.0+2, second.0-2), rng.range(first.1+2, second.1-2));
            let idx = xy_idx(x, y, width);

            let chance = rng.range(1, 101);
            if chance <= 15+floor && !potion {
                if rng.range(1, 101) >= 85 {
                    colors[idx] = HOT_PINK;
                    map[idx] = TileType::Potion;
                } else {
                    colors[idx] = ORANGE_RED;
                    map[idx] = TileType::Potion;
                }
                potion = true;
            } else if chance == 16+floor {
                colors[idx] = GOLD;
                map[idx] = TileType::Coin;
            } else if chance >= 98 {
                colors[idx] = WHITESMOKE;
                map[idx] = TileType::Chest;
            }
        }
    }

    fn generate_tunnel(map: &mut [TileType], colors: &mut [(u8, u8, u8)], width: i32, height: i32, x: i32, y: i32,
        rng: &mut RandomNumberGenerator, ok: bool) -> bool {
        let idx = xy_idx(x, y, width);
        if idx > 0 && idx < (width * height) as usize {
            let chance = rng.range(1, 101);
            if chance <= 2 && !ok {
                map[idx as usize] = TileType::BearTrap;
                colors[idx as usize] = WHITESMOKE;
                return true;
            } else if chance <= 7 && (
                (SOLID_TILES.contains(&map[xy_idx(x+1, y, width)]) && 
                SOLID_TILES.contains(&map[xy_idx(x-1, y, width)]))
                || (SOLID_TILES.contains(&map[xy_idx(x, y+1, width)]) && 
                    SOLID_TILES.contains(&map[xy_idx(x, y-1, width)]))) {
                colors[idx as usize] = WHITESMOKE;
                map[idx as usize] = TileType::Door;
            } else {
                map[idx as usize] = TileType::Floor;
            }
        }
        false
    }

    fn apply_horizontal_tunnel(map: &mut [TileType], colors: &mut [(u8, u8, u8)], x1: i32, x2: i32, y: i32, 
        width: i32, height: i32, rng: &mut RandomNumberGenerator) {
        let mut have = false;
        for x in min(x1, x2)..=max(x1, x2) {
            have = WorldLocation::generate_tunnel(map, colors, 
                width, height, x, y, rng, have)
        }
    }

    fn apply_vertical_tunnel(map: &mut [TileType], colors: &mut [(u8, u8, u8)], y1: i32, y2: i32, x: i32, 
        width: i32, height: i32, rng: &mut RandomNumberGenerator) {
        let mut have = false;
        for y in min(y1, y2)..=max(y1, y2) {
            have = WorldLocation::generate_tunnel(map, colors, 
                width, height, x, y, rng, have)
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if !(1..=self.width - 1).contains(&x) || !(1..=self.height - 1).contains(&y) {
            return false;
        }
        !self.is_solid((x, y)) && self.source[xy_idx(x, y, self.width)] != TileType::BearTrap
    }

    pub fn draw(&mut self, player: &mut Player, engine: &mut BTerm) -> Vec<String> {
        for tile in &player.visible_tiles_far {
            let symbol = match self.source[xy_idx(tile.x, tile.y, self.width)] {
                TileType::Potion => '!',
                TileType::Floor | TileType::BearTrap => ' ',
                TileType::Wall => '#',
                TileType::DamagedWall => 'M',
                TileType::WeakWall => 'W',
                TileType::Exit => '<',
                TileType::Coin => '$',
                TileType::BearTrapActived => '^',
                TileType::BloodStain => '.',
                TileType::Door => '+',
                TileType::Chest => '*',
            };

            if player.pos != (tile.x, tile.y) {    
                if !player.visible_tiles.contains(tile) {
                    if matches!(symbol, '#' | 'W' | 'M') {
                        State::put(engine, player.pos, tile.x, tile.y, 
                            GRAY, symbol);
                    }
                } else {
                    if symbol != '@' {
                        State::put(engine, player.pos, tile.x, tile.y, 
                            self.colors[xy_idx(tile.x, tile.y, self.width)], symbol);
                    } else {
                        State::put_red(engine, player.pos, tile.x, tile.y, 
                            self.colors[xy_idx(tile.x, tile.y, self.width)], symbol);
                    }
                } 
            }
        }

        let mut m = self.clone();
        let mut messages: Vec<String> = vec![];
        for entity in &mut self.entities {
            if player.visible_tiles.contains(&Point::new(entity.x, entity.y)) {
                let event = entity.update(player, &mut m, engine);
                if !event.is_empty() {
                    messages.push(event);
                }
            }
        }
        messages
    }
}

impl BaseMap for WorldLocation {
    fn is_opaque(&self, idx: usize) -> bool {
        !(TRANSPARENT_TILES.contains(&self.source[idx]) || (self.far && &self.source[idx] == &TileType::Door))
    }
    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        Pythagoras.distance2d(p1, p2)
    }
    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0)) };

        exits
    }
}

impl Algorithm2D for WorldLocation {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

#[derive(Clone, Copy)]
pub struct Room {
    pub x1: i32,
    pub x2: i32,
    pub y1: i32,
    pub y2: i32,
}

impl Room {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    pub fn intersect(&self, other: &Self) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}
