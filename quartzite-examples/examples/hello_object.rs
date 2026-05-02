use quartzite::prelude::*;
use quartzite_core::ObjectBase;

#[derive(Extend, DeriveObject)]
#[root]
struct Counter {
    #[base]
    object_base: ObjectBase,
    #[prop(notify = count_changed)]
    pub count: i32,
    #[signal]
    pub count_changed: Signal<(i32,)>,
}

#[object_impl]
impl Counter {
    #[slot]
    fn reset(&mut self) {
        self.count = 0;
    }
}

fn main() {
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
