use elite_math::{Orient, Vec3};
use serde::{Deserialize, Serialize};

use crate::ship::{ShipKind, ShipSlot};

pub const MAX_SLOTS: usize = 12;

/// Everything near the player: the planet, its sun, the station (if any) and up to
/// [`MAX_SLOTS`] ships.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Universe {
    pub planet: Vec3,
    pub sun: Vec3,
    pub station: Option<ShipSlot>,
    pub ships: Vec<ShipSlot>,
}

impl Universe {
    pub fn empty() -> Self {
        Universe {
            planet: Vec3::new(0, 0, crate::STATION_ORBIT),
            sun: Vec3::new(0, 0, crate::STATION_ORBIT + crate::SUN_DISTANCE),
            station: None,
            ships: Vec::new(),
        }
    }

    /// The universe just after launch: the station right behind the player with its slot
    /// facing the way the player is going, and the planet ahead.
    pub fn at_launch() -> Self {
        let mut u = Universe::empty();
        let mut station = ShipSlot::new(
            ShipKind::Coriolis,
            Vec3::new(0, 0, -600),
            Orient::identity(),
        );
        station.roll = 2;
        u.station = Some(station);
        u
    }

    /// Add a ship if a slot is free.
    pub fn spawn(&mut self, slot: ShipSlot) -> bool {
        if self.ships.len() >= MAX_SLOTS {
            return false;
        }
        self.ships.push(slot);
        true
    }

    pub fn hostiles(&self) -> usize {
        self.ships
            .iter()
            .filter(|s| s.ai.hostile && s.is_alive())
            .count()
    }

    pub fn station_distance(&self) -> Option<u32> {
        self.station.map(|s| s.distance())
    }
}
