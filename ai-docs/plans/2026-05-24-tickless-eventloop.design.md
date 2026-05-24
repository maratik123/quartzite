# Design: Tickless eventloop + Object-ify EventLoop and Application

**Issue:** #561
**Spec:** `ai-docs/plans/2026-05-24-tickless-eventloop.spec.md`
**Date:** 2026-05-24

## Approach

Two intertwined changes land in one PR: (1) replace the hard-coded 1 ms `recv_timeout` poll inside `EventLoop::run` with an `Option<Duration>` tick policy (tickless by default), exposed through a new `ApplicationBuilder` (parallel to `WindowedApplicationBuilder`); and (2) Object-ify `EventLoop` and `Application` so `stop` / `quit` become reflection-reachable `#[slot]`s.

**Tick policy storage.** The `tick: Option<Duration>` lives directly on `EventLoop` — no new `TickPolicy` enum (YAGNI; the spec § Key decisions row "Default impl on EventLoop's tick field internals" already mandates `Option<Duration>` on the struct). The `run()` loop branches once at entry between `Receiver::recv()` (tickless) and `Receiver::recv_timeout(d)` (tick-based) so the hot path stays branch-free per iteration. Channel-disconnect → exit in both branches.

**`Some(Duration::ZERO)` handling — normalise silently to `None`.** Chosen over the `Result`-rejecting alternative because: (a) the builder's `build()` already returns `Result<Application, ApplicationError>` for `AlreadyExists`; widening with a `ZeroTickDuration` variant adds a public error path that callers must match, while normalisation never fails; (b) `Duration::ZERO` causing a busy-loop is non-malicious-but-degenerate input — silently doing the sensible thing aligns with the workspace library-safety idiom (`parking_lot`-style infallible API). The normalisation happens in `ApplicationBuilder::build()`, `WindowedApplicationBuilder::build()`, and `EventLoop::with_tick`.

**Direct constructor naming on `EventLoop`.** `EventLoop::with_tick(Option<Duration>) -> Self` — matches the spec § Key decisions naming preference, parallels existing `with_any_thread` on `WindowedApplicationBuilder`. `EventLoop::new()` stays as a tickless convenience delegating to `with_tick(None)`. `EventLoop::spawn` gets the new shape `spawn(tick: Option<Duration>, f)` — single overload, explicit `None` at the call site.

**Builder method naming.** `tick_duration(Option<Duration>) -> Self` on both `ApplicationBuilder` and `WindowedApplicationBuilder` — matches the `quit_on_last_window_closed` chain shape (`const fn` chainable setter).

**`ApplicationBuilder` placement.** New file `quartzite-runtime/src/application_builder.rs` (mirrors the renderer-side sibling). Adding a builder + tick field + new docstring to `application.rs` (currently 301 lines) would push it toward the 400-line soft target; a sibling file keeps both under 200 + 200.

**Object-ification — path b1 (hand-rolled) chosen for `Application`.** Reasons: (a) `Application` is the *only* `Arc<Inner>` Object in the workspace today — extending the macro for one site violates YAGNI; (b) the precedent already exists (`Sender`/`Receiver`/`NullRecv`/`BoolRecv` in `connect.rs` are all hand-rolled). Note: the `connect.rs` precedents are owned, non-`Arc`-wrapped test-internal types; the `Application` b1 impl extends that pattern to an `Arc<Inner>` shape — it follows the same trait surface, but the `object_base_mut` forwarding is novel. The macro extension (b2) is deferred per the spec § Deferred row.

- `ObjectBase` moves into `ApplicationInner` (`base: ObjectBase` field).
- `AsObject` + `Object` hand-rolled on `Application`. `object_base(&self) -> &ObjectBase` forwards to `&self.0.base`.
- **`object_base_mut`** (b1 sub-option (ii) per spec § Open questions): `panic!` with the message `"Application singleton's ObjectBase cannot be mutated through the shared handle; use ObjectTree::rename for name changes"`. Documented in a `# Panics` block. A deferred row tracks the b2 macro-extension revisit.

