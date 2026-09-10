//! Perspective projection and line clipping.

use crate::{CENTRE_X, CENTRE_Y};

/// Focal length: a point at x = z/2 lands on the screen edge (about 53 degrees of view).
pub const FOCAL: i64 = 256;

/// Project a view-space point with z > 0 to screen pixels (y down).
pub fn project(x: i32, y: i32, z: i32) -> (i32, i32) {
    let z = z.max(1) as i64;
    let sx = CENTRE_X as i64 + x as i64 * FOCAL / z;
    let sy = CENTRE_Y as i64 - y as i64 * FOCAL / z;
    (
        sx.clamp(-32000, 32000) as i32,
        sy.clamp(-32000, 32000) as i32,
    )
}

/// Screen radius of a sphere of radius `r` at distance `z`.
pub fn project_radius(r: i32, z: i32) -> i32 {
    (r as i64 * FOCAL / z.max(1) as i64).min(32000) as i32
}

const INSIDE: u8 = 0;
const LEFT: u8 = 1;
const RIGHT: u8 = 2;
const BOTTOM: u8 = 4;
const TOP: u8 = 8;

fn code(x: i32, y: i32, x0: i32, y0: i32, x1: i32, y1: i32) -> u8 {
    let mut c = INSIDE;
    if x < x0 {
        c |= LEFT;
    } else if x > x1 {
        c |= RIGHT;
    }
    if y < y0 {
        c |= TOP;
    } else if y > y1 {
        c |= BOTTOM;
    }
    c
}

/// Cohen-Sutherland clipping to the inclusive rectangle. Returns None if fully outside.
#[allow(clippy::too_many_arguments)]
pub fn clip_line(
    mut ax: i32,
    mut ay: i32,
    mut bx: i32,
    mut by: i32,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
) -> Option<(i32, i32, i32, i32)> {
    let mut ca = code(ax, ay, x0, y0, x1, y1);
    let mut cb = code(bx, by, x0, y0, x1, y1);
    for _ in 0..8 {
        if ca | cb == INSIDE {
            return Some((ax, ay, bx, by));
        }
        if ca & cb != 0 {
            return None;
        }
        let out = if ca != INSIDE { ca } else { cb };
        let (dx, dy) = ((bx - ax) as i64, (by - ay) as i64);
        let (nx, ny) = if out & TOP != 0 {
            (ax as i64 + dx * (y0 as i64 - ay as i64) / dy, y0 as i64)
        } else if out & BOTTOM != 0 {
            (ax as i64 + dx * (y1 as i64 - ay as i64) / dy, y1 as i64)
        } else if out & RIGHT != 0 {
            (x1 as i64, ay as i64 + dy * (x1 as i64 - ax as i64) / dx)
        } else {
            (x0 as i64, ay as i64 + dy * (x0 as i64 - ax as i64) / dx)
        };
        if out == ca {
            ax = nx as i32;
            ay = ny as i32;
            ca = code(ax, ay, x0, y0, x1, y1);
        } else {
            bx = nx as i32;
            by = ny as i32;
            cb = code(bx, by, x0, y0, x1, y1);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centre_projects_to_centre() {
        assert_eq!(project(0, 0, 1000), (128, 96));
        assert_eq!(project(500, 0, 1000), (256, 96));
        assert_eq!(project(0, 375, 1000), (128, 0));
    }

    #[test]
    fn clipping() {
        assert_eq!(
            clip_line(10, 10, 20, 20, 0, 0, 255, 191),
            Some((10, 10, 20, 20))
        );
        assert_eq!(
            clip_line(-10, 10, 10, 10, 0, 0, 255, 191),
            Some((0, 10, 10, 10))
        );
        assert_eq!(clip_line(-10, -10, -5, -5, 0, 0, 255, 191), None);
        let (a, b, c, d) = clip_line(-100, 96, 400, 96, 0, 0, 255, 191).unwrap();
        assert_eq!((a, b, c, d), (0, 96, 255, 96));
    }
}
