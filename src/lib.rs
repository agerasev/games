pub mod games;
pub mod layout;
pub mod text;

use macroquad::{math::Rect, Error};
use std::{future::Future, pin::Pin};

pub trait Game {
    fn name(&self) -> String;
    fn draw_preview(&self, rect: Rect);
    fn launch(&self) -> Pin<Box<dyn Future<Output = Result<(), Error>>>>;
}
