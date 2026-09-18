mod apples;
mod letters;
mod mouse;
pub mod puzzle2048;

use crate::draw::Painter;
use wgame::{canvas::CanvasInput, glam::Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameId {
    Apples,
    Letters,
    Mouse,
}
impl GameId {
    pub const ALL: [Self; 3] = [Self::Apples, Self::Letters, Self::Mouse];
    pub fn slug(self) -> &'static str {
        match self {
            Self::Apples => "apples",
            Self::Letters => "letters",
            Self::Mouse => "mouse",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|id| id.slug() == value)
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Apples => "Считаем яблоки",
            Self::Letters => "Буквы",
            Self::Mouse => "Мышь и сыр",
        }
    }
    pub fn short_hint(self) -> &'static str {
        match self {
            Self::Apples => "Цифры / + / - / Enter",
            Self::Letters => "1 / 2 / 3 / 0 / `",
            Self::Mouse => "Стрелки / WASD",
        }
    }
    pub fn hint(self) -> &'static str {
        match self {
            Self::Apples => "Цифры / + / - / PgUp / PgDn / Enter / ` - шрифт",
            Self::Letters => "1 - русский / 2 - English / 3 - греческий / 0 - цифры / ` - шрифт",
            Self::Mouse => "Стрелки / WASD - движение / Соберите всё!",
        }
    }
}
pub enum Game {
    Apples(apples::Game),
    Letters(letters::Game),
    Mouse(mouse::Game),
}
impl Game {
    pub fn new(id: GameId) -> Self {
        match id {
            GameId::Apples => Self::Apples(apples::Game::new()),
            GameId::Letters => Self::Letters(letters::Game::default()),
            GameId::Mouse => Self::Mouse(mouse::Game::new()),
        }
    }
    pub fn id(&self) -> GameId {
        match self {
            Self::Apples(_) => GameId::Apples,
            Self::Letters(_) => GameId::Letters,
            Self::Mouse(_) => GameId::Mouse,
        }
    }
    pub fn update(&mut self, input: &CanvasInput, dt: f32, size: Vec2) {
        match self {
            Self::Apples(game) => game.update(input, dt, size),
            Self::Letters(game) => game.update(input, size),
            Self::Mouse(game) => game.update(input, dt),
        }
    }
    pub fn draw(&self, painter: &mut Painter<'_>) {
        match self {
            Self::Apples(game) => game.draw(painter),
            Self::Letters(game) => game.draw(painter),
            Self::Mouse(game) => game.draw(painter),
        }
    }
}
