//! Integration tests for the `#[object_impl]` and `#[object_part]` attribute macros across sole-mode and multi-block types.

use quartzite::core::{Object, ObjectBase, Signal, Value};
use quartzite_macros::{Extend, Object as ObjectDerive, object_impl, object_part};

// ── Sole-mode type (no #[object_part] — AC3) ────────────────────────────────

#[derive(Extend, ObjectDerive)]
#[root]
struct Widget {
    #[base]
    object_base: ObjectBase,
    #[prop]
    pub value: i32,
    #[signal]
    pub value_changed: Signal<(i32,)>,
}

#[object_impl]
impl Widget {
    #[slot]
    const fn set_value(&mut self, v: i32) {
        self.value = v;
    }

    #[invokable]
    const fn doubled(&self) -> i32 {
        self.value * 2
    }
}

// ── Multi-block type: #[object_part] + #[object_impl] (AC1/AC4) ─────────────

#[derive(Extend, ObjectDerive)]
#[root]
struct MultiBlock {
    #[base]
    object_base: ObjectBase,
    #[prop]
    pub value: i32,
}

// AC1: #[object_part] on an inherent impl — emits only the cleaned impl block.
#[object_part]
impl MultiBlock {
    #[slot]
    const fn via_part(&mut self) {
        self.value += 1;
    }
}

// AC4: #[object_impl] in terminal mode — drains accumulated methods and emits full output.
#[object_impl]
impl MultiBlock {
    #[invokable]
    const fn via_impl(&self) -> i32 {
        self.value * 3
    }
}

// ── Trait-impl part type (AC2) ───────────────────────────────────────────────

trait Resettable {
    fn reset(&mut self);
}

#[derive(Extend, ObjectDerive)]
#[root]
struct TraitPartWidget {
    #[base]
    object_base: ObjectBase,
    #[prop]
    pub value: i32,
}

// AC2: #[object_part] on a trait impl — emits `impl Trait for Type { … }` with attrs stripped.
#[object_part]
impl Resettable for TraitPartWidget {
    fn reset(&mut self) {
        self.value = 0;
    }
}

#[object_impl]
impl TraitPartWidget {
    #[slot]
    const fn increment(&mut self) {
        self.value += 1;
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

// AC3/AC8 (sole mode): meta_object() returns MetaObject with correct class_name.
#[test]
fn ac8_meta_object_class_name() {
    let w = Widget {
        object_base: ObjectBase::new(),
        value: 0,
        value_changed: Signal::default(),
    };
    let meta = w.meta_object();
    assert_eq!(meta.class_name, "Widget");
}

// AC3: invoke_method dispatches to the slot with correct argument conversion.
#[test]
fn ac9_invoke_method_slot() {
    let mut w = Widget {
        object_base: ObjectBase::new(),
        value: 0,
        value_changed: Signal::default(),
    };
    let result = w.invoke_method("set_value", &[Value::Int(99)]);
    assert_eq!(result, Some(Value::Null));
    assert_eq!(w.value, 99);
}

#[test]
fn ac9_invoke_method_invokable_with_return() {
    let mut w = Widget {
        object_base: ObjectBase::new(),
        value: 7,
        value_changed: Signal::default(),
    };
    let result = w.invoke_method("doubled", &[]);
    assert_eq!(result, Some(Value::Int(14)));
}

#[test]
fn invoke_method_wrong_arity_returns_none() {
    let mut w = Widget {
        object_base: ObjectBase::new(),
        value: 0,
        value_changed: Signal::default(),
    };
    let result = w.invoke_method("set_value", &[]);
    assert_eq!(result, None);
}

#[test]
fn invoke_method_wrong_arg_type_returns_none() {
    let mut w = Widget {
        object_base: ObjectBase::new(),
        value: 0,
        value_changed: Signal::default(),
    };
    let result = w.invoke_method("set_value", &[Value::Bool(true)]);
    assert_eq!(result, None);
}

// AC4: multi-block — methods from both #[object_part] and #[object_impl] are dispatched.
#[test]
fn ac4_multi_block_methods_dispatched() {
    let mut mb = MultiBlock {
        object_base: ObjectBase::new(),
        value: 0,
    };
    // via_part: came from #[object_part] block
    let r1 = mb.invoke_method("via_part", &[]);
    assert_eq!(r1, Some(Value::Null));
    assert_eq!(mb.value, 1);
    // via_impl: came from #[object_impl] terminal block
    let r2 = mb.invoke_method("via_impl", &[]);
    assert_eq!(r2, Some(Value::Int(3)));
}

// AC2: trait impl from #[object_part] is actually applied — the impl Resettable block is emitted.
#[test]
fn ac2_trait_impl_part_is_emitted() {
    let mut w = TraitPartWidget {
        object_base: ObjectBase::new(),
        value: 5,
    };
    w.reset();
    assert_eq!(w.value, 0);
    // increment method from #[object_impl] terminal block is also dispatched
    let r = w.invoke_method("increment", &[]);
    assert_eq!(r, Some(Value::Null));
    assert_eq!(w.value, 1);
}
