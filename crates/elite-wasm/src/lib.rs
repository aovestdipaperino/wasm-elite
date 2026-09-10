//! The browser-facing API. The shell owns a canvas and a keyboard; this crate owns the game.
//!
//! Frames are exchanged as flat typed arrays so no per-primitive JS objects are allocated:
//! lines are 5 x i16 (x1, y1, x2, y2, colour), fills 5 x i16 (x, y, w, h, colour), dots 3 x i16
//! (x, y, colour) and glyphs 4 x i16 (x, y, glyph index, colour).

use elite_render::{Frame, Renderer};
use elite_sim::{Game, Input, Key};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Elite {
    game: Game,
    input: Input,
    renderer: Renderer,
    frame: Frame,
}

impl Default for Elite {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl Elite {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Elite {
        Elite {
            game: Game::new(),
            input: Input::new(),
            renderer: Renderer::new(),
            frame: Frame::default(),
        }
    }

    /// Load a commander saved with [`Elite::save`]. Returns false if the JSON is not understood.
    pub fn load(&mut self, json: &str) -> bool {
        match serde_json::from_str::<elite_sim::Commander>(json) {
            Ok(c) => {
                self.game = Game::with_commander(c);
                true
            }
            Err(_) => false,
        }
    }

    /// The commander record as JSON.
    pub fn save(&self) -> String {
        serde_json::to_string(&self.game.commander).unwrap_or_default()
    }

    /// Start over with Commander Jameson.
    pub fn reset(&mut self) {
        self.game = Game::new();
    }

    /// Report a key transition. `code` is a [`Key`] discriminant.
    pub fn key(&mut self, code: u8, down: bool) {
        if let Some(k) = Key::from_code(code) {
            self.input.set(k, down);
        }
    }

    /// Advance the simulation one step and render. Returns true when the shell should persist
    /// the commander (the player just docked).
    pub fn step(&mut self) -> bool {
        self.game.step(&self.input);
        self.input.end_step();
        self.frame = self.renderer.render(&self.game);
        self.game.save_requested
    }

    pub fn lines(&self) -> Vec<i16> {
        self.frame.lines.iter().flatten().copied().collect()
    }

    pub fn fills(&self) -> Vec<i16> {
        self.frame.fills.iter().flatten().copied().collect()
    }

    pub fn dots(&self) -> Vec<i16> {
        self.frame.dots.iter().flatten().copied().collect()
    }

    pub fn glyphs(&self) -> Vec<i16> {
        self.frame.glyphs.iter().flatten().copied().collect()
    }

    pub fn flash(&self) -> bool {
        self.frame.flash
    }

    /// The 8x8 font, 64 glyphs of 8 rows each, for the shell to rasterise once.
    pub fn font(&self) -> Vec<u8> {
        elite_render::FONT.iter().flatten().copied().collect()
    }

    /// A short status string for the page title: mode and system.
    pub fn status(&self) -> String {
        format!("{:?} at {}", self.game.mode, self.game.present().name)
    }

    pub fn is_docked(&self) -> bool {
        self.game.mode == elite_sim::Mode::Docked
    }

    /// Steps per second the shell should call [`Elite::step`] at.
    pub fn step_hz(&self) -> u32 {
        elite_sim::STEP_HZ
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_and_save_round_trip() {
        let mut e = Elite::new();
        e.key(Key::Select as u8, true);
        e.step();
        assert!(e.is_docked());
        let json = e.save();
        let mut e2 = Elite::new();
        assert!(e2.load(&json));
        assert!(!e2.load("nonsense"));
        assert_eq!(e.font().len(), 512);
        assert!(!e.lines().is_empty());
    }
}
