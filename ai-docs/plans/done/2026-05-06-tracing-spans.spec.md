# Tracing: replace announcements with spans; downgrade rename/clear_name

**Source:** user description
**Date:** 2026-05-06
**Tracked in:** #112

## Scope

1. Downgrade `debug!` → `trace!` for `object_tree::rename` and `object_tree::clear_name` (less significant mutations).
2. Replace every "function announcement" `debug!`/`trace!` call with a `debug_span!`/`trace_span!` guard (`.entered()`) at the top of the function, wrapping the entire body. Targets:
   - `quartzite-runtime/src/object_tree.rs`: `insert`, `reparent`, `rename`, `clear_name`, `destroy`
   - `quartzite-runtime/src/timer.rs`: `start`, `stop`
   - `quartzite-runtime/src/event_loop.rs`: `post`, `stop`
   - `quartzite-runtime/src/timer_drivers.rs`: pool driver `shutdown`
3. Add a `trace_span!` around `signal::emit_unconditionally` in `quartzite-core/src/signal.rs`, gated behind `#[cfg(feature = "verbose-tracing")]` — hot path, zero runtime cost when feature is off.
4. Convert `event_loop::post` to a gated `trace_span!` (same `verbose-tracing` gate) — dispatched once per queued signal, same frequency as the emitting path.
5. Introduce the `verbose-tracing` cargo feature in:
   - `quartzite-core/Cargo.toml`: `verbose-tracing = []`
   - `quartzite-runtime/Cargo.toml`: `verbose-tracing = ["quartzite-core/verbose-tracing"]`
   - Workspace root `Cargo.toml` (facade): `verbose-tracing = ["quartzite-core/verbose-tracing", "quartzite-runtime/verbose-tracing"]`
6. Update AGENTS.md tracing rule to document the span-over-announcement preference and the debug-vs-trace level rule.

## Out of scope

- Adding new tracing points beyond signal emit.
- Other signal.rs calls (`connect`, `disconnect`) — embedded in logic, not announcements; unchanged.
- Timer driver-level `start`/`stop` in `timer_drivers.rs` — deferred; significance needs future investigation.
- `info!`/`warn!`/`error!` calls (none exist currently).

## Deferred

- Timer driver `start`/`stop` logging | needs investigation of future use patterns | no separate issue needed yet

## Key decisions

| Question | Decision |
|---|---|
| Which `debug!` calls to downgrade? | `rename` and `clear_name` in `object_tree` — lower significance than insert/reparent/destroy |
| Span vs bare macro | Prefer `*_span!` with `.entered()` guard at function top for any function-level announcement |
| `insert` log is at fn end (post-op) | Option a: move span to fn top — consistent with other object_tree fns; infallible fn makes start/end distinction artificial |
| Sync vs async | All target functions are synchronous; `.entered()` guard is safe everywhere |
| Level for rename/clear_name spans | `trace_span!` (consistent with their downgraded level) |
| Level for all other spans | `debug_span!` |
| Signal emit + event_loop::post gate | `#[cfg(feature = "verbose-tracing")]` — both are on the dispatch hot path; zero runtime cost when feature is off |
| `verbose-tracing` feature propagation | `quartzite-core` (gate) → `quartzite-runtime` (re-exports core's) → workspace facade (re-exports both); enabling on any upper crate activates all lower crates |
| debug vs trace level | `debug_span!`/`debug!` for significant state mutations (object lifecycle, timer/event-loop lifecycle); `trace_span!`/`trace!` for supplementary or lower-significance operations (name changes, connection bookkeeping, posting) |

## Technical constraints

- All target functions are `fn` (not `async fn`); `.entered()` returns a guard dropped at end of scope.
- Span names: use `"module::function"` style (e.g. `"object_tree::insert"`).
- Field names: carry over from the original log call (e.g. `object_id = ?id`, `new_name`).
- `verbose-tracing` is a purely additive feature; enabling it must not change observable behaviour beyond enabling the span.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `object_tree::insert` opens with `let _span = debug_span!("object_tree::insert", object_id = ?id, parent_id = ?parent_id).entered();`; the old `debug!` at line 87 is removed |
| AC2 | `object_tree::reparent` opens with `let _span = debug_span!("object_tree::reparent", object_id = ?id, new_parent_id = ?new_parent).entered();`; the old `debug!` is removed |
| AC3 | `object_tree::rename` opens with `let _span = trace_span!("object_tree::rename", object_id = ?id, new_name = %new_name).entered();`; the old `debug!` is removed |
| AC4 | `object_tree::clear_name` opens with `let _span = trace_span!("object_tree::clear_name", object_id = ?id).entered();`; the old `debug!` is removed |
| AC5 | `object_tree::destroy` opens with `let _span = debug_span!("object_tree::destroy", object_id = ?id).entered();`; the old `debug!` is removed |
| AC6 | `timer::start` opens with `let _span = debug_span!("timer::start", timer_id = ?self.base.id()).entered();`; the old `debug!` is removed |
| AC7 | `timer::stop` opens with `let _span = debug_span!("timer::stop", timer_id = ?self.base.id()).entered();`; the old `debug!` is removed |
| AC8 | `event_loop::post` body is wrapped in `#[cfg(feature = "verbose-tracing")] let _span = trace_span!("event_loop::post").entered();`; the old `trace!` is removed |
| AC9 | `event_loop::stop` opens with `let _span = debug_span!("event_loop::stop").entered();`; the old `debug!` is removed |
| AC10 | `timer_drivers` pool driver `shutdown` opens with `let _span = debug_span!("pool_driver::shutdown").entered();`; the old `debug!` is removed |
| AC11 | `quartzite-core/Cargo.toml` gains `verbose-tracing = []`; `quartzite-runtime/Cargo.toml` gains `verbose-tracing = ["quartzite-core/verbose-tracing"]`; workspace root `Cargo.toml` gains `verbose-tracing = ["quartzite-core/verbose-tracing", "quartzite-runtime/verbose-tracing"]` |
| AC12 | `signal::emit_unconditionally` wraps its body in `#[cfg(feature = "verbose-tracing")] let _span = trace_span!("signal::emit", direct_slots = self.slots.len()).entered();`; the old unconditional `trace!` is removed |
| AC13 | With `verbose-tracing` disabled (default), the signal emit hot path has zero tracing overhead |
| AC14 | AGENTS.md tracing rule updated: (a) prefer `*_span!` with `.entered()` guard at function top over a bare announcement macro; (b) use `debug_span!`/`debug!` for significant mutations (object/timer/event-loop lifecycle), `trace_span!`/`trace!` for supplementary or lower-significance operations; (c) high-frequency paths (e.g. signal emit or event loops) gate spans behind `verbose-tracing` |
| AC15 | `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` all pass clean; also verified with `--features verbose-tracing` |

## Open questions

- None.
