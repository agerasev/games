//! Sliding number puzzle. Size and merge rules start a new round; spawn policy
//! changes apply to future tiles. Winning leaves play open.
//! Input and drawing share one layout in logical pixels, including pointer swipes.
pub mod model;
mod ui;
mod view;

use crate::{current_events, draw::Painter, layout::contains};
use model::{Board, Direction, Rule, Settings, Spawn, Turn};
use std::collections::VecDeque;
use wgame::{
    canvas::{Button, CanvasInput, Event, Key},
    glam::Vec2,
};

const SLIDE: f32 = 0.12;
const POP: f32 = 0.13;
struct Animation {
    turn: Turn,
    elapsed: f32,
}
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Action {
    Size(usize),
    Rule(Rule),
    Spawn(Spawn),
    Restart,
    Undo,
}
pub struct Game {
    board: Board,
    animation: Option<Animation>,
    pending: VecDeque<Direction>,
    swipe: Option<Vec2>,
    last_size: Vec2,
}
impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
impl Game {
    pub fn new() -> Self {
        let seed = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self::with_seed(seed)
    }
    fn with_seed(seed: u64) -> Self {
        Self {
            board: Board::new(Settings::default(), seed),
            animation: None,
            pending: VecDeque::new(),
            swipe: None,
            last_size: Vec2::ZERO,
        }
    }
    pub(crate) fn action(&mut self, action: Action) {
        let mut settings = self.board.settings();
        match action {
            Action::Size(side) => settings.side = side,
            Action::Rule(rule) => settings.rule = rule,
            Action::Spawn(spawn) => {
                self.board.set_spawn(spawn);
                return;
            }
            Action::Restart => self.board.reset(settings),
            Action::Undo => {
                self.board.undo();
            }
        }
        if settings != self.board.settings() {
            self.board.reset(settings);
        } else if !matches!(action, Action::Restart | Action::Undo) {
            return;
        }
        self.animation = None;
        self.pending.clear();
        self.swipe = None;
    }
    fn enqueue(&mut self, direction: Direction) {
        // Preserve deliberate rapid presses while bounding delayed movement.
        if self.pending.len() < 4 {
            self.pending.push_back(direction);
        }
    }
    pub fn update(&mut self, input: &CanvasInput, dt: f32, size: Vec2) {
        if size != self.last_size || input.events.contains(&Event::Cancelled) {
            self.swipe = None;
            self.pending.clear();
        }
        self.last_size = size;
        if let Some(animation) = &mut self.animation {
            animation.elapsed += dt.max(0.0);
            if animation.elapsed >= SLIDE + POP {
                self.animation = None;
            }
        }
        let layout = view::Layout::new(size);
        for event in current_events(input) {
            match *event {
                Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                } => {
                    let direction = match key {
                        Key::ArrowLeft | Key::Character('a') => Some(Direction::Left),
                        Key::ArrowRight | Key::Character('d') => Some(Direction::Right),
                        Key::ArrowUp | Key::Character('w') => Some(Direction::Up),
                        Key::ArrowDown | Key::Character('s') => Some(Direction::Down),
                        _ => None,
                    };
                    if let Some(direction) = direction {
                        self.enqueue(direction);
                    } else {
                        let settings = self.board.settings();
                        let action = match key {
                            Key::Character('r') => Some(Action::Restart),
                            Key::Character('u' | 'z') | Key::Backspace => Some(Action::Undo),
                            Key::Character(c @ '3'..='6') => {
                                Some(Action::Size(c as usize - '0' as usize))
                            }
                            Key::Character('f') => {
                                Some(Action::Rule(if settings.rule == Rule::Classic {
                                    Rule::Fibonacci
                                } else {
                                    Rule::Classic
                                }))
                            }
                            Key::Character('t') => {
                                Some(Action::Spawn(if settings.spawn == Spawn::SmallOnly {
                                    Spawn::Mixed
                                } else {
                                    Spawn::SmallOnly
                                }))
                            }
                            _ => None,
                        };
                        if let Some(action) = action {
                            self.action(action);
                        }
                    }
                }
                Event::Button {
                    button: Button::Primary,
                    pressed,
                    position,
                } => {
                    if pressed {
                        self.swipe = None;
                        if contains(layout.board, position) {
                            self.swipe = Some(position);
                        }
                    } else if let Some(start) = self.swipe.take() {
                        let delta = position - start;
                        let threshold = (layout.board.size.width * 0.06).clamp(10.0, 28.0);
                        if delta.abs().max_element() >= threshold {
                            self.enqueue(if delta.x.abs() > delta.y.abs() {
                                if delta.x > 0.0 {
                                    Direction::Right
                                } else {
                                    Direction::Left
                                }
                            } else if delta.y > 0.0 {
                                Direction::Down
                            } else {
                                Direction::Up
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        while self.animation.is_none() {
            let Some(direction) = self.pending.pop_front() else {
                break;
            };
            if let Some(turn) = self.board.step(direction) {
                self.animation = Some(Animation { turn, elapsed: 0.0 });
            }
        }
    }
    pub fn board(&self) -> &Board {
        &self.board
    }
    pub fn draw(&self, painter: &mut Painter<'_>) {
        view::draw(self, painter);
    }
}

#[cfg(test)]
mod tests;
