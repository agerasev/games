mod objects;

use self::objects::{Character, Object, Personality};
use anyhow::Error;
use macroquad::{
    color,
    input::{is_key_down, KeyCode},
    math::{Rect, Vec2},
    miniquad::window::screen_size,
    texture::{set_default_filter_mode, FilterMode},
    time::get_frame_time,
    window::{clear_background, next_frame},
};
use objects::{Tree, TreeSpecies};
use std::{future::Future, pin::Pin, time::Duration};

const TILT: f32 = 0.6667;

pub async fn main() -> Result<(), Error> {
    set_default_filter_mode(FilterMode::Nearest);
    let tree = TreeSpecies::load("assets/tree.png", "assets/tree.json").await?;
    let man = Personality::new("assets/man.png", "assets/man.json").await?;

    let some_tree = Tree {
        species: &tree,
        pos: Vec2::new(2.0, -1.0),
        growth: 2.0,
    };
    let mut player = Character::new(&man, Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0));

    while !is_key_down(KeyCode::Escape) {
        let dt = Duration::from_secs_f32(get_frame_time());

        // Move
        {
            let mut motion = Vec2::ZERO;
            if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
                motion += Vec2::from([0.0, -1.0]);
            }
            if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
                motion += Vec2::from([0.0, 1.0]);
            }
            if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
                motion += Vec2::from([-1.0, 0.0]);
            }
            if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
                motion += Vec2::from([1.0, 0.0]);
            }

            player.step(motion, dt);
        }

        // Draw
        {
            let scale = 60.0;
            let offset = Vec2::from(screen_size()) / 2.0;

            clear_background(color::DARKGREEN);

            some_tree.draw(scale, offset);
            player.draw(scale, offset);
        }

        next_frame().await
    }

    Ok(())
}

pub struct Game {}

impl Game {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {})
    }
}

impl crate::Game for Game {
    fn name(&self) -> String {
        "Бег по лесу".to_owned()
    }

    fn draw_preview(&self, _rect: Rect) {}

    fn launch(&self) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        Box::pin(main())
    }
}
