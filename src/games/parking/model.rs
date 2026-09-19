//! Top-down parking in metres, with Y up and heading zero pointing right.
//! A kinematic bicycle model gives rearward steering its natural reversed turn.
//! RK4 integrates the pose at 120 Hz; oriented rectangles handle contact.
use geom2::{HalfPlane, Line, LineSegment, Polygon};
use phy::{Rk4, Solver, System, Var, Visitor};
use wgame::glam::{Mat2, Vec2};

pub const WORLD: Vec2 = Vec2::new(40.0, 28.0);
pub const CAR_HALF: Vec2 = Vec2::new(2.2, 0.9);
pub const STEP: f32 = 1.0 / 120.0;
pub const MAX_STEER: f32 = 0.6;
pub const WHEELBASE: f32 = 2.6;
pub const HOLD_TIME: f32 = 1.0;
pub const PARK_SPEED: f32 = 0.12;
pub const PARK_ANGLE: f32 = 8.0_f32.to_radians();

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Control {
    /// Forward (+1) or reverse (-1); opposite drive first brakes to a stop.
    pub drive: f32,
    pub steer: f32,
    pub brake: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crash {
    Curb,
    ParkedCar,
    Traffic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Driving,
    Parked,
    Crashed(Crash),
}

#[derive(Clone, Copy, Debug)]
pub struct Box2 {
    pub center: Vec2,
    /// Half length along heading, then half width across it.
    pub half: Vec2,
    pub angle: f32,
}
impl Box2 {
    pub fn car(center: Vec2, angle: f32) -> Self {
        Self {
            center,
            half: CAR_HALF,
            angle,
        }
    }
    pub fn point(self, local: Vec2) -> Vec2 {
        self.center + Mat2::from_angle(self.angle) * local
    }
    pub fn corners(self) -> [Vec2; 4] {
        [
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::ONE,
            Vec2::new(-1.0, 1.0),
        ]
        .map(|p| self.point(p * self.half))
    }
    fn planes(self) -> [HalfPlane; 4] {
        let polygon = Polygon::new(self.corners());
        let mut edges = polygon.edges();
        std::array::from_fn(|_| {
            let LineSegment(a, b) = edges.next().unwrap();
            HalfPlane::from_edge(Line(a, b))
        })
    }
    pub fn contains(self, other: Self) -> bool {
        let points = other.corners();
        self.planes()
            .iter()
            .all(|plane| points.iter().all(|&p| plane.distance(p) <= 1e-4))
    }
    pub fn overlaps(self, other: Self) -> bool {
        // A convex pair is separated if either polygon has an edge with every
        // opposite vertex outside. geom2 supplies edges and signed distances.
        let separated = |body: Self, points: [Vec2; 4]| {
            body.planes()
                .iter()
                .any(|plane| points.iter().all(|&p| plane.distance(p) > 0.0))
        };
        !separated(self, other.corners()) && !separated(other, self.corners())
    }
}

pub struct Car {
    pub pos: Var<Vec2, Rk4>,
    pub angle: Var<f32, Rk4>,
    pub speed: f32,
    pub steering: f32,
}
impl Car {
    pub fn new(center: Vec2, angle: f32) -> Self {
        Self {
            pos: Var::new(center),
            angle: Var::new(angle),
            speed: 0.0,
            steering: 0.0,
        }
    }
    pub fn body(&self) -> Box2 {
        Box2::car(*self.pos, *self.angle)
    }
    fn step(&mut self, input: Control) {
        let finite = |x: f32| {
            if x.is_finite() {
                x.clamp(-1.0, 1.0)
            } else {
                0.0
            }
        };
        let steer = finite(input.steer) * MAX_STEER;
        self.steering = approach(self.steering, steer, 2.2 * STEP);
        let drive = finite(input.drive);
        if input.brake {
            self.speed = approach(self.speed, 0.0, 8.0 * STEP);
        } else if drive == 0.0 {
            self.speed = approach(self.speed, 0.0, 0.8 * STEP);
        } else if self.speed * drive < 0.0 {
            self.speed = approach(self.speed, 0.0, 6.0 * STEP);
        } else {
            let limit = if drive > 0.0 { 5.5 } else { 2.8 };
            self.speed = approach(self.speed, drive * limit, 3.0 * STEP);
        }
        Rk4.solve_step(self, STEP);
        *self.angle = (*self.angle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
    }
}
impl System<Rk4> for Car {
    fn compute_derivs(&mut self, _: &<Rk4 as Solver>::Context) {
        let beta = (0.5 * self.steering.tan()).atan();
        self.pos.deriv = Vec2::from_angle(*self.angle + beta) * self.speed;
        self.angle.deriv = self.speed * beta.cos() * self.steering.tan() / WHEELBASE;
    }
    fn visit_vars<V: Visitor<Rk4>>(&mut self, v: &mut V) {
        v.apply(&mut self.pos);
        v.apply(&mut self.angle);
    }
}
fn approach(value: f32, target: f32, amount: f32) -> f32 {
    value + (target - value).clamp(-amount, amount)
}

pub struct Level {
    pub target: Box2,
    pub parked: Vec<Box2>,
    pub start: Box2,
    pub traffic: Vec<Traffic>,
}
pub const NAMES: [&str; 4] = [
    "Свободное место",
    "Между машинами",
    "Вдоль тротуара",
    "Час пик",
];
impl Level {
    pub fn new(index: usize) -> Self {
        use std::f32::consts::{FRAC_PI_2, PI};
        match index {
            0 | 1 => {
                let (x, width, spacing) = if index == 0 {
                    (20.0, 2.5, 6.0)
                } else {
                    (25.0, 1.4, 3.2)
                };
                Self {
                    target: Box2 {
                        center: Vec2::new(x, 22.0),
                        half: Vec2::new(3.2, width),
                        angle: FRAC_PI_2,
                    },
                    parked: [-spacing, spacing]
                        .map(|dx| Box2::car(Vec2::new(x + dx, 22.0), FRAC_PI_2))
                        .to_vec(),
                    start: if index == 0 {
                        Box2::car(Vec2::new(20.0, 7.0), FRAC_PI_2)
                    } else {
                        Box2::car(Vec2::new(8.0, 10.0), 0.0)
                    },
                    traffic: Vec::new(),
                }
            }
            2 | 3 => Self {
                target: Box2 {
                    center: Vec2::new(23.0, 22.0),
                    half: Vec2::new(
                        if index == 2 { 3.3 } else { 2.9 },
                        if index == 2 { 1.4 } else { 1.25 },
                    ),
                    angle: 0.0,
                },
                parked: [16.3, 29.7]
                    .map(|x| Box2::car(Vec2::new(x, 22.0), 0.0))
                    .to_vec(),
                start: Box2::car(Vec2::new(8.0, if index == 2 { 15.0 } else { 12.0 }), 0.0),
                traffic: if index == 3 {
                    vec![
                        Traffic {
                            body: Box2::car(Vec2::new(26.0, 12.0), 0.0),
                            velocity: 3.0,
                        },
                        Traffic {
                            body: Box2::car(Vec2::new(34.0, 17.0), PI),
                            velocity: -3.5,
                        },
                    ]
                } else {
                    Vec::new()
                },
            },
            _ => panic!("invalid parking level"),
        }
    }
}
#[derive(Clone, Copy)]
pub struct Traffic {
    pub body: Box2,
    pub velocity: f32,
}
impl Traffic {
    fn step(&mut self) {
        // Wrap only beyond the visible world, with no swept collision across it.
        self.body.center.x =
            (self.body.center.x + self.velocity * STEP + 5.0).rem_euclid(WORLD.x + 10.0) - 5.0;
    }
}

pub struct Round {
    pub car: Car,
    pub level: Level,
    pub index: usize,
    pub status: Status,
    pub hold: f32,
    accumulator: f64,
}
impl Round {
    pub fn new(index: usize) -> Self {
        let level = Level::new(index);
        Self {
            car: Car::new(level.start.center, level.start.angle),
            level,
            index,
            status: Status::Driving,
            hold: 0.0,
            accumulator: 0.0,
        }
    }
    pub fn aligned(&self) -> bool {
        // Either forward or reverse parking is accepted.
        (*self.car.angle - self.level.target.angle).sin().abs() <= PARK_ANGLE.sin()
    }
    pub fn inside(&self) -> bool {
        self.level.target.contains(self.car.body())
    }
    pub fn needs_update(&self, control: Control) -> bool {
        self.status == Status::Driving
            && (self.car.speed != 0.0
                || control.drive != 0.0
                || (self.car.steering - control.steer * MAX_STEER).abs() > 1e-5
                || !self.level.traffic.is_empty()
                || self.hold > 0.0
                || (self.inside() && self.aligned()))
    }
    pub fn advance(&mut self, control: Control, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 || self.status != Status::Driving {
            return;
        }
        self.accumulator += f64::from(dt.min(0.25));
        while self.accumulator + 1e-9 >= f64::from(STEP) && self.status == Status::Driving {
            self.accumulator = (self.accumulator - f64::from(STEP)).max(0.0);
            self.tick(control);
        }
    }
    fn tick(&mut self, control: Control) {
        let previous = self.car.body();
        self.car.step(control);
        for traffic in &mut self.level.traffic {
            traffic.step();
        }
        let body = self.car.body();
        let street = Box2 {
            center: WORLD * 0.5,
            half: WORLD * 0.5 - Vec2::ONE,
            angle: 0.0,
        };
        let crash = if !street.contains(body) {
            Some(Crash::Curb)
        } else if self.level.parked.iter().any(|p| p.overlaps(body)) {
            Some(Crash::ParkedCar)
        } else if self.level.traffic.iter().any(|p| p.body.overlaps(body)) {
            Some(Crash::Traffic)
        } else {
            None
        };
        if let Some(reason) = crash {
            self.status = Status::Crashed(reason);
            *self.car.pos = previous.center;
            *self.car.angle = previous.angle;
            self.car.speed = 0.0;
            self.hold = 0.0;
        } else if self.inside() && self.aligned() && self.car.speed.abs() <= PARK_SPEED {
            self.hold += STEP;
            if self.hold + 1e-5 >= HOLD_TIME {
                self.status = Status::Parked;
                self.car.speed = 0.0;
                self.hold = HOLD_TIME;
            }
        } else {
            self.hold = 0.0;
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
