use elite_math::{Orient, Vec3};
use elite_sim::{Game, Input, Key, Mode, ShipKind, ShipSlot};

fn in_flight() -> Game {
    let mut g = Game::new();
    g.dock_at_station();
    g.launch();
    g.universe.station = None;
    g.universe.planet = Vec3::new(0, 0, 5_000_000);
    g.universe.sun = Vec3::new(0, 0, 9_000_000);
    g.player.speed = 0;
    g
}

#[test]
fn ship_ahead_approaches_at_player_speed() {
    let mut g = in_flight();
    g.universe.ships.push(ShipSlot::new(
        ShipKind::Asteroid,
        Vec3::new(0, 0, 10_000),
        Orient::identity(),
    ));
    g.player.speed = 40;
    let input = Input::new();
    for _ in 0..10 {
        g.step(&input);
    }
    // Half speed per step at 20 Hz: 10 steps * 20 = 200 units.
    assert_eq!(g.universe.ships[0].pos.z, 9_800);
}

#[test]
fn rolling_keeps_a_ship_ahead_in_place() {
    let mut g = in_flight();
    g.universe.ships.push(ShipSlot::new(
        ShipKind::Asteroid,
        Vec3::new(0, 0, 10_000),
        Orient::identity(),
    ));
    let mut input = Input::new();
    input.set(Key::RollRight, true);
    for _ in 0..200 {
        g.step(&input);
    }
    let p = g.universe.ships[0].pos;
    assert_eq!((p.x, p.y), (0, 0));
    assert!((p.z - 10_000).abs() < 50, "{p:?}");
    assert_eq!(g.player.alpha, 31);
}

#[test]
fn pitching_moves_a_ship_ahead_vertically_and_back() {
    let mut g = in_flight();
    g.universe.ships.push(ShipSlot::new(
        ShipKind::Asteroid,
        Vec3::new(0, 0, 10_000),
        Orient::identity(),
    ));
    let mut input = Input::new();
    input.set(Key::PitchUp, true);
    for _ in 0..20 {
        g.step(&input);
    }
    let p = g.universe.ships[0].pos;
    assert!(
        p.y < -1000,
        "nose up should move a ship ahead down the screen: {p:?}"
    );
    let len = p.length() as i32;
    assert!(
        (len - 10_000).abs() < 100,
        "rotation must preserve distance: {len}"
    );
}

#[test]
fn energy_recharges_when_idle() {
    let mut g = in_flight();
    g.player.energy = 100;
    let input = Input::new();
    for _ in 0..200 {
        g.step(&input);
    }
    assert_eq!(g.player.energy, 255);
    assert_eq!(g.mode, Mode::Flight);
}

#[test]
fn save_and_load_round_trip() {
    let mut g = Game::new();
    g.dock_at_station();
    g.buy(0, 2).unwrap();
    let json = serde_json::to_string(&g.commander).unwrap();
    let c: elite_sim::Commander = serde_json::from_str(&json).unwrap();
    let g2 = Game::with_commander(c);
    assert_eq!(g2.commander.cargo[0], 2);
    assert_eq!(g2.present().name, "Lave");
}
