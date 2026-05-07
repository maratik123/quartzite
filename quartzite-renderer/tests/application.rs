// Each tests/*.rs file is compiled as a separate binary, giving this file
// a fresh OnceLock — necessary for Application singleton tests.

use quartzite_renderer::{RendererError, WindowedApplication};
use quartzite_runtime::ApplicationError;

// Verifies that AlreadyExists is returned when a quartzite Application is already
// live in the process, before WindowedApplication can be constructed.
// All singleton assertions are in one test fn to guarantee a single OnceLock
// lifecycle per process.
//
// We cannot test the Ok path here because winit's EventLoop::new() requires a
// display server (X11/Wayland/etc.) and would fail in headless CI.
#[test]
fn windowed_application_new_returns_already_exists_on_second_call() {
    // Take the quartzite_runtime::Application singleton first. This ensures
    // Application::new() fails inside WindowedApplication::new() without ever
    // constructing a winit EventLoop (which needs a display).
    let _app = quartzite_runtime::Application::new()
        .expect("first Application::new() must succeed in a fresh process");

    // Now WindowedApplication::new() must fail with Application(AlreadyExists)
    // because Application::new() inside it will find the OnceLock already set.
    let result = WindowedApplication::new();
    assert!(
        matches!(
            result,
            Err(RendererError::Application(ApplicationError::AlreadyExists))
        ),
        "WindowedApplication::new() after Application is live must return \
         Err(RendererError::Application(AlreadyExists))"
    );
}
