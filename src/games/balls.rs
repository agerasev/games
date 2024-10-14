use crate::solver::{Euler, SecondOrder, Solver, System, Visitor, Wrapper};
use anyhow::Error;
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    color::{self, hsl_to_rgb, Color},
    input::{is_key_down, KeyCode},
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
const G: Vec2 = Vec2::new(0.0, 0.98);
/// Elasticity of balls
const ELA: f32 = 1.0;
/// Amortization factor.
const AMO: f32 = 0.0;

struct Ball<S: Solver> {
    radius: f32,
    mass: f32,
    pos: S::Wrapper<SecondOrder<Vec2>>,
    color: Color,
}

impl<S: Solver> Ball<S> {
    fn draw(&self) {
        let pos = self.pos.p().p;
        draw_circle(pos.x, pos.y, self.radius, self.color);
    }
}

struct ToyBox<S: Solver> {
    rect: Rect,
    items: Vec<Ball<S>>,
}

impl<S: Solver> System<S> for ToyBox<S> {
    fn compute_derivatives(&mut self) {
        for item in &mut self.items {
            *item.pos.dp_mut() = G;

            let v = item.pos.p().dp;

            let d = item.pos.p().p.x - item.radius - self.rect.left();
            if d < 0.0 {
                item.pos.dp_mut().x -= (ELA * d + AMO * d.abs() * v.x) / item.mass;
            }

            let d = item.pos.p().p.x + item.radius - self.rect.right();
            if d > 0.0 {
                item.pos.dp_mut().x -= (ELA * d + AMO * d.abs() * v.x) / item.mass;
            }

            let d = item.pos.p().p.y - item.radius - self.rect.top();
            if d < 0.0 {
                item.pos.dp_mut().y -= (ELA * d + AMO * d.abs() * v.y) / item.mass;
            }

            let d = item.pos.p().p.y + item.radius - self.rect.bottom();
            if d > 0.0 {
                item.pos.dp_mut().y -= (ELA * d + AMO * d.abs() * v.y) / item.mass;
            }
        }
    }
    fn visit_parameters<V: Visitor<Solver = S>>(&mut self, visitor: &mut V) {
        for item in &mut self.items {
            visitor.apply(&mut item.pos);
        }
    }
}

impl<S: Solver> ToyBox<S> {
    fn new(n_items: usize, mut rng: impl Rng) -> Self {
        let rect = Rect::new(-1.0, -1.0, 2.0, 2.0);
        let mut balls = Vec::new();
        for _ in 0..n_items {
            let radius = rng.sample(Uniform::new(0.1, 0.2));
            balls.push(Ball {
                radius,
                mass: radius * radius,
                pos: S::Wrapper::wrap(SecondOrder {
                    p: Vec2::new(
                        rng.sample(Uniform::new_inclusive(rect.left(), rect.right())),
                        rng.sample(Uniform::new_inclusive(rect.top(), rect.bottom())),
                    ),
                    dp: Vec2::ZERO,
                }),
                color: hsl_to_rgb(rng.sample(Uniform::new(0.0, 1.0)), 1.0, 0.5),
            });
        }
        Self { rect, items: balls }
    }

    fn draw(&self) {
        for item in &self.items {
            item.draw();
        }
    }
}

pub async fn main() -> Result<(), Error> {
    let mut rng = SmallRng::from_entropy();

    let mut toy_box = ToyBox::<Euler>::new(4, &mut rng);

    while !is_key_down(KeyCode::Escape) {
        let viewport = Vec2::from(screen_size());
        let dt = Duration::from_secs_f32(get_frame_time().min(0.1));

        {
            Euler.solve(&mut toy_box, dt.as_secs_f32());
        }

        {
            clear_background(color::BLACK);

            let camera = Camera2D {
                zoom: (viewport.recip() * viewport.min_element()),
                ..Default::default()
            };
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
        "Мячи".to_owned()
    }

    fn draw_preview(&self, _rect: Rect) {}

    fn launch(&self) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        Box::pin(main())
    }
}
