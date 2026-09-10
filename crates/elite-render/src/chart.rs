//! Galactic and short-range charts.

use elite_sim::Game;

use crate::text::{heading, light_years};
use crate::{CYAN, Frame, GREEN, WHITE, YELLOW, scanner};

const GAL_TOP: i16 = 32;

pub fn galactic(g: &Game, f: &mut Frame) {
    heading(f, &format!("GALACTIC CHART {}", g.commander.galaxy + 1));
    for s in &g.systems {
        f.dot(s.x as i16, GAL_TOP + (s.y / 2) as i16, WHITE);
    }
    let here = g.present();
    let (hx, hy) = (here.x as i16, GAL_TOP + (here.y / 2) as i16);
    // Fuel range as an ellipse (the chart's y axis is squashed).
    let r = g.commander.fuel as i16 / 4;
    scanner::ellipse(f, hx, hy, r, r / 2, GREEN);
    cross(
        f,
        g.chart.x as i16,
        GAL_TOP + (g.chart.y / 2) as i16,
        4,
        YELLOW,
    );
    cross(f, hx, hy, 2, GREEN);
    footer(g, f);
}

pub fn short_range(g: &Game, f: &mut Frame) {
    heading(f, "SHORT RANGE CHART");
    let here = g.present();
    let scale = 4;
    let to_screen = |x: u8, y: u8| -> (i16, i16) {
        let dx = x as i16 - here.x as i16;
        let dy = y as i16 - here.y as i16;
        (128 + dx * scale, 104 + dy * scale / 2)
    };
    let r = g.commander.fuel as i16;
    scanner::ellipse(f, 128, 104, r, r / 2, GREEN);
    let mut used_rows: Vec<i16> = Vec::new();
    for s in &g.systems {
        let (sx, sy) = to_screen(s.x, s.y);
        if !(2..254).contains(&sx) || !(22..180).contains(&sy) {
            continue;
        }
        let size = (s.radius / 1200) as i16;
        f.fill(
            sx - size / 2,
            sy - size / 2,
            size.max(1),
            size.max(1),
            WHITE,
        );
        // Avoid printing names on top of each other.
        let mut ty = sy - 3;
        while used_rows.iter().any(|&u| (u - ty).abs() < 8) {
            ty += 8;
        }
        if ty < 180 {
            used_rows.push(ty);
            f.text(
                (sx + size / 2 + 3).min(255 - 8 * s.name.len() as i16),
                ty,
                &s.name.to_ascii_uppercase(),
                CYAN,
            );
        }
    }
    let (cx, cy) = to_screen(g.chart.x, g.chart.y);
    cross(f, cx.clamp(2, 253), cy.clamp(22, 180), 4, YELLOW);
    footer(g, f);
}

fn footer(g: &Game, f: &mut Frame) {
    let t = g.target();
    let d = g.distance_to_target();
    let line = format!("{}  {}", t.name.to_ascii_uppercase(), light_years(d));
    f.text_centred(182, &line, YELLOW);
}

fn cross(f: &mut Frame, x: i16, y: i16, r: i16, colour: i16) {
    f.line(x - r, y, x + r, y, colour);
    f.line(x, y - r, x, y + r, colour);
}
