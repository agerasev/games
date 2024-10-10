use crate::{
    text::{Text, TextAlign},
    Game,
};
use macroquad::{
    color,
    input::{get_keys_pressed, is_key_down, KeyCode},
    math::{Rect, Vec2},
    miniquad::window::screen_size,
    text::load_ttf_font,
    texture::{
        draw_texture_ex, load_texture, set_default_filter_mode, DrawTextureParams, FilterMode,
        Texture2D,
    },
    window::{clear_background, next_frame},
    Error,
};
use rand::{distributions::Uniform, rngs::SmallRng, Rng, SeedableRng};
use std::{future::Future, pin::Pin};

pub async fn main() -> Result<(), Error> {
    set_default_filter_mode(FilterMode::Nearest);
    let item_texture = load_texture("assets/apple.png").await?;
    let item_size = item_texture.size() * 4.0;
    let padding = 10.0;

    let font = load_ttf_font("assets/default.ttf").await?;

    let mut rng = SmallRng::from_entropy();
    let mut number: i64 = rng.sample(Uniform::new_inclusive(1, 10));

    while !is_key_down(KeyCode::Escape) {
        let viewport = Vec2::from(screen_size());
        let scale = viewport.y / 800.0;

        {
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
            let keys = get_keys_pressed();
            for (i, [k, kp]) in NUM_KEYS.iter().enumerate() {
                if keys.contains(k) || keys.contains(kp) {
                    number = i as i64;
                }
            }
            if keys.contains(&KeyCode::Minus) || keys.contains(&KeyCode::KpSubtract) {
                number = (number - 1).max(0);
            }
            if keys.contains(&KeyCode::Equal) || keys.contains(&KeyCode::KpAdd) {
                number = (number + 1).min(10);
            }
        }

        clear_background(color::BLACK);
        {
            let width = padding * (number + 2 * (number / 5)) as f32 + item_size.x * number as f32;
            for i in 0..number {
                draw_texture_ex(
                    &item_texture,
                    viewport.x / 2.0
                        + scale
                            * (-width / 2.0
                                + padding * (i + 2 * (i / 5)) as f32
                                + item_size.x * i as f32),
                    viewport.y / 4.0 - scale * item_size.y / 2.0,
                    color::WHITE,
                    DrawTextureParams {
                        dest_size: Some(item_size * scale),
                        ..Default::default()
                    },
                );
            }
        }

        Text::new("=", 100.0 * scale, Some(&font)).draw(
            viewport.x / 2.0,
            viewport.y / 2.0,
            TextAlign::Center,
            color::WHITE,
        );
        Text::new(format!("{number}"), 100.0 * scale, Some(&font)).draw(
            viewport.x / 2.0,
            viewport.y * 3.0 / 4.0,
            TextAlign::Center,
            color::WHITE,
        );

        Text::new(items_name(number), 50.0 * scale, Some(&font)).draw(
            viewport.x / 2.0,
            viewport.y * 7.0 / 8.0,
            TextAlign::Center,
            color::WHITE,
        );

        next_frame().await
    }

    Ok(())
}

fn items_name(n: i64) -> String {
    let n = n.abs();
    format!(
        "яблок{}",
        if (n % 100) / 10 != 1 {
            match n % 10 {
                0 | 5..=9 => "",
                1 => "о",
                2..=4 => "а",
                _ => unreachable!(),
            }
        } else {
            ""
        }
    )
}

pub struct ApplesGame {
    apple: Texture2D,
}

impl ApplesGame {
    pub async fn new() -> Result<Self, Error> {
        set_default_filter_mode(FilterMode::Nearest);
        Ok(Self {
            apple: load_texture("assets/apple.png").await?,
        })
    }
}

impl Game for ApplesGame {
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
