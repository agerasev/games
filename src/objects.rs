use super::TILT;
use crate::animation::{Animation, AnimationInfo};
use anyhow::{Error, anyhow};
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};
use wgame::{
    Library,
    gfx::Scene,
    glam::Vec2,
    image::Image,
    texture::{Texture, TextureSettings},
};

pub trait Object {
    fn pos(&self) -> Vec2;
    fn draw(&self, lib: &Library, scene: &mut Scene, order: i32);
}

fn texture(lib: &Library, bytes: &[u8]) -> Result<Texture, Error> {
    Ok(lib.make_texture(&Image::decode_auto(bytes)?, TextureSettings::nearest()))
}

fn validate(info: &AnimationInfo, texture: &Texture) -> Result<(), Error> {
    let size = texture.size();
    info.validate([size.width, size.height])
}

#[derive(Clone, Debug, Deserialize)]
pub struct TreeAnimation {
    trunk: AnimationInfo,
    leaves: AnimationInfo,
}

#[derive(Clone, Debug)]
pub struct TreeSpecies {
    texture: Texture,
    animation: TreeAnimation,
}

impl TreeSpecies {
    pub fn load(lib: &Library) -> Result<Self, Error> {
        let texture = texture(lib, include_bytes!("../assets/tree.png"))?;
        let animation: TreeAnimation =
            serde_json::from_slice(include_bytes!("../assets/tree.json"))?;
        validate(&animation.trunk, &texture)?;
        validate(&animation.leaves, &texture)?;
        Ok(Self { texture, animation })
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
    fn draw(&self, lib: &Library, scene: &mut Scene, order: i32) {
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
        trunk.draw(lib, scene, order, pos, size, Duration::ZERO);
        leaves.draw(lib, scene, order + 1, pos, size, Duration::ZERO);
    }
}

#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Orientation {
    Front = 0,
    Back = 1,
    Side = 2,
}

#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    Stand = 0,
    Run = 1,
}

#[derive(Clone, Debug)]
struct PersonAnimations {
    head_torso: [AnimationInfo; 3],
    hands_legs_stand: [AnimationInfo; 3],
    hands_legs_run: [AnimationInfo; 3],
}

impl PersonAnimations {
    fn load() -> Result<Self, Error> {
        let mut container: HashMap<String, AnimationInfo> =
            serde_json::from_slice(include_bytes!("../assets/man.json"))?;

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
    texture: Texture,
    animations: PersonAnimations,
}

impl Personality {
    pub const SIZE: Vec2 = Vec2::new(1.0, 2.0);
    pub const CENTER: Vec2 = Vec2::new(0.5, 1.8);

    pub fn new(lib: &Library) -> Result<Self, Error> {
        let texture = texture(lib, include_bytes!("../assets/man.png"))?;
        let animations = PersonAnimations::load()?;
        for info in animations
            .head_torso
            .iter()
            .chain(&animations.hands_legs_stand)
            .chain(&animations.hands_legs_run)
        {
            validate(info, &texture)?;
        }
        Ok(Self {
            texture,
            animations,
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
    fn draw(&self, lib: &Library, scene: &mut Scene, order: i32) {
        let orientation = if !(-TILT..=TILT).contains(&self.direction.x) {
            Orientation::Side
        } else if self.direction.y > 0.0 {
            Orientation::Front
        } else {
            Orientation::Back
        };
        let flip = self.direction.x < 0.0;
        let action = if self.velocity == Vec2::ZERO {
            Action::Stand
        } else {
            Action::Run
        };

        let size = Personality::SIZE;
        let pos = (self.position - Personality::CENTER) * Vec2::new(1.0, TILT);

        let animation_period = Self::ANIMATION_PERIOD;
        let action_duration = self.action_duration;
        let head_torso = &self.look.animations.head_torso[orientation as usize];
        let hands_legs = match action {
            Action::Stand => &self.look.animations.hands_legs_stand[orientation as usize],
            Action::Run => &self.look.animations.hands_legs_run[orientation as usize],
        };

        let head_torso =
            Animation::new(&self.look.texture, head_torso, animation_period).flip(flip);
        let hands_legs =
            Animation::new(&self.look.texture, hands_legs, animation_period).flip(flip);

        head_torso.draw(lib, scene, order, pos, size, action_duration);
        hands_legs.draw(lib, scene, order + 1, pos, size, action_duration);
    }
}
