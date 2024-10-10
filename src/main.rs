use macroquad::{
    color,
    input::{is_mouse_button_pressed, mouse_position, MouseButton},
    math::{Rect, Vec2},
    miniquad::window::screen_size,
    shapes::draw_rectangle_lines,
    text::load_ttf_font,
    window::{clear_background, next_frame},
    Error,
};
use std::env;
use yarik::{
    games, layout,
    text::{Text, TextAlign},
};

#[macroquad::main("Yarik")]
async fn main() -> Result<(), Error> {
    let games = games::all().await?;
    let font = load_ttf_font("assets/default.ttf").await?;

    if let Some(name) = env::args().nth(1) {
        games
            .get(&name)
            .unwrap_or_else(|| {
                panic!(
                    "Game not found: \"{name}\"\nAvailable games: {:?}",
                    games.keys()
                )
            })
            .launch()
            .await?;
    }
    loop {
        let boxes = layout::grid(screen_size(), games.len());
        clear_background(color::BLACK);
        for ((name, game), rect) in games.iter().zip(boxes.iter().flatten()) {
            game.draw_preview({
                let size = rect.size().min_element() / 2.0;
                Rect::new(
                    rect.center().x - size / 2.0,
                    rect.center().y - size / 2.0,
                    size,
                    size,
                )
            });
            let text = Text::new(name, rect.size().min_element() / 10.0, Some(&font));
            text.draw(
                rect.center().x,
                rect.bottom() - text.size / 2.0,
                TextAlign::Center,
                color::WHITE,
            );
            if rect.contains(Vec2::from(mouse_position())) {
                if is_mouse_button_pressed(MouseButton::Left) {
                    next_frame().await;
                    game.launch().await?;
                    continue;
                }

                let margin = 4.0;
                draw_rectangle_lines(
                    rect.x + margin,
                    rect.y + margin,
                    rect.w - 2.0 * margin,
                    rect.h - 2.0 * margin,
                    8.0,
                    color::GRAY,
                );
            }
        }

        next_frame().await
    }
}
