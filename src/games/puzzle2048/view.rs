use super::{Game, POP, SLIDE, model::Rule};
use crate::{draw::Painter, layout::rect};
use euclid::default::Rect;
use wgame::{
    gfx::types::color,
    glam::{Vec2, Vec3, Vec4},
    prelude::*,
};

pub(super) struct Layout {
    pub board: Rect<f32>,
}
impl Layout {
    pub fn new(size: Vec2) -> Self {
        let margin = (size.min_element() * 0.035).clamp(0.0, 24.0);
        let side = (size.min_element() - margin * 2.0).max(0.0);
        Self {
            board: rect((size.x - side) / 2.0, (size.y - side) / 2.0, side, side),
        }
    }
    fn cell(&self, side: usize, index: usize) -> Rect<f32> {
        let padding = self.board.size.width * 0.025;
        let step = (self.board.size.width - 2.0 * padding) / side as f32;
        let gap = (step * 0.08).min(9.0);
        rect(
            self.board.min_x() + padding + (index % side) as f32 * step + gap / 2.0,
            self.board.min_y() + padding + (index / side) as f32 * step + gap / 2.0,
            step - gap,
            step - gap,
        )
    }
}
fn rounded(painter: &mut Painter<'_>, area: Rect<f32>, radius: f32, color: Vec3) {
    let r = radius
        .min(area.size.width * 0.5)
        .min(area.size.height * 0.5)
        .max(0.0);
    painter.rectangle(
        rect(
            area.min_x() + r,
            area.min_y(),
            area.size.width - r * 2.0,
            area.size.height,
        ),
        color,
    );
    painter.rectangle(
        rect(
            area.min_x(),
            area.min_y() + r,
            area.size.width,
            area.size.height - r * 2.0,
        ),
        color,
    );
    for x in [area.min_x() + r, area.max_x() - r] {
        for y in [area.min_y() + r, area.max_y() - r] {
            painter.scene.add(
                &painter
                    .lib
                    .shapes()
                    .unit_circle()
                    .fill_color(color)
                    .scale(r)
                    .move_to(Vec2::new(x, y)),
            );
        }
    }
}
fn rank(rule: Rule, value: u64) -> usize {
    if rule == Rule::Classic {
        value.ilog2() as usize - 1
    } else {
        let (mut a, mut b, mut index) = (1_u64, 2_u64, 0);
        while a < value {
            let Some(next) = a.checked_add(b) else { break };
            (a, b) = (b, next);
            index += 1;
        }
        index
    }
}
fn tile(painter: &mut Painter<'_>, area: Rect<f32>, value: u64, rule: Rule, scale: f32, glow: f32) {
    let palette = [
        Vec3::new(0.16, 0.34, 0.46),
        Vec3::new(0.17, 0.43, 0.62),
        Vec3::new(0.06, 0.52, 0.53),
        Vec3::new(0.19, 0.57, 0.37),
        Vec3::new(0.66, 0.48, 0.13),
        Vec3::new(0.83, 0.37, 0.12),
        Vec3::new(0.76, 0.22, 0.25),
        Vec3::new(0.66, 0.22, 0.48),
        Vec3::new(0.45, 0.28, 0.69),
        Vec3::new(0.29, 0.34, 0.77),
        Vec3::new(0.69, 0.51, 0.18),
    ];
    let fill = palette[rank(rule, value) % palette.len()];
    let area = area.inflate(
        area.size.width * (scale - 1.0) / 2.0,
        area.size.height * (scale - 1.0) / 2.0,
    );
    let radius = area.size.width * 0.1;
    if glow > 0.0 {
        rounded(
            painter,
            area.inflate(3.0 * glow, 3.0 * glow),
            radius,
            fill.lerp(Vec3::ONE, glow * 0.5),
        );
    }
    rounded(
        painter,
        area.translate((0.0, 3.0).into()),
        radius,
        Vec3::new(0.02, 0.05, 0.08),
    );
    rounded(painter, area, radius, fill);
    painter.label(
        &value.to_string(),
        area.inflate(-3.0, -3.0),
        area.size.height * 0.48,
        0,
        color::WHITE,
    );
}
pub(super) fn draw(game: &Game, painter: &mut Painter<'_>) {
    let size = painter.size;
    let layout = Layout::new(size);
    let settings = game.board.settings();
    painter.rectangle(
        rect(0.0, 0.0, size.x, size.y),
        Vec3::new(0.025, 0.043, 0.065),
    );
    rounded(
        painter,
        layout.board.translate((0.0, 5.0).into()),
        14.0,
        Vec3::new(0.015, 0.025, 0.04),
    );
    rounded(painter, layout.board, 14.0, Vec3::new(0.09, 0.14, 0.19));
    for i in 0..settings.side * settings.side {
        let cell = layout.cell(settings.side, i);
        rounded(
            painter,
            cell,
            cell.size.width * 0.1,
            Vec3::new(0.055, 0.095, 0.13),
        );
    }
    if let Some(animation) = &game.animation
        && animation.elapsed < SLIDE
    {
        let t = 1.0 - (1.0 - animation.elapsed / SLIDE).powi(3);
        for motion in &animation.turn.motion {
            let start = layout.cell(settings.side, motion.from);
            let end = layout.cell(settings.side, motion.to);
            let offset = (end.origin - start.origin) * t;
            tile(
                painter,
                start.translate(offset),
                motion.value,
                settings.rule,
                1.0,
                0.0,
            );
        }
    } else {
        for (i, &value) in game
            .board
            .cells()
            .iter()
            .enumerate()
            .filter(|(_, value)| **value != 0)
        {
            let (mut scale, mut glow) = (1.0, 0.0);
            if let Some(animation) = &game.animation {
                let t = ((animation.elapsed - SLIDE) / POP).clamp(0.0, 1.0);
                if animation.turn.merged.contains(&i) {
                    glow = (t * std::f32::consts::PI).sin();
                    scale += 0.1 * glow;
                } else if animation.turn.spawned == i {
                    scale = 0.4 + 0.6 * (1.0 - (1.0 - t).powi(2));
                }
            }
            tile(
                painter,
                layout.cell(settings.side, i),
                value,
                settings.rule,
                scale,
                glow,
            );
        }
    }
    if !game.board.can_move() && game.animation.is_none() {
        let b = layout.board;
        painter.rectangle(b, Vec4::new(0.02, 0.03, 0.05, 0.78));
        painter.label(
            "Нет ходов",
            rect(
                b.min_x() + 10.0,
                b.center().y - 32.0,
                b.size.width - 20.0,
                40.0,
            ),
            34.0,
            0,
            color::WHITE,
        );
        painter.label(
            "U - отмена  /  R - заново",
            rect(
                b.min_x() + 10.0,
                b.center().y + 14.0,
                b.size.width - 20.0,
                25.0,
            ),
            17.0,
            0,
            color::WHITE,
        );
    }
}
