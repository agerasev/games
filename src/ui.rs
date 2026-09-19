//! Egui owns controls; the canvas owns game input and drawing. Layout is read-only
//! with respect to game state. Collect and deduplicate actions across passes,
//! then apply them once, after `EguiWindow::next_frame`, before simulation.
use crate::App;
use wgame_egui::egui;

#[derive(PartialEq)]
enum Action {
    Game(crate::games::Action),
    Back,
    Quit,
}

#[derive(Default)]
pub struct Actions(Vec<Action>);
impl Actions {
    /// Egui may consume a click in the first pass and omit it from later passes.
    /// Preserve that action while preventing a repeated layout from duplicating it.
    pub fn collect(&mut self, next: Self) {
        for action in next.0 {
            if !self.0.contains(&action) {
                self.0.push(action);
            }
        }
    }
}

pub struct Layout {
    pub canvas: egui::Response,
    pub actions: Actions,
}

/// Configure fonts once; the bundled font includes Cyrillic and Greek labels.
pub fn configure(context: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "games".into(),
        egui::FontData::from_static(include_bytes!("../assets/free-sans-bold.ttf")).into(),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "games".into());
    context.set_fonts(fonts);
    context.all_styles_mut(|style| {
        style.spacing.interact_size.y = 30.0;
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
    });
}

#[cfg(test)]
#[path = "ui_tests.rs"]
mod tests;

impl App {
    pub fn ui(
        &self,
        ui: &mut egui::Ui,
        canvas: impl FnOnce(&mut egui::Ui) -> egui::Response,
    ) -> Layout {
        let mut actions = Actions::default();
        egui::Panel::top("navigation").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if let Some(id) = self.active_id() {
                    if ui.button("Меню / Esc").clicked() {
                        actions.0.push(Action::Back);
                    }
                    ui.heading(id.title());
                    ui.menu_button("Помощь", |ui| {
                        ui.set_max_width(280.0);
                        ui.label(id.hint());
                        ui.label("Щёлкните по полю, чтобы вернуть управление с клавиатуры.");
                    });
                } else {
                    ui.heading("Игры");
                    ui.label("Выберите игру / 1–6 / стрелки и Enter");
                    if ui.button("Выход / Esc").clicked() {
                        actions.0.push(Action::Quit);
                    }
                }
            });
        });
        if let Some(game) = &self.active {
            // Keep room for the canvas even in a narrow portrait window.
            let width = (ui.available_width() * 0.45).min(240.0);
            egui::Panel::left("game-controls")
                .exact_size(width)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.push_id(game.id().slug(), |ui| {
                            actions
                                .0
                                .extend(game.controls(ui).into_iter().map(Action::Game));
                        });
                    });
                });
        }
        let response = egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, canvas)
            .inner;
        if self.focus_canvas {
            response.request_focus();
        }
        Layout {
            canvas: response,
            actions,
        }
    }

    /// Apply collected UI actions once. Returns whether the current input frame
    /// was consumed by a control, and whether the application should keep running.
    pub fn apply_ui(&mut self, actions: Actions) -> (bool, bool) {
        let consumed = actions.0.iter().any(|action| match action {
            Action::Game(action) => action.consumes_input(),
            _ => true,
        });
        self.focus_canvas = consumed;
        for action in actions.0 {
            match action {
                Action::Game(action) => {
                    if let Some(game) = &mut self.active {
                        game.action(action);
                    }
                }
                Action::Back => self.active = None,
                Action::Quit => return (true, false),
            }
        }
        (consumed, true)
    }
}
