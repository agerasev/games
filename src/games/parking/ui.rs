use super::{Action, Game, model::*};
use wgame_egui::egui::{self, Color32};
impl Game {
    pub(crate) fn controls(&self, ui: &mut egui::Ui) -> Vec<Action> {
        let mut actions = Vec::new();
        let round = &self.round;
        ui.horizontal_wrapped(|ui| {
            let title = if !self.started {
                "Старт"
            } else if self.paused {
                "Продолжить / P"
            } else {
                "Пауза / P"
            };
            if ui
                .add_enabled(round.status == Status::Driving, egui::Button::new(title))
                .clicked()
            {
                actions.push(Action::Pause);
            }
            if ui.button("Заново / R").clicked() {
                actions.push(Action::Restart);
            }
            if round.status == Status::Parked && ui.button("Далее / N").clicked() {
                actions.push(Action::Level((round.index + 1) % NAMES.len()));
            }
            ui.menu_button("Уровень", |ui| {
                for (i, name) in NAMES.iter().enumerate() {
                    if ui
                        .selectable_label(round.index == i, format!("{}. {name}", i + 1))
                        .clicked()
                    {
                        actions.push(Action::Level(i));
                        ui.close();
                    }
                }
            });
        });
        ui.separator();
        ui.label(format!("Скорость: {:.1} км/ч", round.car.speed.abs() * 3.6));
        ui.label(if round.car.speed < -0.01 {
            "Движение назад"
        } else {
            "Движение вперёд"
        });
        let good = Color32::from_rgb(110, 225, 175);
        let dim = Color32::from_gray(145);
        for (ok, text) in [
            (round.inside(), "Вся машина в разметке"),
            (round.aligned(), "Машина выровнена"),
            (round.car.speed.abs() <= PARK_SPEED, "Полная остановка"),
        ] {
            ui.colored_label(if ok { good } else { dim }, text);
        }
        ui.add(
            egui::ProgressBar::new(round.hold / HOLD_TIME)
                .desired_width(ui.available_width())
                .text("Зафиксируйте парковку"),
        );
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            for (gear, text) in [(1.0, "Вперёд"), (-1.0, "Назад")] {
                if ui.selectable_label(self.gear == gear, text).clicked() {
                    actions.push(Action::Gear(gear));
                }
            }
        });
        ui.label("Руль: влево / вправо");
        // A retained slider lets one pointer steer and then hold the accelerator.
        let mut wheel = -self.wheel;
        if ui
            .add(egui::Slider::new(&mut wheel, -1.0..=1.0).show_value(false))
            .changed()
        {
            actions.push(Action::Steering(-wheel));
        }
        if ui.button("Руль прямо").clicked() {
            actions.push(Action::Steering(0.0));
        }
        let mut control = Control::default();
        ui.add_enabled_ui(
            round.status == Status::Driving && (!self.started || !self.paused),
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button("Газ / W, S")
                        .on_hover_text("Удерживайте. Передачу выберите выше.")
                        .is_pointer_button_down_on()
                    {
                        control.drive = self.gear;
                    }
                    control.brake = ui.button("Тормоз / Space").is_pointer_button_down_on();
                });
            },
        );
        ui.collapsing("Как парковаться", |ui| {
            ui.label("W / вверх - вперёд; S / вниз - назад. A, D / стрелки - руль; Space - тормоз.");
            ui.label("Полностью въедьте в зелёную рамку, выровняйте машину и остановитесь на секунду. Разрешена парковка передом или задом.");
            ui.label("При движении назад поворот меняет направление. Пунктир показывает путь при текущем руле. Ползунок руля сохраняет положение; кнопка «Руль прямо» возвращает его в центр.");
            ui.label("Касание машин и бордюров заканчивает попытку. В последнем уровне следите за встречным потоком.");
        });
        actions.push(Action::Pilot(control));
        actions
    }
}
