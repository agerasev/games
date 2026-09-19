//! Run with `cargo test --test rendering -- --ignored`; requires a GPU adapter.
use wgame::gfx::{Graphics, Offscreen, prelude::*};
fn graphics() -> Graphics {
    futures::executor::block_on(async {
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("GPU adapter required (Mesa lavapipe works)");
        eprintln!("Adapter: {:?}", adapter.get_info());
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                ..Default::default()
            })
            .await
            .unwrap();
        Graphics::new(adapter, device, queue, wgpu::TextureFormat::Rgba8Unorm)
    })
}
fn pixels(target: &mut Offscreen) -> Vec<u8> {
    let (width, height) = target.size();
    let stride = (width * 4).div_ceil(256) * 256;
    let buffer = target
        .state()
        .device()
        .create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (stride * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
    let texture = target.texture().clone();
    target.encoder().copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    target.submit();
    let (tx, rx) = std::sync::mpsc::channel();
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
    target
        .state()
        .device()
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(30)),
        })
        .unwrap();
    rx.recv().unwrap().unwrap();
    let data = buffer
        .slice(..)
        .get_mapped_range()
        .expect("readback buffer must be mapped");
    let pixels = data
        .chunks(stride as usize)
        .flat_map(|row| row[..width as usize * 4].iter().copied())
        .collect();
    drop(data);
    buffer.unmap();
    pixels
}

use wgame::{
    Library,
    canvas::{CanvasInput, Event, Key},
    gfx::types::color,
    glam::{Affine2, Vec2},
};
use yarik_games::{
    App,
    draw::{Assets, Painter},
    games::GameId,
};

fn draw(app: &App, lib: &Library, assets: &Assets, physical: (u32, u32), scale: f32) -> Vec<u8> {
    let mut target = Offscreen::new(lib.state(), physical);
    let logical = Vec2::new(physical.0 as f32, physical.1 as f32) / scale;
    let mut painter = Painter::new(lib, assets, logical);
    app.draw(&mut painter);
    let camera = target
        .physical_camera()
        .transform(Affine2::from_scale(Vec2::splat(scale)));
    target.clear(color::BLACK);
    target.render_iter(&camera, painter.scene.iter());
    pixels(&mut target)
}
fn save(name: &str, (width, height): (u32, u32), pixels: &[u8]) {
    if let Some(dir) = std::env::var_os("GAMES_RENDER_OUTPUT") {
        use std::io::Write;
        let mut file =
            std::fs::File::create(std::path::Path::new(&dir).join(format!("{name}.ppm"))).unwrap();
        write!(file, "P6\n{width} {height}\n255\n").unwrap();
        for pixel in pixels.as_chunks::<4>().0 {
            file.write_all(&pixel[..3]).unwrap();
        }
    }
}
fn event_input(event: Event) -> CanvasInput {
    let mut input = CanvasInput::default();
    input.events.push(event);
    input
}
fn press(key: Key) -> CanvasInput {
    event_input(Event::Key {
        key,
        pressed: true,
        repeat: false,
    })
}

fn click_control(app: &mut App, label: &str, size: Vec2) {
    use wgame_egui::egui;
    let context = egui::Context::default();
    let mut pos = None;
    for step in 0..4 {
        let mut actions = yarik_games::ui::Actions::default();
        let events = if step >= 2 {
            let pos = pos.expect("control must be drawn before clicking");
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: step == 2,
                    modifiers: Default::default(),
                },
            ]
        } else {
            Vec::new()
        };
        let output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(size.x, size.y),
                )),
                events,
                ..Default::default()
            },
            |ui| {
                let layout = app.ui(ui, |ui| {
                    ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag())
                });
                actions.collect(layout.actions);
            },
        );
        for shape in &output.shapes {
            if let egui::Shape::Text(text) = &shape.shape
                && text.galley.job.text == label
            {
                pos = Some(text.galley.rect.translate(text.pos.to_vec2()).center());
            }
        }
        output.drop_without_applying_deltas();
        app.apply_ui(actions);
    }
}

