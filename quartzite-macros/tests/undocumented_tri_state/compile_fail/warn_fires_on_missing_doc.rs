// AC7 path (a): default `warn` level fires when an annotated item has no `///` doc.
// `#![deny(deprecated)]` escalates the synthesised `#[deprecated]` warning to an error,
// making this a compile-fail fixture.  No per-item or per-invocation override is set.
#![deny(deprecated)]

use quartzite::prelude::*;

#[derive(Extend, Object)]
#[root]
struct Foo {
    /// Base object.
    #[base]
    pub object_base: ObjectBase,
    // No `///` doc on `x` — triggers the warn-level diagnostic.
    #[prop]
    pub x: i32,
}

#[object_impl]
impl Foo {}

fn main() {}
