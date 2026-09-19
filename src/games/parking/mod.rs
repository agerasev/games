//! Parking lessons with shared physics, geometry, canvas drawing, and egui controls.
pub mod model;
mod ui;
mod view;
use crate::{draw::Painter, pressed_keys};
use model::{Control, NAMES, Round, Status};
pub(crate) use view::preview;
use wgame::canvas::{CanvasInput, Key};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Action {
    Pilot(Control),
    Gear(f32),
    Steering(f32),
    Pause,
    Restart,
    Level(usize),
}
pub struct Game {
    round: Round,
    started: bool,
    paused: bool,
    gear: f32,
    wheel: f32,
    pilot: Control,
    control: Control,
}
impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
impl Game {
    pub fn new() -> Self {
        Self {
            round: Round::new(0),
            started: false,
            paused: true,
            gear: 1.0,
            wheel: 0.0,
            pilot: Control::default(),
            control: Control::default(),
        }
    }
    pub fn round(&self) -> &Round {
        &self.round
    }
    fn reset(&mut self, index: usize) {
        *self = Self {
            round: Round::new(index),
            ..Self::new()
        };
    }
    pub(crate) fn action(&mut self, action: Action) {
        match action {
            Action::Pilot(control) => self.pilot = control,
            Action::Gear(gear) => self.gear = gear,
            Action::Steering(wheel) => self.wheel = wheel,
            Action::Pause => {
                self.started = true;
                self.paused = !self.paused;
                self.control = Control::default();
            }
            Action::Restart => self.reset(self.round.index),
            Action::Level(index) => self.reset(index),
        }
    }
    pub(crate) fn repaint_after(&self) -> Option<std::time::Duration> {
        (!self.paused && self.round.needs_update(self.control)).then_some(std::time::Duration::ZERO)
    }
    pub fn update(&mut self, input: &CanvasInput, dt: f32) {
        if !input.window_focused {
            self.paused = true;
            self.pilot = Control::default();
            self.control = Control::default();
            return;
        }
        for key in pressed_keys(input) {
            match key {
                Key::Character('r') => {
                    self.action(Action::Restart);
                    return;
                }
                Key::Character('p') => {
                    self.action(Action::Pause);
                    return;
                }
                Key::Character('n') if self.round.status == Status::Parked => {
                    self.reset((self.round.index + 1) % NAMES.len());
                    return;
                }
                _ => {}
            }
        }
        let held = |arrow, letter| input.key_down(arrow) || input.key_down(Key::Character(letter));
        let drive = f32::from(held(Key::ArrowUp, 'w')) - f32::from(held(Key::ArrowDown, 's'));
        let steering = f32::from(held(Key::ArrowLeft, 'a')) - f32::from(held(Key::ArrowRight, 'd'));
        self.control = Control {
            drive: if drive == 0.0 {
                self.pilot.drive
            } else {
                drive
            },
            steer: if steering == 0.0 {
                self.wheel
            } else {
                steering
            },
            brake: input.key_down(Key::Space) || self.pilot.brake,
        };
        if !self.started && self.control.drive != 0.0 {
            self.started = true;
            self.paused = false;
        }
        if !self.paused {
            self.round.advance(self.control, dt.min(0.04));
        }
    }
    pub fn draw(&self, painter: &mut Painter<'_>) {
        view::draw(self, painter);
    }
}

#[cfg(test)]
mod input_tests {
    use super::*;
    use wgame::canvas::{Event, InputState, Modifiers};
    fn key(input: &mut InputState, key: Key, pressed: bool) {
        input.push(Event::Key {
            key,
            pressed,
            repeat: false,
        });
    }
    #[test]
    fn driving_braking_and_focus_loss_follow_redraw_policy() {
        let mut game = Game::new();
        let mut input = InputState::default();
        input.push(Event::Focused(true));
        let start = *game.round.car.pos;
        key(&mut input, Key::Character('w'), true);
        for _ in 0..30 {
            game.update(&input.finish(true, true, Modifiers::default()), 1.0 / 60.0);
        }
        assert!(game.round.car.pos.y > start.y && game.round.car.speed > 1.0);
        assert_eq!(game.repaint_after(), Some(std::time::Duration::ZERO));
        key(&mut input, Key::Character('w'), false);
        key(&mut input, Key::Space, true);
        for _ in 0..30 {
            game.update(&input.finish(true, true, Modifiers::default()), 1.0 / 60.0);
        }
        assert_eq!(game.round.car.speed, 0.0);
        assert_eq!(game.repaint_after(), None);
        game.action(Action::Level(3));
        game.action(Action::Pause);
        game.update(&input.finish(true, true, Modifiers::default()), 1.0 / 60.0);
        assert!(
            game.repaint_after().is_some(),
            "traffic needs frames even when stopped"
        );
        input.push(Event::Focused(false));
        game.update(input.input(), 1.0);
        let traffic = game.round.level.traffic[0].body.center;
        assert_eq!(game.repaint_after(), None);
        input.push(Event::Focused(true));
        game.update(&input.finish(true, true, Modifiers::default()), 1.0);
        assert_eq!(game.round.level.traffic[0].body.center, traffic);
        key(&mut input, Key::Character('p'), true);
        game.update(&input.finish(true, true, Modifiers::default()), 0.0);
        game.update(&input.finish(true, true, Modifiers::default()), 1.0 / 60.0);
        assert_ne!(game.round.level.traffic[0].body.center, traffic);
        key(&mut input, Key::Character('r'), true);
        game.update(&input.finish(true, true, Modifiers::default()), 0.0);
        assert_eq!(game.round.index, 3);
        assert_eq!(*game.round.car.pos, game.round.level.start.center);
        assert_eq!(game.repaint_after(), None);
    }
}
