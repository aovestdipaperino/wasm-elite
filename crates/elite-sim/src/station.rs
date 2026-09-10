//! Launching, docking and colliding with the station.

use elite_math::{UNIT, Vec3};

use crate::game::{Game, Mode, Screen};
use crate::universe::Universe;

/// Distance from the station centre at which docking or collision is decided.
pub const DOCK_RADIUS: u32 = 280;
/// Player speed above which entering the slot is a crash.
pub const DOCK_MAX_SPEED: u8 = 24;

/// Why a docking attempt failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockError {
    TooFast,
    WrongApproach,
    MissedSlot,
    RollMisaligned,
}

impl Game {
    /// Leave the station: the universe is rebuilt around the station with the player heading
    /// out of the slot toward the planet.
    pub fn launch(&mut self) {
        self.universe = Universe::at_launch();
        self.player = crate::game::Player::fresh();
        self.player.speed = 12;
        self.mode = Mode::Flight;
        self.screen = Screen::Space;
        self.commander.legal = self.commander.legal.saturating_sub(0);
    }

    /// Check whether the player is entering the slot correctly. Returns `Ok(true)` when docked,
    /// `Ok(false)` when not near the station, `Err` on a collision with the station.
    pub fn try_dock(&self) -> Result<bool, DockError> {
        let Some(st) = &self.universe.station else {
            return Ok(false);
        };
        if st.distance() > DOCK_RADIUS {
            return Ok(false);
        }
        // We must fly into the slot face: the station's nose points at us.
        if st.orient.nosev.z > -(UNIT as i32 * 9 / 10) as i16 {
            return Err(DockError::WrongApproach);
        }
        if self.player.speed > DOCK_MAX_SPEED {
            return Err(DockError::TooFast);
        }
        // Offset from the slot axis: the station centre must be close to dead ahead.
        let local = st.orient.view_to_local(-st.pos);
        if local.x.abs() > 100 || local.y.abs() > 100 {
            return Err(DockError::MissedSlot);
        }
        // The slot is a vertical letterbox in station coordinates; our roof must be (anti)parallel to it.
        if (st.orient.roofv.y as i32).abs() < UNIT as i32 * 17 / 20 {
            return Err(DockError::RollMisaligned);
        }
        Ok(true)
    }

    pub(crate) fn check_station(&mut self) {
        if self.mode != Mode::Flight {
            return;
        }
        match self.try_dock() {
            Ok(false) => {}
            Ok(true) => {
                self.say("Docking complete");
                self.arrive_at_station();
            }
            Err(_) => self.die(),
        }
    }

    /// Dock: switch to the docked screens and request a save.
    pub fn arrive_at_station(&mut self) {
        self.dock_at_station();
    }

    /// Place the station for a freshly generated universe, `orbit` units from the planet with its
    /// slot pointing at the planet.
    pub fn place_station(u: &mut Universe) {
        // Direction from planet to station, chosen so the station is off to one side of the
        // player's approach and its slot faces the planet.
        let dir = elite_math::Fp3::new(
            0,
            (UNIT as i32 * 3 / 5) as i16,
            -(UNIT as i32 * 4 / 5) as i16,
        );
        let offset = Vec3::from_fp(dir, crate::STATION_ORBIT);
        let pos = u.planet + offset;
        let mut o = elite_math::Orient {
            nosev: -dir,
            roofv: elite_math::Fp3::new(
                0,
                (UNIT as i32 * 4 / 5) as i16,
                (UNIT as i32 * 3 / 5) as i16,
            ),
            sidev: elite_math::Fp3::new(UNIT, 0, 0),
        };
        o.tidy();
        let mut st = crate::ship::ShipSlot::new(crate::ship::ShipKind::Coriolis, pos, o);
        st.roll = 2;
        u.station = Some(st);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Input;

    #[test]
    fn launch_does_not_immediately_dock() {
        let mut g = Game::new();
        g.dock_at_station();
        g.launch();
        assert_eq!(g.mode, Mode::Flight);
        assert_eq!(g.try_dock(), Ok(false));
        let input = Input::new();
        for _ in 0..40 {
            g.step(&input);
        }
        assert_eq!(
            g.mode,
            Mode::Flight,
            "flying away from the station must not dock or crash"
        );
    }

    #[test]
    fn flying_into_the_slot_docks() {
        let mut g = Game::new();
        g.dock_at_station();
        g.launch();
        // Turn the station around so we are flying into the slot from the front, dead on.
        let st = g.universe.station.as_mut().unwrap();
        st.pos = Vec3::new(0, 0, 2000);
        st.orient = elite_math::Orient::facing_player();
        st.roll = 0;
        g.player.speed = 20;
        let input = Input::new();
        for _ in 0..400 {
            g.step(&input);
            if g.mode == Mode::Docked {
                return;
            }
            assert_eq!(g.mode, Mode::Flight);
        }
        panic!(
            "never docked; station at {:?}",
            g.universe.station.map(|s| s.pos)
        );
    }

    #[test]
    fn hitting_the_station_sideways_is_fatal() {
        let mut g = Game::new();
        g.dock_at_station();
        g.launch();
        let st = g.universe.station.as_mut().unwrap();
        st.pos = Vec3::new(0, 0, 1500);
        st.orient = elite_math::Orient::identity(); // slot facing away from us
        st.roll = 0;
        g.player.speed = 20;
        let input = Input::new();
        for _ in 0..400 {
            g.step(&input);
            if g.mode == Mode::Death {
                return;
            }
        }
        panic!("expected a crash");
    }
}
