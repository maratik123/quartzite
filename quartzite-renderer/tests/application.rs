//! Integration test for `WindowedApplication::new()` rejecting construction when a
//! `quartzite_runtime::Application` is already live.

// Each tests/*.rs file is compiled as a separate binary, giving this file
// a fresh OnceLock — necessary for Application singleton tests.

use quartzite_renderer::{RendererError, WindowedApplication};
use quartzite_runtime::ApplicationError;

// Verifies that AlreadyExists is returned when a quartzite Application is already
// live in the process, before WindowedApplication can be constructed.
#[test]
fn windowed_application_builder_returns_already_exists_on_second_call() {
    // Take the quartzite_runtime::Application singleton first so
    // WindowedApplication::new() fails without ever constructing
    // a winit EventLoop (which needs a display server).
    let _app = quartzite_runtime::Application::new()
        .expect("Application::new() must succeed in a fresh process");

    let result = WindowedApplication::new();
    assert!(
        matches!(
            result,
            Err(RendererError::Application(ApplicationError::AlreadyExists))
        ),
        "new() after Application is live must return \
         Err(RendererError::Application(AlreadyExists))"
    );
}
