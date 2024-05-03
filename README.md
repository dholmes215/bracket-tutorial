# bracket-lib Hello World
This is the "hello world" program from <https://bfnightly.bracketproductions.com/rustbook/chapter_1.html>.

To build it for web, run (on Windows):
```
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen .\target\wasm32-unknown-unknown\release\hello-bracket.wasm --out-dir wasm --no-modules --no-typescript
```
To serve it locally, run:
```
cargo install http-server
http-server wasm
```
See <https://bfnightly.bracketproductions.com/rustbook/webbuild.html> for more details.
