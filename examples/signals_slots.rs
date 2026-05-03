use quartzite::prelude::*;

#[derive(Extend, DeriveObject)]
#[root]
struct Greeter {
    #[base]
    object_base: ObjectBase,
    #[signal]
    pub greeted: Signal<(String,)>,
}

#[object_impl]
impl Greeter {}

fn main() {
    let mut g = Greeter {
        object_base: ObjectBase::new(),
        greeted: Signal::new(),
    };

    // Typed connect: direct field access, compile-time argument types.
    g.greeted
        .connect(|args| println!("typed slot: hello, {}", args.0));

    // Dynamic connect: runtime dispatch via Object trait, args as &[Value].
    g.connect_signal(
        "greeted",
        Box::new(|vals| println!("dynamic slot received {} value(s)", vals.len())),
    );

    g.greeted.emit(&(String::from("world"),));
}
