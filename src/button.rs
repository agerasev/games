use crate::text::{Text, TextAlign};
use macroquad::{
    color::Color,
    input::{
        is_mouse_button_down, is_mouse_button_pressed, is_mouse_button_released, mouse_position,
        MouseButton,
    },
    math::{Rect, Vec2},
    shapes::{draw_rectangle, draw_rectangle_lines},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub enum ButtonState {
    #[default]
    Up,
    Hover,
    Down,
}

#[derive(Clone, Copy, Debug)]
pub struct ButtonStyle {
    pub text_color: Color,
    pub fill_colors: Color,
    pub border_color: Color,
    pub border_width: f32,
}

#[derive(Clone, Default, Debug)]
pub struct Button {
    state: ButtonState,
    pressed: bool,
}

impl Button {
    pub fn state(&self) -> ButtonState {
        self.state
    }

    pub fn draw(&mut self, rect: Rect, text: &Text, style: ButtonStyle) -> bool {
        let mut clicked = false;
        self.state = if rect.contains(Vec2::from(mouse_position())) {
            self.pressed = is_mouse_button_pressed(MouseButton::Left);
            if self.pressed && is_mouse_button_released(MouseButton::Left) {
                clicked = true;
                self.pressed = false;
            }
            if self.pressed && is_mouse_button_down(MouseButton::Left) {
                ButtonState::Down
            } else {
                ButtonState::Hover
            }
        } else {
            self.pressed = false;
            ButtonState::Up
        };

        draw_rectangle(rect.x, rect.y, rect.w, rect.h, style.fill_colors);
        text.draw(
            rect.center().x,
            rect.center().y,
            TextAlign::Center,
            style.text_color,
        );
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            style.border_width,
            style.border_color,
        );

        clicked
    }
}
