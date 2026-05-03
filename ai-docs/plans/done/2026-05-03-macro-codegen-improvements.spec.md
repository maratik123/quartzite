# Macro Codegen Improvements

**Source:** issue #57
**Date:** 2026-05-03
**Tracked in:** #57

## Scope

1. **`proc_macro_crate` path detection** — resolve the correct absolute crate path at macro expansion time using the `proc-macro-crate` crate; strategy: check `quartzite` first (facade), fall back to `quartzite-core`, fall back to hardcoded `::quartzite_core::` if neither is a direct dep. Handle crate renaming transparently.
2. **`#[object_impl]` on multiple impl blocks** — collect `#[slot]` and `#[invokable]` methods across all `#[object_impl]` blocks for the same type; merge into a single `MetaObject`; emit compile error on duplicate method names across blocks. `#[object_impl]` may be applied to both inherent impl blocks and trait impl blocks (`impl Trait for Type`) — slots/invokables from trait impls are registered in the type's `MetaObject` alongside those from inherent impls. Properties and signals remain on struct fields processed by `#[derive(Object)]` — unchanged.
3. **Generic struct support for `#[derive(Extend)]`** — emit `impl<T> AsThing for MyStruct<T>` with minimal bounds (only bounds the generated code strictly requires; no unnecessary constraints propagated from the struct definition).

## Out of scope

- `#[derive(Extend)]` on enums or tuple structs (named fields only — existing constraint, unchanged)
- Generic functions or blanket-impl trait methods — not affected by codegen improvements
- Changing the public API surface of `quartzite-macros` (attribute names, derive names)
- Multi-crate / cross-crate `#[object_impl]` collection (same compilation unit only)

## Deferred

- Generic struct support for `#[derive(Object)]` (not `#[derive(Extend)]`) | bound propagation rules unclear | no separate issue yet
- `#[object_impl]` on generic types | orthogonal to this task | no separate issue yet

## Key decisions

| Question | Decision |
|---|---|
| `proc_macro_crate` fallback when neither dep found | Silent fallback to `::quartzite_core::` (no compile error) |
| Path when both `quartzite` and `quartzite-core` are direct deps | Facade-first: prefer `quartzite` (Option C — Bevy pattern) |
| Crate renaming | Handled transparently via `proc_macro_crate::crate_name()` return value |
| Items collected across `#[object_impl]` blocks | `#[slot]` and `#[invokable]` methods only — props/signals stay on struct |
| `#[object_impl]` on trait impls | Allowed — slots/invokables register in the type's `MetaObject` |
| Duplicate method names across blocks | Compile error |
| Bounds on generic `#[derive(Extend)]` impls | Minimal — only what generated code requires |

## Technical constraints

- `proc-macro-crate` must be added as a dependency of `quartzite-macros`
- The path resolution must cover all absolute crate paths currently hardcoded in macro output
- Multi-impl-block collection: proc-macro invocations are independent (no shared mutable state between `#[object_impl]` calls); the design must account for this — likely requires either an aggregating outer attribute or a deferred-registration runtime mechanism
- Generic `#[derive(Extend)]` must not require changes to the `Extend` trait's own definition

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | When `quartzite` is a direct dep, all macro-generated absolute paths use the facade's resolved name (including renamed aliases) |
| AC2 | When only `quartzite-core` is a direct dep, all macro-generated absolute paths use `quartzite-core`'s resolved name (including renamed aliases) |
| AC3 | When neither `quartzite` nor `quartzite-core` is a direct dep, macro-generated code silently falls back to `::quartzite_core::` without a compile error |
| AC4 | Multiple `#[object_impl]` blocks on the same type compile successfully with all `#[slot]` and `#[invokable]` methods from all blocks registered in the generated `MetaObject` |
| AC5 | Duplicate slot/invokable names across `#[object_impl]` blocks produce a compile error at macro expansion time |
| AC6 | `#[object_impl]` on a trait impl block (`impl Trait for Type`) is accepted; `#[slot]`/`#[invokable]` methods within it are registered in the type's `MetaObject` |
| AC7 | `#[derive(Extend)]` on a generic struct generates a valid `impl<T> AsThing for MyStruct<T>` with only the bounds required by the generated code |
| AC8 | Existing single-block `#[object_impl]` usage and non-generic `#[derive(Extend)]` continue to compile and behave identically to before |

## Open questions

- What mechanism enables collecting `#[object_impl]` methods across multiple independent macro invocations? Proc-macro calls are stateless between invocations. Candidates: `thread_local!` accumulation within one compilation, aggregating terminal attribute (`#[object_meta]`), or `linkme`/`inventory`-style distributed slices. (Design phase decision.)

## Notes

- All hardcoded macro-generated paths are already `::quartzite::core::` — the facade re-exports `quartzite-core` as a `core` submodule. Path detection resolves only the leading crate name.
- `#[derive(Object)]` (struct-level, handles props/signals) is independent of this task and unchanged.
