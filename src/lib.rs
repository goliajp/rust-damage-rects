//! # damage-rects
//!
//! Accumulate dirty rectangles, coalesce overlaps, and emit a minimal set of
//! redraw regions for partial GPU canvas updates.
//!
//! **Status:** placeholder — extraction from [goliajp/tora](https://github.com/goliajp/tora)
//! (`crates/tora-gpu/src/damage_tracker.rs`) pending.
//!
//! Coalesce strategies: `Conservative` (keep small rects, minimize redraw
//! area), `Aggressive` (merge eagerly, minimize draw calls), `Threshold(f32)`
//! (fall back to full-viewport repaint when dirty area exceeds N% of screen).
