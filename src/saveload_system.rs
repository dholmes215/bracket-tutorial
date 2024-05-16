use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use bracket_lib::prelude::Point;
use specs::{Builder, Entity, Join, World, WorldExt};
use specs::saveload::{MarkedBuilder, SimpleMarker, SerializeComponents, DeserializeComponents, SimpleMarkerAllocator};
use crate::components::*;

const SAVE_PATH: &str = "./savegame.json.gz";

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<std::convert::Infallible, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(ecs: &mut World) {
    // Create helper
    let mapcopy = ecs.get_mut::<super::map::Map>().unwrap().clone();
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper { map: mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Actually serialize
    {
        let data = (ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMe>>());

        let writer = File::create(SAVE_PATH).unwrap();
        let gz = flate2::GzBuilder::new().write(writer, flate2::Compression::fast());
        let mut serializer = serde_json::Serializer::new(gz);
        serialize_individually!(ecs, serializer, data, Position, Renderable, Player, Viewshed, Monster,
            Name, BlocksTile, CombatStats, SufferDamage, WantsToMelee, Item, Consumable, Ranged, InflictsDamage,
            AreaOfEffect, Confusion, ProvidesHealing, InBackpack, WantsToPickupItem, WantsToUseItem,
            WantsToDropItem, SerializationHelper
        );
    }

    // Clean up
    ecs.delete_entity(savehelper).expect("Crash on cleanup");
}

#[cfg(target_arch = "wasm32")]
pub fn save_game(_ecs : &mut World) {
}

pub fn does_save_exist() -> bool {
    Path::new(SAVE_PATH).exists()
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<std::convert::Infallible, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &mut $data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

pub fn load_game(ecs: &mut World) {
    {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            ecs.delete_entity(*del).expect("Deletion failed");
        }
    }

    let bufreader = BufReader::new(File::open(SAVE_PATH).expect("Failed to open save file to read"));
    let gz = flate2::bufread::GzDecoder::new(bufreader);
    let mut de = serde_json::Deserializer::from_reader(gz);

    {
        let mut d = (&mut ecs.entities(), &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(), &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>());

        deserialize_individually!(ecs, de, d, Position, Renderable, Player, Viewshed, Monster,
            Name, BlocksTile, CombatStats, SufferDamage, WantsToMelee, Item, Consumable, Ranged, InflictsDamage,
            AreaOfEffect, Confusion, ProvidesHealing, InBackpack, WantsToPickupItem, WantsToUseItem,
            WantsToDropItem, SerializationHelper
        );
    }

    let mut deleteme: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();
        for (e, h) in (&entities, &helper).join() {
            let mut worldmap = ecs.write_resource::<super::map::Map>();
            *worldmap = h.map.clone();
            worldmap.tile_content = vec![Vec::new(); super::map::MAPCOUNT];
            deleteme = Some(e);
        }
        for (e, _p, pos) in (&entities, &player, &position).join() {
            let mut ppos = ecs.write_resource::<Point>();
            *ppos = Point::new(pos.x, pos.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }
    ecs.delete_entity(deleteme.unwrap()).expect("Unable to delete helper");
}

pub fn delete_save() {
    if Path::new(SAVE_PATH).exists() { std::fs::remove_file(SAVE_PATH).expect("Unable to delete file"); }
}