**`request_stop(&self)` / `request_quit(&self)` shims.** The macro forces `stop(&mut self)` and `quit(&mut self)`, but:
- `EventLoop::stop` is called cross-thread via `Arc<EventLoop>` (spawned thread holds `Arc`; `Arc::get_mut` returns `None` while any clone is live). `&self` is the only viable receiver there.
- `Application::quit` is called from timer callbacks / doc examples via `Application::global().unwrap().quit()`. While the returned `Application` is an owned value (making `&mut` technically possible with `let mut app = …`), one-liner usage (`Application::global().unwrap().quit()`) is idiomatic and would break silently with an `&mut self` slot. To preserve the ergonomic one-liner pattern in doc examples and callbacks, a `&self` shim is the cleanest path.

The design adds:
- `pub fn request_stop(&self)` on `EventLoop` — `&self`-callable, identical body to the slot façade.
- `pub fn request_quit(&self)` on `Application` — `&self`-callable, identical body to the slot façade. Justified for ergonomic one-liner doc-example usage and closure-capture callbacks.
- `#[slot] pub fn stop(&mut self)` / `#[slot] pub fn quit(&mut self)` exist purely for the macro-reflection path.
- Cross-thread call sites and `Application::quit` (which calls `self.0.event_loop.request_stop()`) use the `request_*` shims. Spec AC21 is satisfied because existing call sites migrate to `request_stop` / `request_quit`.

**`#[object_impl]` placement.** Single-block `#[object_impl]` (terminal mode) on `EventLoop`'s inherent impl. The impl is ~200 lines — comfortably under any soft target. Multi-block accumulator adds compile-time complexity without justification.

**Test-only tick accessor.** AC9(d): `#[doc(hidden)] pub fn tick(&self) -> Option<Duration>` on `EventLoop` — exposed `pub` with `#[doc(hidden)]` so integration test binaries in `quartzite-runtime/tests/` (separate crates) can reach it, while avoiding visible public API. `pub(crate)` would be invisible to integration test binaries; behavioural observation is timing-flaky. This is the only public item gated by `#[doc(hidden)]` in `event_loop.rs`.

**`ObjectTree::contains` (not `contains_id`).** Spec AC20 cites `contains_id`; the live API is `contains`. Tests use `contains`.

## Rejected alternatives

