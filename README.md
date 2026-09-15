# Run — Бег по лесу

Walk through a seeded forest with animated, layered sprites. Extracted from
`yarik-games`'s `running` game and migrated from Macroquad to wgame.

```sh
git submodule update --init --recursive
cargo run --locked --release
```

The wgame submodule follows the sibling `bounce` and `gravity` projects. Images
and animation metadata are embedded, so the binary runs from any directory.

## Controls

- WASD or arrow keys: move. Diagonal movement has the same speed.
- Mouse wheel: zoom in or out.
- Escape or close the window: quit.

The camera stays centered on the forest, as in the original. Trees and the
character are drawn back to front according to their ground position, with
nearest-neighbor sprite filtering. Losing window focus pauses movement; frame
steps are capped at 100 ms to avoid jumps after a stall. Zoom is bounded.

## Web (WebGL2)

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
./scripts/build-web.sh /run/
```

The script writes static files to `dist/`. Omit the path argument for relative
URLs. For local development:

```sh
NO_COLOR=true trunk serve --no-default-features --features web
```

Click the canvas to focus the keyboard. Refresh after Escape to restart.
Desktop is the default feature; select `web` with `--no-default-features`.

## Checks

```sh
cargo fmt --check
cargo test --locked
cargo check --locked --target wasm32-unknown-unknown --no-default-features --features web
cargo run --locked -- --smoke
```

`--smoke` renders twelve frames and exits on desktop. Tests cover movement,
focus cancellation, animation timing, sprite flipping and embedded metadata.
