# Design: Per-thread event loops

**Issue:** #51
**Date:** 2026-05-06

## Approach

### Chosen solution

Add a process-wide `LoopRegistry` (`parking_lot::RwLock<HashMap<ThreadId, Arc<EventLoop>>>`) as a module-level `OnceLock`-initialised singleton in `quartzite-runtime`. `EventLoop` gains `install_for_current_thread`, `uninstall_for_current_thread`, and `spawn` methods. `Application::new()` calls `install_for_current_thread` and stores the `ThreadId`. `QueuedDispatcher::post` gains a `target: ThreadId` first parameter; `ConnectionTable` uses that target to look up the right loop in `LoopRegistry`, emitting a `tracing::warn!` and dropping `f` when the thread is not registered.

The `target: ThreadId` added to `QueuedDispatcher::post` propagates to every call site in `quartzite-core` (`QueuedSlotInner`, `AutoSlotInner`, and the two paths in `connect.rs`). `QueuedSlotInner` does not currently carry a `ThreadId`, so `connect_queued` must gain a `receiver_thread_id: ThreadId` parameter. This cascades to:

- `Signal::connect_queued` — gains `receiver_thread_id`.
- `connect.rs` Queued branch — reads `to.lock().object_base().thread_id` and passes it.
- `Timer::connect_tick_queued` — gains `receiver_thread_id`.
- Proc-macro `emit_connect_queued_wrappers` — generated wrapper passes `receiver.thread_id`.
- `signal.rs` `TestDispatcher` and all `install_test_dispatcher` tests — `post` signature updated.

Because `event_loop.rs` (currently 240 lines) will grow by the `LoopRegistry` singleton, `install_for_current_thread`, `uninstall_for_current_thread`, and `spawn` (estimate: ~100 non-test lines), it will approach or exceed the 500-line soft limit when tests are included. The spec explicitly permits splitting into `loop_registry.rs`. The registry and its error type are placed in a new `quartzite-runtime/src/loop_registry.rs` file; `event_loop.rs` imports from it.

### Rejected alternatives

**`Option<ThreadId>` in `post`:** The spec explicitly decided against this. Explicit `ThreadId` is clearer and avoids silent routing to a "default" thread.

**Registry inside `Application`:** Would make worker-thread registration impossible without an `Application` reference, violating the spec constraint that the registry is process-wide and independent of `Application`.

**Single `OnceLock<Mutex<...>>` for the global singleton:** A `OnceLock<RwLock<HashMap<...>>>` avoids an extra `Arc` indirection. The `RwLock` is `parking_lot` (consistent with the rest of `quartzite-runtime`; the user specified this explicitly).