1. **`EventLoopBuilder` parallel to `ApplicationBuilder`**: rejected by Round 2 Q1; direct ctor is sufficient.
2. **Adaptive tick**: out of scope per spec § Out of scope item 1.
3. **Macro extension `#[object(arc_inner)]`**: deferred. Single Application Object today.
4. **`Result`-rejecting `Some(Duration::ZERO)`**: rejected; normalisation is simpler.
5. **Inner-mutability via `RwLock<ObjectBase>` for `object_base_mut`**: rejected; can't return a real `&mut` through a guard.
6. **Keeping `stop(&self)` without a shim**: rejected; `Arc<EventLoop>` cross-thread callers need `&self`, but the macro requires `&mut self` for `#[slot]`. Two-method shape is the cleanest resolution.
7. **Splitting `Application::new()` deletion into a follow-up PR**: rejected; pre-publish clean break per AGENTS.md § API Stability.
8. **`pub(crate) fn tick()` accessor**: rejected; `pub(crate)` is invisible to integration test binaries. `pub #[doc(hidden)]` chosen instead.
9. **Dropping `request_quit`**: rejected; `Application::global().unwrap().quit()` one-liner is idiomatic and widespread in doc examples + callbacks; forcing `let mut app = …` everywhere degrades ergonomics for no benefit.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `tick: Option<Duration>` field to `EventLoop`; introduce `with_tick(tick: Option<Duration>) -> Self` (normalises `Some(ZERO)` → `None`); refactor `new()` to delegate to `with_tick(None)`; update `Default` impl; remove `const TICK_MS`; refactor `run()` to dispatch between `recv()` (tickless) and `recv_timeout(d)` (tick-based) with per-branch disconnect handling; update `spawn` to `spawn(tick: Option<Duration>, f)`; update module-level `//!` doc, all doc-strings, and existing `#[cfg(test)] mod tests` call sites for the new signatures. Add `#[doc(hidden)] pub fn tick(&self) -> Option<Duration>` accessor. | `quartzite-runtime/src/event_loop.rs` | — |
| 2 | Object-ify `EventLoop`: add `base: ObjectBase` field with `#[base]` marker; add `#[derive(Extend, Object)] #[root]` on the struct; wrap the inherent impl with `#[object_impl]`; annotate `stop` with `#[slot]` and change receiver to `&mut self`; add `pub fn request_stop(&self)` shim (identical body); add `///` doc for the new `base` field and `request_stop`. Update the inherent tests (`stop_terminates_run`, etc.) to use `request_stop` where cross-thread access is needed. | `quartzite-runtime/src/event_loop.rs` | 1 |
| 3 | Create `ApplicationBuilder` in new file `quartzite-runtime/src/application_builder.rs`: `#[must_use]` struct holding `tick: Option<Duration>`; chainable `tick_duration(Option<Duration>) -> Self` (`const fn`); `build() -> Result<Application, ApplicationError>` constructing `EventLoop::with_tick(tick)` and threading the install path currently in `Application::new`; move `ObjectBase` field into `ApplicationInner`; hand-roll `AsObject` + `Object` for `Application` (forwarding `object_base` to `&self.0.base`; `object_base_mut` panics with documented message; `invoke_method` arm for `"quit"` returns `Some(Value::Null)`; `connect_signal`/`emit_signal` return `None`); add static `MetaObject` with `class_name = "Application"`. Delete `Application::new`. Change `Application::quit` receiver to `&mut self`. Add `pub fn request_quit(&self)` shim with `///` doc. Expose `Application::builder() -> ApplicationBuilder` const fn. Wire `pub use` in `lib.rs`. Update all docstrings (module-level + per-item + `# Examples`). | `quartzite-runtime/src/application_builder.rs` (new), `quartzite-runtime/src/application.rs`, `quartzite-runtime/src/lib.rs` | 2 |
| 4 | Extend `WindowedApplicationBuilder` with `tick: Option<Duration>` field + `tick_duration(Option<Duration>) -> Self` const-fn setter (normalises `Some(ZERO)` → `None`); update `build()` to call `Application::builder().tick_duration(self.tick).build()?`. Doc updates per `missing_docs = "deny"`. | `quartzite-renderer/src/application_builder.rs` | 3 |
| 5 | Sweep all in-tree call sites of `Application::new()` (46 hits) → `Application::builder().build()`; update `EventLoop::spawn(f)` call sites → `EventLoop::spawn(None, f)`. Update `.quit()` call sites → `.request_quit()`; update `.stop()` call sites to use `request_stop()` where `&self` is needed. Update doc-string examples. `quartzite-runtime/src/loop_registry.rs` — no edit expected (uses `EventLoop::new()` which keeps its no-arg signature and does not call `.stop()`/`.quit()`), but must be included in the verification sweep (`grep -L 'Application::new' …` must return it) to confirm zero `Application::new` hits. | All files from the `grep -rln` sweep: `quartzite-runtime/{src/lib.rs,src/factory.rs,src/timer_drivers.rs,src/global_tree.rs,src/object_tree_ext.rs,src/loop_registry.rs,tests/timer.rs,tests/timer_single_shot_app.rs,tests/object_tree_ext.rs,tests/application.rs,tests/event_loop.rs,tests/per_thread_loops.rs}`, `quartzite-renderer/{src/application.rs,tests/application.rs,tests/multi_window.rs,tests/xvfb_smoke.rs}`, `quartzite-core/src/signal.rs`, `examples/timer.rs` | 4 |
| 6 | New tests: AC9 sub-items (a–e); AC15–AC20 Object-ification tests; AC19(a) reflection-based wiring; AC19(b) signal-connection-to-quit test. Unit tests in `event_loop.rs` `#[cfg(test)]`; integration tests in `tests/event_loop.rs` and `tests/application.rs` (isolated binaries for singleton tests). | `quartzite-runtime/src/event_loop.rs`, `quartzite-runtime/tests/event_loop.rs`, `quartzite-runtime/tests/application.rs`, `quartzite-runtime/src/application.rs` | 5 |
| 7 | Workspace gates: `cargo build`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -- --check`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo build -p quartzite --no-default-features --features libm`. Fix any failures. Update `ai-docs/learnings.md` if any in-flow learning emerges. | All touched files | 6 |

