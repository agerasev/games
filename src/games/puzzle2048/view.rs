use super::{
    Action, Game, POP, SLIDE,
    model::{Rule, Settings, Spawn},
};
use crate::{
    draw::Painter,
    layout::{contains, rect},
};
use euclid::default::Rect;
use wgame::{
    gfx::types::color,
    glam::{Vec2, Vec3, Vec4},
    prelude::*,
};

pub(super) struct Layout {
    pub board: Rect<f32>,
    panel: Rect<f32>,
    margin: f32,
}
pub(super) struct Control {
    pub rect: Rect<f32>,
    pub action: Action,
    label: String,
    selected: bool,
}
impl Layout {
    pub fn new(size: Vec2) -> Self {
        let margin = (size.x * 0.035).clamp(4.0, 24.0);
        let wide = size.x >= 560.0 && size.x >= size.y * 1.15;
        let (board, panel) = if wide {
            let panel_width = (size.x * 0.35).clamp(210.0, 302.0);
            let panel_y = 134.0_f32.min((size.y - 220.0).max(88.0));
            let area = Vec2::new(size.x - panel_width - margin * 3.0, size.y - 108.0);
            let side = area.min_element().max(1.0);
            (
                rect(
                    margin + (area.x - side) / 2.0,
                    88.0 + (area.y - side) / 2.0,
                    side,
                    side,
                ),
                rect(
                    size.x - margin - panel_width,
                    panel_y,
                    panel_width,
                    (size.y - panel_y - 12.0).clamp(1.0, 208.0),
                ),
            )
        } else {
            let side = (size.x - margin * 2.0).min(size.y - 322.0).max(1.0);
            let board = rect((size.x - side) / 2.0, 90.0, side, side);
            (
                board,
                rect(margin, board.max_y() + 12.0, size.x - margin * 2.0, 208.0),
            )
        };
        Self {
            board,
            panel,
            margin,
        }
    }
    fn panel_row(&self, x: f32, y: f32, width: f32, height: f32) -> Rect<f32> {
        let scale = self.panel.size.height / 208.0;
        rect(
            self.panel.min_x() + x,
            self.panel.min_y() + y * scale,
            width,
            height * scale,
        )
    }
    pub fn controls(&self, settings: Settings) -> Vec<Control> {
        let mut result = Vec::new();
        let label_width = self.panel.size.width * 0.27;
        let width = self.panel.size.width - label_width;
        let mut row = |y: f32, items: Vec<(String, Action, bool)>| {
            let count = items.len();
            for (i, (label, action, selected)) in items.into_iter().enumerate() {
                let step = width / count as f32;
                result.push(Control {
                    rect: self.panel_row(
                        label_width + i as f32 * step,
                        y,
                        (step - 4.0).max(0.0),
                        32.0,
                    ),
                    action,
                    label,
                    selected,
                });
            }
        };
        row(
            0.0,
            (3..=6)
                .map(|n| (format!("{n}x{n}"), Action::Size(n), settings.side == n))
                .collect(),
        );
        row(
            40.0,
            vec![
                (
                    "2048".into(),
                    Action::Rule(Rule::Classic),
                    settings.rule == Rule::Classic,
                ),
                (
                    "Фибо".into(),
                    Action::Rule(Rule::Fibonacci),
                    settings.rule == Rule::Fibonacci,
                ),
            ],
        );
        let first = settings.rule.first();
        row(
            80.0,
            vec![
                (
                    format!("Только {first}"),
                    Action::Spawn(Spawn::SmallOnly),
                    settings.spawn == Spawn::SmallOnly,
                ),
                (
                    format!("{first} и {}", first * 2),
                    Action::Spawn(Spawn::Mixed),
                    settings.spawn == Spawn::Mixed,
                ),
            ],
        );
        for (i, (label, action)) in [
            ("Заново / R", Action::Restart),
            ("Отмена / U", Action::Undo),
        ]
        .into_iter()
        .enumerate()
        {
            let width = self.panel.size.width / 2.0;
            result.push(Control {
                rect: self.panel_row(i as f32 * width, 123.0, width - 4.0, 34.0),
                action,
                label: label.into(),
                selected: false,
            });
        }
        result
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
    let title_width = (size.x * 0.36).min(300.0);
    painter.label(
        if settings.rule == Rule::Classic {
            "2048"
        } else {
            "Фибо"
        },
        rect(layout.margin, 8.0, title_width, 47.0),
        42.0,
        0,
        color::WHITE,
    );
    let metrics_width = (size.x - title_width - layout.margin * 3.0).min(302.0);
    for (i, (label, value)) in [("СЧЁТ", game.board.score()), ("ХОДЫ", game.board.moves())]
        .into_iter()
        .enumerate()
    {
        let step = metrics_width / 2.0;
        let x = size.x - layout.margin - metrics_width + i as f32 * step;
        rounded(
            painter,
            rect(x, 8.0, step - 5.0, 50.0),
            6.0,
            Vec3::new(0.08, 0.13, 0.18),
        );
        painter.label(
            label,
            rect(x + 3.0, 10.0, step - 11.0, 15.0),
            11.0,
            0,
            Vec3::splat(0.65),
        );
        painter.label(
            &value.to_string(),
            rect(x + 3.0, 25.0, step - 11.0, 29.0),
            23.0,
            0,
            color::WHITE,
        );
    }
    painter.label(
        if settings.rule == Rule::Classic {
            "Равные числа соединяются. Цель: 2048"
        } else {
            "1+1=2, 1+2=3, 2+3=5... Цель: 2584"
        },
        rect(layout.margin, 63.0, size.x - 2.0 * layout.margin, 19.0),
        15.0,
        0,
        Vec3::new(0.55, 0.72, 0.81),
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
    for control in layout.controls(settings) {
        let enabled = !matches!(control.action, Action::Undo) || game.board.can_undo();
        painter.button(
            control.rect,
            control.selected,
            enabled && game.pointer.is_some_and(|p| contains(control.rect, p)),
        );
        painter.label(
            &control.label,
            control.rect.inflate(-3.0, -3.0),
            16.0,
            0,
            if enabled { Vec3::ONE } else { Vec3::splat(0.4) },
        );
    }
    for (label, y) in [
        ("Поле / 3-6", 0.0),
        ("Правила / F", 40.0),
        ("Новые / T", 80.0),
    ] {
        painter.label(
            label,
            layout.panel_row(0.0, y, layout.panel.size.width * 0.27 - 5.0, 32.0),
            13.0,
            0,
            Vec3::new(0.58, 0.70, 0.77),
        );
    }
    let status = if !game.board.can_move() {
        "Нет ходов. Отмена или новая игра."
    } else if game.board.won() {
        "Цель достигнута! Игра продолжается."
    } else {
        "Стрелки / WASD или свайп по полю"
    };
    painter.label(
        status,
        layout.panel_row(0.0, 162.0, layout.panel.size.width, 20.0),
        14.0,
        0,
        if game.board.won() {
            Vec3::new(0.4, 0.95, 0.65)
        } else {
            Vec3::splat(0.85)
        },
    );
    painter.label(
        if settings.spawn == Spawn::Mixed {
            "Новые плитки: 90% / 10%. Правила сбросят игру."
        } else {
            "Выбор размера или правил начнёт новую игру."
        },
        layout.panel_row(0.0, 187.0, layout.panel.size.width, 18.0),
        11.0,
        0,
        Vec3::splat(0.5),
    );
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
