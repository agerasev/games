use super::*;
fn input(events: Vec<Event>) -> CanvasInput {
    let mut input = CanvasInput::default();
    input.events = events;
    input
}
fn key(key: Key) -> Event {
    Event::Key {
        key,
        pressed: true,
        repeat: false,
    }
}
fn pointer(pressed: bool, position: Vec2) -> Event {
    Event::Button {
        button: Button::Primary,
        pressed,
        position,
    }
}
const SIZE: Vec2 = Vec2::new(400.0, 720.0);
#[test]
fn default_game_spawns_both_values_without_toggling_settings() {
    let mut large = 0;
    let mut total = 0;
    let mut initial_large = 0;
    for seed in 0..128 {
        let mut game = Game::with_seed(seed);
        assert_eq!(game.board.settings().spawn, Spawn::Mixed);
        initial_large += game.board.cells().iter().filter(|&&v| v == 4).count();
        for keycode in [
            Key::ArrowLeft,
            Key::ArrowDown,
            Key::ArrowRight,
            Key::ArrowUp,
        ]
        .into_iter()
        .cycle()
        .take(24)
        {
            game.update(&input(vec![key(keycode)]), SLIDE + POP, SIZE);
            if let Some(animation) = &game.animation {
                // Inspect only the new tile, excluding 4s created by merges.
                let value = game.board.cells()[animation.turn.spawned];
                assert!(matches!(value, 2 | 4));
                large += usize::from(value == 4);
                total += 1;
            }
        }
    }
    assert!(initial_large > 0);
    assert!(total > 1000);
    // Fixed seeds keep the test repeatable; generous bounds check the 90/10
    // policy without tying it to an exact sequence from the RNG implementation.
    assert!(large > total / 20 && large < total / 5, "{large}/{total}");
}

#[test]
fn spawn_controls_preserve_animation_buffered_moves_and_undo() {
    let mut game = Game::with_seed(3);
    game.update(
        &input(vec![key(Key::ArrowRight), key(Key::ArrowDown)]),
        0.0,
        SIZE,
    );
    let before = game.board.cells().to_vec();
    let score = game.board.score();
    let moves = game.board.moves();
    let pending = game.pending.clone();
    let elapsed = game.animation.as_ref().unwrap().elapsed;
    game.update(&input(vec![key(Key::Character('t'))]), 0.0, SIZE);
    assert_eq!(game.board.settings().spawn, Spawn::SmallOnly);
    assert_eq!(game.board.cells(), before);
    assert_eq!(game.board.score(), score);
    assert_eq!(game.board.moves(), moves);
    assert!(game.board.can_undo());
    assert_eq!(game.pending, pending);
    assert_eq!(game.animation.as_ref().unwrap().elapsed, elapsed);

    game.action(Action::Spawn(Spawn::Mixed));
    assert_eq!(game.board.settings().spawn, Spawn::Mixed);
    assert_eq!(game.board.cells(), before);
    assert_eq!(game.board.score(), score);
    assert_eq!(game.board.moves(), moves);
    assert!(game.board.can_undo());
    assert_eq!(game.pending, pending);
    assert_eq!(game.animation.as_ref().unwrap().elapsed, elapsed);
}

#[test]
fn swipe_matches_keyboard_and_cancellation_aborts_stale_input() {
    let mut swipe = Game::with_seed(3);
    let mut keyboard = Game::with_seed(3);
    let area = view::Layout::new(SIZE).board;
    let center = Vec2::from_array(area.center().to_array());
    swipe.update(
        &input(vec![
            pointer(true, center),
            pointer(false, center + Vec2::X * 80.0),
        ]),
        0.0,
        SIZE,
    );
    keyboard.update(&input(vec![key(Key::ArrowRight)]), 0.0, SIZE);
    assert_eq!(swipe.board.cells(), keyboard.board.cells());
    assert_eq!(swipe.board.moves(), 1);
    let previous = swipe.board.cells().to_vec();
    swipe.update(
        &input(vec![
            pointer(true, center),
            key(Key::ArrowLeft),
            Event::Cancelled,
            pointer(false, center - Vec2::X * 80.0),
        ]),
        1.0,
        SIZE,
    );
    assert_eq!(swipe.board.cells(), previous);
    assert!(swipe.pending.is_empty());
}
#[test]
fn undo_during_animation_clears_buffered_moves_and_settings_restart_cleanly() {
    let mut game = Game::with_seed(3);
    let original = game.board.cells().to_vec();
    game.update(
        &input(vec![key(Key::ArrowRight), key(Key::ArrowDown)]),
        0.0,
        SIZE,
    );
    assert!(game.animation.is_some());
    assert!(!game.pending.is_empty());
    game.update(&input(vec![key(Key::Character('u'))]), 0.0, SIZE);
    assert_eq!(game.board.cells(), original);
    assert!(game.animation.is_none());
    assert!(game.pending.is_empty());
    game.update(
        &input(vec![key(Key::Character('6')), key(Key::Character('f'))]),
        0.0,
        SIZE,
    );
    let initial = game.board.cells().to_vec();
    game.update(&input(vec![key(Key::Character('t'))]), 0.0, SIZE);
    assert_eq!(
        game.board.settings(),
        Settings {
            side: 6,
            rule: Rule::Fibonacci,
            spawn: Spawn::SmallOnly
        }
    );
    assert_eq!(game.board.cells(), initial);
    assert_eq!(game.board.moves(), 0);
    assert!(!game.board.can_undo());
    game.update(&input(vec![key(Key::Character('r'))]), 0.0, SIZE);
    assert_eq!(game.board.cells().iter().filter(|&&v| v == 1).count(), 2);
}
#[test]
fn board_fits_the_canvas_after_resize() {
    for size in [
        Vec2::new(320.0, 600.0),
        SIZE,
        Vec2::new(1280.0, 680.0),
        Vec2::new(640.0, 440.0),
        Vec2::new(640.0, 320.0),
        Vec2::new(568.0, 280.0),
    ] {
        let layout = view::Layout::new(size);
        assert!(layout.board.min_x() >= 0.0 && layout.board.max_x() <= size.x);
        assert!(layout.board.min_y() >= 0.0 && layout.board.max_y() <= size.y);
        assert!(layout.board.size.width >= 150.0);
    }
}
