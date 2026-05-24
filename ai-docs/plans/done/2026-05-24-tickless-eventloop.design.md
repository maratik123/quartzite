# Design: Tickless eventloop + Object-ify EventLoop and Application

**Issue:** #561
**Spec:** `ai-docs/plans/2026-05-24-tickless-eventloop.spec.md`
**Date:** 2026-05-24

## Approach

Two intertwined changes land in one PR: (1) replace the hard-coded 1 ms `recv_timeout` poll inside `EventLoop::run` with an `Option<Duration>` tick policy (tickless by default), exposed through a new `ApplicationBuilder` (parallel to `WindowedApplicationBuilder`); and (2) Object-ify `EventLoop` and `Application` so `stop` / `quit` become reflection-reachable `#[slot]`s.

**Tick policy storage.** The `tick: Option<Duration>` lives directly on `EventLoop` — no new `TickPolicy` enum (YAGNI; the spec § Key decisions row "Default impl on EventLoop's tick field internals" already mandates `Option<Duration>` on the struct). The `run()` loop branches once at entry between `Receiver::recv()` (tickless) and `Receiver::recv_timeout(d)` (tick-based) so the hot path stays branch-free per iteration. Channel-disconnect → exit in both branches.

**`Some(Duration::ZERO)` handling — normalise silently to `None`.** Chosen over the `Result`-rejecting alternative because: (a) the builder's `build()` already returns `Result<Application, ApplicationError>` for `AlreadyExists`; widening with a `ZeroTickDuration` variant adds a public error path that callers must match, while normalisation never fails; (b) `Duration::ZERO` causing a busy-loop is non-malicious-but-degenerate input — silently doing the sensible thing aligns with the workspace library-safety idiom (`parking_lot`-style infallible API). The normalisation happens in `ApplicationBuilder::build()`, `WindowedApplicationBuilder::build()`, and `EventLoop::with_tick`.

**Direct constructor naming on `EventLoop`.** `EventLoop::with_tick(Option<Duration>) -> Self` — matches the spec § Key decisions naming preference, parallels existing `with_any_thread` on `WindowedApplicationBuilder`. `EventLoop::new()` stays as a tickless convenience delegating to `with_tick(None)`. `EventLoop::spawn` gets the new shape `spawn(tick: Option<Duration>, f)` — single overload, explicit `None` at the call site.

**Builder method naming.** `tick_duration(Option<Duration>) -> Self` on both `ApplicationBuilder` and `WindowedApplicationBuilder` — matches the `quit_on_last_window_closed` chain shape (`const fn` chainable setter). Note: `tick_duration` is **not** `#[inline]` (it contains a `match` branch for `Some(Duration::ZERO)` normalisation — see `## Amendment 2026-05-24 (Round 1 review)` § Decision 2); `quit_on_last_window_closed` is `#[inline]` because it is branch-free.

**`ApplicationBuilder` placement.** New file `quartzite-runtime/src/application_builder.rs` (mirrors the renderer-side sibling). Adding a builder + tick field + new docstring to `application.rs` (currently 301 lines) would push it toward the 400-line soft target; a sibling file keeps both under 200 + 200.

**Object-ification — path b1 (hand-rolled) chosen for `Application`.** Reasons: (a) `Application` is the *only* `Arc<Inner>` Object in the workspace today — extending the macro for one site violates YAGNI; (b) the precedent already exists (`Sender`/`Receiver`/`NullRecv`/`BoolRecv` in `connect.rs` are all hand-rolled). Note: the `connect.rs` precedents are owned, non-`Arc`-wrapped test-internal types; the `Application` b1 impl extends that pattern to an `Arc<Inner>` shape — it follows the same trait surface, but the `object_base_mut` forwarding is novel. The macro extension (b2) is deferred per the spec § Deferred row.

- `ObjectBase` moves into `ApplicationInner` (`base: ObjectBase` field).
- `AsObject` + `Object` hand-rolled on `Application`. `object_base(&self) -> &ObjectBase` forwards to `&self.0.base`.
- **`object_base_mut`** (b1 sub-option (ii) per spec § Open questions): `panic!` with the message `"Application singleton's ObjectBase cannot be mutated through the shared handle; use ObjectTree::rename for name changes"`. Documented in a `# Panics` block. A deferred row tracks the b2 macro-extension revisit.

