use macroquad::{
    color,
    math::{Rect, Vec2},
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimationInfo {
    size: [u32; 2],
    positions: Vec<[u32; 2]>,
}

impl AnimationInfo {
    pub fn size(&self) -> Vec2 {
        Vec2::from(self.size.map(|x| x as f32))
    }
    pub fn position(&self, i: usize) -> Vec2 {
        Vec2::from(self.positions[i].map(|x| x as f32))
    }
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.positions.len()
    }
}

pub struct Animation<'a> {
    texture: &'a Texture2D,
    info: &'a AnimationInfo,
    period: Duration,
    flip_x: bool,
}

impl<'a> Animation<'a> {
    pub fn new(
        texture: &'a Texture2D,
        info: &'a AnimationInfo,
        period: Duration,
        flip_x: bool,
    ) -> Self {
        assert_ne!(info.len(), 0);
        Self {
            texture,
            info,
            period,
            flip_x,
        }
    }
    pub fn draw(&self, pos: Vec2, size: Vec2, time: Duration) {
        let index = (time.div_duration_f64(self.period).fract() * self.info.len() as f64) as usize;
        let tex_pos = self.info.position(index);
        let tex_size = self.info.size();
        draw_texture_ex(
            self.texture,
            pos.x,
            pos.y,
            color::WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                source: Some(Rect::new(tex_pos.x, tex_pos.y, tex_size.x, tex_size.y)),
                flip_x: self.flip_x,
                ..Default::default()
            },
        );
    }
}
