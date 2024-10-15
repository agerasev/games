use crate::physics::{Solver, System, Var, Visitor};
use anyhow::Error;
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    color::{self, hsl_to_rgb, Color},
    input::{
        is_key_down, is_mouse_button_down, is_mouse_button_pressed, is_mouse_button_released,
        mouse_position, KeyCode, MouseButton,
    },
    math::{Rect, Vec2},
    miniquad::window::screen_size,
    shapes::draw_circle,
    time::get_frame_time,
    window::{clear_background, next_frame},
};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rand_distr::Uniform;
use std::{future::Future, pin::Pin, time::Duration};

/// Gravitational acceleration
const G: Vec2 = Vec2::new(0.0, 4.0);
/// Elasticity of balls
const ELA: f32 = 1000.0;
/// Amortization factor.
const AMO: f32 = 4.0;

struct Ball {
    radius: f32,
    mass: f32,
    pos: Var<Vec2>,
    vel: Var<Vec2>,
    color: Color,
}

impl Ball {
    fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, self.radius, self.color);
    }

    fn bounce(&mut self, mut dev: Vec2, other_vel: Vec2) {
        dev = dev.clamp_length_max(self.radius);
        let norm = dev.normalize_or_zero();
        let dev = dev.length();
        let area = dev * dev;
        let ela_f = ELA * area * norm;
        let amo_f = -AMO * (*self.vel - other_vel) * dev;
        self.vel.add_derivative((ela_f + amo_f) / self.mass);
    }
}

struct ToyBox {
    size: Vec2,
    items: Vec<Ball>,
    drag: Option<(usize, Vec2)>,
}

impl System for ToyBox {
    fn compute_derivatives(&mut self) {
        for item in &mut self.items {
            item.pos.add_derivative(*item.vel);

            item.vel.add_derivative(G);

            let pos = *item.pos;
            let eff_size = self.size - Vec2::new(item.radius, item.radius);
            let dev = (pos.abs() - eff_size).max(Vec2::ZERO) * pos.signum();
            item.bounce(-dev, Vec2::ZERO);
        }

        for i in 1..self.items.len() {
            let (left, other_items) = self.items.split_at_mut(i);
            let item = left.last_mut().unwrap();
            for other in other_items {
                let rel_pos = *other.pos - *item.pos;
                let dist = rel_pos.length() - (item.radius + other.radius);
                if dist < 0.0 {
                    let dir = rel_pos.normalize_or_zero();
                    let dev = dist * dir;
                    item.bounce(dev, *other.vel);
                    other.bounce(-dev, *item.vel);
                }
            }
        }

        if let Some((i, drag_pos)) = self.drag {
            let item = &mut self.items[i];
            item.bounce(drag_pos - *item.pos, Vec2::ZERO);
        }
    }

    fn visit_parameters<V: Visitor>(&mut self, visitor: &mut V) {
        for item in &mut self.items {
            visitor.apply(&mut item.pos);
            visitor.apply(&mut item.vel);
        }
    }
}

impl ToyBox {
    fn drag_acquire(&mut self, pos: Vec2) {
        self.drag = self.items.iter().enumerate().find_map(|(i, item)| {
            if (pos - *item.pos).length() < item.radius {
                Some((i, pos))
            } else {
                None
            }
        })
    }
    fn drag_move(&mut self, pos: Vec2) {
        if let Some((_, drag_pos)) = &mut self.drag {
            *drag_pos = pos;
        }
    }
    fn drag_release(&mut self) {
        self.drag = None;
    }
}

impl ToyBox {
    fn new(n_items: usize, mut rng: impl Rng) -> Self {
        let size = Vec2::new(1.0, 1.0);
        let mut balls = Vec::new();
        for _ in 0..n_items {
            let radius = rng.sample(Uniform::new(0.1, 0.3));
            let eff_size = size - Vec2::new(radius, radius);
            balls.push(Ball {
                radius,
                mass: radius * radius,
                pos: Var::new(Vec2::new(
                    rng.sample(Uniform::new_inclusive(-eff_size.x, eff_size.x)),
                    rng.sample(Uniform::new_inclusive(-eff_size.y, eff_size.y)),
                )),
                vel: Var::new(Vec2::ZERO),
                color: hsl_to_rgb(rng.sample(Uniform::new(0.0, 1.0)), 1.0, 0.5),
            });
        }
        Self {
            size,
            items: balls,
            drag: None,
        }
    }

    fn draw(&self) {
        for item in &self.items {
            item.draw();
        }
    }
}

pub async fn main() -> Result<(), Error> {
    let mut rng = SmallRng::from_entropy();

    let mut toy_box = ToyBox::new(8, &mut rng);
    let scale = 640.0;

    while !is_key_down(KeyCode::Escape) {
        let viewport = Vec2::from(screen_size());
        toy_box.size = viewport / scale;

        let camera = Camera2D {
            zoom: viewport.recip() * scale,
            ..Default::default()
        };

        {
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
            let dt = Duration::from_secs_f32(get_frame_time().min(0.1));
            Solver.solve_step(&mut toy_box, dt.as_secs_f32());
        }

        {
            clear_background(color::BLACK);

            set_camera(&camera);

            toy_box.draw();

            set_default_camera();
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
