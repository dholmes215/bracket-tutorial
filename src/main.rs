mod map;
mod player;
mod components;
mod visibility_system;
mod monster_ai_system;
mod map_indexing_system;

use bracket_lib::prelude::*;
use specs::prelude::*;
use components::*;
use crate::map::*;
use crate::map_indexing_system::MapIndexingSystem;
use crate::monster_ai_system::MonsterAI;
use crate::visibility_system::VisibilitySystem;

const TERM_WIDTH: i32 = 80;
const TERM_HEIGHT: i32 = 40;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, Running }

struct State {
    pub ecs: World,
    pub runstate: RunState,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player::player_input(self, ctx);
        }

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
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
        ecs: World::new(),
        runstate: RunState::Running,
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let mut rng = RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let glyph: FontCharType;
        let name: String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {
                glyph = to_cp437('g');
                name = "Goblin".to_string();
            }
            _ => {
                glyph = to_cp437('o');
                name = "Orc".to_string();
            }
        }

        gs.ecs.create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph,
                fg: RGB::named(RED),
                bg: RGB::named(BLACK),
            })
            .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
            .with(Monster {})
            .with(Name { name: format!("{} #{}", &name, i) })
            .with(BlocksTile {})
            .with(CombatStats { max_hp: 16, hp: 16, defense: 1, power: 4 })
            .build();
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));

    gs.ecs.create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: to_cp437('@'),
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK),
        })
        .with(Player {})
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Name { name: "Player".to_string() })
        .with(BlocksTile {})
        .with(CombatStats { max_hp: 30, hp: 30, defense: 2, power: 5 })
        .build();

    main_loop(context, gs)
}
