// AC7 path (d): per-invocation `#[object_impl(undocumented = "allow")]` silences the
// diagnostic for all missing-doc annotated methods in the impl block.
// `#![deny(deprecated)]` is present so that any spurious emission would fail the compile.
#![deny(deprecated)]

use quartzite::prelude::*;

#[derive(Extend, Object)]
#[root]
struct AllowInvocation {
    /// Base object — delegation target.
    #[base]
    pub object_base: ObjectBase,
    /// A documented property.
    #[prop]
    pub count: i32,
}

// Per-invocation allow: missing-doc `#[invokable]` and `#[slot]` are both suppressed.
#[object_impl(undocumented = "allow")]
impl AllowInvocation {
    // No `///` doc — suppressed by per-invocation allow.
    #[invokable]
    const fn value(&self) -> i32 {
        self.count
    }

    // No `///` doc — suppressed by per-invocation allow.
    #[slot]
    const fn reset(&mut self) {
        self.count = 0;
    }
}

fn main() {}
