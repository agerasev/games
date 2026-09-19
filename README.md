# Games

Small games using **wgame** with **wgame-egui** controls, kept together in this
repository:

- **Apples** (`apples`): count apples, pears, or oranges, with Russian number words.
- **Letters** (`letters`): Russian, English, and Greek alphabets, plus digits.
- **Mouse and cheese** (`mouse`): collect food, grow, and start another round.
- **Moon Lander** (`lander`): fuel-limited lunar flight, three landing sites, and physical exhaust particles.
- **2048 / Fibonacci** (`2048`): slide and merge numbers on configurable boards.

[Run](../run/README.md), [Drive](../drive/README.md), and
[Bounce (formerly Balls)](../bounce/README.md) live in their own repositories.

## Desktop

```sh
git submodule update --init --recursive
cargo run --locked --release
# Start a particular game directly:
cargo run --locked --release -- apples
```

All images and fonts are embedded; the binary works from any directory.
Game artwork and launcher preview tiles use the wgame canvas. Navigation,
settings, and scores use egui. Game controls sit in a left sidebar, wrapping and
scrolling on small windows.
2048 settings and font selectors are collapsed by default. Use **Помощь** for
keyboard controls. Click the canvas to give it keyboard focus; using a game
control returns focus to the canvas automatically.
The launcher supports clicking, keys **1–5**, or arrows and **Enter**.
**Escape** or **Меню** returns to the launcher; Escape in the launcher quits.

## Controls

| Game | Controls |
| --- | --- |
| Apples | Fruit buttons; **10/100** range buttons; digits and **Enter/Space**; **+/−** changes by one; **Page Up/Down** changes by ten; **Backspace/Delete** cancels entry; **`** changes the number font. |
| Letters | Alphabet buttons or **1/2/3/0** select an alphabet or digits; **`** switches sans/serif fonts. |
| Mouse | **Arrow keys** or **WASD** move; collecting food grows the mouse. **Заново** restarts the round. A new round starts one second after the last item is collected. |
| Moon Lander | Hold **Space/Up/W** for thrust; **Left/A**, **Right/D** tilt. **P** pauses/resumes, **R** restarts, **N** advances after landing. Egui provides hold-to-fly buttons and a site selector. |
| 2048 | **Arrows/WASD** or a swipe/drag on the board moves tiles. **U/Z/Backspace** undoes; **R** restarts. **3–6** select board size, **F** switches merge rules, **T** switches spawn rules. Open **Настройки** for size, merge rules, and spawn rules. Undo/restart stay visible. |

Apples keeps a number pending while another digit could fit the selected range
(e.g. `1 → 10 → 100`). Enter accepts a shorter number; an unfinished entry is
cancelled after four seconds. Focus loss pauses movement and cancels held input.

## 2048 and Fibonacci

Start directly with `cargo run --locked --release -- 2048`.
Choose a **3×3, 4×4, 5×5, or 6×6** board and either rule set:

- **2048:** equal tiles merge; new tiles are only 2s, or 90% 2s and 10% 4s.
- **Fibonacci:** neighboring values in `1, 2, 3, 5, 8, 13, …` merge in either
  order, plus `1 + 1 → 2`. For example, `2 + 3 → 5`, but `2 + 2` cannot merge.
  New tiles are only 1s, or 90% 1s and 10% 2s.

Each tile merges at most once per move. A successful move creates one new tile;
a blocked move creates none. The score increases by the values created by merges.
Reaching **2048** or **2584** marks the goal, and play continues until no moves
remain. Undo restores the board, score, move count, and random state, so replaying
the same move with the same spawn rule produces the same spawn. Undo can be
repeated back to the start of the round. Changing size or merge rules starts a
fresh round. Changing spawn rules only affects future tiles, preserving the board,
score, and undo history; undo keeps the currently selected spawn rule.
Rounds and undo history last until leaving the game; they are not saved to disk.

## Moon Lander

Start with `cargo run --locked --release -- lander`. Each attempt starts paused;
press **Старт** or hold a flight control to begin. Landing sites progress from a
wide plain to a crater crossing and a narrow ledge. Both feet must reach the green
pad with horizontal speed at most **1.5 m/s**, vertical speed at most **3 m/s**,
tilt at most **12°**, and rotation at most **26°/s**. Colored telemetry shows the
speed and tilt limits. Leaving the flight area ends the attempt.

Thrust follows the craft's tilt, so counter-tilt to brake horizontal motion.
Releasing rotation stabilizes the spin but keeps the current tilt. Main thrust
and steering consume fuel. This is an arcade model with lunar gravity (1.62 m/s²)
and assisted rotation. Window focus loss pauses the flight; resume explicitly.

The `phy` submodule integrates craft motion with RK4 and cosmetic exhaust with
Euler at a fixed 120 Hz. Exhaust inherits ship velocity, falls under gravity, and
scatters into short-lived dust on terrain contact. Terrain collision and landing
rules live in the game. Rendering uses existing wgame shapes; no engine changes
or external assets are required. Progress lasts until leaving the game.

## Browser (WebGL2)

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
./scripts/build-web.sh /games/
```

Serve or publish the contents of `dist/`. Omit the script's argument for relative
URLs. For development:

```sh
NO_COLOR=true trunk serve --no-default-features --features web
```

Click the canvas to focus keyboard input. Mouse and touch can select games and
buttons; Mouse requires a keyboard for movement. Refresh after quitting the
launcher. Desktop and web features are mutually exclusive.

## Checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
# Requires a GPU adapter; Mesa lavapipe also works:
cargo test --locked --test rendering -- --ignored
cargo run --locked -- --smoke
cargo run --locked -- mouse --smoke
cargo run --locked -- 2048 --smoke
cargo run --locked -- lander --smoke
```

`--smoke` presents twelve frames through the egui host and exits. CPU UI tests
cover control clicks, repeated layout passes, undo, and resizing. The offscreen
test covers canvas artwork, alphabet/font changes, counting to 100, 2048 variants
and animation, lunar flight and exhaust, portrait sizing, and DPI scaling. Set `GAMES_RENDER_OUTPUT` to an existing
directory to save PPM images.
