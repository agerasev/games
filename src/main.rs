use anyhow::Error;
use euclid::default::{Rect, Size2D};
use wgame::{Window, gfx::types::color};
/*
use macroquad::{
    color,
    file::set_pc_assets_folder,
    input::{is_mouse_button_pressed, mouse_position, MouseButton},
    math::{Rect, Vec2},
    miniquad::window::{screen_size, set_window_size},
    shapes::draw_rectangle_lines,
    window::{clear_background, next_frame},
};
*/
use std::env;
use yarik_games::{games, layout};
/*
use yarik_games::{
    compat::reset_camera,
    games, layout,
    text::{load_default_font, Text, TextAlign},
};
*/

#[wgame::window(title = "Games", size = (1280, 720), resizable = true)]
async fn main(mut window: Window<'_>) -> Result<(), Error> {
    let gfx = wgame::Library::new(window.graphics());

    let games = games::all(&gfx).await?;
    let font = gfx.load_font("assets/free-sans-bold.ttf").await?;

    if let Some(name) = env::args().nth(1) {
        match games
            .iter()
            .find_map(|(k, v)| if k == &name { Some(v) } else { None })
        {
            Some(game) => {
                return game.launch(&mut window).await;
            }
            None => panic!(
                "Game not found: \"{name}\"\nAvailable games: {:?}",
                games.iter().map(|(k, _)| k).collect::<Vec<_>>()
            ),
        }
    }
    while let Some(mut frame) = window.next_frame().await? {
        let screen = Size2D::from(frame.size()).cast::<f32>();

        frame.clear(color::BLACK);
        let mut renderer = frame.with_physical_camera();

        let boxes = layout::grid(screen, games.len(), 1.0);
        for ((_, game), &rect) in games.iter().zip(boxes.iter().flatten()) {
            game.draw_preview(&mut renderer, rect);

            /*
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
            */
        }
    }

    Ok(())
}
