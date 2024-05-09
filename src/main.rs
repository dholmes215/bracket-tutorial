mod map;

use bracket_lib::prelude::*;
use specs::prelude::*;
use std::cmp::{max, min};
use specs_derive::Component;
use crate::map::new_map_rooms_and_corridors;

const TERM_WIDTH: i32 = 80;
const TERM_HEIGHT: i32 = 40;

const MAP_WIDTH: i32 = TERM_WIDTH;
const MAP_HEIGHT: i32 = TERM_HEIGHT;

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component, Debug)]
struct Player {}

#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall,
    Floor,
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

fn draw_map(map: &[TileType], ctx: &mut BTerm) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        // Render a tile depending on the tile type
        match tile {
            TileType::Floor => {
                ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.), to_cp437('.'));
            }
            TileType::Wall => {
                ctx.set(x, y, RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0., 0., 0.), to_cp437('#'));
            }
        }

        // Move the coordinates
        x += 1;
        if x > MAP_WIDTH - 1 {
            x = 0;
            y += 1;
        }
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[destination_idx] != TileType::Wall {
            pos.x = min(MAP_WIDTH - 1, max(0, pos.x + delta_x));
            pos.y = min(MAP_HEIGHT - 1, max(0, pos.y + delta_y));
        }
    }
}

struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {}
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

fn player_input(gs: &mut State, ctx: &mut BTerm) {
    // Player movement
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),
            
            _ => {}
        }
    }
}

embedded_resource!(FONT, "../resources/terminal_10x16.png");

fn main() -> BError {
    link_resource!(FONT, "resources/terminal_10x16.png");
    let builder = BTermBuilder::new()
        .with_title("Roguelike Tutorial")
        .with_font("terminal_10x16.png", 10, 16)
        .with_tile_dimensions(10, 16)
        .with_simple_console(TERM_WIDTH, TERM_HEIGHT, "terminal_10x16.png")
        .with_dimensions(TERM_WIDTH, TERM_HEIGHT);
    #[cfg(all(any(feature = "opengl", feature = "webgpu"), not(target_arch = "wasm32")))]
        let builder = builder.with_automatic_console_resize(true);
    let context = builder.build()?;

    let mut gs = State {
        ecs: World::new()
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();

    let (rooms, map) = new_map_rooms_and_corridors();
    gs.ecs.insert(map);
    let (player_x, player_y) = rooms[0].center();

    gs.ecs.create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: to_cp437('@'),
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK),
        })
        .with(Player {})
        .build();

    main_loop(context, gs)
}
