//! AC13 mechanical contract: `quartzite-widgets` must NOT pull in `quartzite-style`.
//!
//! The leaf crate `quartzite-style-types` is the cycle-break: widgets re-exports
//! `Palette` / `ColorRole` from the leaf, so the downstream `quartzite-style`
//! crate (which itself depends on widgets) never closes a cycle. This test
//! shells out to `cargo tree -p quartzite-widgets` and asserts that no line in
//! the resolved dependency tree names `quartzite-style` (the trailing space in
//! the grep avoids false-positives against `quartzite-style-types`).

use std::process::Command;

#[test]
fn quartzite_widgets_does_not_depend_on_quartzite_style() {
    let output = Command::new(env!("CARGO"))
        .args([
            "tree",
            "-p",
            "quartzite-widgets",
            "--prefix",
            "none",
            "--no-dedupe",
        ])
        .output()
        .expect("cargo tree should be runnable");

    assert!(
        output.status.success(),
        "cargo tree exited non-zero: stderr={}",
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8(output.stdout).expect("cargo tree output is UTF-8");

    for line in stdout.lines() {
        // Each tree line starts with the crate name, optionally followed by
        // a space and metadata (e.g. "quartzite-paint v0.1.0 (...)" or
        // "quartzite-style-types v0.1.0 (...)"). The contract: no line may
        // start with the literal "quartzite-style " (with trailing space —
        // anchors against `quartzite-style-types`).
        assert!(
            !line.starts_with("quartzite-style "),
            "quartzite-widgets pulls in quartzite-style — AC13 cycle-break broken \
             (offending line: {line:?})",
        );
    }
}
