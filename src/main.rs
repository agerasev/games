mod animation;
mod objects;

use objects::{Character, Object, Personality, Tree, TreeSpecies};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use rand_distr::{Normal, Poisson, Uniform};
use std::time::Duration;
use wgame::{
    Library, Result, Window, WindowHost,
    app::time::Instant,
    canvas::{CanvasInput, Event, Key},
    glam::{Affine2, Vec2},
    prelude::*,
};

const TILT: f32 = 0.6667;

fn motion(input: &CanvasInput) -> Vec2 {
    let held = |arrow, letter| input.key_down(arrow) || input.key_down(Key::Character(letter));
    Vec2::new(
        i32::from(held(Key::ArrowRight, 'd')) as f32 - i32::from(held(Key::ArrowLeft, 'a')) as f32,
        i32::from(held(Key::ArrowDown, 's')) as f32 - i32::from(held(Key::ArrowUp, 'w')) as f32,
    )
}

#[wgame::window(title = "Run — Бег по лесу", logical_size = (1280.0, 720.0), resizable = true, vsync = true)]
async fn main(window: Window<'_>) -> Result<()> {
    run(window).await
}

async fn run(mut host: impl WindowHost) -> Result<()> {
    let lib = Library::new(host.graphics());
    let species = TreeSpecies::load(&lib)?;
    let person = Personality::new(&lib)?;
    let mut rng = SmallRng::seed_from_u64(0xdeadbeef);
    let normal = Normal::new(0.0, 10.0).unwrap();
    let growth = Uniform::new(1.0, 3.0).unwrap();
    let mut trees: Vec<_> = (0..rng.sample(Poisson::new(64_f32).unwrap()).round() as usize)
        .map(|_| Tree {
            species: &species,
            pos: Vec2::new(rng.sample(normal), rng.sample(normal)),
            growth: rng.sample(growth),
        })
        .collect();
    trees.sort_by(|a, b| a.pos.y.total_cmp(&b.pos.y));
    let mut player = Character::new(&person, Vec2::ZERO, Vec2::Y);
    let mut zoom = 0.1_f32;
    let mut last = Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    #[cfg(target_arch = "wasm32")]
    let smoke = false;
    let mut frames = 0;

    while let Some(mut frame) = host.next_frame().await? {
        if frame.input().events.iter().any(|event| {
            matches!(
                event,
                Event::Key {
                    key: Key::Escape,
                    pressed: true,
                    ..
                }
            )
        }) {
            frame.discard();
            break;
        }
        let now = Instant::now();
        let reset_clock = frame
            .input()
            .events
            .iter()
            .any(|event| matches!(event, Event::Cancelled | Event::Focused(_)));
        let dt = if reset_clock || !frame.input().window_focused || !frame.visible() {
            Duration::ZERO
        } else {
            (now - last).min(Duration::from_millis(100))
        };
        last = now;
        player.step(motion(frame.input()), dt);
        let scroll: f32 = frame
            .input()
            .events
            .iter()
            .filter_map(|event| match event {
                Event::Scroll(delta) => Some(delta.y / 40.0),
                _ => None,
            })
            .sum();
        zoom = (zoom * (0.2 * scroll.clamp(-1.0, 1.0)).exp()).clamp(0.01, 10.0);

        let (width, height) = frame.logical_size();
        let viewport = Vec2::new(width as f32, height as f32);
        // Match the original fixed, centered camera; world Y points down.
        let camera = frame
            .logical_camera()
            .transform(Affine2::from_scale_angle_translation(
                Vec2::splat(0.5 * viewport.min_element() * zoom),
                0.0,
                0.5 * viewport,
            ));
        frame.clear(wgame::rgb::Rgb::new(0.0_f32, 82.0 / 255.0, 44.0 / 255.0));
        let visible = frame.visible();
        let mut scene = frame.scene();
        scene.camera = camera;
        if visible {
            // Give both layers of each object their own order, so batching cannot
            // place a foreground trunk behind the player's limbs.
            let split = trees.partition_point(|tree| tree.pos.y < player.pos().y);
            let objects = trees[..split]
                .iter()
                .map(|t| t as &dyn Object)
                .chain([&player as &dyn Object])
                .chain(trees[split..].iter().map(|t| t as &dyn Object));
            for (index, object) in objects.enumerate() {
                object.draw(&lib, &mut scene, 2 * index as i32);
            }
        }
        scene.render();
        frame.present();
        frames += 1;
        if smoke && frames == 12 {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgame::canvas::{InputState, Modifiers};

    #[test]
    fn movement_handles_aliases_opposites_and_focus_loss() {
        let mut input = InputState::default();
        for key in [Key::ArrowUp, Key::Character('w'), Key::Character('d')] {
            input.push(Event::Key {
                key,
                pressed: true,
                repeat: false,
            });
        }
        assert_eq!(
            motion(&input.finish(true, true, Modifiers::default())),
            Vec2::new(1.0, -1.0)
        );
        input.push(Event::Key {
            key: Key::ArrowDown,
            pressed: true,
            repeat: false,
        });
        assert_eq!(motion(input.input()), Vec2::X);
        input.push(Event::Focused(false));
        assert_eq!(motion(input.input()), Vec2::ZERO);
    }
}
