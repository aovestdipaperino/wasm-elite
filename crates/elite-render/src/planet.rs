//! Planet and sun.

use elite_math::Vec3;
use elite_sim::combat::view_coords;
use elite_sim::{Game, PLANET_RADIUS, SUN_RADIUS, View};

use crate::project::{project, project_radius};
use crate::{Frame, VIEW_HEIGHT, WHITE, WIDTH, YELLOW};

const SEGMENTS: usize = 48;

/// A circle as a polyline, clipped to the view.
pub fn circle(f: &mut Frame, cx: i32, cy: i32, r: i32, colour: i16) {
    if r <= 0 {
        return;
    }
    if cx + r < 0 || cx - r >= WIDTH as i32 || cy + r < 0 || cy - r >= VIEW_HEIGHT as i32 {
        return;
    }
    if r < 2 {
        if (0..WIDTH as i32).contains(&cx) && (0..VIEW_HEIGHT as i32).contains(&cy) {
            f.dot(cx as i16, cy as i16, colour);
        }
        return;
    }
    let segs = if r < 12 { 16 } else { SEGMENTS };
    let mut prev = None;
    for i in 0..=segs {
        let a = (i * 256 / segs) as u8;
        let x = cx + (r as i64 * elite_math::cos_q(a) as i64 / 32767) as i32;
        let y = cy + (r as i64 * elite_math::sin_q(a) as i64 / 32767) as i32;
        if let Some((px, py)) = prev {
            f.view_line(px, py, x, y, colour);
        }
        prev = Some((x, y));
    }
}

pub fn draw_planet(g: &Game, f: &mut Frame, view: View) {
    let (x, y, z) = view_coords(g.universe.planet, view);
    if z <= 0 {
        return;
    }
    let r = project_radius(PLANET_RADIUS, z);
    let (cx, cy) = project(x, y, z);
    circle(f, cx, cy, r, WHITE);
    // A meridian and an equator hint at the sphere, as the original's planet does.
    if r > 8 && r < 4000 {
        let tilt = ((g.step / 6) % 256) as u8;
        ellipse_arc(f, cx, cy, r, tilt, WHITE);
    }
}

/// The visible half of a great circle seen edge-on at angle `tilt`.
fn ellipse_arc(f: &mut Frame, cx: i32, cy: i32, r: i32, tilt: u8, colour: i16) {
    let squash = elite_math::cos_q(tilt) as i64; // -1..1 in Q15
    let mut prev = None;
    for i in 0..=24 {
        let a = (i * 128 / 24) as u8; // half circle
        let px = (r as i64 * elite_math::cos_q(a) as i64 / 32767) as i32;
        let py = (r as i64 * elite_math::sin_q(a) as i64 * squash / (32767 * 32767)) as i32;
        let (x, y) = (cx + px, cy + py);
        if let Some((qx, qy)) = prev {
            f.view_line(qx, qy, x, y, colour);
        }
        prev = Some((x, y));
    }
}

pub fn draw_sun(g: &Game, f: &mut Frame, view: View) {
    let (x, y, z) = view_coords(g.universe.sun, view);
    if z <= 0 {
        return;
    }
    let r = project_radius(SUN_RADIUS, z);
    if r > 6000 {
        return;
    }
    let (cx, cy) = project(x, y, z);
    if cx + r < 0 || cx - r >= WIDTH as i32 || cy + r < 0 || cy - r >= VIEW_HEIGHT as i32 {
        return;
    }
    // Filled with horizontal lines and a ragged fringe, like the original's SUN routine.
    let step = if r > 40 { 2 } else { 1 };
    let mut row = -r;
    while row <= r {
        let sy = cy + row;
        if (0..VIEW_HEIGHT as i32).contains(&sy) {
            let half = ((r as i64 * r as i64 - row as i64 * row as i64) as f64).sqrt() as i32;
            let fringe = ((sy * 7 + g.step as i32 * 3) % 5) - 2;
            let h = (half + fringe).max(0);
            f.view_line(cx - h, sy, cx + h, sy, YELLOW);
        }
        row += step;
    }
    let _ = Vec3::ZERO;
}
