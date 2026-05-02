# Threading & Runtime

Items extracted from completed plans. See [index](../deferred-items.md).

## Deferred

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| No-std validation \| design for it but don't enforce until runtime exists | [core-types spec](../plans/done/2026-05-01-core-types.spec.md) | ✅ done | |
| Multi-window support \| needs platform backend first | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) | | #53 |
| Thread event loops (one loop per thread) \| defer until threading model decided | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) | | #51 |
| Stale `thread_id` invalidation \| needs object-mobility / thread-affinity-change API first | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | | #52 |
| `AutoConnection` in no_std \| same gating as `Queued`; unblocked when std feature is defined | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | ✅ done | |

## Out of scope

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Thread migration — if an object moves to another thread after a connection is established, the captured `thread_id` will be stale; this requires a separate object-mobility design | [auto-connection spec](../plans/done/2026-05-01-auto-connection.spec.md) | | #52 |

## Open questions

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Should `ConnectionTable` use `DashMap` (lock-free) or `Mutex<HashMap>`? | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) | ✅ done | |
| Should the event loop be `async`-runtime-agnostic (pluggable executor) or std-thread-based only? | [runtime spec](../plans/done/2026-05-01-runtime.spec.md) | ✅ done | |
| **`QueuedDispatcher` trait location**: Proposed to live in quartzite-core behind `feature = "std"`. Alternative: define it in quartzite-runtime and have core use a raw function pointer or a `OnceLock<fn(Box<dyn FnOnce() + Send>)>`. Needs decision before Task 5. | [runtime design](../plans/done/2026-05-01-runtime.design.md) | ✅ done | |
| **`ObjectFactory` global vs. per-`Application`**: Should there be a process-wide factory singleton (like `ConnectionTable`), or one per `Application`? **Proposed**: per-`Application`, accessible via `Application::factory()`. | [runtime design](../plans/done/2026-05-01-runtime.design.md) | | #61 |
| **`ThreadPool` shutdown semantics**: Should `ThreadPool::drop` wait for in-flight tasks to complete (graceful) or abandon them? **Proposed**: graceful — close the sender and join all workers. Panic in a worker thread propagates as a resumed panic in `join()`. | [runtime design](../plans/done/2026-05-01-runtime.design.md) | ✅ done | |
