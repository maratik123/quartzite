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
    env_logger::init();
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
        ConnectionType::Direct,
    );

    // Generated emit_greeted wrapper uses emit! internally — checks signals_blocked
    // before firing any slots.
    println!("--- emit (not blocked) ---");
    g.emit_greeted(String::from("world"));

    // Block signals: emit_greeted is now a no-op.
    g.object_base.block_signals();
    println!("--- emit (blocked — no output expected) ---");
    g.emit_greeted(String::from("suppressed"));

    // Unblock: emission resumes.
    g.object_base.unblock_signals();
    println!("--- emit (unblocked) ---");
    g.emit_greeted(String::from("back"));
}
