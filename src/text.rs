use macroquad::{
    color::Color,
    text::{camera_font_scale, draw_text_ex, measure_text, Font, TextDimensions, TextParams},
};

#[derive(Clone, Debug)]
pub struct Text<'a> {
    pub value: String,
    pub size: f32,
    pub font: Option<&'a Font>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl<'a> Text<'a> {
    pub fn new<S: AsRef<str>>(value: S, size: f32, font: Option<&'a Font>) -> Self {
        let value = value.as_ref().to_string();
        Self { value, size, font }
    }

    pub fn measure(&self) -> TextDimensions {
        let (font_size, font_scale, font_aspect) = camera_font_scale(self.size);
        let mut dims = measure_text(&self.value, self.font, font_size, font_scale);
        dims.width *= font_aspect;
        dims
    }

    pub fn draw(&self, x: f32, y: f32, align: TextAlign, color: Color) {
        let (font_size, font_scale, font_scale_aspect) = camera_font_scale(self.size);
        let TextDimensions { mut width, .. } =
            measure_text(&self.value, self.font, font_size, font_scale);
        width *= font_scale_aspect;
        let x = x - match align {
            TextAlign::Left => 0.0,
            TextAlign::Center => width / 2.0,
            TextAlign::Right => width,
        };
        draw_text_ex(
            &self.value,
            x,
            y,
            TextParams {
                font: self.font,
                font_size,
                font_scale,
                font_scale_aspect,
                color,
                ..Default::default()
            },
        );
    }
}
