mod map;
mod player;
mod components;
mod visibility_system;
mod monster_ai_system;
mod map_indexing_system;
mod melee_combat_system;
mod damage_system;
mod gui;
mod gamelog;
mod spawner;
mod item_collection_system;
mod inventory_system;

use bracket_lib::prelude::*;
use specs::prelude::*;
use specs::saveload::SimpleMarkerAllocator;
use components::*;
use crate::damage_system::DamageSystem;
use crate::gui::{ItemMenuResult, MainMenuResult, MainMenuSelection, TargetingResult};
use crate::inventory_system::{ItemDropSystem, ItemUseSystem};
use crate::item_collection_system::ItemCollectionSystem;
use crate::map::*;
use crate::map_indexing_system::MapIndexingSystem;
use crate::melee_combat_system::MeleeCombatSystem;
use crate::monster_ai_system::MonsterAI;
use crate::player::player_input;
use crate::spawner::confusion_scroll;
use crate::visibility_system::VisibilitySystem;

const TERM_WIDTH: i32 = 80;
const TERM_HEIGHT: i32 = 50;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ItemMenu(ItemMenuOp),
    ShowTargeting { range: i32, item: Entity },
    MainMenu { menu_selection: MainMenuSelection },
    SaveGame
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ItemMenuOp {
    Use,
    Drop,
}

struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut melee_combat_system = MeleeCombatSystem {};
        melee_combat_system.run_now(&self.ecs);
        let mut damage_system = DamageSystem {};
        damage_system.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);
        let mut potions = ItemUseSystem {};
        potions.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        ctx.cls();

        match newrunstate {
            RunState::MainMenu {..} => {}
            _ => {
                draw_map(&self.ecs, ctx);

                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let map = self.ecs.fetch::<Map>();
                    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));

                    for (pos, render) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] {
                            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
                        }
                    }

                    gui::draw_ui(&self.ecs, ctx);
                }

            }
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ItemMenu(op) => {
                let result = match op {
                    ItemMenuOp::Use => gui::show_inventory(self, ctx),
                    ItemMenuOp::Drop => gui::drop_item_menu(self, ctx),
                };
                match result {
                    ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::SelectedItem(item_entity) => {
                        match op {
                            ItemMenuOp::Use => {
                                let is_ranged = self.ecs.read_storage::<Ranged>();
                                let is_item_ranged = is_ranged.get(item_entity);
                                if let Some(is_item_ranged) = is_item_ranged {
                                    newrunstate = RunState::ShowTargeting { range: is_item_ranged.range, item: item_entity };
                                } else {
                                    let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                                    intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item: item_entity, target: None }).expect("Unable to insert intent");
                                    newrunstate = RunState::PlayerTurn;
                                }
                            }
                            ItemMenuOp::Drop => {
                                let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                                intent.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem { item: item_entity }).expect("Unable to insert intent");
                                newrunstate = RunState::PlayerTurn;
                            }
                        }
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let result = gui::ranged_target(self, ctx, range);
                match result {
                    TargetingResult::Cancel => newrunstate = RunState::AwaitingInput,
                    TargetingResult::NoResponse => {}
                    TargetingResult::SelectedPoint(point) => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item, target: Some(point) }).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    MainMenuResult::NoSelection { selected } => newrunstate = RunState::MainMenu { menu_selection: selected },
                    MainMenuResult::Selected { selected } => {
                        match selected {
                            MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                            MainMenuSelection::LoadGame => newrunstate = RunState::PreRun,
                            MainMenuSelection::Quit => std::process::exit(0),
                        }
                    }
                }
            }
            RunState::SaveGame => {
                let data = serde_json::to_string(&*self.ecs.fetch::<Map>()).unwrap();
                println!("{}", data);
                newrunstate = RunState::MainMenu { menu_selection: MainMenuSelection::LoadGame };
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);
    }
}

embedded_resource!(FONT, "../resources/terminal_10x16_modified.png");

fn main() -> BError {
    link_resource!(FONT, "resources/terminal_10x16_modified.png");
    let builder = BTermBuilder::new()
        .with_title("Roguelike Tutorial")
        .with_font("terminal_10x16_modified.png", 10, 16)
        .with_tile_dimensions(10, 16)
        .with_simple_console(TERM_WIDTH, TERM_HEIGHT, "terminal_10x16_modified.png")
        .with_dimensions(TERM_WIDTH, TERM_HEIGHT);
    #[cfg(all(any(feature = "opengl", feature = "webgpu"), not(target_arch = "wasm32")))]
        let builder = builder.with_automatic_console_resize(true);
    let mut context = builder.build()?;
    context.with_post_scanlines(true);

    let mut gs = State {
        ecs: World::new(),
    };
    register_all_components(&mut gs);

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);
    confusion_scroll(&mut gs.ecs, player_x, player_y);  // TODO: for testing, remove later

    gs.ecs.insert(RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog { entries: vec!["Welcome to Rusty Roguelike".to_string()] });
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    main_loop(context, gs)
}
