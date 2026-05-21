# MM3Sim

A real-time physics simulator for tuning a marble-machine snare drum trigger, built with Rust, Bevy 0.18, and Rapier. Runs natively on desktop or in the browser via WebAssembly.

## Overview

MM3Sim models the classic Wintergatan Marble Machine trigger mechanism: two steel marbles are released simultaneously toward a snare drum head. Your task is to tune the chute geometry so both marbles arrive at exactly the same time (**Δt = 0 ms**) with maximum consistency across repeated runs.

## Quick Start

### Desktop
```sh
cargo run
```

### Browser (WebAssembly)
```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve --open
```

## What it simulates

Two steel marbles are released at the same time:

| Marble | Color | Behavior |
|--------|-------|----------|
| **Drop marble** | Orange | Falls ~1 m from above the snare centre. Flight time is nearly fixed by gravity (~450 ms). |
| **Chute marble** | Blue | Spawns at rest on a shaped ramp, slides down, lifts off, then flies to the snare. Chute geometry controls slide duration and liftoff velocity. |

## Controls

| Input | Action |
|-------|--------|
| Left Click | Spawn a marble pair (drop + chute) |
| Right Click + Drag | Orbit camera around scene |
| Middle Click + Drag | Pan camera horizontally/vertically |
| Scroll Wheel | Zoom in/out (disabled over UI panels) |
| Drag handle sphere | Move that Bézier control point to reshape curve |
| Drag chute body | Translate entire chute as a unit |

## UI Panels

All panels can be toggled by clicking their title bars.

### Stats Panel (top-left)
Shows live marble velocities, per-run impact data, and aggregate summary statistics. Key features:

- **Live marbles** — Current speed, vertical velocity, horizontal speed, angle of attack, and spin for each marble in flight
- **Per-run stats** — For each complete run, view flight time, impact speed, AoA, kinetic energy, vx/vy/vz components, spin rate, arm angle, and hit location
- **Summary statistics** — Mean ± std deviation across all complete runs, with min/max range for Δt only

Use **Expand All / Collapse All**, **Reset**, or individual run checkboxes to manage large datasets. Click the **Vib** dropdown in Stats to switch to vibraphone mode (see below).

### Parameters Panel (top-right)
Controls the chute Bézier curve:

- Adjust P0, CP1, CP2, and P3 coordinates numerically
- Toggle handle visibility with checkboxes
- Enable **Straight line mode** to create a pure ramp (all handles collapse onto P0–P3 line)
- Drag control-point spheres directly in 3D for intuitive curve editing

### Help Panel (bottom-left)
Comprehensive reference including:
- Keyboard controls and camera navigation
- Physics model details
- Statistics glossary with formulas
- Tuning guide and tips
- Chute editor instructions
- Graph interpretation

### Velocity & Acceleration Graphs (floating windows)
Open per run via the **Graph** button in any run entry. Each graph shows:

| Line Style | Metric | Description |
|------------|--------|-------------|
| Solid | speed | Total velocity magnitude √(vx²+vy²+vz²) |
| Dashed | vy | Vertical velocity (m/s); negative while falling |
| Dotted | vz | Forward velocity along chute/arm axis (chute marble only) |
| Short dashes | spin | Surface speed ω × r (chute marble only) |
| Solid | \|a\| | Smoothed acceleration magnitude in Y-Z plane (10 ms window). ~9.81 m/s² during free flight, spikes at impact |

Multiple graphs can be open simultaneously. Close via the × button or toggle in run entry.

### Axes Widget (bottom-right)
3D orientation overlay showing X/Y/Z axes for spatial reference.

## Vibraphone Mode

Click the **Vib** dropdown in the Stats panel to switch from snare mode:

- Each run selects one of 37 vibraphone bars (F3 through E5, spanning two octaves)
- Hit location on the drum determines which note plays via a physical node system
- Default pattern transcribed from Wintergatan's Marble Machine MusicXML score
- Experiment with melodic triggers by adjusting chute geometry

## Tuning Guide

The reference flight time is **450 ms** (theoretical 1 m free-fall). The Δt statistic shows `chute_fly_time − 450 ms`. To achieve a perfect trigger:

