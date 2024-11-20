use crate::{
    layout::grid,
    text::{draw_text_aligned, load_default_font, TextAlign},
};
use anyhow::Error;
use itertools::Itertools;
use macroquad::{
    color,
    input::{is_key_down, is_key_pressed, KeyCode},
    math::Rect,
    miniquad::window::screen_size,
    text::{load_ttf_font, Font},
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

pub async fn main() -> Result<(), Error> {
    let fonts = [
        load_ttf_font("free-sans-bold.ttf").await?,
        load_ttf_font("free-serif-bold.ttf").await?,
    ];

    let mut alphabet = &RUSSIAN[..];
    let mut font_index = 0;

    while !is_key_down(KeyCode::Escape) {
        if is_key_pressed(KeyCode::Key1) {
            alphabet = &RUSSIAN[..];
        } else if is_key_pressed(KeyCode::Key2) {
            alphabet = &ENGLISH[..];
        } else if is_key_pressed(KeyCode::Key3) {
            alphabet = &GREEK[..];
        } else if is_key_pressed(KeyCode::Key0) {
            alphabet = &NUMBERS[..];
        }
        if is_key_pressed(KeyCode::GraveAccent) {
            font_index = (font_index + 1) % fonts.len();
        }

        clear_background(color::BLACK);

        let font = &fonts[font_index];
        let mut i = 0;
        for line in grid(screen_size(), alphabet.len(), 2.0) {
            for rect in line {
                alphabet[i].draw(rect, font);
                i += 1;
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
