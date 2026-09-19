use crate::{
    draw::{Painter, Sprite},
    layout::rect,
};
use rand::{RngExt, SeedableRng, rngs::SmallRng};
use rand_distr::Poisson;
use std::f32::consts::PI;
use wgame::{
    canvas::{CanvasInput, Key},
    gfx::types::color,
    glam::{Affine2, Vec2, Vec3},
};

const MAP_SIZE: Vec2 = Vec2::new(40.0, 30.0);
const MEAN_ITEMS: f32 = 16.0;

struct Item {
    pos: Vec2,
    sprite: Sprite,
}

pub struct Game {
    rng: SmallRng,
    player: Vec2,
    radius: f32,
    items: Vec<Item>,
    total: usize,
    timeout: f32,
    elapsed: f32,
}
impl Game {
    pub fn new() -> Self {
        let mut game = Self {
            rng: SmallRng::seed_from_u64(0xdeadbeef),
            player: MAP_SIZE / 2.0,
            radius: 0.75,
            items: Vec::new(),
            total: 0,
            timeout: 1.0,
            elapsed: 0.0,
        };
        game.restart();
        game
    }
    pub(crate) fn restart(&mut self) {
        self.player = MAP_SIZE / 2.0;
        self.radius = 0.75;
        self.total = (self.rng.sample(Poisson::new(MEAN_ITEMS).unwrap()).round() as usize).max(1);
        self.items = (0..self.total)
            .map(|_| Item {
                pos: Vec2::new(
                    self.rng.random_range(0.5..MAP_SIZE.x - 0.5),
                    self.rng.random_range(0.5..MAP_SIZE.y - 0.5),
                ),
                sprite: if self.rng.random_bool(0.8) {
                    Sprite::Cheese
                } else {
                    Sprite::Apple
                },
            })
            .collect();
        self.timeout = 1.0;
    }
    pub(crate) fn controls(&self, ui: &mut wgame_egui::egui::Ui) -> bool {
        let mut restart = false;
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!("Собрано: {}", self.total - self.items.len()));
            ui.label(format!("Осталось: {}", self.items.len()));
            restart = ui.button("Заново").clicked();
        });
        restart
    }
    pub fn update(&mut self, input: &CanvasInput, dt: f32) {
        self.elapsed = (self.elapsed + dt) % 2.0;
        self.player = (self.player + motion(input) * (10.0 * dt))
            .clamp(Vec2::splat(self.radius), MAP_SIZE - self.radius);
        self.items.retain(|item| {
            if self.player.distance(item.pos) > self.radius + 0.5 {
                true
            } else {
                self.radius += 1.0 / (MEAN_ITEMS * (2.0 * self.radius).sqrt());
                false
            }
        });
        self.player = self
            .player
            .clamp(Vec2::splat(self.radius), MAP_SIZE - self.radius);
        if self.items.is_empty() {
            self.timeout = (self.timeout - dt).max(0.0);
            if self.timeout == 0.0 {
                self.restart();
            }
        }
    }
    pub fn draw(&self, painter: &mut Painter<'_>) {
        let viewport = painter.size;
        let scale = (viewport / MAP_SIZE).min_element();
        let origin = (viewport - MAP_SIZE * scale) / 2.0;
        painter.rectangle(
            rect(origin.x, origin.y, MAP_SIZE.x * scale, MAP_SIZE.y * scale),
            Vec3::splat(0.22),
        );
        let transform = |pos: Vec2, radius: f32| {
            Affine2::from_scale_angle_translation(
                Vec2::splat(radius * scale),
                0.0,
                origin + pos * scale,
            )
        };
        for item in &self.items {
            let bob = Vec2::new(0.0, 0.1 * (PI * self.elapsed).sin());
            painter.sprite_transform(item.sprite, transform(item.pos + bob, 0.5), color::WHITE);
        }
        painter.sprite_transform(
            Sprite::Mouse,
            transform(self.player, self.radius),
            color::WHITE,
        );
    }
}

fn motion(input: &CanvasInput) -> Vec2 {
    let held = |arrow, letter| input.key_down(arrow) || input.key_down(Key::Character(letter));
    Vec2::new(
        f32::from(held(Key::ArrowRight, 'd')) - f32::from(held(Key::ArrowLeft, 'a')),
        f32::from(held(Key::ArrowDown, 's')) - f32::from(held(Key::ArrowUp, 'w')),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgame::canvas::{Event, InputState};
    #[test]
    fn movement_collection_and_restart() {
        let mut game = Game::new();
        game.items = vec![Item {
            pos: game.player + Vec2::X,
            sprite: Sprite::Cheese,
        }];
        game.total = 1;
        let mut input = InputState::default();
        input.push(Event::Key {
            key: Key::Character('d'),
            pressed: true,
            repeat: false,
        });
        game.update(input.input(), 0.05);
        assert!(game.items.is_empty());
        assert!(game.radius > 0.75);
        let player = game.player;
        input.push(Event::Focused(false));
        game.update(input.input(), 0.0);
        assert_eq!(game.player, player);
        for _ in 0..30 {
            game.update(input.input(), 0.04);
        }
        assert!(!game.items.is_empty());
        assert_eq!(game.radius, 0.75);
        assert_eq!(game.player, MAP_SIZE / 2.0);
    }
    #[test]
    fn movement_aliases_do_not_double_speed_and_stay_in_bounds() {
        let mut input = InputState::default();
        for key in [Key::ArrowRight, Key::Character('d')] {
            input.push(Event::Key {
                key,
                pressed: true,
                repeat: false,
            });
        }
        assert_eq!(motion(input.input()), Vec2::X);
        let mut game = Game::new();
        game.update(input.input(), 100.0);
        assert_eq!(game.player.x, MAP_SIZE.x - game.radius);
        input.push(Event::Key {
            key: Key::ArrowLeft,
            pressed: true,
            repeat: false,
        });
        assert_eq!(motion(input.input()), Vec2::ZERO);
    }
}
