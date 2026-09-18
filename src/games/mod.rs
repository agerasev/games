mod apples;
mod letters;
pub mod moon_lander;
mod mouse;
pub mod puzzle2048;

use crate::draw::Painter;
use wgame::{canvas::CanvasInput, glam::Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameId {
    Apples,
    Letters,
    Mouse,
    Puzzle2048,
    MoonLander,
}
impl GameId {
    pub const ALL: [Self; 5] = [
        Self::Apples,
        Self::Letters,
        Self::Mouse,
        Self::Puzzle2048,
        Self::MoonLander,
    ];
    pub fn slug(self) -> &'static str {
        match self {
            Self::Apples => "apples",
            Self::Letters => "letters",
            Self::Mouse => "mouse",
            Self::Puzzle2048 => "2048",
            Self::MoonLander => "lander",
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
            Self::Puzzle2048 => "2048 / Фибоначчи",
            Self::MoonLander => "Лунный модуль",
        }
    }
    pub fn short_hint(self) -> &'static str {
        match self {
            Self::Apples => "Цифры / + / - / Enter",
            Self::Letters => "1 / 2 / 3 / 0 / `",
            Self::Mouse => "Стрелки / WASD",
            Self::Puzzle2048 => "Стрелки / свайп / U / R",
            Self::MoonLander => "Space / A / D / P / R",
        }
    }
    pub fn hint(self) -> &'static str {
        match self {
            Self::Apples => "Цифры / + / - / PgUp / PgDn / Enter / ` - шрифт",
            Self::Letters => "1 - русский / 2 - English / 3 - греческий / 0 - цифры / ` - шрифт",
            Self::Mouse => "Стрелки / WASD - движение / Соберите всё!",
            Self::Puzzle2048 => "Стрелки / WASD / свайп - ход / U - отмена / R - заново",
            Self::MoonLander => {
                "Space / W / вверх - тяга / A, D / влево, вправо - наклон / P - пауза / R - заново / N - следующая посадка"
            }
        }
    }
}
#[derive(PartialEq)]
pub(crate) enum Action {
    Apples(apples::Action),
    Letters(letters::Action),
    MouseRestart,
    Puzzle2048(puzzle2048::Action),
    MoonLander(moon_lander::Action),
}

impl Action {
    pub(crate) fn consumes_input(&self) -> bool {
        !matches!(self, Self::MoonLander(moon_lander::Action::Pilot(_)))
    }
}

pub enum Game {
    Apples(apples::Game),
    Letters(letters::Game),
    Mouse(mouse::Game),
    Puzzle2048(puzzle2048::Game),
    MoonLander(moon_lander::Game),
}
impl Game {
    pub fn new(id: GameId) -> Self {
        match id {
            GameId::Apples => Self::Apples(apples::Game::new()),
            GameId::Letters => Self::Letters(letters::Game::default()),
            GameId::Mouse => Self::Mouse(mouse::Game::new()),
            GameId::Puzzle2048 => Self::Puzzle2048(puzzle2048::Game::new()),
            GameId::MoonLander => Self::MoonLander(moon_lander::Game::new()),
        }
    }
    pub fn id(&self) -> GameId {
        match self {
            Self::Apples(_) => GameId::Apples,
            Self::Letters(_) => GameId::Letters,
            Self::Mouse(_) => GameId::Mouse,
            Self::Puzzle2048(_) => GameId::Puzzle2048,
            Self::MoonLander(_) => GameId::MoonLander,
        }
    }
    pub(crate) fn action(&mut self, action: Action) {
        match (self, action) {
            (Self::Apples(game), Action::Apples(action)) => game.action(action),
            (Self::Letters(game), Action::Letters(action)) => game.action(action),
            (Self::Mouse(game), Action::MouseRestart) => game.restart(),
            (Self::Puzzle2048(game), Action::Puzzle2048(action)) => game.action(action),
            (Self::MoonLander(game), Action::MoonLander(action)) => game.action(action),
            _ => {}
        }
    }
    pub(crate) fn controls(&self, ui: &mut wgame_egui::egui::Ui) -> Vec<Action> {
        match self {
            Self::Apples(game) => game.controls(ui).into_iter().map(Action::Apples).collect(),
            Self::Letters(game) => game.controls(ui).into_iter().map(Action::Letters).collect(),
            Self::MoonLander(game) => game
                .controls(ui)
                .into_iter()
                .map(Action::MoonLander)
                .collect(),
            Self::Mouse(game) => {
                if game.controls(ui) {
                    vec![Action::MouseRestart]
                } else {
                    Vec::new()
                }
            }
            Self::Puzzle2048(game) => game
                .controls(ui)
                .into_iter()
                .map(Action::Puzzle2048)
                .collect(),
        }
    }
    pub fn update(&mut self, input: &CanvasInput, dt: f32, size: Vec2) {
        match self {
            Self::Apples(game) => game.update(input, dt),
            Self::Letters(game) => game.update(input),
            Self::Mouse(game) => game.update(input, dt),
            Self::Puzzle2048(game) => game.update(input, dt, size),
            Self::MoonLander(game) => game.update(input, dt),
        }
    }
    pub fn draw(&self, painter: &mut Painter<'_>) {
        match self {
            Self::Apples(game) => game.draw(painter),
            Self::Letters(game) => game.draw(painter),
            Self::Mouse(game) => game.draw(painter),
            Self::Puzzle2048(game) => game.draw(painter),
            Self::MoonLander(game) => game.draw(painter),
        }
    }
}
