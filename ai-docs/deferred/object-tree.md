# Object Tree

Items extracted from completed plans. See [index](../deferred-items.md).

## Deferred

| Item | Source |
|------|--------|
| `ObjectTree::find_by_name` subtree scoping \| future; flat search sufficient for v1 | [lookup-perf spec](../plans/done/2026-05-02-lookup-perf.spec.md) |

## Out of scope

| Item | Source |
|------|--------|
| Object tree / arena / ownership model (quartzite-runtime) | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) |
| Subtree-scoped `find_by_name` (flat search sufficient for now) | [lookup-perf spec](../plans/done/2026-05-02-lookup-perf.spec.md) |
| Async/reactive name-change notifications (needs event system) | [lookup-perf spec](../plans/done/2026-05-02-lookup-perf.spec.md) |

## Open questions

| Item | Source |
|------|--------|
| Should `WeakObjectRef` be a type alias or a newtype? (Depends on runtime ownership choice) | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) |
| Should `ObjectBase` derive `Debug`? (Useful for testing; `dynamic_properties` contains `Value`) | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) |
| **Object ownership**: arena (SlotMap + ObjectId keys) vs `Rc<RefCell<dyn Object>>`? This is the primary decision blocking implementation. | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) |
| Should `ObjectRef<T>` be a typed wrapper around `ObjectId` or around `Rc<RefCell<T>>`? | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) |
| **`ObjectExt::parent()` / `children()` accessor mechanism**: With parent/children removed from `ObjectBase`, these methods need to reach `ObjectTree`. The simplest approach is a process-global `fn with_tree<R>(f: impl FnOnce(&ObjectTree) -> R) -> R` registered by `Application`. Needs decision before Task 4 finalizes the `ObjectExt` blanket impl. | [runtime design](../plans/done/2026-05-01-runtime.design.md) |
| Should `ObjectTree::rename` be a no-op when called with the object's current name? Implementation must be correct either way; no special case required. | [lookup-perf spec](../plans/done/2026-05-02-lookup-perf.spec.md) |
