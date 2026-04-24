# damage-rects

[![Crates.io](https://img.shields.io/crates/v/damage-rects?style=flat-square&logo=rust)](https://crates.io/crates/damage-rects)
[![docs.rs](https://img.shields.io/docsrs/damage-rects?style=flat-square&logo=docs.rs)](https://docs.rs/damage-rects)
[![License](https://img.shields.io/crates/l/damage-rects?style=flat-square)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.94-blue?style=flat-square&logo=rust)](Cargo.toml)
[![Downloads](https://img.shields.io/crates/d/damage-rects?style=flat-square)](https://crates.io/crates/damage-rects)

**English** | [简体中文](README.zh-CN.md) | [日本語](README.ja.md)

Accumulate dirty rectangles and emit a merged redraw region for partial
GPU canvas updates. Zero dependencies, no GPU coupling, no viewport
assumptions — pairs with any 2D renderer (Skia, wgpu, softbuffer, raw
Metal).

> **Also known as / if you're searching for:** damage tracking, dirty
> rectangle tracking, partial redraw, invalidation region, repaint
> region, region coalescing, editor/terminal incremental redraw.

## Why

Every interactive GPU-rendered app has a choice: redraw the whole frame
every tick, or track which regions changed and redraw only those. The
first is simple but wastes power; the second is what editors, design
tools, terminals, and dashboards actually do.

GUI frameworks (`gpui`, `iced`, `slint`) bake damage tracking into
their internals and never expose it as a reusable crate. This is that
crate.

## API

```rust
use damage_rects::{DamageRect, DamageTracker};

let mut tracker = DamageTracker::new();

// during state updates
tracker.add(DamageRect::new(10.0, 20.0, 100.0, 30.0)); // a line changed
tracker.add(DamageRect::new(50.0, 30.0, 80.0, 40.0));  // cursor moved

// at frame time
if tracker.is_full() {
    // something invalidated the whole viewport — redraw everything
} else if let Some(region) = tracker.merged() {
    // render(region)
}
tracker.clear();
```

Two orthogonal states:

- **full-damage flag** (`mark_full`) — set when something invalidates
  the whole viewport. `merged()` then returns `None` and
  `is_full()` returns `true`. Individual rects are ignored.
- **pending rects** (`add`) — merged via `merged()` into one
  bounding rect, or exposed via `rects()` if you want to implement a
  custom coalescing strategy (conservative / aggressive / threshold).

## Threshold strategy

When dirty area is a large fraction of total, it's cheaper to redraw
the whole viewport than many small rects. `area_upper_bound()` gives
a cheap (overcounts overlap) sum of rect areas:

```rust
if tracker.area_upper_bound() > viewport_area * 0.5 {
    // give up on partial — render everything
    tracker.mark_full();
}
```

## Demo

```bash
cargo run --example visualize -p damage-rects
```

Interactive window:

- click to add a dirty rect
- `Space` to render the merged region (green outline, then clear)
- `F` to toggle full-damage flag (yellow tint)
- `C` to clear

## Install

```toml
[dependencies]
damage-rects = "0.1"
```

## Coordinate system

`DamageRect { x, y, width, height }` in `f32`. Half-open at right/bottom
(`contains_point` returns `false` on right/bottom edges). Coordinate
orientation is up to the caller — the library only uses arithmetic.

<!-- ECOSYSTEM BEGIN (synced by claws/opensource/scripts/sync-ecosystem.py — edit ecosystem.toml, not this block) -->

## Ecosystem

[metal-live-resize](https://crates.io/crates/metal-live-resize) · [coalesce-worker](https://crates.io/crates/coalesce-worker) · **damage-rects**

<!-- ECOSYSTEM END -->

## License

MIT — see [LICENSE](LICENSE).
