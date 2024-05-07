use bracket_lib::prelude::*;

struct State {}
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

    let gs = State{ };
    main_loop(context, gs)
}
