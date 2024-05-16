use specs_derive::{Component, ConvertSaveload};
use specs::prelude::*;
use specs::saveload::{Marker, ConvertSaveload};
use specs::error::NoError;
use bracket_lib::prelude::*;
use bracket_lib::color::RGB;
use specs::saveload::SimpleMarker;
use serde::{Serialize, Deserialize};
use crate::State;

#[derive(Component, ConvertSaveload, Copy, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Player {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Monster {}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct BlocksTile {}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount: vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Item {}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<Point>,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct WantsToDropItem {
    pub item: Entity
}

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct Consumable {}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct Confusion {
    pub turns: i32,
}

pub struct SerializeMe;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: super::map::Map,
}

pub fn register_all_components(gs: &mut State) {
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
}
