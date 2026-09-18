use super::{Game, model::*};
use crate::{draw::Painter, layout::rect};
use euclid::default::Rect;
use wgame::{
    glam::{Mat2, Vec2, Vec3, Vec4},
    prelude::*,
};

const MINT: Vec3 = Vec3::new(0.4, 0.95, 0.75);
fn line(p: &mut Painter<'_>, a: Vec2, b: Vec2, width: f32, color: Vec3) {
    p.scene
        .add(&p.lib.shapes().line(a, b, width).fill_color(color));
}
fn circle(p: &mut Painter<'_>, center: Vec2, radius: f32, color: Vec4) {
    p.scene.add(
        &p.lib
            .shapes()
            .unit_circle()
            .fill_color(color)
            .scale(radius)
            .move_to(center),
    );
}
fn craft(
    p: &mut Painter<'_>,
    center: Vec2,
    scale: f32,
    angle: f32,
    power: f32,
    time: f32,
    crashed: bool,
) {
    let local = |x, y| {
        let v = Mat2::from_angle(angle) * Vec2::new(x, y);
        center + Vec2::new(v.x, -v.y) * scale
    };
    let metal = if crashed {
        Vec3::splat(0.35)
    } else {
        Vec3::new(0.85, 0.91, 0.95)
    };
    if power > 0.0 {
        p.scene.add(
            &p.lib
                .shapes()
                .triangle(
                    local(-0.5, -1.1),
                    local(0.5, -1.1),
                    local(0.0, -2.8 - (time * 43.0).sin().abs()),
                )
                .fill_color(Vec3::new(1.0, 0.65, 0.2)),
        );
    }
    for side in [-1.0, 1.0] {
        line(
            p,
            local(side, -0.4),
            local(side * 2.5, -FOOT_DROP),
            scale.max(1.0) * 0.3,
            metal,
        );
        line(
            p,
            local(side * 1.3, -1.0),
            local(side * 2.5, -1.7),
            scale.max(1.0) * 0.2,
            metal,
        );
        line(
            p,
            local(side * 2.0, -FOOT_DROP),
            local(side * 3.0, -FOOT_DROP),
            scale.max(1.0) * 0.3,
            metal,
        );
    }
    p.scene.add(
        &p.lib
            .shapes()
            .quad(
                local(-1.3, -1.0),
                local(1.3, -1.0),
                local(1.3, 0.1),
                local(-1.3, 0.1),
            )
            .fill_color(Vec3::new(0.8, 0.55, 0.22)),
    );
    p.scene.add(
        &p.lib
            .shapes()
            .quad(
                local(-1.2, 0.0),
                local(1.2, 0.0),
                local(1.0, 1.5),
                local(-1.0, 1.5),
            )
            .fill_color(metal),
    );
    p.scene.add(
        &p.lib
            .shapes()
            .triangle(local(-1.0, 1.5), local(1.0, 1.5), local(0.0, 2.1))
            .fill_color(metal),
    );
    p.scene.add(
        &p.lib
            .shapes()
            .quad(
                local(-0.65, 0.5),
                local(0.65, 0.5),
                local(0.55, 1.2),
                local(-0.55, 1.2),
            )
            .fill_color(Vec3::new(0.08, 0.32, 0.48)),
    );
}
pub(crate) fn preview(p: &mut Painter<'_>, r: Rect<f32>) {
    let s = r.size.width.min(r.size.height);
    let center = Vec2::new(r.center().x, r.center().y - s * 0.08);
    craft(p, center, s * 0.105, -0.18, 1.0, 0.0, false);
    line(
        p,
        Vec2::new(r.min_x() + s * 0.12, r.max_y() - s * 0.1),
        Vec2::new(r.max_x() - s * 0.12, r.max_y() - s * 0.1),
        3.0,
        MINT,
    );
}
pub(super) fn draw(game: &Game, p: &mut Painter<'_>) {
    let f = &game.flight;
    let s = (p.size.x / WIDTH).min(p.size.y / HEIGHT);
    if s <= 0.0 {
        return;
    }
    let offset = (p.size - Vec2::new(WIDTH, HEIGHT) * s) * 0.5;
    let world = |v: Vec2| offset + Vec2::new(v.x, HEIGHT - v.y) * s;
    p.rectangle(
        rect(0.0, 0.0, p.size.x, p.size.y),
        Vec3::new(0.016, 0.027, 0.065),
    );
    for i in 0..85u32 {
        let x = ((i * 73 + 17) % 997) as f32 / 997.0 * p.size.x;
        let y = ((i * 181 + 29) % 991) as f32 / 991.0 * p.size.y;
        let brightness = 0.2 + (i % 5) as f32 * 0.1;
        circle(
            p,
            Vec2::new(x, y),
            if i % 7 == 0 { 1.3 } else { 0.7 },
            Vec3::splat(brightness).extend(1.0),
        );
    }
    let earth = world(Vec2::new(102.0, 76.0));
    circle(p, earth, 4.3 * s, Vec4::new(0.15, 0.29, 0.5, 1.0));
    circle(
        p,
        earth + Vec2::new(-0.7, -0.6) * s,
        3.6 * s,
        Vec4::new(0.25, 0.54, 0.68, 1.0),
    );
    line(
        p,
        earth + Vec2::new(-2.8, -1.0) * s,
        earth + Vec2::new(1.8, 0.7) * s,
        s,
        Vec3::new(0.65, 0.79, 0.79),
    );
    for i in 0..12 {
        let x = i as f32 * 10.0;
        p.scene.add(
            &p.lib
                .shapes()
                .triangle(
                    world(Vec2::new(x - 10.0, 0.0)),
                    world(Vec2::new(x + 18.0, 0.0)),
                    world(Vec2::new(x + 3.0, 15.0 + (i as f32 * 2.3).sin() * 6.0)),
                )
                .fill_color(Vec3::new(0.07, 0.10, 0.17)),
        );
    }
    for pair in f.terrain.points.windows(2) {
        let [a, b] = [pair[0], pair[1]];
        p.scene.add(
            &p.lib
                .shapes()
                .quad(
                    world(Vec2::new(a.x, 0.0)),
                    world(Vec2::new(b.x, 0.0)),
                    world(b),
                    world(a),
                )
                .fill_color(Vec3::new(0.16, 0.19, 0.25)),
        );
        line(p, world(a), world(b), 1.4, Vec3::new(0.35, 0.40, 0.47));
    }
    let pad = f.terrain.pad;
    line(
        p,
        world(Vec2::new(pad.left, pad.height)),
        world(Vec2::new(pad.right, pad.height)),
        (s * 0.45).max(2.0),
        MINT,
    );
    for x in [pad.left, pad.right] {
        line(
            p,
            world(Vec2::new(x, pad.height)),
            world(Vec2::new(x, pad.height + 1.7)),
            1.2,
            MINT,
        );
        circle(
            p,
            world(Vec2::new(x, pad.height + 1.7)),
            2.0,
            MINT.extend(0.7 + 0.3 * (f.elapsed * 3.0).cos()),
        );
    }
    for part in &f.particles {
        let alpha = (part.life / part.duration).clamp(0.0, 1.0);
        let color = if part.dust {
            Vec3::new(0.6, 0.63, 0.68)
        } else {
            Vec3::new(1.0, 0.25 + 0.65 * alpha, 0.12 + 0.6 * alpha)
        };
        circle(
            p,
            world(*part.pos),
            ((if part.dust { 0.5 } else { 0.22 }) * s).max(0.8),
            color.extend(alpha),
        );
    }
    craft(
        p,
        world(*f.craft.pos),
        s,
        *f.craft.angle,
        f.power,
        f.elapsed,
        matches!(f.status, Status::Crashed(_)),
    );
    p.label(
        &format!("{:02} / {}", f.site + 1, Flight::SITES[f.site]),
        rect(12.0, 8.0, p.size.x - 24.0, 24.0),
        20.0,
        0,
        Vec3::new(0.75, 0.82, 0.9),
    );
    let message = match f.status {
        Status::Landed => Some(("Мягкая посадка!", "Следующая площадка / N")),
        Status::Crashed(reason) => Some((
            match reason {
                Crash::Terrain => "Посадка вне площадки",
                Crash::Speed => "Слишком высокая скорость",
                Crash::Tilt => "Слишком сильный наклон или вращение",
                Crash::Bounds => "Вы покинули район посадки",
            },
            "Заново / R",
        )),
        Status::Flying if !game.started => Some(("Лунный модуль", "Space - тяга / A, D - наклон")),
        Status::Flying if game.paused => Some(("Пауза", "Продолжить / P")),
        _ => None,
    };
    if let Some((title, hint)) = message {
        let w = (p.size.x - 20.0).min(520.0);
        let x = (p.size.x - w) * 0.5;
        let y = p.size.y * 0.43;
        p.rectangle(rect(x, y, w, 72.0), Vec4::new(0.025, 0.045, 0.085, 0.94));
        p.label(
            title,
            rect(x + 10.0, y + 8.0, w - 20.0, 28.0),
            26.0,
            0,
            Vec3::ONE,
        );
        p.label(
            hint,
            rect(x + 10.0, y + 40.0, w - 20.0, 20.0),
            17.0,
            0,
            MINT,
        );
    }
}
