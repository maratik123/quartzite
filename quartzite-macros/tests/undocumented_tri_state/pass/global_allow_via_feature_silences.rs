// AC7 path (e): global `undocumented-allow` cargo feature silences the diagnostic for
// all annotated items workspace-wide.
//
// This fixture is only run by the `global_allow_feature_fixture` test in
// `undocumented_tri_state.rs`, which is gated on `#[cfg(feature = "undocumented-allow")]`.
// Run with:
//   cargo test -p quartzite-macros --test undocumented_tri_state \
//     --features undocumented-allow global_allow_feature_fixture
//
// `#![deny(deprecated)]` is present so that if any `deprecated` warning were emitted
// (i.e. the global allow were NOT effective), the compile would fail — proving the
// feature is the sole suppressor.
#![deny(deprecated)]

use quartzite::prelude::*;

// quartzite-macros compiled with `undocumented-allow` feature bakes Level::Allow globally.
// No per-item or per-invocation override needed; the global level silences everything.
#[derive(Extend, Object)]
#[root]
struct GlobalAllowed {
    // No `///` docs on any of these fields — suppressed by the global feature.
    #[base]
    pub object_base: ObjectBase,
    #[property]
    pub a: i32,
    #[signal]
    pub a_changed: Signal<(i32,)>,
}

#[object_impl]
impl GlobalAllowed {
    // No `///` doc — suppressed by global feature.
    #[invoke]
    const fn value(&self) -> i32 {
        self.a
    }
}

fn main() {}
