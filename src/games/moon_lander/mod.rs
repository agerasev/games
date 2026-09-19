//! Lunar landing with shared wgame drawing and egui flight controls.
pub mod model;
mod ui;
mod view;

use crate::{draw::Painter, pressed_keys};
use model::{Control, Flight, Status};
pub(crate) use view::preview;
use wgame::canvas::{CanvasInput, Key};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Action {
    Pilot(Control),
    TogglePause,
    Restart,
    Site(usize),
}

pub struct Game {
    flight: Flight,
    started: bool,
    paused: bool,
    pilot: Control,
}
impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
impl Game {
    pub fn new() -> Self {
        Self {
            flight: Flight::new(0),
            started: false,
            paused: true,
            pilot: Control::default(),
        }
    }
    pub fn flight(&self) -> &Flight {
        &self.flight
    }
    pub(crate) fn action(&mut self, action: Action) {
        match action {
            Action::Pilot(control) => self.pilot = control,
            Action::TogglePause => {
                self.started = true;
                self.paused = !self.paused;
                self.flight.power = 0.0;
            }
            Action::Restart => self.reset(self.flight.site),
            Action::Site(site) => self.reset(site),
        }
    }
    fn reset(&mut self, site: usize) {
        self.flight = Flight::new(site);
        self.started = false;
        self.paused = true;
        self.pilot = Control::default();
    }
    pub(crate) fn repaint_after(&self) -> Option<std::time::Duration> {
        (!self.paused
            && (self.flight.status == Status::Flying || !self.flight.particles.is_empty()))
        .then_some(std::time::Duration::ZERO)
    }
    pub fn update(&mut self, input: &CanvasInput, dt: f32) {
        if !input.window_focused {
            self.pilot = Control::default();
            self.paused = true;
            self.flight.power = 0.0;
            return;
        }
        for key in pressed_keys(input) {
            match key {
                Key::Character('r') => {
                    self.reset(self.flight.site);
                    return;
                }
                Key::Character('p') => {
                    self.action(Action::TogglePause);
                    return;
                }
                Key::Character('n') if self.flight.status == Status::Landed => {
                    self.reset((self.flight.site + 1) % Flight::SITES.len());
                    return;
                }
                _ => {}
            }
        }
        let held = |arrow, letter| input.key_down(arrow) || input.key_down(Key::Character(letter));
        let control = Control {
            thrust: self.pilot.thrust || input.key_down(Key::Space) || held(Key::ArrowUp, 'w'),
            turn: (self.pilot.turn + f32::from(held(Key::ArrowLeft, 'a'))
                - f32::from(held(Key::ArrowRight, 'd')))
            .clamp(-1.0, 1.0),
        };
        if !self.started && (control.thrust || control.turn != 0.0) {
            self.started = true;
            self.paused = false;
        }
        if !self.paused {
            self.flight.advance(control, dt);
        }
    }
    pub fn draw(&self, painter: &mut Painter<'_>) {
        view::draw(self, painter);
    }
}

#[cfg(test)]
mod input_tests {
    use super::*;
    use wgame::canvas::{Event, InputState};
    #[test]
    fn keyboard_thrust_and_turn_cancel_without_sticking() {
        let mut game = Game::new();
        let mut input = InputState::default();
        input.push(Event::Focused(true));
        for key in [Key::Space, Key::ArrowLeft] {
            input.push(Event::Key {
                key,
                pressed: true,
                repeat: false,
            });
        }
        game.update(input.input(), 0.2);
        assert!(game.flight.power > 0.0 && *game.flight.craft.angle > 0.0);
        let fuel = game.flight.fuel;
        input.push(Event::Cancelled);
        game.update(input.input(), 0.2);
        assert_eq!(game.flight.power, 0.0);
        assert_eq!(game.flight.fuel, fuel);
    }
}
