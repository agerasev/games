//! Fixed-step lunar flight in metres and seconds, with Y pointing up.
//! RK4 integrates the craft; Euler integrates cosmetic exhaust and dust.
//! Terrain contact and landing rules are deliberately separate from the solver.
use phy::{Euler, Rk4, Solver, System, Var, Visitor};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use wgame::glam::{Mat2, Vec2};

pub const WIDTH: f32 = 120.0;
pub const HEIGHT: f32 = 90.0;
pub const GRAVITY: f32 = 1.62;
pub const STEP: f32 = 1.0 / 120.0;
pub const FOOT_DROP: f32 = 2.4;
pub const MAX_VERTICAL: f32 = 3.0;
pub const MAX_HORIZONTAL: f32 = 1.5;
pub const MAX_TILT: f32 = 12.0_f32.to_radians();
pub const MAX_SPIN: f32 = 0.45;
const BURN: f32 = 4.0;
const MAX_PARTICLES: usize = 400;

pub const HULL: [Vec2; 7] = [
    Vec2::new(-2.5, -FOOT_DROP),
    Vec2::new(-1.3, -0.5),
    Vec2::new(-1.2, 1.5),
    Vec2::new(0.0, 2.1),
    Vec2::new(1.2, 1.5),
    Vec2::new(1.3, -0.5),
    Vec2::new(2.5, -FOOT_DROP),
];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Control {
    pub thrust: bool,
    /// Positive rotates counterclockwise (left), negative rotates right.
    pub turn: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crash {
    Terrain,
    Speed,
    Tilt,
    Bounds,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Flying,
    Landed,
    Crashed(Crash),
}

#[derive(Clone, Copy)]
pub struct Pad {
    pub left: f32,
    pub right: f32,
    pub height: f32,
}
impl Pad {
    pub fn center(self) -> f32 {
        (self.left + self.right) * 0.5
    }
}
pub struct Terrain {
    pub points: Vec<Vec2>,
    pub pad: Pad,
}
impl Terrain {
    fn new(site: usize) -> Self {
        let (center, width) = [(60.0, 30.0), (86.0, 18.0), (42.0, 10.0)][site];
        let pad = Pad {
            left: center - width * 0.5,
            right: center + width * 0.5,
            height: 6.0,
        };
        let mut points: Vec<_> = (0..=10)
            .filter_map(|i| {
                let x = i as f32 * 12.0;
                (x < pad.left - 5.0 || x > pad.right + 5.0)
                    .then(|| Vec2::new(x, 10.0 + 5.0 * (i as f32 * 1.7 + site as f32).sin()))
            })
            .collect();
        points.extend([
            Vec2::new(pad.left - 5.0, 9.0),
            Vec2::new(pad.left, pad.height),
            Vec2::new(pad.right, pad.height),
            Vec2::new(pad.right + 5.0, 9.0),
        ]);
        points.sort_by(|a, b| a.x.total_cmp(&b.x));
        Self { points, pad }
    }
    pub fn height(&self, x: f32) -> f32 {
        for p in self.points.windows(2) {
            if x <= p[1].x {
                return p[0].y
                    + (p[1].y - p[0].y) * ((x - p[0].x) / (p[1].x - p[0].x)).clamp(0.0, 1.0);
            }
        }
        self.points.last().unwrap().y
    }
    fn contact(&self, hull: &[Vec2; 7]) -> bool {
        if hull.iter().any(|p| p.y <= self.height(p.x)) {
            return true;
        }
        // Also check terrain peaks between hull vertices, so a ridge cannot
        // pass through the underside while both feet remain above the ground.
        for (a, b) in hull
            .iter()
            .zip(hull.iter().cycle().skip(1))
            .take(hull.len())
        {
            if (a.x - b.x).abs() < 1e-6 {
                continue;
            }
            for p in &self.points {
                if p.x >= a.x.min(b.x) && p.x <= a.x.max(b.x) {
                    let y = a.y + (b.y - a.y) * ((p.x - a.x) / (b.x - a.x));
                    if p.y >= y {
                        return true;
                    }
                }
            }
        }
        false
    }
}

pub struct Craft {
    pub pos: Var<Vec2, Rk4>,
    pub vel: Var<Vec2, Rk4>,
    pub angle: Var<f32, Rk4>,
    pub spin: Var<f32, Rk4>,
    power: f32,
    steering: f32,
}
impl Craft {
    pub fn point(&self, local: Vec2) -> Vec2 {
        *self.pos + Mat2::from_angle(*self.angle) * local
    }
    pub fn hull(&self) -> [Vec2; 7] {
        HULL.map(|p| self.point(p))
    }
}
impl System<Rk4> for Craft {
    fn compute_derivs(&mut self, _: &<Rk4 as Solver>::Context) {
        self.pos.deriv = *self.vel;
        self.vel.deriv =
            Vec2::new(0.0, -GRAVITY) + Mat2::from_angle(*self.angle) * Vec2::Y * (5.4 * self.power);
        self.angle.deriv = *self.spin;
        // Reaction-control stabilization, not atmospheric drag.
        self.spin.deriv = self.steering * 2.2 - *self.spin * 3.0;
    }
    fn visit_vars<V: Visitor<Rk4>>(&mut self, v: &mut V) {
        v.apply(&mut self.pos);
        v.apply(&mut self.vel);
        v.apply(&mut self.angle);
        v.apply(&mut self.spin);
    }
}

pub struct Particle {
    pub pos: Var<Vec2, Euler>,
    vel: Var<Vec2, Euler>,
    pub life: f32,
    pub duration: f32,
    pub dust: bool,
}
impl System<Euler> for Particle {
    fn compute_derivs(&mut self, _: &<Euler as Solver>::Context) {
        self.pos.deriv = *self.vel;
        self.vel.deriv = Vec2::new(0.0, -GRAVITY);
    }
    fn visit_vars<V: Visitor<Euler>>(&mut self, v: &mut V) {
        v.apply(&mut self.pos);
        v.apply(&mut self.vel);
    }
}

pub struct Flight {
    pub craft: Craft,
    pub terrain: Terrain,
    pub particles: Vec<Particle>,
    pub status: Status,
    pub fuel: f32,
    pub elapsed: f32,
    pub power: f32,
    pub site: usize,
    rng: SmallRng,
    accumulator: f64,
    emission: f32,
}
impl Flight {
    pub const SITES: [&'static str; 3] = ["Море Спокойствия", "Край кратера", "Узкий уступ"];
    pub fn new(site: usize) -> Self {
        assert!(site < Self::SITES.len());
        let (x, y, vx) = [(60.0, 64.0, 0.0), (28.0, 70.0, 1.5), (92.0, 74.0, -2.0)][site];
        Self {
            craft: Craft {
                pos: Var::new(Vec2::new(x, y)),
                vel: Var::new(Vec2::new(vx, 0.0)),
                angle: Var::new(0.0),
                spin: Var::new(0.0),
                power: 0.0,
                steering: 0.0,
            },
            terrain: Terrain::new(site),
            particles: Vec::new(),
            status: Status::Flying,
            fuel: 100.0,
            elapsed: 0.0,
            power: 0.0,
            site,
            rng: SmallRng::seed_from_u64(0x4d4f4f4e + site as u64),
            accumulator: 0.0,
            emission: 0.0,
        }
    }
    pub fn altitude(&self) -> f32 {
        self.craft
            .hull()
            .iter()
            .map(|p| p.y - self.terrain.height(p.x))
            .fold(f32::INFINITY, f32::min)
            .max(0.0)
    }
    pub fn advance(&mut self, control: Control, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        self.accumulator += f64::from(dt.min(0.25));
        while self.accumulator + 1e-9 >= f64::from(STEP) {
            self.accumulator = (self.accumulator - f64::from(STEP)).max(0.0);
            self.tick(control);
        }
    }
    fn tick(&mut self, control: Control) {
        if self.status == Status::Flying {
            let thrust = f32::from(control.thrust);
            let turn = if control.turn.is_finite() {
                control.turn.clamp(-1.0, 1.0)
            } else {
                0.0
            };
            let needed = STEP * (BURN * thrust + 0.3 * turn.abs());
            let fraction = if needed > 0.0 {
                (self.fuel / needed).min(1.0)
            } else {
                0.0
            };
            self.fuel = (self.fuel - needed).max(0.0);
            self.power = thrust * fraction;
            self.craft.power = self.power;
            self.craft.steering = turn * fraction;
            Rk4.solve_step(&mut self.craft, STEP);
            *self.craft.angle = (*self.craft.angle + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            self.elapsed += STEP;
            let hull = self.craft.hull();
            if self.terrain.contact(&hull) {
                let pad = self.terrain.pad;
                let on_pad = [hull[0], hull[6]]
                    .iter()
                    .all(|p| p.x >= pad.left && p.x <= pad.right);
                self.status = if !on_pad {
                    Status::Crashed(Crash::Terrain)
                } else if self.craft.vel.x.abs() > MAX_HORIZONTAL
                    || self.craft.vel.y.abs() > MAX_VERTICAL
                {
                    Status::Crashed(Crash::Speed)
                } else if self.craft.angle.abs() > MAX_TILT || self.craft.spin.abs() > MAX_SPIN {
                    Status::Crashed(Crash::Tilt)
                } else {
                    Status::Landed
                };
                if self.status == Status::Landed {
                    self.craft.pos.y = pad.height + FOOT_DROP;
                    *self.craft.angle = 0.0;
                } else {
                    self.burst();
                }
                *self.craft.vel = Vec2::ZERO;
                *self.craft.spin = 0.0;
            } else if self.craft.pos.x < 0.0
                || self.craft.pos.x > WIDTH
                || self.craft.pos.y > HEIGHT
            {
                self.status = Status::Crashed(Crash::Bounds);
            }
            if self.status != Status::Flying {
                self.power = 0.0;
            }
        }
        self.emission += self.power * STEP * 100.0;
        while self.emission >= 1.0 {
            self.emission -= 1.0;
            let nozzle = self.craft.point(Vec2::new(0.0, -1.3));
            let exhaust = Mat2::from_angle(*self.craft.angle)
                * Vec2::new(
                    self.rng.random_range(-3.0..3.0),
                    self.rng.random_range(-20.0..-12.0),
                );
            self.particle(nozzle, *self.craft.vel + exhaust, 1.6, false);
        }
        for p in &mut self.particles {
            Euler.solve_step(p, STEP);
            p.life -= STEP;
            let ground = self.terrain.height(p.pos.x);
            if p.pos.y <= ground {
                p.pos.y = ground + 0.05;
                p.vel.y = p.vel.y.abs() * 0.2;
                p.vel.x *= 0.8;
                if !p.dust {
                    p.life = p.life.min(0.8);
                    p.dust = true;
                }
            }
        }
        self.particles.retain(|p| p.life > 0.0);
    }
    fn particle(&mut self, pos: Vec2, vel: Vec2, life: f32, dust: bool) {
        if self.particles.len() < MAX_PARTICLES {
            self.particles.push(Particle {
                pos: Var::new(pos),
                vel: Var::new(vel),
                life,
                duration: life,
                dust,
            });
        }
    }
    fn burst(&mut self) {
        for _ in 0..65 {
            let velocity = Vec2::new(
                self.rng.random_range(-12.0..12.0),
                self.rng.random_range(1.0..15.0),
            );
            self.particle(*self.craft.pos, velocity, 2.0, false);
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
