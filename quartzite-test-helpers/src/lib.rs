//! Workspace-internal test-serialisation helper for quartzite.
//!
//! Provides a single [`test_lock()`] entry point that returns a
//! [`parking_lot::MutexGuard`] over a private static [`parking_lot::Mutex<()>`].
//! Tests that mutate process-global state (e.g. the global dispatcher, the
//! snapshot registry) hold the guard for their entire body to serialise
//! execution against one another.
//!
//! ## Per-binary serialisation semantics
//!
//! Each Rust test binary (`--lib` for every crate that links this helper, plus
//! every `tests/*.rs` integration-test binary in those crates) is a separate
//! process with its own address space. Even though the underlying `static
//! TEST_LOCK` is declared exactly once in this crate's source, **each
//! downstream test binary that links `quartzite-test-helpers` ends up with its
//! own instance of that static**. The lock therefore serialises tests **within
//! a single test binary**, matching the per-binary semantics that the previous
//! attribute-based serialisation helper provided. Cross-binary serialisation is
//! **not** provided (nor was it provided by the previous helper — `cargo test`
//! runs each test binary as an independent process).
//!
//! ## Usage
//!
//! Add `quartzite-test-helpers = { path = "../quartzite-test-helpers" }` to the
//! consumer crate's `[dev-dependencies]` and call [`test_lock()`] as the first
//! statement of any test that needs serialisation:
//!
//! ```no_run
//! #[test]
//! fn my_test() {
//!     let _lock = quartzite_test_helpers::test_lock();
//!     // body that mutates process-global state
//! }
//! ```
//!
//! Bind the guard to a named variable (`_lock` / `_guard`), **never** `let _ =
//! test_lock();` — the latter drops the guard immediately and defeats the
//! serialisation contract.

use parking_lot::{Mutex, MutexGuard};

static TEST_LOCK: Mutex<()> = Mutex::new(());

/// Acquires the per-binary test serialisation lock.
///
/// Hold the returned guard for the entire duration of any test that mutates
/// process-global state. The lock is released when the guard is dropped (end
/// of scope by default).
///
/// # Examples
///
/// ```no_run
/// let _lock = quartzite_test_helpers::test_lock();
/// // ... test body ...
/// ```
#[inline]
pub fn test_lock() -> MutexGuard<'static, ()> {
    TEST_LOCK.lock()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_acquires_and_releases() {
        // Happy path — acquire, drop, reacquire on the same thread.
        let g1 = test_lock();
        drop(g1);
        let g2 = test_lock();
        drop(g2);

        // Compile-time return-type check — enforces the public signature
        // stays stable across refactors.
        let _guard: MutexGuard<'static, ()> = test_lock();
    }
}
