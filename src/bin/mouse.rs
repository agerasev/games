use std::time::Duration;

use derive_more::derive::{Deref, DerefMut};
use futures::{future::try_join_all, TryFutureExt};
use macroquad::{
    color,
    input::{is_key_down, KeyCode},
    math::Vec2,
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
    pub fn size(&self) -> Vec2 {
        2.0 * Vec2::new(self.radius, self.radius)
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
    let player_image = load_texture("assets/mouse.png").await?;
    let items_images_and_probs = try_join_all(
        vec![("assets/cheese.png", 0.8), ("assets/apple.png", 0.2)]
            .into_iter()
            .map(|(path, prob)| load_texture(path).map_ok(move |t| (t, prob))),
    )
    .await?;
    set_default_filter_mode(FilterMode::Nearest);

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
        let mut counter = 0;

        'game: loop {
            let scale = 30.0;
            let dt = Duration::from_secs_f32(get_frame_time());

            // Draw frame
            {
                clear_background(color::BLACK);

                for item in &items {
                    let scaled_pos = scale * item.pos;
                    draw_texture_ex(
                        &item.image,
                        scaled_pos.x,
                        scaled_pos.y,
                        color::WHITE,
                        DrawTextureParams {
                            dest_size: Some(scale * item.size()),
                            ..Default::default()
                        },
                    );
                }
                let scaled_pos = scale * player.pos;
                draw_texture_ex(
                    &player.image,
                    scaled_pos.x,
                    scaled_pos.y,
                    color::WHITE,
                    DrawTextureParams {
                        dest_size: Some(scale * player.size()),
                        ..Default::default()
                    },
                );

                draw_text(&format!("{counter}"), 0.0, 20.0, 40.0, color::WHITE);
            }

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

                player.pos = player.pos.clamp(player.size(), map_size - player.size());
            }

            // Collect items and exit if no items remain
            {
                items.retain(|item| {
                    (player.pos - item.pos).length() > (player.radius + item.radius)
                });
                counter = num_items - items.len();

                if items.is_empty() {
                    if timeout.is_zero() {
                        break 'game;
                    } else {
                        timeout = timeout.saturating_sub(dt);
                    }
                }
            }

            next_frame().await;
        }
    }
}
