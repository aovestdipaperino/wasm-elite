use crate::vec::{Vec3, isqrt};

/// 1.0 in orientation-vector fixed point, as in the original (0x6000 = 24576).
pub const UNIT: i16 = 0x6000;

/// A unit vector component-wise scaled by [`UNIT`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fp3 {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl Fp3 {
    pub const fn new(x: i16, y: i16, z: i16) -> Self {
        Fp3 { x, y, z }
    }

    /// Dot product of two unit vectors, scaled so that parallel vectors give `UNIT`.
    pub fn dot(self, o: Fp3) -> i32 {
        let s =
            self.x as i64 * o.x as i64 + self.y as i64 * o.y as i64 + self.z as i64 * o.z as i64;
        (s / UNIT as i64) as i32
    }

    pub fn cross(self, o: Fp3) -> Fp3 {
        let cx = (self.y as i64 * o.z as i64 - self.z as i64 * o.y as i64) / UNIT as i64;
        let cy = (self.z as i64 * o.x as i64 - self.x as i64 * o.z as i64) / UNIT as i64;
        let cz = (self.x as i64 * o.y as i64 - self.y as i64 * o.x as i64) / UNIT as i64;
        Fp3::new(cx as i16, cy as i16, cz as i16)
    }

    /// Rescale to unit length (the NORM routine).
    pub fn normalise(self) -> Fp3 {
        let s = (self.x as i64).pow(2) + (self.y as i64).pow(2) + (self.z as i64).pow(2);
        let len = isqrt(s as u64) as i64;
        if len == 0 {
            return Fp3::new(0, 0, UNIT);
        }
        Fp3::new(
            (self.x as i64 * UNIT as i64 / len) as i16,
            (self.y as i64 * UNIT as i64 / len) as i16,
            (self.z as i64 * UNIT as i64 / len) as i16,
        )
    }

    /// Unit vector pointing from the origin toward `v`.
    pub fn from_vec(v: Vec3) -> Fp3 {
        let len = v.length() as i64;
        if len == 0 {
            return Fp3::new(0, 0, UNIT);
        }
        Fp3::new(
            (v.x as i64 * UNIT as i64 / len) as i16,
            (v.y as i64 * UNIT as i64 / len) as i16,
            (v.z as i64 * UNIT as i64 / len) as i16,
        )
    }

    /// Rotate `self` by a small angle in the plane spanned by the two axes given
    /// through the closure order: returns `(a - b*k, b + a*k)` where k = angle/256
    /// applied to the components selected. Used by [`Orient::rotate`].
    fn rot_pair(a: i16, b: i16, k: i8) -> (i16, i16) {
        let a32 = a as i32;
        let b32 = b as i32;
        let na = a32 - ((b32 * k as i32) >> 8);
        let nb = b32 + ((a32 * k as i32) >> 8);
        (
            na.clamp(-32767, 32767) as i16,
            nb.clamp(-32767, 32767) as i16,
        )
    }
}

impl core::ops::Neg for Fp3 {
    type Output = Fp3;
    fn neg(self) -> Fp3 {
        Fp3::new(-self.x, -self.y, -self.z)
    }
}

/// The three orientation vectors of a ship, as seen in the player's frame.
/// `nosev` points out of the nose, `roofv` out of the roof, `sidev` out of the right side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Orient {
    pub nosev: Fp3,
    pub roofv: Fp3,
    pub sidev: Fp3,
}

impl Default for Orient {
    fn default() -> Self {
        Self::identity()
    }
}

impl Orient {
    /// A ship facing the same way as the player.
    pub const fn identity() -> Self {
        Orient {
            nosev: Fp3::new(0, 0, UNIT),
            roofv: Fp3::new(0, UNIT, 0),
            sidev: Fp3::new(UNIT, 0, 0),
        }
    }

    /// A ship facing the player (nose toward -z, and right side flipped so the matrix stays right-handed).
    pub const fn facing_player() -> Self {
        Orient {
            nosev: Fp3::new(0, 0, -UNIT),
            roofv: Fp3::new(0, UNIT, 0),
            sidev: Fp3::new(-UNIT, 0, 0),
        }
    }

