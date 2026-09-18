use std::{cell::RefCell, rc::Rc};
use wgame::{
    Library, Result, Window, WindowHost, app::time::Instant, canvas::Event, gfx::types::color,
    glam::Vec2, prelude::*,
};
use wgame_egui::EguiWindow;
use yarik_games::{
    App,
    draw::{Assets, Painter},
    games::GameId,
};

#[wgame::window(title = "Games", logical_size = (1280.0, 720.0), resizable = true, vsync = true)]
async fn main(window: Window<'_>) -> Result<()> {
    let (start, smoke) = options()?;
    let app = Rc::new(RefCell::new(App::new(start)));
    let actions = Rc::new(RefCell::new(yarik_games::ui::Actions::default()));
    let mut host = EguiWindow::new(window, {
        let app = app.clone();
        let actions = actions.clone();
        move |ui, canvas: &wgame_egui::Canvas| {
            let layout = app.borrow().ui(ui, |ui| canvas.show(ui));
            actions.borrow_mut().collect(layout.actions);
            layout.canvas
        }
    });
    yarik_games::ui::configure(host.context());
    run(&mut host, &app, &actions, smoke).await
}

fn options() -> Result<(Option<GameId>, bool)> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut start = None;
        let mut smoke = false;
        for arg in std::env::args().skip(1) {
            if arg == "--smoke" {
                smoke = true;
            } else {
                start = Some(GameId::parse(&arg).ok_or_else(|| {
                    wgame::Error::msg(format!(
                        "Unknown game {arg:?}; choose apples, letters, mouse, 2048, or lander"
                    ))
                })?);
            }
        }
        Ok((start, smoke))
    }
    #[cfg(target_arch = "wasm32")]
    {
        Ok((None, false))
    }
}

async fn run(
    host: &mut impl WindowHost,
    app: &RefCell<App>,
    actions: &RefCell<yarik_games::ui::Actions>,
    smoke: bool,
) -> Result<()> {
    let lib = Library::new(host.graphics());
    let mut assets = Assets::new(&lib)?;
    let mut last = Instant::now();
    let mut frames = 0;
    while let Some(mut frame) = host.next_frame().await? {
        let now = Instant::now();
        let reset = frame
            .input()
            .events
            .iter()
            .any(|event| matches!(event, Event::Cancelled | Event::Focused(_)));
        let dt = if reset || !frame.visible() || !frame.input().window_focused {
            0.0
        } else {
            (now - last).as_secs_f32().min(0.04)
        };
        last = now;
        let (width, height) = frame.logical_size();
        let size = Vec2::new(width as f32, height as f32);
        let mut app = app.borrow_mut();
        let (consumed, running) = app.apply_ui(std::mem::take(&mut *actions.borrow_mut()));
        if !running || (!consumed && !app.update(frame.input(), dt, size)) {
            frame.discard();
            break;
        }
        assets.set_scale_factor(frame.scale_factor());
        frame.clear(color::BLACK);
        if frame.visible() {
            let mut painter = Painter::new(&lib, &assets, size);
            app.draw(&mut painter);
            let camera = frame.logical_camera();
            frame.render_iter(&camera, painter.scene.iter());
        }
        frame.present();
        frames += 1;
        if smoke && frames >= 12 {
            break;
        }
    }
    Ok(())
}
