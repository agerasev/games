use crate::{
    algebra::Rot2,
    numerical::{Solver, System, Var, Visitor},
    physics::{angular_to_linear2, torque2},
    text::{draw_text_aligned, TextAlign},
};
use anyhow::Error;
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    color::{self, hsl_to_rgb, Color},
    input::{
        is_key_down, is_key_pressed, is_mouse_button_down, is_mouse_button_pressed,
        is_mouse_button_released, mouse_delta_position, mouse_position, KeyCode, MouseButton,
    },
    math::{Rect, Vec2},
    miniquad::window::screen_size,
    shapes::draw_circle,
    text::load_ttf_font,
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
    time::get_frame_time,
    window::{clear_background, next_frame},
};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rand_distr::Uniform;
use std::{future::Future, pin::Pin, time::Duration};

/// Gravitational acceleration
const G: Vec2 = Vec2::new(0.0, 4.0);
/// Mass factor
const MASF: f32 = 1.0;
/// Moment of inertia factor
const INMF: f32 = 0.2;

/// Elasticity of balls
const ELAST: f32 = 100.0;

/// Damping factor.
const DAMP: f32 = 0.2;
/// Liquid friction.
const FRICT: f32 = 1.0;

/// Attraction damping.
const ADAMP: f32 = 4.0;

struct Ball {
    radius: f32,

    mass: f32,
    pos: Var<Vec2>,
    vel: Var<Vec2>,

    /// Moment of inertia
    inm: f32,
    /// Rotation.
    rot: Var<Rot2>,
    /// Angular speed.
    asp: Var<f32>,

    color: Color,
}

impl Ball {
    fn draw(&self, texture: &Texture2D) {
        draw_texture_ex(
            texture,
            self.pos.x - self.radius,
            self.pos.y - self.radius,
            self.color,
            DrawTextureParams {
                dest_size: Some(2.0 * Vec2::new(self.radius, self.radius)),
                rotation: self.rot.angle(),
                ..Default::default()
            },
        );
    }

    fn vel_at(&self, p: Vec2) -> Vec2 {
        *self.vel + angular_to_linear2(*self.asp, p - *self.pos)
    }

    fn push(&mut self, def: Vec2) {
        self.vel.add_deriv(ELAST * def / self.mass);
    }

    /// Influence ball by directed deformation `def` at point of contact `pos` moving with velocity `vel`.
    fn contact(&mut self, def: Vec2, pos: Vec2, vel: Vec2) {
        let rel_pos = pos - *self.pos;

        let norm = def.normalize_or_zero();
        // Elastic force (normal reaction)
        let elast_f = ELAST * def;

        // Damping force (parallel to `norm`)
        let damp_f = -DAMP * (*self.vel - vel).dot(norm) * elast_f;
        // Liquid friction force (perpendicular to `norm`)
        let frict_f = -FRICT
            * (*self.vel + angular_to_linear2(*self.asp, rel_pos) - vel).dot(norm.perp())
            * elast_f.perp();
        // Total force
        let total_f = elast_f + damp_f + frict_f;

        self.vel.add_deriv(total_f / self.mass);
        self.asp.add_deriv(torque2(rel_pos, total_f) / self.inm);
    }

    fn attract(&mut self, pos: Vec2) {
        let rel_pos = pos - *self.pos;

        // Elastic attraction
        let elast_f = ELAST * rel_pos;

        // Constant damping
        let total_f = elast_f - ADAMP * self.radius * self.vel.length().sqrt() * *self.vel;

        self.vel.add_deriv(total_f / self.mass);
        self.asp.add_deriv(torque2(Vec2::ZERO, total_f) / self.inm);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Contact { vel: Vec2 },
    Attraction,
}

fn contact_wall(item: &mut Ball, offset: f32, norm: Vec2) {
    let dist = item.pos.dot(norm) + offset;
    if dist < item.radius {
        if dist > 0.0 {
            item.contact(
                norm * (item.radius - dist),
                item.pos.reject_from(norm) - offset * norm,
                Vec2::ZERO,
            );
        } else {
            item.push(norm * item.radius);
        };
    }
}

struct ToyBox {
    size: Vec2,
    items: Vec<Ball>,
    drag: Option<(usize, Vec2)>,
}

impl System for ToyBox {
    fn compute_derivs(&mut self, _dt: f32) {
        for item in &mut self.items {
            item.pos.add_deriv(*item.vel);
            item.rot.add_deriv(*item.asp);

            // Gravity
            item.vel.add_deriv(G);

            // Walls
            contact_wall(item, self.size.x, Vec2::new(1.0, 0.0));
            contact_wall(item, self.size.x, Vec2::new(-1.0, 0.0));
            contact_wall(item, self.size.y, Vec2::new(0.0, 1.0));
            contact_wall(item, self.size.y, Vec2::new(0.0, -1.0));
        }

        for i in 1..self.items.len() {
            let (left, other_items) = self.items.split_at_mut(i);
            let item = left.last_mut().unwrap();
            for other in other_items {
                let rel_pos = *other.pos - *item.pos;
                let dist = rel_pos.length();
                let sum_radius = item.radius + other.radius;
                if dist < sum_radius {
                    let dir = rel_pos.normalize_or_zero();
                    let dev = (sum_radius - dist) / 2.0;
                    let min_radius = f32::min(item.radius, other.radius);
                    if 2.0 * dev < min_radius {
                        let poc = *item.pos + dir * (item.radius - dev);
                        item.contact(-dev * dir, poc, other.vel_at(poc));
                        other.contact(dev * dir, poc, item.vel_at(poc));
                    } else {
                        item.push(-min_radius * dir);
                        other.push(min_radius * dir);
                    }
                }
            }
        }

        if let Some((i, drag_pos)) = self.drag {
            let item = &mut self.items[i];
            item.attract(drag_pos);
        }
    }

