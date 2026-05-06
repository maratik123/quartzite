# Design: Tracing — replace announcements with spans; downgrade rename/clear_name

**Issue:** #112
**Date:** 2026-05-06

## Approach

All current tracing calls in the target functions are "announcement" logs — a bare
`debug!`/`trace!` fired at the top (or bottom, for `insert`) of the function to record
entry. The tracing crate's span model is a better fit: a `debug_span!`/`trace_span!` with
`.entered()` at function entry produces an RAII guard that:

- captures the span name and fields once on creation,
- automatically closes (exits) when the guard is dropped at end of scope,
- nests correctly with any parent spans the caller holds.

Replacing announcement macros with span guards therefore gives richer context (span
duration, nesting) with no additional lines of code.

### Level mapping (from spec key decisions)

| Functions | Old call | New span macro |
|---|---|---|
| `object_tree::insert`, `reparent`, `destroy` | `debug!` | `debug_span!` |
| `timer::start`, `timer::stop` | `debug!` | `debug_span!` |
| `event_loop::stop` | `debug!` | `debug_span!` |
| `timer_drivers::PoolDriver::drop` (shutdown) | `debug!` | `debug_span!` |
| `object_tree::rename`, `clear_name` | `debug!` | `trace_span!` (downgrade) |
| `event_loop::post` | `trace!` | `trace_span!` gated behind `verbose-tracing` |
| `signal::emit_unconditionally` | `trace!` | `trace_span!` gated behind `verbose-tracing` |

### Gate for hot paths

`signal::emit_unconditionally` and `event_loop::post` are on the dispatch hot path —
called once per queued signal. Their tracing is wrapped in `#[cfg(feature = "verbose-tracing")]`
so the default build incurs zero overhead.

### `verbose-tracing` feature propagation

The feature is purely additive and must be defined in:
- `quartzite-core/Cargo.toml`: `verbose-tracing = []`
- `quartzite-runtime/Cargo.toml`: `verbose-tracing = ["quartzite-core/verbose-tracing"]`
- workspace root `Cargo.toml`: `verbose-tracing = ["quartzite-core/verbose-tracing", "quartzite-runtime/verbose-tracing"]`

### `insert` placement fix

`object_tree::insert` currently logs at line 87 (after the insertion logic). The span
guard is placed at the top of the function instead, consistent with every other function.
For an infallible function the start/end distinction is artificial, and uniform placement
aids readability.

### Rejected alternatives

- **Instrument macro (`#[instrument]`)**: adds hidden `async`-aware machinery, requires
  the full `tracing` feature set, and generates a wrapper function rather than a guard.
  For simple synchronous functions the manual `let _span = …_span!(…).entered()` pattern
  is less surprising and aligns with the existing style.
- **Keep bare macros**: loses duration information and hierarchical nesting; spec requires
  spans.
- **Gate all spans behind `verbose-tracing`**: over-restricts visibility of lifecycle
  events (insert/destroy/start/stop) that are genuinely useful at `debug` level in
  non-hot paths.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `verbose-tracing` feature to all three Cargo.toml files | `quartzite-core/Cargo.toml`, `quartzite-runtime/Cargo.toml`, `Cargo.toml` | — |
| 2 | Replace `debug!` announcements with `debug_span!` guards in `object_tree` (`insert`, `reparent`, `destroy`); downgrade `rename` and `clear_name` to `trace_span!`; fix `insert` placement from line-end to line-start | `quartzite-runtime/src/object_tree.rs` | — |
| 3 | Replace `debug!` announcements with `debug_span!` guards in `timer::start` and `timer::stop`; update `use` imports | `quartzite-runtime/src/timer.rs` | — |
| 4 | Replace `debug!`/`trace!` announcements in `event_loop::stop` and `event_loop::post`; gate `post` span behind `#[cfg(feature = "verbose-tracing")]`; update `use` imports | `quartzite-runtime/src/event_loop.rs` | 1 |
| 5 | Replace `debug!` in `PoolDriver::drop` (shutdown) with `debug_span!`; update `use` import | `quartzite-runtime/src/timer_drivers.rs` | — |
| 6 | Gate `signal::emit_unconditionally` trace in `#[cfg(feature = "verbose-tracing")]` `trace_span!`; update the `use tracing::trace` import | `quartzite-core/src/signal.rs` | 1 |
| 7 | Update AGENTS.md tracing rule: span-over-announcement preference, debug-vs-trace level rule, hot-path `verbose-tracing` gate | `AGENTS.md` | — |

Tasks 1–3, 5, and 7 have no dependency on each other and can be done in any order.
Task 4 depends on task 1 (needs the feature defined before it can be referenced).
Task 6 depends on task 1 for the same reason.

## Risks

- **Import churn causes compile errors**: removing `use tracing::debug` when also adding
  `use tracing::debug_span` (and similarly `trace`/`trace_span`) must be done atomically
  within each file. Mitigation: each file is a single atomic task; compiler catches missing
  imports immediately.
- **`#[cfg(feature = "verbose-tracing")]` on a local `let` binding**: Rust allows
  `#[cfg(...)]` on `let` statements in expression position since edition 2024. The
  workspace uses edition 2024, so this is safe. Mitigation: confirmed by edition in
  `Cargo.toml` (`edition = "2024"`).
- **`tracing` default-features = false in `quartzite-core`**: `quartzite-core` already
  depends on `tracing` with `default-features = false`. The `verbose-tracing` feature flag
  introduces no new `tracing` features — `trace_span!` is available with no features.
  Mitigation: no change to the `tracing` dependency line needed.
- **No `log` feature in core tracing dep**: `quartzite-core/Cargo.toml` has tracing
  without `features = ["log"]`, while `quartzite-runtime` has `features = ["log"]`. The
  `verbose-tracing` spans in `quartzite-core` will therefore not forward to `log` by
  default. This is by design — they are opt-in spans. No mitigation needed.
- **AGENTS.md rule update must also propagate to agent/skill files**: per the Propagation
  Rule in AGENTS.md, any change to a rule in AGENTS.md must be grepped across
  `.claude/agents/` and `.claude/skills/` and applied to matching files. Mitigation: task
  7 includes that grep step.

## Test Design

No new logic is introduced. The changes are mechanical substitutions of tracing macros for
span guards and feature-flag additions. There are no new code paths to unit-test at the
function level.

The acceptance criteria are verified by the CI gate (AC15):

- `cargo build` — confirms the feature additions and import changes compile.
- `cargo test` — confirms existing tests still pass with the new import/span usage.
- `cargo clippy -- -D warnings` — confirms no lint regressions (unused imports removed, no
  dead code from the `#[cfg]` guards).
- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` — confirms
  no doc regressions.
- All of the above also run with `--features verbose-tracing` to exercise the gated code
  paths.

No additional test cases are required because:
- The span guard `let _span = …_span!(…).entered()` pattern is idiomatic and relies solely
  on `tracing`'s own drop semantics — not new logic.
- `#[cfg(feature = "verbose-tracing")]` on a `let` binding compiles to a no-op when the
  feature is absent; the guarded line is the only change to the hot-path function bodies.
- Existing tests for `object_tree`, `timer`, `event_loop`, `signal` cover the observable
  behaviour of those functions; span emission does not change observable behaviour.

## Open questions

- None.
