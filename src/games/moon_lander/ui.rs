use super::{Action, Game, model::*};
use wgame_egui::egui::{self, Color32};

impl Game {
    pub(crate) fn controls(&self, ui: &mut egui::Ui) -> Vec<Action> {
        let mut actions = Vec::new();
        let f = &self.flight;
        ui.horizontal_wrapped(|ui| {
            let pause = if !self.started {
                "Старт"
            } else if self.paused {
                "Продолжить / P"
            } else {
                "Пауза / P"
            };
            if ui
                .add_enabled(f.status == Status::Flying, egui::Button::new(pause))
                .clicked()
            {
                actions.push(Action::TogglePause);
            }
            if ui.button("Заново / R").clicked() {
                actions.push(Action::Restart);
            }
            if f.status == Status::Landed && ui.button("Следующая / N").clicked() {
                actions.push(Action::Site((f.site + 1) % Flight::SITES.len()));
            }
            ui.menu_button("Место посадки", |ui| {
                for (i, name) in Flight::SITES.iter().enumerate() {
                    if ui
                        .selectable_label(i == f.site, format!("{}. {name}", i + 1))
                        .clicked()
                    {
                        actions.push(Action::Site(i));
                        ui.close();
                    }
                }
            });
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("Высота {:.1} м", f.altitude()));
            let safe = Color32::from_rgb(115, 230, 190);
            let danger = Color32::from_rgb(255, 160, 100);
            for (label, value, limit, unit) in [
                ("Vx", f.craft.vel.x, MAX_HORIZONTAL, "м/с"),
                ("Vy", f.craft.vel.y, MAX_VERTICAL, "м/с"),
                (
                    "Наклон",
                    f.craft.angle.to_degrees(),
                    MAX_TILT.to_degrees(),
                    "°",
                ),
            ] {
                ui.colored_label(
                    if value.abs() <= limit { safe } else { danger },
                    format!("{label} {value:.1} {unit}"),
                );
            }
            ui.add(
                egui::ProgressBar::new(f.fuel / 100.0)
                    .desired_width(120.0)
                    .text(format!("Топливо {:.0}%", f.fuel)),
            );
        });
        let mut control = Control::default();
        ui.horizontal_wrapped(|ui| {
            ui.add_enabled_ui(f.status == Status::Flying && (!self.started || !self.paused), |ui| {
                ui.horizontal_wrapped(|ui| {
                    let left = ui.button("A / <").on_hover_text("Удерживайте для наклона влево").is_pointer_button_down_on();
                    control.thrust = ui.button("Тяга / Space").on_hover_text("Удерживайте для работы двигателя").is_pointer_button_down_on();
                    let right = ui.button("D / >").on_hover_text("Удерживайте для наклона вправо").is_pointer_button_down_on();
                    control.turn = f32::from(left) - f32::from(right);
                });
            });
            ui.label("Удерживайте кнопки");
            ui.menu_button("Посадка", |ui| {
                ui.set_max_width(290.0);
                ui.label("Обе опоры на зелёной площадке. Скорость: |Vx| ≤ 1.5 м/с, |Vy| ≤ 3 м/с. Наклон ≤ 12°, вращение ≤ 26°/с.");
                ui.label("Наклоните модуль, чтобы лететь вбок. Для торможения наклонитесь в обратную сторону. Отпускание поворота гасит вращение, но не выравнивает модуль.");
            });
        });
        // Held controls must not consume the frame: physics continues while pressed.
        actions.push(Action::Pilot(control));
        actions
    }
}
