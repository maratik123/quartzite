//! Shows Extend, DeriveObject, object_impl, object_part, and emit! working together.
//!
//! Layout:
//!  - `Counter` — root object; property with notify + two signals
//!  - `#[object_part]` — increment / decrement slots (first impl block)
//!  - `#[object_part]` on a trait impl — Resettable (second impl block)
//!  - `#[object_impl]` — merges both parts; adds an invokable
//!  - `LimitedCounter` — derives from Counter, adds a `max` property
//!  - `main` — wires signals, invokes slots, demonstrates blocking

use quartzite::prelude::*;

// ── Root object ───────────────────────────────────────────────────────────────

#[derive(Extend, DeriveObject)]
#[root]
struct Counter {
    #[base]
    object_base: ObjectBase,
    /// Current value; write_property fires count_changed automatically.
    #[prop(notify = count_changed)]
    pub count: i32,
    #[signal]
    pub count_changed: Signal<(i32,)>,
    /// Fired (via emit!) when the counter reaches zero.
    #[signal]
    pub zeroed: Signal<()>,
}

// Part 1 — arithmetic slots.
#[object_part]
impl Counter {
    #[slot]
    fn increment(&mut self) {
        // write_property uses emit! internally for the notify signal.
        self.write_property("count", Value::Int((self.count + 1).into()));
    }

    #[slot]
    fn decrement(&mut self) {
        let new = (self.count - 1).max(0);
        self.write_property("count", Value::Int(new.into()));
        if new == 0 {
            // emit! checks signals_blocked before firing.
            emit!(self.zeroed, &());
        }
    }
}

// Part 2 — Resettable trait; object_part on a trait impl emits the cleaned
// impl block so the trait is satisfied while slots are still accumulated.
trait Resettable {
    fn reset(&mut self);
}

#[object_part]
impl Resettable for Counter {
    #[slot]
    fn reset(&mut self) {
        self.count = 0;
        // Emit both signals explicitly; emit! is the same guard the generated
        // emit_count_changed wrapper uses under the hood.
        emit!(self.count_changed, &(0,));
        emit!(self.zeroed, &());
    }
}

// Final object_impl — drains both parts and adds an invokable.
#[object_impl]
impl Counter {
    #[invokable]
    fn value(&self) -> i32 {
        self.count
    }
}

// ── Derived object ────────────────────────────────────────────────────────────

/// A Counter that clamps at a configurable maximum.
#[derive(Extend, DeriveObject)]
struct LimitedCounter {
    #[base]
    base: Counter,
    /// Upper bound; increment is a no-op once count == max.
    #[prop]
    pub max: i32,
}

#[object_part]
impl LimitedCounter {
    #[slot]
    fn increment(&mut self) {
        if self.base.count < self.max {
            self.base
                .write_property("count", Value::Int((self.base.count + 1).into()));
        }
    }
}

#[object_impl]
impl LimitedCounter {}

// ── Demo ──────────────────────────────────────────────────────────────────────

fn main() {
    env_logger::init();

    let mut c = Counter {
        object_base: ObjectBase::new(),
        count: 0,
        count_changed: Signal::new(),
        zeroed: Signal::new(),
    };

    c.count_changed
        .connect(|args| println!("  count_changed → {}", args.0));
    c.zeroed.connect(|_| println!("  zeroed!"));

    println!("--- increment ×3 via invoke_method ---");
    c.invoke_method("increment", &[]);
    c.invoke_method("increment", &[]);
    c.invoke_method("increment", &[]);

    println!("value via invokable: {:?}", c.invoke_method("value", &[]));

    // Block signals: emit! inside write_property (notify) and explicit emit!
    // calls are both suppressed.
    c.object_base.block_signals();
    println!("--- increment while blocked (no signals) ---");
    c.invoke_method("increment", &[]);
    println!("  count is now {} (changed, no signal)", c.count);
    c.object_base.unblock_signals();

    println!("--- decrement to zero (fires zeroed) ---");
    c.invoke_method("decrement", &[]);
    c.invoke_method("decrement", &[]);
    c.invoke_method("decrement", &[]);
    c.invoke_method("decrement", &[]);

    println!("--- reset via Resettable trait slot ---");
    c.invoke_method("reset", &[]);

    // LimitedCounter: inherits Counter's signals and clamps at max.
    println!("\n--- LimitedCounter (max = 2) ---");
    let mut lc = LimitedCounter {
        base: Counter {
            object_base: ObjectBase::new(),
            count: 0,
            count_changed: Signal::new(),
            zeroed: Signal::new(),
        },
        max: 2,
    };

    lc.base
        .count_changed
        .connect(|args| println!("  limited count_changed → {}", args.0));

    lc.invoke_method("increment", &[]);
    lc.invoke_method("increment", &[]);
    lc.invoke_method("increment", &[]); // clamped — no signal
    println!("  final count: {} (capped at max=2)", lc.base.count);
}
