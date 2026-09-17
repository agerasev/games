use crate::{
    clicks,
    draw::{Painter, Sprite},
    layout::{contains, rect},
    pressed_keys,
};
use euclid::default::Rect;
use rand::{Rng, SeedableRng, rngs::SmallRng};
use wgame::{
    canvas::{CanvasInput, Event, Key},
    gfx::types::color,
    glam::{Vec2, Vec3},
};

#[derive(Clone, Copy, Debug)]
enum Gender {
    Masculine,
    Feminine,
    Neuter,
}
struct Fruit {
    sprite: Sprite,
    stem: &'static str,
    endings: [&'static str; 3],
    gender: Gender,
}
const FRUITS: [Fruit; 3] = [
    Fruit {
        sprite: Sprite::Apple,
        stem: "яблок",
        endings: ["о", "а", ""],
        gender: Gender::Neuter,
    },
    Fruit {
        sprite: Sprite::Pear,
        stem: "груш",
        endings: ["а", "и", ""],
        gender: Gender::Feminine,
    },
    Fruit {
        sprite: Sprite::Orange,
        stem: "апельсин",
        endings: ["", "а", "ов"],
        gender: Gender::Masculine,
    },
];

pub struct Game {
    number: u16,
    max_number: u16,
    fruit: usize,
    font: usize,
    pending: Option<u16>,
    cooldown: f32,
    pointer: Option<Vec2>,
}
impl Game {
    pub fn new() -> Self {
        Self {
            number: SmallRng::seed_from_u64(0xdeadbeef).random_range(1..=10),
            max_number: 10,
            fruit: 0,
            font: 0,
            pending: None,
            cooldown: 0.0,
            pointer: None,
        }
    }
    fn apply(&mut self, add: i16) {
        self.number = (self.pending.take().unwrap_or(self.number) as i16 + add)
            .clamp(0, self.max_number as i16) as u16;
    }
    fn key(&mut self, key: Key) {
        match key {
            Key::Character(c @ '0'..='9') => {
                let n = self.pending.unwrap_or(0) * 10 + (c as u16 - '0' as u16);
                self.pending = Some(n.min(self.max_number));
                self.cooldown = 4.0;
                // Keep valid prefixes (1 -> 10 -> 100); commit when no further
                // digit fits the selected range. Enter commits a shorter number.
                if n == 0 || n * 10 > self.max_number {
                    self.apply(0);
                }
            }
            Key::Plus => self.apply(1),
            Key::Minus => self.apply(-1),
            Key::PageUp => self.apply(10),
            Key::PageDown => self.apply(-10),
            Key::Enter | Key::Space => self.apply(0),
            Key::Backspace | Key::Delete => self.pending = None,
            Key::Character('`') => self.font = 1 - self.font,
            _ => {}
        }
    }
    pub fn update(&mut self, input: &CanvasInput, dt: f32, size: Vec2) {
        self.pointer = input.pointer;
        self.cooldown = (self.cooldown - dt).max(0.0);
        if self.cooldown == 0.0 || input.events.contains(&Event::Cancelled) {
            self.pending = None;
        }
        for key in pressed_keys(input) {
            self.key(key);
        }
        for pos in clicks(input) {
            for i in 0..3 {
                if contains(fruit_button(size, i), pos) {
                    self.fruit = i;
                }
            }
            for (i, max) in [10, 100].into_iter().enumerate() {
                if contains(range_button(size, i), pos) {
                    self.max_number = max;
                    self.number = self.number.min(max);
                    self.pending = None;
                }
            }
        }
    }
    pub fn draw(&self, painter: &mut Painter<'_>) {
        let size = painter.size;
        let fruit = &FRUITS[self.fruit];
        let top = size.y.min(50.0);
        let height = (size.y - top).max(0.0);
        let wide = self.max_number > 10 && size.x > size.y * 1.1;
        let fruit_box = if wide {
            rect(size.x * 0.03, top, size.x * 0.43, height)
        } else {
            rect(
                size.x * 0.04,
                top,
                size.x * 0.92,
                height * if self.max_number == 10 { 0.3 } else { 0.58 },
            )
        };
        let rows = self.number.div_ceil(10).max(1);
        let columns = self.number.clamp(1, 10);
        let cell = (fruit_box.size.width / (columns as f32 * 1.1 + 0.4))
            .min(fruit_box.size.height / (rows as f32 * 1.1));
        let origin = Vec2::new(
            fruit_box.center().x - cell * (columns as f32 * 1.1 + 0.2) / 2.0,
            fruit_box.center().y - cell * rows as f32 * 1.1 / 2.0,
        );
        for i in 0..self.number {
            let col = i % 10;
            let gap = if col >= 5 { 0.2 } else { 0.0 };
            painter.sprite(
                fruit.sprite,
                rect(
                    origin.x + cell * (col as f32 * 1.1 + gap),
                    origin.y + cell * (i / 10) as f32 * 1.1,
                    cell,
                    cell,
                ),
            );
        }
        let result = if wide {
            rect(size.x * 0.54, top, size.x * 0.43, height)
        } else {
            rect(
                size.x * 0.04,
                fruit_box.max_y(),
                size.x * 0.92,
                size.y - fruit_box.max_y(),
            )
        };
        let equals = if wide {
            rect(size.x * 0.46, size.y * 0.4, size.x * 0.08, height * 0.2)
        } else {
            rect(
                result.min_x(),
                result.min_y(),
                result.size.width,
                result.size.height * 0.2,
            )
        };
        painter.label("=", equals, height * 0.14, self.font, color::WHITE);
        painter.label(
            &self.number.to_string(),
            rect(
                result.min_x(),
                result.min_y() + result.size.height * 0.3,
                result.size.width,
                result.size.height * 0.35,
            ),
            height * 0.26,
            self.font,
            color::WHITE,
        );
        if let Some(n) = self.pending {
            painter.label(
                &format!("{n}_"),
                rect(
                    result.min_x(),
                    result.min_y() + result.size.height * 0.2,
                    result.size.width,
                    result.size.height * 0.1,
                ),
                height * 0.05,
                self.font,
                Vec3::splat(0.6),
            );
        }
        painter.label(
            &items_text(
                i64::from(self.number),
                fruit.stem,
                fruit.endings,
                fruit.gender,
            ),
            rect(
                result.min_x(),
                result.min_y() + result.size.height * 0.72,
                result.size.width,
                result.size.height * 0.2,
            ),
            height * 0.08,
            0,
            color::WHITE,
        );
        for (i, fruit) in FRUITS.iter().enumerate() {
            let button = fruit_button(size, i);
            let inner = painter.button(
                button,
                i == self.fruit,
                self.pointer.is_some_and(|p| contains(button, p)),
            );
            painter.sprite(fruit.sprite, inner);
        }
        for (i, max) in [10, 100].into_iter().enumerate() {
            let button = range_button(size, i);
            painter.button(
                button,
                max == self.max_number,
                self.pointer.is_some_and(|p| contains(button, p)),
            );
            painter.label(&max.to_string(), button, 24.0, 0, color::WHITE);
        }
    }
}
fn fruit_button(size: Vec2, i: usize) -> Rect<f32> {
    let side = (size.x / 8.0).min(36.0).min(size.y * 0.1);
    rect(4.0 + i as f32 * (side + 4.0), 4.0, side, side)
}
fn range_button(size: Vec2, i: usize) -> Rect<f32> {
    let width = (size.x / 6.0).min(55.0);
    rect(
        size.x - (2 - i) as f32 * (width + 4.0),
        4.0,
        width,
        36.0_f32.min(size.y * 0.1),
    )
}

fn items_text(mut n: i64, stem: &str, endings: [&str; 3], gender: Gender) -> String {
    n = n.abs();
    let mut words = Vec::new();

    if n == 0 {
        words.push("ноль".to_string());
    } else {
        let h = n / 100;
        if h == 1 {
            words.push("сто".to_string());
        } else if h != 0 {
            unimplemented!();
        }
        n %= 100;

        let d = n / 10;
        let u = n % 10;
        if d == 1 {
            words.push(
                [
                    "десять",
                    "одиннадцать",
                    "двенадцать",
                    "тринадцать",
                    "четырнадцать",
                    "пятнадцать",
                    "шестнадцать",
                    "семнадцать",
                    "восемнадцать",
                    "девятнадцать",
                ][u as usize]
                    .to_string(),
            )
        } else {
            if d > 1 {
                words.push(
                    [
                        "двадцать",
                        "тридцать",
                        "сорок",
                        "пятьдесят",
                        "шестьдесят",
                        "семьдесят",
                        "восемьдесят",
                        "девяносто",
                    ][(d - 2) as usize]
                        .to_string(),
                );
            }
            if u != 0 {
                words.push(
                    [
                        match gender {
                            Gender::Masculine => "один",
                            Gender::Feminine => "одна",
                            Gender::Neuter => "одно",
                        },
                        match gender {
                            Gender::Feminine => "две",
                            _ => "два",
                        },
                        "три",
                        "четыре",
                        "пять",
                        "шесть",
                        "семь",
                        "восемь",
                        "девять",
                    ][(u - 1) as usize]
                        .to_string(),
                );
            }
        }
    }

    words.push(format!(
        "{stem}{}",
        if (n % 100) / 10 != 1 {
            match n % 10 {
                1 => endings[0],
                2..=4 => endings[1],
                0 | 5..=9 => endings[2],
                _ => unreachable!(),
            }
        } else {
            endings[2]
        }
    ));

    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn entry_accepts_ten_and_one_hundred_and_clamps_adjustments() {
        let mut game = Game::new();
        for c in "10".chars() {
            game.key(Key::Character(c));
        }
        assert_eq!(game.number, 10);
        game.max_number = 100;
        for c in "100".chars() {
            game.key(Key::Character(c));
        }
        assert_eq!(game.number, 100);
        assert_eq!(game.pending, None);
        game.key(Key::Plus);
        assert_eq!(game.number, 100);
        game.key(Key::Character('2'));
        game.key(Key::Enter);
        assert_eq!(game.number, 2);
        game.key(Key::PageDown);
        assert_eq!(game.number, 0);
        game.key(Key::PageUp);
        assert_eq!(game.number, 10);
    }
    #[test]
    fn pending_entry_times_out_and_delete_cancels() {
        let mut game = Game::new();
        game.max_number = 100;
        let original = game.number;
        game.key(Key::Character('3'));
        game.update(&CanvasInput::default(), 4.1, Vec2::new(800.0, 600.0));
        assert_eq!(game.pending, None);
        assert_eq!(game.number, original);
        game.key(Key::Character('1'));
        game.key(Key::Backspace);
        assert_eq!(game.pending, None);
    }
    #[test]
    fn russian_numbers_and_endings() {
        let phrase = |n, i: usize| {
            let f = &FRUITS[i];
            items_text(n, f.stem, f.endings, f.gender)
        };
        for (n, expected) in [
            (0, "ноль яблок"),
            (1, "одно яблоко"),
            (2, "два яблока"),
            (11, "одиннадцать яблок"),
            (21, "двадцать одно яблоко"),
            (100, "сто яблок"),
        ] {
            assert_eq!(phrase(n, 0), expected);
        }
        assert_eq!(phrase(1, 1), "одна груша");
        assert_eq!(phrase(2, 1), "две груши");
        assert_eq!(phrase(12, 1), "двенадцать груш");
        assert_eq!(phrase(1, 2), "один апельсин");
        assert_eq!(phrase(5, 2), "пять апельсинов");
    }
}
