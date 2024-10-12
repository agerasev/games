use crate::animation::{Animation, AnimationInfo};
use anyhow::{anyhow, Error};
use macroquad::{
    color,
    file::load_file,
    input::{is_key_down, KeyCode},
    math::{Rect, Vec2},
    texture::{load_texture, set_default_filter_mode, FilterMode, Texture2D},
    time::get_frame_time,
    window::{clear_background, next_frame},
};
use serde::Deserialize;
use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};

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
    fn new(look: &'a Personality, pos: Vec2, dir: Vec2) -> Self {
        Self {
            look,
            position: pos,
            direction: dir,
            velocity: Vec2::ZERO,
            action_duration: Duration::ZERO,
        }
    }
    fn step(&mut self, dt: Duration) {
        self.action_duration += dt;
    }
    fn draw(&self, scale: f32) {
        use Orientation::*;

        let orientation = match self.direction.y {
            ..-0.5 => Back,
            0.5.. => Front,
            _ => Side,
        };
        let flip_x = self.direction.x < 0.0;
        let head_torso = &self.look.animations.head_torso[orientation as usize];
        let hands_legs = if self.velocity == Vec2::ZERO {
            &self.look.animations.hands_legs_stand[orientation as usize]
        } else {
            &self.look.animations.hands_legs_run[orientation as usize]
        };

        let animation_period = Duration::from_secs_f32(1.0);
        let head_torso = Animation::new(&self.look.texture, head_torso, animation_period, flip_x);
        let hands_legs = Animation::new(&self.look.texture, hands_legs, animation_period, flip_x);

        let pos = scale * self.position;
        let size = scale * Vec2::new(1.0, 2.0);
        head_torso.draw(pos, size, self.action_duration);
        hands_legs.draw(pos, size, self.action_duration);
    }
}

pub async fn main() -> Result<(), Error> {
    set_default_filter_mode(FilterMode::Nearest);
    let man = Personality::new("assets/man.png", "assets/man.json").await?;

    let mut men = [
        Character::new(&man, Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0)),
        Character::new(&man, Vec2::new(2.0, 0.0), Vec2::new(0.0, -1.0)),
        Character::new(&man, Vec2::new(4.0, 0.0), Vec2::new(-1.0, 0.0)),
        Character::new(&man, Vec2::new(6.0, 0.0), Vec2::new(1.0, 0.0)),
        Character::new(&man, Vec2::new(0.0, 2.0), Vec2::new(0.0, 1.0)),
        Character::new(&man, Vec2::new(2.0, 2.0), Vec2::new(0.0, -1.0)),
        Character::new(&man, Vec2::new(4.0, 2.0), Vec2::new(-1.0, 0.0)),
        Character::new(&man, Vec2::new(6.0, 2.0), Vec2::new(1.0, 0.0)),
    ];
    for m in &mut men[4..] {
        m.velocity = Vec2::new(0.0, 1.0);
    }

    while !is_key_down(KeyCode::Escape) {
        let scale = 160.0;

        clear_background(color::BLACK);

        for m in &mut men {
            m.draw(scale);
            m.step(Duration::from_secs_f32(get_frame_time()));
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
