//! Egui owns controls; the canvas owns game input and drawing. Layout is read-only
//! with respect to game state. Collect and deduplicate actions across passes, then apply them
//! once, after `EguiWindow::next_frame`, before updating the simulation.
use crate::{App, games::GameId};
use wgame_egui::egui;

#[derive(PartialEq)]
enum Action {
    Launch(GameId),
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
                    ui.label("?").on_hover_text(id.hint());
                } else {
                    ui.heading("Игры");
                    if ui.button("Выход / Esc").clicked() {
                        actions.0.push(Action::Quit);
                    }
                }
            });
        });
        let response = egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                if self.active.is_none() {
                    let size = ui.available_size();
                    ui.allocate_ui(egui::vec2(size.x, (size.y - 1.0).max(0.0)), |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add_space(16.0);
                            ui.label("Выберите игру / 1–4 / стрелки и Enter");
                            ui.add_space(12.0);
                            let columns = if ui.available_width() >= 600.0 { 2 } else { 1 };
                            let width = (ui.available_width() / columns as f32 - 8.0).max(1.0);
                            egui::Grid::new("games")
                                .spacing(egui::vec2(8.0, 8.0))
                                .show(ui, |ui| {
                                    for (index, id) in GameId::ALL.into_iter().enumerate() {
                                        if ui
                                            .add_sized(
                                                [width, 84.0],
                                                egui::Button::selectable(
                                                    self.selection == index,
                                                    format!("{}. {}", index + 1, id.title()),
                                                )
                                                .wrap(),
                                            )
                                            .clicked()
                                        {
                                            actions.0.push(Action::Launch(id));
                                        }
                                        if (index + 1) % columns == 0 {
                                            ui.end_row();
                                        }
                                    }
                                });
                        });
                    });
                }
                canvas(ui)
            })
            .inner;
        // Return focus after a control action; its triggering input is consumed
        // by apply_ui, so it cannot also perform a move in the new round.
        if !actions.0.is_empty() || ui.memory(|m| m.focused().is_none()) {
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
        let consumed = !actions.0.is_empty();
        for action in actions.0 {
            match action {
                Action::Launch(id) => {
                    self.selection = GameId::ALL.iter().position(|&item| item == id).unwrap();
                    self.active = Some(crate::games::Game::new(id));
                }
                Action::Back => self.active = None,
                Action::Quit => return (true, false),
            }
        }
        (consumed, true)
    }
}
