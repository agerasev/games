use crate::{
    layout::grid,
    text::{draw_text_aligned, load_default_font, TextAlign},
};
use anyhow::Error;
use glam::Vec2;
use itertools::Itertools;
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    color,
    input::{
        is_key_down, is_key_pressed, is_mouse_button_down, is_mouse_button_pressed, mouse_position,
        KeyCode, MouseButton,
    },
    math::Rect,
    miniquad::window::screen_size,
    shapes::draw_rectangle,
    text::{load_ttf_font, Font},
    texture::{
        draw_texture_ex, load_texture, render_target, set_default_filter_mode, DrawTextureParams,
        FilterMode, RenderTarget, Texture2D,
    },
    window::{clear_background, next_frame},
};
use std::{future::Future, pin::Pin};

#[derive(Clone, Copy, Debug)]
struct Letter {
    char: char,
    type_: LetterType,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum LetterType {
    Vowel,
    Consonant,
    Other,
}

impl Letter {
    fn draw(&self, rect: Rect, font: &Font) {
        let text: String = self
            .char
            .to_uppercase()
            .chain(self.char.to_lowercase())
            .dedup()
            .join(" ");
        let size = rect.size().min_element() / 2.0;
        draw_text_aligned(
            &text,
            rect.center().x,
            rect.center().y + size / 2.0,
            TextAlign::Center,
            Some(font),
            size,
            match self.type_ {
                LetterType::Vowel => color::RED,
                LetterType::Consonant => color::BLUE,
                LetterType::Other => color::GRAY,
            },
        );
    }
}

fn draw_flag(texture: &Texture2D, rect: Rect) {
    let aspect = rect.w / rect.h;
    let tex_aspect = texture.width() / texture.height();
    draw_texture_ex(
        texture,
        rect.x,
        rect.y,
        color::WHITE,
        DrawTextureParams {
            dest_size: Some(rect.size()),
            source: Some(Rect::new(
                0.0,
                0.0,
                texture.width() * (aspect / tex_aspect).min(1.0),
                texture.height() * (aspect / tex_aspect).min(1.0),
            )),
            ..Default::default()
        },
    )
}

/// FIXME: Issues
/// + When dropping render_pass texture is blank
/// + When using some texture to draw on other texture then first texture becomes blank
fn render_to_texture(width: u32, height: u32, render: impl FnOnce(Vec2)) -> RenderTarget {
    let render_target = render_target(width, height);
    render_target.texture.set_filter(FilterMode::Linear);

    let size = Vec2::new(width as f32, height as f32);
    set_camera(&Camera2D {
        render_target: Some(render_target.clone()),
        ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, size.x, size.y))
    });
    clear_background(color::BROWN);
    render(size);
    set_default_camera();

    render_target
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
enum ButtonState {
    #[default]
    Inactive,
    Hover,
    Down,
    Active,
}

fn button(rect: Rect, active: bool, content: impl FnOnce(Rect, ButtonState)) -> bool {
    let mut pressed = false;
    let state = if rect.contains(Vec2::from(mouse_position())) {
        if is_mouse_button_pressed(MouseButton::Left) {
            pressed = true;
        }
        if is_mouse_button_down(MouseButton::Left) {
            ButtonState::Down
        } else {
            ButtonState::Hover
        }
    } else if active {
        ButtonState::Active
    } else {
        ButtonState::Inactive
    };

    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        match state {
            ButtonState::Inactive => color::BLACK,
            ButtonState::Hover => color::WHITE,
            ButtonState::Down => color::RED,
            ButtonState::Active => color::GRAY,
        },
    );

    let padding_factor = 0.1;
    content(
        Rect::new(
            rect.x + padding_factor * rect.w,
            rect.y + padding_factor * rect.h,
            (1.0 - 2.0 * padding_factor) * rect.w,
            (1.0 - 2.0 * padding_factor) * rect.h,
        ),
        state,
    );

    pressed
}

