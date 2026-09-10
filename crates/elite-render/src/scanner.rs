//! The 3D scanner and the compass.

use elite_math::Vec3;
use elite_sim::{Game, SCANNER_RANGE};

use crate::{CYAN, Frame, GREEN, RED, WHITE, YELLOW};

pub const SCANNER_CX: i16 = 128;
pub const SCANNER_CY: i16 = 224;
pub const SCANNER_RX: i16 = 36;
pub const SCANNER_RY: i16 = 13;
pub const COMPASS_CX: i16 = 84;
pub const COMPASS_CY: i16 = 236;
pub const COMPASS_R: i16 = 6;

pub fn draw(g: &Game, f: &mut Frame) {
    ellipse(f, SCANNER_CX, SCANNER_CY, SCANNER_RX, SCANNER_RY, WHITE);
    // Centre cross.
    f.line(
        SCANNER_CX - 3,
        SCANNER_CY,
        SCANNER_CX + 3,
        SCANNER_CY,
        WHITE,
    );
    f.line(
        SCANNER_CX,
        SCANNER_CY - 2,
        SCANNER_CX,
        SCANNER_CY + 2,
        WHITE,
    );

    if let Some(st) = &g.universe.station {
        blip(f, st.pos, GREEN, true);
    }
    for s in &g.universe.ships {
        if !s.is_alive() {
            continue;
        }
        let colour = if s.ai.hostile { YELLOW } else { CYAN };
        blip(f, s.pos, colour, false);
    }
    compass(g, f);
}

/// A ship on the scanner: a dot at its floor position with a stalk up or down to its height.
fn blip(f: &mut Frame, p: Vec3, colour: i16, square: bool) {
    if p.x.abs() > SCANNER_RANGE || p.z.abs() > SCANNER_RANGE || p.y.abs() > SCANNER_RANGE {
        return;
    }
    let sx = SCANNER_CX + (p.x as i64 * SCANNER_RX as i64 / SCANNER_RANGE as i64) as i16;
    let sz = SCANNER_CY - (p.z as i64 * SCANNER_RY as i64 / SCANNER_RANGE as i64) as i16;
    let h = (p.y as i64 * 12 / SCANNER_RANGE as i64) as i16;
    let top = sz - h;
    if h != 0 {
        f.line(sx, sz, sx, top, colour);
    }
    if square {
        f.fill(sx - 1, top - 1, 3, 3, colour);
    } else {
        f.fill(sx - 1, top, 3, 1, colour);
    }
}

fn compass(g: &Game, f: &mut Frame) {
    circle_small(f, COMPASS_CX, COMPASS_CY, COMPASS_R, WHITE);
    // Points at the station when it is near, otherwise at the planet.
    let target = match &g.universe.station {
        Some(st) if st.pos.length() < 600_000 => st.pos,
        _ => g.universe.planet,
    };
    let len = target.length().max(1) as i64;
    let dx = (target.x as i64 * (COMPASS_R as i64 - 1) / len) as i16;
    let dy = (target.y as i64 * (COMPASS_R as i64 - 1) / len) as i16;
    let colour = if target.z >= 0 { GREEN } else { RED };
    f.fill(COMPASS_CX + dx - 1, COMPASS_CY - dy - 1, 3, 3, colour);
}

pub fn ellipse(f: &mut Frame, cx: i16, cy: i16, rx: i16, ry: i16, colour: i16) {
    let segs = 32;
    let mut prev = None;
    for i in 0..=segs {
        let a = (i * 256 / segs) as u8;
        let x = cx + (rx as i64 * elite_math::cos_q(a) as i64 / 32767) as i16;
        let y = cy + (ry as i64 * elite_math::sin_q(a) as i64 / 32767) as i16;
        if let Some((px, py)) = prev {
            f.line(px, py, x, y, colour);
        }
        prev = Some((x, y));
    }
}

pub fn circle_small(f: &mut Frame, cx: i16, cy: i16, r: i16, colour: i16) {
    ellipse(f, cx, cy, r, r, colour);
}
