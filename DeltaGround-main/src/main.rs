use std::{process, time::Instant, fs };
use bracket_lib::terminal::{main_loop, GameState, BTerm, BTermBuilder, VirtualKeyCode, RGB, BLACK, WHITE, YELLOW, GRAY100, GRAY, };
use chrono::Utc;

pub mod player;
use player::{map::{Map, MapType, TileType, HEIGHT, WIDTH}, gui, Player};

pub mod entity;

const REAL_WIDTH: i32 = 64;
const REAL_HEIGHT: i32 = 36;

#[derive(Clone)]
struct State {
    map: Map,
    player: Player,
    moves: u128,
    floor: u16,
    messages: Vec<String>,
    timer: Instant,
    game_over: bool,
    total_time: Instant,
    final_time: u64,
}

impl GameState for State {
    fn tick(&mut self, engine: &mut BTerm) {
        if self.player.health == 0 {
            self.game_over(engine);
            self.game_over = true;
        }

        let input =  self.input(engine);
        if !self.game_over {
            if self.moves == 0 {
                self.messages.push(String::from("Welcome to DeltaGround!"));
                self.restart(engine);
            }

            if !input.1.is_empty() {
                self.messages.push(input.1);
            }

            let mut i = false;
            match input.0 {
                1 | 3 => i = true,
                2 => {
                    self.floor += 1;
                    self.map = Map::new(engine, MapType::Dungeon, 0, self.floor);
                    self.player.pos.0 = self.map.start_pos.0;
                    self.player.pos.1 = self.map.start_pos.1;
                    self.restart(engine);
                }
                _ => {},
            }
            if i {
                let timer = Instant::now();
                self.moves += 1;
                println!("Moves made: {}", self.moves);

                engine.cls();

                let message ; 
                if input.0 == 3 {
                    message = self.map.draw(&mut self.player, false, engine);
                }
                else {
                    message = self.map.draw(&mut self.player, true, engine);
                }

                if !message.is_empty() {
                    self.messages.push(message);
                }
                
                self.player.draw(engine);
                println!("The move is made in {} seconds", timer.elapsed().as_secs_f32());
            }

            self.what_is_it(engine);
            gui::draw_ui(&self.messages, self.player.clone(), self.floor, engine);
        }
    }
}

impl State {
    pub fn new(engine: &mut BTerm, t: MapType, seed: u64) -> Self {
        fs::create_dir_all("./screenshots").unwrap();
        let map = Map::new(engine, t, seed, 1);
        Self { 
            map: map.clone(),
            player: Player::new(map.clone().start_pos.0, map.clone().start_pos.1, 10),
            moves: 0,
            floor: 1,
            messages: vec![],
            timer: Instant::now(),
            game_over: false,
            total_time: Instant::now(),
            final_time: 0,
        }
    }

    fn input(&mut self, engine: &mut BTerm) -> (u8, String) {
        match engine.key {
            Some(key) => match key {
                VirtualKeyCode::Escape => process::exit(0),
                VirtualKeyCode::F2 => { 
                    engine.screenshot(format!("screenshots/screen{}.png", Utc::now().timestamp())); 
                    (0, String::new())
                }

                _ => if !self.game_over
                    { return self.player.action(key, &mut self.map) } else { (0, String::new()) },
            },
            None => (0, String::new()), 
        }
    }

    fn restart(&mut self, engine: &mut BTerm) {
        self.player.action(VirtualKeyCode::F3, &mut self.map);
        engine.cls();
        self.map.draw(&mut self.player, true, engine);
        self.player.draw(engine);
        self.moves += 1;

        self.messages.push(String::from(format!("Floor number: {}", self.floor)));
        gui::draw_ui(&self.messages, self.player.clone(), self.floor, engine);
    }

    fn what_is_it(&mut self, engine: &mut BTerm) {
        let mut message: &str = "";
        if engine.left_click && self.timer.elapsed().as_secs_f32() >= 0.1 {            
            let mouse_pos = engine.mouse_pos();
            if mouse_pos.0 < WIDTH && mouse_pos.1 < HEIGHT {    
                if self.player.is_visible(mouse_pos) {    
                    let tile = self.map.source[Map::xy_idx(mouse_pos.0, mouse_pos.1)] ;
                    message = match tile {
                        TileType::Tree1 | TileType::Tree2 => "This is a tree",
                        TileType::Wall => "This is a wall",
                        TileType::DamagedWall => "This is a damaged wall",
                        TileType::WeakWall => "This is a week wall",
                        TileType::Exit => "This is the ladder to the next dungeon",
                        TileType::Column => "This is a column that supporting the dungeon ceiling",
                        TileType::Scarecrow => "This is a scarecrow",
                        TileType::HealthPotion => "This is a health potion",
                        TileType::Floor => {
                            let mut str = "This is just nothing";
                            for entity in &self.map.entities {
                                if (entity.x, entity.y) == mouse_pos {
                                    match entity.t {
                                        entity::EntityType::Zombie => str = "This is a zombie",
                                        entity::EntityType::FrozenTroll => str = "This is a frozen troll",
                                        entity::EntityType::Human => str = "This is lonely traveler",
                                        entity::EntityType::Orc => str = "This is a angry orc"
                                    }
                                }
                            }
                            if self.player.pos == mouse_pos {
                                str = "This is you"
                            }
                            str
                        },
                    };
                }
                self.timer = Instant::now();
            }
        }
        if message != "" {
            self.messages.push(String::from(message));
        }
    }

    fn game_over(&mut self, engine: &mut BTerm) {
        if !self.game_over {
            self.final_time = self.total_time.elapsed().as_secs();
        }

        engine.cls();
        engine.print_centered(1, "GAME OVER!");
        engine.print_centered(2, "Press Escape to exit.");
        engine.print_centered(4, "Your journey has ended!");
        engine.print_centered(6, format!("Levels(Dungeons) completed: {}", self.floor-1));
        engine.print_centered(7, format!("Time elapsed since game start: {} seconds or {} minutes.", 
            self.final_time, self.final_time/60));
        engine.print_centered(9, format!("Moves done: {}", self.moves));
    }
}

fn main() -> bracket_lib::prelude::BResult<()> {
    let mut context = BTermBuilder::simple(REAL_WIDTH, REAL_HEIGHT).unwrap()
        .with_tile_dimensions(8, 8)
        .with_title("DeltaGround")
        .with_fullscreen(true)
        .build()?;

    context.with_post_scanlines(false);

    let game = State::new(&mut context, MapType::Dungeon, 0);
    main_loop(context, game)
}
