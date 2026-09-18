use super::*;

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
            .unwrap_or_else(|| panic!("missing text {text}"))
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
fn launcher_and_back_buttons_work_after_resize_and_repeated_layout() {
    let mut h = Harness::new(egui::vec2(1280.0, 720.0));
    for size in [egui::vec2(1280.0, 720.0), egui::vec2(320.0, 640.0)] {
        h.size = size;
        h.frame(Vec::new(), false);
        h.frame(Vec::new(), false);
        for (index, id) in GameId::ALL.into_iter().enumerate() {
            let label = format!("{}. {}", index + 1, id.title());
            let bounds = h.text(&label);
            assert!(egui::Rect::from_min_size(egui::Pos2::ZERO, size).contains_rect(bounds));
            assert_eq!(h.click(&label, true), 1);
            assert_eq!(h.app.active_id(), Some(id));
            h.frame(Vec::new(), false);
            assert!(h.canvas.height() > 200.0);
            assert_eq!(h.click("Меню / Esc", true), 1);
            assert_eq!(h.app.active_id(), None);
            h.frame(Vec::new(), false);
        }
    }
}