**`request_stop(&self)` shim / `quit(&self)` unified receiver.** The macro forces `stop(&mut self)`, but `EventLoop::stop` is called cross-thread via `Arc<EventLoop>` (spawned thread holds `Arc`; `Arc::get_mut` returns `None` while any clone is live). `&self` is the only viable receiver there — hence a `pub fn request_stop(&self)` shim alongside `#[slot] stop(&mut self)`.

For `Application::quit`, the initial design added a `request_quit(&self)` shim by the same reasoning. In a follow-up refactor (post-merge) the two methods were merged: `quit` receiver changed to `&self`, which is valid because the hand-rolled `invoke_method(&mut self, …)` arm coerces `&mut Application` → `&Application` at the call site. The merged shape removes a name to learn and makes closures, signal connections, and `invoke_method` all use the same symbol.

The design adds:
- `pub fn request_stop(&self)` on `EventLoop` — `&self`-callable, identical body to the `#[slot] stop` façade.
- `pub fn quit(&self)` on `Application` — single method, callable from any context including `invoke_method` dispatch.
- `#[slot] pub fn stop(&mut self)` on `EventLoop` exists for the macro-reflection path.
- Cross-thread call sites use `request_stop`; `Application::quit` is called as-is from all contexts.

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
7. **Deleting `Application::new()`**: reversed during Round 1 review (see `## Amendment 2026-05-24 (Round 1 review)` § Decision 1). The function is retained as an `#[inline]` tickless-default shorthand. The original rationale (pre-publish clean break per AGENTS.md § API Stability) was outweighed by the zero-cost convenience across 46 in-tree call sites.
8. **`pub(crate) fn tick()` accessor**: rejected; `pub(crate)` is invisible to integration test binaries. `pub #[doc(hidden)]` chosen instead.
9. **Dropping `request_quit` initially**: deferred to a follow-up commit; `quit` receiver changed to `&self` post-merge, making `request_quit` redundant and eliminating it cleanly.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `tick: Option<Duration>` field to `EventLoop`; introduce `with_tick(tick: Option<Duration>) -> Self` (normalises `Some(ZERO)` → `None`); refactor `new()` to delegate to `with_tick(None)`; update `Default` impl; remove `const TICK_MS`; refactor `run()` to dispatch between `recv()` (tickless) and `recv_timeout(d)` (tick-based) with per-branch disconnect handling; update `spawn` to `spawn(tick: Option<Duration>, f)`; update module-level `//!` doc, all doc-strings, and existing `#[cfg(test)] mod tests` call sites for the new signatures. Add `#[doc(hidden)] pub fn tick(&self) -> Option<Duration>` accessor. | `quartzite-runtime/src/event_loop.rs` | — |
| 2 | Object-ify `EventLoop`: add `base: ObjectBase` field with `#[base]` marker; add `#[derive(Extend, Object)] #[root]` on the struct; wrap the inherent impl with `#[object_impl]`; annotate `stop` with `#[slot]` and change receiver to `&mut self`; add `pub fn request_stop(&self)` shim (identical body); add `///` doc for the new `base` field and `request_stop`. Update the inherent tests (`stop_terminates_run`, etc.) to use `request_stop` where cross-thread access is needed. | `quartzite-runtime/src/event_loop.rs` | 1 |
| 3 | Create `ApplicationBuilder` in new file `quartzite-runtime/src/application_builder.rs`: `#[must_use]` struct holding `tick: Option<Duration>`; chainable `tick_duration(Option<Duration>) -> Self` (`const fn`, not `#[inline]` — has `match` branch); `build() -> Result<Application, ApplicationError>` constructing `EventLoop::with_tick(tick)` and threading the install path previously in `Application::new`; move `ObjectBase` field into `ApplicationInner`; hand-roll `AsObject` + `Object` for `Application` (forwarding `object_base` to `&self.0.base`; `object_base_mut` panics with documented message; `invoke_method` arm for `"quit"` returns `Some(Value::Null)`; `connect_signal`/`emit_signal` return `None`); add static `MetaObject` with `class_name = "Application"`. Retain `Application::new` as `#[inline]` shorthand for `Self::builder().build()` (see Amendment 2026-05-24 (Round 1 review) § Decision 1). Add `pub fn quit(&self)` (receiver `&self` — works from any context including `invoke_method` dispatch). Expose `Application::builder() -> ApplicationBuilder` const fn. Wire `pub use` in `lib.rs`. Update all docstrings (module-level + per-item + `# Examples`). | `quartzite-runtime/src/application_builder.rs` (new), `quartzite-runtime/src/application.rs`, `quartzite-runtime/src/lib.rs` | 2 |
| 4 | Extend `WindowedApplicationBuilder` with `tick: Option<Duration>` field + `tick_duration(Option<Duration>) -> Self` const-fn setter (normalises `Some(ZERO)` → `None`); update `build()` to call `Application::builder().tick_duration(self.tick).build()?`. Doc updates per `missing_docs = "deny"`. | `quartzite-renderer/src/application_builder.rs` | 3 |
| 5 | Sweep all in-tree call sites of `Application::new()` (46 hits) → `Application::builder().build()`; update `EventLoop::spawn(f)` call sites → `EventLoop::spawn(None, f)`. Update `.stop()` call sites to use `request_stop()` where `&self` is needed. Update doc-string examples. `quartzite-runtime/src/loop_registry.rs` — no edit expected (uses `EventLoop::new()` which keeps its no-arg signature and does not call `.stop()`/`.quit()`), but must be included in the verification sweep to confirm all in-tree callers compile against the current API (no removed call sites remain). Note: `Application::new()` is retained as an `#[inline]` shorthand (see `## Amendment 2026-05-24 (Round 1 review)` § Decision 1), so call sites that previously used `Application::new()` for the default tickless path remain valid; the sweep migrates them to `Application::builder().build()` at author discretion but is not invalidated by a residual `Application::new()` site. | All files from the `grep -rln` sweep: `quartzite-runtime/{src/lib.rs,src/factory.rs,src/timer_drivers.rs,src/global_tree.rs,src/object_tree_ext.rs,src/loop_registry.rs,tests/timer.rs,tests/timer_single_shot_app.rs,tests/object_tree_ext.rs,tests/application.rs,tests/event_loop.rs,tests/per_thread_loops.rs}`, `quartzite-renderer/{src/application.rs,tests/application.rs,tests/multi_window.rs,tests/xvfb_smoke.rs}`, `quartzite-core/src/signal.rs`, `examples/timer.rs` | 4 |
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

