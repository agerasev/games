mod objects;

use self::objects::{Character, Object, Personality};
use anyhow::Error;
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    color,
    input::{is_key_down, mouse_wheel, KeyCode},
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
    let tree = TreeSpecies::load("tree.png", "tree.json").await?;
    let man = Personality::new("man.png", "man.json").await?;

    let some_tree = Tree {
        species: &tree,
        pos: Vec2::new(2.0, -1.0),
        growth: 2.0,
    };
    let mut player = Character::new(&man, Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0));
    let mut zoom = 0.1;

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
            zoom *= (0.2 * mouse_wheel().1).exp();

            player.step(motion, dt);
        }

        // Draw
        {
            clear_background(color::DARKGREEN);

            let viewport = Vec2::from(screen_size());
            let camera = Camera2D {
                zoom: (viewport.recip() * viewport.min_element()) * zoom,
                ..Default::default()
            };
            set_camera(&camera);

            some_tree.draw();
            player.draw();

            set_default_camera();
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
