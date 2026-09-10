use crate::mat::{Fp3, UNIT};

/// A position in space. Elite uses 24-bit sign-magnitude coordinates; we widen to i32.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vec3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0, y: 0, z: 0 };

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Vec3 { x, y, z }
    }
    /// Dot product with a unit vector, result in the same scale as `self`.
    pub fn dot_fp(self, u: Fp3) -> i32 {
        let x = self.x as i64 * u.x as i64;
        let y = self.y as i64 * u.y as i64;
        let z = self.z as i64 * u.z as i64;
        ((x + y + z) / UNIT as i64) as i32
    }

    /// Scale a unit vector by a magnitude.
    pub fn from_fp(u: Fp3, mag: i32) -> Vec3 {
        Vec3::new(
            (u.x as i64 * mag as i64 / UNIT as i64) as i32,
            (u.y as i64 * mag as i64 / UNIT as i64) as i32,
            (u.z as i64 * mag as i64 / UNIT as i64) as i32,
        )
    }

    /// Elite's cheap length: largest component plus half the sum of the other two.
    /// Overestimates by up to about 40%, which the original's ranges account for.
    pub fn length_approx(self) -> u32 {
        let (a, b, c) = (
            self.x.unsigned_abs(),
            self.y.unsigned_abs(),
            self.z.unsigned_abs(),
        );
        let max = a.max(b).max(c);
        max + ((a + b + c - max) >> 1)
    }

    /// Exact integer length, for tests and range checks.
    pub fn length(self) -> u32 {
        let s = (self.x as i64).pow(2) + (self.y as i64).pow(2) + (self.z as i64).pow(2);
        isqrt(s as u64) as u32
    }

    pub fn max_abs(self) -> u32 {
        self.x
            .unsigned_abs()
            .max(self.y.unsigned_abs())
            .max(self.z.unsigned_abs())
    }
}

impl core::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.x.wrapping_add(o.x),
            self.y.wrapping_add(o.y),
            self.z.wrapping_add(o.z),
        )
    }
}

impl core::ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.x.wrapping_sub(o.x),
            self.y.wrapping_sub(o.y),
            self.z.wrapping_sub(o.z),
        )
    }
}

impl core::ops::Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

/// Arithmetic shift right on every component.
impl core::ops::Shr<u8> for Vec3 {
    type Output = Vec3;
    fn shr(self, s: u8) -> Vec3 {
        Vec3::new(self.x >> s, self.y >> s, self.z >> s)
    }
}

pub(crate) fn isqrt(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let mut x = 1u64 << ((64 - n.leading_zeros()).div_ceil(2));
    loop {
        let y = (x + n / x) >> 1;
        if y >= x {
            return x;
        }
        x = y;
    }
}
