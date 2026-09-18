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
