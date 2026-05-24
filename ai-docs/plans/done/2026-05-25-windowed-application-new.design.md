# Design: Add `WindowedApplication::new()` shorthand

**Issue:** #567
**Date:** 2026-05-25

## Approach

Add `WindowedApplication::new() -> Result<Self, RendererError>` as an `#[inline]` shorthand for `WindowedApplication::builder().build()` in `quartzite-renderer/src/application.rs`. The shape mirrors the precedent `Application::new()` shipped in PR #565 (`quartzite-runtime/src/application.rs:214-217`).

The body is a single delegating call `Self::builder().build()` — qualifies for `#[inline]` per AGENTS.md § *Code Style → `#[inline]` and the `_Simple._` doc tag* (concrete fn on a concrete type, ≤ 1 non-simple call, no branches/loops). Same shape as `Application::new()` which already carries `#[inline]`.

Doc surface follows AGENTS.md § *Documentation* / `ai-docs/doc-convention.md`: one-line `///` summary describing the shorthand relationship, `# Errors` block listing both variants `WindowedApplicationBuilder::build()` documents (`RendererError::Application`, `RendererError::EventLoop`), `# Examples` block with `no_run` (the example would construct a winit `EventLoop` which is not safe under doctests). Cannot be `const` because `WindowedApplicationBuilder::build()` calls `EventLoop::<AppEvent>::with_user_event().build()` which is not `const`.

Call-site sweep replaces every plain `WindowedApplication::builder().build()` call (no builder options set) with `WindowedApplication::new()` at the four sites enumerated in Spec § Scope item 2. The two opt-in sites (`tests/multi_window.rs:181` and `tests/support/mod.rs:128`) stay on the builder because they set `with_any_thread(true)`. The `xvfb_smoke.rs` prose at lines 14 and 19 gets the same rename in its `## Why this test does not go through ...` doc-comment header (Scope item 3; the test body bypasses both `new()` and `builder()`).

**Rejected alternatives:**
- *Rename `builder()` to `with_options()` and rely on `new()` everywhere.* Out of scope per Spec § Out of scope. Pre-publish API freedom does not require it; the builder remains the only entry that exposes `tick_duration` / `quit_on_last_window_closed` / `with_any_thread`.
- *Make `new()` a `const fn`.* Rejected per Spec § Key decisions: `WindowedApplicationBuilder::build()` is non-`const` (constructs a winit `EventLoop`).
- *Re-export `WindowedApplication::new` as `WindowedApplication::default()`.* Not equivalent — `Default::default()` cannot return `Result`, and a `Default` impl that panics is worse than the explicit `new() -> Result`.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `WindowedApplication::new() -> Result<Self, RendererError>` with `#[inline]`, one-line summary, `# Errors`, `# Examples` (`no_run`). Body: `Self::builder().build()`. Place between the existing `builder()` (line 51) and `from_parts()` (line 58) so the public-API entry points cluster at the top of `impl WindowedApplication`. | `quartzite-renderer/src/application.rs` | — |
| 2 | Add unit test `windowed_application_new_succeeds` in the existing `#[cfg(test)] mod tests` block of `quartzite-renderer/src/application.rs`. No platform gate needed: the test forces the early-return path by taking the `quartzite_runtime::Application` singleton via `Application::new()`, so `WindowedApplicationBuilder::build()` returns `Err(AlreadyExists)` at the `Application::builder().build()?` call (line 174 of `application_builder.rs`) **before** reaching `EventLoop::new()` — the winit main-thread check is never invoked, making the test platform-agnostic. Accept the triad documented in AC2 (`Ok`, `Application(AlreadyExists)`, `EventLoop(_)`) — accepting the broader set keeps the test robust if singleton state is shared with `quartzite_application_new_succeeds` running in parallel. | `quartzite-renderer/src/application.rs` | 1 |
| 3 | Update three `///` doc-comment examples to call `WindowedApplication::new()`: `application.rs:48` (`builder()` doc example), `application.rs:80` (`event_proxy` doc example), `application_builder.rs:21` (`AppEvent` doctest preamble — note the leading `# ` hidden-line marker stays). | `quartzite-renderer/src/application.rs`, `quartzite-renderer/src/application_builder.rs` | 1 |
| 4 | Update integration test `windowed_application_builder_returns_already_exists_on_second_call` at `tests/application.rs:20` to call `WindowedApplication::new()` instead of `WindowedApplication::builder().build()`. Also update the `//!` file-level doc at line 1 ("Integration test for `WindowedApplication::builder()` …") and the inline comment at line 15 so the prose still matches what the test exercises. The test name is intentionally NOT renamed (out-of-scope churn; the test still verifies the AlreadyExists path regardless of which entry point invokes it). | `quartzite-renderer/tests/application.rs` | 1 |
| 5 | Update `xvfb_smoke.rs` `//!` doc lines 14 and 19 to reference `WindowedApplication::new()` instead of `WindowedApplication::builder().build()`. Header text reads "Why this test does not go through `WindowedApplication::new()`". Test body is unchanged — it directly builds an `EventLoop` with `with_any_thread(true)` and never went through either entry point. | `quartzite-renderer/tests/xvfb_smoke.rs` | 1 |
| 6 | Run full AC6 + AC7 gate: `cargo build`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -- --check`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo build -p quartzite --no-default-features --features libm`. Fix any fallout from the rename (doctest compilation, intra-doc links). | (verification only) | 1, 2, 3, 4, 5 |