**Not changing `connect_queued` signature:** `QueuedSlotInner` must know the target `ThreadId` to include it in the `post` call. The only other option would be a `thread_local!` push/pop scheme or a second OnceLock per-connection, both of which are more complex and less explicit than adding a single `ThreadId` parameter.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `LoopRegistry` singleton and `LoopAlreadyInstalled` error in a new `loop_registry.rs`; re-export from `lib.rs` | `quartzite-runtime/src/loop_registry.rs`, `quartzite-runtime/src/lib.rs` | — |
| 2 | Add `install_for_current_thread`, `uninstall_for_current_thread`, `spawn` to `EventLoop`; `run()` wraps the post-loop cleanup in an inline `RegistryGuard` RAII struct (defined in `loop_registry.rs`, calls `uninstall_for_current_thread` on `Drop`) so cleanup executes even on panic-unwind | `quartzite-runtime/src/event_loop.rs`, `quartzite-runtime/src/loop_registry.rs` | 1 |
| 3 | Update `QueuedDispatcher::post` signature to `fn post(&self, target: ThreadId, f: Box<dyn FnOnce() + Send>)`; update `TestDispatcher` and all unit-test call sites in `quartzite-core` | `quartzite-core/src/signal.rs` | — |
| 4 | Add `receiver_thread_id: ThreadId` to `Signal::connect_queued`; update `QueuedSlotInner` to carry and forward it | `quartzite-core/src/signal.rs` | 3 |
| 5 | Update all `dispatcher.post(...)` call sites in `quartzite-core` to pass `ThreadId`: `connect.rs` `connect_signal_to_signal` Queued arm, `connect.rs` `connect_signals` Queued arm (reads `receiver.object_base().thread_id`), `connect.rs` Auto paths, and `AutoSlotInner::dispatch` | `quartzite-core/src/connect.rs`, `quartzite-core/src/signal.rs` | 3, 4 |
| 6 | Update `ConnectionTable`: remove `event_loop` field; implement `QueuedDispatcher::post(target, f)` via `LoopRegistry` lookup with `tracing::warn!`-and-drop on miss | `quartzite-runtime/src/connection_table.rs` | 1, 3 |
| 7 | Update `Application`: call `install_for_current_thread`, store `main_thread_id`, expose `main_thread_id() -> ThreadId`; update `ConnectionTable::new` call | `quartzite-runtime/src/application.rs` | 2, 6 |
| 8 | Update proc-macro `emit_connect_queued_wrappers` to pass `receiver.thread_id` to `connect_queued`; update codegen tests | `quartzite-macros/src/object/codegen.rs` | 4 |
| 9 | Update `Timer::connect_tick_queued` to accept and forward `receiver_thread_id: ThreadId` | `quartzite-runtime/src/timer.rs` | 4 |
| 10 | Add integration tests: cross-thread dispatch via `EventLoop::spawn`; warn-and-drop after loop exit | `quartzite-runtime/tests/per_thread_loops.rs` | 2, 6, 7 |

Ten tasks; all are direct AC requirements and cascade effects with no added abstractions. No split is proposed.

## Risks

- **`connect_queued` signature break**: Every call site of `Signal::connect_queued` must be updated simultaneously — `connect.rs`, `timer.rs`, proc-macro codegen, and test code. Missing one produces a compile error rather than silent misbehaviour, so the risk is contained to compile-time.
- **Test isolation for `LoopRegistry`**: The registry is a process-wide `OnceLock<RwLock<...>>` singleton. Integration tests that call `install_for_current_thread` from multiple test threads may collide. Mitigation: each integration test uses `EventLoop::spawn` (which creates a fresh thread) rather than installing on the test thread directly, or uses `uninstall_for_current_thread` in teardown.
- **`EventLoop::run` cleanup on panic**: Resolved by Task 2. An inline `RegistryGuard` RAII struct is defined in `loop_registry.rs` and entered at the start of `run()`'s post-install section; its `Drop` impl calls `uninstall_for_current_thread`, so the registry entry is removed even when `run` unwinds. No `scopeguard` dependency is needed — the struct is ~5 lines. Skeleton:

  ```rust
  struct RegistryGuard;
  impl Drop for RegistryGuard {
      fn drop(&mut self) {
          LoopRegistry::uninstall(std::thread::current().id());
      }
  }
  ```

  `run()` creates `let _guard = RegistryGuard;` immediately after `install_for_current_thread` succeeds.
- **`Application::new` on a non-main thread**: `Application::new` calls `install_for_current_thread`; if called from a non-main thread (valid today), `main_thread_id()` returns that thread's id. Mitigation: document that `Application::new` must be called from the main thread; this is consistent with existing behaviour and the spec.
- **`QueuedDispatcher` is a `std::sync::OnceLock` singleton**: The existing `QUEUED_DISPATCHER` in `quartzite-core` is frozen at first registration. Test binaries that call `set_queued_dispatcher` once cannot change it. Tests using the updated `TestDispatcher::post(target, f)` signature must update their assertion logic but the singleton-once constraint is unchanged.

## Test Design

### Task 1 — `LoopRegistry`

