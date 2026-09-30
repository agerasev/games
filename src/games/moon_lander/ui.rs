use super::{Action, Game, model::*};
use wgame_egui::egui::{self, Color32};

impl Game {
    pub(crate) fn controls(
        &self,
        ui: &mut egui::Ui,
        language: crate::language::Language,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let f = &self.flight;
        ui.horizontal_wrapped(|ui| {
            let pause = if !self.started {
                language.text("Start", "Старт")
            } else if self.paused {
                language.text("Resume / P", "Продолжить / P")
            } else {
                language.text("Pause / P", "Пауза / P")
            };
            if ui
                .add_enabled(f.status == Status::Flying, egui::Button::new(pause))
                .clicked()
            {
                actions.push(Action::TogglePause);
            }
            if ui
                .button(language.text("Restart / R", "Заново / R"))
                .clicked()
            {
                actions.push(Action::Restart);
            }
            if f.status == Status::Landed
                && ui
                    .button(language.text("Next / N", "Следующая / N"))
                    .clicked()
            {
                actions.push(Action::Site((f.site + 1) % Flight::SITES.len()));
            }
            ui.menu_button(
                language.text("Landing site", "Место посадки"),
                |ui| {
                    for (i, name) in Flight::SITES.iter().enumerate() {
                        if ui
                            .selectable_label(
                                i == f.site,
                                format!("{}. {}", i + 1, name.get(language)),
                            )
                            .clicked()
                        {
                            actions.push(Action::Site(i));
                            ui.close();
                        }
                    }
                },
            );
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "{} {:.1} {}",
                language.text("Altitude", "Высота"),
                f.altitude(),
                language.text("m", "м")
            ));
            let safe = Color32::from_rgb(115, 230, 190);
            let danger = Color32::from_rgb(255, 160, 100);
            for (label, value, limit, unit) in [
                (
                    "Vx",
                    f.craft.vel.x,
                    MAX_HORIZONTAL,
                    language.text("m/s", "м/с"),
                ),
                (
                    "Vy",
                    f.craft.vel.y,
                    MAX_VERTICAL,
                    language.text("m/s", "м/с"),
                ),
                (
                    language.text("Tilt", "Наклон"),
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
                    .text(format!(
                        "{} {:.0}%",
                        language.text("Fuel", "Топливо"),
                        f.fuel
                    )),
            );
        });
        let mut control = Control::default();
        ui.horizontal_wrapped(|ui| {
            ui.add_enabled_ui(f.status == Status::Flying && (!self.started || !self.paused), |ui| {
                ui.horizontal_wrapped(|ui| {
                    let left = ui.button("A / <").on_hover_text(language.text("Hold to tilt left", "Удерживайте для наклона влево")).is_pointer_button_down_on();
                    control.thrust = ui.button(language.text("Thrust / Space", "Тяга / Space")).on_hover_text(language.text("Hold to thrust", "Удерживайте для работы двигателя")).is_pointer_button_down_on();
                    let right = ui.button("D / >").on_hover_text(language.text("Hold to tilt right", "Удерживайте для наклона вправо")).is_pointer_button_down_on();
                    control.turn = f32::from(left) - f32::from(right);
                });
            });
            ui.label(language.text("Hold the buttons", "Удерживайте кнопки"));
            ui.menu_button(language.text("Landing", "Посадка"), |ui| {
                ui.set_max_width(290.0);
                ui.label(language.text("Both feet must touch the green pad. Speed: |Vx| ≤ 1.5 m/s, |Vy| ≤ 3 m/s. Tilt ≤ 12°, spin ≤ 26°/s.", "Обе опоры на зелёной площадке. Скорость: |Vx| ≤ 1.5 м/с, |Vy| ≤ 3 м/с. Наклон ≤ 12°, вращение ≤ 26°/с."));
                ui.label(language.text("Tilt to fly sideways. Tilt the other way to brake. Releasing steering stops the spin but keeps the current tilt.", "Наклоните модуль, чтобы лететь вбок. Для торможения наклонитесь в обратную сторону. Отпускание поворота гасит вращение, но не выравнивает модуль."));
            });
        });
        // Held controls must not consume the frame: physics continues while pressed.
        actions.push(Action::Pilot(control));
        actions
    }
}
