//! CPU-only rules. A successful move merges toward the leading edge, scores the
//! resulting values, then spawns exactly one tile. A blocked move changes nothing,
//! including random state and undo history. Undo restores the entire previous turn.
use rand::{Rng, SeedableRng, rngs::SmallRng};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rule {
    #[default]
    Classic,
    Fibonacci,
}
impl Rule {
    pub fn goal(self) -> u64 {
        match self {
            Self::Classic => 2048,
            Self::Fibonacci => 2584,
        }
    }
    pub fn first(self) -> u64 {
        match self {
            Self::Classic => 2,
            Self::Fibonacci => 1,
        }
    }
    /// Fibonacci neighbors merge in either order; 1 + 1 is the sole equal pair.
    /// Values that would overflow remain separate.
    pub fn merge(self, a: u64, b: u64) -> Option<u64> {
        if a == 0 || b == 0 {
            return None;
        }
        let compatible = match self {
            Self::Classic => a == b,
            Self::Fibonacci => {
                let (low, high) = (a.min(b), a.max(b));
                let (mut prev, mut next) = (1_u64, 1_u64);
                while next < high {
                    let sum = prev.checked_add(next)?;
                    (prev, next) = (next, sum);
                }
                (prev, next) == (low, high)
            }
        };
        compatible.then(|| a.checked_add(b)).flatten()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Spawn {
    SmallOnly,
    /// 90% smallest tile; 10% next tile (2/4 or 1/2).
    #[default]
    Mixed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub side: usize,
    pub rule: Rule,
    pub spawn: Spawn,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            side: 4,
            rule: Rule::Classic,
            spawn: Spawn::Mixed,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
impl Direction {
    fn index(self, side: usize, line: usize, offset: usize) -> usize {
        match self {
            Self::Left => line * side + offset,
            Self::Right => line * side + side - 1 - offset,
            Self::Up => offset * side + line,
            Self::Down => (side - 1 - offset) * side + line,
        }
    }
}
/// One original tile's path, including both contributors to a merge.
#[derive(Clone, Debug)]
pub struct Motion {
    pub from: usize,
    pub to: usize,
    pub value: u64,
}
#[derive(Clone, Debug)]
pub struct Turn {
    pub motion: Vec<Motion>,
    pub merged: Vec<usize>,
    pub spawned: usize,
    pub gained: u64,
}
#[derive(Clone)]
struct State {
    cells: Vec<u64>,
    score: u64,
    moves: u64,
    rng: SmallRng,
}
pub struct Board {
    settings: Settings,
    state: State,
    history: Vec<State>,
}
impl Board {
    /// Starts with two tiles. Panics for a side outside the supported 3..=6 range.
    pub fn new(settings: Settings, seed: u64) -> Self {
        let mut board = Self {
            settings,
            state: State {
                cells: Vec::new(),
                score: 0,
                moves: 0,
                rng: SmallRng::seed_from_u64(seed),
            },
            history: Vec::new(),
        };
        board.reset(settings);
        board
    }
    /// Start a fresh board while advancing the existing random stream.
    pub fn reset(&mut self, settings: Settings) {
        assert!((3..=6).contains(&settings.side));
        self.settings = settings;
        self.state.cells = vec![0; settings.side * settings.side];
        self.state.score = 0;
        self.state.moves = 0;
        self.history.clear();
        self.spawn();
        self.spawn();
    }
    pub fn settings(&self) -> Settings {
        self.settings
    }
    /// Change future spawns without touching the board, random stream, or undo.
    /// Undo restores turns while keeping the currently selected spawn policy.
    pub fn set_spawn(&mut self, spawn: Spawn) {
        self.settings.spawn = spawn;
    }
    pub fn cells(&self) -> &[u64] {
        &self.state.cells
    }
    pub fn score(&self) -> u64 {
        self.state.score
    }
    pub fn moves(&self) -> u64 {
        self.state.moves
    }
    pub fn max_tile(&self) -> u64 {
        self.cells().iter().copied().max().unwrap_or(0)
    }
    /// Reaching the goal never prevents further moves: play continues endlessly.
    pub fn won(&self) -> bool {
        self.max_tile() >= self.settings.rule.goal()
    }
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }
    pub fn undo(&mut self) -> bool {
        if let Some(state) = self.history.pop() {
            self.state = state;
            true
        } else {
            false
        }
    }
    pub fn can_move(&self) -> bool {
        let n = self.settings.side;
        self.cells().iter().enumerate().any(|(i, &v)| {
            v == 0
                || [
                    ((i % n + 1 < n).then_some(i + 1)),
                    ((i / n + 1 < n).then_some(i + n)),
                ]
                .into_iter()
                .flatten()
                .any(|j| self.settings.rule.merge(v, self.cells()[j]).is_some())
        })
    }
    fn spawn(&mut self) -> usize {
        let empty: Vec<_> = self
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(i, &v)| (v == 0).then_some(i))
            .collect();
        // Only called at startup or after a changed move, which leaves a free cell.
        let index = empty[self.state.rng.random_range(0..empty.len())];
        let large = self.settings.spawn == Spawn::Mixed && self.state.rng.random_bool(0.1);
        self.state.cells[index] = self.settings.rule.first() * if large { 2 } else { 1 };
        index
    }
    pub fn step(&mut self, direction: Direction) -> Option<Turn> {
        let n = self.settings.side;
        let mut cells = vec![0; n * n];
        let mut motion = Vec::new();
        let mut merged = Vec::new();
        let mut gained = 0_u64;
        for line in 0..n {
            let input: Vec<_> = (0..n)
                .map(|k| direction.index(n, line, k))
                .filter(|&i| self.cells()[i] != 0)
                .collect();
            let (mut source, mut dest) = (0, 0);
            while source < input.len() {
                let from = input[source];
                let value = self.cells()[from];
                let to = direction.index(n, line, dest);
                motion.push(Motion { from, to, value });
                let combined = input
                    .get(source + 1)
                    .and_then(|&i| self.settings.rule.merge(value, self.cells()[i]));
                if let Some(sum) = combined {
                    motion.push(Motion {
                        from: input[source + 1],
                        to,
                        value: self.cells()[input[source + 1]],
                    });
                    cells[to] = sum;
                    merged.push(to);
                    gained = gained.saturating_add(sum);
                    source += 2;
                } else {
                    cells[to] = value;
                    source += 1;
                }
                dest += 1;
            }
        }
        if cells == self.state.cells {
            return None;
        }
        self.history.push(self.state.clone());
        self.state.cells = cells;
        self.state.score = self.state.score.saturating_add(gained);
        self.state.moves += 1;
        let spawned = self.spawn();
        Some(Turn {
            motion,
            merged,
            spawned,
            gained,
        })
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
