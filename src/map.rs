use std::cmp::{max, min};
use bracket_lib::prelude::RandomNumberGenerator;
use crate::{MAP_HEIGHT, MAP_WIDTH, TileType, xy_idx};

// Makes a map with solid boundaries and 400 randomly placed walls. No guarantees that it won't
// look awful.
fn new_map_test() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize];

    // Make the boundaries walls
    for x in 0..MAP_WIDTH {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, MAP_HEIGHT - 1)] = TileType::Wall;
    }

    for y in 0..MAP_HEIGHT {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(MAP_WIDTH - 1, y)] = TileType::Wall;
    }

    // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
    // First obtain the thread-local RNG:
    let mut rng = RandomNumberGenerator::new();

    for _i in 0..400 {
        let x = rng.roll_dice(1, MAP_WIDTH - 1);
        let y = rng.roll_dice(1, MAP_HEIGHT - 1);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}

pub struct Rect {
    pub x1: i32,
    pub x2: i32,
    pub y1: i32,
    pub y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x1: x, y1: y, x2: x + w, y2: y + h }
    }

    // Returns true if this overlaps with other
    pub fn intersect(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}

fn apply_room_to_map(room: &Rect, map: &mut [TileType]) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            map[xy_idx(x, y)] = TileType::Floor;
        }
    }
}

fn apply_horizontal_tunnel(map: &mut [TileType], x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < (MAP_WIDTH * MAP_HEIGHT) as usize {
            map[idx as usize] = TileType::Floor;
        }
    }
}

fn apply_vertical_tunnel(map: &mut [TileType], y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < (MAP_WIDTH * MAP_HEIGHT) as usize {
            map[idx as usize] = TileType::Floor;
        }
    }
}

pub fn new_map_rooms_and_corridors() -> (Vec<Rect>, Vec<TileType>) {
    let mut map = vec![TileType::Wall; (MAP_WIDTH * MAP_HEIGHT) as usize];

    let mut rooms: Vec<Rect> = Vec::new();
    const MAX_ROOMS: i32 = 30;
    const MIN_SIZE: i32 = 6;
    const MAX_SIZE: i32 = 10;

    let mut rng = RandomNumberGenerator::new();

    for _ in 0..MAX_ROOMS {
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, MAP_WIDTH - w - 1) - 1;
        let y = rng.roll_dice(1, MAP_HEIGHT - h - 1) - 1;
        let new_room = Rect::new(x, y, w, h);
        let mut ok = true;
        for other_room in rooms.iter() {
            if new_room.intersect(other_room) { ok = false }
        }
        if ok {
            apply_room_to_map(&new_room, &mut map);
            if !rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                if rng.range(0, 2) == 1 {
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_tunnel(&mut map, prev_y, new_y, prev_x);
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, new_y);
                }
            }

            rooms.push(new_room);
        }
    }

    (rooms, map)
}