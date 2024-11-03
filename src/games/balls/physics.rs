use crate::{
    algebra::Rot2,
    numerical::{System, Var, Visitor},
    physics::{angular_to_linear2, torque2},
};
use macroquad::math::Vec2;

use super::{Item, World};

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
    pub fn push(&mut self, def: Vec2) {
        let total_f = ELAST * def - ADAMP * *self.vel;
        self.vel.add_deriv(total_f / self.mass);
    }

    /// Influence item by directed deformation `def` at point of contact `pos` moving with velocity `vel`.
    pub fn contact(&mut self, def: Vec2, pos: Vec2, vel: Vec2) {
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

    /// Pin `loc_pos` point in local item coordinates to `target` point in world space.
    pub fn attract(&mut self, target: Vec2, loc_pos: Vec2) {
        let loc_pos = self.rot.transform(loc_pos);
        let rel_pos = target - (*self.pos + loc_pos);
        let vel = *self.vel + angular_to_linear2(*self.asp, loc_pos);

        // Elastic attraction
        let elast_f = ELAST * rel_pos;
        // Constant damping
        let damp_f = -ADAMP * vel;
        // Total force
        let total_f = elast_f + damp_f;

        self.vel.add_deriv(total_f / self.mass);
        self.asp.add_deriv(torque2(loc_pos, total_f) / self.inm);
    }
}

fn contact_wall(item: &mut Item, offset: f32, norm: Vec2) {
    let pos = item.pos;
    let dist = pos.dot(norm) + offset;
    let radius = item.shape.radius();
    if dist < radius {
        if dist > 0.0 {
            item.body.contact(
                norm * (radius - dist),
                pos.reject_from(norm) - offset * norm,
                Vec2::ZERO,
            );
        } else {
            item.body.push(norm * radius);
        }
    }
}

impl System for World {
    fn compute_derivs(&mut self, _dt: f32) {
        for item in self.items.iter_mut() {
            let radius = item.shape.radius();
            let body = &mut item.body;

            body.pos.add_deriv(*body.vel);
            body.rot.add_deriv(*body.asp);

            // Gravity
            body.vel.add_deriv(GRAV);

            // Air resistance
            body.vel.add_deriv(-(AIRF * radius / body.mass) * *body.vel);
            body.asp.add_deriv(-(AIRF * radius / body.inm) * *body.asp);

            // Walls
            contact_wall(item, self.size.x, Vec2::new(1.0, 0.0));
            contact_wall(item, self.size.x, Vec2::new(-1.0, 0.0));
            contact_wall(item, self.size.y, Vec2::new(0.0, 1.0));
            contact_wall(item, self.size.y, Vec2::new(0.0, -1.0));
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
                        this.contact(-dev * dir, poc, other.vel_at(poc));
                        other.contact(dev * dir, poc, this.vel_at(poc));
                    } else {
                        this.push(-min_radius * dir);
                        other.push(min_radius * dir);
                    }
                }
            }
        }

        if let Some((i, target, loc_pos)) = self.drag {
            let item = &mut self.items[i];
            item.body.attract(target, loc_pos);
        }
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
