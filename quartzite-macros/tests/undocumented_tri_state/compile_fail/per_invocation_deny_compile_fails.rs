// AC7 path (g): per-invocation `#[object_impl(undocumented = "deny")]` on an impl block
// with a missing-doc `#[invoke]` method escalates to `compile_error!` directly.
use quartzite::prelude::*;

#[derive(Extend, Object)]
#[root]
struct Baz {
    /// Base object.
    #[base]
    pub object_base: ObjectBase,
    /// A documented property.
    #[property]
    pub z: i32,
}

#[object_impl(undocumented = "deny")]
impl Baz {
    // No `///` doc on `get_z`; per-invocation deny escalates to compile error.
    #[invoke]
    const fn get_z(&self) -> i32 {
        self.z
    }
}

fn main() {}
