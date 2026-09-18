//! Shared sprites, fonts, and drawing helpers; geometry uses logical pixels.
use euclid::default::Rect;
use std::collections::HashMap;
use wgame::{
    Library, Result,
    gfx::{
        Scene,
        types::{Color, color},
    },
    glam::{Affine2, Vec2, Vec3},
    image::Image,
    prelude::*,
    texture::{Texture, TextureSettings},
    typography::{Font, FontData, FontTexture, TextAlign},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sprite {
    Apple,
    Pear,
    Orange,
    Mouse,
    Cheese,
}

pub struct Assets {
    textures: HashMap<Sprite, Texture>,
    fonts: [Font; 2],
    rasters: [FontTexture; 2],
    scale_factor: f64,
}

impl Assets {
    pub fn new(lib: &Library) -> Result<Self> {
        let mut textures = HashMap::new();
        for (id, bytes, settings) in [
            (
                Sprite::Apple,
                &include_bytes!("../assets/apple.png")[..],
                TextureSettings::nearest(),
            ),
            (
                Sprite::Pear,
                &include_bytes!("../assets/pear.png")[..],
                TextureSettings::nearest(),
            ),
            (
                Sprite::Orange,
                &include_bytes!("../assets/orange.png")[..],
                TextureSettings::nearest(),
            ),
            (
                Sprite::Mouse,
                &include_bytes!("../assets/mouse.png")[..],
                TextureSettings::nearest(),
            ),
            (
                Sprite::Cheese,
                &include_bytes!("../assets/cheese.png")[..],
                TextureSettings::nearest(),
            ),
        ] {
            textures.insert(id, lib.make_texture(&Image::decode_auto(bytes)?, settings));
        }
        let fonts = [
            lib.make_font(&FontData::new(
                include_bytes!("../assets/free-sans-bold.ttf").to_vec(),
                0,
            )?),
            lib.make_font(&FontData::new(
                include_bytes!("../assets/free-serif-bold.ttf").to_vec(),
                0,
            )?),
        ];
        let rasters = fonts.each_ref().map(|font| font.rasterize(128.0));
        Ok(Self {
            textures,
            fonts,
            rasters,
            scale_factor: 1.0,
        })
    }
    pub fn set_scale_factor(&mut self, scale: f64) {
        if self.scale_factor != scale {
            self.scale_factor = scale;
            self.rasters = self
                .fonts
                .each_ref()
                .map(|font| font.rasterize((128.0 * scale as f32).clamp(32.0, 512.0)));
        }
    }
}

pub struct Painter<'a> {
    pub lib: &'a Library,
    pub assets: &'a Assets,
    pub scene: Scene,
    pub size: Vec2,
}
impl<'a> Painter<'a> {
    pub fn new(lib: &'a Library, assets: &'a Assets, size: Vec2) -> Self {
        Self {
            lib,
            assets,
            scene: Scene::default(),
            size,
        }
    }
    pub fn rectangle(&mut self, rect: Rect<f32>, color: impl Color) {
        self.scene.add(
            &self
                .lib
                .shapes()
                .rectangle((
                    Vec2::new(rect.min_x(), rect.min_y()),
                    Vec2::new(rect.max_x(), rect.max_y()),
                ))
                .fill_color(color),
        );
    }
    pub fn sprite(&mut self, sprite: Sprite, rect: Rect<f32>) {
        self.sprite_transform(
            sprite,
            Affine2::from_scale_angle_translation(
                Vec2::from_array(rect.size.to_array()) / 2.0,
                0.0,
                Vec2::from_array(rect.center().to_array()),
            ),
            color::WHITE,
        );
    }
    pub fn sprite_transform(&mut self, sprite: Sprite, transform: Affine2, color: impl Color) {
        self.scene.add(
            &self
                .lib
                .shapes()
                .unit_quad()
                .fill_texture(&self.assets.textures[&sprite])
                .multiply_color(color)
                .transform(transform),
        );
    }
    /// Center text in its box, shrinking it to keep long labels inside the cell.
    pub fn label(
        &mut self,
        text: &str,
        rect: Rect<f32>,
        size: f32,
        font: usize,
        color: impl Color,
    ) {
        let text = self.assets.rasters[font].text(text);
        let metrics = text.metrics();
        let width = metrics.width() / metrics.size();
        let size = size
            .min(rect.size.height)
            .min(rect.size.width / width.max(0.01));
        if size <= 0.0 {
            return;
        }
        self.scene.add(
            &text
                .align(TextAlign::Center)
                .scale(size)
                .multiply_color(color)
                .move_to(Vec2::new(rect.center().x, rect.center().y + size * 0.32)),
        );
    }
    pub fn button(&mut self, rect: Rect<f32>, selected: bool, hovered: bool) -> Rect<f32> {
        self.rectangle(
            rect,
            if hovered {
                Vec3::splat(0.32)
            } else if selected {
                Vec3::new(0.12, 0.28, 0.38)
            } else {
                Vec3::splat(0.12)
            },
        );
        rect.inflate(
            -rect.size.width.min(4.0) * 0.5,
            -rect.size.height.min(4.0) * 0.5,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::GameId;
    #[test]
    fn bundled_font_covers_navigation_and_control_labels() {
        let font =
            FontData::new(include_bytes!("../assets/free-sans-bold.ttf").to_vec(), 0).unwrap();
        for text in GameId::ALL
            .into_iter()
            .flat_map(|id| [id.title(), id.hint(), id.short_hint()])
            .chain([
                "Меню",
                "Выберите игру / 1-5 / стрелки и Enter / Esc - выход",
                "1. 2. 3. 4. 5.",
                "Space - тяга / A, D - наклон / A / < / D / >",
            ])
        {
            for ch in text.chars() {
                assert_ne!(
                    font.as_ref().charmap().map(ch),
                    0,
                    "font is missing {ch:?} in {text:?}"
                );
            }
        }
    }
}
