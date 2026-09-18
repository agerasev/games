use super::*;
use crate::games::{Game, GameId};
use wgame::{
    canvas::{Button, CanvasInput, Event, Key},
    glam::Vec2,
};

struct Harness {
    context: egui::Context,
    app: App,
    size: egui::Vec2,
    shapes: Vec<egui::epaint::ClippedShape>,
    canvas: egui::Rect,
}
impl Harness {
    fn new(size: egui::Vec2) -> Self {
        let context = egui::Context::default();
        configure(&context);
        context.all_styles_mut(|s| s.animation_time = 0.0);
        let mut this = Self {
            context,
            app: App::new(None),
            size,
            shapes: Vec::new(),
            canvas: egui::Rect::NOTHING,
        };
        this.frame(Vec::new(), false);
        this.frame(Vec::new(), false);
        this
    }
    fn frame(&mut self, events: Vec<egui::Event>, multipass: bool) -> usize {
        let mut actions = Actions::default();
        let output = self.context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, self.size)),
                events,
                ..Default::default()
            },
            |ui| {
                let layout = self.app.ui(ui, |ui| {
                    let (_, rect) = ui.allocate_space(ui.available_size());
                    ui.interact(
                        rect,
                        egui::Id::new("test-canvas"),
                        egui::Sense::click_and_drag(),
                    )
                });
                self.canvas = layout.canvas.rect;
                actions.collect(layout.actions);
                if multipass && ui.ctx().current_pass_index() == 0 {
                    ui.ctx().request_discard("exercise repeated layout");
                }
            },
        );
        self.shapes = output.shapes.clone();
        output.drop_without_applying_deltas();
        let count = actions.0.len();
        assert!(self.app.apply_ui(actions).1);
        count
    }
    fn text(&self, text: &str) -> egui::Rect {
        fn find(shape: &egui::Shape, text: &str) -> Option<egui::Rect> {
            match shape {
                egui::Shape::Text(t) if t.galley.job.text == text => {
                    Some(t.galley.rect.translate(t.pos.to_vec2()))
                }
                egui::Shape::Vec(shapes) => shapes.iter().find_map(|s| find(s, text)),
                _ => None,
            }
        }
        self.shapes
            .iter()
            .find_map(|s| find(&s.shape, text))
            .unwrap_or_else(|| {
                panic!(
                    "missing text {text} at {:?}: {:?}",
                    self.size,
                    self.shapes
                        .iter()
                        .filter_map(|s| if let egui::Shape::Text(t) = &s.shape {
                            Some(t.galley.job.text.as_str())
                        } else {
                            None
                        })
                        .collect::<Vec<_>>()
                )
            })
    }
    fn click(&mut self, text: &str, multipass: bool) -> usize {
        let pos = self.text(text).center();
        self.frame(
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
            false,
        );
        self.frame(
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
            multipass,
        )
    }
}

#[test]
fn preview_tiles_and_egui_back_button_work_after_resize() {
    let mut h = Harness::new(egui::vec2(1280.0, 720.0));
    for size in [egui::vec2(1280.0, 720.0), egui::vec2(320.0, 640.0)] {
        h.size = size;
        for (index, id) in GameId::ALL.into_iter().enumerate() {
            h.frame(Vec::new(), false);
            let canvas_size = Vec2::new(h.canvas.width(), h.canvas.height());
            let tile =
                crate::layout::grid(canvas_size, GameId::ALL.len(), crate::MENU_ASPECT)[index];
            let mut input = CanvasInput::default();
            input.events.push(Event::Button {
                button: Button::Primary,
                pressed: true,
                position: Vec2::from_array(tile.center().to_array()),
            });
            h.app.update(&input, 0.0, canvas_size);
            assert_eq!(h.app.active_id(), Some(id));
            h.frame(Vec::new(), false);
            assert!(h.canvas.height() > 200.0);
            assert_eq!(h.click("Меню / Esc", true), 1);
            assert_eq!(h.app.active_id(), None);
        }
    }
}

impl Harness {
    fn play(&mut self, key: Key) {
        let mut input = CanvasInput::default();
        input.events.push(Event::Key {
            key,
            pressed: true,
            repeat: false,
        });
        self.app.update(
            &input,
            0.3,
            Vec2::new(self.canvas.width(), self.canvas.height()),
        );
        self.frame(Vec::new(), false);
    }
    fn puzzle(&self) -> &crate::games::puzzle2048::model::Board {
        let Some(Game::Puzzle2048(game)) = &self.app.active else {
            panic!("expected 2048")
        };
        game.board()
    }
}

#[test]
fn puzzle_controls_preserve_the_round_and_apply_undo_once_across_passes() {
    for size in [
        egui::vec2(1280.0, 720.0),
        egui::vec2(320.0, 640.0),
        egui::vec2(640.0, 360.0),
    ] {
        let mut h = Harness::new(size);
        h.app = App::new(Some(GameId::Puzzle2048));
        h.frame(Vec::new(), false);
        h.play(Key::ArrowLeft);
        h.play(Key::ArrowRight);
        h.play(Key::ArrowDown);
        let previous = h.puzzle().cells().to_vec();
        let moves = h.puzzle().moves();
        assert!(moves >= 2);
        h.click("Настройки", false);
        // Let the collapsing section reach its full height.
        h.frame(Vec::new(), false);
        h.frame(Vec::new(), false);
        let bounds = h.text("Только 2");
        assert!(egui::Rect::from_min_size(egui::Pos2::ZERO, size).contains_rect(bounds));
        assert_eq!(h.click("Только 2", true), 1);
        assert_eq!(h.puzzle().cells(), previous);
        assert_eq!(h.puzzle().moves(), moves);
        assert!(h.puzzle().can_undo());
        assert_eq!(
            h.puzzle().settings().spawn,
            crate::games::puzzle2048::model::Spawn::SmallOnly
        );
        h.frame(Vec::new(), false);
        assert_eq!(h.click("Отмена / U", true), 1);
        assert_eq!(h.puzzle().moves(), moves - 1);
        h.frame(Vec::new(), false);
        assert_eq!(h.click("6×6", true), 1);
        assert_eq!(h.puzzle().settings().side, 6);
        assert_eq!(h.puzzle().moves(), 0);
        assert!(!h.puzzle().can_undo());
        assert!(h.canvas.width() > 150.0 && h.canvas.height() > 100.0);
    }
}
