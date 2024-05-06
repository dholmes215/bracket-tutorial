# Bracket Roguelike Tutorial
This is me implementing Section 1 of the Rust Roguelike Tutorial at <https://bfnightly.bracketproductions.com/rustbook/chapter_1.html>.

To build it for web, run (on Windows):
```
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen .\target\wasm32-unknown-unknown\release\bracket-tutorial.wasm --out-dir wasm --no-modules --no-typescript
```
To serve it locally, run:
```
cargo install http-server
http-server wasm
```
See <https://bfnightly.bracketproductions.com/rustbook/webbuild.html> for more details.
