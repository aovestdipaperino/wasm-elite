//! Enemy behaviour: turn toward the player, shoot, run away when hurt.

use elite_math::UNIT;

use crate::game::Game;

/// Own roll/pitch rate used when turning toward a target.
const TURN: i8 = 4;

impl Game {
    /// Run tactics for every ship. Each ship thinks once every four steps.
    pub(crate) fn step_ships(&mut self) {
        let step = self.step;
        let mut damage_to_player: Vec<(i32, u8)> = Vec::new();
        for (i, s) in self.universe.ships.iter_mut().enumerate() {
            s.hit_flash = false;
            if !s.is_alive() || !s.ai.active {
                continue;
            }
            if !(step as usize + i).is_multiple_of(4) {
                continue;
            }
            let bp = s.blueprint();
            // Where is the player, in the ship's own frame?
            let to_player = -s.pos;
            let local = s.orient.view_to_local(to_player);
            let dist = to_player.length().max(1) as i64;

            if s.energy < bp.energy / 4 {
                s.ai.fleeing = true;
            }
            let want_away = s.ai.fleeing;

            // Steer: roll so the target lies in the roof plane, then pitch toward it.
            let target_x = if want_away { -local.x } else { local.x };
            let target_y = if want_away { -local.y } else { local.y };
            let target_z = if want_away { -local.z } else { local.z };
            s.roll = if target_x.abs() as i64 * 8 > dist {
                TURN * target_x.signum() as i8
            } else {
                0
            };
            s.pitch = if target_y.abs() as i64 * 32 > dist || target_z < 0 {
                if target_y == 0 {
                    TURN
                } else {
                    TURN * target_y.signum() as i8
                }
            } else {
                0
            };

            // Speed: charge when far, hold off when close.
            let max = bp.speed;
            s.speed = if want_away || dist > 6000 {
                max
            } else if dist > 2500 {
                max / 2
            } else {
                (max / 4).max(1)
            };

            // Fire when lined up and in range.
            let lined_up =
                target_z > 0 && (local.x.abs() as i64 + local.y.abs() as i64) * 12 < dist;
            if !want_away && lined_up && dist < crate::LASER_RANGE as i64 && bp.laser > 0 {
                let chance = (200 - (dist / 96).min(160)) as u8;
                if self.rng.chance(chance) {
                    let dmg = bp.laser * 4 + self.rng.below(8);
                    damage_to_player.push((s.pos.z, dmg));
                }
            }
            if s.energy < bp.energy && step.is_multiple_of(8) {
                s.energy += 1;
            }
        }
        for (z, dmg) in damage_to_player {
            self.damage_player(z >= 0, dmg);
        }
        // Collisions with the player.
        let mut crash = None;
        for (i, s) in self.universe.ships.iter().enumerate() {
            if s.is_alive() && s.pos.length() < (s.blueprint().radius() + 60) as u32 {
                crash = Some(i);
            }
        }
        if let Some(i) = crash {
            let e = self.universe.ships[i].energy;
            self.universe.ships[i].exploding = Some(24);
            self.damage_player(true, e.max(64));
        }
        let _ = UNIT;
    }
}
