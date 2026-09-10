/// Sine of `a` where 256 steps make a full turn, in Q15 (0x7FFF = 1.0).
pub fn sin_q(a: u8) -> i16 {
    const QUARTER: [i16; 65] = build_quarter();
    let a = a as usize;
    match a >> 6 {
        0 => QUARTER[a],
        1 => QUARTER[128 - a],
        2 => -QUARTER[a - 128],
        _ => -QUARTER[256 - a],
    }
}

/// Cosine, same convention as [`sin_q`].
pub fn cos_q(a: u8) -> i16 {
    sin_q(a.wrapping_add(64))
}

const fn build_quarter() -> [i16; 65] {
    // sin(k * pi/128) * 32767 for k in 0..=64, computed with a const Taylor series.
    let mut out = [0i16; 65];
    let mut k = 0;
    while k <= 64 {
        // angle in radians scaled by 2^30
        let angle = (k as i64 * 3373259426i64) / 128; // pi * 2^30 = 3373259426
        out[k] = (sin_fixed(angle) * 32767 / (1i64 << 30)) as i16;
        k += 1;
    }
    out[64] = 32767;
    out
}

/// sin(x) for x in [0, pi/2] given as Q30, returning Q30, by Taylor series.
const fn sin_fixed(x: i64) -> i64 {
    let one = 1i64 << 30;
    let x2 = (x * x) / one;
    let mut term = x;
    let mut sum = x;
    let mut n = 1i64;
    while n < 12 {
        term = -(term * x2 / one) / ((2 * n) * (2 * n + 1));
        sum += term;
        n += 1;
    }
    sum
}

/// Angle of the point (x, y) in 0..256 steps per turn, measured from +x toward +y.
pub fn atan_q(y: i32, x: i32) -> u8 {
    if x == 0 && y == 0 {
        return 0;
    }
    let (ax, ay) = (x.unsigned_abs() as u64, y.unsigned_abs() as u64);
    // first-octant ratio r in 0..=1 as Q16, then atan(r) approx r*(pi/4)*(1 + 0.273*(1-r)) scaled to 32 steps per quarter... use a table of 65 entries.
    let (num, den, swap) = if ay > ax {
        (ax, ay, true)
    } else {
        (ay, ax, false)
    };
    let r = ((num << 6) / den) as usize; // 0..=64
    const ATAN: [u8; 65] = build_atan();
    let mut a = ATAN[r] as i32; // 0..=32 steps within the octant
    if swap {
        a = 64 - a;
    }
    if x < 0 {
        a = 128 - a;
    }
    if y < 0 {
        a = -a;
    }
    (a & 255) as u8
}

const fn build_atan() -> [u8; 65] {
    let mut out = [0u8; 65];
    let mut i = 0;
    while i <= 64 {
        // atan(i/64) / (2*pi) * 256, via atan series on Q30
        let r = ((i as i64) << 30) / 64;
        let a = atan_fixed(r); // Q30 radians
        // 256 steps per 2*pi: a * 256 / (2*pi)
        let steps = (a * 256 * 1000000) / (6746518852i64 * 1000000 / (1 << 30)) / (1 << 30);
        let _ = steps;
        let v = (a * 128) / 3373259426i64; // a / pi * 128
        out[i] = v as u8;
        i += 1;
    }
    out[64] = 32;
    out
}

/// atan(r) for r in [0,1] as Q30, using atan(r) = r - r^3/3 + ... accelerated by argument reduction.
const fn atan_fixed(r: i64) -> i64 {
    let one = 1i64 << 30;
    // reduce: atan(r) = pi/4 + atan((r-1)/(r+1)) for better convergence near 1
    let pi4 = 3373259426i64 / 4;
    let t = ((r - one) * one) / (r + one); // in [-1, 0]
    let t2 = (t * t) / one;
    let mut term = t;
    let mut sum = t;
    let mut n = 1i64;
    while n < 40 {
        term = -(term * t2) / one;
        sum += term / (2 * n + 1);
        n += 1;
    }
    pi4 + sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_table_landmarks() {
        assert_eq!(sin_q(0), 0);
        assert_eq!(sin_q(64), 32767);
        assert_eq!(sin_q(128), 0);
        assert_eq!(sin_q(192), -32767);
        assert!((sin_q(32) - 23170).abs() <= 2, "sin(45deg) = {}", sin_q(32));
        assert!((cos_q(0) - 32767).abs() <= 1);
    }

    #[test]
    fn atan_quadrants() {
        assert_eq!(atan_q(0, 100), 0);
        assert_eq!(atan_q(100, 0), 64);
        assert_eq!(atan_q(0, -100), 128);
        assert_eq!(atan_q(-100, 0), 192);
        assert!((atan_q(100, 100) as i32 - 32).abs() <= 1);
        assert!((atan_q(58, 100) as i32 - 21).abs() <= 1); // atan(0.58) ~ 30 deg
    }
}
