# M1: First Playable Loop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A browser-playable Elite session: title screen, launch from Lave station, fly, hyperspace, trade, fight pirates, dock, save.

**Architecture:** Pure-Rust workspace. `elite-math` (fixed-point 3D), `elite-ships` (generated blueprints), `elite-sim` (game rules, no I/O), `elite-render` (line-list renderer), `elite-wasm` (wasm-bindgen glue), `web/` (canvas + keyboard shell). The simulation is stepped at a fixed rate from JS; rendering returns typed arrays of line segments and glyphs.

**Tech Stack:** Rust 1.98 edition 2024, wasm-bindgen 0.2, wasm-pack, wasm-opt, plain HTML/JS, Netlify CLI. `elite-core` from `aovestdipaperino/elite-galaxy` as a git dependency.

## Global Constraints

- Workspace authors `["Enzo Lombardi <enzinol@gmail.com>"]`, repository `https://github.com/aovestdipaperino/wasm-elite`, license MIT, edition 2024.
- No floats in the flight model: positions are `i32` (Elite's 24-bit sign-magnitude coords widened), orientation matrices hold `i16` components scaled so 1.0 = 0x6000 like the original.
- Logical screen: 256 wide x 248 high. Space view is rows 0..192, dashboard rows 192..248.
- Only `elite-wasm` and `web/` may reference wasm-bindgen or the browser.
- Every task ends with `cargo test --workspace` green and a commit.

---

### Task 1: Workspace skeleton and elite-math

**Files:** `Cargo.toml`, `crates/elite-math/{Cargo.toml,src/lib.rs,src/vec.rs,src/mat.rs,src/rng.rs,src/tables.rs}`

**Produces:**
- `pub struct Vec3 { pub x: i32, pub y: i32, pub z: i32 }` with `add, sub, scale(shift: u8) -> Vec3, dot_fp(&Fp3) -> i32, length_approx() -> u32` (Elite's max + half min approximation).
- `pub struct Fp3 { pub x: i16, pub y: i16, pub z: i16 }` unit vector scaled by `UNIT: i16 = 0x6000`; `pub struct Orient { pub nosev: Fp3, pub roofv: Fp3, pub sidev: Fp3 }` with `Orient::identity()`, `rotate(roll: i8, pitch: i8)` (MVS4/MVS5 style small-angle rotation), `tidy()` (orthonormalise as TIDY does).
- `pub struct Rng { seed: [u8; 4] }` with `new(seed: u32)`, `next_u8() -> u8` (DORND algorithm).
- `pub fn sin_q(a: u8) -> i16` 256-step sine in Q15; `pub fn atan_q(y: i32, x: i32) -> u8` 0..255 angle.

**Tests:** identity dot products, `tidy` restores orthonormality after 1000 random small rotations within tolerance of 2 units on each axis dot, RNG first bytes match a hand-computed sequence, `sin_q(64) == 0x7FFF`, `sin_q(0) == 0`.

Commit: `feat(math): fixed-point vectors, orientation matrices, rng`

### Task 2: elite-ships crate from generated data

**Files:** `crates/elite-ships/{Cargo.toml,src/lib.rs,src/blueprints.rs}`, `scripts/extract_ships.py` (copied from scratch, reads a Moxon checkout path from argv).

**Produces:**
```rust
pub struct Vertex { pub x: i16, pub y: i16, pub z: i16, pub faces: [u8; 4], pub vis: u8 }
pub struct Edge { pub v1: u8, pub v2: u8, pub faces: [u8; 2], pub vis: u8 }
pub struct Face { pub nx: i16, pub ny: i16, pub nz: i16, pub vis: u8 }
pub struct Blueprint { pub name: &'static str, canisters, scoop, area, max_edges, gun_vertex, explosion, bounty: u16, vis, energy, speed, scale, laser, missiles: u8, vertices: &'static [Vertex], edges: &'static [Edge], faces: &'static [Face] }
pub const CORIOLIS, COBRA_MK_3, SIDEWINDER, MAMBA, KRAIT, ADDER, GECKO, COBRA_MK_1, VIPER, PYTHON, BOA, ANACONDA, ASP_MK_2, FER_DE_LANCE, MORAY, THARGOID, THARGON, CONSTRICTOR, WORM, TRANSPORTER, SHUTTLE, DODO, ASTEROID, BOULDER, SPLINTER, CANISTER, ESCAPE_POD, PLATE, MISSILE: Blueprint;
pub const ALL: &[&Blueprint];
```
Missile lives in `elite-missile.asm`; extractor handles it too.

**Tests:** Coriolis has 16 vertices, 28 edges, 14 faces; Cobra Mk III 28/38/13; every edge's vertex indices are in range; every face index referenced by a vertex or edge is in range.

Commit: `feat(ships): blueprints transcribed from the disc disassembly`

### Task 3: elite-sim core state and flight

**Files:** `crates/elite-sim/{Cargo.toml,src/lib.rs,src/commander.rs,src/universe.rs,src/ship.rs,src/flight.rs,src/input.rs}`

**Produces:**
- `pub struct Commander { name: String, credits: u32 (in 0.1 Cr), fuel: u8 (0.1 LY, max 70), galaxy: u8, seed: elite_core::Seed, present: u8 (system index), target: u8, cargo: [u8; 17], cargo_capacity: u8, legal: u8, kills: u16, energy_unit: bool, ... }` with `Commander::jameson()` and serde `Serialize/Deserialize`.
- `pub struct ShipSlot { pub kind: ShipKind, pub pos: Vec3, pub orient: Orient, pub speed: u8, pub energy: u8, pub roll: i8, pub pitch: i8, pub ai: AiFlags, pub missiles: u8, pub exploding: Option<u8>, pub target_lock: bool }`.
- `pub struct Universe { pub slots: Vec<ShipSlot> (max 13), pub planet: ShipSlot, pub station: Option<ShipSlot>, pub sun: ShipSlot }`.
- `pub enum Key { RollLeft, RollRight, PitchUp, PitchDown, SpeedUp, SlowDown, Fire, Jump, Hyperspace, Dock, View(u8), ... }` and `pub struct Input { held: [bool; N], pressed: [bool; N] }`.
- `Game { commander, universe, mode: Mode, rng, player: PlayerState { alpha: i8, beta: i8, delta: u8, energy, shields fore/aft, view } }`.
- `Game::step(&mut self, input: &Input)` runs one main-loop iteration: read controls with Elite's damping (alpha/beta increment 1 per step, max 8 pitch / 31 roll as in DK4), move every slot (MVEIT: rotate by player alpha/beta, translate by player speed and own speed, apply own roll/pitch, tidy every 8 steps), fire lasers, energy regeneration.

**Tests:** a ship dead ahead at z=10000 with player speed 40 for 10 steps ends at z=9600; a ship at (0,0,1000) after player rolls right 256 steps returns near its start; energy climbs to max when idle.

Commit: `feat(sim): commander, universe, flight model`

### Task 4: Stations, docking, launch, hyperspace, charts data

**Files:** `crates/elite-sim/src/{station.rs,hyperspace.rs}`

**Produces:** `Game::launch()` places the player outside a Coriolis rotating slowly and sets `Mode::Flight`; `Game::check_dock(&mut self)` docks when the player is within the station's docking slot volume with `nosev.z < -0x6000+tolerance`, speed <= 24, roll within tolerance, else `Mode::Death` on collision; `Game::hyperspace()` validates distance <= fuel, deducts fuel, generates a fresh universe near the destination planet with the station on the far side, updates `present`. `Game::galaxy()` returns the 256 systems via `elite_core::galaxy(n)`; `distance_ly(a, b) -> u16` as the original (4 * sqrt(dx^2 + (dy/2)^2) in 0.1 LY).

**Tests:** launch then immediately `check_dock` is false; flying straight in reversed docks; Lave to Zaonce distance is 9.4 LY (94); hyperspace with insufficient fuel is rejected.

Commit: `feat(sim): docking, launch, hyperspace`

### Task 5: Trading and equipment (M1 subset)

**Files:** `crates/elite-sim/src/market.rs`

**Produces:** `Game::market_prices() -> [Quote; 17]` (from elite-core with the system's fluctuation byte), `buy(item, qty) -> Result<(), TradeError>`, `sell(item, qty) -> Result<(), TradeError>` with cargo capacity 20 t, `buy_fuel()`, `buy_missile()`. Station stock decrements on buy and increments on sell per the original.

**Tests:** buy 3 t of food at Lave then sell them returns to the same credits minus zero (same price at same station); buying beyond capacity errors; fuel tops out at 7.0 LY.

Commit: `feat(sim): market trading`

### Task 6: Pirates, tactics, combat, death

**Files:** `crates/elite-sim/src/{spawn.rs,tactics.rs,combat.rs}`

**Produces:** `spawn_pirates()` on a timer as in the main loop (chance rises with distance from station); `tactics(slot)` implements the pirate behaviours: turn toward player using the dot product of the target direction with roofv and sidev, fire when within angle and range, flee when low on energy; player laser: on Fire with laser temperature < 242, ray from origin along nosev hits a slot if its projected screen position is within the sight and it is nearest; damage reduces energy, kills award bounty and `kills += 1`; enemy lasers hit the player when facing and within range, reducing shields then energy; energy 0 -> `Mode::Death`.

**Tests:** a pirate at 1000 units ahead with energy 1 dies after one Fire; a pirate placed behind the player with tactics running turns to face within 200 steps; player energy reaches 0 leads to `Mode::Death`.

Commit: `feat(sim): pirates, tactics, lasers`

### Task 7: elite-render

**Files:** `crates/elite-render/{Cargo.toml,src/lib.rs,src/project.rs,src/wire.rs,src/planet.rs,src/scanner.rs,src/dash.rs,src/font.rs,src/text.rs}`

**Produces:** `pub struct Frame { pub lines: Vec<[i16; 4]>, pub glyphs: Vec<(u8 x, u8 y, u8 ch)>, pub fills: Vec<[i16; 4]> (dashboard bars) }`, `pub fn render(game: &Game) -> Frame`. Projection: `sx = 128 + x * 256 / z` style with Elite's divide-by-z scaling, clip to the space view, wireframe with back-face culling using face normals dotted with the view vector; planet drawn as a circle by the midpoint algorithm segmented into lines, sun as horizontal bands; scanner ellipse with ship dots and stalks; compass toward planet or station; dashboard bars for speed, roll, pitch, energy banks, shields, fuel, laser temperature, altitude, cabin temperature; 8x8 hand-authored font for A-Z, 0-9 and punctuation (no MOS ROM data). Mode-specific screens: title, status, market, data on system, galactic chart, short-range chart, death.

**Tests:** a Cobra at (0,0,5000) facing the player renders more than 20 lines all inside the view; a planet at z far renders a circle of radius 10 +/- 1 at centre; snapshot hash of the title frame is stable.

Commit: `feat(render): wireframe, planets, scanner, dashboard, text`

### Task 8: elite-wasm and web shell

**Files:** `crates/elite-wasm/{Cargo.toml,src/lib.rs}`, `web/{index.html,main.js,style.css,netlify.toml}`, `build.sh`, `README.md`

**Produces:** `#[wasm_bindgen] pub struct Elite` with `new() -> Elite`, `load(json: &str) -> bool`, `save() -> String`, `key(code: u8, down: bool)`, `step() -> ()`, `lines() -> Vec<i16>`, `glyphs() -> Vec<u8>`, `fills() -> Vec<i16>`. Key codes are an enum shared with `elite-sim::Key`. `main.js` maps `KeyboardEvent.code` (KeyS, KeyX, Comma, Period, KeyA, Space, Slash, KeyH, KeyJ, KeyC, F1..F4, Digit1..Digit9 for screens, Enter, Escape) to codes, runs `step` at 25 Hz and draws the last frame each animation frame, scales the 256x248 canvas with integer factors and `image-rendering: pixelated`, saves to `localStorage["wasm-elite.commander"]` whenever docked, offers "New commander" and "Fullscreen".

**Tests:** `cargo test -p elite-wasm` compiles for host; `wasm-pack build` succeeds; manual check in Chrome that the title Cobra rotates and launching works.

Commit: `feat(web): wasm glue and canvas shell`

### Task 9: Deploy

Create Netlify site `wasm-elite`, write `web/.netlify/state.json`, deploy per the spec, add the live URL to README.

Commit: `docs: live URL`

## Self-review

Spec coverage for M1: title (7), launch (4), flight and views (3, 7), scanner/compass (7), hyperspace with fuel (4), system data and charts (7 using 4), market (5), pirates and combat (6), manual docking (4), energy/shields/death (3, 6, 7), save/load (8), deploy (9). Type names are consistent: `Game`, `Commander`, `Universe`, `ShipSlot`, `Orient`, `Fp3`, `Vec3`, `Frame`, `Key`, `Input`.
