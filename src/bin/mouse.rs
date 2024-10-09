use std::time::Duration;

use derive_more::derive::{Deref, DerefMut};
use futures::{future::try_join_all, TryFutureExt};
use macroquad::{
    color,
    input::{is_key_down, KeyCode},
    math::Vec2,
    miniquad::window::screen_size,
    shapes::draw_rectangle_lines,
    text::draw_text,
    texture::{
        draw_texture_ex, load_texture, set_default_filter_mode, DrawTextureParams, FilterMode,
        Texture2D,
    },
    time::get_frame_time,
    window::{clear_background, next_frame},
    Error,
};
use rand::{
    distributions::{Uniform, WeightedIndex},
    rngs::SmallRng,
    Rng, SeedableRng,
};

#[derive(Clone, Debug)]
pub struct Item {
    pub pos: Vec2,
    pub image: Texture2D,
    pub radius: f32,
}

impl Item {
    pub fn draw(&self, scale: f32) {
        draw_texture_ex(
            &self.image,
            (self.pos.x - self.radius) * scale,
            (self.pos.y - self.radius) * scale,
            color::WHITE,
            DrawTextureParams {
                dest_size: Some(scale * 2.0 * Vec2::new(self.radius, self.radius)),
                ..Default::default()
            },
        );
    }
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct Player {
    #[deref]
    #[deref_mut]
    pub base: Item,
    pub speed: f32,
}

#[macroquad::main("Mouse")]
async fn main() -> Result<(), Error> {
    set_default_filter_mode(FilterMode::Nearest);
    let player_image = load_texture("assets/mouse.png").await?;
    let items_images_and_probs = try_join_all(
        vec![("assets/cheese.png", 0.8), ("assets/apple.png", 0.2)]
            .into_iter()
            .map(|(path, prob)| load_texture(path).map_ok(move |t| (t, prob))),
    )
    .await?;

    let mut rng = SmallRng::from_entropy();

    loop {
        let map_size = Vec2::from([42.0, 24.0]);
        let num_items = 16;

        let mut player = Player {
            base: Item {
                pos: map_size / 2.0,
                image: player_image.clone(),
                radius: 1.0,
            },
            speed: 10.0,
        };

        let mut items: Vec<_> = {
            let item_radius = 0.5;
            (0..num_items)
                .map(|_| Item {
                    pos: Vec2::from([
                        rng.sample(Uniform::new(item_radius, map_size.x - item_radius)),
                        rng.sample(Uniform::new(item_radius, map_size.y - item_radius)),
                    ]),
                    image: items_images_and_probs[rng.sample(
                        WeightedIndex::new(items_images_and_probs.iter().map(|(_, prob)| prob))
                            .unwrap(),
                    )]
                    .0
                    .clone(),
                    radius: item_radius,
                })
                .collect()
        };

        let mut timeout = Duration::from_secs_f32(1.0);

        'game: loop {
            let scale = (Vec2::from(screen_size()) / map_size).min_element();
            let dt = Duration::from_secs_f32(get_frame_time());

            // Move player
            {
                let mut motion = Vec2::ZERO;
                if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
                    motion -= Vec2::from([0.0, 1.0]);
                }
                if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
                    motion += Vec2::from([0.0, 1.0]);
                }
                if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
                    motion -= Vec2::from([1.0, 0.0]);
                }
                if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
                    motion += Vec2::from([1.0, 0.0]);
                }
                let step = player.speed * dt.as_secs_f32();
                player.pos += motion * step;

                player.pos = player.pos.clamp(
                    Vec2::from([player.radius; 2]),
                    map_size - Vec2::from([player.radius; 2]),
                );
            }

            // Collect items and exit if no items remain
            {
                items.retain(|item| {
                    (player.pos - item.pos).length() > (player.radius + item.radius)
                });

                if items.is_empty() {
                    if timeout.is_zero() {
                        break 'game;
                    } else {
                        timeout = timeout.saturating_sub(dt);
                    }
                }
            }

            // Draw frame
            {
                clear_background(color::BLACK);

                draw_rectangle_lines(
                    0.0,
                    0.0,
                    scale * map_size.x,
                    scale * map_size.y,
                    2.0,
                    color::GRAY,
                );

                for item in &items {
                    item.draw(scale);
                }
                player.draw(scale);

                draw_text(
                    &format!("{}", num_items - items.len()),
                    4.0,
                    scale * 1.2 + 4.0,
                    scale * 2.0,
                    color::WHITE,
                );
            }

            next_frame().await;
        }
    }
}
