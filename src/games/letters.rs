use crate::{draw::Painter, layout::grid, pressed_keys};
use wgame::{
    canvas::{CanvasInput, Key},
    gfx::types::color,
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

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Action {
    Alphabet(usize),
    Font(usize),
}

#[derive(Default)]
pub struct Game {
    language: usize,
    font: usize,
}
impl Game {
    pub(crate) fn action(&mut self, action: Action) {
        match action {
            Action::Alphabet(index) => self.language = index,
            Action::Font(index) => self.font = index,
        }
    }
    pub(crate) fn controls(&self, ui: &mut wgame_egui::egui::Ui) -> Vec<Action> {
        let mut actions = Vec::new();
        ui.horizontal_wrapped(|ui| {
            for (index, label) in ["Русский", "English", "Ελληνικά", "123"]
                .into_iter()
                .enumerate()
            {
                if ui.selectable_label(self.language == index, label).clicked() {
                    actions.push(Action::Alphabet(index));
                }
            }
        });
        ui.collapsing("Шрифт", |ui| {
            ui.horizontal_wrapped(|ui| {
                for (index, label) in ["Без засечек", "С засечками"].into_iter().enumerate()
                {
                    if ui.selectable_label(self.font == index, label).clicked() {
                        actions.push(Action::Font(index));
                    }
                }
            });
        });
        actions
    }
    pub fn update(&mut self, input: &CanvasInput) {
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
    }
    pub fn draw(&self, painter: &mut Painter<'_>) {
        let alphabet = [&RUSSIAN[..], &ENGLISH[..], &GREEK[..], &NUMBERS[..]][self.language];
        let size = painter.size;
        for (letter, cell) in alphabet.iter().zip(grid(size, alphabet.len(), 2.0)) {
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
    }
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
