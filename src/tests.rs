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
fn pointer_launch_and_back_use_logical_coordinates_after_resize() {
    for size in [Vec2::new(1280.0, 720.0), Vec2::new(320.0, 640.0)] {
        let mut app = App::new(None);
        for (id, cell) in
            GameId::ALL
                .into_iter()
                .zip(grid(content_size(size), GameId::ALL.len(), 1.3))
        {
            let click = |pos| Event::Button {
                button: Button::Primary,
                pressed: true,
                position: pos,
            };
            app.update(
                &input([click(Vec2::from_array(cell.center().to_array()))]),
                0.0,
                size,
            );
            assert_eq!(app.active_id(), Some(id));
            app.update(&input([click(Vec2::new(40.0, size.y - 20.0))]), 0.0, size);
            assert_eq!(app.active_id(), None);
        }
    }
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
