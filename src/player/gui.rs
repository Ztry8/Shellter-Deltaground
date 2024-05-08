use crate::{REAL_HEIGHT, REAL_WIDTH};
use bracket_lib::{color::{CYAN, CYAN1, CYAN3, GOLD, GREEN3, HOT_PINK, PURPLE, WHITESMOKE}, terminal::{to_cp437, BTerm, BLACK, RED, RGB, WHITE, YELLOW}};
use super::{ItemType, Player};

pub const RIGHT_TABLE_WIDTH: i32 = 23;
pub const BOTTOM_TABLE_HEIGHT: i32 = 9;
const EQUIPMENT_TABLE_WIDTH: i32 = 25;
const INVENTORY_TABLE_WIDTH: i32 = REAL_WIDTH-RIGHT_TABLE_WIDTH-EQUIPMENT_TABLE_WIDTH-1;
const INVENTORY_TABLE_HEIGHT: i32 = REAL_HEIGHT-BOTTOM_TABLE_HEIGHT-2;

pub fn draw_inventory(engine: &mut BTerm, player: &mut Player, wheel: i32) {
    engine.draw_box(0, -1,
        INVENTORY_TABLE_WIDTH,
        INVENTORY_TABLE_HEIGHT+1,
        RGB::named(WHITE),
        RGB::named(BLACK));
    engine.draw_box(INVENTORY_TABLE_WIDTH+1, 0,
        EQUIPMENT_TABLE_WIDTH-2,
        INVENTORY_TABLE_HEIGHT,
        RGB::named(WHITE),
        RGB::named(BLACK));
    draw_line_equip(engine, format!("Currently"), WHITE, -1);
    draw_line_equip(engine, format!("equipped with:"), WHITE, 0);

    if let Some(ItemType::Weapon(weapon_type, material, damage))
        = player.weapon.clone() {
            draw_line_equip(engine, 
                format!("Weapon: {} {}", material, weapon_type), CYAN, 2);
            draw_line_equip(engine, format!("Damage: {}", damage), WHITE, 3);
            draw_line_equip(engine, format!("Weight: {}kg", Player::weight_by_weapon(weapon_type, material)), 
                WHITE, 4);
    } 
    
    
    let mut yy = 2+wheel;
    let mut cont = true;
    for i in 0..player.inventory.len() {
        let (ss, ss2): (String, String) = match player.inventory[i].clone() {
            super::ItemType::Potion(size, color) => 
                (format!("{} {} potion", size, Player::color_to_str(color)), 
                format!("Weight: {}", Player::weight_by_size(size.clone()))),
            super::ItemType::Weapon(weapon_type, material , damage) => 
                (format!("{} {}", material, weapon_type.to_string().to_lowercase()), 
                format!("Damage: {} Weight: {}kg", damage, 
                Player::weight_by_weapon(weapon_type, material.clone()))),
        };
        let mut j = 0;
        if i >= 8 {
            j += 1;
        }

        if !cont {
            draw_line_inventory(engine, 
                format!("{}) {}", ((97+i+j) as u8 as char).to_uppercase(), ss), 
                CYAN, 
                yy-1 as i32);
            draw_line_inventory(engine, 
                format!("{}", ss2), 
                CYAN, 
                yy as i32);
        } else {
            draw_line_inventory(engine, 
                format!("{}) {}", ((97+i+j) as u8 as char).to_uppercase(), ss), 
                WHITE, 
                yy-1 as i32);
            draw_line_inventory(engine, 
                format!("{}", ss2), 
                WHITE, 
                yy as i32);
        }

        yy += 2;
        cont = !cont;
    }
    draw_line_inventory(engine, format!("Inventory:"), CYAN, INVENTORY_TABLE_HEIGHT-3);
    draw_line_inventory(engine, format!("PageUp/PageDown for list"), CYAN,INVENTORY_TABLE_HEIGHT-2);
    draw_line_inventory(engine, format!("Press I to close"), CYAN,INVENTORY_TABLE_HEIGHT-1);
}

