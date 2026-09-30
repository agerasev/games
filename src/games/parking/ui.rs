use super::{Action, Game, model::*};
use wgame_egui::egui::{self, Color32};
impl Game {
    pub(crate) fn controls(
        &self,
        ui: &mut egui::Ui,
        language: crate::language::Language,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let round = &self.round;
        ui.horizontal_wrapped(|ui| {
            let title = if !self.started {
                language.text("Start", "Старт")
            } else if self.paused {
                language.text("Resume / P", "Продолжить / P")
            } else {
                language.text("Pause / P", "Пауза / P")
            };
            if ui
                .add_enabled(round.status == Status::Driving, egui::Button::new(title))
                .clicked()
            {
                actions.push(Action::Pause);
            }
            if ui
                .button(language.text("Restart / R", "Заново / R"))
                .clicked()
            {
                actions.push(Action::Restart);
            }
            if round.status == Status::Parked
                && ui.button(language.text("Next / N", "Далее / N")).clicked()
            {
                actions.push(Action::Level((round.index + 1) % NAMES.len()));
            }
            ui.menu_button(language.text("Level", "Уровень"), |ui| {
                for (i, name) in NAMES.iter().enumerate() {
                    if ui
                        .selectable_label(
                            round.index == i,
                            format!("{}. {}", i + 1, name.get(language)),
                        )
                        .clicked()
                    {
                        actions.push(Action::Level(i));
                        ui.close();
                    }
                }
            });
        });
        ui.separator();
        ui.label(format!(
            "{}: {:.1} {}",
            language.text("Speed", "Скорость"),
            round.car.speed.abs() * 3.6,
            language.text("km/h", "км/ч")
        ));
        ui.label(if round.car.speed < -0.01 {
            language.text("Moving backward", "Движение назад")
        } else {
            language.text("Moving forward", "Движение вперёд")
        });
        let good = Color32::from_rgb(110, 225, 175);
        let dim = Color32::from_gray(145);
        for (ok, text) in [
            (
                round.inside(),
                language.text("Whole car inside the bay", "Вся машина в разметке"),
            ),
            (
                round.aligned(),
                language.text("Car aligned", "Машина выровнена"),
            ),
            (
                round.car.speed.abs() <= PARK_SPEED,
                language.text("Full stop", "Полная остановка"),
            ),
        ] {
            ui.colored_label(if ok { good } else { dim }, text);
        }
        ui.add(
            egui::ProgressBar::new(round.hold / HOLD_TIME)
                .desired_width(ui.available_width())
                .text(language.text("Hold the parking position", "Зафиксируйте парковку")),
        );
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            for (gear, text) in [
                (1.0, language.text("Forward", "Вперёд")),
                (-1.0, language.text("Reverse", "Назад")),
            ] {
                if ui.selectable_label(self.gear == gear, text).clicked() {
                    actions.push(Action::Gear(gear));
                }
            }
        });
        ui.label(language.text("Steering: left / right", "Руль: влево / вправо"));
        // A retained slider lets one pointer steer and then hold the accelerator.
        let mut wheel = -self.wheel;
        if ui
            .add(egui::Slider::new(&mut wheel, -1.0..=1.0).show_value(false))
            .changed()
        {
            actions.push(Action::Steering(-wheel));
        }
        if ui
            .button(language.text("Center steering", "Руль прямо"))
            .clicked()
        {
            actions.push(Action::Steering(0.0));
        }
        let mut control = Control::default();
        ui.add_enabled_ui(
            round.status == Status::Driving && (!self.started || !self.paused),
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(language.text("Accelerate / W, S", "Газ / W, S"))
                        .on_hover_text(language.text(
                            "Hold to drive. Choose a gear above.",
                            "Удерживайте. Передачу выберите выше.",
                        ))
                        .is_pointer_button_down_on()
                    {
                        control.drive = self.gear;
                    }
                    control.brake = ui
                        .button(language.text("Brake / Space", "Тормоз / Space"))
                        .is_pointer_button_down_on();
                });
            },
        );
        wgame_egui::egui::CollapsingHeader::new(language.text("How to park", "Как парковаться"))
            .id_salt("parking-help")
            .show(ui, |ui| {
            ui.label(language.text("W / up - forward; S / down - reverse. A, D / arrows - steer; Space - brake.", "W / вверх - вперёд; S / вниз - назад. A, D / стрелки - руль; Space - тормоз."));
            ui.label(language.text("Put the whole car inside the green bay, align it, and stop for one second. You may park facing either direction.", "Полностью въедьте в зелёную рамку, выровняйте машину и остановитесь на секунду. Разрешена парковка передом или задом."));
            ui.label(language.text("Steering reverses when backing up. The dotted guide shows the path at the current steering angle. The slider keeps its position; Center steering resets it.", "При движении назад поворот меняет направление. Пунктир показывает путь при текущем руле. Ползунок руля сохраняет положение; кнопка «Руль прямо» возвращает его в центр."));
            ui.label(language.text("Hitting a car or curb ends the attempt. Watch for oncoming traffic in the last level.", "Касание машин и бордюров заканчивает попытку. В последнем уровне следите за встречным потоком."));
        });
        actions.push(Action::Pilot(control));
        actions
    }
}
