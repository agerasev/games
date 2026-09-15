use serde::Deserialize;
use std::time::Duration;
use wgame::{
    Library,
    gfx::Scene,
    glam::{Affine2, Vec2},
    prelude::*,
    texture::Texture,
};

#[derive(Clone, Debug, Deserialize)]
pub struct AnimationInfo {
    size: [u32; 2],
    positions: Vec<[u32; 2]>,
}

impl AnimationInfo {
    pub fn validate(&self, size: [u32; 2]) -> wgame::Result<()> {
        anyhow::ensure!(!self.positions.is_empty(), "animation has no frames");
        anyhow::ensure!(self.size.iter().all(|&v| v > 0), "empty animation frame");
        for pos in &self.positions {
            for axis in 0..2 {
                anyhow::ensure!(
                    pos[axis]
                        .checked_add(self.size[axis])
                        .is_some_and(|end| end <= size[axis]),
                    "animation frame outside sprite sheet"
                );
            }
        }
        Ok(())
    }

    fn frame(&self, period: Duration, time: Duration) -> usize {
        (time.div_duration_f64(period).fract() * self.positions.len() as f64) as usize
    }

    fn uv(&self, texture_size: Vec2, index: usize, flip: bool) -> Affine2 {
        let size = Vec2::from_array(self.size.map(|v| v as f32)) / texture_size;
        let pos = Vec2::from_array(self.positions[index].map(|v| v as f32)) / texture_size;
        Affine2::from_scale_angle_translation(
            size * Vec2::new(if flip { -1.0 } else { 1.0 }, 1.0),
            0.0,
            pos + if flip {
                Vec2::new(size.x, 0.0)
            } else {
                Vec2::ZERO
            },
        )
    }
}

pub struct Animation<'a> {
    texture: &'a Texture,
    info: &'a AnimationInfo,
    period: Duration,
    flip: bool,
}

impl<'a> Animation<'a> {
    pub fn new(texture: &'a Texture, info: &'a AnimationInfo, period: Duration) -> Self {
        assert!(!info.positions.is_empty() && !period.is_zero());
        Self {
            texture,
            info,
            period,
            flip: false,
        }
    }

    pub fn flip(mut self, x: bool) -> Self {
        self.flip = x;
        self
    }

    pub fn draw(
        &self,
        lib: &Library,
        scene: &mut Scene,
        order: i32,
        pos: Vec2,
        size: Vec2,
        time: Duration,
    ) {
        let tex_size = self.texture.size();
        let uv = self.info.uv(
            Vec2::new(tex_size.width as f32, tex_size.height as f32),
            self.info.frame(self.period, time),
            self.flip,
        );
        scene.add(
            &lib.shapes()
                .rectangle((pos, pos + size))
                .fill_texture(&self.texture.transform_coord(uv))
                .order(order),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames_wrap_and_flip_within_selected_cell() {
        let info = AnimationInfo {
            size: [60, 120],
            positions: vec![[120, 0], [180, 0]],
        };
        let period = Duration::from_millis(800);
        assert_eq!(info.frame(period, Duration::from_millis(399)), 0);
        assert_eq!(info.frame(period, Duration::from_millis(400)), 1);
        assert_eq!(info.frame(period, period), 0);
        let normal = info.uv(Vec2::new(600.0, 360.0), 1, false);
        let flipped = info.uv(Vec2::new(600.0, 360.0), 1, true);
        assert!(
            (normal.transform_point2(Vec2::ZERO) - flipped.transform_point2(Vec2::X)).length()
                < 1e-6
        );
        assert!(
            (normal.transform_point2(Vec2::ONE) - flipped.transform_point2(Vec2::Y)).length()
                < 1e-6
        );
    }
}

#[cfg(test)]
mod asset_tests {
    use super::*;
    use std::collections::HashMap;
    use wgame::image::{Image, ImageBase};

    #[test]
    fn embedded_frames_fit_decoded_sprite_sheets() {
        for (png, json, count) in [
            (
                &include_bytes!("../assets/man.png")[..],
                &include_bytes!("../assets/man.json")[..],
                9,
            ),
            (
                &include_bytes!("../assets/tree.png")[..],
                &include_bytes!("../assets/tree.json")[..],
                2,
            ),
        ] {
            let image = Image::decode_auto(png).unwrap();
            let size = image.size();
            let animations: HashMap<String, AnimationInfo> = serde_json::from_slice(json).unwrap();
            assert_eq!(animations.len(), count);
            for info in animations.values() {
                info.validate([size.width, size.height]).unwrap();
            }
        }
    }
}
