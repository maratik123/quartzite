# Macros & Codegen

Items extracted from completed plans. See [index](../deferred-items.md).

## Deferred

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| `#[object_impl]` on multiple impl blocks for one type \| single impl block only for now | [macros spec](../plans/done/2026-05-01-macros.spec.md) | | #57 |
| `proc_macro_crate` based path detection in `quartzite-macros` \| would let users depend on only `quartzite` or only `quartzite-core`; non-trivial, separate task | [examples-crate spec](../plans/done/2026-05-02-examples-crate.spec.md) | | #57 |

## Out of scope

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Runtime-specific code in macros (macros emit code that calls quartzite-core APIs) | [macros spec](../plans/done/2026-05-01-macros.spec.md) | ✅ done | |
| `#[derive(Extend)]` on enums or tuple structs (named fields only) | [macros spec](../plans/done/2026-05-01-macros.spec.md) | | — |
| Test-only `AsObject` impls inside `#[cfg(test)]` modules | [inline-simple-fns spec](../plans/done/2026-05-02-inline-simple-fns.spec.md) | | — |
| Any generated or hand-written function with branches, loops, or more than one call to a non-simple function | [inline-simple-fns spec](../plans/done/2026-05-02-inline-simple-fns.spec.md) (superseded by AGENTS.md Code Style → `#[inline]` recursive rule) | | — |
| Codegen emission of `/// _Simple._` doc tag for generated generic simple fns and for generated trait-method docs whose impls are always simple | AGENTS.md Code Style → `#[inline]` and the `_Simple._` doc tag | | #117 |
| Changing `quartzite-macros` codegen (deferred — `proc_macro_crate` is a separate future task) | [examples-crate spec](../plans/done/2026-05-02-examples-crate.spec.md) | | #57 |

## Open questions

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Should `Base` suffix be stripped when forming `As{Name}` trait? (`WidgetBase` → `AsWidget` or `AsWidgetBase`?) | [macros spec](../plans/done/2026-05-01-macros.spec.md) | ✅ done | |
| Should `#[derive(Object)]` require `#[derive(Extend)]` to be present, or are they independent? | [macros spec](../plans/done/2026-05-01-macros.spec.md) | ✅ done | |
| How to handle generic structs with `#[derive(Extend)]`? | [macros spec](../plans/done/2026-05-01-macros.spec.md) | | #57 |
| Should `MetaObject::new()` be extended with fn pointer params or replaced by struct literal construction in the macro? Decide in design phase. | [lookup-perf spec](../plans/done/2026-05-02-lookup-perf.spec.md) | ✅ done | |
