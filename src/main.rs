use macroquad::{
    color,
    input::{is_mouse_button_pressed, mouse_position, MouseButton},
    math::{Rect, Vec2},
    miniquad::window::screen_size,
    shapes::draw_rectangle_lines,
    text::draw_text,
    window::{clear_background, next_frame},
    Error,
};
use yarik::{games, layout};

#[macroquad::main("Yarik")]
async fn main() -> Result<(), Error> {
    let games = games::all().await?;

    loop {
        let boxes = layout::grid(screen_size(), games.len());
        clear_background(color::BLACK);
        for ((name, game), rect) in games.iter().zip(boxes.iter()) {
            game.draw_preview({
                let size = rect.size().min_element() / 2.0;
                Rect::new(
                    rect.center().x - size / 2.0,
                    rect.center().y - size / 2.0,
                    size,
                    size,
                )
            });
            draw_text(
                name,
                rect.center().x,
                rect.bottom() - 20.0,
                40.0,
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
