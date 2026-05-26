// AC7 path (c): per-item `#[undocumented(allow)]` silences the diagnostic for a
// missing-doc annotated field.  `#![deny(deprecated)]` is present so that any
// spurious `deprecated` emission would fail the compile — proving the allow is effective.
#![deny(deprecated)]

use quartzite::prelude::*;

#[derive(Extend, Object)]
#[root]
struct AllowItem {
    /// Base object — delegation target.
    #[base]
    pub object_base: ObjectBase,
    // No `///` doc on `val`; per-item allow suppresses the diagnostic.
    #[undocumented(allow)]
    #[property]
    pub val: i32,
}

#[object_impl]
impl AllowItem {}

fn main() {}
