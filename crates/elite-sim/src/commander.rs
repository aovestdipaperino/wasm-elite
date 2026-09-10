use serde::{Deserialize, Serialize};

/// Laser types by power value, as in the original.
pub const LASER_NONE: u8 = 0;
pub const LASER_PULSE: u8 = 0x0F;
pub const LASER_BEAM: u8 = 0x8F;
pub const LASER_MILITARY: u8 = 0x97;
pub const LASER_MINING: u8 = 0x32;

pub const MAX_FUEL: u8 = 70;
pub const CARGO_CAPACITY: u8 = 20;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equipment {
    pub ecm: bool,
    pub fuel_scoops: bool,
    pub energy_bomb: bool,
    pub energy_unit: bool,
    pub docking_computer: bool,
    pub escape_pod: bool,
    pub galactic_hyperdrive: bool,
    pub large_cargo_bay: bool,
    /// Laser power per view: front, rear, left, right.
    pub lasers: [u8; 4],
}

/// The persistent commander record, equivalent to the original's save file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commander {
    pub name: String,
    /// Credits in tenths.
    pub credits: u32,
    /// Fuel in tenths of a light year, max 70.
    pub fuel: u8,
    pub galaxy: u8,
    /// Index of the current system in the galaxy.
    pub present: u8,
    /// Index of the hyperspace target.
    pub target: u8,
    pub cargo: [u8; 17],
    pub legal: u8,
    pub kills: u16,
    pub missiles: u8,
    pub equipment: Equipment,
    /// The station's stock at the current system.
    pub stock: [u8; 17],
    /// The market fluctuation byte for this visit.
    pub fluctuation: u8,
}

impl Commander {
    /// Commander Jameson, as every game begins.
    pub fn jameson() -> Self {
        Commander {
            name: "JAMESON".into(),
            credits: 1000,
            fuel: MAX_FUEL,
            galaxy: 0,
            present: 7, // Lave
            target: 7,
            cargo: [0; 17],
            legal: 0,
            kills: 0,
            missiles: 3,
            equipment: Equipment {
                lasers: [LASER_PULSE, 0, 0, 0],
                ..Default::default()
            },
            stock: [0; 17],
            fluctuation: 0,
        }
    }

    pub fn cargo_capacity(&self) -> u8 {
        if self.equipment.large_cargo_bay {
            CARGO_CAPACITY + 15
        } else {
            CARGO_CAPACITY
        }
    }

    /// Tonnes used by cargo. Gold, platinum and gem-stones are in kg and g and do not count.
    pub fn cargo_used(&self) -> u8 {
        self.cargo.iter().take(13).sum::<u8>() + self.cargo[16]
    }
}

/// Combat rating for a kill count.
pub fn rating(kills: u16) -> &'static str {
    match kills {
        0..8 => "Harmless",
        8..16 => "Mostly Harmless",
        16..32 => "Poor",
        32..64 => "Average",
        64..128 => "Above Average",
        128..512 => "Competent",
        512..2560 => "Dangerous",
        2560..6400 => "Deadly",
        _ => "E L I T E",
    }
}

/// Legal status text.
pub fn legal_status(legal: u8) -> &'static str {
    match legal {
        0 => "Clean",
        1..0x40 => "Offender",
        _ => "Fugitive",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jameson_defaults() {
        let c = Commander::jameson();
        assert_eq!(c.credits, 1000);
        assert_eq!(c.fuel, 70);
        assert_eq!(rating(c.kills), "Harmless");
        let json = serde_json::to_string(&c).unwrap();
        let back: Commander = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
    }
}
