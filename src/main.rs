use bracket_lib::prelude::*;
use specs::prelude::*;
use std::cmp::{max, min};
use specs_derive::Component;

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

struct State {
    ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print(1, 1, "Hello Rust World");
    }
}

embedded_resource!(FONT, "../resources/terminal_10x16.png");

fn main() -> BError {
    link_resource!(FONT, "resources/terminal_10x16.png");
    let builder = BTermBuilder::new()
        .with_title("Roguelike Tutorial")
        .with_font("terminal_10x16.png", 10, 16)
        .with_tile_dimensions(10, 16)
        .with_simple_console(80, 25, "terminal_10x16.png")
        .with_dimensions(80, 25);
    #[cfg(not(target_arch = "wasm32"))]
        let builder = builder.with_automatic_console_resize(true);
    let context = builder.build()?;

    let mut gs = State {
        ecs: World::new()
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();

    main_loop(context, gs)
}
