//! The Elite simulation: everything about the game except drawing and I/O.
//!
//! Coordinates follow the original: the player sits at the origin looking down +z,
//! and every object is stored in the player's frame. Positions are in ship units
//! (a Cobra Mk III is about 130 units long). The simulation is stepped at
//! [`STEP_HZ`] from outside.

pub mod combat;
pub mod commander;
pub mod flight;
pub mod game;
pub mod hyperspace;
pub mod input;
pub mod market;
pub mod ship;
pub mod spawn;
pub mod station;
pub mod tactics;
pub mod universe;

pub use commander::{Commander, Equipment, rating};
pub use game::{Game, Mode, Screen, View};
pub use input::{Input, Key};
pub use ship::{AiFlags, ShipKind, ShipSlot};
pub use universe::Universe;

/// Simulation steps per second.
pub const STEP_HZ: u32 = 20;
/// Planet radius in ship units.
pub const PLANET_RADIUS: i32 = 300_000;
/// Distance from planet centre to the station.
pub const STATION_ORBIT: i32 = 4 * PLANET_RADIUS;
/// Sun radius and distance from the planet.
pub const SUN_RADIUS: i32 = 2 * PLANET_RADIUS;
pub const SUN_DISTANCE: i32 = 20 * PLANET_RADIUS;
/// Ships further than this (max axis) leave the universe.
pub const DESPAWN_DISTANCE: i32 = 60_000;
/// Scanner half-range on x and z.
pub const SCANNER_RANGE: i32 = 16_384;
/// Laser range along z.
pub const LASER_RANGE: i32 = 16_384;
/// Cobra Mk III maximum speed.
pub const MAX_SPEED: u8 = 40;
