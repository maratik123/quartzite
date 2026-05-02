# Macros & Codegen

Items extracted from completed plans. See [index](../deferred-items.md).

## Deferred

| Item | Source | Status |
|------|--------|--------|
| `#[object_impl]` on multiple impl blocks for one type \| single impl block only for now | [macros spec](../plans/done/2026-05-01-macros.spec.md) | |
| `proc_macro_crate` based path detection in `quartzite-macros` \| would let users depend on only `quartzite` or only `quartzite-core`; non-trivial, separate task | [examples-crate spec](../plans/done/2026-05-02-examples-crate.spec.md) | |

## Out of scope

| Item | Source | Status |
|------|--------|--------|
| Runtime-specific code in macros (macros emit code that calls quartzite-core APIs) | [macros spec](../plans/done/2026-05-01-macros.spec.md) | ✅ done |
| `#[derive(Extend)]` on enums or tuple structs (named fields only) | [macros spec](../plans/done/2026-05-01-macros.spec.md) | |
| Generic functions and blanket-impl trait methods (`ObjectRef<T>`, `WeakRef<T>`, `Signal<Args>`, `ObjectExt` default methods) — monomorphized, no cross-crate inlining benefit | [inline-simple-fns spec](../plans/done/2026-05-02-inline-simple-fns.spec.md) | |
| Test-only `AsObject` impls inside `#[cfg(test)]` modules | [inline-simple-fns spec](../plans/done/2026-05-02-inline-simple-fns.spec.md) | |
| Any generated or hand-written function with branches, loops, or more than one function call | [inline-simple-fns spec](../plans/done/2026-05-02-inline-simple-fns.spec.md) | |
| Changing `quartzite-macros` codegen (deferred — `proc_macro_crate` is a separate future task) | [examples-crate spec](../plans/done/2026-05-02-examples-crate.spec.md) | |

## Open questions

| Item | Source | Status |
|------|--------|--------|
| Should `Base` suffix be stripped when forming `As{Name}` trait? (`WidgetBase` → `AsWidget` or `AsWidgetBase`?) | [macros spec](../plans/done/2026-05-01-macros.spec.md) | ✅ done |
| Should `#[derive(Object)]` require `#[derive(Extend)]` to be present, or are they independent? | [macros spec](../plans/done/2026-05-01-macros.spec.md) | ✅ done |
| How to handle generic structs with `#[derive(Extend)]`? | [macros spec](../plans/done/2026-05-01-macros.spec.md) | |
| Should `MetaObject::new()` be extended with fn pointer params or replaced by struct literal construction in the macro? Decide in design phase. | [lookup-perf spec](../plans/done/2026-05-02-lookup-perf.spec.md) | ✅ done |
