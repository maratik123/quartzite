// AC7 path (f): per-item `#[undocumented(deny)]` on a missing-doc field escalates to
// `compile_error!` directly — no `#![deny(deprecated)]` needed.
use quartzite::prelude::*;

#[derive(Extend, Object)]
#[root]
struct Bar {
    /// Base object.
    #[base]
    pub object_base: ObjectBase,
    // No `///` doc on `y`; per-item deny escalates to compile error.
    #[undocumented(deny)]
    #[prop]
    pub y: i32,
}

#[object_impl]
impl Bar {}

fn main() {}