1. Click to spawn marble pairs
2. Watch the Summary section — target mean Δt of **0 ± a few ms**
3. Adjust chute handles until both marbles arrive simultaneously
4. Reduce standard deviation (σ) for consistent triggers across runs

### Quick Tuning Tips

| Observation | Adjustment | Effect |
|-------------|------------|--------|
| Δt < 0 (chute arrives early) | Raise P3 or add curve to delay liftoff | Increases slide time, delays arrival |
| Δt > 0 (chute arrives late) | Lower P3 or straighten the curve | Decreases slide time, speeds up chute marble |
| High standard deviation | Refine curve shape for smoother descent | Reduces run-to-run variance |

Use **Straight line mode** as a starting point, then sculpt with Bézier handles for fine control.

## Statistics Glossary

| Term | Formula/Definition | Notes |
|------|-------------------|-------|
| **Δt** | `chute_fly_time − 450 ms` | Target: 0 ms (negative = early, positive = late) |
| **fly** | Time from spawn to snare contact | In seconds; nearly constant for drop marble (~0.45 s) |
| **spd** | \|v\| at impact | Speed magnitude in m/s |
| **AoA** | Angle between velocity and snare normal | 0° = perpendicular (ideal), 90° = grazing |
| **KE** | ½mv² in millijoules | Marble mass: 14 g |
| **vx/vy/vz** | Velocity components at impact | y is vertical (down = negative) |
| **spin** | ω × r surface speed | m/s; indicates rotational energy at impact |
| **slide** | Contact duration on chute | Only for chute marble; determines lift-off timing |
| **liftoff spd** | √(vy² + vz²) at lift-off | Determines post-chute trajectory |

## Project Structure

```
src/
├── components/
│   ├── chute.rs        — Bézier mesh with adaptive tessellation
│   └── snare.rs        — Snare drum with counterweighted pivot arm
├── resources/
│   ├── constants.rs    — Physical and visual constants
│   ├── chute_params.rs — Bézier control points and editor state
│   ├── vibraphone_params.rs — Vibraphone bar configuration
│   └── marble_runs.rs  — Per-run data model (HitRecord, RunHistory)
├── systems/
│   ├── setup.rs        — Scene initialization
│   ├── camera.rs       — Orbit/pan/zoom controls
│   ├── chute_editor.rs — Parameters UI + curve rebuild
│   ├── chute_handles.rs — Handle sphere picking and dragging
│   ├── hud.rs          — Stats panel, help panel, summary statistics
│   ├── instrument.rs   — Snare/vibraphone instruments
│   ├── marble_graph.rs — Per-run velocity graph rendering
│   ├── marble.rs       — Spawning, trails, impact data collection
│   ├── programming_wheel.rs — Song programming interface
│   ├── axes.rs         — 3D axis orientation widget
│   └── sound.rs        — Impact sound synthesis (WAV/Web Audio)
├── main.rs             — Application entry point
└── Trunk.toml          — WASM build configuration
```

## Chute Editor

The chute is a cubic Bézier curve in the Y–Z plane (height × depth). Coordinates are relative to the snare top-face centre.

| Point | Description |
|-------|-------------|
| **P0** (entry) | Top spawn point where marbles begin |
| **CP1** (handle 1) | First control point; shapes curvature near entry |
| **CP2** (handle 2) | Second control point; shapes curvature near exit |
| **P3** (exit) | Bottom endpoint where marbles leave the chute |

Enable **Straight line mode** to create a pure ramp by collapsing CP1/CP2 onto the P0–P3 line. Drag handle spheres directly in 3D for intuitive curve editing.

## Physics Model

All objects use Rapier3D rigid-body physics:

| Parameter | Value |
|-----------|-------|
| Marble diameter | 20 mm (radius = 10 mm) |
| Marble mass | 14 g |
| Restitution (steel-on-steel) | 0.60 |
| Friction (steel-on-steel) | 0.18 |
| Chute restitution | 0.35 |
| Chute friction | 0.20 |
| Drop height | 1.0 m above snare top-face |
| Snare diameter | 14″ (356 mm) |
| Snare depth | 5.5″ (140 mm) |

## License

MIT License — see LICENSE for details.

## Acknowledgments

Based on the Wintergatan Marble Machine design. The vibraphone mode uses notes from the original MusicXML score.
