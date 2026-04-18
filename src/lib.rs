//! Accumulate dirty rectangles and emit minimal redraw regions for
//! partial GPU canvas updates.
//!
//! Every interactive GPU-rendered app has a choice: redraw the whole
//! frame every tick, or track which regions changed and redraw only
//! those. The first is simple but wastes power; the second is what
//! editors, design tools, terminals, and dashboards actually do.
//!
//! This crate is the bookkeeping — you call [`DamageTracker::add`] when
//! state changes, and at frame time you ask for the union of pending
//! regions via [`DamageTracker::merged`]. It has no opinion on your
//! coordinate system, no viewport tracking, and no GPU dependency —
//! pairs with any 2D renderer (Skia, wgpu, raw Metal, softbuffer).
//!
//! # Example
//!
//! ```
//! use damage_rects::{DamageRect, DamageTracker};
//!
//! let mut tracker = DamageTracker::new();
//!
//! // during state updates:
//! tracker.add(DamageRect::new(10.0, 20.0, 100.0, 30.0)); // a line changed
//! tracker.add(DamageRect::new(50.0, 30.0, 80.0, 40.0));  // cursor moved
//!
//! // at frame time:
//! if let Some(redraw_region) = tracker.merged() {
//!     // render(redraw_region.x, redraw_region.y, redraw_region.width, redraw_region.height)
//!     tracker.clear();
//! }
//! ```
//!
//! # Full-damage shortcut
//!
//! Some events invalidate the whole viewport (window resize, theme
//! switch). Call [`DamageTracker::mark_full`] and subsequent
//! [`DamageTracker::merged`] returns `None`; consult
//! [`DamageTracker::is_full`] and render the whole viewport in that
//! case. The flag prevents collecting individual rects you don't need.

#![deny(missing_docs)]

use core::fmt;

/// An axis-aligned rectangle in `f32` space. Coordinate system is up to
/// the caller — the library treats `y` as "down" only in the sense that
/// [`DamageRect::bottom`] returns `y + height`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageRect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width; must be non-negative.
    pub width: f32,
    /// Height; must be non-negative.
    pub height: f32,
}

impl DamageRect {
    /// Construct a new rectangle.
    ///
    /// Negative `width` or `height` is not rejected but produces nonsense
    /// from [`DamageRect::intersects`] and [`DamageRect::union`]; the
    /// caller is responsible for ensuring non-negativity.
    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Right edge (`x + width`).
    #[inline]
    #[must_use]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Bottom edge (`y + height`).
    #[inline]
    #[must_use]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Area (`width * height`).
    #[inline]
    #[must_use]
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Whether `self` overlaps `other` (zero-area edge contact returns
    /// `false`).
    #[inline]
    #[must_use]
    pub fn intersects(&self, other: &DamageRect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }

    /// Smallest rectangle that contains both `self` and `other`.
    #[must_use]
    pub fn union(&self, other: &DamageRect) -> DamageRect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        DamageRect::new(x, y, right - x, bottom - y)
    }

    /// Whether the point `(px, py)` lies inside `self`. Matches
    /// half-open semantics: left/top inclusive, right/bottom exclusive.
    #[inline]
    #[must_use]
    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }
}

impl fmt::Display for DamageRect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:.1}, {:.1}] {:.1}x{:.1}",
            self.x, self.y, self.width, self.height
        )
    }
}

/// Accumulates dirty rectangles across a frame and emits a merged
/// redraw region at frame time.
///
/// Two orthogonal states:
///
/// - **full-damage flag** — set by [`mark_full`](Self::mark_full) when
///   something invalidates the whole viewport. In this state the
///   individual rect list is ignored.
/// - **pending rects** — added via [`add`](Self::add).
///
/// [`merged`](Self::merged) unions all pending rects into one;
/// [`rects`](Self::rects) exposes the raw list for callers that want
/// to implement their own coalescing strategy (conservative — keep
/// small rects, aggressive — union everything, or threshold — fall
/// back to full viewport when dirty area exceeds N% of total).
#[derive(Debug, Clone, Default)]
pub struct DamageTracker {
    rects: Vec<DamageRect>,
    full: bool,
}

