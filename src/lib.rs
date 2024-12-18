pub mod algebra;
pub mod animation;
pub mod compat;
pub mod games;
pub mod geometry;
pub mod layout;
pub mod model;
pub mod numerical;
pub mod physics;
pub mod text;
pub mod texture;

use anyhow::Error;
use macroquad::math::Rect;
use std::{future::Future, pin::Pin};

pub trait Game {
    fn name(&self) -> String;
    fn draw_preview(&self, rect: Rect);
    fn launch(&self) -> Pin<Box<dyn Future<Output = Result<(), Error>>>>;
}
