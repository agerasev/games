use crate::{
    clicks,
    draw::{Painter, Sprite},
    layout::{contains, grid, rect},
    pressed_keys,
};
use wgame::{
    canvas::{CanvasInput, Key},
    gfx::types::color,
    glam::Vec2,
};

#[derive(Clone, Copy, Debug)]
struct Letter {
    char: char,
    type_: LetterType,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LetterType {
    Vowel,
    Consonant,
    Other,
}
impl Letter {
    fn text(self) -> String {
        let upper = self.char.to_uppercase().to_string();
        let lower = self.char.to_lowercase().to_string();
        if upper == lower {
            upper
        } else {
            format!("{upper} {lower}")
        }
    }
}

#[derive(Default)]
pub struct Game {
    language: usize,
    font: usize,
    pointer: Option<Vec2>,
}
impl Game {
    pub fn update(&mut self, input: &CanvasInput, size: Vec2) {
        self.pointer = input.pointer;
        for key in pressed_keys(input) {
            match key {
                Key::Character('1') => self.language = 0,
                Key::Character('2') => self.language = 1,
                Key::Character('3') => self.language = 2,
                Key::Character('0') => self.language = 3,
                Key::Character('`') => self.font = 1 - self.font,
                _ => {}
            }
        }
        for pos in clicks(input) {
            for i in 0..4 {
                if contains(button_rect(size, i), pos) {
                    self.language = i;
                }
            }
        }
    }
    pub fn draw(&self, painter: &mut Painter<'_>) {
        let alphabet = [&RUSSIAN[..], &ENGLISH[..], &GREEK[..], &NUMBERS[..]][self.language];
        let size = painter.size;
        let sidebar = (size.x * 0.18).min(80.0);
        for (letter, cell) in alphabet.iter().zip(grid(
            Vec2::new(size.x - sidebar, size.y),
            alphabet.len(),
            2.0,
        )) {
            let color = match letter.type_ {
                LetterType::Vowel => color::RED,
                LetterType::Consonant => color::BLUE,
                LetterType::Other => wgame::rgb::Rgb::new(0.65, 0.65, 0.65),
            };
            painter.label(
                &letter.text(),
                cell.inflate(-cell.size.width * 0.08, -cell.size.height * 0.08),
                cell.size.height * 0.65,
                self.font,
                color,
            );
        }
        for i in 0..4 {
            let button = button_rect(size, i);
            let inner = painter.button(
                button,
                self.language == i,
                self.pointer.is_some_and(|p| contains(button, p)),
            );
            if let Some(sprite) = [Sprite::Russian, Sprite::English, Sprite::Greek].get(i) {
                painter.sprite(*sprite, inner);
            } else {
                painter.label("123", inner, 28.0, 0, color::WHITE);
            }
        }
    }
}
fn button_rect(size: Vec2, i: usize) -> euclid::default::Rect<f32> {
    let width = (size.x * 0.15).min(60.0);
    let height = (size.y / 5.0).min(40.0);
    rect(
        size.x - width - width / 6.0,
        height / 4.0 + i as f32 * height * 1.25,
        width,
        height,
    )
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
    c('G'),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alphabets_have_unique_letters_and_both_cases() {
        for alphabet in [&RUSSIAN[..], &ENGLISH[..], &GREEK[..], &NUMBERS[..]] {
            let chars: std::collections::BTreeSet<_> =
                alphabet.iter().map(|letter| letter.char).collect();
            assert_eq!(chars.len(), alphabet.len());
        }
        assert_eq!(
            ENGLISH.iter().map(|letter| letter.char).collect::<String>(),
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        );
        assert_eq!(RUSSIAN[6].text(), "Ё ё");
        assert_eq!(GREEK[0].text(), "Α α");
        assert_eq!(NUMBERS[0].text(), "0");
    }
}
