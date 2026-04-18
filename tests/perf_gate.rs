//! Performance gate for hot-path operations.
//!
//! Budgets apply only to release builds — debug assertions and
//! bounds checking make debug too slow to be meaningful. Tests still
//! execute in debug mode (for behaviour coverage), but only release
//! enforces the time budget.
//!
//! Budgets sized against the slowest realistic target (GitHub Actions
//! `ubuntu-latest` runner, ~2–4× slower than a recent Apple Silicon
//! Mac), per `.claude/rules/rust/patterns.md`. A local dev machine
//! will typically be well under budget; that is expected. The goal is
//! to catch regressions on the slowest surface, not set a vanity
//! number for fast hardware.
//!
//! Never weaken a budget without re-measuring P95 on the CI runner
//! and justifying in the commit message.

use damage_rects::{DamageRect, DamageTracker};
#[cfg(not(debug_assertions))]
use std::time::Duration;
use std::time::Instant;

#[test]
fn merged_on_100_rects_under_50us() {
    let mut t = DamageTracker::new();
    for i in 0..100 {
        t.add(DamageRect::new(i as f32, i as f32, 10.0, 10.0));
    }

    let start = Instant::now();
    let merged = t.merged();
    let elapsed = start.elapsed();

    assert!(merged.is_some());
    #[cfg(not(debug_assertions))]
    assert!(
        elapsed < Duration::from_micros(50),
        "merged() on 100 rects took {elapsed:?}, budget 50µs"
    );
    let _ = elapsed;
}

#[test]
fn add_10k_rects_under_1ms() {
    let mut t = DamageTracker::new();

    let start = Instant::now();
    for i in 0..10_000 {
        t.add(DamageRect::new(i as f32, 0.0, 1.0, 1.0));
    }
    let elapsed = start.elapsed();

    assert_eq!(t.len(), 10_000);
    #[cfg(not(debug_assertions))]
    assert!(
        elapsed < Duration::from_millis(1),
        "add x10_000 took {elapsed:?}, budget 1ms"
    );
    let _ = elapsed;
}

#[test]
fn area_upper_bound_on_1k_rects_under_20us() {
    let mut t = DamageTracker::new();
    for i in 0..1_000 {
        t.add(DamageRect::new(i as f32, 0.0, 10.0, 10.0));
    }

    let start = Instant::now();
    let sum = t.area_upper_bound();
    let elapsed = start.elapsed();

    assert!(sum > 0.0);
    #[cfg(not(debug_assertions))]
    assert!(
        elapsed < Duration::from_micros(20),
        "area_upper_bound on 1k rects took {elapsed:?}, budget 20µs"
    );
    let _ = elapsed;
}
