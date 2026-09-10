//! Random encounters.

use elite_math::{Orient, Vec3};

use crate::game::Game;
use crate::ship::{ShipKind, ShipSlot};
use crate::universe::MAX_SLOTS;

impl Game {
    pub(crate) fn maybe_spawn(&mut self) {
        if self.mode != crate::game::Mode::Flight || self.player.jumping {
            return;
        }
        // Retire dead or distant ships.
        self.universe.ships.retain(|s| {
            let gone = s.exploding == Some(0);
            let far = s.pos.max_abs() > crate::DESPAWN_DISTANCE as u32;
            !(gone || far)
        });
        for s in &mut self.universe.ships {
            if let Some(t) = &mut s.exploding {
                *t -= 1;
            }
        }
        // No pirates in the station's safe zone.
        let near_station = self
            .universe
            .station_distance()
            .is_some_and(|d| d < 600_000);
        if near_station || self.universe.ships.len() >= MAX_SLOTS - 3 {
            return;
        }
        let roll = self.rng.next_u8();
        if roll < 2 && self.universe.hostiles() == 0 {
            let n = 1 + self.rng.below(3);
            for _ in 0..n {
                self.spawn_pirate();
            }
        } else if roll == 255 {
            self.spawn_asteroid();
        }
    }

    fn spawn_point(&mut self) -> Vec3 {
        let x = self.rng.next_i8() as i32 * 64;
        let y = self.rng.next_i8() as i32 * 64;
        let z = 24_000 + self.rng.next_u8() as i32 * 32;
        // Half the time they come from behind.
        if self.rng.next_u8() & 1 == 1 {
            Vec3::new(x, y, -z)
        } else {
            Vec3::new(x, y, z)
        }
    }

    pub fn spawn_pirate(&mut self) {
        let kind = ShipKind::PIRATES[self.rng.below(ShipKind::PIRATES.len() as u8) as usize];
        let pos = self.spawn_point();
        let mut s = ShipSlot::new(kind, pos, Orient::facing_player());
        s.ai.hostile = true;
        s.ai.active = true;
        s.speed = kind.blueprint().speed / 2;
        self.universe.spawn(s);
    }

    pub fn spawn_asteroid(&mut self) {
        let pos = self.spawn_point();
        let mut s = ShipSlot::new(ShipKind::Asteroid, pos, Orient::identity());
        s.roll = self.rng.next_i8() >> 4;
        s.pitch = self.rng.next_i8() >> 5;
        s.speed = 4;
        self.universe.spawn(s);
    }
}
