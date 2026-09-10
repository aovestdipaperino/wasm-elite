/// Every key the game reacts to. The numeric value is the code exchanged with the shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Key {
    RollLeft = 0,
    RollRight,
    PitchUp,
    PitchDown,
    SpeedUp,
    SlowDown,
    Fire,
    Jump,
    Hyperspace,
    Dock,
    ViewFront,
    ViewRear,
    ViewLeft,
    ViewRight,
    Launch,
    Buy,
    Sell,
    Equip,
    GalacticChart,
    ShortRangeChart,
    SystemData,
    Market,
    Status,
    Inventory,
    Up,
    Down,
    Left,
    Right,
    Select,
    Back,
    FindHome,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
}

pub const KEY_COUNT: usize = Key::Digit9 as usize + 1;

impl Key {
    pub fn from_code(c: u8) -> Option<Key> {
        if (c as usize) < KEY_COUNT {
            // SAFETY-free: match by table.
            Some(ALL_KEYS[c as usize])
        } else {
            None
        }
    }

    pub fn digit(self) -> Option<u8> {
        let d = self as i32 - Key::Digit0 as i32;
        (0..=9).contains(&d).then_some(d as u8)
    }
}

const ALL_KEYS: [Key; KEY_COUNT] = [
    Key::RollLeft,
    Key::RollRight,
    Key::PitchUp,
    Key::PitchDown,
    Key::SpeedUp,
    Key::SlowDown,
    Key::Fire,
    Key::Jump,
    Key::Hyperspace,
    Key::Dock,
    Key::ViewFront,
    Key::ViewRear,
    Key::ViewLeft,
    Key::ViewRight,
    Key::Launch,
    Key::Buy,
    Key::Sell,
    Key::Equip,
    Key::GalacticChart,
    Key::ShortRangeChart,
    Key::SystemData,
    Key::Market,
    Key::Status,
    Key::Inventory,
    Key::Up,
    Key::Down,
    Key::Left,
    Key::Right,
    Key::Select,
    Key::Back,
    Key::FindHome,
    Key::Digit0,
    Key::Digit1,
    Key::Digit2,
    Key::Digit3,
    Key::Digit4,
    Key::Digit5,
    Key::Digit6,
    Key::Digit7,
    Key::Digit8,
    Key::Digit9,
];

/// Key state for one step: `held` is the current state, `pressed` is set for keys that went
/// down since the last step (edge events).
#[derive(Clone, Copy, Debug)]
pub struct Input {
    held: [bool; KEY_COUNT],
    pressed: [bool; KEY_COUNT],
}

impl Default for Input {
    fn default() -> Self {
        Input {
            held: [false; KEY_COUNT],
            pressed: [false; KEY_COUNT],
        }
    }
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a key transition from the shell.
    pub fn set(&mut self, k: Key, down: bool) {
        let i = k as usize;
        if down && !self.held[i] {
            self.pressed[i] = true;
        }
        self.held[i] = down;
    }

    /// Tap a key (down and up within one step).
    pub fn tap(&mut self, k: Key) {
        self.pressed[k as usize] = true;
    }

    pub fn held(&self, k: Key) -> bool {
        self.held[k as usize]
    }

    pub fn pressed(&self, k: Key) -> bool {
        self.pressed[k as usize]
    }

    /// Forget edge events after they have been consumed by a step.
    pub fn end_step(&mut self) {
        self.pressed = [false; KEY_COUNT];
    }

    pub fn pressed_digit(&self) -> Option<u8> {
        ALL_KEYS
            .iter()
            .filter(|k| self.pressed(**k))
            .find_map(|k| k.digit())
    }
}
