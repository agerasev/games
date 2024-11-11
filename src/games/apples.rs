use crate::text::{draw_text_aligned, TextAlign};
use anyhow::Error;
use macroquad::{
    color,
    input::{is_key_down, is_key_pressed, KeyCode},
    math::{Rect, Vec2},
    miniquad::window::screen_size,
    text::load_ttf_font,
    texture::{
        draw_texture_ex, load_texture, set_default_filter_mode, DrawTextureParams, FilterMode,
        Texture2D,
    },
    time::get_frame_time,
    ui::{root_ui, widgets::Button, Skin},
    window::{clear_background, next_frame},
};
use rand::{distributions::Uniform, rngs::SmallRng, Rng, SeedableRng};
use std::{future::Future, pin::Pin, time::Duration};

/// Grammatical gender
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

struct Item {
    image: Texture2D,
    stem: &'static str,
    endings: [&'static str; 3],
    gender: Gender,
}

const INPUT_TIMEOUT: Duration = Duration::from_secs(4);

pub async fn main() -> Result<(), Error> {
    set_default_filter_mode(FilterMode::Nearest);
    let items = [
        Item {
            image: load_texture("apple.png").await?,
            stem: "яблок",
            endings: ["о", "а", ""],
            gender: Gender::Neuter,
        },
        Item {
            image: load_texture("pear.png").await?,
            stem: "груш",
            endings: ["а", "и", ""],
            gender: Gender::Feminine,
        },
        Item {
            image: load_texture("orange.png").await?,
            stem: "апельсин",
            endings: ["", "а", "ов"],
            gender: Gender::Masculine,
        },
    ];

    let font = load_ttf_font("default.ttf").await?;
    {
        let mut ui = root_ui();
        let style = ui
            .style_builder()
            .with_font(&font)?
            .font_size(20)
            .color(color::BLACK)
            .color_hovered(color::WHITE)
            .color_clicked(color::RED)
            .text_color(color::WHITE)
            .text_color_hovered(color::BLACK)
            .text_color_clicked(color::BLACK)
            .build();
        let skin = Skin {
            button_style: style,
            ..ui.default_skin()
        };
        ui.push_skin(&skin);
        ui.clear_input_focus();
    }
    let mut number_font = Some(&font);

    let mut max_number = 10;

    let mut rng = SmallRng::from_entropy();
    let mut number: i64 = rng.sample(Uniform::new_inclusive(1, max_number));
    let mut item = &items[0];

    let mut input = Vec::<i64>::new();
    let mut input_cooldown = Duration::ZERO;

    while !is_key_down(KeyCode::Escape) {
        if number > max_number {
            number = max_number;
        }
        let dt = Duration::from_secs_f32(get_frame_time());

        let viewport = Vec2::from(screen_size());
        let scale = viewport.y / 10.0;

        {
            if is_key_pressed(KeyCode::GraveAccent) {
                number_font = match number_font {
                    Some(_) => None,
                    None => Some(&font),
                };
            }

            const NUM_KEYS: [[KeyCode; 2]; 10] = [
                [KeyCode::Key0, KeyCode::Kp0],
                [KeyCode::Key1, KeyCode::Kp1],
                [KeyCode::Key2, KeyCode::Kp2],
                [KeyCode::Key3, KeyCode::Kp3],
                [KeyCode::Key4, KeyCode::Kp4],
                [KeyCode::Key5, KeyCode::Kp5],
                [KeyCode::Key6, KeyCode::Kp6],
                [KeyCode::Key7, KeyCode::Kp7],
                [KeyCode::Key8, KeyCode::Kp8],
                [KeyCode::Key9, KeyCode::Kp9],
            ];
            let mut key_num = None;
            for (i, [k, kp]) in NUM_KEYS.iter().copied().enumerate() {
                if is_key_pressed(k) || is_key_pressed(kp) {
                    key_num = Some(i as i64);
                }
            }

            let mut apply = false;

            if !input.is_empty() {
                input_cooldown = input_cooldown.saturating_sub(dt);
                if input_cooldown.is_zero() {
                    input.clear();
                }
            }
            if let Some(n) = key_num {
                input.push(n);
                input_cooldown = INPUT_TIMEOUT;
            }

            let mut add = 0;
            if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
                add -= 1;
                apply = true;
            }
            if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
                add += 1;
                apply = true;
            }
            if is_key_pressed(KeyCode::PageDown) {
                add -= 10;
                apply = true;
            }
            if is_key_pressed(KeyCode::PageUp) {
                add += 10;
                apply = true;
            }
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                apply = true;
            }
            if is_key_pressed(KeyCode::Backspace) || is_key_pressed(KeyCode::Delete) {
                input.clear();
            }

