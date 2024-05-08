use bracket_lib::{color::{BLACK, CYAN, GREEN, PURPLE, RED, RGB, WHITESMOKE, YELLOWGREEN}, terminal::{letter_to_option, main_loop, to_cp437, BTerm, BTermBuilder, GameState, Point, VirtualKeyCode}};
use chrono::Utc;
use std::{fs, process, time::Instant};

pub mod player;
pub mod location;
pub mod entity;

use location::{WorldLocation, TileType, xy_idx};
use player::{
    gui::{self, draw_ui, BOTTOM_TABLE_HEIGHT, RIGHT_TABLE_WIDTH},
    Player,
};

const REAL_WIDTH: i32 = 80;
const REAL_HEIGHT: i32 = 45;

#[derive(Clone)]
struct State {
    root_map: WorldLocation,
    player: Player,
    moves: u128,
    messages: Vec<String>,
    timer: Instant,
    total_time: Instant,
    final_time: u64,
    game_start: bool,
    floor: i32,
    luck: i32, 
    strength: i32, 
    intelligence: i32, 
    agility: i32,
    points: i32,
    dir: i32,
    inventory: bool, 
    wheel: i32,
}

impl GameState for State {
    fn tick(&mut self, engine: &mut BTerm) {
        let input = self.input(engine);
        if !self.game_start {
            engine.cls();
            engine.print_color_centered(3, YELLOWGREEN, BLACK, "It would be an extremely dangerous adventure,");
            engine.print_color_centered(4, YELLOWGREEN, BLACK, "that hundreds of already dead adventurers have agreed to...");
            
            engine.print_color_centered(6, YELLOWGREEN, BLACK, "The dark forces of evil have already spread to the entire dungeon,");
            engine.print_color_centered(7, YELLOWGREEN, BLACK, "but it is still possible to save the legendary treasures in the dungeon...");
            
            engine.print_color_centered(9, WHITESMOKE, BLACK, "Who are you?");
            engine.print_color_centered(10, WHITESMOKE, BLACK, "Press the space bar when you're ready");
            engine.print_color_centered(12, WHITESMOKE, BLACK, "Use Up/Down to change skill,");
            engine.print_color_centered(13, WHITESMOKE, BLACK, "Right/Left keys to increase/decrease skill value");

            engine.print_centered(15, format!("Available skill points: {}", self.points));

            if self.dir == 0 {
                engine.print_color_centered(16, GREEN, BLACK, format!("=> Luck: {}/5", self.luck));
                engine.print_color_centered(17, RED, BLACK, format!("Strength: {}/5", self.strength));
                engine.print_color_centered(18, CYAN, BLACK, format!("Intelligence: {}/5", self.intelligence));
                engine.print_color_centered(19, PURPLE, BLACK, format!("Agility: {}/5", self.agility));
            } else if self.dir == 1 {
                engine.print_color_centered(16, GREEN, BLACK, format!("Luck: {}/5", self.luck));
                engine.print_color_centered(17, RED, BLACK, format!("=> Strength: {}/5", self.strength));
                engine.print_color_centered(18, CYAN, BLACK, format!("Intelligence: {}/5", self.intelligence));
                engine.print_color_centered(19, PURPLE, BLACK, format!("Agility: {}/5", self.agility));
            } else if self.dir == 2 {
                engine.print_color_centered(16, GREEN, BLACK, format!("Luck: {}/5", self.luck));
                engine.print_color_centered(17, RED, BLACK, format!("Strength: {}/5", self.strength));
                engine.print_color_centered(18, CYAN, BLACK, format!("=> Intelligence: {}/5", self.intelligence));
                engine.print_color_centered(19, PURPLE, BLACK, format!("Agility: {}/5", self.agility));
            } else if self.dir == 3 {
                engine.print_color_centered(16, GREEN, BLACK, format!("Luck: {}/5", self.luck));
                engine.print_color_centered(17, RED, BLACK, format!("Strength: {}/5", self.strength));
                engine.print_color_centered(18, CYAN, BLACK, format!("Intelligence: {}/5", self.intelligence));
                engine.print_color_centered(19, PURPLE, BLACK, format!("=> Agility: {}/5", self.agility));
            }
        } else {
            if self.player.health <= 0 {
                self.game_over(engine);
            } else {
                if !input.1.is_empty() {
                    let ss = input.1.clone();
                    self.messages.push(input.1);
                    
                    if ss == String::from("You go down to the floor below...") {
                        self.root_map = WorldLocation::new(engine, self.root_map.width*3/2, 
                            self.root_map.height*3/2, self.player.exp, self.floor);
                        self.player.pos = (self.root_map.start_x, self.root_map.start_y);
                        self.restart(engine);
                        self.floor += 1;
                    }
                }
                if input.0 {
                    let timer = Instant::now();
                    self.moves += 1;
                    println!("Moves made: {}", self.moves);
            
                    engine.cls();
                    let mut messages = self.root_map.draw(&mut self.player, engine);
                    self.player.draw(engine);
                    if !messages.is_empty() {
                        self.messages.append(&mut messages);
                    }
                        
                    println!("The move is made in {} seconds", timer.elapsed().as_secs_f32());
                }

                self.what_is_it(engine);
                draw_ui(&self.messages, self.player.clone(), engine);
            }
        }
    }
}