    /// Rotate the whole matrix around the player's axes, as the universe does when the
    /// player rolls (`roll`, about the z axis) and pitches (`pitch`, about the x axis).
    /// Angles are in 1/256 radian steps, matching the original's alpha and beta.
    pub fn rotate_by_player(&mut self, roll: i8, pitch: i8) {
        for v in [&mut self.nosev, &mut self.roofv, &mut self.sidev] {
            // roll: rotate x and y about z
            let (x, y) = Fp3::rot_pair(v.x, v.y, roll);
            // pitch: rotate y and z about x
            let (y, z) = Fp3::rot_pair(y, v.z, pitch);
            *v = Fp3::new(x, y, z);
        }
    }

    /// Rotate a point in the same way [`rotate_by_player`] rotates vectors.
    pub fn rotate_point_by_player(p: Vec3, roll: i8, pitch: i8) -> Vec3 {
        let rx = p.x as i64 - ((p.y as i64 * roll as i64) >> 8);
        let ry = p.y as i64 + ((p.x as i64 * roll as i64) >> 8);
        let ry2 = ry - ((p.z as i64 * pitch as i64) >> 8);
        let rz = p.z as i64 + ((ry * pitch as i64) >> 8);
        Vec3::new(rx as i32, ry2 as i32, rz as i32)
    }

    /// Rotate the ship about its own axes (the MVS4-style own-roll and own-pitch step).
    /// Rolling rotates roofv and sidev about nosev; pitching rotates nosev and roofv about sidev.
    pub fn rotate_own(&mut self, roll: i8, pitch: i8) {
        if roll != 0 {
            let (r, s) = (self.roofv, self.sidev);
            self.roofv = lerp_rot(r, s, roll);
            self.sidev = lerp_rot(s, -r, roll);
        }
        if pitch != 0 {
            let (n, r) = (self.nosev, self.roofv);
            self.nosev = lerp_rot(n, r, pitch);
            self.roofv = lerp_rot(r, -n, pitch);
        }
    }

    /// Orthonormalise the matrix (TIDY): renormalise nosev, make roofv perpendicular, and
    /// rebuild sidev as the cross product.
    pub fn tidy(&mut self) {
        self.nosev = self.nosev.normalise();
        let d = self.roofv.dot(self.nosev);
        let r = Fp3::new(
            (self.roofv.x as i64 - d as i64 * self.nosev.x as i64 / UNIT as i64) as i16,
            (self.roofv.y as i64 - d as i64 * self.nosev.y as i64 / UNIT as i64) as i16,
            (self.roofv.z as i64 - d as i64 * self.nosev.z as i64 / UNIT as i64) as i16,
        );
        self.roofv = r.normalise();
        self.sidev = self.roofv.cross(self.nosev).normalise();
    }

    /// Transform a point given in the ship's local frame into the player's frame.
    pub fn local_to_view(&self, p: Vec3) -> Vec3 {
        Vec3::new(
            ((p.x as i64 * self.sidev.x as i64
                + p.y as i64 * self.roofv.x as i64
                + p.z as i64 * self.nosev.x as i64)
                / UNIT as i64) as i32,
            ((p.x as i64 * self.sidev.y as i64
                + p.y as i64 * self.roofv.y as i64
                + p.z as i64 * self.nosev.y as i64)
                / UNIT as i64) as i32,
            ((p.x as i64 * self.sidev.z as i64
                + p.y as i64 * self.roofv.z as i64
                + p.z as i64 * self.nosev.z as i64)
                / UNIT as i64) as i32,
        )
    }

    /// Express a player-frame vector in the ship's own axes (the transpose).
    pub fn view_to_local(&self, p: Vec3) -> Vec3 {
        Vec3::new(
            p.dot_fp(self.sidev),
            p.dot_fp(self.roofv),
            p.dot_fp(self.nosev),
        )
    }
}

/// a' = a + b * k/256, a small-angle rotation of `a` toward `b`.
fn lerp_rot(a: Fp3, b: Fp3, k: i8) -> Fp3 {
    Fp3::new(
        (a.x as i32 + ((b.x as i32 * k as i32) >> 8)).clamp(-32767, 32767) as i16,
        (a.y as i32 + ((b.y as i32 * k as i32) >> 8)).clamp(-32767, 32767) as i16,
        (a.z as i32 + ((b.z as i32 * k as i32) >> 8)).clamp(-32767, 32767) as i16,
    )
}
