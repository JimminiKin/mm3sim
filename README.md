# MM3Sim

A Rust/WebAssembly marble vibraphone simulator built with Bevy and Rapier.

## What this project includes

- `bevy` for 3D rendering and input handling
- `bevy_rapier3d` for physics simulation of dynamic marbles
- WebAssembly target support via `bevy_webgl2`
- Click-to-drop marble spawning over a vibraphone plate

## Build instructions

1. Install Rust and Cargo:
   - https://rustup.rs
2. Add the WebAssembly target:
   - `rustup target add wasm32-unknown-unknown`
3. Install Trunk for local WebAssembly development:
   - `cargo install trunk`
4. Build and serve the app:
   - `trunk serve --open`

## Notes

- The physics scene is defined in `src/main.rs`.
- The browser entry point is `index.html`.
- If `cargo` is not available in your terminal, install Rust first and then reopen your workspace.
