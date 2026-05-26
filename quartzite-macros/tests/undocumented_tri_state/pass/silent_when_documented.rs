// AC7 path (b): default `warn` level is silent when every annotated item has a `///` doc.
// `#![deny(deprecated)]` is present to ensure no spurious `deprecated` warning fires.
#![deny(deprecated)]

use quartzite::prelude::*;

#[derive(Extend, Object)]
#[root]
struct Documented {
    /// Base object — delegation target.
    #[base]
    pub object_base: ObjectBase,
    /// Current counter value.
    #[prop]
    pub count: i32,
    /// Fired when `count` changes.
    #[signal]
    pub count_changed: Signal<(i32,)>,
}

#[object_impl]
impl Documented {
    /// Resets the counter to zero.
    #[slot]
    const fn reset(&mut self) {
        self.count = 0;
    }

    /// Returns the current value.
    #[invokable]
    const fn value(&self) -> i32 {
        self.count
    }
}

fn main() {}
