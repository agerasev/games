//! Seeded forest and layered sprite animation, hosted by the shared game canvas.
mod animation;
mod objects;

use crate::{draw::Painter, layout::rect};
use euclid::default::Rect;
use objects::{Character, Personality, Tree, TreeSpecies};
use rand::{RngExt, SeedableRng, rngs::SmallRng};
use rand_distr::{Normal, Poisson, Uniform};
use std::time::Duration;
use wgame::{
    Library, Result,
    canvas::{CanvasInput, Event, Key},
    glam::{Affine2, Vec2, Vec3},
};

const TILT: f32 = 0.6667;

pub(crate) struct Assets {
    species: TreeSpecies,
    person: Personality,
}
impl Assets {
    pub fn new(lib: &Library) -> Result<Self> {
        Ok(Self {
            species: TreeSpecies::load(lib)?,
            person: Personality::new(lib)?,
        })
    }
}

pub struct Game {
    trees: Vec<Tree>,
    player: Character,
    zoom: f32,
}
impl Game {
    pub fn new() -> Self {
        let mut rng = SmallRng::seed_from_u64(0xdeadbeef);
        let normal = Normal::new(0.0, 10.0).unwrap();
        let growth = Uniform::new(1.0, 3.0).unwrap();
        let mut trees: Vec<_> = (0..rng.sample(Poisson::new(64_f32).unwrap()).round() as usize)
            .map(|_| Tree {
                pos: Vec2::new(rng.sample(normal), rng.sample(normal)),
                growth: rng.sample(growth),
            })
            .collect();
        trees.sort_by(|a, b| a.pos.y.total_cmp(&b.pos.y));
        Self {
            trees,
            player: Character::new(Vec2::ZERO, Vec2::Y),
            zoom: 0.1,
        }
    }

    pub fn update(&mut self, input: &CanvasInput, dt: f32) {
        let reset = input
            .events
            .iter()
            .any(|event| matches!(event, Event::Cancelled | Event::Focused(_)));
        let dt = if reset || !input.window_focused {
            Duration::ZERO
        } else {
            Duration::from_secs_f32(dt.clamp(0.0, 0.1))
        };
        self.player.step(motion(input), dt);
        if input.window_focused {
            let scroll: f32 = crate::current_events(input)
                .iter()
                .filter_map(|event| match event {
                    Event::Scroll(delta) => Some(delta.y / 40.0),
                    _ => None,
                })
                .sum();
            self.zoom = (self.zoom * (0.2 * scroll.clamp(-1.0, 1.0)).exp()).clamp(0.01, 10.0);
        }
    }

    pub fn draw(&self, painter: &mut Painter<'_>) {
        painter.rectangle(
            rect(0.0, 0.0, painter.size.x, painter.size.y),
            Vec3::new(0.0, 82.0 / 255.0, 44.0 / 255.0),
        );
        // The original camera is fixed at the forest center, with world Y down.
        let view = Affine2::from_scale_angle_translation(
            Vec2::splat(0.5 * painter.size.min_element() * self.zoom),
            0.0,
            0.5 * painter.size,
        );
        let assets = &painter.assets.running;
        let split = self
            .trees
            .partition_point(|tree| tree.pos.y < self.player.pos().y);
        // Each object's two layers stay adjacent in ground-position order.
        for (index, tree) in self.trees.iter().enumerate() {
            let index = index + usize::from(index >= split);
            tree.draw(
                &assets.species,
                painter.lib,
                &mut painter.scene,
                view,
                1 + 2 * index as i32,
            );
        }
        self.player.draw(
            &assets.person,
            painter.lib,
            &mut painter.scene,
            view,
            1 + 2 * split as i32,
        );
    }
}

fn motion(input: &CanvasInput) -> Vec2 {
    if !input.window_focused {
        return Vec2::ZERO;
    }
    let held = |arrow, letter| input.key_down(arrow) || input.key_down(Key::Character(letter));
    Vec2::new(
        f32::from(held(Key::ArrowRight, 'd')) - f32::from(held(Key::ArrowLeft, 'a')),
        f32::from(held(Key::ArrowDown, 's')) - f32::from(held(Key::ArrowUp, 'w')),
    )
}

pub(crate) fn preview(painter: &mut Painter<'_>, bounds: Rect<f32>) {
    let view = Affine2::from_scale_angle_translation(
        Vec2::splat(bounds.size.width.min(bounds.size.height) / 3.0),
        0.0,
        Vec2::from_array(bounds.center().to_array()),
    );
    let assets = &painter.assets.running;
    Tree {
        pos: Vec2::new(0.65, 0.3),
        growth: 1.0,
    }
    .draw(&assets.species, painter.lib, &mut painter.scene, view, 1);
    Character::new(Vec2::new(-0.5, 0.9), Vec2::Y).draw(
        &assets.person,
        painter.lib,
        &mut painter.scene,
        view,
        3,
    );
}

#[cfg(test)]
mod tests;
