# Estragon

**Stop waiting for Godot UI. Design it on the web.**

Estragon is a web studio for designing and theming game UI that exports **native Godot output**: real `Control` trees laid out by CSS Grid, not a browser embedded in your game.

> Status: **layout spike.** The core idea (one layout engine, identical results in the browser and in Godot) works. Everything else is ahead of us.

## How it works

Think React Native rather than React. Instead of supporting all of CSS, Estragon offers a small, deliberately limited set of primitives and a style vocabulary that Godot can express natively. Layout uses **[Taffy](https://github.com/DioxusLabs/taffy)** (Rust, spec-faithful CSS Grid + Flexbox) **compiled for both sides**:

```
                   schema/  (.ui.json: the shared contract)
                      │
          crates/estragon-layout  (Rust + Taffy)
             ┌────────┴─────────┐
   crates/estragon-wasm     crates/estragon-godot
   (wasm-bindgen)           (gdext GDExtension)
        │                        │
   apps/studio              EstragonGrid: a Container node
   (SvelteKit)              children stay ordinary Controls
```

Godot lays out one `Container` at a time, so the core exposes exactly that operation, and the studio walks its tree the same way. Same engine, same algorithm, same numbers.

## Layout

```
crates/estragon-layout   Schema types + Taffy translation. The single source of layout truth.
crates/estragon-godot    GDExtension: EstragonGrid (Container) + Estragon.load_document()
crates/estragon-wasm     WASM bindings for the studio
packages/layout-wasm     JS wrapper + TypeScript types for the WASM build
apps/studio              SvelteKit studio (spike: editor + Taffy vs. native CSS Grid comparison)
godot/                   Sandbox Godot project + headless parity tooling
schema/fixtures          Shared test documents
scripts/parity.mjs       Lays out a fixture in headless Godot and in the Rust core, then diffs every rect
```

## Getting started

Requires Rust via rustup (≥ 1.94) with `wasm32-unknown-unknown`, `wasm-bindgen-cli` matching the crate version, Node 22+, pnpm, and Godot 4.5+ on your `PATH` as `godot`.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
pnpm install

pnpm test      # Rust unit tests
pnpm parity    # Godot vs. core layout comparison (headless)
pnpm dev       # build WASM + run the studio
```

Open `godot/` in the Godot editor to see the HUD fixture rendered by `EstragonGrid`.

### `EstragonGrid` in Godot

Container properties: `columns`, `rows`, `areas`, `auto_rows`, `auto_columns`, `gap`, `row_gap`, `column_gap`, `padding` (CSS syntax: `240px 1fr`, `repeat(4, 1fr)`, `minmax(80px, 1fr)` …).

Per-child placement uses metadata: `estragon_area`, `estragon_column` (`1 / 3`, `span 2`), `estragon_row`, `estragon_justify_self`, `estragon_align_self`. A child's minimum size is always respected, as Godot requires.

## Known issues

- **Crash on first headless import (Godot 4.7.2 + gdext 0.5.5).** `godot --headless --import` on a fresh project crashes on exit when *any* gdext class is registered (reproduced with a trivial extension). The import completes; later runs are unaffected. To be reported upstream.

## Roadmap

1. ~~Layout parity spike~~ ✓
2. Editor integration: inspector section for grid-item properties, live editor preview
3. Generate TS types from the Rust schema (ts-rs / specta)
4. Primitives: Text (shared font measurement), Image, NineSlice, Button, Input …
5. Tokens → Godot `Theme` (`.tres`), variants → theme type variations
6. Components → inherited `.tscn` scenes
7. Studio ⇄ editor hot reload over WebSocket

## License

MIT
