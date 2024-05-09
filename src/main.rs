mod map;
mod player;
mod components;

use bracket_lib::prelude::*;
use specs::prelude::*;
use components::{Player, Position, Renderable};
use crate::components::Viewshed;
use crate::map::*;

const TERM_WIDTH: i32 = 80;
const TERM_HEIGHT: i32 = 40;

struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {}
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        player::player_input(self, ctx);
        self.run_systems();

        let map = self.ecs.fetch::<Map>();
        draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
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
    let mut context = builder.build()?;
    context.with_post_scanlines(true);

    let mut gs = State {
        ecs: World::new()
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    gs.ecs.insert(map);

    gs.ecs.create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: to_cp437('@'),
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK),
        })
        .with(Player {})
        .with(Viewshed { visible_tiles: Vec::new(), range: 8 })
        .build();

    main_loop(context, gs)
}
