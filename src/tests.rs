use super::*;
use wgame::canvas::{InputState, Modifiers};

fn input(events: impl IntoIterator<Item = Event>) -> CanvasInput {
    let mut state = InputState::default();
    for event in events {
        state.push(event);
    }
    state.finish(true, true, Modifiers::default())
}
fn key(key: Key) -> Event {
    Event::Key {
        key,
        pressed: true,
        repeat: false,
    }
}

#[test]
fn every_game_launches_and_returns_to_menu() {
    let size = Vec2::new(1280.0, 720.0);
    let mut app = App::new(None);
    for (i, id) in GameId::ALL.into_iter().enumerate() {
        assert!(app.update(
            &input([key(Key::Character(char::from(b'1' + i as u8)))]),
            0.0,
            size
        ));
        assert_eq!(app.active_id(), Some(id));
        // Holding Escape cannot close both the game and the menu.
        let escape = input([key(Key::Escape)]);
        assert!(app.update(&escape, 0.0, size));
        assert_eq!(app.active_id(), None);
        assert!(app.update(
            &input([Event::Key {
                key: Key::Escape,
                pressed: true,
                repeat: true
            }]),
            0.0,
            size
        ));
    }
    assert!(!app.update(&input([key(Key::Escape)]), 0.0, size));
}

#[test]
fn cancellation_does_not_activate_a_stale_click_or_key() {
    let mut app = App::new(None);
    let events = [
        Event::Button {
            button: Button::Primary,
            pressed: true,
            position: Vec2::splat(100.0),
        },
        key(Key::Character('2')),
        Event::Cancelled,
    ];
    assert!(app.update(&input(events), 0.0, Vec2::new(800.0, 600.0)));
    assert_eq!(app.active_id(), None);
}

#[test]
fn static_games_sleep_and_timed_work_stops_requesting_frames() {
    use std::time::Duration;
    for id in [
        None,
        Some(GameId::Letters),
        Some(GameId::Apples),
        Some(GameId::Puzzle2048),
        Some(GameId::MoonLander),
    ] {
        assert_eq!(App::new(id).repaint_after(), None);
    }
    assert_eq!(
        App::new(Some(GameId::Mouse)).repaint_after(),
        Some(Duration::ZERO)
    );
    let mut puzzle = App::new(Some(GameId::Puzzle2048));
    let size = Vec2::new(640.0, 480.0);
    for key in [Key::ArrowLeft, Key::ArrowRight] {
        let mut input = CanvasInput::default();
        input.events.push(Event::Key {
            key,
            pressed: true,
            repeat: false,
        });
        puzzle.update(&input, 0.0, size);
    }
    assert_eq!(puzzle.repaint_after(), Some(Duration::ZERO));
    for _ in 0..60 {
        puzzle.update(&CanvasInput::default(), 1.0 / 60.0, size);
    }
    assert_eq!(puzzle.repaint_after(), None);

    let mut apples = App::new(Some(GameId::Apples));
    let mut digit = CanvasInput::default();
    digit.events.push(Event::Key {
        key: Key::Character('1'),
        pressed: true,
        repeat: false,
    });
    apples.update(&digit, 0.0, size);
    assert_eq!(apples.repaint_after(), Some(Duration::from_secs(4)));
    apples.advance_timers(1.0); // UI controls may consume input, but not elapsed time.
    assert_eq!(apples.repaint_after(), Some(Duration::from_secs(3)));
    apples.update(&CanvasInput::default(), 3.0, size);
    assert_eq!(apples.repaint_after(), None);
}