            if apply && !input.is_empty() || 10i64.pow(input.len() as u32) >= max_number {
                number = input.iter().fold(0, |a, n| a * 10 + n);
                input.clear();
            }
            if apply {
                number = (number + add).clamp(0, max_number);
            }
        }

        clear_background(color::BLACK);

        if max_number <= 10 {
            draw_items(
                number,
                Vec2::new(viewport.x / 2.0, viewport.y / 4.0),
                scale,
                &item.image,
                true,
            );
        } else {
            draw_items(
                number,
                Vec2::new(viewport.x / 4.0, viewport.y / 4.0),
                0.5 * scale,
                &item.image,
                false,
            );
        }

        draw_text_aligned(
            "=",
            viewport.x / 2.0,
            viewport.y / 2.0,
            TextAlign::Center,
            number_font,
            2.0 * scale,
            color::WHITE,
        );

        let text_pos = if max_number <= 10 {
            Vec2::new(0.5 * viewport.x, 0.75 * viewport.y)
        } else {
            Vec2::new(0.75 * viewport.x, 0.5 * viewport.y)
        };
        if !input.is_empty() {
            let text = input.iter().fold(String::new(), |s, n| s + &n.to_string()) + "_";
            draw_text_aligned(
                &text,
                text_pos.x,
                text_pos.y - 2.0 * scale,
                TextAlign::Center,
                number_font,
                0.25 * scale,
                color::DARKGRAY,
            );
        }
        draw_text_aligned(
            &format!("{number}"),
            text_pos.x,
            text_pos.y,
            TextAlign::Center,
            number_font,
            2.0 * scale,
            color::WHITE,
        );
        draw_text_aligned(
            &items_text(number, item.stem, item.endings, item.gender),
            text_pos.x,
            text_pos.y + 1.0 * scale,
            TextAlign::Center,
            Some(&font),
            0.5 * scale,
            color::WHITE,
        );

        {
            let mut ui = root_ui();

            for (i, it) in items.iter().enumerate() {
                if Button::new(it.image.clone())
                    .position(Vec2::new(10.0 + i as f32 * (10.0 + 32.0), 10.0))
                    .size(Vec2::new(32.0, 32.0))
                    .ui(&mut ui)
                {
                    item = it;
                }
            }

            if Button::new("10")
                .position(Vec2::new(viewport.x - 70.0, 10.0))
                .size(Vec2::new(60.0, 30.0))
                .ui(&mut ui)
            {
                max_number = 10;
            }
            if Button::new("100")
                .position(Vec2::new(viewport.x - 70.0, 50.0))
                .size(Vec2::new(60.0, 30.0))
                .ui(&mut ui)
            {
                max_number = 100;
            }
        }

        next_frame().await
    }

    Ok(())
}

fn draw_items(number: i64, pos: Vec2, scale: f32, texture: &Texture2D, gap: bool) {
    let padding = 0.1;
    let width = {
        let n = number.min(10);
        padding * (n + if gap { 2 * (n / 5) } else { 0 }) as f32 + n as f32
    };
    for j in 0..=(number / 10) {
        for i in 0..(number - j * 10).min(10) {
            draw_texture_ex(
                texture,
                pos.x
                    + scale
                        * (-width / 2.0
                            + padding * if gap { i + 2 * (i / 5) } else { 0 } as f32
                            + i as f32),
                pos.y + scale * (-0.5 + (1.0 + padding) * j as f32),
                color::WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(scale, scale)),
                    ..Default::default()
                },
            );
        }
    }
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
                    "одиинадцать",
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

pub struct Game {
    apple: Texture2D,
}

impl Game {
    pub async fn new() -> Result<Self, Error> {
        set_default_filter_mode(FilterMode::Nearest);
        Ok(Self {
            apple: load_texture("apple.png").await?,
        })
    }
}

impl crate::Game for Game {
    fn name(&self) -> String {
        "Считаем яблоки".to_owned()
    }

    fn draw_preview(&self, rect: Rect) {
        draw_texture_ex(
            &self.apple,
            rect.x,
            rect.y,
            color::WHITE,
            DrawTextureParams {
                dest_size: Some(rect.size()),
                ..Default::default()
            },
        );
    }

    fn launch(&self) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        Box::pin(main())
    }
}
