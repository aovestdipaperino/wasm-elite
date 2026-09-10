use elite_math::{Orient, Vec3};
use elite_ships::Blueprint;
use serde::{Deserialize, Serialize};

/// Every ship type that can occupy a slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipKind {
    Coriolis,
    Dodo,
    Missile,
    EscapePod,
    Canister,
    Asteroid,
    Boulder,
    Splinter,
    Sidewinder,
    Mamba,
    Krait,
    Adder,
    Gecko,
    CobraMk1,
    CobraMk3,
    CobraMk3Pirate,
    Worm,
    Viper,
    Python,
    PythonPirate,
    Boa,
    Anaconda,
    AspMk2,
    FerDeLance,
    Moray,
    Thargoid,
    Thargon,
    Constrictor,
    Transporter,
    Shuttle,
}

impl ShipKind {
    pub fn blueprint(self) -> &'static Blueprint {
        use elite_ships as b;
        match self {
            ShipKind::Coriolis => &b::CORIOLIS,
            ShipKind::Dodo => &b::DODO,
            ShipKind::Missile => &b::MISSILE,
            ShipKind::EscapePod => &b::ESCAPE_POD,
            ShipKind::Canister => &b::CANISTER,
            ShipKind::Asteroid => &b::ASTEROID,
            ShipKind::Boulder => &b::BOULDER,
            ShipKind::Splinter => &b::SPLINTER,
            ShipKind::Sidewinder => &b::SIDEWINDER,
            ShipKind::Mamba => &b::MAMBA,
            ShipKind::Krait => &b::KRAIT,
            ShipKind::Adder => &b::ADDER,
            ShipKind::Gecko => &b::GECKO,
            ShipKind::CobraMk1 => &b::COBRA_MK_1,
            ShipKind::CobraMk3 => &b::COBRA_MK_3,
            ShipKind::CobraMk3Pirate => &b::COBRA_MK_3_P,
            ShipKind::Worm => &b::WORM,
            ShipKind::Viper => &b::VIPER,
            ShipKind::Python => &b::PYTHON,
            ShipKind::PythonPirate => &b::PYTHON_P,
            ShipKind::Boa => &b::BOA,
            ShipKind::Anaconda => &b::ANACONDA,
            ShipKind::AspMk2 => &b::ASP_MK_2,
            ShipKind::FerDeLance => &b::FER_DE_LANCE,
            ShipKind::Moray => &b::MORAY,
            ShipKind::Thargoid => &b::THARGOID,
            ShipKind::Thargon => &b::THARGON,
            ShipKind::Constrictor => &b::CONSTRICTOR,
            ShipKind::Transporter => &b::TRANSPORTER,
            ShipKind::Shuttle => &b::SHUTTLE,
        }
    }

    pub fn is_station(self) -> bool {
        matches!(self, ShipKind::Coriolis | ShipKind::Dodo)
    }

    /// Pirate ship types spawned in M1.
    pub const PIRATES: [ShipKind; 8] = [
        ShipKind::Sidewinder,
        ShipKind::Mamba,
        ShipKind::Krait,
        ShipKind::Adder,
        ShipKind::Gecko,
        ShipKind::CobraMk1,
        ShipKind::CobraMk3Pirate,
        ShipKind::Worm,
    ];
}

/// Behaviour flags for a ship in a slot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiFlags {
    /// Actively hunts the player.
    pub hostile: bool,
    /// Runs tactics at all (asteroids and canisters do not).
    pub active: bool,
    /// Low on energy, trying to escape.
    pub fleeing: bool,
}

/// One object in the universe, expressed in the player's frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShipSlot {
    pub kind: ShipKind,
    pub pos: Vec3,
    pub orient: Orient,
    pub speed: u8,
    pub energy: u8,
    /// Own roll and pitch rates applied every step.
    pub roll: i8,
    pub pitch: i8,
    pub ai: AiFlags,
    pub missiles: u8,
    /// Steps remaining of the explosion, once destroyed.
    pub exploding: Option<u8>,
    /// Set by the player's laser this step, for the renderer's hit flash.
    pub hit_flash: bool,
}

impl ShipSlot {
    pub fn new(kind: ShipKind, pos: Vec3, orient: Orient) -> Self {
        let b = kind.blueprint();
        ShipSlot {
            kind,
            pos,
            orient,
            speed: 0,
            energy: b.energy,
            roll: 0,
            pitch: 0,
            ai: AiFlags::default(),
            missiles: b.missiles,
            exploding: None,
            hit_flash: false,
        }
    }

    pub fn blueprint(&self) -> &'static Blueprint {
        self.kind.blueprint()
    }

    pub fn is_alive(&self) -> bool {
        self.exploding.is_none()
    }

    pub fn distance(&self) -> u32 {
        self.pos.length()
    }
}
