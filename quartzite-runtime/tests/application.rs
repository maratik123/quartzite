// Each tests/*.rs file is compiled as a separate binary, giving this file
// a fresh OnceLock — necessary for Application singleton tests.

use quartzite_runtime::{Application, ApplicationError};

// AC9 — first new() succeeds; second returns Err; global() is Some afterward.
// All assertions in one test to guarantee sequential execution and a single
// OnceLock lifecycle per process.
#[test]
fn application_singleton_enforced() {
    // First creation must succeed.
    let app = Application::new();
    assert!(app.is_ok(), "first Application::new() must succeed");

    // global() must be Some after creation.
    assert!(
        Application::global().is_some(),
        "Application::global() must be Some after new()"
    );

    // Second creation must fail.
    let app2 = Application::new();
    assert!(
        matches!(app2, Err(ApplicationError::AlreadyExists)),
        "second Application::new() must return Err(AlreadyExists)"
    );
}