#[test]
#[ignore = "requires a GPU adapter (Mesa lavapipe works)"]
fn games_render_after_navigation_resize_and_dpi_changes() {
    let gfx = graphics();
    let lib = Library::new(&gfx);
    let mut assets = Assets::new(&lib).unwrap();
    for (physical, scale) in [
        ((960, 640), 1.0),
        ((320, 640), 1.0),
        ((640, 360), 1.0),
        ((1280, 960), 2.0),
    ] {
        assets.set_scale_factor(f64::from(scale));
        let logical = Vec2::new(physical.0 as f32, physical.1 as f32) / scale;
        let mut app = App::new(None);
        let menu = draw(&app, &lib, &assets, physical, scale);
        save(&format!("menu-{}", physical.0), physical, &menu);
        for (i, id) in GameId::ALL.into_iter().enumerate() {
            app.update(
                &press(Key::Character(char::from(b'1' + i as u8))),
                0.0,
                logical,
            );
            assert_eq!(app.active_id(), Some(id));
            let pixels = draw(&app, &lib, &assets, physical, scale);
            let rgb = pixels.as_chunks::<4>().0;
            assert!(
                rgb.iter()
                    .filter(|p| p[0].abs_diff(p[1]) > 30 || p[1].abs_diff(p[2]) > 30)
                    .count()
                    > 100,
                "{} has no colored sprites/letters",
                id.slug()
            );
            assert!(
                matches!(id, GameId::Mouse | GameId::Letters)
                    || rgb
                        .iter()
                        .filter(|p| p[0] > 180 && p[1] > 180 && p[2] > 180)
                        .count()
                        > 100,
                "{} has no text",
                id.slug()
            );
            assert_ne!(menu, pixels);
            save(&format!("{}-{}", id.slug(), physical.0), physical, &pixels);
            if id == GameId::Parking {
                let mut held = wgame::canvas::InputState::default();
                held.push(Event::Focused(true));
                held.push(Event::Key {
                    key: Key::Character('w'),
                    pressed: true,
                    repeat: false,
                });
                for _ in 0..45 {
                    app.update(held.input(), 1.0 / 60.0, logical);
                }
                let driving = draw(&app, &lib, &assets, physical, scale);
                assert_ne!(driving, pixels, "acceleration must move the car");
                save(
                    &format!("parking-driving-{}", physical.0),
                    physical,
                    &driving,
                );
            } else if id == GameId::MoonLander {
                let mut held = wgame::canvas::InputState::default();
                held.push(Event::Focused(true));
                held.push(Event::Key {
                    key: Key::Space,
                    pressed: true,
                    repeat: false,
                });
                for _ in 0..45 {
                    app.update(held.input(), 1.0 / 60.0, logical);
                }
                let flying = draw(&app, &lib, &assets, physical, scale);
                assert_ne!(
                    flying, pixels,
                    "thrust must move the craft and draw exhaust"
                );
                save(&format!("lander-flying-{}", physical.0), physical, &flying);
            } else if id == GameId::Letters {
                let mut previous = pixels;
                for c in ['2', '3', '0', '`'] {
                    match c {
                        '2' => click_control(&mut app, "English", logical),
                        '3' => click_control(&mut app, "Ελληνικά", logical),
                        '0' => click_control(&mut app, "123", logical),
                        _ => {
                            app.update(&press(Key::Character(c)), 0.0, logical);
                        }
                    }
                    let next = draw(&app, &lib, &assets, physical, scale);
                    assert_ne!(next, previous, "alphabet/font switch must change pixels");
                    save(&format!("letters-{c}-{}", physical.0), physical, &next);
                    previous = next;
                }
            } else if id == GameId::Apples {
                click_control(&mut app, "До 100", logical);
                for c in "100".chars() {
                    app.update(&press(Key::Character(c)), 0.0, logical);
                }
                let next = draw(&app, &lib, &assets, physical, scale);
                assert_ne!(next, pixels);
                save(&format!("apples-100-{}", physical.0), physical, &next);
            } else if id == GameId::Puzzle2048 {
                let mut previous = pixels;
                for c in ['3', '6', 'f'] {
                    app.update(&press(Key::Character(c)), 0.0, logical);
                    let next = draw(&app, &lib, &assets, physical, scale);
                    assert_ne!(next, previous, "2048 settings must change the board");
                    save(&format!("2048-{c}-{}", physical.0), physical, &next);
                    previous = next;
                }
                // Opposite directions guarantee movement regardless of initial spawn.
                for key in [Key::ArrowLeft, Key::ArrowRight] {
                    app.update(&press(key), 0.3, logical);
                }
                app.update(&CanvasInput::default(), 0.06, logical);
                let sliding = draw(&app, &lib, &assets, physical, scale);
                app.update(&CanvasInput::default(), 0.13, logical);
                let popping = draw(&app, &lib, &assets, physical, scale);
                app.update(&CanvasInput::default(), 0.2, logical);
                let settled = draw(&app, &lib, &assets, physical, scale);
                assert_ne!(sliding, popping, "tile movement/spawning must animate");
                assert_ne!(popping, settled, "spawn animation must settle");
                save(&format!("2048-play-{}", physical.0), physical, &settled);
                app.update(&press(Key::Character('u')), 0.0, logical);
                assert_ne!(draw(&app, &lib, &assets, physical, scale), settled);
            }
            app.update(&press(Key::Escape), 0.0, logical);
            assert_eq!(app.active_id(), None);
        }
    }
}
