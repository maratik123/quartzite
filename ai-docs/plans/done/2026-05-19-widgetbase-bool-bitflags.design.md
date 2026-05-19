# Design: Collapse `WidgetBase` bool fields into `BitFlags<WidgetState>`

**Issue:** #480
**Date:** 2026-05-19

## Approach

Replace the six `bool` fields on `WidgetBase` (`visible`, `enabled`, `pending_update`, `hovered`, `pressed`, `focused`) with a single `pub state: BitFlags<WidgetState>` field, where `WidgetState` is a new `#[bitflags] #[repr(u8)] pub enum` declared in `quartzite-widgets/src/widget_base.rs`. The `WidgetExt` accessor surface (`is_visible` / `set_visible` / `show` / `hide` / `is_enabled` / `set_enabled` / `is_hovered` / `set_hovered` / `is_pressed` / `set_pressed` / `is_focused` / `set_focused` / `update`) keeps byte-identical public signatures; only their bodies change to delegate to `BitFlags::{contains, insert, remove}` or `state.set(WidgetState::Foo, value)`.

**Why `enumflags2::BitFlags<WidgetState>` and not a hand-rolled `bitflags!` macro:**

- The issue body and spec explicitly name `BitFlags<WidgetState>`.
- `enumflags2 = "0.7"` is already in the workspace (`quartzite-core/Cargo.toml`, `quartzite-events/Cargo.toml`) and `no_std`-compatible — AC12 (`--no-default-features --features libm`) keeps passing.
- Three existing workspace types use the exact same `#[bitflags] #[repr(u8)] enum Foo { ... = 0b…; }` + `pub type FooSet = BitFlags<Foo>;` pattern (`MouseButton` / `MouseButtons` in `quartzite-events/src/mouse.rs`, `KeyModifier` / `KeyModifiers` in `quartzite-events/src/keyboard.rs`, `PropertyFlag` / `PropertyFlags` in `quartzite-core/src/meta.rs`). Reusing it keeps the workspace internally consistent.
- The variant enum (`WidgetState`) is itself useful as a function argument — callers can write `state.contains(WidgetState::Visible)` without needing a separate constant module.

**Why a single `pub state: BitFlags<WidgetState>` field (no getter/setter wrapper, no per-bool shim methods on `WidgetBase` directly):**

- The current `WidgetBase` shape already exposes ten `pub` fields (`geometry`, `font`, `palette`, `layout`, `min_size`, `max_size`, `focus_policy`, `size_policy`, `cursor`, `event_filters`); keeping the new state field `pub` matches that convention.
- Per `AGENTS.md` § *API Stability*: pre-publish, the workspace prefers clean breaks. Renaming six pub bools to one pub `BitFlags` field is exactly that — no `#[deprecated]` wrappers, no per-bool `fn visible(&self) -> bool` accessors duplicating `WidgetExt::is_visible`.
- `WidgetExt`'s blanket impl (`impl<T: AsWidget> WidgetExt for T`) already supplies every read/write accessor downstream callers need.

**Default value:** `WidgetState::Enabled.into()` — exactly one bit set, mapping today's `enabled: true, visible: false, pending_update: false, hovered: false, pressed: false, focused: false` semantics. This is preferred over `#[bitflags(default = Enabled)]` because (a) the workspace's `Default for WidgetBase` already exists and constructs the whole struct explicitly, and (b) the existing `#[bitflags(default = ...)]` usage (`PropertyFlag` in `meta.rs`) is for callers that derive `Default` on the `BitFlags` itself — `WidgetBase::new()` directly sets the field, so the explicit `WidgetState::Enabled.into()` is more readable.

**Bit values:** following the `MouseButton` convention — `Visible = 0b0000_0001` through `Focused = 0b0010_0000`, in declaration order. Six variants fit in `u8`.

**Variant ordering rationale:** declaration order mirrors the original field declaration order in `WidgetBase` (`visible`, `enabled`, `pending_update`, `hovered`, `pressed`, `focused`), so the rename is mechanically traceable.

**Rejected alternatives:**

