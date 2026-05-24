//! Integration test enforcing `Application` singleton semantics: first `builder().build()` succeeds, second returns `AlreadyExists`, `global()` is `Some` afterwards.

// Each tests/*.rs file is compiled as a separate binary, giving this file
// a fresh OnceLock — necessary for Application singleton tests.

use quartzite_runtime::{Application, ApplicationError};

// AC9 — first builder().build() succeeds; second returns Err; global() is Some afterward.
// All assertions in one test to guarantee sequential execution and a single
// OnceLock lifecycle per process.
#[test]
fn application_singleton_enforced() {
    // First creation must succeed.
    let app = Application::builder().build();
    assert!(
        app.is_ok(),
        "first Application::builder().build() must succeed"
    );

    // global() must be Some after creation.
    assert!(
        Application::global().is_some(),
        "Application::global() must be Some after builder().build()"
    );

    // Second creation must fail.
    let app2 = Application::builder().build();
    assert!(
        matches!(app2, Err(ApplicationError::AlreadyExists)),
        "second Application::builder().build() must return Err(AlreadyExists)"
    );
}
