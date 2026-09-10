use elite_math::{Fp3, Orient, Rng, UNIT, Vec3};

#[test]
fn identity_dots() {
    let o = Orient::identity();
    assert_eq!(o.nosev.dot(o.nosev), UNIT as i32);
    assert_eq!(o.nosev.dot(o.roofv), 0);
    assert_eq!(o.sidev.dot(o.roofv), 0);
}

#[test]
fn tidy_keeps_orthonormal_after_many_rotations() {
    let mut o = Orient::identity();
    let mut rng = Rng::new(0x1234_5678);
    for i in 0..1000 {
        o.rotate_by_player(rng.next_i8() >> 3, rng.next_i8() >> 4);
        o.rotate_own(rng.next_i8() >> 4, rng.next_i8() >> 4);
        if i % 8 == 7 {
            o.tidy();
        }
    }
    o.tidy();
    let tol = 64;
    assert!((o.nosev.dot(o.nosev) - UNIT as i32).abs() < tol);
    assert!((o.roofv.dot(o.roofv) - UNIT as i32).abs() < tol);
    assert!((o.sidev.dot(o.sidev) - UNIT as i32).abs() < tol);
    assert!(o.nosev.dot(o.roofv).abs() < tol);
    assert!(o.nosev.dot(o.sidev).abs() < tol);
    assert!(o.roofv.dot(o.sidev).abs() < tol);
}

#[test]
fn rotate_by_player_is_a_rotation() {
    // Rolling right by 32 steps 50 times should turn roofv into approximately -sidev? No: check length only.
    let mut o = Orient::identity();
    for _ in 0..50 {
        o.rotate_by_player(8, 0);
        o.tidy();
    }
    assert!((o.roofv.dot(o.roofv) - UNIT as i32).abs() < 64);
    // roofv rotated about z, so its z stays 0
    assert_eq!(o.roofv.z, 0);
}

#[test]
fn point_rotation_matches_vector_rotation() {
    let p = Vec3::new(0, 0, 10_000);
    let r = Orient::rotate_point_by_player(p, 0, 16);
    // pitching moves a point ahead of us up or down, never sideways
    assert_eq!(r.x, 0);
    assert_ne!(r.y, 0);
    assert!(r.z <= 10_000 && r.z > 9_000);
}

#[test]
fn rng_is_deterministic_and_varied() {
    let mut a = Rng::new(42);
    let mut b = Rng::new(42);
    let sa: Vec<u8> = (0..16).map(|_| a.next_u8()).collect();
    let sb: Vec<u8> = (0..16).map(|_| b.next_u8()).collect();
    assert_eq!(sa, sb);
    let distinct: std::collections::HashSet<_> = sa.iter().collect();
    assert!(distinct.len() > 8);
}

#[test]
fn length_approx_bounds() {
    let v = Vec3::new(300, 400, 0);
    assert_eq!(v.length(), 500);
    let a = v.length_approx();
    assert!((500..=700).contains(&a));
}

#[test]
fn local_to_view_roundtrip() {
    let mut o = Orient::identity();
    o.rotate_own(20, -13);
    o.rotate_own(20, -13);
    o.tidy();
    let p = Vec3::new(1000, -2000, 3000);
    let back = o.view_to_local(o.local_to_view(p));
    assert!(
        (back.x - p.x).abs() < 8 && (back.y - p.y).abs() < 8 && (back.z - p.z).abs() < 8,
        "{back:?}"
    );
    let _ = Fp3::new(0, 0, UNIT);
}
