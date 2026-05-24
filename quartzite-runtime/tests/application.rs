//! Integration tests for `Application`: singleton semantics, Object-ification
//! (AC16, AC18, AC20), tick propagation (`AC9d`), and `object_base_mut` panic.
//!
//! Each tests/*.rs file is compiled as a separate binary, giving this file
//! a fresh `OnceLock` — necessary for Application singleton tests.
//!
//! All tests that require a live singleton are grouped in ONE test function
//! (`application_singleton_enforced`) to avoid inter-test race conditions on
//! the global `OnceLock`. Tests that do not require a singleton run independently.

use std::time::Duration;

use quartzite_core::{AsObject, ConnectionType, Object, ObjectExt, Value};
use quartzite_runtime::{Application, ApplicationError};

// ────────────────────────────────────────────────────────────────────────────
// Singleton-dependent tests — all in one function to avoid OnceLock races.
// Covers: AC3, AC9(d), AC16, AC18, AC20, and object_base_mut panic.
// ────────────────────────────────────────────────────────────────────────────

#[test]
fn application_singleton_enforced() {
    // ── AC3: first builder().build() must succeed ──────────────────────────
    let app = Application::builder()
        .tick_duration(Some(Duration::from_millis(50)))
        .build();
    assert!(
        app.is_ok(),
        "first Application::builder().build() must succeed"
    );
    let mut app = app.unwrap();

    // ── AC3: global() must be Some after creation ──────────────────────────
    assert!(
        Application::global().is_some(),
        "Application::global() must be Some after builder().build()"
    );

    // ── AC3: second creation must fail with AlreadyExists ──────────────────
    let app2 = Application::builder().build();
    assert!(
        matches!(app2, Err(ApplicationError::AlreadyExists)),
        "second Application::builder().build() must return Err(AlreadyExists)"
    );

    // ── AC16: class_name == "Application" ─────────────────────────────────
    assert_eq!(
        app.meta_object().class_name,
        "Application",
        "Application meta_object().class_name must be \"Application\""
    );

    // ── AC20: id() is non-zero ─────────────────────────────────────────────
    assert_ne!(
        app.id().raw(),
        0,
        "Application ObjectId must be non-zero after builder().build()"
    );

    // ── AC20: Application is NOT in the ObjectTree ─────────────────────────
    let app_id = app.id();
    assert!(
        !app.object_tree().lock().contains(app_id),
        "Application must not be registered in the ObjectTree (AC20)"
    );

    // ── AC9(d): tick_duration(Some(50ms)) propagates to EventLoop::tick() ──
    assert_eq!(
        app.event_loop().tick(),
        Some(Duration::from_millis(50)),
        "tick_duration(Some(50ms)) must propagate to the inner EventLoop"
    );

    // ── object_base_mut panics with documented message ─────────────────────
    let mut app_clone = Application::global().expect("singleton must be live");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: &mut _ = app_clone.object_base_mut();
    }));
    assert!(
        result.is_err(),
        "object_base_mut() on Application singleton must panic"
    );
    if let Err(payload) = result {
        let msg = payload
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
            .unwrap_or("");
        assert!(
            msg.contains("cannot be mutated through the shared handle"),
            "panic message must contain \"cannot be mutated through the shared handle\", got: {msg:?}"
        );
    }

    // ── Stub Object trait methods always return None / false ──────────────
    assert_eq!(app.read_property("x"), None, "read_property always returns None");
    assert!(
        !app.write_property("x", Value::Null),
        "write_property always returns false"
    );
    assert_eq!(
        app.invoke_method("unknown_slot", &[]),
        None,
        "invoke_method unknown name returns None"
    );
    assert_eq!(
        app.invoke_method("quit", &[Value::Null]),
        None,
        "invoke_method(\"quit\") with non-empty args returns None"
    );
    assert_eq!(
        app.connect_signal("sig", Box::new(|_| {}), ConnectionType::Direct),
        None,
        "connect_signal always returns None"
    );
    assert_eq!(app.emit_signal("sig", &[]), None, "emit_signal always returns None");

    // ── as_any / as_any_mut downcasts ─────────────────────────────────────
    assert!(app.as_any().is::<Application>(), "as_any() must be downcast-able");
    assert!(app.as_any_mut().is::<Application>(), "as_any_mut() must be downcast-able");

    // ── object_base returns a valid base ──────────────────────────────────
    let _ = app.object_base();

    // ── AC18: invoke_method("quit") returns Some(Value::Null) ──────────────
    // Called last so the singleton can be consumed.
    let result = app.invoke_method("quit", &[]);
    assert_eq!(
        result,
        Some(Value::Null),
        "invoke_method(\"quit\") must return Some(Value::Null) (AC18)"
    );
}

// ────────────────────────────────────────────────────────────────────────────
// Non-singleton tests — do not require a live Application.
// ────────────────────────────────────────────────────────────────────────────

// tick_duration(Some(ZERO)) normalises to tickless — tested through EventLoop
// since ApplicationBuilder::tick field is private.
#[test]
fn builder_tick_duration_zero_normalises_to_tickless() {
    use quartzite_runtime::EventLoop;
    let el = EventLoop::with_tick(Some(Duration::ZERO));
    assert_eq!(
        el.tick(),
        None,
        "EventLoop::with_tick(Some(ZERO)) must normalise to None (tickless)"
    );
}
