/// Compile-time checks that `AsObject` and `Object` remain object-safe.
/// If either trait breaks object-safety, these functions will fail to compile.
fn _assert_object_safe_as_object() {
    let _: Box<dyn quartzite_core::AsObject>;
}

fn _assert_object_safe_object() {
    let _: Box<dyn quartzite_core::Object>;
}