impl State {
    pub fn new(engine: &mut BTerm) -> Self {
        fs::create_dir_all("./screenshots").unwrap();
        let map = WorldLocation::new(engine, 50, 50, 0, 1);
        Self {
            root_map: map.clone(),
            game_start: false,
            player: Player::new(map.start_x, map.start_y, 10, 0, 0, 0, 0),
            moves: 0,
            messages: vec![],
            timer: Instant::now(),
            total_time: Instant::now(),
            final_time: 0,
            floor: 1,
            points: 10,
            luck: 0, 
            strength: 0,
            intelligence: 0, 
            agility: 0,
            dir: 0,
            wheel: 0,
            inventory: false,
        }
    }

    pub fn put(engine: &mut BTerm, player_pos: (i32, i32), x: i32, y: i32, color: (u8, u8, u8), symbol: char) {
        let xy = (x + (REAL_WIDTH - RIGHT_TABLE_WIDTH) / 2 - player_pos.0, 
        y + (REAL_HEIGHT - BOTTOM_TABLE_HEIGHT) / 2 - player_pos.1);
  
        engine.set(
            xy.0,
            xy.1,
            RGB::named(color),
            RGB::named(BLACK),
            to_cp437(symbol),
        );   
    }

    pub fn put_red(engine: &mut BTerm, player_pos: (i32, i32), x: i32, y: i32, color: (u8, u8, u8), symbol: char) {
        let xy = (x + (REAL_WIDTH - RIGHT_TABLE_WIDTH) / 2 - player_pos.0, 
        y + (REAL_HEIGHT - BOTTOM_TABLE_HEIGHT) / 2 - player_pos.1);
  
        engine.set(
            xy.0,
            xy.1,
            RGB::named(color),
            RGB::named(RED),
            to_cp437(symbol),
        );   
    }

    fn input(&mut self, engine: &mut BTerm) -> (bool, String) {
        let mut str = String::new();
        let mut ok = false;
        match engine.key {
            Some(key) => match key {
                VirtualKeyCode::Escape => process::exit(0),
                VirtualKeyCode::I => {
                    if !self.inventory && self.player.health >= 0 {
                        gui::draw_inventory(engine, &mut self.player, self.wheel)
                    } else {
                        self.wheel = 0;
                        self.restart(engine);
                    }
                    self.inventory = !self.inventory;
                },
                VirtualKeyCode::F2 => engine.screenshot(format!("screenshots/screen{}.png", Utc::now().timestamp())),
                _ => { 
                    if self.player.health >= 0 && self.game_start && !self.inventory {
                        ok = true;
                        str = self.player.action(key, &mut self.root_map);
                    } else {
                        match key {
                            VirtualKeyCode::Space => {
                                if !self.game_start {
                                    self.player = Player::new(
                                        self.root_map.start_x, 
                                        self.root_map.start_y, 
                                        self.player.fov_range, 
                                        self.luck,
                                        self.strength, 
                                        self.intelligence, 
                                        self.agility);
                                    self.game_start = true;
                                    self.restart(engine);
                                }
                            }
                            VirtualKeyCode::Up => {
                                self.dir -= 1;
                                if self.dir < 0 {
                                    self.dir = 0;
                                }
                            }
                            VirtualKeyCode::Down => {
                                self.dir += 1;
                                if self.dir > 3 {
                                    self.dir = 3;
                                }
                            }
                            VirtualKeyCode::Right => {
                                match self.dir {
                                    0  => {
                                        if self.luck < 5 && self.points > 0 {
                                            self.luck += 1;
                                            self.points -= 1;
                                        }
                                    },
                                    1 => {
                                        if self.strength < 5 && self.points > 0  {
                                            self.strength += 1;
                                            self.points -= 1;
                                        }
                                    },
                                    2 => {
                                        if self.intelligence < 5 && self.points > 0  {
                                            self.intelligence += 1;
                                            self.points -= 1;
                                        }
                                    },
                                    3 => {
                                        if self.agility < 5 && self.points > 0  {
                                            self.agility += 1;
                                            self.points -= 1;
                                        }
                                    },
                                    _ => {}, 
                                };
                            }
                            VirtualKeyCode::Left => {
                                match self.dir {
                                    0  => {
                                        if self.luck > 0 {
                                            self.luck -= 1;
                                            self.points += 1;
                                        }
                                    },
                                    1 => {
                                        if self.strength > 0 {
                                            self.strength -= 1;
                                            self.points += 1;
                                        }
                                    },
                                    2 => {
                                        if self.intelligence > 0 {
                                            self.intelligence -= 1;
                                            self.points += 1;
                                        }
                                    },
                                    3 => {
                                        if self.agility > 0 {
                                            self.agility -= 1;
                                            self.points += 1;
                                        }
                                    },
                                    _ => {}, 
                                };
                            }
                            _ => str = {
                                if key == VirtualKeyCode::PageUp && self.wheel < 0 { 
                                    self.wheel += 1;
                                } else if key == VirtualKeyCode::PageDown {
                                        self.wheel -= 1;
                                }

                                let mut j = letter_to_option(key);
                                if j > 8 {
                                    j -= 1;
                                }
                                let s = self.player.use_inventory(j); 
                                gui::draw_inventory(engine, &mut self.player, self.wheel);
                                s
                            }
                        }
                    }
                },
            },
            None => {}
        };
        (ok, str)
    }

