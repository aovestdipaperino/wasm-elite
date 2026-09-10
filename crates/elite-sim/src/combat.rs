//! Lasers, damage, kills and death.

use crate::commander::LASER_PULSE;
use crate::game::{Game, Mode, View};

/// Laser temperature above which it will not fire.
pub const LASER_OVERHEAT: u8 = 242;

impl Game {
    /// The laser power for the current view (0 = no laser fitted).
    pub fn current_laser(&self) -> u8 {
        self.commander.equipment.lasers[self.player.view.index()]
    }

    pub(crate) fn player_fire(&mut self, trigger: bool) {
        if !trigger || self.mode != Mode::Flight {
            return;
        }
        let power = self.current_laser();
        if power == 0 || self.player.laser_temp > LASER_OVERHEAT || self.player.laser_cooldown > 0 {
            return;
        }
        let is_pulse = power == LASER_PULSE;
        self.player.laser_cooldown = if is_pulse { 4 } else { 1 };
        self.player.laser_temp = self.player.laser_temp.saturating_add(8);
        self.player.laser_fired = true;
        let damage = power & 0x7F;

        // Find the nearest ship under the sights in the current view.
        let view = self.player.view;
        let mut best: Option<(i32, usize)> = None;
        for (i, s) in self.universe.ships.iter().enumerate() {
            if !s.is_alive() {
                continue;
            }
            let (x, y, z) = view_coords(s.pos, view);
            if z <= 0 || z > crate::LASER_RANGE {
                continue;
            }
            let area = s.blueprint().area as i64;
            // Screen-space radius squared of the target, in the projection used by the renderer.
            let r2 = (area * 256 / z as i64).pow(2).max(16);
            let sx = x as i64 * 256 / z as i64;
            let sy = y as i64 * 256 / z as i64;
            if sx * sx + sy * sy <= r2 && best.is_none_or(|(bz, _)| z < bz) {
                best = Some((z, i));
            }
        }
        if let Some((_, i)) = best {
            self.damage_ship(i, damage);
        }
    }

    /// Apply laser damage to a ship; destroy it and pay the bounty when its energy is gone.
    pub fn damage_ship(&mut self, i: usize, damage: u8) {
        let s = &mut self.universe.ships[i];
        s.hit_flash = true;
        if s.energy > damage {
            s.energy -= damage;
            // Being shot wakes up a peaceful ship.
            if s.ai.active {
                s.ai.hostile = true;
            }
            return;
        }
        s.energy = 0;
        s.exploding = Some(24);
        let bounty = s.blueprint().bounty as u32;
        let hostile = s.ai.hostile || s.kind != crate::ship::ShipKind::Asteroid;
        if hostile {
            self.commander.kills += 1;
        }
        if bounty > 0 {
            self.commander.credits += bounty;
            let whole = bounty / 10;
            let tenth = bounty % 10;
            self.say(format!("Bounty: {whole}.{tenth} Cr"));
        }
    }

    /// Damage from an attacker ahead (`from_front`) or behind.
    pub fn damage_player(&mut self, from_front: bool, damage: u8) {
        if self.mode != Mode::Flight {
            return;
        }
        let p = &mut self.player;
        p.damage_flash = 4;
        let shield = if from_front {
            &mut p.fore_shield
        } else {
            &mut p.aft_shield
        };
        if *shield >= damage {
            *shield -= damage;
            return;
        }
        let rest = damage - *shield;
        *shield = 0;
        if p.energy > rest {
            p.energy -= rest;
        } else {
            self.die();
        }
    }
}

/// Rotate a player-frame position into the coordinates of the given cockpit view,
/// where +z is "out of that window".
pub fn view_coords(p: elite_math::Vec3, view: View) -> (i32, i32, i32) {
    match view {
        View::Front => (p.x, p.y, p.z),
        View::Rear => (-p.x, p.y, -p.z),
        View::Left => (p.z, p.y, -p.x),
        View::Right => (-p.z, p.y, p.x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{Input, Key};
    use crate::ship::{ShipKind, ShipSlot};
    use elite_math::{Orient, Vec3};

    fn in_flight() -> Game {
        let mut g = Game::new();
        g.dock_at_station();
        g.launch();
        g.universe.station = None;
        g.universe.planet = Vec3::new(0, 0, 5_000_000);
        g.universe.sun = Vec3::new(0, 0, 9_000_000);
        g.player.speed = 0;
        g
    }

    #[test]
    fn weak_pirate_ahead_dies_from_one_shot() {
        let mut g = in_flight();
        let mut s = ShipSlot::new(
            ShipKind::Sidewinder,
            Vec3::new(0, 0, 1000),
            Orient::facing_player(),
        );
        s.energy = 1;
        s.ai.hostile = true;
        g.universe.ships.push(s);
        let mut input = Input::new();
        input.set(Key::Fire, true);
        g.step(&input);
        assert!(g.universe.ships[0].exploding.is_some());
        assert_eq!(g.commander.kills, 1);
        assert_eq!(g.commander.credits, 1000 + 50); // Sidewinder bounty 5.0 Cr
    }

    #[test]
    fn pirate_behind_turns_to_face_us() {
        let mut g = in_flight();
        let mut s = ShipSlot::new(
            ShipKind::Mamba,
            Vec3::new(0, 0, -8000),
            Orient::facing_player(),
        );
        s.ai.hostile = true;
        s.ai.active = true;
        g.universe.ships.push(s);
        let input = Input::new();
        let mut faced = false;
        for _ in 0..300 {
            g.step(&input);
            if g.mode != Mode::Flight {
                break; // it may have killed us, which also proves it turned around
            }
            let s = &g.universe.ships[0];
            let local = s.orient.view_to_local(-s.pos);
            if local.z > 0 && (local.x.abs() + local.y.abs()) * 4 < local.z {
                faced = true;
                break;
            }
        }
        assert!(
            faced || g.mode == Mode::Death,
            "pirate never turned toward the player"
        );
    }

    #[test]
    fn running_out_of_energy_is_death() {
        let mut g = in_flight();
        g.player.fore_shield = 0;
        g.player.energy = 10;
        g.damage_player(true, 20);
        assert_eq!(g.mode, Mode::Death);
    }

    #[test]
    fn shields_absorb_before_energy() {
        let mut g = in_flight();
        g.damage_player(false, 100);
        assert_eq!(g.player.aft_shield, 155);
        assert_eq!(g.player.energy, 255);
        g.damage_player(false, 200);
        assert_eq!(g.player.aft_shield, 0);
        assert_eq!(g.player.energy, 210);
    }
}