- *Hand-rolled `bitflags!` macro from the `bitflags` crate.* Adds a new top-level dep when `enumflags2` is already in-tree; loses the named variant enum (the `bitflags!` macro produces an opaque struct with associated constants, not an enum that can be matched / passed by value). Inconsistent with the rest of the workspace.
- *Per-bool `pub fn visible(&self) -> bool` shims on `WidgetBase` mirroring the old field names.* Duplicates the existing `WidgetExt::is_visible` accessor, violates the pre-publish clean-break rule, and grows the inherent-method surface for zero benefit.
- *Private `state: BitFlags<WidgetState>` with `pub fn state()` getter + `set_state()` setter.* Inconsistent with the other `pub` fields on `WidgetBase`; adds bodies that are pure plumbing.
- *Splitting `WidgetState` into two enums (e.g. `VisibilityState` + `InputState`).* YAGNI — the six bits are semantically a single widget-state vector; no consumer wants to take a `VisibilityState` without also reading the input bits.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Declare `pub enum WidgetState` (6 variants, `#[bitflags] #[repr(u8)]`, doc comments mirroring the field-level docs) in `quartzite-widgets/src/widget_base.rs`. Add the `use enumflags2::{BitFlags, bitflags};` import. Add `enumflags2 = "0.7"` to `quartzite-widgets/Cargo.toml` `[dependencies]`. **Do not yet remove the bool fields** — keep the struct compiling so subsequent steps can be land-tested incrementally. | `quartzite-widgets/src/widget_base.rs`, `quartzite-widgets/Cargo.toml` | — |
| 2 | Replace the 6 `pub <name>: bool` fields on `WidgetBase` with a single `pub state: BitFlags<WidgetState>`. Update `WidgetBase::new()` to initialise `state: WidgetState::Enabled.into()`. Remove the `#[allow(clippy::struct_excessive_bools, …)]` annotation (the struct now has zero `bool` fields, so the lint no longer fires). Update the two doc-tests on `WidgetBase::new` and `Default::default` (lines 94–95 and 130) to use `state.contains(...)` reads. Update the `#[cfg(test)] mod tests` block (lines 247–287) — `new_widget_base_defaults`, `as_widget_self_ref`, `show_hide`, `update_sets_pending` — to read `state` via `contains`. Add `WidgetState` to the `pub use widget_base::{…}` re-export line in `lib.rs` so doc-tests and external callers use the terse `quartzite_widgets::WidgetState` path. | `quartzite-widgets/src/widget_base.rs`, `quartzite-widgets/src/lib.rs` | 1 |
| 3 | Update every `WidgetExt` accessor body in `quartzite-widgets/src/widget_ext.rs` (14 total touch points = 13 accessor bodies + 1 doc-test at line 364, matching spec § Scope item 4: `show` / `hide` writes; `is_visible` / `set_visible` / `is_enabled` / `set_enabled` reads+writes; `is_hovered` / `set_hovered` / `is_pressed` / `set_pressed` / `is_focused` / `set_focused` reads+writes; `update` write). Reads → `self.widget_base().state.contains(WidgetState::Foo)`. Writes-true → `self.widget_base_mut().state.insert(WidgetState::Foo)`. Writes-false → `self.widget_base_mut().state.remove(WidgetState::Foo)`. Variable writes (e.g. `set_visible(visible: bool)` body) → `self.widget_base_mut().state.set(WidgetState::Foo, value)` — `BitFlags::set(flag, bool)` from `enumflags2 ≥ 0.7.4` toggles a single bit. Also update the `update` doc-test snippet at line 364 (`assert!(w.widget_base().pending_update)` → `assert!(w.widget_base().state.contains(WidgetState::PendingUpdate))`). | `quartzite-widgets/src/widget_ext.rs` | 2 |
| 4 | Update the `#[cfg(test)] mod tests` block in `quartzite-widgets/src/widget_ext.rs` (5 read sites at lines 601, 712, 720, 727, 735) — rewrite `w.widget_base().pending_update / .pressed / .focused` reads to `state.contains(...)`. Test names and bodies stay otherwise unchanged; this is a read-shape rewrite, not a behavioural test edit. | `quartzite-widgets/src/widget_ext.rs` | 3 |
| 5 | Update `quartzite-style-dispatch/src/dispatch.rs` lines 152 + 163 (`if !widget.widget_base().visible` and `if !child.widget_base().visible`) — replace with `if !widget.is_visible()` and `if !child.is_visible()`. Using the `WidgetExt::is_visible()` accessor (rather than `state.contains(WidgetState::Visible)`) avoids importing `WidgetState` into this crate and routes the check through the canonical accessor. | `quartzite-style-dispatch/src/dispatch.rs` | 3 |
| 6 | Run the full gate locally: `cargo fmt`, `cargo build`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo build -p quartzite --no-default-features --features libm`. Re-run AC13 grep: `grep -rnE '\.(visible\|enabled\|pending_update\|hovered\|pressed\|focused)\b' quartzite-widgets/src quartzite-style-dispatch/src quartzite-style/src` returns zero hits on the six collapsed names as field accesses (false positives on `return_pressed`, `pressed_buttons`, `MouseButton::*` etc. inspected and filtered). Update `Cargo.lock` if changed. | repo-wide gate | 1, 2, 3, 4, 5 |

Scope is 6 subtasks — within the 1..=7 limit, no split needed.

## Handoff plan

Total subtasks `M = 6`. Per `.claude/skills/task/SKILL.md` Step 8 and `.claude/agents/design.md` § Rules → handoff-grouping, every group ≥ 1 requires `/context-reset`; non-terminal groups must be exactly 3 consecutive subtasks; the terminal group is sized within `1..=3`.

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent /task starts Group A with fresh context.
- **Group A:** subtasks 1–3 — declare `WidgetState`, collapse the fields with the new `state` member and updated defaults, then rewrite the `WidgetExt` accessor bodies that drive every other read/write site.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent /task resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — terminal group (3 subtasks; within the 1..=3 range). Rewrite the `widget_ext.rs` test-block reads, the `dispatch.rs` visibility checks, and run the workspace-wide gate (clippy / fmt / test / doc / `--no-default-features --features libm` / AC13 grep / `Cargo.lock` refresh).

## Risks

- **Compilation cascade if a call site is missed.** Mitigation: the field rename to `state` is a hard compile break — `cargo build` will surface every missed `.visible` / `.enabled` / `.pending_update` / `.hovered` / `.pressed` / `.focused` read on `WidgetBase` as `E0609 no field` and every missed write as the same error on the LHS. Subtask 6's full gate (clippy + test + doc + no-default-features build) catches anything `cargo build` alone misses (e.g. doc-test-only references). AC13 grep is the belt-and-braces final check.
- **`BitFlags::set(flag, bool)` API availability.** `enumflags2 = "0.7"` ships with `BitFlags::set(self, BitFlag, bool)`. If for some reason `set` is unavailable in the resolved minor version, fall back to the explicit branch: `if value { state.insert(F) } else { state.remove(F) }`. The fallback is byte-equivalent in semantics and idiomatically used in `quartzite-renderer/src/wrapped_handler.rs` (`self.pressed_buttons |= …` / `self.pressed_buttons &= !…`). Verify on first build that `BitFlags::set` resolves; switch to the fallback if not.
- **Doc-test rendering of the new struct.** `WidgetBase::new` and `Default::default` doc-tests currently read `w.visible` / `w.enabled`. Rewriting them to `w.state.contains(WidgetState::Visible)` adds a single `use quartzite_widgets::WidgetState;` line per doc-test (or `quartzite_widgets::widget_base::WidgetState`). Mitigation: re-export `WidgetState` from `quartzite-widgets/src/lib.rs` alongside the existing `WidgetBase` re-export — keeps the doc-test imports minimal and gives external callers a stable path. Add to `pub use widget_base::{...}` in `lib.rs` as part of Subtask 2.
- **`no_std` / `libm` regression (AC12).** `enumflags2` is `no_std`-compatible — already exercised by `quartzite-core` (a `no_std`-capable crate). `quartzite-widgets` depends on `std` (uses `std::sync::Arc`, `std::collections`), but AC12 builds `quartzite` (not `-widgets`) with `--no-default-features --features libm`; the dependency edge to `quartzite-widgets` is gated by the `widgets` feature in the parent crate (verify). Mitigation: Subtask 6 runs `cargo build -p quartzite --no-default-features --features libm` explicitly; failure mode is a feature-gating error the build surfaces directly.
- **Hidden binary-size cost of `BitFlags::set` / `contains` not inlining cross-crate.** `enumflags2`'s public methods are already `#[inline]` upstream and the call sites are all in trivial accessor bodies (`#[inline]` on the `WidgetExt` defaults). No measurable change expected. Mitigation: no action; flag only if clippy fires `large_enum_variant` or similar (unlikely on a `u8`-backed `BitFlags`).
- **Removal of `#[allow(clippy::struct_excessive_bools, …)]` annotation.** The annotation's `reason` cites the planned collapse, so removing it is correct. Mitigation: subtask 2 removes the annotation in the same edit that removes the six bool fields — keeps the lint posture coherent at every intermediate commit.

