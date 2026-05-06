//! Demonstrates the `emit!` macro for blocked-aware signal emission.
//!
//! `emit!(receiver.field, &args)` checks `signals_blocked()` on the owning
//! object and skips all slots when blocked — without the caller having to
//! extract the flag manually.  This is how the generated `emit_<signal>`
//! wrappers work under the hood; the macro is also available for hand-written
//! `AsObject` types that do not use `#[derive(Object)]`.

use quartzite::prelude::*;

// A minimal hand-rolled object — no derive macros — to show the macro works
// for any AsObject implementor, not just generated code.
struct Sensor {
    base: ObjectBase,
    pub reading: Signal<(f32,)>,
}

impl AsObject for Sensor {
    fn object_base(&self) -> &ObjectBase {
        &self.base
    }
    fn object_base_mut(&mut self) -> &mut ObjectBase {
        &mut self.base
    }
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
        self
    }
}

fn main() {
    env_logger::init();

    let mut sensor = Sensor {
        base: ObjectBase::new(),
        reading: Signal::new(),
    };

    sensor
        .reading
        .connect(|args| println!("reading: {:.1}", args.0));

    // Normal emission — slot fires.
    println!("--- not blocked ---");
    emit!(sensor.reading, &(1.5_f32,));

    // Block signals — emit! suppresses without any extra guard from the caller.
    sensor.base.block_signals();
    println!("--- blocked (no output expected) ---");
    emit!(sensor.reading, &(2.0_f32,));

    // Unblock — emission resumes.
    sensor.base.unblock_signals();
    println!("--- unblocked ---");
    emit!(sensor.reading, &(3.7_f32,));
}