- **Location:** `quartzite-runtime/src/loop_registry.rs` `#[cfg(test)]` module
- **Entry points:** `LoopRegistry::install`, `LoopRegistry::uninstall`, `LoopRegistry::get`
- **Scenarios:**
  - Install an `Arc<EventLoop>` for the current thread; `get(current_id)` returns `Some`.
  - Double-install returns `Err(LoopAlreadyInstalled)` and the first entry is unchanged.
  - `uninstall` of an installed entry removes it; subsequent `get` returns `None`.
  - `uninstall` when no entry exists is a no-op (no panic).
  - Install on thread A, verify thread B cannot see it via `get(thread_b_id)`. This scenario requires a real `std::thread::spawn` to obtain a distinct `ThreadId`; use an `other_thread_id()` helper (same pattern as in `quartzite-core/src/signal.rs` tests, lines ~696–703: spawn a thread, capture its id in a channel, join, return the id).
- **Fixtures:** `Arc::new(EventLoop::new())` helper; `other_thread_id()` helper (spawns a thread, captures its `ThreadId`, joins).

### Task 2 — `EventLoop::install_for_current_thread` / `uninstall_for_current_thread` / `spawn`

- **Location:** `quartzite-runtime/src/event_loop.rs` `#[cfg(test)]` module + `quartzite-runtime/tests/event_loop.rs`
- **Entry points:** `install_for_current_thread`, `uninstall_for_current_thread`, `spawn`
- **Scenarios:**
  - After `install_for_current_thread`, `LoopRegistry::get(current_id)` returns the loop.
  - After `uninstall_for_current_thread`, `LoopRegistry::get` returns `None`.
  - `run()` calls `uninstall_for_current_thread` after the loop drains: call `install_for_current_thread`, then `stop()` from another thread, then `run()`, then verify registry entry is gone.
  - `spawn(f)`: `f` is called on the new thread; the loop is installed while `f` runs; after the handle is joined the entry is deregistered.
  - `install_for_current_thread` on an already-installed loop returns `Err(LoopAlreadyInstalled)`.
- **Fixtures:** helper that installs + runs + stops a loop on a fresh thread.

### Task 3 — `QueuedDispatcher::post` updated signature

- **Location:** `quartzite-core/src/signal.rs` `#[cfg(test)]` module
- **Entry points:** `TestDispatcher::post`
- **Scenarios:** Existing tests updated to pass a `ThreadId`; no new scenarios needed for the trait itself.

### Task 6 — `ConnectionTable::post` routing

- **Location:** `quartzite-runtime/src/connection_table.rs` `#[cfg(test)]` module
- **Entry points:** `ConnectionTable` as `QueuedDispatcher` implementation
- **Scenarios:**
  - Post to a registered thread id: closure executes on that thread's loop (requires a running loop; use `EventLoop::spawn`).
  - Post to an unregistered thread id: closure is dropped; `tracing::warn!` is emitted (verify via `tracing_subscriber` capture or simply verify the closure does not execute).
- **Fixtures:** none beyond existing `make_table()`; replace its `EventLoop` arg removal.

### Task 10 — Integration tests

- **Location:** `quartzite-runtime/tests/per_thread_loops.rs`
- **Entry points:** `EventLoop::spawn`, `QueuedDispatcher::post`, `Application::main_thread_id`
- **Scenarios (AC12):** Spawn a worker loop via `EventLoop::spawn`; post a closure that records `thread::current().id()`; join the handle; verify recorded id equals the worker thread's id and differs from the test-thread's id.
- **Scenarios (AC13):** Spawn a worker loop; record its `ThreadId`; stop it and join; post to the now-deregistered `ThreadId` via `ConnectionTable`; verify the closure did not execute.
- **Fixtures:** `Arc<AtomicBool>` or `Arc<Mutex<Option<ThreadId>>>` for result capture; a minimal `Application` (required for `ConnectionTable`).

## Open questions

_(none)_
