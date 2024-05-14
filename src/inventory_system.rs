use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};
use crate::components::{CombatStats, Consumable, InBackpack, Name, Position, ProvidesHealing, WantsToUseItem, WantsToDropItem};
use crate::gamelog::GameLog;

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (ReadExpect<'a, Entity>,
                       WriteExpect<'a, GameLog>,
                       Entities<'a>,
                       WriteStorage<'a, WantsToUseItem>,
                       ReadStorage<'a, Name>,
                       ReadStorage<'a, Consumable>,
                       ReadStorage<'a, ProvidesHealing>,
                       WriteStorage<'a, CombatStats>);

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, wants_use, names, consumables, healing, mut combat_stats) = data;

        for (entity, useitem, stats) in (&entities, &wants_use, &mut combat_stats).join() {
            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.push(format!("You drink the {}, healing {} hp.", names.get(useitem.item).unwrap().name, healer.heal_amount));
                    }
                    let consumable = consumables.get(useitem.item);
                    if let Some(_) = consumable {
                        entities.delete(useitem.item).expect("Delete failed");
                    }
                }
            }
        }
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (ReadExpect<'a, Entity>,
                       WriteExpect<'a, GameLog>,
                       Entities<'a>,
                       WriteStorage<'a, WantsToDropItem>,
                       ReadStorage<'a, Name>,
                       WriteStorage<'a, Position>,
                       WriteStorage<'a, InBackpack>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let dropper_pos = *positions.get(entity).unwrap();
            positions.insert(to_drop.item, dropper_pos).expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!("You drop the {}.", names.get(to_drop.item).unwrap().name));
            }
        }

        wants_drop.clear();
    }
}
