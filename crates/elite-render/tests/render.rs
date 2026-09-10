use elite_math::{Orient, Vec3};
use elite_render::{Frame, Renderer, VIEW_HEIGHT, WIDTH};
use elite_sim::{Game, Screen, ShipKind, ShipSlot};

fn in_flight() -> Game {
    let mut g = Game::new();
    g.dock_at_station();
    g.launch();
    g.universe.station = None;
    g.universe.planet = Vec3::new(0, 0, 50_000_000);
    g.universe.sun = Vec3::new(0, 0, -90_000_000);
    g.player.speed = 0;
    g
}

/// Lines inside the space view, excluding the frame border.
fn view_lines(f: &Frame) -> Vec<[i16; 5]> {
    f.lines
        .iter()
        .copied()
        .filter(|l| l[1] < VIEW_HEIGHT && l[3] < VIEW_HEIGHT)
        .filter(|l| {
            !(l[0] == l[2] && (l[0] == 0 || l[0] == WIDTH - 1)) && !(l[1] == 0 && l[3] == 0)
        })
        .collect()
}

#[test]
fn cobra_ahead_renders_inside_the_view() {
    let mut g = in_flight();
    g.universe.ships.push(ShipSlot::new(
        ShipKind::CobraMk3,
        Vec3::new(0, 0, 1500),
        Orient::facing_player(),
    ));
    let f = Renderer::new().render(&g);
    let lines = view_lines(&f);
    assert!(
        lines.len() > 20,
        "expected a wireframe, got {} lines",
        lines.len()
    );
    for l in &lines {
        assert!(
            (0..WIDTH).contains(&l[0]) && (0..WIDTH).contains(&l[2]),
            "{l:?}"
        );
        assert!(
            (0..VIEW_HEIGHT).contains(&l[1]) && (0..VIEW_HEIGHT).contains(&l[3]),
            "{l:?}"
        );
    }
}

#[test]
fn distant_ship_is_a_dot() {
    let mut g = in_flight();
    g.universe.ships.push(ShipSlot::new(
        ShipKind::CobraMk3,
        Vec3::new(0, 0, 40_000),
        Orient::facing_player(),
    ));
    let f = Renderer::new().render(&g);
    assert!(f.dots.iter().any(|d| d[0] == 128 && d[1] == 96));
}

#[test]
fn planet_circle_has_expected_radius() {
    let mut g = in_flight();
    // radius 300_000 at z = 7_680_000 projects to 10 px
    g.universe.planet = Vec3::new(0, 0, 7_680_000);
    g.commander.equipment.lasers[0] = 0; // no sights in the way
    let f = Renderer::new().render(&g);
    let lines = view_lines(&f);
    let xs: Vec<i16> = lines.iter().flat_map(|l| [l[0], l[2]]).collect();
    let (min, max) = (*xs.iter().min().unwrap(), *xs.iter().max().unwrap());
    assert!(
        (117..=119).contains(&min) && (137..=139).contains(&max),
        "min {min} max {max}"
    );
}

#[test]
fn every_screen_renders_without_panicking() {
    let mut g = Game::new();
    let mut r = Renderer::new();
    let _ = r.render(&g);
    g.dock_at_station();
    for s in [
        Screen::Status,
        Screen::Market,
        Screen::SystemData,
        Screen::GalacticChart,
        Screen::ShortRangeChart,
        Screen::Inventory,
        Screen::Equip,
        Screen::Space,
    ] {
        g.screen = s;
        let f = r.render(&g);
        assert!(!f.lines.is_empty() || !f.glyphs.is_empty());
        for gl in &f.glyphs {
            assert!((0..64).contains(&gl[2]), "glyph code out of range: {gl:?}");
        }
    }
    g.launch();
    g.screen = Screen::Space;
    let f = r.render(&g);
    assert!(f.lines.len() > 10);
}

#[test]
fn title_frame_is_stable() {
    let g = Game::new();
    let a = Renderer::new().render(&g);
    let b = Renderer::new().render(&g);
    assert_eq!(a, b);
}
