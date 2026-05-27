# MM3Sim — Codebase Guide

Real-time physics simulator (Bevy 0.18 + Avian3D) for tuning a marble-machine snare drum trigger.
Two marbles — one dropping freely, one sliding down a configurable chute — must arrive at the snare simultaneously (Δt = 0 ms).

## Stack

| Layer | Crate |
|---|---|
| Game engine / ECS | `bevy 0.18` |
| Physics (XPBD, 1000 Hz) | `avian3d 0.6` |
| UI panels | `bevy_egui 0.39` + `egui_plot 0.34` |
| WASM target | `wasm-bindgen`, `web-sys` |
| RNG | `rand 0.10` |

Build: `cargo run` (native) · `trunk serve` (WASM dev) · `trunk build --release` (WASM prod).

## Source layout

```
src/
├── main.rs                     – App init, all plugin/system registration
├── components/
│   ├── instrument.rs           – Instrument marker + MarbleSpawner
│   ├── pivot_arm.rs            – Shared pivot-arm spawner (snare + vibraphone)
│   ├── snare.rs                – SnareDrum, PivotArm, SnarePart; spawn_snare()
│   ├── vibraphone.rs           – VibraphoneBar, VibraphoneArm; spawn_vibraphone()
│   ├── programming_wheel.rs    – ProgrammingWheelCylinder; visual mesh spawner
│   └── mod.rs
├── resources/
│   ├── constants.rs            – Every tuned physics/geometry constant (single source of truth)
│   ├── programming_wheel_params.rs  – Channel table (CHANNEL_DEFS), WheelNote, ProgrammingWheelParams
│   ├── chute_params.rs         – ChuteParams + ChuteGeometry (3-part curve)
│   ├── marble_runs.rs          – HitRecord, Run, RunHistory (all hit data lives here)
│   ├── vibraphone_params.rs    – VibraphoneParams (per-rebuild tuning)
│   ├── snare_params.rs         – SnareParams.pos (XYZ offset)
│   ├── marble_collisions.rs    – MarbleCollisions toggle
│   ├── stats_intake.rs         – StatsIntake toggle (enables path/sample recording)
│   ├── layers.rs               – GameLayer enum for Avian collision filtering
│   └── mod.rs
└── systems/
    ├── setup.rs                – One-shot scene initialisation (lights, floor)
    ├── camera.rs               – Orbit/pan/zoom (right-drag / mid-drag / scroll)
    ├── instrument.rs           – detect_instrument_hits + record_instrument_hits
    ├── marble.rs               – spawn_marble(), flight timers, despawn, AutoSpawn
    ├── marble_graph.rs         – Per-run velocity graphs + ghost marble rendering
    ├── hud.rs                  – Stats panel, run history, help panel (egui)
    ├── chute_editor.rs         – Chute/snare/vibraphone parameter UI + rebuild triggers
    ├── chute_handles.rs        – 3D drag handles for Bézier control points
    ├── programming_wheel.rs    – Wheel rotation, beat detection, piano-roll editor UI
    ├── sound.rs                – Native WAV synthesis + Web Audio API fallback
    ├── axes.rs                 – Corner orientation overlay (XYZ axes widget)
    └── mod.rs
```

## Key concepts

### Channels (`WHEEL_CH_*`)
Every marble has a **spawn channel** — an index into `CHANNEL_DEFS` in `programming_wheel_params.rs`.
The channel encodes: instrument target, display name, colour, and XZ jitter.

| Channel | Constant | Target |
|---|---|---|
| 0 | `WHEEL_CH_CHUTE` | Ghost snare — marble enters via chute |
| 1 | `WHEEL_CH_DROP` | Direct snare drop (centre) |
| 2–7 | — | Direct snare drops (±2/4/6 cm X offsets) |
| 8–44 | `WHEEL_CH_VIB_FIRST` + bar_idx | Vibraphone bar 0–36 (F3–F6) |

To add a new instrument: append a `ChannelDef` to `CHANNEL_DEFS`, spawn an `Instrument` entity
with the matching channel, and handle its spawn position in `sync_instrument_spawners`.

### Instrument hit pipeline (FixedUpdate, after physics)
```
capture_prev_velocity   (before PhysicsSystems::First)
    ↓
advance_flight_timers
    ↓
detect_instrument_hits  → InstrumentHits (event bus)
    ↓
record_instrument_hits  → RunHistory
    ↓
play_instrument_sounds
    ↓
record_marble_samples / record_marble_paths  (when StatsIntake enabled)
```

### Rebuilding instruments
`chute_editor.rs` contains `rebuild_snare_system`, `rebuild_chute_system`, and
`rebuild_vibraphone_system`. Each runs in `Update`, checks if its params resource is
`Changed`, despawns the old assembly by marker component (`SnarePart`, `ChuteSegment`,
`VibraphoneEntity`), then re-spawns everything fresh. This is intentional and keeps
spawning logic simple at the cost of a one-frame gap on change.

### Avian physics
- Fixed timestep: 1000 Hz (`SIMULATION_TPS`).
- Marble–marble collisions are OFF by default (`MarbleCollisions`); toggle in the UI.
- `SweptCcd` enabled on every marble to prevent tunnelling.
- Collision events use `CollisionEventsEnabled` on instruments; marbles use `MessageReader<CollisionStart>`.

### Programming wheel
`ProgrammingWheelParams::notes` is the sequence. `rotate_programming_wheel_system` advances
`angle` and populates `pending_spawns` each frame. `programming_wheel_spawn_system` drains
`pending_spawns` and spawns marbles via `spawn_marble()`. Spawner world-positions are
maintained by `sync_instrument_spawners` which runs first each Update frame.

## Constants discipline
All physics/geometry numbers live in `src/resources/constants.rs` — add there, never inline.
UI layout constants (window sizes, offsets) live at the top of their respective system file.

## No tests
There is no test suite. The physics constants are carefully tuned; do not change them
without re-validating Δt against physical measurements.
