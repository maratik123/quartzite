//! Minimal example introducing the `Counter` object: property read/write and notify-signal wiring through the `quartzite` facade.

use quartzite::prelude::*;

#[derive(Extend, Object)]
#[root]
struct Counter {
    /// Object infrastructure (signals, dynamic dispatch) provided by `Extend`.
    #[base]
    object_base: ObjectBase,
    /// Current value; writing it fires `count_changed`.
    #[prop(notify = count_changed)]
    pub count: i32,
    /// Emitted with the new value whenever `count` changes.
    #[signal]
    pub count_changed: Signal<(i32,)>,
}

#[object_impl]
impl Counter {
    /// Resets `count` back to zero.
    #[slot]
    const fn reset(&mut self) {
        self.count = 0;
    }
}

fn main() {
    env_logger::init();
    let mut c = Counter {
        object_base: ObjectBase::new(),
        count: 0,
        count_changed: Signal::new(),
    };
    println!("initial count: {:?}", c.read_property("count")); // Some(Int(0))
    c.write_property("count", Value::Int(42));
    println!("after write:   {:?}", c.read_property("count")); // Some(Int(42))
    c.invoke_method("reset", &[]);
    println!("after reset:   {:?}", c.read_property("count")); // Some(Int(0))
}