## Test Design

The collapse is mechanical; the existing test suite is the regression net. No new behavioural tests are added — every change is verified through (a) rewriting existing assertions to read through the new `state` field, (b) AC8 / AC9 / AC10 / AC11 / AC12 gates, and (c) AC13 grep.

For each non-trivial task:

### Subtask 1 — declare `WidgetState`

- **Location:** `quartzite-widgets/src/widget_base.rs` `#[cfg(test)] mod tests`.
- **Entry point:** `WidgetState` variants (declared at module scope) — no test code; the doc-test on the enum itself (one `# Examples` block per AGENTS.md § *Documentation*) doubles as a behavioural smoke test (`WidgetState::Enabled | WidgetState::Visible` constructs without panic, `contains` round-trips).
- **Scenarios:** doc-test demonstrates OR-combining two variants and `contains` retrieval — mirrors the existing `MouseButton` / `MouseButtons` doc-test shape at `quartzite-events/src/mouse.rs:14–23` exactly (use that as the implementation template).
- **Fixtures:** none.

### Subtask 2 — collapse fields and rewrite `WidgetBase::new` / `Default` doc-tests + internal tests

- **Location:** `quartzite-widgets/src/widget_base.rs` `#[cfg(test)] mod tests` (existing tests `new_widget_base_defaults`, `as_widget_self_ref`, `show_hide`, `update_sets_pending`).
- **Entry point:** `WidgetBase::new()` / `Default::default()` / `WidgetExt::show` / `WidgetExt::hide` / `WidgetExt::update`.
- **Scenarios after rewrite:**
  - `new_widget_base_defaults`: assert `!state.contains(Visible)`, `state.contains(Enabled)`, `!state.contains(PendingUpdate)`, `!state.contains(Hovered)`, `!state.contains(Pressed)`, `!state.contains(Focused)`. (Reads-only; no new behaviour.)
  - `as_widget_self_ref`: assert `!r.state.contains(WidgetState::Visible)`.
  - `show_hide`: `w.show()` → `state.contains(Visible)`; `w.hide()` → `!state.contains(Visible)`.
  - `update_sets_pending`: `!state.contains(PendingUpdate)` initially; after `update()` → `state.contains(PendingUpdate)`.
