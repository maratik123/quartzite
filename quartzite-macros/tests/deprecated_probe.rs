// Probe: does #[deprecated] lint fire when the deprecated `const fn` definition
// and its immediate `const _: ()` invocation both appear in the same file?
// This mimics the synthesised codegen shape from KD1 (subtask 6c).
//
// Probe outcome (recorded 2026-05-26):
// When `#![deny(deprecated)]` and `const _: () = { __probe(); };` are active,
// `cargo build -p quartzite-macros --tests` FAILS with:
//   error: use of deprecated function `__probe`: probe-msg
//   --> quartzite-macros/tests/deprecated_probe.rs:15:17
// Conclusion: const _: shape fires the deprecated lint ✓  (KD1 option (a) confirmed).
//
// The active lines are commented out so this file compiles cleanly in normal
// workspace builds; subtask 7 trybuild fixtures supersede this probe.

// #![deny(deprecated)]

#[deprecated = "probe-msg"]
const fn __probe() {}

// const _: () = { __probe(); };
