# Object Tree

Items extracted from completed plans. See [index](../deferred-items.md).

## Deferred

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| `ObjectTree::find_by_name` subtree scoping \| future; flat search sufficient for v1 | [lookup-perf spec](../plans/done/2026-05-02-lookup-perf.spec.md) | | #54 |

## Out of scope

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Object tree / arena / ownership model (quartzite-runtime) | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) | ✅ done | |
| Subtree-scoped `find_by_name` (flat search sufficient for now) | [lookup-perf spec](../plans/done/2026-05-02-lookup-perf.spec.md) | | #54 |
| Async/reactive name-change notifications (needs event system) | [lookup-perf spec](../plans/done/2026-05-02-lookup-perf.spec.md) | | #54 |

## Open questions

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Should `WeakObjectRef` be a type alias or a newtype? (Depends on runtime ownership choice) | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) | ✅ done | |
| Should `ObjectBase` derive `Debug`? (Useful for testing; `dynamic_properties` contains `Value`) | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) | | #61 |
| **Object ownership**: arena (SlotMap + ObjectId keys) vs `Rc<RefCell<dyn Object>>`? This is the primary decision blocking implementation. | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) | ✅ done | |
| Should `ObjectRef<T>` be a typed wrapper around `ObjectId` or around `Rc<RefCell<T>>`? | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) | ✅ done | |
| **`ObjectExt::parent()` / `children()` accessor mechanism**: With parent/children removed from `ObjectBase`, these methods need to reach `ObjectTree`. The simplest approach is a process-global `fn with_tree<R>(f: impl FnOnce(&ObjectTree) -> R) -> R` registered by `Application`. Needs decision before Task 4 finalizes the `ObjectExt` blanket impl. | [runtime design](../plans/done/2026-05-01-runtime.design.md) | | #55 |
| Should `ObjectTree::rename` be a no-op when called with the object's current name? Implementation must be correct either way; no special case required. | [lookup-perf spec](../plans/done/2026-05-02-lookup-perf.spec.md) | | #61 |
