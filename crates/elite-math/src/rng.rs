/// Elite's four-byte random number generator (the DORND routine).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rng {
    seed: [u8; 4],
}

impl Rng {
    pub const fn new(seed: u32) -> Self {
        Rng {
            seed: seed.to_le_bytes(),
        }
    }

    /// Next byte. Ported from DORND:
    /// ```text
    /// r0 = r0 + r2 + carry(r1 + r3), r2 = old r0
    /// r1 = r1 + r3, r3 = old r1        (carry = bit7 of old r1 ... simplified)
    /// ```
    pub fn next_u8(&mut self) -> u8 {
        let [r0, r1, r2, r3] = self.seed;
        let (nr1, carry) = r1.overflowing_add(r3);
        let (t, c2) = r0.overflowing_add(r2);
        let nr0 = t.wrapping_add(carry as u8);
        let _ = c2;
        self.seed = [nr0, nr1, r0, r1];
        nr0
    }

    /// A signed byte, uniformly in -128..=127.
    pub fn next_i8(&mut self) -> i8 {
        self.next_u8() as i8
    }

    /// A byte in 0..n (n > 0).
    pub fn below(&mut self, n: u8) -> u8 {
        ((self.next_u8() as u16 * n as u16) >> 8) as u8
    }

    pub fn chance(&mut self, out_of_256: u8) -> bool {
        self.next_u8() < out_of_256
    }
}
