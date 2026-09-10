//! Title, status, market, data, inventory, equip and death screens, plus shared decorations.

use elite_core::{COMMODITIES, ECONOMIES, GOVERNMENTS};
use elite_sim::commander::{LASER_BEAM, LASER_MILITARY, LASER_MINING, LASER_PULSE, legal_status};
use elite_sim::{Game, Mode, View, rating};

use crate::text::{Text, credits, heading, light_years};
use crate::{CYAN, Frame, GREEN, VIEW_HEIGHT, WHITE, WIDTH, YELLOW, wire};

pub fn frame_border(f: &mut Frame) {
    f.line(0, 0, WIDTH - 1, 0, WHITE);
    f.line(0, 0, 0, VIEW_HEIGHT - 1, WHITE);
    f.line(WIDTH - 1, 0, WIDTH - 1, VIEW_HEIGHT - 1, WHITE);
}

pub fn message(g: &Game, f: &mut Frame) {
    if let Some(m) = &g.message {
        f.text_centred(VIEW_HEIGHT - 14, &m.text.to_ascii_uppercase(), WHITE);
    }
}

pub fn title(g: &Game, f: &mut Frame) {
    frame_border(f);
    f.line(0, VIEW_HEIGHT, WIDTH - 1, VIEW_HEIGHT, WHITE);
    f.text_centred(10, "---- E L I T E ----", YELLOW);
    if let Some(cobra) = g.universe.ships.first() {
        wire::draw_ship(cobra, View::Front, f, g.step);
    }
    f.text_centred(160, "PRESS FIRE OR SPACE, COMMANDER.", WHITE);
    let mut t = Text::new(0, VIEW_HEIGHT + 4);
    t.line_height = 8;
    t.line(f, "A FAN REMAKE OF ELITE (1984) BY");
    t.line(f, "IAN BELL AND DAVID BRABEN.");
    t.line(f, "RUST + WEBASSEMBLY, UNOFFICIAL.");
    t.line(f, "S/X PITCH </> ROLL SPACE/? SPEED");
    t.line(f, "A FIRE  H HYPERSPACE  J JUMP");
    t.line(f, "F1-F4 VIEWS   1 = LAUNCH");
}

pub fn death(g: &Game, f: &mut Frame) {
    if g.death_timer > 20 {
        f.text_centred(84, "GAME OVER", YELLOW);
    }
    if g.death_timer > 40 {
        f.text_centred(100, "PRESS FIRE OR SPACE", WHITE);
    }
}

/// The docked view of the space window: the station's interior message.
pub fn docked_space(g: &Game, f: &mut Frame) {
    let _ = g;
    f.text_centred(80, "DOCKED", WHITE);
    f.text_centred(100, "PRESS 1 TO LAUNCH", WHITE);
}

pub fn status(g: &Game, f: &mut Frame) {
    let c = &g.commander;
    heading(f, &format!("COMMANDER {}", c.name));
    let mut t = Text::new(8, 24);
    t.pair(
        f,
        "Present System   : ",
        &g.present().name.to_ascii_uppercase(),
    );
    t.pair(
        f,
        "Hyperspace System: ",
        &g.target().name.to_ascii_uppercase(),
    );
    let condition = match g.mode {
        Mode::Docked => "Docked",
        _ if g.universe.hostiles() > 0 => "Red",
        _ if g.universe.ships.iter().any(|s| s.is_alive()) => "Yellow",
        _ => "Green",
    };
    t.pair(f, "Condition        : ", condition);
    t.pair(f, "Fuel: ", &light_years(c.fuel as u16));
    t.pair(f, "Cash: ", &credits(c.credits));
    t.pair(f, "Legal Status: ", legal_status(c.legal));
    t.pair(f, "Rating: ", rating(c.kills));
    t.blank();
    t.line(f, "EQUIPMENT:");
    let e = &c.equipment;
    let mut items: Vec<String> = Vec::new();
    if e.large_cargo_bay {
        items.push("Large Cargo Bay".into());
    }
    if e.ecm {
        items.push("E.C.M. System".into());
    }
    if e.fuel_scoops {
        items.push("Fuel Scoops".into());
    }
    if e.escape_pod {
        items.push("Escape Pod".into());
    }
    if e.energy_bomb {
        items.push("Energy Bomb".into());
    }
    if e.energy_unit {
        items.push("Extra Energy Unit".into());
    }
    if e.docking_computer {
        items.push("Docking Computers".into());
    }
    if e.galactic_hyperdrive {
        items.push("Galactic Hyperspace".into());
    }
    for (i, name) in ["Front", "Rear", "Left", "Right"].iter().enumerate() {
        let laser = match e.lasers[i] {
            0 => continue,
            LASER_PULSE => "Pulse Laser",
            LASER_BEAM => "Beam Laser",
            LASER_MILITARY => "Military Laser",
            LASER_MINING => "Mining Laser",
            _ => "Laser",
        };
        items.push(format!("{name} {laser}"));
    }
    if c.missiles > 0 {
        items.push(format!(
            "{} Missile{}",
            c.missiles,
            if c.missiles > 1 { "s" } else { "" }
        ));
    }
    let mut t2 = Text::new(24, t.y);
    for item in items {
        t2.line(f, &item);
    }
}

