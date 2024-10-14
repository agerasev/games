use crate::text::{Text, TextAlign};
use anyhow::Error;
use core::f32;
use derive_more::derive::{Deref, DerefMut};
use futures::{future::try_join_all, TryFutureExt};
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    color,
    input::{is_key_down, KeyCode},
    math::{Rect, Vec2},
    miniquad::window::screen_size,
    shapes::draw_rectangle,
    text::load_ttf_font,
    texture::{
        draw_texture_ex, load_texture, set_default_filter_mode, DrawTextureParams, FilterMode,
        Texture2D,
    },
    time::{get_frame_time, get_time},
    window::{clear_background, next_frame},
};
use rand::{
    distributions::{Uniform, WeightedIndex},
    rngs::SmallRng,
    Rng, SeedableRng,
};
use rand_distr::Poisson;
use std::{f32::consts::PI, future::Future, pin::Pin, time::Duration};

#[derive(Clone, Debug)]
pub struct Item {
    pub pos: Vec2,
    pub image: Texture2D,
    pub radius: f32,
}

impl Item {
    pub fn draw(&self, offset: Vec2) {
        draw_texture_ex(
            &self.image,
            self.pos.x + offset.x - self.radius,
            self.pos.y + offset.y - self.radius,
            color::WHITE,
            DrawTextureParams {
                dest_size: Some(2.0 * Vec2::new(self.radius, self.radius)),
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

pub async fn main() -> Result<(), Error> {
    set_default_filter_mode(FilterMode::Nearest);
    let player_image = load_texture("assets/mouse.png").await?;
    let items_images_and_probs = try_join_all(
        vec![("assets/cheese.png", 0.8), ("assets/apple.png", 0.2)]
            .into_iter()
            .map(|(path, prob)| load_texture(path).map_ok(move |t| (t, prob))),
    )
    .await?;
    let font = load_ttf_font("assets/default.ttf").await?;

    let mut rng = SmallRng::from_entropy();

    loop {
        let map_size = Vec2::from([40.0, 30.0]);
        let mean_items: f32 = 16.0;
        let num_items = rng.sample(Poisson::new(mean_items)?).round() as usize;

        let mut player = Player {
            base: Item {
                pos: map_size / 2.0,
                image: player_image.clone(),
                radius: 0.75,
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

        loop {
            if is_key_down(KeyCode::Escape) {
                return Ok(());
            }

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
                    if (player.pos - item.pos).length() > (player.radius + item.radius) {
                        true
                    } else {
                        player.radius += 1.0 / (mean_items * (2.0 * player.radius).sqrt());
                        false
                    }
                });

                if items.is_empty() {
                    if timeout.is_zero() {
                        break;
                    } else {
                        timeout = timeout.saturating_sub(dt);
                    }
                }
            }

            // Draw frame
            {
                let viewport = Vec2::from(screen_size());
                let scale = (viewport / map_size).min_element();

                clear_background(color::BLACK);

                {
                    let camera = Camera2D {
                        zoom: 2.0 * viewport.recip() * scale,
                        target: map_size / 2.0,
                        ..Default::default()
                    };
                    set_camera(&camera);

                    draw_rectangle(0.0, 0.0, map_size.x, map_size.y, color::DARKGRAY);

                    for item in &items {
                        item.draw(Vec2::new(0.0, 0.1 * (PI * get_time() as f32).sin()));
                    }
                    player.draw(Vec2::ZERO);

                    set_default_camera();
                }

                let text_offset = 6.0;
                Text::new("Собрано", scale * 0.8, Some(&font)).draw(
                    text_offset,
                    scale * 0.8,
                    TextAlign::Left,
                    color::WHITE,
                );
                Text::new(
                    format!("{}", num_items - items.len()),
                    scale * 2.0,
                    Some(&font),
                )
                .draw(text_offset, scale * 2.6, TextAlign::Left, color::WHITE);

                Text::new("Осталось", scale * 0.8, Some(&font)).draw(
                    viewport.x - text_offset,
                    scale * 0.8,
                    TextAlign::Right,
                    color::WHITE,
                );
                Text::new(format!("{}", items.len()), scale * 2.0, Some(&font)).draw(
                    viewport.x - text_offset,
                    scale * 2.6,
                    TextAlign::Right,
                    color::WHITE,
                );
            }

            next_frame().await;
        }
    }
}

pub struct Game {
    mouse: Texture2D,
    cheese: Texture2D,
}

impl Game {
    pub async fn new() -> Result<Self, Error> {
        set_default_filter_mode(FilterMode::Nearest);
        Ok(Self {
            mouse: load_texture("assets/mouse.png").await?,
            cheese: load_texture("assets/cheese.png").await?,
        })
    }
}

impl crate::Game for Game {
    fn name(&self) -> String {
        "Мышь и сыр".to_owned()
    }

    fn draw_preview(&self, rect: Rect) {
        draw_texture_ex(
            &self.mouse,
            rect.x,
            rect.y,
            color::WHITE,
            DrawTextureParams {
                dest_size: Some(rect.size()),
                ..Default::default()
            },
        );
        draw_texture_ex(
            &self.cheese,
            rect.x,
            rect.y + rect.h / 2.0,
            color::WHITE,
            DrawTextureParams {
                dest_size: Some(rect.size() / 2.0),
                ..Default::default()
            },
        );
    }

    fn launch(&self) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        Box::pin(main())
    }
}