    fn restart(&mut self, engine: &mut BTerm) {
        self.player.action(VirtualKeyCode::F3, &mut self.root_map);
        engine.cls();
        self.player.draw(engine);
        self.root_map.draw(&mut self.player, engine);  
        gui::draw_ui(&self.messages, self.player.clone(), engine);
    }

    fn what_is_it(&mut self, engine: &mut BTerm) {
        let mut message = String::new();
        if engine.left_click && self.timer.elapsed().as_secs_f32() >= 0.15 {
            let mouse_pos = engine.mouse_pos();
            let xy = (mouse_pos.0 + self.player.pos.0 - (REAL_WIDTH - RIGHT_TABLE_WIDTH) / 2, 
                mouse_pos.1 + self.player.pos.1 - (REAL_HEIGHT - BOTTOM_TABLE_HEIGHT) / 2);
                
            if self.player.visible_tiles.contains(&Point::new(xy.0, xy.1)) {
                message = match self.root_map.source[xy_idx(xy.0, xy.1, self.root_map.width)] {
                    TileType::Wall => String::from("It's wall"),
                    TileType::Door => String::from("It's door"),
                    TileType::DamagedWall => String::from("It's damaged wall"),
                    TileType::WeakWall => String::from("It's week wall"),
                    TileType::Exit => String::from("It's ladder to the next dungeon"),
                    TileType::Coin => String::from("It's pile of old coins"),
                    TileType::Potion => String::from("It's potion"),
                    TileType::BloodStain => String::from("These are bloodstains on the floor"),
                    TileType::BearTrapActived => String::from("It's activated trap"),
                    TileType::Chest => String::from("It's chest"),
                    TileType::Floor | TileType::BearTrap => {
                        let mut str = String::new();
                        for entity in &self.root_map.entities {
                            let ss;
                            if entity.health > 0 {
                                ss = format!("{}: {}HP; {} Damage", entity.name().clone(), entity.health.clone(),
                                    entity.damage.clone());
                            } else {
                                ss = format!("dead {}", entity.name().clone())
                            }

                            for i in -1..2 {
                                if (entity.x+i, entity.y) == xy {
                                    str = format!("It's {}", ss)
                                }
                            }
                            for i in -1..2 {
                                if (entity.x, entity.y+i) == xy {
                                    str = format!("It's {}", ss)
                                }
                            }
                        }
                        if self.player.pos == xy {
                            str = String::from("It's you")
                        }
                        str
                    }
                };
            }
            self.timer = Instant::now();
        }
        if !message.is_empty() {
            self.messages.push(message);
        }
    }

    fn game_over(&mut self, engine: &mut BTerm) {
        if self.final_time == 0 {
            self.final_time = self.total_time.elapsed().as_secs();
        }

        engine.cls();
        engine.print_centered(1, "GAME OVER!");
        engine.print_centered(2, "Press Escape to exit.");
        engine.print_centered(4, "Your journey has ended!");
        engine.print_centered(
            7,
            format!(
                "Time elapsed since game start: {} seconds or {} minutes.",
                self.final_time,
                self.final_time / 60
            ),
        );
        engine.print_centered(9, format!("Moves done: {}", self.moves));
    }
}

fn main() -> bracket_lib::prelude::BResult<()> {
    let mut context = BTermBuilder::simple(REAL_WIDTH, REAL_HEIGHT)
        .unwrap()
        .with_fps_cap(15.0)
        .with_tile_dimensions(8, 8)
        .with_title("SheLLteR")
        .with_fullscreen(true)
        .build()?;

    context.with_post_scanlines(true);

    let game = State::new(&mut context);
    main_loop(context, game)
}
