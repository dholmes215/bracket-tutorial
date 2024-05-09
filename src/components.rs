use specs_derive::Component;
use specs::prelude::*;
use bracket_lib::prelude::FontCharType;
use bracket_lib::color::RGB;

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}
