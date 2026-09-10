//! Wireframe ships, explosions and the laser sights.

use elite_math::{Rng, Vec3};
use elite_sim::combat::view_coords;
use elite_sim::{Game, ShipSlot};

use crate::planet;
use crate::project::{project, project_radius};
use crate::{CENTRE_X, CENTRE_Y, Frame, RED, VIEW_HEIGHT, WHITE, WIDTH, YELLOW};

/// Everything visible through the current window.
pub fn draw_space(g: &Game, f: &mut Frame) {
    let view = g.player.view;
    planet::draw_planet(g, f, view);
    planet::draw_sun(g, f, view);
    if let Some(st) = &g.universe.station {
        draw_ship(st, view, f, g.step);
    }
    for s in &g.universe.ships {
        draw_ship(s, view, f, g.step);
    }
    if g.player.laser_fired {
        draw_laser(f, g.step);
    }
}

fn rotate_orient_for_view(s: &ShipSlot, view: elite_sim::View) -> elite_math::Orient {
    let r = |v: elite_math::Fp3| {
        let (x, y, z) = view_coords(Vec3::new(v.x as i32, v.y as i32, v.z as i32), view);
        elite_math::Fp3::new(x as i16, y as i16, z as i16)
    };
    elite_math::Orient {
        nosev: r(s.orient.nosev),
        roofv: r(s.orient.roofv),
        sidev: r(s.orient.sidev),
    }
}

pub fn draw_ship(s: &ShipSlot, view: elite_sim::View, f: &mut Frame, step: u32) {
    let (px, py, pz) = view_coords(s.pos, view);
    let pos = Vec3::new(px, py, pz);
    if let Some(t) = s.exploding {
        draw_explosion(pos, s, t, f, step);
        return;
    }
    if pz <= 0 {
        return;
    }
    let bp = s.blueprint();
    // Too far for detail: a single dot.
    if (pz >> 8) > bp.vis as i32 || pos.length_approx() > 60_000 {
        let (sx, sy) = project(px, py, pz);
        if (0..WIDTH as i32).contains(&sx) && (0..VIEW_HEIGHT as i32).contains(&sy) {
            f.dot(sx as i16, sy as i16, WHITE);
        }
        return;
    }
    let o = rotate_orient_for_view(s, view);
    let colour = if s.hit_flash { YELLOW } else { WHITE };

    // Face visibility: a face is seen if its normal points toward the camera.
    let mut visible = [true; 16];
    for (i, face) in bp.faces.iter().enumerate().take(16) {
        let n = o.local_to_view(Vec3::new(face.nx as i32, face.ny as i32, face.nz as i32));
        // Camera at origin; the face sits near the ship centre plus the normal direction.
        let scale = bp.radius().max(1);
        let point = pos + Vec3::new(n.x * scale / 160, n.y * scale / 160, n.z * scale / 160);
        let d =
            n.x as i64 * point.x as i64 + n.y as i64 * point.y as i64 + n.z as i64 * point.z as i64;
        visible[i] = d < 0;
    }

    let mut screen: Vec<Option<(i32, i32)>> = Vec::with_capacity(bp.vertices.len());
    for v in bp.vertices {
        let w = o.local_to_view(Vec3::new(v.x as i32, v.y as i32, v.z as i32)) + pos;
        screen.push(if w.z > 0 {
            Some(project(w.x, w.y, w.z))
        } else {
            None
        });
    }
    let far = (pz >> 8) as u8;
    for e in bp.edges {
        if e.vis < far.min(31) {
            continue;
        }
        let shown = e
            .faces
            .iter()
            .any(|&fi| fi == 15 || (fi as usize) < bp.faces.len() && visible[fi as usize]);
        if !shown {
            continue;
        }
        if let (Some(a), Some(b)) = (screen[e.v1 as usize], screen[e.v2 as usize]) {
            f.view_line(a.0, a.1, b.0, b.1, colour);
        }
    }
}

fn draw_explosion(pos: Vec3, s: &ShipSlot, t: u8, f: &mut Frame, step: u32) {
    if pos.z <= 0 {
        return;
    }
    let (cx, cy) = project(pos.x, pos.y, pos.z);
    let radius = project_radius(s.blueprint().radius(), pos.z);
    let age = 24 - t as i32;
    let spread = (radius * (age + 4) / 8).max(1);
    // Deterministic per ship and frame so the cloud does not jitter randomly between frames.
    let mut rng = Rng::new((pos.x as u32) ^ (pos.y as u32).rotate_left(7) ^ (step / 4));
    let n = (s.blueprint().explosion as i32 * 3).min(60);
    for _ in 0..n {
        let dx = rng.next_i8() as i32 * spread / 128;
        let dy = rng.next_i8() as i32 * spread / 128;
        let (x, y) = (cx + dx, cy + dy);
        if (0..WIDTH as i32).contains(&x) && (0..VIEW_HEIGHT as i32).contains(&y) {
            f.dot(x as i16, y as i16, if age < 6 { YELLOW } else { WHITE });
        }
    }
}

fn draw_laser(f: &mut Frame, step: u32) {
    let jitter = ((step * 7) % 5) as i16 - 2;
    let tx = CENTRE_X + jitter;
    let ty = CENTRE_Y + 8;
    f.line(0, VIEW_HEIGHT - 1, tx, ty, RED);
    f.line(WIDTH - 1, VIEW_HEIGHT - 1, tx, ty, RED);
    f.line(8, VIEW_HEIGHT - 1, tx, ty, RED);
    f.line(WIDTH - 9, VIEW_HEIGHT - 1, tx, ty, RED);
}

pub fn draw_sights(g: &Game, f: &mut Frame) {
    if g.current_laser() == 0 {
        return;
    }
    let (x, y) = (CENTRE_X, CENTRE_Y);
    f.line(x, y - 12, x, y - 4, WHITE);
    f.line(x, y + 4, x, y + 12, WHITE);
    f.line(x - 12, y, x - 4, y, WHITE);
    f.line(x + 4, y, x + 12, y, WHITE);
}
