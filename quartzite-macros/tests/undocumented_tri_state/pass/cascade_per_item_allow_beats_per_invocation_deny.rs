// AC7 path (h): per-item `#[undocumented(allow)]` beats per-invocation `deny`.
//
// The `#[object_impl(undocumented = "deny")]` block raises the per-invocation level to deny,
// but the `overridden` method carries `#[undocumented(allow)]` — the per-item allow wins
// (most-specific-wins cascade) and no `compile_error!` is emitted for that method.
// The documented sibling method `value` emits nothing either (doc present → no diagnostic).
// The compile MUST succeed.
use quartzite::prelude::*;

#[derive(Extend, Object)]
#[root]
struct Cascade {
    /// Base object — delegation target.
    #[base]
    pub object_base: ObjectBase,
    /// A documented property.
    #[property]
    pub count: i32,
}

#[object_impl(undocumented = "deny")]
impl Cascade {
    // No `///` doc, but per-item `allow` beats the per-invocation `deny`.
    #[undocumented(allow)]
    #[invoke]
    const fn overridden(&self) -> i32 {
        self.count
    }

    /// Returns twice the count.
    #[invoke]
    const fn value(&self) -> i32 {
        self.count * 2
    }
}

fn main() {}