pub fn market(g: &Game, f: &mut Frame) {
    heading(
        f,
        &format!("{} MARKET PRICES", g.present().name.to_ascii_uppercase()),
    );
    f.text(8, 22, "PRODUCT", WHITE);
    f.text(112, 22, "UNIT", WHITE);
    f.text(152, 22, "PRICE", WHITE);
    f.text(208, 22, "QTY", WHITE);
    let quotes = g.market_prices();
    let docked = g.mode == Mode::Docked;
    for (i, (c, q)) in COMMODITIES.iter().zip(quotes.iter()).enumerate() {
        let y = 32 + 8 * i as i16;
        let selected = docked && g.market_cursor as usize == i;
        let colour = if selected { YELLOW } else { WHITE };
        if selected {
            f.text(0, y, ">", YELLOW);
        }
        f.text(8, y, &c.name.to_ascii_uppercase(), colour);
        f.text(112, y, c.unit, colour);
        f.text(
            152,
            y,
            &format!("{:>5}", credits(q.price as u32).trim_end_matches(" Cr")),
            colour,
        );
        let qty = if q.quantity == 0 {
            "-".to_string()
        } else {
            format!("{}{}", q.quantity, c.unit)
        };
        f.text(208, y, &qty, colour);
        let held = g.commander.cargo[i];
        if held > 0 {
            f.text(240, y, &held.to_string(), GREEN);
        }
    }
    let footer = if docked {
        format!("CASH {}  B BUY  S SELL", credits(g.commander.credits))
    } else {
        format!("CASH {}", credits(g.commander.credits))
    };
    f.text_centred(VIEW_HEIGHT - 16, &footer, CYAN);
}

pub fn inventory(g: &Game, f: &mut Frame) {
    heading(f, "INVENTORY");
    let mut t = Text::new(8, 24);
    t.pair(f, "Fuel: ", &light_years(g.commander.fuel as u16));
    t.pair(f, "Cash: ", &credits(g.commander.credits));
    t.blank();
    let mut any = false;
    for (i, c) in COMMODITIES.iter().enumerate() {
        let n = g.commander.cargo[i];
        if n > 0 {
            any = true;
            t.line(
                f,
                &format!("{:<16}{}{}", c.name.to_ascii_uppercase(), n, c.unit),
            );
        }
    }
    if !any {
        t.line(f, "CARGO HOLD EMPTY");
    }
    t.blank();
    t.line(
        f,
        &format!(
            "{} OF {}T USED",
            g.commander.cargo_used(),
            g.commander.cargo_capacity()
        ),
    );
}

pub fn equip(g: &Game, f: &mut Frame) {
    heading(f, "EQUIP SHIP");
    let mut t = Text::new(8, 24);
    t.pair(f, "Cash: ", &credits(g.commander.credits));
    t.blank();
    t.line(f, "1  FUEL            0.2 CR PER 0.1 LY");
    t.line(f, "2  MISSILE                  30.0 CR");
    t.blank();
    t.line(f, "OTHER EQUIPMENT ARRIVES IN A LATER");
    t.line(f, "RELEASE. PRESS THE NUMBER TO BUY.");
}

pub fn system_data(g: &Game, f: &mut Frame) {
    let s = g.target();
    heading(f, &format!("DATA ON {}", s.name.to_ascii_uppercase()));
    let mut t = Text::new(8, 24);
    t.pair(f, "Distance: ", &light_years(g.distance_to_target()));
    t.pair(f, "Economy: ", ECONOMIES[s.economy as usize]);
    t.pair(f, "Government: ", GOVERNMENTS[s.government as usize]);
    t.pair(f, "Tech. Level: ", &(s.tech_level + 1).to_string());
    t.pair(
        f,
        "Population: ",
        &format!("{}.{} Billion", s.population / 10, s.population % 10),
    );
    t.pair(
        f,
        "Gross Productivity: ",
        &format!("{} M CR", s.productivity),
    );
    t.pair(f, "Average Radius: ", &format!("{} km", s.radius));
    t.blank();
    t.paragraph(f, &s.describe(), 30);
}
