# Games

Three small games using **wgame**, kept together in this repository:

- **Apples** (`apples`): count apples, pears, or oranges, with Russian number words.
- **Letters** (`letters`): Russian, English, and Greek alphabets, plus digits.
- **Mouse and cheese** (`mouse`): collect food, grow, and start another round.

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
The launcher supports clicking, keys **1–3**, or arrows and **Enter**.
**Escape** or **Меню** returns to the launcher; Escape in the launcher quits.

## Controls

| Game | Controls |
| --- | --- |
| Apples | Fruit buttons; **10/100** range buttons; digits and **Enter/Space**; **+/−** changes by one; **Page Up/Down** changes by ten; **Backspace/Delete** cancels entry; **`** changes the number font. |
| Letters | Flag/123 buttons or **1/2/3/0** select an alphabet or digits; **`** switches sans/serif fonts. |
| Mouse | **Arrow keys** or **WASD** move; collecting food grows the mouse. A new round starts one second after the last item is collected. |

Apples keeps a number pending while another digit could fit the selected range
(e.g. `1 → 10 → 100`). Enter accepts a shorter number; an unfinished entry is
cancelled after four seconds. Focus loss pauses movement and cancels held input.

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
```

`--smoke` presents twelve frames and exits. The offscreen test covers the launcher,
all three games, alphabet/font changes, counting to 100, portrait sizing, and DPI
scaling. Set `GAMES_RENDER_OUTPUT` to an existing directory to save PPM images.
