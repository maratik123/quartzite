# Per-thread event loops

**Source:** issue #51
**Date:** 2026-05-06
**Tracked in:** #51

## Scope

- Process-wide `LoopRegistry` (`RwLock<HashMap<ThreadId, Arc<EventLoop>>>`) in `quartzite-runtime`
- `EventLoop::install_for_current_thread()` — explicit registration for the calling thread
- `EventLoop::uninstall_for_current_thread()` — explicit deregistration; also called automatically by `run()` on exit
- `EventLoop::spawn(f)` convenience API — spawns a thread with an installed, running loop and calls `f` on it (easy-install path)
- `Application::new()` auto-installs the main-thread `EventLoop` in `LoopRegistry`
- `Application::main_thread_id()` accessor
- `QueuedDispatcher::post` signature updated to carry `target: ThreadId`
- `ConnectionTable` impl routes to the target thread's loop via `LoopRegistry`; warns + drops if no loop is registered for the target thread
- Document the warn-and-drop behaviour explicitly in the `QueuedDispatcher` trait doc

## Out of scope

- `BlockingQueued` connection type (#48) — needs this plan but is a separate deliverable
- Object mobility / thread-affinity change (#52)
- `Auto` connection improvements beyond what per-thread loops already enable
- No-std path (event loops require `std`)
- Pluggable async executors

## Deferred

- `BlockingQueued` (#48) | needs per-thread loops first | separate issue already exists
- Stale `thread_id` invalidation on object migration | needs object-mobility API | #52

## Key decisions

| Question | Decision |
|---|---|
| `QueuedDispatcher::post` signature | `fn post(&self, target: ThreadId, f: Box<dyn FnOnce() + Send>)` — always explicit; no `Option<ThreadId>` |
| Fallback when target thread has no loop | `tracing::warn!` + drop `f`; documented in trait; no silent reroute to main |
| Easy-install path | `EventLoop::spawn(f)` spawns a thread, installs a fresh loop, calls `f`, then runs; returns `JoinHandle` |
| Full-control path | `install_for_current_thread()` + manual `run()` / `stop()` / `uninstall_for_current_thread()` |
| Registry location | `quartzite-runtime` (registry holds `Arc<EventLoop>`, which lives there); `quartzite-core` only sees `ThreadId` via the updated trait |
| Deregistration trigger | `EventLoop::run()` calls `uninstall_for_current_thread()` after queue drain; explicit `uninstall_for_current_thread()` also available |
| Main-thread loop | `Application::new()` installs its `EventLoop` on the calling thread; `Application::main_thread_id()` returns that `ThreadId` |

## Technical constraints

- `LoopRegistry` must be a process-wide singleton independent of `Application` so worker threads can register without an `Application` reference.
- `QueuedDispatcher` trait lives in `quartzite-core`; the trait signature change touches core, but the routing impl stays in `quartzite-runtime`.
- `LoopAlreadyInstalled` error on double-install (non-panicking, per AGENTS.md library-safety idioms).
- `EventLoop::spawn` must document that the spawned thread owns the loop for its lifetime; the `JoinHandle` joining is the only shutdown mechanism.
- File-size limits (AGENTS.md): if `event_loop.rs` exceeds soft limit after additions, split registry into `loop_registry.rs`.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `LoopRegistry` is a process-wide singleton (`RwLock<HashMap<ThreadId, Arc<EventLoop>>>`) in `quartzite-runtime`; readable without an `Application` being live. |
| AC2 | `EventLoop::install_for_current_thread(self: Arc<Self>) -> Result<(), LoopAlreadyInstalled>` registers the loop for the calling thread. |
| AC3 | `EventLoop::uninstall_for_current_thread()` removes the calling thread's entry from `LoopRegistry`; is a no-op when no entry exists. |
| AC4 | `EventLoop::run()` calls `uninstall_for_current_thread()` after queue drain and before returning, so the registry is always clean after a loop exits. |
| AC5 | `EventLoop::spawn(f: impl FnOnce() + Send + 'static) -> JoinHandle<()>` spawns a thread that installs a fresh loop, calls `f`, then runs the loop until stopped. |
| AC6 | `Application::new()` calls `install_for_current_thread()` on its main-thread `EventLoop`. |
| AC7 | `Application::main_thread_id() -> ThreadId` returns the `ThreadId` of the thread that called `Application::new()`. |
| AC8 | `QueuedDispatcher::post` signature is `fn post(&self, target: ThreadId, f: Box<dyn FnOnce() + Send>)`. |
| AC9 | `ConnectionTable` implements `QueuedDispatcher::post` by looking up `target` in `LoopRegistry` and posting to the found loop. |
| AC10 | When `LoopRegistry` has no entry for `target`, `ConnectionTable::post` emits `tracing::warn!` and drops `f` without executing it. |
| AC11 | The warn-and-drop behaviour for unregistered threads is documented on `QueuedDispatcher::post`. |
| AC12 | Integration test: a closure posted via `QueuedDispatcher` with a worker thread's `ThreadId` executes on that worker thread's loop, not the calling thread. |
| AC13 | Integration test: after a worker loop stops and its thread joins, posting to the now-deregistered `ThreadId` triggers the warn path and the closure does not execute. |

## Open questions

_(none)_