    fn visit_vars<V: Visitor>(&mut self, visitor: &mut V) {
        for item in &mut self.items {
            visitor.apply(&mut item.pos);
            visitor.apply(&mut item.vel);
            visitor.apply(&mut item.rot);
            visitor.apply(&mut item.asp);
        }
    }
}

impl ToyBox {
    fn new(size: Vec2) -> Self {
        Self {
            size,
            items: Vec::new(),
            drag: None,
        }
    }

    fn drag_acquire(&mut self, pos: Vec2) {
        self.drag = self.items.iter().enumerate().find_map(|(i, item)| {
            if (pos - *item.pos).length() < item.radius {
                Some((i, pos))
            } else {
                None
            }
        })
    }
    fn drag_move(&mut self, pos: Vec2, _vel: Vec2) {
        if let Some((_, drag_pos)) = &mut self.drag {
            *drag_pos = pos;
        }
    }
    fn drag_release(&mut self) {
        self.drag = None;
    }

    fn n_items(&self) -> usize {
        self.items.len()
    }
    fn remove_item(&mut self, i: usize) -> Ball {
        self.drag = None;
        self.items.remove(i)
    }
    fn insert_item(&mut self, item: Ball) {
        self.items.push(item);
    }

    fn resize(&mut self, size: Vec2) {
        self.size = size;
    }
    fn draw(&self, ball_tex: &Texture2D) {
        for item in &self.items {
            item.draw(ball_tex);
        }
    }
}

fn sample_item(mut rng: impl Rng, box_size: Vec2) -> Ball {
    let radius: f32 = rng.sample(Uniform::new(0.1, 0.3));
    let mass = MASF * radius;
    let eff_size = (box_size - Vec2::splat(radius)).max(Vec2::ZERO);
    Ball {
        radius,
        mass,
        pos: Var::new(Vec2::new(
            rng.sample(Uniform::new_inclusive(-eff_size.x, eff_size.x)),
            rng.sample(Uniform::new_inclusive(-eff_size.y, eff_size.y)),
        )),
        vel: Var::default(),
        inm: INMF * mass * radius,
        rot: Var::new(Rot2::default()),
        asp: Var::default(),
        color: hsl_to_rgb(rng.sample(Uniform::new(0.0, 1.0)), 1.0, 0.75),
    }
}

pub async fn main() -> Result<(), Error> {
    let ball_tex = load_texture("ball.png").await?;
    let font = load_ttf_font("default.ttf").await?;

    let mut rng = SmallRng::from_entropy();
    let scale = 640.0;
    let mut viewport = Vec2::from(screen_size());

    let mut toy_box = ToyBox::new(viewport / scale);
    for _ in 0..8 {
        toy_box.insert_item(sample_item(&mut rng, toy_box.size));
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
                toy_box.insert_item(sample_item(&mut rng, toy_box.size));
            }
            if toy_box.n_items() != 0
                && (is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract))
            {
                toy_box.remove_item(rng.sample(Uniform::new(0, toy_box.n_items())));
            }

            let mouse_pos = camera.screen_to_world(Vec2::from(mouse_position()));
            let mouse_vel = mouse_delta_position() / camera.zoom;
            if is_mouse_button_pressed(MouseButton::Left) {
                toy_box.drag_acquire(mouse_pos);
            }
            if is_mouse_button_released(MouseButton::Left) {
                toy_box.drag_release();
            }
            if is_mouse_button_down(MouseButton::Left) {
                toy_box.drag_move(mouse_pos, mouse_vel);
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

            toy_box.draw(&ball_tex);

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
