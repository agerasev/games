use super::{Game, model::*};
use crate::{draw::Painter, layout::rect};
use euclid::default::Rect;
use wgame::{
    glam::{Vec2, Vec3, Vec4},
    prelude::*,
};

const MINT: Vec3 = Vec3::new(0.4, 0.95, 0.75);
fn line(p: &mut Painter<'_>, a: Vec2, b: Vec2, width: f32, color: Vec3) {
    p.scene
        .add(&p.lib.shapes().line(a, b, width).fill_color(color));
}
fn fill(p: &mut Painter<'_>, body: Box2, world: &impl Fn(Vec2) -> Vec2, color: Vec4) {
    let [a, b, c, d] = body.corners().map(world);
    p.scene
        .add(&p.lib.shapes().quad(a, b, c, d).fill_color(color));
}
fn outline(
    p: &mut Painter<'_>,
    body: Box2,
    world: &impl Fn(Vec2) -> Vec2,
    width: f32,
    color: Vec3,
) {
    let points = body.corners().map(world);
    for i in 0..4 {
        line(p, points[i], points[(i + 1) % 4], width, color);
    }
}
fn car(p: &mut Painter<'_>, body: Box2, steer: f32, world: &impl Fn(Vec2) -> Vec2, color: Vec3) {
    fill(
        p,
        Box2 {
            center: body.center + Vec2::new(0.15, -0.15),
            ..body
        },
        world,
        Vec4::new(0.0, 0.0, 0.0, 0.3),
    );
    let part = |x, y, length, width, angle| Box2 {
        center: body.point(Vec2::new(x, y)),
        half: Vec2::new(length, width) * 0.5,
        angle: body.angle + angle,
    };
    for x in [-1.3, 1.3] {
        for y in [-0.9, 0.9] {
            fill(
                p,
                part(x, y, 0.75, 0.28, if x > 0.0 { steer } else { 0.0 }),
                world,
                Vec3::splat(0.025).extend(1.0),
            );
        }
    }
    fill(p, body, world, color.extend(1.0));
    fill(
        p,
        part(-0.2, 0.0, 2.4, 1.55, 0.0),
        world,
        Vec3::new(0.09, 0.17, 0.23).extend(1.0),
    );
    fill(
        p,
        part(-0.3, 0.0, 1.3, 1.45, 0.0),
        world,
        (color * 0.8).extend(1.0),
    );
    fill(
        p,
        part(0.75, 0.0, 0.45, 1.4, 0.0),
        world,
        Vec3::new(0.52, 0.77, 0.84).extend(1.0),
    );
    for y in [-0.62, 0.62] {
        fill(
            p,
            part(2.06, y, 0.25, 0.36, 0.0),
            world,
            Vec3::new(1.0, 0.98, 0.75).extend(1.0),
        );
        fill(
            p,
            part(-2.06, y, 0.25, 0.36, 0.0),
            world,
            Vec3::new(1.0, 0.23, 0.18).extend(1.0),
        );
    }
}
pub(crate) fn preview(p: &mut Painter<'_>, r: Rect<f32>) {
    let scale = r.size.width.min(r.size.height) / 9.0;
    let world = |v: Vec2| Vec2::new(r.center().x, r.center().y) + Vec2::new(v.x, -v.y) * scale;
    let bay = Box2 {
        center: Vec2::ZERO,
        half: Vec2::new(3.2, 1.7),
        angle: 0.25,
    };
    fill(p, bay, &world, MINT.extend(0.1));
    outline(p, bay, &world, 2.0, MINT);
    car(
        p,
        Box2::car(Vec2::ZERO, 0.1),
        0.2,
        &world,
        Vec3::new(0.3, 0.75, 0.98),
    );
}
pub(super) fn draw(game: &Game, p: &mut Painter<'_>) {
    let round = &game.round;
    let scale = (p.size.x / WORLD.x).min((p.size.y - 40.0).max(0.0) / WORLD.y);
    if scale <= 0.0 {
        return;
    }
    let offset = (p.size - WORLD * scale) * 0.5 + Vec2::new(0.0, 12.0);
    let world = |v: Vec2| offset + Vec2::new(v.x, WORLD.y - v.y) * scale;
    p.rectangle(
        rect(0.0, 0.0, p.size.x, p.size.y),
        Vec3::new(0.035, 0.065, 0.09),
    );
    let bounds = Box2 {
        center: WORLD * 0.5,
        half: WORLD * 0.5,
        angle: 0.0,
    };
    fill(p, bounds, &world, Vec3::new(0.36, 0.41, 0.43).extend(1.0));
    fill(
        p,
        Box2 {
            half: bounds.half - Vec2::ONE,
            ..bounds
        },
        &world,
        Vec3::new(0.14, 0.19, 0.23).extend(1.0),
    );
    fill(
        p,
        Box2 {
            center: Vec2::new(20.0, 23.0),
            half: Vec2::new(18.0, 3.0),
            angle: 0.0,
        },
        &world,
        Vec3::new(0.17, 0.23, 0.26).extend(1.0),
    );
    for i in 0..10 {
        let x = 2.0 + i as f32 * 3.8;
        line(
            p,
            world(Vec2::new(x, 14.5)),
            world(Vec2::new(x + 1.5, 14.5)),
            (scale * 0.10).max(1.0),
            Vec3::new(0.43, 0.49, 0.51),
        );
    }
    for (x, y, direction) in [(5.0, 12.0, 1.0), (35.0, 17.0, -1.0)] {
        let tip = Vec2::new(x + direction, y);
        let color = Vec3::new(0.37, 0.44, 0.47);
        line(
            p,
            world(Vec2::new(x - direction, y)),
            world(tip),
            scale * 0.12,
            color,
        );
        for side in [-1.0, 1.0] {
            line(
                p,
                world(tip),
                world(tip + Vec2::new(-direction * 0.7, side * 0.5)),
                scale * 0.12,
                color,
            );
        }
    }
    let target = round.level.target;
    fill(p, target, &world, MINT.extend(0.12 + round.hold * 0.25));
    outline(p, target, &world, (scale * 0.12).max(1.5), MINT);
    let marker = world(target.center);
    p.label(
        "P",
        rect(marker.x - scale, marker.y - scale, scale * 2.0, scale * 2.0),
        scale * 1.7,
        0,
        MINT,
    );
    for (i, &body) in round.level.parked.iter().enumerate() {
        car(
            p,
            body,
            0.0,
            &world,
            if i % 2 == 0 {
                Vec3::new(0.49, 0.57, 0.64)
            } else {
                Vec3::new(0.73, 0.59, 0.38)
            },
        );
    }
    for traffic in &round.level.traffic {
        car(p, traffic.body, 0.0, &world, Vec3::new(0.88, 0.4, 0.3));
    }
    if round.status == Status::Driving {
        // Sample the same bicycle curve as the model, at a constant distance per dot.
        let direction = if round.car.speed.abs() > 0.05 {
            round.car.speed.signum()
        } else {
            game.gear
        };
        let beta = (0.5 * round.car.steering.tan()).atan();
        let turn = beta.cos() * round.car.steering.tan() / WHEELBASE;
        let mut pos = *round.car.pos;
        let mut angle = *round.car.angle;
        for i in 0..18 {
            let step = 0.4 * direction;
            pos += Vec2::from_angle(angle + beta + turn * step * 0.5) * step;
            angle += turn * step;
            if pos.cmpgt(Vec2::ONE).all() && pos.cmplt(WORLD - Vec2::ONE).all() {
                let alpha = 0.7 * (1.0 - i as f32 / 22.0);
                p.scene.add(
                    &p.lib
                        .shapes()
                        .unit_circle()
                        .fill_color(MINT.extend(alpha))
                        .scale((scale * 0.09).max(1.0))
                        .move_to(world(pos)),
                );
            }
        }
    }
    car(
        p,
        round.car.body(),
        round.car.steering,
        &world,
        if matches!(round.status, Status::Crashed(_)) {
            Vec3::new(0.62, 0.35, 0.35)
        } else {
            Vec3::new(0.3, 0.75, 0.98)
        },
    );
    p.label(
        &format!("{:02} / {}", round.index + 1, NAMES[round.index]),
        rect(10.0, 8.0, p.size.x - 20.0, 24.0),
        20.0,
        0,
        Vec3::ONE,
    );
    let message = match round.status {
        Status::Parked => Some(("Припарковано!", "Следующий уровень / N")),
        Status::Crashed(reason) => Some((
            match reason {
                Crash::Curb => "Задет бордюр",
                Crash::ParkedCar => "Задета машина",
                Crash::Traffic => "Столкновение с потоком",
            },
            "Заново / R",
        )),
        Status::Driving if !game.started => {
            Some(("Парковка", "W, S - газ / A, D - руль / Space - тормоз"))
        }
        Status::Driving if game.paused => Some(("Пауза", "Продолжить / P")),
        _ => None,
    };
    if let Some((title, hint)) = message {
        let w = (p.size.x - 20.0).min(510.0);
        let x = (p.size.x - w) * 0.5;
        let y = p.size.y * 0.70;
        p.rectangle(rect(x, y, w, 72.0), Vec4::new(0.025, 0.045, 0.085, 0.94));
        p.label(
            title,
            rect(x + 8.0, y + 8.0, w - 16.0, 28.0),
            26.0,
            0,
            Vec3::ONE,
        );
        p.label(hint, rect(x + 8.0, y + 40.0, w - 16.0, 20.0), 17.0, 0, MINT);
    }
}
