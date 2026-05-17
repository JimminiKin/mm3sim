# MM3Sim

A real-time physics simulator for tuning a marble-machine snare drum trigger, built with Rust, Bevy, and Rapier. Runs natively and in the browser via WebAssembly.

## What it simulates

A marble-machine trigger fires two steel marbles at a snare drum head simultaneously:

- **Drop marble** (orange) — falls ~1 m from above the snare centre. Its flight time is nearly fixed by gravity.
- **Chute marble** (blue) — starts at rest at the top of a shaped ramp, slides down, lifts off, then flies to the snare.

The goal is to tune the chute geometry so both marbles arrive at exactly the same time (Δt = 0 ms) with maximum consistency across repeated runs.

## Features

- **Cubic Bézier chute** with adaptive tessellation — drag control-point handles directly in 3D or adjust coordinates numerically
- **Per-run statistics** — flight time, impact speed, angle of attack, kinetic energy, spin, slide duration, liftoff velocity
- **Aggregate summary** — mean ± std, min/max across all complete runs
- **Velocity graph** — per-run chute marble velocity profile (vy, vz, speed, spin) over time
- **Counterweighted pivot arm** — physically simulated snare mount with stop posts
- **Impact sound** — volume scales with impact speed (native: WAV synthesis; WASM: Web Audio API)
- **Marble trail** — semi-transparent dots show the trajectory of each run

## Build

**Requirements:** Rust stable (rustup.rs)

### Native (desktop)

```sh
cargo run
```

### WebAssembly (browser)

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve --open
```

## Controls

| Input | Action |
|---|---|
| Left Click | Spawn a marble pair (drop + chute) |
| Right Click + Drag | Orbit camera |
| Middle Click + Drag | Pan camera |
| Scroll Wheel | Zoom (ignored when cursor is over a UI panel) |
| Drag handle sphere | Move that Bézier control point |
| Drag chute body | Translate the entire chute curve |

## UI Panels

All panels collapse and expand by clicking their title bar.

### Stats (top-left)
Live marble velocities, per-run impact data, and aggregate summary statistics. Use **Expand All / Collapse All** to manage many runs, and **Reset** to clear history.

### Parameters (top-right)
Shape the chute curve. P3 (exit) and P0 (entry) are the endpoints; CP2 and CP1 are the inner Bézier handles. Enable **Straight line** mode to collapse the curve to a pure ramp (curve handles are hidden automatically). Individual handle sphere visibility is controlled by **Show curve handles** and **Show endpoint handles**.

### Help (bottom-left)
In-app reference for all controls, stats, and the physics model. Stays collapsed until opened.

### Velocity Graph (floating)
Open per run via **Show Graph** inside each run entry. Shows the chute marble's velocity components over time. Multiple graphs can be open simultaneously.

## Project layout

```
src/
  components/
    chute.rs        — Bézier chute mesh + collider spawning (adaptive tessellation)
    snare.rs        — Snare drum + counterweighted pivot arm
  resources/
    constants.rs    — All tunable physical and visual constants
    chute_params.rs — Bézier control points and editor state
    marble_runs.rs  — Per-run data model (HitRecord, VelocitySample, RunHistory)
    marble_collisions.rs — Marble-marble collision toggle
  systems/
    setup.rs        — Scene initialisation (camera, lights, initial chute + snare)
    marble.rs       — Marble spawning, trail recording, slide/impact data collection
    camera.rs       — Orbit camera (right-click orbit, middle-click pan, scroll zoom)
    chute_handles.rs — Handle sphere picking, drag (individual + body translate), gizmos
    chute_editor.rs — Parameters panel UI + chute rebuild on dirty flag
    hud.rs          — Stats panel, Help panel, summary statistics
    marble_graph.rs — Per-run velocity graph recording and rendering
    axes.rs         — 3D axis orientation widget (bottom-right overlay)
    sound.rs        — Impact sound synthesis
```

## Physics model

All objects use Rapier3D rigid-body physics.

| Parameter | Value |
|---|---|
| Marble diameter | 20 mm |
| Marble mass | 14 g |
| Restitution (steel) | 0.60 |
| Friction (steel) | 0.18 |
| Chute restitution | 0.35 |
| Chute friction | 0.20 |
| Drop height | 1.0 m above snare top |
| Snare diameter | 14″ (356 mm) |
| Snare depth | 5.5″ (140 mm) |