## Handoff plan

`M = 7` subtasks.

- **Group A (subtasks 1–3):** The orchestrator spawns Group A under `/context-reset` per `.claude/skills/context-reset/SKILL.md` at task entry. Covers `EventLoop` tick policy + `EventLoop` Object-ification + `ApplicationBuilder` + hand-rolled `Application` Object. Non-terminal group (3 subtasks).
- **Handoff after Group A:** orchestrator validates branch, `base_commit`, and `git diff --quiet`; then spawns Group B under `/context-reset`.
- **Group B (subtasks 4–6):** `WindowedApplicationBuilder` wiring + workspace-wide call-site sweep + new tests. Non-terminal group (3 subtasks).
- **Handoff after Group B:** orchestrator validates; then spawns Group C under `/context-reset`.
- **Group C (subtask 7):** terminal group (1 subtask). Workspace gates + shake-out.

## Risks

- **`request_stop` / `request_quit` adds public surface.** Two extra methods per type beyond the spec's letter. Justified: `request_stop` is necessary for `Arc<EventLoop>` cross-thread callers; `request_quit` preserves the ergonomic `Application::global().unwrap().request_quit()` one-liner that doc examples and callbacks use.
- **b1 `object_base_mut` panic.** Documented in `# Panics` block. Deferred row tracks revisit. Extends the `connect.rs` hand-rolled precedent to the `Arc<Inner>` shape — novel but structurally consistent.
- **`#[doc(hidden)] pub fn tick()`** adds a `pub` item that clippy / rustdoc may warn about. Mitigation: `#[doc(hidden)]` suppresses rustdoc; `#[allow(dead_code)]` is not needed if the integration tests reference it. If clippy fires `unreachable_pub`, add a scoped `#[allow(unreachable_pub)]` on that item only.
- **`ObjectTree::contains` naming.** Live method is `contains` (not `contains_id`). Tests use the correct name.
- **File size.** `application.rs` (301 lines + ~80 lines hand-rolled impl = ~381 lines) stays inside the 400-line soft target. If it crosses 400, extract the meta-impl into `application_meta.rs`.
- **Performance.** Tickless removes 1 wake-up/ms in idle loops — a measurable win. Timer-driver delivery latency is bounded by the driver's `park_timeout` / `condvar::wait_for`, not the loop tick (verified: spec § Technical constraints "Timer-driver wake-up in both modes").

## Test design

### Task 1 — tick policy (unit tests in `event_loop.rs`)
- `EventLoop::new()` → `tick = None` (via `tick()` accessor).
- `EventLoop::with_tick(Some(50ms))` → `tick = Some(50ms)`.
- `EventLoop::with_tick(Some(ZERO))` → `tick = None` (normalisation — AC9(e)).
- Existing `stop_terminates_run` passes with tickless default.

### Task 2 — `EventLoop` Object (unit + integration)
- `meta_object().class_name == "EventLoop"`.
- `object_base()` returns valid `&ObjectBase`; `id()` non-zero.
- `el.invoke_method("stop", &[])` returns `Some(Value::Null)`; loop exits.
- **AC9(a):** tickless `run` exits when spawned thread calls `request_stop` — assert `handle.join()` completes within 200 ms.
- **AC9(b):** tickless `EventLoop` processes a posted closure with no spurious wake-ups — post exactly one counter-incrementing closure, start the loop, idle 100 ms, call `request_stop`, join the thread, assert counter == 1 (only the one legitimate wake occurred; any spurious wake would over-count).
- **AC9(c):** tick-based `EventLoop` with `Some(50ms)` drives `stop()` to completion — post a tick-counting closure that increments a shared `AtomicU32`; call `request_stop` after 200 ms; `handle.join()` must complete within 300 ms total; counter value must be ≥ 2 (at least 2 ticks fired).

