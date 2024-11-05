use super::{geometry::intersect_circle_and_plane, Item, World};
use crate::{
    algebra::Rot2,
    numerical::{System, Var, Visitor},
    physics::{angular_to_linear2, torque2},
};
use macroquad::math::Vec2;

/// Mass factor
pub const MASF: f32 = 1.0;
/// Moment of inertia factor
pub const INMF: f32 = 0.2;

/// Gravitaty
const GRAV: Vec2 = Vec2::new(0.0, 4.0);
/// Air resistance
const AIRF: f32 = 0.01;

/// Elasticity of balls
const ELAST: f32 = 100.0;

/// Damping factor.
const DAMP: f32 = 0.2;
/// Liquid friction
const FRICT: f32 = 0.5;

/// Attraction damping.
const ADAMP: f32 = 4.0;

#[derive(Clone, Debug)]
pub enum Shape {
    Circle {
        radius: f32,
    },
    Rectangle {
        /// Half len of rectangle sides
        size: Vec2,
    },
}

impl Shape {
    pub fn radius(&self) -> f32 {
        match self {
            Shape::Circle { radius } => *radius,
            Shape::Rectangle { size } => size.min_element(),
        }
    }
}

pub trait Actor {
    /// Apply force to the specific point of the body.
    fn apply(&mut self, body: &mut Body, pos: Vec2, force: Vec2);
}

struct DerivActor;
impl Actor for DerivActor {
    fn apply(&mut self, body: &mut Body, pos: Vec2, force: Vec2) {
        body.vel.add_deriv(force / body.mass);
        body.asp
            .add_deriv(torque2(pos - *body.pos, force) / body.inm);
    }
}

/// Rigid body
#[derive(Clone, Debug)]
pub struct Body {
    pub mass: f32,
    pub pos: Var<Vec2>,
    pub vel: Var<Vec2>,

    /// Moment of inertia
    pub inm: f32,
    /// Rotation.
    pub rot: Var<Rot2>,
    /// Angular speed.
    pub asp: Var<f32>,
}

impl Body {
    fn vel_at(&self, p: Vec2) -> Vec2 {
        *self.vel + angular_to_linear2(*self.asp, p - *self.pos)
    }

    /// Push item at center by deformation .
    pub fn push(&mut self, actor: &mut impl Actor, def: Vec2) {
        let total_f = ELAST * def - ADAMP * *self.vel;
        actor.apply(self, *self.pos, total_f);
    }

    /// Influence item by directed deformation `def` at point of contact `pos` moving with velocity `vel`.
    pub fn contact(&mut self, actor: &mut impl Actor, def: Vec2, pos: Vec2, vel: Vec2) {
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

        actor.apply(self, pos, total_f);
    }

    /// Pin `loc_pos` point in local item coordinates to `target` point in world space.
    pub fn attract(&mut self, actor: &mut impl Actor, target: Vec2, self_pos: Vec2) {
        let loc_pos = self.rot.transform(self_pos);
        let rel_pos = target - (*self.pos + loc_pos);
        let vel = *self.vel + angular_to_linear2(*self.asp, loc_pos);

        // Elastic attraction
        let elast_f = ELAST * rel_pos;
        // Constant damping
        let damp_f = -ADAMP * vel;
        // Total force
        let total_f = elast_f + damp_f;

        actor.apply(self, *self.pos + loc_pos, total_f);
    }
}

fn contact_wall(actor: &mut impl Actor, item: &mut Item, offset: f32, normal: Vec2) {
    if let Some((area, barycenter)) =
        intersect_circle_and_plane(*item.pos, item.shape.radius(), offset, normal)
    {
        item.body
            .contact(actor, -0.1 * normal * area, barycenter, Vec2::ZERO);
    }
}

/// Wall offset factor
pub const WALL_OFFSET: f32 = 0.04;

impl World {
    pub fn compute_derivs_ext(&mut self, actor: &mut impl Actor) {
        for item in self.items.iter_mut() {
            let radius = item.shape.radius();
            let body = &mut item.body;

            body.pos.add_deriv(*body.vel);
            body.rot.add_deriv(*body.asp);

            // Gravity
            actor.apply(body, *body.pos, GRAV * body.mass);

            // Air resistance
            body.vel.add_deriv(-(AIRF * radius / body.mass) * *body.vel);
            body.asp.add_deriv(-(AIRF * radius / body.inm) * *body.asp);

            // Walls
            let wall_size = self.size - WALL_OFFSET * self.size.min_element();
            contact_wall(actor, item, -wall_size.x, Vec2::new(1.0, 0.0));
            contact_wall(actor, item, -wall_size.x, Vec2::new(-1.0, 0.0));
            contact_wall(actor, item, -wall_size.y, Vec2::new(0.0, 1.0));
            contact_wall(actor, item, -wall_size.y, Vec2::new(0.0, -1.0));
        }

        for i in 1..self.items.len() {
            let (left, other_items) = self.items.split_at_mut(i);
            let this = left.last_mut().unwrap();
            for other in other_items {
                let rel_pos = *other.body.pos - *this.body.pos;
                let dist = rel_pos.length();
                let sum_radius = this.shape.radius() + other.shape.radius();
                if dist < sum_radius {
                    let dir = rel_pos.normalize_or_zero();
                    let dev = (sum_radius - dist) / 2.0;
                    let min_radius = f32::min(this.shape.radius(), other.shape.radius());
                    if 2.0 * dev < min_radius {
                        let poc = *this.pos + dir * (this.shape.radius() - dev);
                        this.contact(actor, -dev * dir, poc, other.vel_at(poc));
                        other.contact(actor, dev * dir, poc, this.vel_at(poc));
                    } else {
                        this.push(actor, -(min_radius / 2.0) * dir);
                        other.push(actor, (min_radius / 2.0) * dir);
                    }
                }
            }
        }

        if let Some((i, target, loc_pos)) = self.drag {
            let item = &mut self.items[i];
            item.body.attract(actor, target, loc_pos);
        }
    }
}

impl System for World {
    fn compute_derivs(&mut self, _dt: f32) {
        self.compute_derivs_ext(&mut DerivActor);
    }
    fn visit_vars<V: Visitor>(&mut self, visitor: &mut V) {
        for ent in &mut self.items {
            visitor.apply(&mut ent.pos);
            visitor.apply(&mut ent.vel);
            visitor.apply(&mut ent.rot);
            visitor.apply(&mut ent.asp);
        }
    }
}
