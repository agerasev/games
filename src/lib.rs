//pub mod animation;
//pub mod compat;
pub mod games;
//pub mod geometry;
pub mod layout;
//pub mod model;
//pub mod text;
//pub mod texture;

use anyhow::Error;
use euclid::default::Rect;
use std::pin::Pin;
use wgame::{Window, gfx::CollectorWithContext};

pub trait Game {
    fn name(&self) -> String;
    fn draw_preview(&self, renderer: &mut CollectorWithContext, rect: Rect<f32>);
    fn launch<'a>(
        &self,
        window: &'a mut Window,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>;
}