pub async fn main() -> Result<(), Error> {
    let fonts = [
        load_ttf_font("free-sans-bold.ttf").await?,
        load_ttf_font("free-serif-bold.ttf").await?,
    ];

    let alphabets = [&RUSSIAN[..], &ENGLISH[..], &GREEK[..], &NUMBERS[..]];

    set_default_filter_mode(FilterMode::Linear);
    let render_target = render_to_texture(60, 40, |size| {
        draw_text_aligned(
            "123",
            0.5 * size.x,
            0.9 * size.y,
            TextAlign::Center,
            None, //Some(&fonts[0]),
            0.8 * size.y,
            color::WHITE,
        )
    });
    let flags = [
        load_texture("flags/ru.png").await?,
        load_texture("flags/us.png").await?,
        load_texture("flags/gr.png").await?,
        render_target.texture.clone(),
    ];

    let mut lang_index = 0;
    let mut font_index = 0;

    while !is_key_down(KeyCode::Escape) {
        set_default_camera();

        let screen = Vec2::from(screen_size());
        let font = &fonts[font_index];

        if is_key_pressed(KeyCode::Key1) {
            lang_index = 0;
        } else if is_key_pressed(KeyCode::Key2) {
            lang_index = 1;
        } else if is_key_pressed(KeyCode::Key3) {
            lang_index = 2;
        } else if is_key_pressed(KeyCode::Key0) {
            lang_index = 3;
        }
        if is_key_pressed(KeyCode::GraveAccent) {
            font_index = (font_index + 1) % fonts.len();
        }

        clear_background(color::BLACK);

        let button_size = Vec2::new(60.0, 40.0);
        let button_padding = 10.0;

        {
            let alphabet = alphabets[lang_index];
            let mut i = 0;
            for line in grid(
                (screen.x - button_size.x - 2.0 * button_padding, screen.y),
                alphabet.len(),
                2.0,
            ) {
                for rect in line {
                    alphabet[i].draw(rect, font);
                    i += 1;
                }
            }
        }

        {
            for (i, flag) in flags.iter().enumerate() {
                if button(
                    Rect::new(
                        screen.x - button_size.x - button_padding,
                        button_padding + i as f32 * (button_size.y + button_padding),
                        button_size.x,
                        button_size.y,
                    ),
                    i == lang_index,
                    |rect, _| draw_flag(flag, rect),
                ) {
                    lang_index = i;
                }
            }
        }

        next_frame().await
    }

    Ok(())
}

const fn v(char: char) -> Letter {
    Letter {
        char,
        type_: LetterType::Vowel,
    }
}

const fn c(char: char) -> Letter {
    Letter {
        char,
        type_: LetterType::Consonant,
    }
}

const fn o(char: char) -> Letter {
    Letter {
        char,
        type_: LetterType::Other,
    }
}

const RUSSIAN: [Letter; 33] = [
    v('А'),
    c('Б'),
    c('В'),
    c('Г'),
    c('Д'),
    v('Е'),
    v('Ё'),
    c('Ж'),
    c('З'),
    v('И'),
    c('Й'),
    c('К'),
    c('Л'),
    c('М'),
    c('Н'),
    v('О'),
    c('П'),
    c('Р'),
    c('С'),
    c('Т'),
    v('У'),
    c('Ф'),
    c('Х'),
    c('Ц'),
    c('Ч'),
    c('Ш'),
    c('Щ'),
    o('Ъ'),
    v('Ы'),
    o('Ь'),
    v('Э'),
    v('Ю'),
    v('Я'),
];

const ENGLISH: [Letter; 26] = [
    v('A'),
    c('B'),
    c('C'),
    c('D'),
    v('E'),
    c('F'),
    c('J'),
    c('H'),
    v('I'),
    c('J'),
    c('K'),
    c('L'),
    c('M'),
    c('N'),
    v('O'),
    c('P'),
    c('Q'),
    c('R'),
    c('S'),
    c('T'),
    v('U'),
    c('V'),
    c('W'),
    c('X'),
    c('Y'),
    c('Z'),
];

const GREEK: [Letter; 24] = [
    v('Α'),
    c('Β'),
    c('Γ'),
    c('Δ'),
    v('Ε'),
    c('Ζ'),
    v('Η'),
    c('Θ'),
    v('Ι'),
    c('Κ'),
    c('Λ'),
    c('Μ'),
    c('Ν'),
    c('Ξ'),
    v('Ο'),
    c('Π'),
    c('Ρ'),
    c('Σ'),
    c('Τ'),
    v('Υ'),
    c('Φ'),
    c('Χ'),
    c('Ψ'),
    v('Ω'),
];

const NUMBERS: [Letter; 10] = [
    o('0'),
    o('1'),
    o('2'),
    o('3'),
    o('4'),
    o('5'),
    o('6'),
    o('7'),
    o('8'),
    o('9'),
];

pub struct Game {
    font: Font,
}

impl Game {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {
            font: load_default_font().await?,
        })
    }
}

impl crate::Game for Game {
    fn name(&self) -> String {
        "Буквы".to_owned()
    }

    fn draw_preview(&self, rect: Rect) {
        RUSSIAN[0].draw(rect, &self.font)
    }

    fn launch(&self) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        Box::pin(main())
    }
}
