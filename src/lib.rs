#![forbid(unsafe_code)]

pub mod draw;
pub mod games;
pub mod layout;

use draw::{Painter, Sprite};
use games::{Game, GameId};
use layout::{contains, grid, rect};
use wgame::{
    canvas::{Button, CanvasInput, Event, Key},
    gfx::types::color,
    glam::{Vec2, Vec3},
};

const FOOTER: f32 = 40.0;
const MENU_ASPECT: f32 = 1.8;

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
    pointer: Option<Vec2>,
}
impl App {
    pub fn new(game: Option<GameId>) -> Self {
        Self {
            active: game.map(Game::new),
            selection: 0,
            pointer: None,
        }
    }
    pub fn active_id(&self) -> Option<GameId> {
        self.active.as_ref().map(Game::id)
    }
    /// Returns false when Escape is pressed in the launcher.
    pub fn update(&mut self, input: &CanvasInput, dt: f32, size: Vec2) -> bool {
        self.pointer = input.pointer;
        let content = content_size(size);
        let back = rect(8.0, content.y + 4.0, 100.0_f32.min(size.x), 32.0);
        if pressed_keys(input).any(|key| key == Key::Escape)
            || (self.active.is_some() && clicks(input).any(|pos| contains(back, pos)))
        {
            return self.active.take().is_some();
        }
        if let Some(game) = &mut self.active {
            game.update(input, dt, content);
        } else {
            let boxes = grid(content, GameId::ALL.len(), MENU_ASPECT);
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
            for pos in clicks(input) {
                if let Some(index) = boxes.iter().position(|&cell| contains(cell, pos)) {
                    launch = Some(index);
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
        let size = painter.size;
        painter.size = content_size(size);
        if let Some(game) = &self.active {
            game.draw(painter);
        } else {
            for (index, (id, cell)) in GameId::ALL
                .into_iter()
                .zip(grid(painter.size, GameId::ALL.len(), MENU_ASPECT))
                .enumerate()
            {
                let cell = cell.inflate(
                    -cell.size.width.min(8.0) / 2.0,
                    -cell.size.height.min(8.0) / 2.0,
                );
                let hovered = self.pointer.is_some_and(|p| contains(cell, p));
                painter.button(cell, self.selection == index, hovered);
                let side = (cell.size.height * 0.7).min(cell.size.width * 0.7);
                let preview = rect(
                    cell.center().x - side / 2.0,
                    cell.min_y() + cell.size.height * 0.04,
                    side,
                    side,
                );
                match id {
                    GameId::Apples => painter.sprite(Sprite::Apple, preview),
                    GameId::Letters => painter.label("А а", preview, side * 0.6, 0, color::RED),
                    GameId::Puzzle2048 => {
                        painter.rectangle(preview, Vec3::new(0.17, 0.43, 0.62));
                        painter.label("2048", preview, side * 0.3, 0, color::WHITE);
                    }
                    GameId::Mouse => {
                        painter.sprite(Sprite::Mouse, preview);
                        painter.sprite(
                            Sprite::Cheese,
                            rect(preview.min_x(), preview.center().y, side / 2.0, side / 2.0),
                        );
                    }
                }
                painter.label(
                    &format!("{}. {}", index + 1, id.title()),
                    rect(
                        cell.min_x() + 8.0,
                        cell.max_y() - cell.size.height * 0.2,
                        cell.size.width - 16.0,
                        cell.size.height * 0.16,
                    ),
                    30.0,
                    0,
                    color::WHITE,
                );
            }
        }
        let footer = rect(0.0, painter.size.y, size.x, FOOTER);
        painter.rectangle(footer, Vec3::splat(0.06));
        let hint = if let Some(game) = &self.active {
            let button = rect(8.0, painter.size.y + 4.0, 100.0_f32.min(size.x), 32.0);
            painter.button(
                button,
                false,
                self.pointer.is_some_and(|p| contains(button, p)),
            );
            painter.label("Меню", button, 17.0, 0, color::WHITE);
            if size.x < 900.0 {
                game.id().short_hint()
            } else {
                game.id().hint()
            }
        } else if size.x < 640.0 {
            "Выберите игру / 1-4 / Enter / Esc"
        } else {
            "Выберите игру / 1-4 / стрелки и Enter / Esc - выход"
        };
        let x = if self.active.is_some() { 120.0 } else { 8.0 };
        painter.label(
            hint,
            rect(x, painter.size.y, size.x - x - 8.0, FOOTER),
            16.0,
            0,
            Vec3::splat(0.75),
        );
        painter.size = size;
    }
}
fn content_size(size: Vec2) -> Vec2 {
    Vec2::new(size.x.max(1.0), (size.y - FOOTER).max(1.0))
}

#[cfg(test)]
mod tests;
