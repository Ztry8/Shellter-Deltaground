use bracket_lib::terminal::{RGB, BTerm, WHITE, BLACK, RED, YELLOW, CYAN};
use crate::{REAL_HEIGHT, REAL_WIDTH};

use super::Player;

pub fn draw_ui(messages: &Vec<String>, player: Player, floor: u16, engine: &mut BTerm) {
    engine.draw_box(0, REAL_HEIGHT-10, REAL_WIDTH-1, 9, RGB::named(WHITE), RGB::named(BLACK));
    
    if player.health <= (player.max_health as f32 * 0.2) as u16 {
        engine.print_color(3, REAL_HEIGHT-10, RED, YELLOW, format!("!HP:{}/{}♥!", player.health, player.max_health));
    }
    else {
        engine.print_color(3, REAL_HEIGHT-10, RED, BLACK, format!("HP:{}/{}♥", player.health, player.max_health));
    }
    engine.print_color(14, REAL_HEIGHT-10, CYAN, BLACK, format!("Dungeon floor:{}♦", floor));

    let mut y = REAL_HEIGHT-9;
    for s in messages.iter().rev() {
        if y <= 52 { 
            engine.print(1, y, format!("[*] {}", s));
        }
        y += 1;
    }
}