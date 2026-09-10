# wasm-elite: gameplay-faithful Elite in Rust/WebAssembly

Date: 2026-09-10. Status: approved.

## Goal

A playable, browser-hosted reimplementation of the BBC Micro disc version of Elite (1984, Bell and
Braben), written in Rust and compiled to WebAssembly. Fidelity target is gameplay-faithful, not
bit-exact: same galaxy, ships, flight feel, combat, trading, equipment and missions, structured as
idiomatic Rust modules. Mark Moxon's annotated disassembly of the disc version is the mechanics
reference. The galaxy, market and planet-description generator is reused from the existing
bit-exact crate `elite-core` in `aovestdipaperino/elite-galaxy`.

Rendering is 2D canvas wireframe at the original 256 x 248 logical resolution (192 lines of space
view plus the dashboard) scaled with crisp pixels. Controls are the original BBC keys with arrow
keys as aliases. Saves live in the browser's localStorage.

## Non-goals

Bit-exact reproduction of the 6502 code, sound-chip emulation, filled polygons, network play.
Touch and gamepad input are deferred past milestone 3.

## Architecture

Cargo workspace, edition 2024, MIT licensed code. Ship designs, names and text are the property of
their copyright holders; the README states the project is a fan reimplementation from the
annotated disassembly and contains no original binary.

```
wasm-elite/
  Cargo.toml              workspace; authors Enzo Lombardi <enzinol@gmail.com>
  crates/elite-math/      fixed-point vectors and 3x3 orientation matrices, TIDY orthonormalisation,
                          Elite's sine/arctan tables, the game RNG, fast multiply/divide helpers
  crates/elite-ships/     const blueprints for every disc-version ship: vertices, edges, faces,
                          hit points, laser power, speed, bounty, cargo, plus the blueprint metadata
  crates/elite-sim/       the game state and rules: Universe with ship slots, flight model, tactics
                          (AI), lasers and missiles, damage and energy, docking and launching,
                          hyperspace, market and cargo, equipment, legal status, rating, missions;
                          depends on elite-core and the two crates above; no I/O
  crates/elite-render/    projection and line clipping, wireframe drawing with hidden-face check,
                          planet and sun, scanner and compass, dashboard bars, 8x8 text font;
                          produces per-frame Vec<Line> and Vec<Glyph> in logical coordinates
  crates/elite-wasm/      wasm-bindgen glue: Game::new(), load(json), save() -> json,
                          key(code, down), tick(dt_ms) -> frame views as typed arrays
  web/                    index.html, main.js (canvas scaling, keyboard, localStorage,
                          requestAnimationFrame), style.css, netlify.toml
  docs/                   this spec and the milestone plans
```

Data flow per frame: main.js collects key state, calls `tick`, the simulation advances one main-loop
step (move ships, run tactics, collisions, lasers, energy and shields, then render), and returns
typed arrays of line segments and glyphs that main.js draws with `imageSmoothingEnabled = false`.
Only `elite-wasm` and `web/` know about the browser; everything else runs and tests natively.

## Milestones

M1, first playable loop: title screen with rotating Cobra Mk III; launch from a Coriolis station;
flight with roll, pitch, speed, front/rear/left/right views; scanner and compass; hyperspace with
fuel and range check; system data, galactic chart and short-range chart; market with buy and sell;
pirates spawn and attack; pulse laser, kills and credits; manual docking; energy, shields and death;
save/load commander in localStorage; deployed to Netlify.

M2: docking computer, all equipment (beam and military lasers, ECM, fuel scoops, escape pod,
energy bomb, extra energy unit, missiles), police Vipers and legal status, the complete disc-version
ship set, sun and planet detail, WebAudio sound effects.

M3: the Constrictor and Thargoid-plans missions, galactic hyperdrive, Thargoids and Thargons,
combat rating progression, Trumbles, rock hermits, asteroid mining, polish.

## Testing

Every crate has unit tests. elite-math checks fixed-point ops against known values and that TIDY
keeps matrices orthonormal. elite-ships asserts vertex, edge and face counts per ship against the
documented blueprint figures. elite-sim runs scenario tests: launch then dock with no input is
impossible but launch, turn 180 degrees and re-enter at the right angle docks; a pirate is destroyed
within N ticks of continuous laser fire; market buy then sell round trip returns to the starting
cargo. elite-render has golden tests that hash the line list of fixed scenes. A headless example
binary plays scripted input for CI. elite-core keeps its bit-exact golden tests upstream.

## Deployment

Build with `wasm-pack build crates/elite-wasm --target web --release`, run `wasm-opt -Oz`, copy the
package into `web/pkg/`, and deploy from `web/` with
`npx -y netlify-cli deploy --prod --dir=. --no-build --message "<what changed>"` after creating the
Netlify site `wasm-elite`. No deploy on git push.

## Risks

Flight feel depends on the fixed-point arithmetic of the original; rotation and TIDY are ported
faithfully rather than replaced by floats. Ship blueprints are transcribed from the disassembly and
cross-checked by count tests. The codebase is large; each milestone gets its own implementation plan
and a code review before the next starts.
