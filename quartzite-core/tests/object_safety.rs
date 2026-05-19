//! Compile-time object-safety checks for the `AsObject` and `Object` traits.

/// Compile-time checks that `AsObject` and `Object` remain object-safe.
/// If either trait breaks object-safety, these functions will fail to compile.
const fn _assert_object_safe_as_object() {
    let _: Box<dyn quartzite_core::AsObject>;
}

const fn _assert_object_safe_object() {
    let _: Box<dyn quartzite_core::Object>;
}
