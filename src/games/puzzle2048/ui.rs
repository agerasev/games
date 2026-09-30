use super::{
    Action, Game,
    model::{Rule, Spawn},
};
use wgame_egui::egui;

impl Game {
    pub(crate) fn controls(
        &self,
        ui: &mut egui::Ui,
        language: crate::language::Language,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let settings = self.board.settings();
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!(
                "{}: {}",
                language.text("Score", "Счёт"),
                self.board.score()
            ));
            ui.label(format!(
                "{}: {}",
                language.text("Moves", "Ходы"),
                self.board.moves()
            ));
            if ui
                .add_enabled(
                    self.board.can_undo(),
                    egui::Button::new(language.text("Undo / U", "Отмена / U")),
                )
                .clicked()
            {
                actions.push(Action::Undo);
            }
            if ui
                .button(language.text("Restart / R", "Заново / R"))
                .clicked()
            {
                actions.push(Action::Restart);
            }
        });
        wgame_egui::egui::CollapsingHeader::new(language.text("Settings", "Настройки"))
            .id_salt("settings")
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(language.text("Board:", "Поле:"));
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
                    ui.label(language.text("Rules:", "Правила:"));
                    for (rule, title) in [
                        (Rule::Classic, "2048"),
                        (Rule::Fibonacci, language.text("Fibonacci", "Фибоначчи")),
                    ] {
                        if ui.selectable_label(settings.rule == rule, title).clicked() {
                            actions.push(Action::Rule(rule));
                        }
                    }
                });
                ui.small(language.text(
                    "Changing the size or rules starts a new round.",
                    "Размер и правила начинают новую игру.",
                ));
                let first = settings.rule.first();
                ui.horizontal_wrapped(|ui| {
                    ui.label(language.text("New tiles:", "Новые плитки:"));
                    for (spawn, title) in [
                        (
                            Spawn::SmallOnly,
                            format!("{} {first}", language.text("Only", "Только")),
                        ),
                        (
                            Spawn::Mixed,
                            format!(
                                "{first} {} {} (90% / 10%)",
                                language.text("and", "и"),
                                first * 2
                            ),
                        ),
                    ] {
                        if ui
                            .selectable_label(settings.spawn == spawn, title)
                            .clicked()
                        {
                            actions.push(Action::Spawn(spawn));
                        }
                    }
                });
                ui.small(language.text(
                    "Change new tiles without resetting the board.",
                    "Новые плитки можно менять без сброса поля.",
                ));
                ui.label(if settings.rule == Rule::Classic {
                    language.text(
                        "Equal numbers merge. Goal: 2048.",
                        "Равные числа соединяются. Цель: 2048.",
                    )
                } else {
                    language.text(
                        "1+1=2, 1+2=3, 2+3=5… Goal: 2584.",
                        "1+1=2, 1+2=3, 2+3=5… Цель: 2584.",
                    )
                });
            });
        ui.small(if !self.board.can_move() {
            language.text(
                "No moves left. Undo or start again.",
                "Нет ходов. Отмена или новая игра.",
            )
        } else if self.board.won() {
            language.text(
                "Goal reached! Keep playing.",
                "Цель достигнута! Игра продолжается.",
            )
        } else {
            language.text(
                "Arrows / WASD or swipe the board",
                "Стрелки / WASD или свайп по полю",
            )
        });
        actions
    }
}