## Handoff plan

`M = 6` → two groups, 3 + 3:

- **Handoff entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
- **Group A:** subtasks 1–3 — add `new()`, add its unit test, rewrite the three production-source doc examples (`application.rs:48`, `application.rs:80`, `application_builder.rs:21`).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — terminal group. Rewrites the two test-file call sites / doc references and runs the AC6 + AC7 verification gate.

## Risks

- **Unit test panics on worker thread:** if subtask 2's test body fails to take the `Application` singleton first (e.g. another test already dropped it), `WindowedApplication::new()` will reach `EventLoop::new()` and panic on Linux worker threads instead of returning Err. *Mitigation:* take the singleton at the top of the test body via `let _app = Application::new();` (no `.unwrap()` — accept either `Ok` or `Err(AlreadyExists)`) and hold `_app` for the test's lifetime.

- **Test-binary singleton-state coupling:** the new test sits in the same binary as `quartzite_application_new_succeeds`, which also calls `Application::new()`. *Mitigation:* the existing test already tolerates this race (accepts `Ok || AlreadyExists`). The new test must do the same — accepting the triad means any drop-order outcome is valid.

- **Doctest hidden-line marker drift in `application_builder.rs:21`:** the doc example uses `# let app = WindowedApplication::builder().build().unwrap();` — the `# ` prefix hides the line from rendered docs but compiles it. The rewrite must preserve the `# ` marker: `# let app = WindowedApplication::new().unwrap();`. *Mitigation:* call out explicitly in subtask 3; verify via `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc` in subtask 6.

- **Doc-comment vs code-comment update split in `tests/application.rs`:** subtask 4 must update three different surfaces in one file — the `//!` module-level header at line 1, the inline `// …` comment at line 15, and the actual code at line 20. *Mitigation:* enumerated in subtask 4; subtask 6 catches anything missed.

- **Coordinate ordering between `new()` and `from_parts()`:** subtask 1 places `new()` between `builder()` and `from_parts()`. *Mitigation:* the placement is specified in subtask 1; subtask 6's `cargo doc` gate catches any layout regression.

## Test Design

**Subtask 2 — `windowed_application_new_succeeds` unit test**

- *Location:* `quartzite-renderer/src/application.rs` `#[cfg(test)] mod tests` block (existing block; no platform gate — the test forces the early-return path before `EventLoop::new()` and is therefore platform-agnostic).
- *Entry point:* `WindowedApplication::new()`.
- *Strategy:* take `Application` singleton via `let _app = Application::new();` then call `WindowedApplication::new()`. The `AlreadyExists` short-circuit in `WindowedApplicationBuilder::build()` at line 174 fires before the winit EventLoop is created, making the test safe on all platforms.
- *Assert shape:* mirrors `build_result_is_ok_or_already_exists` in `quartzite-renderer/src/application_builder.rs:259-281`.

**Subtask 6 — AC6 + AC7 gate + sweep**

Final `ast-index search "WindowedApplication::builder().build()"` sweep expecting zero hits outside `tests/multi_window.rs:181` and `tests/support/mod.rs:128` (the two opt-in sites kept unchanged per AC4).

## Open questions

- _(none)_