### Task 3 — `ApplicationBuilder` + `Application` Object (isolated integration binaries)
- `builder().build()` succeeds; second call returns `Err(AlreadyExists)`.
- `tick_duration(Some(50ms))` propagates to `EventLoop::tick()` (via `#[doc(hidden)] pub fn tick()`).
- `meta_object().class_name == "Application"`.
- `invoke_method("quit", &[])` returns `Some(Value::Null)`.
- `id().into_inner() != 0`.
- `app.object_tree().lock().contains(app.id()) == false`.
- `tick_duration(Some(ZERO))` normalises to tickless.
- `object_base_mut` panics with documented message (via `std::panic::catch_unwind`).

### Task 4 — `WindowedApplicationBuilder::tick_duration` (unit tests)
- `tick_duration(Some(50ms))` propagates; default is `None`.
- `quit_on_last_window_closed` chain composes independently.

### Task 6 — AC9(d) + AC19 reflection-based wiring
- **AC9(d):** `ApplicationBuilder::tick_duration(Some(50ms)).build()` → `Application`'s internal `EventLoop` has `tick() == Some(50ms)`. Integration binary; reads via `#[doc(hidden)] pub fn tick()`.
- **AC19(a):** Direct `invoke_method("stop", &[])` on a standalone `EventLoop` — assert returns `Some(Value::Null)` and loop thread exits.
- **AC19(a):** Direct `invoke_method("quit", &[])` on the `Application` singleton — assert returns `Some(Value::Null)`.
- **AC19(b):** Create a `Signal<()>`, connect a closure `|_| Application::global().unwrap().request_quit()` to it (no captured handle needed — `global()` returns a fresh `Arc` ref), start `app.exec()` in a thread, fire the signal, assert the thread exits within 200 ms. Demonstrates that any signal source (e.g. a `Button`'s `clicked`) wires to `Application::quit` via the `&self` shim without a mutable handle. Test lives in a separate integration binary `quartzite-runtime/tests/application_signal_to_quit.rs` to avoid `OnceLock` conflicts with the existing `tests/application.rs` singleton.

## Open questions resolved

- `Some(Duration::ZERO)`: normalise to `None` silently.
- Constructor name: `EventLoop::with_tick(Option<Duration>) -> Self`.
- Builder setter name: `tick_duration(Option<Duration>) -> Self`.
- `ApplicationBuilder` placement: new sibling file `application_builder.rs`.
- `Application` Object path: b1 (hand-rolled).
- `object_base_mut` on `Application`: panic with documented message.
- Cross-thread stop/quit: `request_stop(&self)` / `request_quit(&self)` shims.
- `#[object_impl]` placement: single-block terminal mode.
- Test-only tick accessor: `#[doc(hidden)] pub fn tick(&self)` (not `pub(crate)` — integration tests need visibility).
- `ObjectTree::contains` (not `contains_id`) in AC20 test. Spec AC20 uses `contains_id` (non-existent API); spec has been amended to use `contains` — cosmetic rename, AC semantics unchanged.
- AC18 amended (self-review round 1): `Application(Arc<ApplicationInner>)` is a tuple struct; `#[slot]` annotation cannot be used (requires `#[object_impl]` which requires `#[derive(Object)]`). AC18 now describes the b1 hand-rolled `invoke_method` dispatch, names `request_quit(&self)` as the idiomatic signal-connection shim, and documents `Application::global()` as the `&mut self` path (no `Clone` — `global_tree::deregister()` is a flat `AtomicBool::store(false)`, so multiple handles would prematurely clear the flag; `Application` stays non-Clone). AC19(b) added: `Signal<()>` test using `Application::global().unwrap().request_quit()` in a separate integration binary to avoid OnceLock conflicts.

## Key files

- `quartzite-runtime/src/event_loop.rs` — primary change target
- `quartzite-runtime/src/application.rs` — `Application` Object-ification
- `quartzite-runtime/src/application_builder.rs` (new)
- `quartzite-runtime/src/lib.rs` — pub use
- `quartzite-runtime/src/timer.rs` — precedent for `#[derive(Extend, Object)] + #[object_impl]`
- `quartzite-renderer/src/application_builder.rs` — `tick_duration` setter
- `quartzite-core/src/connect.rs` (lines 400–494) — hand-rolled `AsObject`/`Object` precedent
- `quartzite-core/src/traits.rs` — `AsObject` / `Object` trait definitions
- `quartzite-core/src/object_base.rs` — `ObjectBase` struct
