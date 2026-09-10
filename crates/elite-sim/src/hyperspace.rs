//! Hyperspace between systems and the universe that greets the player on arrival.

use elite_core::System;
use elite_math::Vec3;

use crate::game::{Game, Mode, Screen};
use crate::universe::Universe;

pub const HYPER_COUNTDOWN: u8 = 15 * 2;

/// Distance between two systems in tenths of a light year, as the original computes it.
pub fn distance_ly(a: &System, b: &System) -> u16 {
    let dx = (a.x as i32 - b.x as i32).abs();
    let dy = (a.y as i32 - b.y as i32).abs() / 2;
    let d2 = dx * dx + dy * dy;
    (4 * isqrt(d2 as u32)) as u16
}

fn isqrt(n: u32) -> u32 {
    let mut x = (n as f64).sqrt() as u32;
    while x * x > n {
        x -= 1;
    }
    while (x + 1) * (x + 1) <= n {
        x += 1;
    }
    x
}

impl Game {
    /// Begin the hyperspace countdown if the target is in range.
    pub fn start_hyperspace(&mut self) {
        if self.player.hyper_countdown > 0 {
            return;
        }
        if self.commander.target == self.commander.present {
            self.say("Hyperspace: select a target");
            return;
        }
        let d = self.distance_to_target();
        if d > self.commander.fuel as u16 {
            self.say("Hyperspace range exceeded");
            return;
        }
        self.player.hyper_countdown = HYPER_COUNTDOWN;
        let name = self.target().name.to_ascii_uppercase();
        self.say(format!("Hyperspace: {name}"));
    }

    pub(crate) fn tick_hyperspace(&mut self) {
        if self.player.hyper_countdown == 0 || self.mode != Mode::Flight {
            return;
        }
        self.player.hyper_countdown -= 1;
        if self.player.hyper_countdown == 0 {
            self.jump();
        }
    }

    /// Perform the jump: burn fuel, move to the target, and build the arrival universe.
    pub fn jump(&mut self) {
        let d = self.distance_to_target();
        self.commander.fuel = self.commander.fuel.saturating_sub(d as u8);
        self.commander.present = self.commander.target;
        self.chart.x = self.present().x;
        self.chart.y = self.present().y;
        self.restock();
        self.universe = self.arrival_universe();
        self.player.speed = self.player.speed.max(20);
        self.screen = Screen::Space;
    }

    /// A fresh market for this visit.
    pub fn restock(&mut self) {
        self.commander.fluctuation = self.rng.next_u8();
        let quotes = elite_core::market(self.present(), self.commander.fluctuation);
        for (s, q) in self.commander.stock.iter_mut().zip(quotes.iter()) {
            *s = q.quantity;
        }
    }

    fn arrival_universe(&mut self) -> Universe {
        let mut u = Universe::empty();
        let ox = self.rng.next_i8() as i32 * 2048;
        let oy = self.rng.next_i8() as i32 * 2048;
        u.planet = Vec3::new(ox, oy, 10 * crate::PLANET_RADIUS);
        u.sun = u.planet + Vec3::new(-ox * 4, oy * 6, crate::SUN_DISTANCE);
        Game::place_station(&mut u);
        u
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lave_to_zaonce_is_5_6_ly() {
        let g = Game::new();
        let lave = g.systems.iter().position(|s| s.name == "Lave").unwrap();
        let zaonce = g.systems.iter().position(|s| s.name == "Zaonce").unwrap();
        assert_eq!(distance_ly(&g.systems[lave], &g.systems[zaonce]), 56);
    }

    #[test]
    fn out_of_range_jump_is_rejected() {
        let mut g = Game::new();
        g.dock_at_station();
        g.launch();
        g.commander.fuel = 10;
        let zaonce = g.systems.iter().position(|s| s.name == "Zaonce").unwrap();
        g.commander.target = zaonce as u8;
        g.start_hyperspace();
        assert_eq!(g.player.hyper_countdown, 0);
        g.commander.fuel = 70;
        g.start_hyperspace();
        assert!(g.player.hyper_countdown > 0);
        let input = crate::input::Input::new();
        for _ in 0..HYPER_COUNTDOWN as usize + 1 {
            g.step(&input);
        }
        assert_eq!(g.present().name, "Zaonce");
        assert_eq!(g.commander.fuel, 70 - 56);
        assert!(g.universe.station.is_some());
    }
}
