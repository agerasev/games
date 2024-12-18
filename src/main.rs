use anyhow::Error;
use macroquad::{
    color,
    file::set_pc_assets_folder,
    input::{is_mouse_button_pressed, mouse_position, MouseButton},
    math::{Rect, Vec2},
    miniquad::window::{screen_size, set_window_size},
    shapes::draw_rectangle_lines,
    window::{clear_background, next_frame},
};
use std::env;
use yarik_games::{
    compat::reset_camera,
    games, layout,
    text::{load_default_font, Text, TextAlign},
};

#[macroquad::main("Yarik Games")]
async fn main() -> Result<(), Error> {
    set_pc_assets_folder("assets");

    let games = games::all().await?;
    let font = load_default_font().await?;

    set_window_size(1280, 720);

    if let Some(name) = env::args().nth(1) {
        match games
            .iter()
            .find_map(|(k, v)| if k == &name { Some(v) } else { None })
        {
            Some(game) => {
                return game.launch().await;
            }
            None => panic!(
                "Game not found: \"{name}\"\nAvailable games: {:?}",
                games.iter().map(|(k, _)| k).collect::<Vec<_>>()
            ),
        }
    }
    loop {
        let screen = Vec2::from(screen_size());
        let boxes = layout::grid((screen.x, screen.y), games.len(), 1.0);

        clear_background(color::BLACK);
        for ((_, game), &rect) in games.iter().zip(boxes.iter().flatten()) {
            {
                let size = rect.size().min_element() / 2.0;
                let rect = Rect::new(
                    rect.center().x - size / 2.0,
                    rect.center().y - size / 2.0,
                    size,
                    size,
                );
                game.draw_preview(rect);
                reset_camera();
            }

            let text = Text::new(
                game.name(),
                Some(font.clone()),
                rect.size().min_element() / 10.0,
            );
            text.draw_aligned(
                rect.center().x,
                rect.bottom() - text.size / 2.0,
                TextAlign::Center,
                color::WHITE,
            );
            if rect.contains(Vec2::from(mouse_position())) {
                if is_mouse_button_pressed(MouseButton::Left) {
                    next_frame().await;
                    game.launch().await?;
                    reset_camera();
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
