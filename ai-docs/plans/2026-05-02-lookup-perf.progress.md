# Progress: Lookup Performance & API Improvements

**Spec:** ai-docs/plans/2026-05-02-lookup-perf.spec.md
**Design:** ai-docs/plans/2026-05-02-lookup-perf.design.md
**Branch:** feat/2026-05-02-lookup-perf
**Base commit:** eaafc36e0c575499b5322df0c1a5a5caee9daf23

## Subtasks

| # | Task | Status |
|---|------|--------|
| 1 | Add `indexmap` dep to `quartzite-core/Cargo.toml` | ✅ |
| 2 | IndexMap in `signal.rs` | ✅ |
| 3a | fn-pointer fields on `MetaObject` | ⬜ |
| 3b | fn-pointer fields on `EnumMeta` + no-op helpers | ⬜ |
| 3c | Update hand-written `MetaObject::new` / `EnumMeta::new` call sites | ⬜ |
| 3d | Emit match lookups from `#[object_impl]` | ⬜ |
| 3e | Emit match lookups from `#[meta_enum]` | ⬜ |
| 4a | `ObjectBase::name` → `Option<String>`; remove `set_name` | ⬜ |
| 4b | `by_name` index in `ObjectTree` | ⬜ |
| 4c | `rename` / `clear_name` / `find_by_name` return `&[ObjectId]` | ⬜ |
| 4d | Update all callers | ⬜ |

## Files touched

- `quartzite-core/Cargo.toml`
- `quartzite-core/src/signal.rs`
- `quartzite-core/src/meta.rs`
- `quartzite-core/src/object_base.rs`
- `quartzite-core/src/traits.rs`
- `quartzite-macros/src/object_impl/codegen.rs`
- `quartzite-macros/src/meta_enum/codegen.rs`
- `quartzite-runtime/src/object_tree.rs`
- `quartzite-runtime/src/factory.rs`
- `quartzite-runtime/tests/object_tree.rs`
- `quartzite-runtime/tests/factory.rs`

## Design deviation — indexmap no_std

The design stated `indexmap = { version = "2", default-features = false }` would be sufficient for
`no_std` builds. This was incorrect: without `indexmap/std`, `IndexMap<K, V>` (two-param form)
is unavailable because `RandomState` has no default. Resolution (no design amendment required —
purely additive):
- `quartzite-core` std feature now propagates to `indexmap/std`
- Added `hashbrown = { version = "0.15", default-features = false }` as a direct dep
- `signal.rs` uses a conditional type alias for `no_std`: `type IndexMap<K, V> = indexmap::IndexMap<K, V, hashbrown::DefaultHashBuilder>`

## Next action

Continue with task 3a: fn-pointer fields on `MetaObject` in `quartzite-core/src/meta.rs`.
