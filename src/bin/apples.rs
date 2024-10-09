use macroquad::{
    color,
    input::{get_keys_pressed, KeyCode},
    math::Vec2,
    miniquad::window::screen_size,
    text::draw_text,
    texture::{
        draw_texture_ex, load_texture, set_default_filter_mode, DrawTextureParams, FilterMode,
    },
    window::{clear_background, next_frame},
    Error,
};
use rand::{distributions::Uniform, rngs::SmallRng, Rng, SeedableRng};

#[macroquad::main("Apples")]
async fn main() -> Result<(), Error> {
    set_default_filter_mode(FilterMode::Nearest);
    let item_texture = load_texture("assets/apple.png").await?;
    let item_size = item_texture.size() * 4.0;
    let padding = 10.0;

    let mut rng = SmallRng::from_entropy();
    let mut number: i64 = rng.sample(Uniform::new_inclusive(1, 10));

    loop {
        let viewport = Vec2::from(screen_size());

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
        let width = padding * (number + number / 5) as f32 + item_size.x * number as f32;
        for i in 0..number {
            draw_texture_ex(
                &item_texture,
                viewport.x / 2.0 - width / 2.0
                    + padding * (i + 2 * (i / 5)) as f32
                    + item_size.x * i as f32,
                viewport.y / 4.0 - item_size.y / 2.0,
                color::WHITE,
                DrawTextureParams {
                    dest_size: Some(item_size),
                    ..Default::default()
                },
            );
        }

        draw_text("=", viewport.x / 2.0, viewport.y / 2.0, 160.0, color::WHITE);
        draw_text(
            &format!("{number}"),
            viewport.x / 2.0,
            viewport.y * 3.0 / 4.0,
            160.0,
            color::WHITE,
        );

        next_frame().await
    }
}
