# Design: ReceiverGuard for Auto Connections

**Issue:** #50
**Date:** 2026-05-03

## Approach

### Summary

Three targeted changes across two crates:

1. **`quartzite-core/src/signal.rs`** — add `guard: Weak<ReceiverGuard>` to `AutoSlotInner`; check it in `DynAutoSlot::dispatch` on both the same-thread (direct) and cross-thread (post) paths; update `Signal::connect_auto` signature to accept `guard: Weak<ReceiverGuard>` as a second parameter (after `receiver_thread_id`, before `f`).
2. **`quartzite-core/tests/`** — update six existing `connect_auto` call sites to pass a guard; add two new guard-expiry tests; update `auto_no_dispatcher.rs` (one call site).
3. **`quartzite-macros/src/object/codegen.rs`** — add a new `emit_connect_auto_wrappers` function that generates `connect_<signal>_auto` methods on `impl TypeName` — gated `#[cfg(feature = "std")]`, `#[inline]`, extracting `thread_id` and `Weak<ReceiverGuard>` from a `receiver: &::quartzite::core::ObjectBase` argument.

### Guard check in `dispatch`

```
fn dispatch(&self, emit_thread_id: ThreadId, args: &Args) {
    if self.guard.upgrade().is_none() { return; }  // both paths
    if emit_thread_id == self.receiver_thread_id {
        (self.callback)(args.clone());
    } else if let Some(dispatcher) = queued_dispatcher() {
        ...post...
    }
}
```

The guard check comes first, before any branching on thread equality, so both paths are uniformly gated.

### `connect_auto` new signature

```rust
pub fn connect_auto<F>(
    &mut self,
    receiver_thread_id: std::thread::ThreadId,
    guard: std::sync::Weak<ReceiverGuard>,
    f: F,
) -> ConnectionId
```

The docstring example is updated accordingly. The docstring previously omitted the guard; it now mirrors `connect_queued` in style.

### Codegen wrapper

For each `#[signal]` field `foo_bar: Signal<Args>` the generated method is:

```rust
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[inline]
pub fn connect_foo_bar_auto<F>(
    &mut self,
    receiver: &::quartzite::core::ObjectBase,
    f: F,
) -> ::quartzite::core::ConnectionId
where
    F: Fn(Args) + Send + Sync + 'static,
    // Args: Clone + Send is on connect_auto itself, propagates via monomorphization
{
    self.foo_bar.connect_auto(
        receiver.thread_id,
        ::std::sync::Arc::downgrade(receiver.receiver_guard()),
        f,
    )
}
```

The wrapper uses `receiver.thread_id` (public field) and `receiver.receiver_guard()` (public method returning `&Arc<ReceiverGuard>`), both already accessible via `::quartzite::core::ObjectBase`. The `where` clause on `Args` is not restated in the wrapper — it propagates from `connect_auto` itself. Since this wrapper is `#[inline]` and a simple delegation, the compiler will inline it.

The method lives in the same outer `impl TypeName` block that already contains the `emit_<signal>` wrappers. No new impl block is introduced.

### Rejected alternatives

- **Separate `connect_auto_guarded` method** — would leave the unguarded `connect_auto` as a footgun. Changing the signature is a minor breaking change but there are only test call sites.
- **Storing `Weak<ReceiverGuard>` as `Option<Weak<…>>`** — unnecessary complexity; callers that have no guard can supply `Weak::new()` (always dead), but that pattern is not needed for this scope.
- **Guard check only on the cross-thread path** — spec AC1 requires same-thread path to also skip if guard is dead. Uniform upfront check is simpler.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `guard` field to `AutoSlotInner`; check in `dispatch` (both paths) | `quartzite-core/src/signal.rs` | — |
| 2 | Update `Signal::connect_auto` signature: add `guard` param; update docstring + doctest | `quartzite-core/src/signal.rs` | 1 |
| 3 | Update existing `connect_auto` call sites in unit tests and integration tests | `quartzite-core/src/signal.rs` (6 tests), `quartzite-core/tests/auto_no_dispatcher.rs` | 2 |
| 4 | Add new tests: `auto_same_thread_slot_not_called_after_receiver_destroyed` and `auto_cross_thread_slot_not_posted_after_receiver_destroyed` | `quartzite-core/src/signal.rs` | 3 |
| 5 | Generate `connect_<signal>_auto` method in `#[derive(Object)]` codegen | `quartzite-macros/src/object/codegen.rs` | 2 |
| 6 | Add codegen test: `connect_auto_wrapper_generated_for_signal` | `quartzite-macros/src/object/codegen.rs` | 5 |

