mod objects;

use self::objects::{Character, Object, Personality};
use anyhow::Error;
use glam::Vec2;
use itertools::Itertools;
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    color,
    input::{is_key_down, mouse_wheel, KeyCode},
    math::Rect,
    miniquad::window::screen_size,
    texture::{set_default_filter_mode, FilterMode},
    time::{get_frame_time, get_time},
    window::{clear_background, next_frame},
};
use objects::{Action, Orientation, Tree, TreeSpecies};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rand_distr::{Normal, Poisson, Uniform};
use std::{future::Future, pin::Pin, time::Duration};

const TILT: f32 = 0.6667;

pub async fn main() -> Result<(), Error> {
    set_default_filter_mode(FilterMode::Nearest);
    let tree = TreeSpecies::load("tree.png", "tree.json").await?;
    let man = Personality::new("man.png", "man.json").await?;

    let mut rng = SmallRng::from_entropy();
    let mut static_objects = (0..rng.sample(Poisson::new(64_f32)?).round() as usize)
        .map(|_| {
            Ok(Box::new(Tree {
                species: &tree,
                pos: Vec2::new(
                    rng.sample(Normal::new(0.0, 10.0)?),
                    rng.sample(Normal::new(0.0, 10.0)?),
                ),
                growth: rng.sample(Uniform::new(1.0, 3.0)),
            }) as Box<dyn Object>)
        })
        .collect::<Result<Vec<_>, Error>>()?;
    static_objects.sort_by(|a, b| f32::total_cmp(&a.pos().y, &b.pos().y));

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

            for obj in static_objects
                .iter()
                .map(|o| o.as_ref())
                .merge_by([&player as &dyn Object], |a, b| a.pos().y < b.pos().y)
            {
                obj.draw();
            }

            set_default_camera();
        }

        next_frame().await
    }

    Ok(())
}

pub struct Game {
    man: Personality,
}

impl Game {
    pub async fn new() -> Result<Self, Error> {
        set_default_filter_mode(FilterMode::Nearest);
        Ok(Self {
            man: Personality::new("man.png", "man.json").await?,
        })
    }
}

impl crate::Game for Game {
    fn name(&self) -> String {
        "Бег".to_owned()
    }

    fn draw_preview(&self, rect: Rect) {
        let person_size = (rect.size() / Personality::SIZE).min_element() * Personality::SIZE;
        let offset = rect.point() + 0.5 * (rect.size() - person_size);
        self.man.draw(
            offset,
            person_size,
            (Orientation::Side, false),
            Action::Run,
            (
                Duration::from_millis(1200),
                Duration::from_secs_f64(get_time()),
            ),
        )
    }

    fn launch(&self) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        Box::pin(main())
    }
}
