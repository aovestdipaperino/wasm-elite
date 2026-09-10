# wasm-elite

A gameplay-faithful reimplementation of the 1984 BBC Micro game **Elite** (disc version) in Rust,
compiled to WebAssembly and playable in a browser. No emulator, no original binary: the game rules,
3D wireframe pipeline, scanner, market and galaxy are rewritten in Rust using Mark Moxon's
[annotated disassembly](https://elite.bbcelite.com) as the reference. The galaxy, market and planet
descriptions come from the bit-exact [`elite-core`](https://github.com/aovestdipaperino/elite-galaxy) crate.

Elite was written by Ian Bell and David Braben. This is an unofficial fan project; ship designs,
names and text remain the property of their owners.

Live at <https://wasm-elite.netlify.app>.

## Play

Run `./build.sh` (needs `wasm-bindgen`, `wasm-opt` and the `wasm32-unknown-unknown` target), then
serve `web/` with any static server, for example `python3 -m http.server -d web 8765`, and open
<http://localhost:8765/>.

| Key | Action |
| --- | --- |
| `S` / `X` (or `↑` / `↓`) | dive / climb |
| `<` / `>` (or `←` / `→`) | roll left / right |
| `Space` / `?` | speed up / slow down |
| `A` | fire lasers |
| `J` | in-system jump (hold; only with no ships nearby) |
| `H` | hyperspace to the selected system |
| `F1` `F2` `F3` `F4` | front, rear, left, right view |
| `1` | launch when docked |
| `5` / `6` | galactic chart / short range chart (arrows move the cursor, `O` homes it) |
| `7` / `8` / `9` / `0` | data on system / market / status / inventory |
| `4` | equip ship (fuel and missiles for now) |
| Market: `↑` `↓` then `B` / `S` | select an item, buy, sell |

Dock by flying into the station's slot slowly, nose first, with your roll matched to the slot. The
commander is saved to the browser's localStorage each time you dock.

## Layout

```
crates/elite-math     fixed-point vectors, orientation matrices, TIDY, RNG, trig tables
crates/elite-ships    ship blueprints generated from the disassembly by scripts/extract_ships.py
crates/elite-sim      the game: flight model, tactics, combat, docking, hyperspace, trading
crates/elite-render   projection, wireframe, planet and sun, scanner, dashboard, charts, text
crates/elite-wasm     wasm-bindgen glue exposing step/keys/frame buffers
web/                  index.html, main.js (canvas + keyboard shell), netlify.toml
docs/superpowers      design spec and milestone plans
```

`cargo test --workspace` runs the unit and scenario tests natively; the browser is only needed for
the final wasm build.

## Roadmap

- **M1 (this release):** title, launch, flight, hyperspace, charts, market, pirates, lasers, manual docking, save/load.
- **M2:** docking computer, all equipment, police and legal status, the full ship set, sound.
- **M3:** the Constrictor and Thargoid missions, galactic hyperdrive, Thargoids, rating progression, polish.
