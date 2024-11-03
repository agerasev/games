mod physics;

use crate::{
    algebra::Rot2,
    numerical::{Solver, Var},
    text::{draw_text_aligned, TextAlign},
    texture::noisy_texture,
};
use anyhow::Error;
use derive_more::derive::{Deref, DerefMut};
use glam::{Vec2, Vec3};
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    color::{self, hsl_to_rgb, Color},
    input::{
        is_key_down, is_key_pressed, is_mouse_button_down, is_mouse_button_pressed,
        is_mouse_button_released, mouse_position, KeyCode, MouseButton,
    },
    math::Rect,
    miniquad::window::screen_size,
    shapes::{draw_circle, draw_rectangle_lines_ex, DrawRectangleParams},
    text::load_ttf_font,
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
    time::get_frame_time,
    window::{clear_background, next_frame},
};
use physics::{Body, Shape};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rand_distr::{Standard, Uniform};
use std::{future::Future, pin::Pin, time::Duration};

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct Item {
    #[deref]
    #[deref_mut]
    pub body: Body,
    pub shape: Shape,

    pub texture: Texture2D,
    pub color: Color,
}

impl Item {
    fn draw(&self) {
        let size = match &self.shape {
            Shape::Circle { radius } => Vec2::splat(*radius),
            Shape::Rectangle { size } => *size,
        };
        draw_texture_ex(
            &self.texture,
            self.pos.x - size.x,
            self.pos.y - size.y,
            self.color,
            DrawTextureParams {
                dest_size: Some(2.0 * size),
                rotation: self.rot.angle(),
                ..Default::default()
            },
        );
        if let Shape::Rectangle { .. } = &self.shape {
            draw_rectangle_lines_ex(
                self.pos.x,
                self.pos.y,
                2.0 * size.x,
                2.0 * size.y,
                size.min_element() / 20.0,
                DrawRectangleParams {
                    offset: Vec2::new(0.5, 0.5),
                    rotation: self.rot.angle(),
                    color: color::BLACK,
                },
            );
        }
    }
}

struct World {
    /// Half of world sides
    size: Vec2,
    items: Vec<Item>,
    drag: Option<(usize, Vec2, Vec2)>,
}

impl World {
    fn new(size: Vec2) -> Self {
        Self {
            size,
            items: Vec::new(),
            drag: None,
        }
    }

    fn drag_acquire(&mut self, pos: Vec2) {
        self.drag = self.items.iter().enumerate().find_map(|(i, item)| {
            let rel_pos = pos - *item.pos;
            if rel_pos.length() < item.shape.radius() {
                let rpos = item.rot.inverse().transform(rel_pos);
                Some((i, pos, rpos))
            } else {
                None
            }
        })
    }
    fn drag_move(&mut self, pos: Vec2) {
        if let Some((_, target, ..)) = &mut self.drag {
            *target = pos;
        }
    }
    fn drag_release(&mut self) {
        self.drag = None;
    }

    fn n_items(&self) -> usize {
        self.items.len()
    }
    fn remove_item(&mut self, i: usize) -> Item {
        self.drag = None;
        self.items.remove(i)
    }
    fn insert_item(&mut self, item: Item) {
        self.items.push(item);
    }

    fn resize(&mut self, size: Vec2) {
        self.size = size;
    }
    fn draw(&self) {
        for item in &self.items {
            item.draw();
        }
    }
}

fn sample_item(mut rng: impl Rng, box_size: Vec2, textures: &TextureStorage) -> Item {
    let radius: f32 = rng.sample(Uniform::new(0.1, 0.3));
    let mass = physics::MASF * radius;
    let eff_size = (box_size - Vec2::splat(radius)).max(Vec2::ZERO);
    let shape = if rng.sample(Standard) {
        Shape::Circle { radius }
    } else {
        Shape::Rectangle {
            size: Vec2::splat(radius),
        }
    };
    Item {
        body: Body {
            mass,
            pos: Var::new(Vec2::new(
                rng.sample(Uniform::new_inclusive(-eff_size.x, eff_size.x)),
                rng.sample(Uniform::new_inclusive(-eff_size.y, eff_size.y)),
            )),
            vel: Var::default(),
            inm: physics::INMF * mass * radius,
            rot: Var::new(Rot2::default()),
            asp: Var::default(),
        },
        color: hsl_to_rgb(rng.sample(Uniform::new(0.0, 1.0)), 1.0, 0.75),
        texture: match &shape {
            Shape::Circle { .. } => textures.ball.clone(),
            Shape::Rectangle { .. } => textures.noise.clone(),
        },
        shape,
    }
}

struct TextureStorage {
    ball: Texture2D,
    noise: Texture2D,
}

pub async fn main() -> Result<(), Error> {
    let mut rng = SmallRng::from_entropy();

    let font = load_ttf_font("default.ttf").await?;

    let textures = TextureStorage {
        ball: load_texture("ball.png").await?,
        noise: noisy_texture(&mut rng, 32, 32, Vec3::splat(0.75), Vec3::splat(0.25)),
    };

    let scale = 640.0;
    let mut viewport = Vec2::from(screen_size());

    let mut toy_box = World::new(viewport / scale);

    for _ in 0..8 {
        toy_box.insert_item(sample_item(&mut rng, toy_box.size, &textures));
    }

    while !is_key_down(KeyCode::Escape) {
        viewport = Vec2::from(screen_size());
        toy_box.resize(viewport / scale);

        let camera = Camera2D {
            zoom: viewport.recip() * scale,
            ..Default::default()
        };

        {
            if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
                toy_box.insert_item(sample_item(&mut rng, toy_box.size, &textures));
            }
            if toy_box.n_items() != 0
                && (is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract))
            {
                toy_box.remove_item(rng.sample(Uniform::new(0, toy_box.n_items())));
            }

            let mouse_pos = camera.screen_to_world(Vec2::from(mouse_position()));
            if is_mouse_button_pressed(MouseButton::Left) {
                toy_box.drag_acquire(mouse_pos);
            }
            if is_mouse_button_released(MouseButton::Left) {
                toy_box.drag_release();
            }
            if is_mouse_button_down(MouseButton::Left) {
                toy_box.drag_move(mouse_pos);
            } else {
                toy_box.drag_release();
            }
        }

        {
            let dt = Duration::from_secs_f32(get_frame_time().min(0.04));
            Solver.solve_step(&mut toy_box, dt.as_secs_f32());
        }

        {
            clear_background(color::GRAY);

            set_camera(&camera);

            toy_box.draw();

            set_default_camera();

            draw_text_aligned(
                &format!("{}", toy_box.n_items()),
                viewport.x - 10.0,
                40.0,
                TextAlign::Right,
                Some(&font),
                40.0,
                color::WHITE,
            );
        }

        next_frame().await
    }

    Ok(())
}

pub struct Game {}

impl Game {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {})
    }
}

impl crate::Game for Game {
    fn name(&self) -> String {
        "Прыгучие мячи".to_owned()
    }

    fn draw_preview(&self, rect: Rect) {
        let size = rect.size();

        let rad = 0.5 * rect.size().min_element();
        draw_circle(
            rect.x + size.x - rad,
            rect.y + size.y - rad,
            rad,
            color::BLUE,
        );

        let rad = 0.25 * rect.size().min_element();
        draw_circle(rect.x + rad, rect.y + size.y - rad, rad, color::GREEN);
    }

    fn launch(&self) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        Box::pin(main())
    }
}
