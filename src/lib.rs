#![forbid(unsafe_code)]

pub mod draw;
pub mod games;
pub mod layout;

pub mod ui;

use draw::Painter;
use games::{Game, GameId};
use wgame::{
    canvas::{Button, CanvasInput, Event, Key},
    glam::Vec2,
};

fn current_events(input: &CanvasInput) -> &[Event] {
    let start = input
        .events
        .iter()
        .rposition(|event| matches!(event, Event::Cancelled))
        .map_or(0, |i| i + 1);
    &input.events[start..]
}
pub fn pressed_keys(input: &CanvasInput) -> impl Iterator<Item = Key> + '_ {
    current_events(input)
        .iter()
        .filter_map(|event| match event {
            Event::Key {
                key,
                pressed: true,
                repeat: false,
            } => Some(*key),
            _ => None,
        })
}
pub fn clicks(input: &CanvasInput) -> impl Iterator<Item = Vec2> + '_ {
    current_events(input)
        .iter()
        .filter_map(|event| match event {
            Event::Button {
                button: Button::Primary,
                pressed: true,
                position,
            } => Some(*position),
            _ => None,
        })
}

/// CPU game state shared by native and browser hosts. A transition consumes the
/// input frame so selecting a game cannot also activate one of its controls.
pub struct App {
    active: Option<Game>,
    selection: usize,
}
impl App {
    pub fn new(game: Option<GameId>) -> Self {
        Self {
            active: game.map(Game::new),
            selection: 0,
        }
    }
    pub fn active_id(&self) -> Option<GameId> {
        self.active.as_ref().map(Game::id)
    }
    /// Returns false when Escape is pressed in the launcher.
    pub fn update(&mut self, input: &CanvasInput, dt: f32, size: Vec2) -> bool {
        if pressed_keys(input).any(|key| key == Key::Escape) {
            return self.active.take().is_some();
        }
        if let Some(game) = &mut self.active {
            game.update(input, dt, size);
        } else {
            let mut launch = None;
            for key in pressed_keys(input) {
                match key {
                    Key::ArrowRight | Key::ArrowDown | Key::Tab => {
                        self.selection = (self.selection + 1) % GameId::ALL.len()
                    }
                    Key::ArrowLeft | Key::ArrowUp => {
                        self.selection =
                            (self.selection + GameId::ALL.len() - 1) % GameId::ALL.len()
                    }
                    Key::Enter | Key::Space => launch = Some(self.selection),
                    Key::Character(c @ '1'..='4') => launch = Some(c as usize - '1' as usize),
                    _ => {}
                }
            }
            if let Some(index) = launch {
                self.selection = index;
                self.active = Some(Game::new(GameId::ALL[index]));
            }
        }
        true
    }
    pub fn draw(&self, painter: &mut Painter<'_>) {
        if let Some(game) = &self.active {
            game.draw(painter);
        }
    }
}

#[cfg(test)]
mod tests;
