//! Trybuild integration tests for the tri-state `#[undocumented]` diagnostic (AC7 of spec #564).
//!
//! # Layout
//!
//! - `compile_fail/` — fixtures that MUST fail to compile (deny-level or warn-escalated-to-error).
//! - `pass/` — fixtures that MUST compile successfully (allow-level, documented, or cascade wins).
//!
//! # AC7 path coverage
//!
//! | Path | Fixture | Dir |
//! |------|---------|-----|
//! | (a) `warn` fires on missing doc (escalated via `#![deny(deprecated)]`) | `warn_fires_on_missing_doc.rs` | `compile_fail/` |
//! | (b) silent when every annotated item is documented | `silent_when_documented.rs` | `pass/` |
//! | (c) per-item `#[undocumented(allow)]` silences missing doc | `per_item_allow_silences.rs` | `pass/` |
//! | (d) per-invocation `#[object_impl(undocumented = "allow")]` silences block | `per_invocation_allow_silences.rs` | `pass/` |
//! | (e) global `undocumented-allow` cargo feature silences all | `global_allow_via_feature_silences.rs` | `pass/` (feature-gated) |
//! | (f) per-item `#[undocumented(deny)]` → `compile_error!` | `per_item_deny_compile_fails.rs` | `compile_fail/` |
//! | (g) per-invocation `#[object_impl(undocumented = "deny")]` → `compile_error!` | `per_invocation_deny_compile_fails.rs` | `compile_fail/` |
//! | (h) per-item `allow` beats per-invocation `deny` | `cascade_per_item_allow_beats_per_invocation_deny.rs` | `pass/` |
//!
//! # Running the global-feature fixture (AC7 path (e))
//!
//! `cargo test -p quartzite-macros --test undocumented_tri_state --features undocumented-allow`
//!
//! The `global_allow_feature_fixture` test is compiled only when `undocumented-allow` is active
//! (the feature is baked into the `quartzite-macros` binary at its compile time, per KD9).
//!
//! # Note on running with `--features undocumented-allow`
//!
//! The `warn_fires_on_missing_doc_compile_fail` test is excluded when `undocumented-allow` is
//! active: the global allow suppresses the `warn`-level diagnostic so that fixture compiles
//! successfully (no longer a compile failure), which is the correct and expected behaviour.
//! The deny-level tests (f) and (g) remain compile failures regardless — `compile_error!` is
//! emitted at the per-item / per-invocation scope, which the global allow cannot override.

/// AC7 paths (b)–(d) and (h): pass fixtures that must compile successfully.
#[test]
fn pass_fixtures() {
    let t = trybuild::TestCases::new();
    t.pass("tests/undocumented_tri_state/pass/silent_when_documented.rs");
    t.pass("tests/undocumented_tri_state/pass/per_item_allow_silences.rs");
    t.pass("tests/undocumented_tri_state/pass/per_invocation_allow_silences.rs");
    t.pass("tests/undocumented_tri_state/pass/cascade_per_item_allow_beats_per_invocation_deny.rs");
}

/// AC7 path (a): default `warn` level fires when doc is missing and `#![deny(deprecated)]`
/// escalates it to an error.  Only run without `undocumented-allow` — that feature suppresses
/// the `warn` level globally, making this fixture compile successfully (pass) instead.
#[cfg(not(feature = "undocumented-allow"))]
#[test]
fn warn_fires_on_missing_doc_compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/undocumented_tri_state/compile_fail/warn_fires_on_missing_doc.rs");
}

/// AC7 paths (f) and (g): per-item and per-invocation `deny` escalate to `compile_error!`.
/// These are compile failures regardless of global allow — `compile_error!` is emitted at
/// per-item / per-invocation scope, which takes precedence over the global allow level.
#[test]
fn deny_compile_fail_fixtures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/undocumented_tri_state/compile_fail/per_item_deny_compile_fails.rs");
    t.compile_fail(
        "tests/undocumented_tri_state/compile_fail/per_invocation_deny_compile_fails.rs",
    );
}

/// AC7 path (e): global `undocumented-allow` cargo feature silences the diagnostic.
///
/// Run with: `cargo test -p quartzite-macros --test undocumented_tri_state
///            --features undocumented-allow global_allow_feature_fixture`
#[cfg(feature = "undocumented-allow")]
#[test]
fn global_allow_feature_fixture() {
    let t = trybuild::TestCases::new();
    t.pass("tests/undocumented_tri_state/pass/global_allow_via_feature_silences.rs");
}