impl DamageTracker {
    /// New empty tracker. Initial state: no damage.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            rects: Vec::new(),
            full: false,
        }
    }

    /// New tracker pre-flagged for full-viewport damage. Useful on
    /// startup or after a window resize.
    #[inline]
    #[must_use]
    pub const fn with_full_damage() -> Self {
        Self {
            rects: Vec::new(),
            full: true,
        }
    }

    /// Add a dirty rectangle. Ignored if the full-damage flag is set.
    pub fn add(&mut self, rect: DamageRect) {
        if !self.full {
            self.rects.push(rect);
        }
    }

    /// Mark the entire viewport as dirty; clears individual rects.
    pub fn mark_full(&mut self) {
        self.rects.clear();
        self.full = true;
    }

    /// Reset to no-damage state.
    pub fn clear(&mut self) {
        self.rects.clear();
        self.full = false;
    }

    /// Whether the full-damage flag is set.
    #[inline]
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.full
    }

    /// Whether there is anything to redraw (full flag or any rects).
    #[inline]
    #[must_use]
    pub fn has_damage(&self) -> bool {
        self.full || !self.rects.is_empty()
    }

    /// Immutable view of the pending rects (empty if none or if full
    /// flag is set — check [`is_full`](Self::is_full) first).
    #[inline]
    #[must_use]
    pub fn rects(&self) -> &[DamageRect] {
        &self.rects
    }

    /// Union of all pending rects, or `None` if the full flag is set or
    /// there are no rects.
    ///
    /// When the full flag is set, returns `None` — the caller should
    /// consult [`is_full`](Self::is_full) and render the whole viewport.
    /// The library doesn't know the viewport size so it cannot produce
    /// the full-viewport rect on your behalf.
    #[must_use]
    pub fn merged(&self) -> Option<DamageRect> {
        if self.full {
            return None;
        }
        self.rects.iter().copied().reduce(|a, b| a.union(&b))
    }

    /// Sum of the individual rect areas. **Over-counts overlap** — if
    /// rects A and B overlap by area X, the returned value is
    /// `area(A) + area(B)` which double-counts X.
    ///
    /// Use this as a cheap upper bound when deciding whether to fall
    /// back to full viewport redraw via a threshold.
    #[must_use]
    pub fn area_upper_bound(&self) -> f32 {
        self.rects.iter().map(DamageRect::area).sum()
    }

    /// Number of pending rects (zero if full flag is set).
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.rects.len()
    }

    /// Whether the pending rect list is empty. Note: can return `true`
    /// while [`is_full`](Self::is_full) also returns `true` — use
    /// [`has_damage`](Self::has_damage) for "anything to redraw".
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn new_tracker_has_no_damage() {
        let t = DamageTracker::new();
        assert!(!t.is_full());
        assert!(!t.has_damage());
        assert!(t.merged().is_none());
    }

    #[test]
    fn with_full_damage_sets_flag() {
        let t = DamageTracker::with_full_damage();
        assert!(t.is_full());
        assert!(t.has_damage());
        assert!(t.merged().is_none()); // documented: full returns None
    }

    #[test]
    fn add_then_merged_returns_union() {
        let mut t = DamageTracker::new();
        t.add(DamageRect::new(0.0, 0.0, 100.0, 20.0));
        t.add(DamageRect::new(50.0, 10.0, 100.0, 30.0));
        let merged = t.merged().expect("has merged");
        assert!(approx_eq(merged.x, 0.0));
        assert!(approx_eq(merged.y, 0.0));
        assert!(approx_eq(merged.right(), 150.0));
        assert!(approx_eq(merged.bottom(), 40.0));
    }

    #[test]
    fn mark_full_clears_individual_rects() {
        let mut t = DamageTracker::new();
        t.add(DamageRect::new(0.0, 0.0, 10.0, 10.0));
        t.add(DamageRect::new(50.0, 50.0, 10.0, 10.0));
        t.mark_full();
        assert!(t.is_full());
        assert_eq!(t.len(), 0);
    }

    #[test]
    fn add_is_ignored_when_full() {
        let mut t = DamageTracker::with_full_damage();
        t.add(DamageRect::new(0.0, 0.0, 10.0, 10.0));
        assert!(t.is_full());
        assert_eq!(t.len(), 0);
    }

    #[test]
    fn clear_resets_both_flag_and_rects() {
        let mut t = DamageTracker::with_full_damage();
        t.clear();
        assert!(!t.is_full());
        assert!(!t.has_damage());
    }

    #[test]
    fn area_upper_bound_overcounts_overlap() {
        let mut t = DamageTracker::new();
        t.add(DamageRect::new(0.0, 0.0, 10.0, 10.0)); // area 100
        t.add(DamageRect::new(5.0, 5.0, 10.0, 10.0)); // area 100, overlaps
        // Actual covered area is ~175; upper bound over-counts by 25.
        assert!(approx_eq(t.area_upper_bound(), 200.0));
    }

    #[test]
    fn single_rect_merges_to_itself() {
        let mut t = DamageTracker::new();
        let r = DamageRect::new(1.0, 2.0, 3.0, 4.0);
        t.add(r);
        assert_eq!(t.merged(), Some(r));
    }

    #[test]
    fn rect_intersects() {
        let a = DamageRect::new(0.0, 0.0, 10.0, 10.0);
        let b = DamageRect::new(5.0, 5.0, 10.0, 10.0);
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));

        let c = DamageRect::new(20.0, 20.0, 10.0, 10.0);
        assert!(!a.intersects(&c));

        // Edge contact does not count as intersection.
        let d = DamageRect::new(10.0, 0.0, 10.0, 10.0);
        assert!(!a.intersects(&d));
    }

    #[test]
    fn rect_union() {
        let a = DamageRect::new(0.0, 0.0, 10.0, 10.0);
        let b = DamageRect::new(20.0, 5.0, 10.0, 10.0);
        let u = a.union(&b);
        assert!(approx_eq(u.x, 0.0));
        assert!(approx_eq(u.y, 0.0));
        assert!(approx_eq(u.right(), 30.0));
        assert!(approx_eq(u.bottom(), 15.0));
    }

    #[test]
    fn contains_point_half_open() {
        let r = DamageRect::new(0.0, 0.0, 10.0, 10.0);
        assert!(r.contains_point(0.0, 0.0)); // top-left inclusive
        assert!(r.contains_point(5.0, 5.0));
        assert!(!r.contains_point(10.0, 5.0)); // right exclusive
        assert!(!r.contains_point(5.0, 10.0)); // bottom exclusive
        assert!(!r.contains_point(-1.0, 5.0));
    }
}