## Risks

- **Signature breakage of `connect_auto`:** Only test call sites exist (no production callers per spec). All are in `quartzite-core/src/signal.rs` and `tests/auto_no_dispatcher.rs`. The compiler enforces the fix at task 3. Risk: low.
- **`Weak::new()` in test doctests:** The `connect_auto` docstring example uses `thread::current().id()` directly. It must now also supply a guard. Use `ReceiverGuard::new_pair()` in the example (or `Weak::new()` if only illustrating the call, since doctest should compile). Use `Weak::new()` in the docstring `# Examples` block so it compiles without an `ObjectBase` dependency in `quartzite-core`.
- **No-std path (`cargo build -p quartzite --no-default-features`):** `AutoSlotInner`, `connect_auto`, and the new guard check are all inside `#[cfg(feature = "std")]`. The generated wrapper is gated `#[cfg(feature = "std")]`. No impact on the no-std path. Risk: none.
- **Codegen `#[cfg_attr(docsrs, ...)]`**: The existing `emit_<signal>` wrappers do not carry `#[cfg_attr(docsrs, doc(cfg(feature = "std")))]` because they are always available. The new `connect_<signal>_auto` wrappers are `#[cfg(feature = "std")]` so they must carry the docsrs attribute.
- **`thread_id` access in codegen**: `ObjectBase::thread_id` is a `pub` field in the struct. Codegen-emitted code accesses it as `receiver.thread_id`. If it were ever made private this would break; but it is documented as `pub` and the spec says "both accessible in codegen-emitted code via `::quartzite::core`". Risk: none for this change.

## Test Design

### Task 3 — Updated call sites (regression guard)

No new test logic; existing tests confirm the old semantics still hold after the signature change. Each updated call site passes `Weak::new()` (always dead guard — but the tests that verify live delivery pass a *live* guard, i.e., must use `ReceiverGuard::new_pair()` and hold the `Arc` alive). Review each test to determine which guard to use:

- Tests asserting the slot *is called*: must pass a live guard (hold `Arc` alive for the test duration).
- Tests asserting the slot *is not called* due to disconnect: guard liveness does not matter; `Weak::new()` is fine.
- `auto_no_dispatcher.rs`: slot is never called (silent drop); `Weak::new()` is fine.

### Task 4 — New guard-expiry tests

**Location:** `quartzite-core/src/signal.rs` `#[cfg(test)]` module

**`auto_same_thread_slot_not_called_after_receiver_destroyed`**

- Entry point: `Signal::emit`
- Setup: `install_test_dispatcher()`, `ReceiverGuard::new_pair()`, connect with same-thread `thread_id` and `Weak` from the pair, drop the `Arc`.
- Emit once, assert `called` remains `false`.
- Assert dispatcher queue length did not grow (no post occurred either).
- Fixtures: `Arc<AtomicBool>`, shared `TestDispatcher` (via `install_test_dispatcher()`), `serial` annotation.

**`auto_cross_thread_slot_not_posted_after_receiver_destroyed`**

- Entry point: `Signal::emit`
- Setup: `install_test_dispatcher()`, `other_thread_id()`, `ReceiverGuard::new_pair()`, connect with foreign thread id and `Weak`, drop the `Arc`.
- Emit once, assert no new closures were added to `dispatcher.posted` (compare `pre_len` before and after).
- Fixtures: same as above plus `other_thread_id()` helper.

### Task 5+6 — Codegen test

**Location:** `quartzite-macros/src/object/codegen.rs` `#[cfg(test)]` module

**`connect_auto_wrapper_generated_for_signal`**

- Entry point: `emit()` (the test helper that runs `parse` then `codegen`) with a single `#[signal]` field.
- Assert: output string contains `pub fn connect_foo_auto` (exact name for a signal `foo`).
- Assert: output contains `# [cfg (feature = \"std\")]` before the wrapper.
- Assert: output contains `# [inline]`.
- Assert: output contains `receiver . thread_id` and `receiver_guard`.
- Assert: wrapper is present in the outer `impl Foo` block, not inside `mod __quartzite_Foo`.

## Open questions

_(none)_
