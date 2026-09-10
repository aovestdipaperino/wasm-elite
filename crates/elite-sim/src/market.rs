//! Buying and selling at the station, fuel and missiles.

use elite_core::Quote;

use crate::commander::MAX_FUEL;
use crate::game::{Game, Screen};
use crate::input::{Input, Key};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeError {
    NotDocked,
    InsufficientCredits,
    OutOfStock,
    CargoFull,
    NothingToSell,
}

/// Price of a tenth of a light year, in tenths of a credit.
pub const FUEL_PRICE: u32 = 2;
/// Price of a missile in tenths of a credit.
pub const MISSILE_PRICE: u32 = 300;

impl Game {
    pub fn market_prices(&self) -> [Quote; 17] {
        let mut q = elite_core::market(self.present(), self.commander.fluctuation);
        for (quote, stock) in q.iter_mut().zip(self.commander.stock.iter()) {
            quote.quantity = *stock;
        }
        q
    }

    pub fn buy(&mut self, item: usize, qty: u8) -> Result<(), TradeError> {
        if self.mode != crate::game::Mode::Docked {
            return Err(TradeError::NotDocked);
        }
        let quote = self.market_prices()[item];
        if quote.quantity < qty || qty == 0 {
            return Err(TradeError::OutOfStock);
        }
        let cost = quote.price as u32 * qty as u32;
        if cost > self.commander.credits {
            return Err(TradeError::InsufficientCredits);
        }
        let counts_as_tonnes = item < 13 || item == 16;
        if counts_as_tonnes && self.commander.cargo_used() + qty > self.commander.cargo_capacity() {
            return Err(TradeError::CargoFull);
        }
        self.commander.credits -= cost;
        self.commander.cargo[item] += qty;
        self.commander.stock[item] -= qty;
        Ok(())
    }

    pub fn sell(&mut self, item: usize, qty: u8) -> Result<(), TradeError> {
        if self.mode != crate::game::Mode::Docked {
            return Err(TradeError::NotDocked);
        }
        if self.commander.cargo[item] < qty || qty == 0 {
            return Err(TradeError::NothingToSell);
        }
        let quote = self.market_prices()[item];
        self.commander.credits += quote.price as u32 * qty as u32;
        self.commander.cargo[item] -= qty;
        self.commander.stock[item] = self.commander.stock[item].saturating_add(qty).min(63);
        Ok(())
    }

    /// Fill the tank; returns tenths of a light year bought.
    pub fn buy_fuel(&mut self) -> u8 {
        let need = (MAX_FUEL - self.commander.fuel) as u32;
        let affordable = (self.commander.credits / FUEL_PRICE).min(need) as u8;
        self.commander.fuel += affordable;
        self.commander.credits -= affordable as u32 * FUEL_PRICE;
        affordable
    }

    pub fn buy_missile(&mut self) -> Result<(), TradeError> {
        if self.commander.missiles >= 4 {
            return Err(TradeError::CargoFull);
        }
        if self.commander.credits < MISSILE_PRICE {
            return Err(TradeError::InsufficientCredits);
        }
        self.commander.credits -= MISSILE_PRICE;
        self.commander.missiles += 1;
        Ok(())
    }

    /// Market screen keys: up/down move the cursor, Buy/Sell (or right/left) trade one unit.
    pub(crate) fn market_keys(&mut self, input: &Input) {
        if input.pressed(Key::Up) || input.pressed(Key::PitchDown) {
            self.market_cursor = self.market_cursor.checked_sub(1).unwrap_or(16);
        }
        if input.pressed(Key::Down) || input.pressed(Key::PitchUp) {
            self.market_cursor = (self.market_cursor + 1) % 17;
        }
        let item = self.market_cursor as usize;
        if input.pressed(Key::Buy) || input.pressed(Key::Right) || input.pressed(Key::RollRight) {
            match self.buy(item, 1) {
                Ok(()) => {}
                Err(TradeError::InsufficientCredits) => self.say("Not enough credits"),
                Err(TradeError::OutOfStock) => self.say("Out of stock"),
                Err(TradeError::CargoFull) => self.say("Cargo hold full"),
                Err(_) => {}
            }
        }
        if (input.pressed(Key::Sell) || input.pressed(Key::Left) || input.pressed(Key::RollLeft))
            && self.sell(item, 1) == Err(TradeError::NothingToSell)
        {
            self.say("Nothing to sell");
        }
    }

    /// Equip screen (M1 subset): 1 = fuel, 2 = missile.
    pub(crate) fn equip_keys(&mut self, input: &Input) {
        match input.pressed_digit() {
            Some(1) => {
                let bought = self.buy_fuel();
                self.say(if bought > 0 {
                    "Fuel topped up"
                } else {
                    "Tank already full"
                });
            }
            Some(2) => match self.buy_missile() {
                Ok(()) => self.say("Missile loaded"),
                Err(TradeError::CargoFull) => self.say("Missile pylons full"),
                Err(_) => self.say("Not enough credits"),
            },
            _ => {}
        }
        if input.pressed(Key::Back) {
            self.screen = Screen::Status;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buy_then_sell_round_trip() {
        let mut g = Game::new();
        g.dock_at_station();
        let credits = g.commander.credits;
        g.buy(0, 3).unwrap();
        assert_eq!(g.commander.cargo[0], 3);
        assert!(g.commander.credits < credits);
        g.sell(0, 3).unwrap();
        assert_eq!(g.commander.credits, credits);
        assert_eq!(g.commander.cargo[0], 0);
    }

    #[test]
    fn cargo_capacity_is_enforced() {
        let mut g = Game::new();
        g.dock_at_station();
        g.commander.credits = 1_000_000;
        g.commander.stock[0] = 63;
        assert_eq!(g.buy(0, 21), Err(TradeError::CargoFull));
        g.buy(0, 20).unwrap();
        assert_eq!(g.buy(0, 1), Err(TradeError::CargoFull));
    }

    #[test]
    fn fuel_tops_out() {
        let mut g = Game::new();
        g.dock_at_station();
        g.commander.fuel = 10;
        assert_eq!(g.buy_fuel(), 60);
        assert_eq!(g.commander.fuel, MAX_FUEL);
        assert_eq!(g.commander.credits, 1000 - 120);
        assert_eq!(g.buy_fuel(), 0);
    }
}
