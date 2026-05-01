use quartzite_core::{Object, ObjectBase, Signal, Value};
use quartzite_macros::{Extend, Object as ObjectDerive, object_impl};

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
    fn set_value(&mut self, v: i32) {
        self.value = v;
    }

    #[invokable]
    fn doubled(&self) -> i32 {
        self.value * 2
    }
}

// AC8: meta_object() returns MetaObject with correct class_name.
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

// AC9: invoke_method dispatches to the slot with correct argument conversion.
#[test]
fn ac9_invoke_method_slot() {
    let mut w = Widget {
        object_base: ObjectBase::new(),
        value: 0,
        value_changed: Signal::default(),
    };
    let result = w.invoke_method("set_value", &[Value::Int(99)]);
    // slot returns (), codegen emits Some(Value::Null)
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
    // set_value expects 1 arg, pass 0
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
    // set_value expects Value::Int (i32), but we pass Value::Bool
    let result = w.invoke_method("set_value", &[Value::Bool(true)]);
    assert_eq!(result, None);
}
