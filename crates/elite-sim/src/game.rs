use elite_core::System;
use elite_math::Rng;
use serde::{Deserialize, Serialize};

use crate::commander::Commander;
use crate::input::{Input, Key};
use crate::universe::Universe;

/// Which cockpit window the player is looking through.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum View {
    Front,
    Rear,
    Left,
    Right,
}

impl View {
    pub fn index(self) -> usize {
        match self {
            View::Front => 0,
            View::Rear => 1,
            View::Left => 2,
            View::Right => 3,
        }
    }
}

/// What the main display area is showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen {
    Space,
    Status,
    Market,
    SystemData,
    GalacticChart,
    ShortRangeChart,
    Inventory,
    Equip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    /// The rotating Cobra and the "load commander" prompt.
    Title,
    Docked,
    Flight,
    /// Game over; `Death` shows debris until the player presses a key.
    Death,
}

/// The player's ship and its transient state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    /// Roll rate, positive = roll right, in 1/512 radian per step.
    pub alpha: i8,
    /// Pitch rate, positive = nose up.
    pub beta: i8,
    pub speed: u8,
    pub energy: u8,
    pub fore_shield: u8,
    pub aft_shield: u8,
    pub laser_temp: u8,
    pub cabin_temp: u8,
    /// Altitude above the planet surface, saturated to 255 (in 2^12 units).
    pub altitude: u8,
    pub view: View,
    /// Steps until the hyperspace jump happens; 0 = not counting.
    pub hyper_countdown: u8,
    /// Set while the in-system jump is being used.
    pub jumping: bool,
    /// Steps since the laser last fired (pulse laser rate limit).
    pub laser_cooldown: u8,
    /// Set for the frame the laser fired, for rendering.
    pub laser_fired: bool,
    /// Frames of damage flash left.
    pub damage_flash: u8,
}

impl Player {
    pub fn fresh() -> Self {
        Player {
            alpha: 0,
            beta: 0,
            speed: 0,
            energy: 255,
            fore_shield: 255,
            aft_shield: 255,
            laser_temp: 0,
            cabin_temp: 30,
            altitude: 255,
            view: View::Front,
            hyper_countdown: 0,
            jumping: false,
            laser_cooldown: 0,
            laser_fired: false,
            damage_flash: 0,
        }
    }
}

/// Cursor and selection state on the charts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chart {
    /// Cursor in galactic chart coordinates (0..256, 0..256 with y halved on screen).
    pub x: u8,
    pub y: u8,
}

/// Transient message shown on the dashboard's message line.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub text: String,
    pub ttl: u8,
}

/// The whole game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub commander: Commander,
    pub universe: Universe,
    pub player: Player,
    pub mode: Mode,
    pub screen: Screen,
    pub chart: Chart,
    pub rng: Rng,
    /// Step counter, used for pacing (tidy every 8, tactics round robin, pulse laser rate).
    pub step: u32,
    /// Selected market item on the market screen.
    pub market_cursor: u8,
    pub message: Option<Message>,
    /// Steps of the death sequence elapsed.
    pub death_timer: u16,
    /// Set for one step when the game wants the shell to persist the commander.
    pub save_requested: bool,
    /// Cached systems of the current galaxy.
    #[serde(skip)]
    pub systems: Vec<System>,
}

impl Game {
    pub fn new() -> Self {
        Self::with_commander(Commander::jameson())
    }

    pub fn with_commander(commander: Commander) -> Self {
        let systems = elite_core::galaxy(commander.galaxy);
        let mut g = Game {
            chart: Chart {
                x: systems[commander.present as usize].x,
                y: systems[commander.present as usize].y,
            },
            commander,
            universe: Universe::empty(),
            player: Player::fresh(),
            mode: Mode::Title,
            screen: Screen::Space,
            rng: Rng::new(0x5A4D_1E17),
            step: 0,
            market_cursor: 0,
            message: None,
            death_timer: 0,
            save_requested: false,
            systems,
        };
        if g.commander.stock.iter().all(|&s| s == 0) {
            g.restock();
        }
        g
    }

    /// Rebuild the systems cache after loading a commander.
    pub fn reload_systems(&mut self) {
        self.systems = elite_core::galaxy(self.commander.galaxy);
    }

    pub fn present(&self) -> &System {
        &self.systems[self.commander.present as usize]
    }

    pub fn target(&self) -> &System {
        &self.systems[self.commander.target as usize]
    }

    pub fn say(&mut self, text: impl Into<String>) {
        self.message = Some(Message {
            text: text.into(),
            ttl: 60,
        });
    }

