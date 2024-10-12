use crate::animation::{Animation, AnimationInfo};
use anyhow::{anyhow, Error};
use macroquad::{
    color,
    file::load_file,
    input::{is_key_down, KeyCode},
    math::{Rect, Vec2},
    miniquad::window::screen_size,
    texture::{load_texture, set_default_filter_mode, FilterMode, Texture2D},
    time::get_frame_time,
    window::{clear_background, next_frame},
};
use serde::Deserialize;
use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};

const TILT: f32 = 0.6667;

trait Object {
    fn pos(&self) -> Vec2;
    fn draw(&self, scale: f32, offset: Vec2);
}

#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Orientation {
    Front = 0,
    Back = 1,
    Side = 2,
}

#[derive(Clone, Debug, Deserialize)]
struct PersonAnimations {
    head_torso: [AnimationInfo; 3],
    hands_legs_stand: [AnimationInfo; 3],
    hands_legs_run: [AnimationInfo; 3],
}

impl PersonAnimations {
    async fn load(path: &str) -> Result<Self, Error> {
        let mut container: HashMap<String, AnimationInfo> =
            serde_json::from_slice(&load_file(path).await?)?;

        let mut extract_group = |name: &str| -> Result<[AnimationInfo; 3], Error> {
            Ok(["front", "back", "side"]
                .into_iter()
                .map(|orientation| {
                    let key = name.replace("{}", orientation);
                    container.remove(&key).ok_or(anyhow!("No such key: {key}"))
                })
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .unwrap())
        };

        Ok(Self {
            head_torso: extract_group("head-torso-{}")?,
            hands_legs_stand: extract_group("hands-legs-{}-stand")?,
            hands_legs_run: extract_group("hands-legs-{}-run")?,
        })
    }
}

#[derive(Clone, Debug)]
struct Personality {
    texture: Texture2D,
    animations: PersonAnimations,
}

impl Personality {
    async fn new(texture_path: &str, animations_path: &str) -> Result<Self, Error> {
        Ok(Self {
            texture: load_texture(texture_path).await?,
            animations: PersonAnimations::load(animations_path).await?,
        })
    }
}

#[derive(Clone, Debug)]
struct Character<'a> {
    look: &'a Personality,
    position: Vec2,
    direction: Vec2,
    velocity: Vec2,
    action_duration: Duration,
}

impl<'a> Character<'a> {
    const SHAPE: Vec2 = Vec2::new(1.0, 2.0);
    const CENTER: Vec2 = Vec2::new(0.5, 1.8);
    const SPEED: f32 = 2.7778;
    const ANIMATION_PERIOD: Duration = Duration::from_millis(800);

    fn new(look: &'a Personality, pos: Vec2, dir: Vec2) -> Self {
        Self {
            look,
            position: pos,
            direction: dir,
            velocity: Vec2::ZERO,
            action_duration: Duration::ZERO,
        }
    }

    fn step(&mut self, mut motion: Vec2, dt: Duration) {
        motion = motion.normalize_or_zero();
        if motion != Vec2::ZERO {
            self.direction = motion;
        }
        let speed: f32 = Self::SPEED;
        self.velocity = motion * speed;
        self.position += self.velocity * dt.as_secs_f32();
        self.action_duration += dt;
    }
}

impl<'a> Object for Character<'a> {
    fn pos(&self) -> Vec2 {
        self.position
    }
    fn draw(&self, scale: f32, offset: Vec2) {
        let orientation = if !(-TILT..=TILT).contains(&self.direction.x) {
            Orientation::Side
        } else if self.direction.y > 0.0 {
            Orientation::Front
        } else {
            Orientation::Back
        };
        let flip_x = self.direction.x < 0.0;
        let head_torso = &self.look.animations.head_torso[orientation as usize];
        let hands_legs = if self.velocity == Vec2::ZERO {
            &self.look.animations.hands_legs_stand[orientation as usize]
        } else {
            &self.look.animations.hands_legs_run[orientation as usize]
        };

        let head_torso = Animation::new(
            &self.look.texture,
            head_torso,
            Self::ANIMATION_PERIOD,
            flip_x,
        );
        let hands_legs = Animation::new(
            &self.look.texture,
            hands_legs,
            Self::ANIMATION_PERIOD,
            flip_x,
        );

        let size = scale * Self::SHAPE;
        let pos = scale * self.position * Vec2::new(1.0, TILT) + offset - Self::CENTER;
        head_torso.draw(pos, size, self.action_duration);
        hands_legs.draw(pos, size, self.action_duration);
    }
}

pub async fn main() -> Result<(), Error> {
    set_default_filter_mode(FilterMode::Nearest);
    let man = Personality::new("assets/man.png", "assets/man.json").await?;

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
            let scale = 120.0;
            let offset = Vec2::from(screen_size()) / 2.0;

            clear_background(color::DARKGREEN);

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
