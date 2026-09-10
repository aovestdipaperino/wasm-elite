//! Turns a [`Game`] into a list of primitives in a 256 x 248 logical screen.
//! The shell draws the primitives; nothing here knows about a canvas.

mod chart;
mod dash;
mod flight_consts;
mod font;
mod planet;
mod project;
mod scanner;
mod screens;
mod stardust;
mod text;
mod wire;

use elite_sim::{Game, Mode, Screen};

pub use font::FONT;
pub use text::Text;

pub const WIDTH: i16 = 256;
pub const HEIGHT: i16 = 248;
/// Height of the space view / text area; the dashboard is below it.
pub const VIEW_HEIGHT: i16 = 192;
pub const CENTRE_X: i16 = 128;
pub const CENTRE_Y: i16 = 96;

/// Colour indices. The shell maps them to a palette.
pub const WHITE: i16 = 0;
pub const RED: i16 = 1;
pub const YELLOW: i16 = 2;
pub const CYAN: i16 = 3;
pub const GREEN: i16 = 4;
pub const DIM: i16 = 5;

/// One frame of primitives.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    /// x1, y1, x2, y2, colour.
    pub lines: Vec<[i16; 5]>,
    /// x, y, w, h, colour.
    pub fills: Vec<[i16; 5]>,
    /// x, y, colour.
    pub dots: Vec<[i16; 3]>,
    /// x, y, ascii, colour.
    pub glyphs: Vec<[i16; 4]>,
    /// Invert the whole display for this frame (taking damage).
    pub flash: bool,
}

impl Frame {
    pub fn line(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, c: i16) {
        self.lines.push([x1, y1, x2, y2, c]);
    }

    pub fn dot(&mut self, x: i16, y: i16, c: i16) {
        self.dots.push([x, y, c]);
    }

    pub fn fill(&mut self, x: i16, y: i16, w: i16, h: i16, c: i16) {
        self.fills.push([x, y, w, h, c]);
    }

    /// Draw text at a pixel position, 8 pixels per character.
    pub fn text(&mut self, x: i16, y: i16, s: &str, c: i16) {
        for (i, ch) in s.chars().enumerate() {
            self.glyphs
                .push([x + 8 * i as i16, y, font::glyph_code(ch) as i16, c]);
        }
    }

    /// Text centred in the 256-wide screen.
    pub fn text_centred(&mut self, y: i16, s: &str, c: i16) {
        let w = 8 * s.chars().count() as i16;
        self.text((WIDTH - w) / 2, y, s, c);
    }

    /// Line clipped to the space view.
    pub fn view_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i16) {
        if let Some((a, b, cc, d)) = project::clip_line(
            x1,
            y1,
            x2,
            y2,
            0,
            0,
            WIDTH as i32 - 1,
            VIEW_HEIGHT as i32 - 1,
        ) {
            self.line(a as i16, b as i16, cc as i16, d as i16, c);
        }
    }
}

/// Renderer state that persists across frames (the stardust field).
#[derive(Clone, Debug, Default)]
pub struct Renderer {
    dust: stardust::Stardust,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            dust: stardust::Stardust::new(),
        }
    }

    pub fn render(&mut self, g: &Game) -> Frame {
        let mut f = Frame::default();
        match g.mode {
            Mode::Title => screens::title(g, &mut f),
            Mode::Death => {
                self.dust.update(g);
                wire::draw_space(g, &mut f);
                self.dust.draw(g, &mut f);
                screens::death(g, &mut f);
            }
            Mode::Docked | Mode::Flight => match g.screen {
                Screen::Space => {
                    if g.mode == Mode::Flight {
                        self.dust.update(g);
                        wire::draw_space(g, &mut f);
                        self.dust.draw(g, &mut f);
                        wire::draw_sights(g, &mut f);
                    } else {
                        screens::docked_space(g, &mut f);
                    }
                }
                Screen::Status => screens::status(g, &mut f),
                Screen::Market => screens::market(g, &mut f),
                Screen::SystemData => screens::system_data(g, &mut f),
                Screen::GalacticChart => chart::galactic(g, &mut f),
                Screen::ShortRangeChart => chart::short_range(g, &mut f),
                Screen::Inventory => screens::inventory(g, &mut f),
                Screen::Equip => screens::equip(g, &mut f),
            },
        }
        if g.mode != Mode::Title {
            screens::frame_border(&mut f);
            dash::draw(g, &mut f);
            screens::message(g, &mut f);
        }
        f.flash = g.player.damage_flash > 2;
        f
    }
}