- **`request_stop` adds one extra public method on `EventLoop`.** Necessary: `Arc<EventLoop>` cross-thread callers need `&self`; the `#[slot]` macro shape forces `&mut self` on `stop`. `Application::quit` takes `&self` directly — no shim needed there.
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
- **AC19(b):** Create a `Signal<()>`, connect a closure `|()| Application::global().unwrap().quit()` to it (no captured handle needed — `global()` returns a fresh `Arc` ref), start `app.exec()` in a thread, fire the signal, assert the thread exits within 200 ms. Demonstrates that any signal source (e.g. a `Button`'s `clicked`) wires to `Application::quit` without a mutable handle. Test lives in a separate integration binary `quartzite-runtime/tests/application_signal_to_quit.rs` to avoid `OnceLock` conflicts with the existing `tests/application.rs` singleton.

## Open questions resolved

- `Some(Duration::ZERO)`: normalise to `None` silently.
- Constructor name: `EventLoop::with_tick(Option<Duration>) -> Self`.
- Builder setter name: `tick_duration(Option<Duration>) -> Self`.
- `ApplicationBuilder` placement: new sibling file `application_builder.rs`.
- `Application` Object path: b1 (hand-rolled).
- `object_base_mut` on `Application`: panic with documented message.
- Cross-thread stop: `request_stop(&self)` shim on `EventLoop`. `Application::quit` takes `&self` directly.
- `#[object_impl]` placement: single-block terminal mode.
- Test-only tick accessor: `#[doc(hidden)] pub fn tick(&self)` (not `pub(crate)` — integration tests need visibility).
- `ObjectTree::contains` (not `contains_id`) in AC20 test. Spec AC20 uses `contains_id` (non-existent API); spec has been amended to use `contains` — cosmetic rename, AC semantics unchanged.
- AC18 amended (self-review round 1): `Application(Arc<ApplicationInner>)` is a tuple struct; `#[slot]` annotation cannot be used (requires `#[object_impl]` which requires `#[derive(Object)]`). AC18 now describes the b1 hand-rolled `invoke_method` dispatch. `Application::quit` takes `&self` — ergonomic from any context; `invoke_method` arm coerces `&mut Application` → `&Application` at the call site. AC19(b) added: `Signal<()>` test using `Application::global().unwrap().quit()` in a separate integration binary to avoid OnceLock conflicts.

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

## Amendment 2026-05-24 — AC22 `connect_signal_to_slot` + AC23 reserved class name

### AC22 — `connect_signal_to_slot`

**Scope:** new public function in `quartzite-core/src/connect.rs`, alongside the existing `connect_signal_to_signal`.

**Signature** (mirrors `connect_signal_to_signal`'s target shape):
```rust
pub fn connect_signal_to_slot(
    source: &mut dyn Object,
    signal_name: &str,
    target: &Arc<Mutex<dyn Object>>,
    slot_name: &str,
) -> Result<ConnectionId, SignalConnectionError>
```

`Mutex` is `parking_lot::Mutex` (workspace default). `Arc<Mutex<dyn Object>>` enables a `Weak` downgrade so the callback does not keep the target alive — same lifetime pattern as `connect_signal_to_signal`.

**Behaviour:**
- Validates `signal_name` via `source.meta_object().signal(signal_name)` — returns `SignalConnectionError::UnknownFromSignal` if absent.
- Slot-name validation is lazy: the registered callback calls `target.invoke_method(slot_name, &[])` on each emission and silently ignores a `None` return (unknown slot = no-op). Trade-off documented in the function's `///` doc.
- Target handle capture: the callback holds `Weak<Mutex<dyn Object>>` (downgraded from the passed `Arc`); on emission it upgrades, locks, and calls `invoke_method(slot_name, &[])` — identical lifecycle to `connect_signal_to_signal`'s forwarder.
- Callback shape: `Box<dyn Fn(&[Value]) + Send + Sync>` — `Fn` (not `FnMut`); takes `&[Value]` (args forwarded from the signal emission but unused since slots are zero-arg). This matches `SignalCallback = Box<dyn Fn(&[Value]) + Send + Sync>` at `quartzite-core/src/traits.rs`.
- Zero-arg slot constraint: the callback invokes `invoke_method(slot_name, &[])` with empty args — valid only for zero-arg slots. Non-zero-arg slots silently no-op (same as unknown slot). Documented in `# Notes`.

**`Application` use case:**
```rust
use parking_lot::Mutex;
use std::sync::Arc;

let app = Application::global().unwrap();  // Application is Send + Sync
let target: Arc<Mutex<dyn Object>> = Arc::new(Mutex::new(app));
connect_signal_to_slot(&mut source_obj, "click", &target, "quit")?;
// When "click" fires: target is locked, invoke_method("quit", &[]) called,
// which dispatches to Application::quit(&self) → exec() returns.
```
`Application::global()` constructs a fresh handle via `Arc::clone` of the singleton inner; wrapping it in `Mutex` is a one-shot connect-time cost.

**Error variants:** `UnknownFromSignal` already exists in `SignalConnectionError`. No new variants needed.

**Handoff plan for the amendment (standalone):**
`M = 1` (subtask 8 only — subtasks 1–7 are already merged).

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 8 | Add `connect_signal_to_slot` to `quartzite-core/src/connect.rs` following the `connect_signal_to_signal` precedent (`Weak<Mutex<dyn Object>>` lifetime, `Fn(&[Value]) + Send + Sync` callback, `UnknownFromSignal` validation, lazy slot validation); add AC23 doc note to `Application` type doc; add integration test in `quartzite-runtime/tests/connect_signal_to_slot.rs` (separate binary — Application singleton) that wraps `Application::global().unwrap()` in `Arc<Mutex<dyn Object>>`, wires a `Signal<()>`-backed source object's "click" to "quit", asserts `exec()` returns within 200 ms. Run workspace gates. | `quartzite-core/src/connect.rs`, `quartzite-runtime/src/application.rs` (doc only), `quartzite-runtime/tests/connect_signal_to_slot.rs` (new) | — |

**Group A (subtask 8):** terminal group (1 subtask). Spawn under `/context-reset` at amendment entry.

### AC23 — `Application` reserved class name

**Scope:** one-line note in `Application`'s type-level `///` doc in `quartzite-runtime/src/application.rs`.

**Content:** "The class name `\"Application\"` is reserved for this framework-managed singleton. User-created objects should not use `\"Application\"` as their `meta_object().class_name`."

**Decomposition:** folded into subtask 8 (same commit).

## Amendment 2026-05-24 (Round 1 review)

Two clarifications applied during `/pr-commented` Round 1 of PR #565, both already landed in the implementation.

### Decision 1 — keep `Application::new()` as a tickless-default shorthand

Spec AC3 (Round 1 amendment): `Application::new() -> Result<Self, ApplicationError>` is retained, **not** removed. It is a boilerplate reducer that delegates to `Self::builder().build()`. The original spec position (delete the function as a pre-publish clean break per AGENTS.md § API Stability) was reversed during review because the convenience shorthand carries zero behavioural cost (tickless is already the builder default) and removes call-site noise across 46 in-tree sites that would otherwise spell `Application::builder().build()` for the default path.

**Shape** (`quartzite-runtime/src/application.rs`):
```rust
#[inline]
pub fn new() -> Result<Self, ApplicationError> {
    Self::builder().build()
}
```

`#[inline]` is correct here: the body is a single non-branching call (`Self::builder().build()`) — simple per AGENTS.md § Code Style → `#[inline]` and the `_Simple._` doc tag (concrete inherent fn, no branches/loops, ≤ 1 non-simple call).

This restores `Application::new()` without re-introducing any of the pre-existing semantics: the constructor goes through the same builder path, with tickless as the default. No alias for any other removed API is reintroduced; the clean-break stance from AGENTS.md § API Stability still applies to every other surface in this PR.

### Decision 2 — `#[inline]` removed from `tick_duration` setters

Spec § Technical constraints (Round 1 amendment): `ApplicationBuilder::tick_duration` and `WindowedApplicationBuilder::tick_duration` contain a `match` expression (branching) for `Some(Duration::ZERO)` normalisation:

```rust
self.tick = match tick {
    Some(d) if d.is_zero() => None,
    other => other,
};
```

Per AGENTS.md § Code Style → `#[inline]` and the `_Simple._` doc tag, the simplicity rule is "no branches/loops, ≤ 1 non-simple call". The `match` is a branch and disqualifies the function from `#[inline]`. The attribute has been removed from both setters.

The neighbouring `const fn` modifier is retained — `const fn` is independent of `#[inline]` and remains valid for the `match` body.

### Decomposition impact — none

Subtasks 1–8 remain unchanged:
- Subtask 3 (`ApplicationBuilder` construction) already produces `tick_duration` without `#[inline]` per the amended technical-constraints text. No structural rework needed; the marker simply isn't applied.
- Subtask 4 (`WindowedApplicationBuilder::tick_duration`) — same.
- Subtask 3 (deletion of `Application::new`) becomes addition-by-retention. The shorthand is one inlined line living next to `builder()`; it does not change the `ApplicationBuilder` structure, the `Object`-ification path, the file split, or the call-site sweep in Subtask 5 (which migrates *other* call sites — `Application::new()` callers can stay or migrate to `builder().build()` at the author's discretion).
- Subtask 5 (in-tree call-site sweep) — the sweep's invariant ("no `Application::new()` callers remain") softens to "all callers compile against the current API"; since `Application::new()` is the same shape as before (`-> Result<Self, ApplicationError>`), existing callers that survived the merge still compile unchanged. No additional file touched.
- Subtask 6 (new tests) — no test is added or removed; AC9(d) and AC22 still observe the same behaviour via the builder path.
- Subtask 7 (workspace gates) — unchanged.
- Subtask 8 (AC22/AC23) — unchanged.

### Handoff plan — unchanged

`M = 7` (subtasks 1–7, primary tickless + Object PR) + `M = 1` (subtask 8, AC22/AC23 amendment) — both already merged. The Round 1 review amendment introduces no new subtask; the two implementation changes (one-line `Application::new()` restoration + two `#[inline]` removals) landed inside the existing Group C / amendment scope and the subtask groupings above continue to describe the merged PR state.
