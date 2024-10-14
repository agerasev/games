use super::TILT;
use crate::animation::{Animation, AnimationInfo};
use anyhow::{anyhow, Error};
use macroquad::{
    file::load_file,
    math::Vec2,
    texture::{load_texture, Texture2D},
};
use serde::Deserialize;
use std::{cell::RefCell, collections::HashMap, time::Duration};

pub trait Object {
    fn pos(&self) -> Vec2;
    fn draw(&self);
}

impl<T: Object> Object for RefCell<T> {
    fn pos(&self) -> Vec2 {
        self.borrow().pos()
    }
    fn draw(&self) {
        self.borrow().draw();
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TreeAnimation {
    trunk: AnimationInfo,
    leaves: AnimationInfo,
}

#[derive(Clone, Debug)]
pub struct TreeSpecies {
    texture: Texture2D,
    animation: TreeAnimation,
}

impl TreeSpecies {
    pub async fn load(texture_path: &str, animation_path: &str) -> Result<Self, Error> {
        Ok(Self {
            texture: load_texture(texture_path).await?,
            animation: serde_json::from_slice(&load_file(animation_path).await?)?,
        })
    }
}

pub struct Tree<'a> {
    pub species: &'a TreeSpecies,
    pub pos: Vec2,
    pub growth: f32,
}

impl<'a> Tree<'a> {
    const ANIMATION_PERIOD: Duration = Duration::from_secs(5);
    const SHAPE: Vec2 = Vec2::new(1.0, 2.0);
    const CENTER: Vec2 = Vec2::new(0.5, 1.9);
}

impl<'a> Object for Tree<'a> {
    fn pos(&self) -> Vec2 {
        self.pos
    }
    fn draw(&self) {
        let trunk = Animation::new(
            &self.species.texture,
            &self.species.animation.trunk,
            Self::ANIMATION_PERIOD,
        );
        let leaves = Animation::new(
            &self.species.texture,
            &self.species.animation.leaves,
            Self::ANIMATION_PERIOD,
        );

        let size = self.growth * Self::SHAPE;
        let pos = (self.pos - self.growth * Self::CENTER) * Vec2::new(1.0, TILT);
        trunk.draw(pos, size, Duration::ZERO);
        leaves.draw(pos, size, Duration::ZERO);
    }
}

#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Orientation {
    Front = 0,
    Back = 1,
    Side = 2,
}

#[derive(Clone, Debug)]
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
pub struct Personality {
    texture: Texture2D,
    animations: PersonAnimations,
}

impl Personality {
    pub async fn new(texture_path: &str, animations_path: &str) -> Result<Self, Error> {
        Ok(Self {
            texture: load_texture(texture_path).await?,
            animations: PersonAnimations::load(animations_path).await?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Character<'a> {
    look: &'a Personality,
    position: Vec2,
    direction: Vec2,
    velocity: Vec2,
    action_duration: Duration,
}

impl<'a> Character<'a> {
    const SIZE: Vec2 = Vec2::new(1.0, 2.0);
    const CENTER: Vec2 = Vec2::new(0.5, 1.8);
    const SPEED: f32 = 2.7778;
    const ANIMATION_PERIOD: Duration = Duration::from_millis(800);

    pub fn new(look: &'a Personality, pos: Vec2, dir: Vec2) -> Self {
        Self {
            look,
            position: pos,
            direction: dir,
            velocity: Vec2::ZERO,
            action_duration: Duration::ZERO,
        }
    }

    pub fn step(&mut self, mut motion: Vec2, dt: Duration) {
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
    fn draw(&self) {
        let orientation = if !(-TILT..=TILT).contains(&self.direction.x) {
            Orientation::Side
        } else if self.direction.y > 0.0 {
            Orientation::Front
        } else {
            Orientation::Back
        };
        let flip = self.direction.x < 0.0;
        let head_torso = &self.look.animations.head_torso[orientation as usize];
        let hands_legs = if self.velocity == Vec2::ZERO {
            &self.look.animations.hands_legs_stand[orientation as usize]
        } else {
            &self.look.animations.hands_legs_run[orientation as usize]
        };

        let head_torso = Animation::new(&self.look.texture, head_torso, Self::ANIMATION_PERIOD)
            .flip(flip, false);
        let hands_legs = Animation::new(&self.look.texture, hands_legs, Self::ANIMATION_PERIOD)
            .flip(flip, false);

        let size = Self::SIZE;
        let pos = (self.position - Self::CENTER) * Vec2::new(1.0, TILT);
        head_torso.draw(pos, size, self.action_duration);
        hands_legs.draw(pos, size, self.action_duration);
    }
}
