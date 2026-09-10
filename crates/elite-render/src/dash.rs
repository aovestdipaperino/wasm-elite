//! The dashboard: bars for speed, roll, pitch, energy, shields, fuel, temperatures, altitude.

use elite_sim::{Game, MAX_SPEED};

use crate::flight_consts::{MAX_PITCH, MAX_ROLL};
use crate::{CYAN, Frame, HEIGHT, RED, VIEW_HEIGHT, WHITE, WIDTH, YELLOW, scanner};

const LEFT_LABEL_X: i16 = 2;
const LEFT_BAR_X: i16 = 20;
const RIGHT_BAR_X: i16 = 176;
const RIGHT_LABEL_X: i16 = 236;
const BAR_W: i16 = 56;

pub fn draw(g: &Game, f: &mut Frame) {
    // Dashboard frame.
    f.line(0, VIEW_HEIGHT, WIDTH - 1, VIEW_HEIGHT, WHITE);
    f.line(0, HEIGHT - 1, WIDTH - 1, HEIGHT - 1, WHITE);
    f.line(0, VIEW_HEIGHT, 0, HEIGHT - 1, WHITE);
    f.line(WIDTH - 1, VIEW_HEIGHT, WIDTH - 1, HEIGHT - 1, WHITE);

    let p = &g.player;
    let mut y = VIEW_HEIGHT + 3;
    let warn = |v: u8, low: u8| if v < low { RED } else { YELLOW };

    // Left column: speed, roll, pitch, then four energy banks.
    left(f, y, "SP", p.speed as i16, MAX_SPEED as i16, YELLOW);
    y += 7;
    centred_bar(f, LEFT_BAR_X, y, p.alpha as i16, MAX_ROLL as i16, "RL");
    y += 7;
    centred_bar(f, LEFT_BAR_X, y, p.beta as i16, MAX_PITCH as i16, "DC");
    y += 8;
    // 255 energy across four banks of 64.
    for bank in (0..4).rev() {
        let level = (p.energy as i16 - bank * 64).clamp(0, 64);
        let colour = if p.energy < 64 { RED } else { YELLOW };
        bar(f, LEFT_BAR_X, y, level, 64, colour);
        f.text(LEFT_LABEL_X, y - 1, &(4 - bank).to_string(), WHITE);
        y += 5;
    }

    // Right column.
    let mut y = VIEW_HEIGHT + 4;
    right(
        f,
        y,
        "FS",
        p.fore_shield as i16,
        255,
        warn(p.fore_shield, 48),
    );
    y += 8;
    right(f, y, "AS", p.aft_shield as i16, 255, warn(p.aft_shield, 48));
    y += 7;
    right(f, y, "FU", g.commander.fuel as i16, 70, YELLOW);
    y += 7;
    right(
        f,
        y,
        "CT",
        p.cabin_temp as i16,
        255,
        if p.cabin_temp > 200 { RED } else { YELLOW },
    );
    y += 7;
    right(
        f,
        y,
        "LT",
        p.laser_temp as i16,
        255,
        if p.laser_temp > 200 { RED } else { YELLOW },
    );
    y += 7;
    right(
        f,
        y,
        "AL",
        p.altitude as i16,
        255,
        if p.altitude < 24 { RED } else { YELLOW },
    );
    y += 7;
    // Missiles as small boxes.
    f.text(RIGHT_LABEL_X, y - 1, "MI", WHITE);
    for i in 0..4 {
        let x = RIGHT_BAR_X + i * 8;
        if i < g.commander.missiles as i16 {
            f.fill(x, y, 6, 5, CYAN);
        } else {
            f.line(x, y, x + 5, y, WHITE);
            f.line(x, y + 4, x + 5, y + 4, WHITE);
            f.line(x, y, x, y + 4, WHITE);
            f.line(x + 5, y, x + 5, y + 4, WHITE);
        }
    }

    scanner::draw(g, f);

    // Hyperspace countdown, top left of the view.
    if p.hyper_countdown > 0 {
        f.text(4, 4, &(p.hyper_countdown / 2 + 1).to_string(), WHITE);
    }
    if p.jumping {
        f.text(WIDTH - 4 * 8 - 4, 4, "JUMP", WHITE);
    }
}

fn left(f: &mut Frame, y: i16, label: &str, v: i16, max: i16, colour: i16) {
    f.text(LEFT_LABEL_X, y - 1, label, WHITE);
    bar(f, LEFT_BAR_X, y, v, max, colour);
}

fn right(f: &mut Frame, y: i16, label: &str, v: i16, max: i16, colour: i16) {
    f.text(RIGHT_LABEL_X, y - 1, label, WHITE);
    bar(f, RIGHT_BAR_X, y, v, max, colour);
}

fn bar(f: &mut Frame, x: i16, y: i16, v: i16, max: i16, colour: i16) {
    let w = (v.clamp(0, max) as i32 * BAR_W as i32 / max.max(1) as i32) as i16;
    f.line(x, y + 5, x + BAR_W, y + 5, WHITE);
    if w > 0 {
        f.fill(x, y, w, 4, colour);
    }
}

/// A bar with its zero in the middle, for roll and pitch.
fn centred_bar(f: &mut Frame, x: i16, y: i16, v: i16, max: i16, label: &str) {
    f.text(LEFT_LABEL_X, y - 1, label, WHITE);
    let mid = x + BAR_W / 2;
    f.line(x, y + 5, x + BAR_W, y + 5, WHITE);
    f.line(mid, y - 1, mid, y + 5, WHITE);
    let dx = (v.clamp(-max, max) as i32 * (BAR_W / 2) as i32 / max.max(1) as i32) as i16;
    f.fill(mid + dx - 1, y, 3, 4, YELLOW);
}
