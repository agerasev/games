use super::{
    Action, Game,
    model::{Rule, Spawn},
};
use wgame_egui::egui;

impl Game {
    pub(crate) fn controls(&self, ui: &mut egui::Ui) -> Vec<Action> {
        let mut actions = Vec::new();
        let settings = self.board.settings();
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!("Счёт: {}", self.board.score()));
            ui.label(format!("Ходы: {}", self.board.moves()));
            if ui
                .add_enabled(self.board.can_undo(), egui::Button::new("Отмена / U"))
                .clicked()
            {
                actions.push(Action::Undo);
            }
            if ui.button("Заново / R").clicked() {
                actions.push(Action::Restart);
            }
        });
        ui.collapsing("Настройки", |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Поле:");
                for side in 3..=6 {
                    if ui
                        .selectable_label(settings.side == side, format!("{side}×{side}"))
                        .clicked()
                    {
                        actions.push(Action::Size(side));
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("Правила:");
                for (rule, title) in [(Rule::Classic, "2048"), (Rule::Fibonacci, "Фибоначчи")]
                {
                    if ui.selectable_label(settings.rule == rule, title).clicked() {
                        actions.push(Action::Rule(rule));
                    }
                }
            });
            ui.small("Размер и правила начинают новую игру.");
            let first = settings.rule.first();
            ui.horizontal_wrapped(|ui| {
                ui.label("Новые плитки:");
                for (spawn, title) in [
                    (Spawn::SmallOnly, format!("Только {first}")),
                    (Spawn::Mixed, format!("{first} и {} (90% / 10%)", first * 2)),
                ] {
                    if ui
                        .selectable_label(settings.spawn == spawn, title)
                        .clicked()
                    {
                        actions.push(Action::Spawn(spawn));
                    }
                }
            });
            ui.small("Новые плитки можно менять без сброса поля.");
            ui.label(if settings.rule == Rule::Classic {
                "Равные числа соединяются. Цель: 2048."
            } else {
                "1+1=2, 1+2=3, 2+3=5… Цель: 2584."
            });
        });
        ui.small(if !self.board.can_move() {
            "Нет ходов. Отмена или новая игра."
        } else if self.board.won() {
            "Цель достигнута! Игра продолжается."
        } else {
            "Стрелки / WASD или свайп по полю"
        });
        actions
    }
}