- **Fixtures:** none — `WidgetBase::new()` covers all.

### Subtask 3 — `WidgetExt` accessor bodies

- **Location:** `quartzite-widgets/src/widget_ext.rs` `#[cfg(test)] mod tests` (16 existing tests cover this surface end-to-end).
- **Entry point:** the public `WidgetExt` accessor methods.
- **Scenarios:** existing tests (`show_sets_visible`, `hide_clears_visible`, `set_visible_true_and_false`, `is_enabled_default_true`, `set_enabled_false`, `is_hovered_default_false`, `set_hovered_flips`, `is_pressed_default_false`, `set_pressed_flips`, `is_focused_default_false`, `set_focused_flips`, `update_sets_pending_update`, `on_mouse_press_default_sets_pressed`, `on_mouse_release_default_clears_pressed`, `on_focus_in_default_sets_focused`, `on_focus_out_default_clears_focused`) all exercise the accessors directly — they pass iff the new `state` plumbing is wired correctly.
- **Fixtures:** `fake_mouse_event(MouseEventKind)` helper already in the test module (unchanged).

### Subtask 4 — test-block reads through `state`

- **Location:** `quartzite-widgets/src/widget_ext.rs` `#[cfg(test)] mod tests` lines 601, 712, 720, 727, 735 (5 read sites; the 6th touch point in Subtask 3's count is the line-364 doc-test, which Subtask 3 already covers).
- **Scenarios:** read shape only — `w.widget_base().pending_update` → `w.widget_base().state.contains(WidgetState::PendingUpdate)` (and similarly for `pressed` / `focused`). Test behaviour unchanged. No new test functions.

### Subtask 5 — `dispatch.rs` visibility checks

- **Location:** `quartzite-style-dispatch/src/dispatch.rs` `#[cfg(test)] mod tests` (already has `dispatch_paint_invokes_draw_widget_once_per_visible_widget`, `hidden_root_produces_zero_paints`, `hidden_subtree_skipped_with_no_save_or_translate`, `scroll_area_with_content_paints_chrome_and_content`, …).
- **Entry point:** `visit()` / `dispatch_paint()` paths gated by the visibility check.
- **Scenarios:** all existing dispatch tests pass — they construct widgets via `WidgetBase::new()` and call `root.show()` / leave hidden, then verify `count_fill_rects` and event sequences. Routing the check through `is_visible()` is byte-equivalent at runtime.
- **Fixtures:** existing `StubResolver`, `MarkStyle`, `RecordingPainter` (unchanged).

### Subtask 6 — full workspace gate

- **Location:** workspace-level shell commands (no in-source test).
- **Scenarios:** AC8–AC13 verified explicitly. AC13 grep run programmatically; visual-inspect any matches against the false-positive list (`return_pressed` signal in `line_edit.rs`, `pressed_buttons` in `wrapped_handler.rs`, `MouseButton::*` references, etc.).
- **Fixtures:** none.

## Open questions

(none)
