//! The in-flight main loop step: controls, movement, and housekeeping.

use elite_math::{Orient, Vec3};

use crate::game::{Game, Mode, Screen, View};
use crate::input::{Input, Key};
use crate::{MAX_SPEED, PLANET_RADIUS};

/// Maximum roll and pitch rates (in 1/512 rad per step, so half the original's per-frame values at
/// twice the frame rate).
pub const MAX_ROLL: i8 = 31;
pub const MAX_PITCH: i8 = 8;

impl Game {
    pub(crate) fn step_flight(&mut self, input: &Input) {
        self.read_controls(input);
        self.handle_screen_keys(input, false);
        if input.pressed(Key::ViewFront) {
            self.screen = Screen::Space;
            self.player.view = View::Front;
        }
        if input.pressed(Key::ViewRear) {
            self.screen = Screen::Space;
            self.player.view = View::Rear;
        }
        if input.pressed(Key::ViewLeft) {
            self.screen = Screen::Space;
            self.player.view = View::Left;
        }
        if input.pressed(Key::ViewRight) {
            self.screen = Screen::Space;
            self.player.view = View::Right;
        }
        if self.screen == Screen::GalacticChart || self.screen == Screen::ShortRangeChart {
            self.chart_keys(input);
        }
        if input.pressed(Key::Hyperspace) {
            self.start_hyperspace();
        }
        self.player.jumping = input.held(Key::Jump) && self.can_jump();

        move_universe(self);
        self.step_ships();
        self.player_fire(input.held(Key::Fire) && self.screen == Screen::Space);
        self.housekeeping();
        self.check_station();
        self.check_planet();
        self.tick_hyperspace();
        self.maybe_spawn();
    }

    /// Roll/pitch with the original's damping: rates build by one per step while a key is held and
    /// decay by one per step otherwise. Speed changes by one per step.
    fn read_controls(&mut self, input: &Input) {
        let p = &mut self.player;
        if input.held(Key::RollRight) {
            p.alpha = (p.alpha + 1).min(MAX_ROLL);
        } else if input.held(Key::RollLeft) {
            p.alpha = (p.alpha - 1).max(-MAX_ROLL);
        } else {
            p.alpha -= p.alpha.signum();
        }
        if input.held(Key::PitchUp) {
            p.beta = (p.beta + 1).min(MAX_PITCH);
        } else if input.held(Key::PitchDown) {
            p.beta = (p.beta - 1).max(-MAX_PITCH);
        } else {
            p.beta -= p.beta.signum();
        }
        if input.held(Key::SpeedUp) {
            p.speed = (p.speed + 1).min(MAX_SPEED);
        }
        if input.held(Key::SlowDown) {
            p.speed = p.speed.saturating_sub(1);
        }
    }

    /// Energy regeneration, shield recharge, laser cooling, altitude, cabin temperature.
    fn housekeeping(&mut self) {
        let p = &mut self.player;
        let regen = if self.commander.equipment.energy_unit {
            2
        } else {
            1
        };
        if p.energy < 255 {
            p.energy = p.energy.saturating_add(regen);
        } else if self.step.is_multiple_of(2) {
            // Full energy trickles into whichever shield is lower.
            if p.fore_shield <= p.aft_shield {
                if p.fore_shield < 255 {
                    p.fore_shield += 1;
                    p.energy -= 1;
                }
            } else if p.aft_shield < 255 {
                p.aft_shield += 1;
                p.energy -= 1;
            }
        }
        p.laser_temp = p.laser_temp.saturating_sub(1);
        p.laser_cooldown = p.laser_cooldown.saturating_sub(1);
        p.laser_fired = false;
        p.damage_flash = p.damage_flash.saturating_sub(1);

        // Altitude: distance to the planet surface in 4096-unit steps.
        let d = self.universe.planet.length() as i64 - PLANET_RADIUS as i64;
        p.altitude = (d >> 12).clamp(0, 255) as u8;

        // Cabin temperature rises near the sun.
        let sd = self.universe.sun.length() as i64;
        let heat = ((crate::SUN_RADIUS as i64 * 4 * 255) / sd.max(1)).clamp(0, 255) as u8;
        p.cabin_temp = heat.max(30);
        if p.cabin_temp == 255 {
            self.die();
        }
    }

    fn check_planet(&mut self) {
        if self.mode == Mode::Flight && self.universe.planet.length() < PLANET_RADIUS as u32 {
            self.die();
        }
    }

    /// The in-system jump works only with no ships around and away from the planet and sun.
    pub fn can_jump(&self) -> bool {
        self.universe
            .ships
            .iter()
            .all(|s| !s.is_alive() || s.kind == crate::ship::ShipKind::Canister)
            && self.universe.planet.length() > (2 * PLANET_RADIUS) as u32 + 400_000
            && self.universe.sun.length() > (3 * crate::SUN_RADIUS) as u32
            && self.player.hyper_countdown == 0
    }

    pub fn die(&mut self) {
        if self.mode != Mode::Death {
            self.mode = Mode::Death;
            self.screen = Screen::Space;
            self.death_timer = 0;
            self.player.energy = 0;
            self.message = None;
        }
    }
}

/// Move every object by the player's speed and rotation (the MVEIT step for the player's part),
/// then apply each ship's own motion.
pub fn move_universe(g: &mut Game) {
    let alpha = g.player.alpha;
    let beta = g.player.beta;
    // Half the speed per step, alternating rounding so odd speeds still move.
    let mut dz = (g.player.speed as i32 + (g.step & 1) as i32) >> 1;
    if g.player.jumping {
        dz *= 256;
    }
    let translate = Vec3::new(0, 0, -dz);
    let tidy = g.step.is_multiple_of(8);

    let rotate_pos = |p: &mut Vec3| {
        *p = Orient::rotate_point_by_player(*p + translate, alpha, beta);
    };
    rotate_pos(&mut g.universe.planet);
    rotate_pos(&mut g.universe.sun);
    if let Some(st) = &mut g.universe.station {
        // The station spins about its own axis.
        st.orient.rotate_own(st.roll, 0);
        rotate_pos(&mut st.pos);
        st.orient.rotate_by_player(alpha, beta);
        if tidy {
            st.orient.tidy();
        }
    }
    for s in &mut g.universe.ships {
        if s.is_alive() {
            s.orient.rotate_own(s.roll, s.pitch);
            let own = Vec3::from_fp(s.orient.nosev, (s.speed as i32 + (g.step & 1) as i32) >> 1);
            s.pos = s.pos + own;
        }
        rotate_pos(&mut s.pos);
        s.orient.rotate_by_player(alpha, beta);
        if tidy {
            s.orient.tidy();
        }
    }
}
