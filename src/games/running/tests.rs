use super::*;
use wgame::canvas::{InputState, Modifiers};

fn press(input: &mut InputState, key: Key) {
    input.push(Event::Key {
        key,
        pressed: true,
        repeat: false,
    });
}

#[test]
fn movement_handles_aliases_opposites_and_focus_loss() {
    let mut input = InputState::default();
    input.push(Event::Focused(true));
    input.finish(true, true, Modifiers::default());
    for key in [Key::ArrowUp, Key::Character('w'), Key::Character('d')] {
        press(&mut input, key);
    }
    assert_eq!(
        motion(&input.finish(true, true, Modifiers::default())),
        Vec2::new(1.0, -1.0)
    );
    press(&mut input, Key::ArrowDown);
    assert_eq!(motion(input.input()), Vec2::X);
    input.push(Event::Focused(false));
    assert_eq!(motion(input.input()), Vec2::ZERO);
}

#[test]
fn movement_is_normalized_bounded_in_time_and_pauses_on_focus_loss() {
    let mut input = InputState::default();
    input.push(Event::Focused(true));
    input.finish(true, true, Modifiers::default());
    press(&mut input, Key::Character('d'));
    let held = input.finish(true, true, Modifiers::default());
    let mut straight = Game::new();
    straight.update(&held, 0.1);
    press(&mut input, Key::Character('s'));
    let held = input.finish(true, true, Modifiers::default());
    let mut diagonal = Game::new();
    diagonal.update(&held, 10.0);
    assert!((straight.player.pos().length() - diagonal.player.pos().length()).abs() < 1e-6);
    assert!(straight.player.pos().length() > 0.2);
    let pos = diagonal.player.pos();
    input.push(Event::Focused(false));
    diagonal.update(&input.finish(false, false, Modifiers::default()), 1.0);
    assert_eq!(diagonal.player.pos(), pos);
}

#[test]
fn zoom_is_bounded_and_discards_cancelled_scroll() {
    let mut game = Game::new();
    let mut input = CanvasInput::default();
    input.window_focused = true;
    input.events.push(Event::Scroll(Vec2::new(0.0, 1e6)));
    for _ in 0..100 {
        game.update(&input, 0.0);
    }
    assert_eq!(game.zoom, 10.0);
    input.events.push(Event::Cancelled);
    input.events.push(Event::Scroll(Vec2::new(0.0, -1e6)));
    for _ in 0..100 {
        game.update(&input, 0.0);
    }
    assert_eq!(game.zoom, 0.01);
    input.events.push(Event::Cancelled);
    game.update(&input, 0.0);
    assert_eq!(game.zoom, 0.01);
}
