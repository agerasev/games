use anyhow::Result;
use macroquad::{
    color::Color,
    text::{
        camera_font_scale, draw_text_ex, load_ttf_font, measure_text, Font, TextDimensions,
        TextParams,
    },
};

#[derive(Clone, Debug)]
pub struct Text {
    pub value: String,
    pub font: Option<Font>,
    pub size: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl Text {
    pub fn new<S: AsRef<str>>(value: S, font: Option<Font>, size: f32) -> Self {
        let value = value.as_ref().to_string();
        Self { value, size, font }
    }

    pub fn measure(&self) -> TextDimensions {
        let (font_size, font_scale, font_aspect) = camera_font_scale(self.size);
        let mut dims = measure_text(&self.value, self.font.as_ref(), font_size, font_scale);
        dims.width *= font_aspect;
        dims
    }

    pub fn draw_aligned(&self, x: f32, y: f32, align: TextAlign, color: Color) {
        let (font_size, font_scale, font_scale_aspect) = camera_font_scale(self.size);
        let TextDimensions { mut width, .. } =
            measure_text(&self.value, self.font.as_ref(), font_size, font_scale);
        width *= font_scale_aspect;
        let x = x - match align {
            TextAlign::Left => 0.0,
            TextAlign::Center => width / 2.0,
            TextAlign::Right => width,
        };
        let params = TextParams {
            font: self.font.as_ref(),
            font_size,
            font_scale,
            font_scale_aspect,
            color,
            ..Default::default()
        };
        draw_text_ex(&self.value, x, y, params);
    }
}

pub fn draw_text_aligned(
    text: &str,
    x: f32,
    y: f32,
    align: TextAlign,
    font: Option<&Font>,
    size: f32,
    color: Color,
) {
    // TODO: Remove unnecessary text and font copying
    Text::new(text, font.cloned(), size).draw_aligned(x, y, align, color)
}

pub async fn load_default_font() -> Result<Font> {
    Ok(load_ttf_font("free-sans-bold.ttf").await?)
}
