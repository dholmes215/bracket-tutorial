use bracket_lib::prelude::*;
use specs::prelude::*;

use crate::components::*;
use crate::gamelog::GameLog;
use crate::map::Map;
use crate::{RunState, State, TERM_HEIGHT};
use crate::gui::MainMenuSelection::{LoadGame, NewGame, Quit};

pub fn draw_ui(ecs: &World, ctx: &mut BTerm) {
    ctx.draw_box(0, TERM_HEIGHT - 7, 79, 6, RGB::named(WHITE), RGB::named(BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, TERM_HEIGHT - 7, RGB::named(YELLOW), RGB::named(BLACK), &health);

        ctx.draw_bar_horizontal(28, TERM_HEIGHT - 7, 51, stats.hp, stats.max_hp, RGB::named(RED), RGB::named(BLACK));
    }

    let log = ecs.fetch::<GameLog>();

    // let mut y = 44;
    // for s in log.entries.iter().rev() {
    //     if y < 49 { ctx.print(2, y, s); }
    //     y += 1;
    // }

    let mut y = TERM_HEIGHT - 2;
    for s in log.entries.iter().rev() {
        if y == TERM_HEIGHT - 2 { ctx.print(2, y, s); } else if y > TERM_HEIGHT - 7 { ctx.print_color(2, y, RGB::named(GREY), RGB::named(BLACK), s); }
        y -= 1;
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(MAGENTA));

    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut BTerm) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height { return; }
    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.x, position.y);
        if position.x == mouse_pos.0 && position.y == mouse_pos.1 && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 { width = s.len() as i32; }
        }
        width += 3;

        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, RGB::named(WHITE), RGB::named(GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i, y, RGB::named(WHITE), RGB::named(GREY), &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(WHITE), RGB::named(GREY), &"->".to_string());
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, RGB::named(WHITE), RGB::named(GREY), s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y, RGB::named(WHITE), RGB::named(GREY), &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(WHITE), RGB::named(GREY), &"<-".to_string());
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    SelectedItem(Entity),
}

#[derive(PartialEq, Copy, Clone)]
pub enum TargetingResult {
    Cancel,
    NoResponse,
    SelectedPoint(Point),
}

pub fn item_menu(gs: &mut State, ctx: &mut BTerm, title: &str) -> ItemMenuResult {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y - 2, 31, (count + 3) as i32, RGB::named(WHITE), RGB::named(BLACK));
    ctx.print_color(18, y - 2, RGB::named(YELLOW), RGB::named(BLACK), title);
    ctx.print_color(18, y + count as i32 + 1, RGB::named(YELLOW), RGB::named(BLACK), "ESCAPE to cancel");

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity) {
        ctx.set(17, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437('('));
        ctx.set(18, y, RGB::named(YELLOW), RGB::named(BLACK), 97 + j as FontCharType);
        ctx.set(19, y, RGB::named(WHITE), RGB::named(BLACK), to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => ItemMenuResult::Cancel,
                _ => {
                    let selection = letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return ItemMenuResult::SelectedItem(equippable[selection as usize]);
                    }
                    ItemMenuResult::NoResponse
                }
            }
        }
    }
}

pub fn show_inventory(gs: &mut State, ctx: &mut BTerm) -> ItemMenuResult {
    item_menu(gs, ctx, "Inventory")
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut BTerm) -> ItemMenuResult {
    item_menu(gs, ctx, "Drop Which Item?")
}

pub fn ranged_target(gs: &mut State, ctx: &mut BTerm, range: i32) -> TargetingResult {
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(5, 0, RGB::named(YELLOW), RGB::named(BLACK), "Select Target:");

    // Highlight available target cells
    let mut available_cells = Vec::new();
    let visible = viewsheds.get(*player_entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for idx in visible.visible_tiles.iter() {
            let distance = DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                ctx.set_bg(idx.x, idx.y, RGB::named(BLUE));
                available_cells.push(idx);
            }
        }
    } else {
        return TargetingResult::Cancel;
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_pos.0 && idx.y == mouse_pos.1 { valid_target = true; }
    }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(CYAN));
        if ctx.left_click {
            return TargetingResult::SelectedPoint(Point::new(mouse_pos.0, mouse_pos.1));
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(RED));
        if ctx.left_click {
            return TargetingResult::Cancel;
        }
    }

    TargetingResult::NoResponse
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MainMenuSelection { NewGame, LoadGame, Quit }

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult { NoSelection { selected: MainMenuSelection }, Selected { selected: MainMenuSelection } }

pub fn main_menu(gs: &mut State, ctx: &mut BTerm) -> MainMenuResult {
    use MainMenuSelection::*;
    use MainMenuResult::*;
    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(15, RGB::named(YELLOW), RGB::named(BLACK), "Rust Roguelike Tutorial");

    if let RunState::MainMenu { menu_selection: selected } = *runstate {
        let get_color = |s| if selected == s { RGB::named(MAGENTA) } else { RGB::named(WHITE) };
        ctx.print_color_centered(24, get_color(NewGame), RGB::named(BLACK), "Begin New Game");
        if save_exists {
            ctx.print_color_centered(25, get_color(LoadGame), RGB::named(BLACK), "Load Game");
        }
        ctx.print_color_centered(26, get_color(Quit), RGB::named(BLACK), "Quit");

        return match ctx.key {
            None => NoSelection { selected },
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => NoSelection { selected: Quit },
                    VirtualKeyCode::Up => {
                        let mut newselection;
                        match selected {
                            NewGame => newselection = Quit,
                            LoadGame => newselection = NewGame,
                            Quit => newselection = LoadGame
                        }
                        if newselection == LoadGame && !save_exists {
                            newselection = NewGame;
                        }
                        NoSelection { selected: newselection }
                    }
                    VirtualKeyCode::Down => {

                        let mut newselection;
                        match selected {
                            NewGame => newselection = LoadGame,
                            LoadGame => newselection = Quit,
                            Quit => newselection = NewGame
                        }
                        if newselection == LoadGame && !save_exists {
                            newselection = Quit;
                        }
                        NoSelection { selected: newselection }
                    }
                    VirtualKeyCode::Return => Selected { selected },
                    _ => NoSelection { selected },
                }
            }
        };
    }

    NoSelection { selected: NewGame }
}
