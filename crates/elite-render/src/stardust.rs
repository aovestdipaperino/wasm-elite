//! The moving stardust that gives a sense of speed.

use elite_math::{Orient, Rng, Vec3};
use elite_sim::combat::view_coords;
use elite_sim::{Game, View};

use crate::project::project;
use crate::{Frame, VIEW_HEIGHT, WHITE, WIDTH};

const COUNT: usize = 18;
const DEPTH: i32 = 8000;

#[derive(Clone, Debug)]
pub struct Stardust {
    /// Particles in player-frame coordinates (front view frame).
    dust: [Vec3; COUNT],
    rng: Rng,
}

impl Default for Stardust {
    fn default() -> Self {
        Self::new()
    }
}

impl Stardust {
    pub fn new() -> Self {
        let mut s = Stardust {
            dust: [Vec3::ZERO; COUNT],
            rng: Rng::new(0xD057),
        };
        for i in 0..COUNT {
            s.dust[i] = s.fresh(View::Front, true);
        }
        s
    }

    fn fresh(&mut self, view: View, anywhere: bool) -> Vec3 {
        let x = self.rng.next_i8() as i32 * 24;
        let y = self.rng.next_i8() as i32 * 18;
        let z = if anywhere {
            500 + self.rng.next_u8() as i32 * 30
        } else {
            DEPTH
        };
        // Spawn in the frame of the current window so new dust enters where the view looks.
        let (fx, fy, fz) = match view {
            View::Front => (x, y, z),
            View::Rear => (-x, y, -z),
            View::Left => (-z, y, x),
            View::Right => (z, y, -x),
        };
        Vec3::new(fx, fy, fz)
    }

    pub fn update(&mut self, g: &Game) {
        let dz = ((g.player.speed as i32) >> 1) * if g.player.jumping { 64 } else { 1 };
        let view = g.player.view;
        for i in 0..COUNT {
            let p = self.dust[i] + Vec3::new(0, 0, -dz);
            let p = Orient::rotate_point_by_player(p, g.player.alpha, g.player.beta);
            let (_, _, vz) = view_coords(p, view);
            self.dust[i] = if vz <= 40 || vz > DEPTH {
                self.fresh(view, false)
            } else {
                p
            };
        }
    }

    pub fn draw(&self, g: &Game, f: &mut Frame) {
        let view = g.player.view;
        for p in &self.dust {
            let (x, y, z) = view_coords(*p, view);
            if z <= 0 {
                continue;
            }
            let (sx, sy) = project(x, y, z);
            if (0..WIDTH as i32).contains(&sx) && (0..VIEW_HEIGHT as i32).contains(&sy) {
                f.dot(sx as i16, sy as i16, WHITE);
                if z < 1500 {
                    f.dot((sx + 1).min(WIDTH as i32 - 1) as i16, sy as i16, WHITE);
                }
            }
        }
    }
}
