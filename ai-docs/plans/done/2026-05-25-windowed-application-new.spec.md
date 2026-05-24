# Add `WindowedApplication::new()` shorthand

**Source:** issue #567
**Date:** 2026-05-25
**Tracked in:** #567

## Scope

1. Add `WindowedApplication::new() -> Result<Self, RendererError>` as an
   `#[inline]` shorthand for `WindowedApplication::builder().build()` in
   `quartzite-renderer/src/application.rs`.
2. Update every `WindowedApplication::builder().build()` call site that
   sets no builder options to use `WindowedApplication::new()`:
   - `quartzite-renderer/src/application.rs:48` — `builder()` doc example.
   - `quartzite-renderer/src/application.rs:80` — `event_proxy` doc example.
   - `quartzite-renderer/src/application_builder.rs:21` — `AppEvent` doc
     example.
   - `quartzite-renderer/tests/application.rs:20` — integration test
     `windowed_application_builder_returns_already_exists_on_second_call`.
3. Update the `xvfb_smoke.rs` doc comments at lines 14 and 19 that
   currently reference `WindowedApplication::builder().build()` so the
   prose matches the new idiomatic call (`WindowedApplication::new()`).
   The test body itself does not call the shorthand — it bypasses both
   `builder()` and `new()` to set `with_any_thread(true)`.
4. Add at least one unit test in `quartzite-renderer/src/application.rs`
   `#[cfg(test)] mod tests` block covering `WindowedApplication::new()`
   (parallel to the existing `quartzite_application_new_succeeds` test).

## Out of scope

- Renaming, moving, or removing `WindowedApplication::builder()` — the
  builder stays available for callers that need
  `tick_duration` / `quit_on_last_window_closed` / `with_any_thread`.
- Changing `WindowedApplicationBuilder::build()` signature or behaviour.
- Changes to `Application::new()` in `quartzite-runtime` (already shipped
  via #565).
- Touching `quartzite-renderer/tests/multi_window.rs:181` — it sets
  `.with_any_thread(true)`, so the builder form is required.
- Touching `quartzite-renderer/tests/support/mod.rs:128` — same reason
  (the test-support helper sets options on the builder).

## Deferred

- _(none — single-cycle shorthand mirror; no follow-up items expected.)_

## Key decisions

| Question | Decision |
|---|---|
| Signature shape | `pub fn new() -> Result<Self, RendererError>` — mirrors `Application::new()` in `quartzite-runtime/src/application.rs:215`. Not `const` because `WindowedApplicationBuilder::build()` is not `const` (it creates a `winit::EventLoop`). |
| `#[inline]` marker | Yes — body is a single delegating call (`Self::builder().build()`), which matches the *Simple* rule in AGENTS.md § *Code Style → `#[inline]` and the `_Simple._` doc tag* (concrete fn, ≤ 1 non-simple call). `Application::new()` carries `#[inline]` for the same reason. |
| Doc shape | One-line `///` summary + `# Errors` block listing the same two variants `WindowedApplicationBuilder::build()` documents (`RendererError::Application`, `RendererError::EventLoop`) + `# Examples` block (per AGENTS.md § *Documentation*). |
| `xvfb_smoke.rs` doc comments | Update prose to reference `WindowedApplication::new()` instead of `WindowedApplication::builder().build()` — the explanation ("we bypass the standard constructor because winit's main-thread check panics on worker threads") still applies; only the name of the standard constructor changes. |

## Technical constraints

- `WindowedApplicationBuilder::build()` returns
  `Result<WindowedApplication, RendererError>`. The new shorthand
  delegates without changing the error contract.
- `WindowedApplicationBuilder::new()` is `pub(crate) const`; the public
  entry stays through `WindowedApplication::builder()`.
- AGENTS.md § *API Stability* — pre-publish, no shim layer needed.
- AGENTS.md § *Code Style → `#[inline]`*: mark the new shorthand
  `#[inline]` (concrete fn, simple body).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `WindowedApplication::new() -> Result<Self, RendererError>` exists in `quartzite-renderer/src/application.rs`, carries `#[inline]`, and delegates to `WindowedApplication::builder().build()`. |
| AC2 | A `#[cfg(test)]` unit test in `quartzite-renderer/src/application.rs` (`windowed_application_new_succeeds`, parallel to the existing `quartzite_application_new_succeeds`) asserts that `WindowedApplication::new()` returns `Ok` or one of the platform-acceptable error variants (`RendererError::Application(ApplicationError::AlreadyExists)`, `RendererError::EventLoop(_)` — same triad the existing `build_result_is_ok_or_already_exists` test in `application_builder.rs` accepts). |
| AC3 | Every `WindowedApplication::builder().build()` call site enumerated in *Scope* item 2 has been rewritten to `WindowedApplication::new()`. |
| AC4 | Every `WindowedApplication::builder().build()` call site with options set (`multi_window.rs:181`, `tests/support/mod.rs:128`) is **unchanged**. |
| AC5 | `xvfb_smoke.rs` doc comments at lines 14 and 19 reference `WindowedApplication::new()` (or use a single phrasing that names the new shorthand) instead of `WindowedApplication::builder().build()`. |
| AC6 | `cargo build`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -- --check`, and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` all pass. |
| AC7 | `cargo build -p quartzite --no-default-features --features libm` still passes (workspace AGENTS.md gate). |

## Open questions

- _(none — the issue body specifies the API shape, the call-site sweep, and the precedent (PR #565 `Application::new()`) is mergeable without further input.)_

```yaml
---
status: ready
round: 1
---
```