    /// Advance the game by one step with the given input.
    pub fn step(&mut self, input: &Input) {
        self.step = self.step.wrapping_add(1);
        self.save_requested = false;
        if let Some(m) = &mut self.message {
            m.ttl = m.ttl.saturating_sub(1);
            if m.ttl == 0 {
                self.message = None;
            }
        }
        match self.mode {
            Mode::Title => self.step_title(input),
            Mode::Docked => self.step_docked(input),
            Mode::Flight => self.step_flight(input),
            Mode::Death => self.step_death(input),
        }
    }

    fn step_title(&mut self, input: &Input) {
        // Spin a Cobra for the renderer: the universe holds one ship.
        if self.universe.ships.is_empty() {
            let mut cobra = crate::ship::ShipSlot::new(
                crate::ship::ShipKind::CobraMk3,
                elite_math::Vec3::new(0, 0, 900),
                elite_math::Orient::identity(),
            );
            cobra.roll = 3;
            cobra.pitch = 1;
            self.universe.ships.push(cobra);
        }
        let cobra = &mut self.universe.ships[0];
        cobra.orient.rotate_own(cobra.roll, cobra.pitch);
        if self.step.is_multiple_of(8) {
            cobra.orient.tidy();
        }
        if input.pressed(Key::Select) || input.pressed(Key::Fire) || input.pressed(Key::Launch) {
            self.dock_at_station();
        }
    }

    /// Enter the docked state at the current system.
    pub fn dock_at_station(&mut self) {
        self.mode = Mode::Docked;
        self.screen = Screen::Status;
        self.universe = Universe::empty();
        self.player = Player::fresh();
        self.player.view = View::Front;
        self.save_requested = true;
    }

    fn step_docked(&mut self, input: &Input) {
        if input.pressed(Key::Launch) {
            self.launch();
            return;
        }
        self.handle_screen_keys(input, true);
        match self.screen {
            Screen::Market => self.market_keys(input),
            Screen::Equip => self.equip_keys(input),
            Screen::GalacticChart | Screen::ShortRangeChart => self.chart_keys(input),
            _ => {}
        }
    }

    fn step_death(&mut self, input: &Input) {
        self.death_timer = self.death_timer.saturating_add(1);
        // Debris drifts past: keep moving the universe with the player's last motion.
        self.player.alpha = 0;
        self.player.beta = 0;
        self.player.speed = 4;
        crate::flight::move_universe(self);
        if self.death_timer > 40 && (input.pressed(Key::Select) || input.pressed(Key::Fire)) {
            *self = Game::new();
        }
    }

    /// Common screen switching keys. In flight the charts and status pages are allowed too.
    pub(crate) fn handle_screen_keys(&mut self, input: &Input, docked: bool) {
        let mut next = None;
        if input.pressed(Key::Status) {
            next = Some(Screen::Status);
        }
        if input.pressed(Key::Market) {
            next = Some(Screen::Market);
        }
        if input.pressed(Key::SystemData) {
            next = Some(Screen::SystemData);
        }
        if input.pressed(Key::GalacticChart) {
            next = Some(Screen::GalacticChart);
        }
        if input.pressed(Key::ShortRangeChart) {
            next = Some(Screen::ShortRangeChart);
        }
        if input.pressed(Key::Inventory) {
            next = Some(Screen::Inventory);
        }
        if input.pressed(Key::Equip) && docked {
            next = Some(Screen::Equip);
        }
        if input.pressed(Key::FindHome) {
            self.chart.x = self.present().x;
            self.chart.y = self.present().y;
            self.commander.target = self.commander.present;
        }
        if let Some(s) = next {
            self.screen = s;
        }
    }

    pub(crate) fn chart_keys(&mut self, input: &Input) {
        let fast = if input.held(Key::Fire) { 4 } else { 1 };
        let step = if self.screen == Screen::ShortRangeChart {
            1
        } else {
            2
        } * fast;
        if input.held(Key::Left) || input.held(Key::RollLeft) {
            self.chart.x = self.chart.x.saturating_sub(step);
        }
        if input.held(Key::Right) || input.held(Key::RollRight) {
            self.chart.x = self.chart.x.saturating_add(step);
        }
        if input.held(Key::Up) || input.held(Key::PitchDown) {
            self.chart.y = self.chart.y.saturating_sub(step);
        }
        if input.held(Key::Down) || input.held(Key::PitchUp) {
            self.chart.y = self.chart.y.saturating_add(step);
        }
        // The target is the system nearest the cursor.
        let (cx, cy) = (self.chart.x as i32, self.chart.y as i32);
        let mut best = (i32::MAX, self.commander.target);
        for (i, s) in self.systems.iter().enumerate() {
            let d = (s.x as i32 - cx).pow(2) + ((s.y as i32 - cy) / 2).pow(2);
            if d < best.0 {
                best = (d, i as u8);
            }
        }
        self.commander.target = best.1;
    }

    pub fn distance_to_target(&self) -> u16 {
        crate::hyperspace::distance_ly(self.present(), self.target())
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