fn draw_line_inventory(engine: &mut BTerm, string: String, fg: (u8, u8, u8), y: i32) {
    engine.print_color(
        1, 0+y+1,
        RGB::named(fg),
        RGB::named(BLACK),
        string);
}

fn draw_line_equip(engine: &mut BTerm, string: String, fg: (u8, u8, u8), y: i32) {
    engine.print_color(1+1+INVENTORY_TABLE_WIDTH, 0+y+1,
        RGB::named(fg),
        RGB::named(BLACK),
        string);
}

fn draw_line_right(engine: &mut BTerm, string: String, fg: (u8, u8, u8), bg: (u8, u8, u8), y: i32) {
    engine.print_color(
        REAL_WIDTH - RIGHT_TABLE_WIDTH as i32,
        y+1,
        fg,
        bg,
        string,
    );
}

fn draw_message_bar(engine: &mut BTerm, messages: &[String]) {
    let mut y = REAL_HEIGHT - 9;
    
    engine.draw_box(
        0,
        REAL_HEIGHT - 10,
        REAL_WIDTH - 1,
        BOTTOM_TABLE_HEIGHT+1,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );

    let mut cont = true;
    for s in messages.iter().rev() {
        if y < REAL_HEIGHT {
            let real_s: Vec<&str>  = s.split("\n").collect();
            if cont {
                engine.print_color(1, y, WHITE, BLACK, format!("{}", real_s[0]));
            } else {
                engine.print_color(1, y, CYAN3, BLACK, format!("{}", real_s[0]));
            }
            cont = !cont;
            for i in 1..real_s.len() {
                y += 1;
                if cont {
                    engine.print_color(1, y, WHITE, BLACK, format!("    {}", real_s[i]));
                } else {
                    engine.print_color(1, y, CYAN3, BLACK, format!("    {}", real_s[i]));
                }
                cont = !cont;
            }
        }
        y += 1;
    }
}

fn draw_right_bar(engine: &mut BTerm, player: Player) {
    engine.draw_box(
        REAL_WIDTH - 1 - RIGHT_TABLE_WIDTH as i32,
        0,
        RIGHT_TABLE_WIDTH,
        REAL_HEIGHT - 2 + 1 - BOTTOM_TABLE_HEIGHT as i32,
        RGB::named(WHITE),
        RGB::named(BLACK),
    );

    if player.health <= (player.max_health as f32 * 0.2) as i32 {
        draw_line_right(engine, format!("HP:{}/{}", player.health, player.max_health), 
            RED, YELLOW, 0);
    } else {
        draw_line_right(engine, format!("HP:{}/{}", player.health, player.max_health),
            RED, BLACK,  0);
    }
    draw_line_right(engine, format!("Damage:{}", player.damage), 
        HOT_PINK, BLACK,  1);
    draw_line_right(engine, format!("ChanceOfCritical:{}%", player.kchance), 
        CYAN1, BLACK,  2);
    
    draw_line_right(engine, format!("Weight:{}/{}kg", player.weight, player.max_weight),
        WHITESMOKE, BLACK,  4);
    draw_line_right(engine, format!("Coins:{}$", player.coins), 
        GOLD, BLACK,  5);
    draw_line_right(engine, format!("EXP:{}", player.exp), 
        GREEN3, BLACK,  6);


    draw_line_right(engine, format!("Lockpick:{}%", player.lockpick), 
        PURPLE, BLACK,  8);
}

pub fn draw_ui(messages: &[String], player: Player, engine: &mut BTerm) {
    draw_message_bar(engine, messages);
    draw_right_bar(engine, player);
    engine.set(REAL_WIDTH-RIGHT_TABLE_WIDTH-1, REAL_HEIGHT-BOTTOM_TABLE_HEIGHT-1, 
        RGB::named(WHITE), RGB::named(BLACK), to_cp437('┴'));
    engine.set(REAL_WIDTH-RIGHT_TABLE_WIDTH+RIGHT_TABLE_WIDTH-1, REAL_HEIGHT-BOTTOM_TABLE_HEIGHT-1, 
        RGB::named(WHITE), RGB::named(BLACK), to_cp437('┤'));
}
