//! Fixed-point 3D math for Elite: positions are `i32`, orientation vectors are
//! `i16` unit vectors where [`UNIT`] (0x6000) represents 1.0, as in the original.

#![no_std]

mod mat;
mod rng;
mod tables;
mod vec;

pub use mat::{Fp3, Orient, UNIT};
pub use rng::Rng;
pub use tables::{atan_q, cos_q, sin_q};
pub use vec::Vec3;
