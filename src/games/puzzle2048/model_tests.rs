use super::*;

fn board(rule: Rule, cells: &[u64]) -> Board {
    let mut board = Board::new(
        Settings {
            rule,
            ..Default::default()
        },
        17,
    );
    board.state.cells[..cells.len()].copy_from_slice(cells);
    board.state.cells[cells.len()..].fill(0);
    board
}
fn without_spawn(board: &Board, turn: &Turn) -> Vec<u64> {
    let mut cells = board.cells().to_vec();
    cells[turn.spawned] = 0;
    cells
}
#[test]
fn classic_merges_once_from_the_leading_edge_and_records_paths() {
    for (input, output, score) in [
        ([2, 2, 2, 2], [4, 4, 0, 0], 8),
        ([2, 2, 4, 0], [4, 4, 0, 0], 4),
        ([4, 0, 4, 4], [8, 4, 0, 0], 8),
        ([0, 2, 0, 4], [2, 4, 0, 0], 0),
    ] {
        let mut b = board(Rule::Classic, &input);
        let turn = b.step(Direction::Left).unwrap();
        assert_eq!(&without_spawn(&b, &turn)[..4], &output);
        assert_eq!(b.score(), score);
        assert_eq!(turn.motion.len(), input.iter().filter(|&&v| v != 0).count());
        for motion in turn.motion {
            assert_eq!(motion.value, input[motion.from]);
            assert!(motion.to <= motion.from);
        }
    }
}
#[test]
fn fibonacci_neighbors_merge_in_both_orders_but_never_chain() {
    for (a, b, result) in [
        (1, 1, Some(2)),
        (1, 2, Some(3)),
        (2, 1, Some(3)),
        (2, 3, Some(5)),
        (5, 3, Some(8)),
        (2, 2, None),
        (3, 3, None),
        (1, 3, None),
        (4, 5, None),
    ] {
        assert_eq!(Rule::Fibonacci.merge(a, b), result);
    }
    for (input, output) in [
        ([1, 1, 2, 3], [2, 5, 0, 0]),
        ([1, 2, 3, 5], [3, 8, 0, 0]),
        ([2, 2, 3, 0], [2, 5, 0, 0]),
    ] {
        let mut b = board(Rule::Fibonacci, &input);
        let t = b.step(Direction::Left).unwrap();
        assert_eq!(&without_spawn(&b, &t)[..4], &output);
    }
    assert_eq!(Rule::Classic.merge(1 << 63, 1 << 63), None);
    assert_eq!(Rule::Fibonacci.merge(u64::MAX, u64::MAX), None);
}
#[test]
fn all_directions_and_sizes_conserve_tiles_except_one_spawn() {
    for side in 3..=6 {
        for direction in [
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ] {
            for rule in [Rule::Classic, Rule::Fibonacci] {
                let mut b = Board::new(
                    Settings {
                        side,
                        rule,
                        spawn: Spawn::SmallOnly,
                    },
                    1,
                );
                b.state.cells.fill(0);
                let a = direction.index(side, 1, side - 1);
                let c = direction.index(side, 1, side - 2);
                b.state.cells[a] = rule.first();
                b.state.cells[c] = rule.first();
                let turn = b.step(direction).unwrap();
                let cells = without_spawn(&b, &turn);
                assert_eq!(cells[direction.index(side, 1, 0)], rule.first() * 2);
                assert_eq!(cells.iter().sum::<u64>(), rule.first() * 2);
                assert_eq!(b.cells().iter().sum::<u64>(), rule.first() * 3);
                assert_eq!(b.moves(), 1);
            }
        }
    }
}
#[test]
fn invalid_moves_do_not_spawn_or_consume_undo_and_replay_restores_randomness() {
    let mut b = board(Rule::Classic, &[2, 4, 0, 0]);
    let initial = b.cells().to_vec();
    assert!(b.step(Direction::Left).is_none());
    assert_eq!(initial, b.cells());
    assert!(!b.can_undo());
    b.step(Direction::Right).unwrap();
    let after = b.cells().to_vec();
    let score = b.score();
    assert!(b.undo());
    assert_eq!(b.cells(), initial);
    assert_eq!(b.moves(), 0);
    b.step(Direction::Right).unwrap();
    assert_eq!(b.cells(), after);
    assert_eq!(b.score(), score);
    assert!(b.undo());
    assert!(!b.undo());
}
#[test]
fn game_over_goal_and_reset_use_the_selected_rules() {
    let mut b = board(
        Rule::Classic,
        &[2, 4, 2, 4, 4, 2, 4, 2, 2, 4, 2, 4, 4, 2, 4, 2],
    );
    assert!(!b.can_move());
    b.state.cells[0] = 4;
    assert!(b.can_move());
    b.state.cells[0] = 2048;
    assert!(b.won());
    b.reset(Settings {
        side: 6,
        rule: Rule::Fibonacci,
        spawn: Spawn::SmallOnly,
    });
    assert_eq!(b.cells().len(), 36);
    assert_eq!(b.cells().iter().filter(|&&v| v == 1).count(), 2);
    assert_eq!(b.score(), 0);
    assert!(!b.won());
    assert!(!b.can_undo());
    b.state.cells.fill(2);
    assert!(!b.can_move());
    b.state.cells[1] = 3;
    assert!(b.can_move());
    b.state.cells[0] = 2584;
    assert!(b.won());
}
#[test]
fn spawn_modes_only_produce_their_allowed_values() {
    for rule in [Rule::Classic, Rule::Fibonacci] {
        for spawn in [Spawn::SmallOnly, Spawn::Mixed] {
            let mut b = Board::new(
                Settings {
                    side: 4,
                    rule,
                    spawn,
                },
                321,
            );
            let mut large = 0;
            for _ in 0..200 {
                b.reset(b.settings());
                for &value in b.cells().iter().filter(|&&v| v != 0) {
                    assert!(
                        value == rule.first()
                            || (spawn == Spawn::Mixed && value == rule.first() * 2)
                    );
                    large += usize::from(value > rule.first());
                }
            }
            assert_eq!(large > 0, spawn == Spawn::Mixed);
        }
    }
}
