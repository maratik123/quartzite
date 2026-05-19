# Collapse `WidgetBase` bool fields into `BitFlags<WidgetState>`

**Source:** issue #480
**Date:** 2026-05-19
**Tracked in:** #480

## Scope

1. Define a new `#[bitflags] #[repr(u8)] pub enum WidgetState` in `quartzite-widgets` (placement up to design — most natural is `quartzite-widgets/src/widget_base.rs`) with 6 variants corresponding to the existing bools: `Visible`, `Enabled`, `PendingUpdate`, `Hovered`, `Pressed`, `Focused`. Each variant is documented with `///` doc comments mirroring the field-level docs that exist today.
2. Replace the 6 `bool` fields in `WidgetBase` (`visible`, `enabled`, `pending_update`, `hovered`, `pressed`, `focused`) with a single `pub state: BitFlags<WidgetState>` field. Public-field shape matches the rest of `WidgetBase`'s public fields (`pub geometry`, `pub font`, etc.).
3. Update `WidgetBase::new()` and `Default for WidgetBase` so the initial value is `WidgetState::Enabled.into()` (i.e. `Enabled` bit set, all others cleared) — preserving today's `enabled: true` / everything-else-false defaults.
4. Update every access site to read/write through the `state` field:
   - `quartzite-widgets/src/widget_ext.rs` — 14 read/write call sites across `is_visible`/`set_visible`/`show`/`hide`/`is_enabled`/`set_enabled`/`is_hovered`/`set_hovered`/`is_pressed`/`set_pressed`/`is_focused`/`set_focused`/`update` plus the doc comment at `update`.
   - `quartzite-widgets/src/widget_base.rs` — internal `#[cfg(test)] mod tests` assertions (`!w.visible`, `w.enabled`, `!w.pending_update`, etc.) plus the two doc-test snippets on `WidgetBase::new` and `Default::default`.
   - `quartzite-widgets/src/widget_ext.rs` `#[cfg(test)] mod tests` — assertions that read `w.widget_base().pending_update` / `.pressed` / `.focused` directly.
   - `quartzite-style-dispatch/src/dispatch.rs` — 2 visibility checks (`if !widget.widget_base().visible` and `if !child.widget_base().visible`).
5. Add `enumflags2 = "0.7"` to `quartzite-widgets/Cargo.toml` `[dependencies]` (already transitively present via `quartzite-core` and `quartzite-events`, but `widget_base.rs` will name `BitFlags` and `WidgetState` directly).
6. Remove the per-item `#[allow(clippy::struct_excessive_bools, reason = "planned collapse into BitFlags<WidgetState> tracked in #480")]` annotation from `WidgetBase`. With only one `bool` field remaining (there are none after the collapse — `WidgetBase` now has zero `bool` fields), `struct_excessive_bools` does not fire and the `#[allow]` is dead weight.

## Out of scope

- Renaming or restructuring the `WidgetExt` accessor surface (`is_visible`, `set_visible`, `is_hovered`, …). Bodies change to delegate to `BitFlags::{contains, insert, remove, set}`; public signatures stay byte-identical.
- Adding new `WidgetState` variants beyond the 6 mapped 1:1 from existing bools (e.g. no new `Checked` variant for `Button` — that lives on `Button` itself and is out of scope).
- Refactoring `quartzite-style/src/default_style.rs`, which already uses only `is_*()` accessors (no direct field access).
- Lifting `enumflags2` to a workspace dependency or otherwise touching the dependency graph beyond `quartzite-widgets/Cargo.toml`.
- Touching `quartzite-widgets/src/widgets/line_edit.rs` — the issue body names it as a presumed access site, but a fresh grep of the file shows it contains no direct reads/writes of the 6 collapsed fields (its `return_pressed: Signal<()>` member is unrelated). The compiler will surface any missed site during build; if `line_edit.rs` turns out to need edits after all, they fold into Subtask 4's per-call-site update naturally.

## Deferred

- (none)

## Key decisions

| Question | Decision |
|---|---|
| `enumflags2::BitFlags<WidgetState>` vs hand-rolled `bitflags!` macro | Use `enumflags2`. The issue body explicitly names `BitFlags<WidgetState>`; the dep is already in-tree (`quartzite-core/Cargo.toml`, `quartzite-events/Cargo.toml`); three existing workspace types (`MouseButtons`, `KeyModifiers`, `PropertyFlags`) use the same `#[bitflags] #[repr(u8)] enum` + `pub type Foo = BitFlags<FooVariant>;` pattern. Stays consistent. |
| Single `pub state: BitFlags<WidgetState>` field vs private + `state()` getter | `pub state`. Matches the existing convention of `pub geometry`, `pub font`, `pub palette`, `pub layout`, `pub min_size`, `pub max_size` on `WidgetBase`. Per AGENTS.md API Stability, the 6 pub bools → 1 pub `BitFlags` rename is a clean break. |
| Per-bool shim methods (`pub fn visible(&self) -> bool` etc. on `WidgetBase` direct) | None. The existing `WidgetExt::is_visible` etc. blanket-trait accessors cover ergonomic reads; per AGENTS.md API Stability the old field shape is gone. |
| Default value | `WidgetState::Enabled.into()` — `Enabled` bit set, all other bits clear. Preserves today's `enabled: true, visible: false, pending_update: false, hovered: false, pressed: false, focused: false` semantics. |
| Bit values | Implementation detail — the standard `0b0000_0001` … `0b0010_0000` shape used by `MouseButton` is the natural fit; design agent picks. |

## Technical constraints

- `cargo clippy --workspace --all-targets -- -D warnings` must pass (AC from issue).
- `cargo test --workspace` must pass (AC from issue).
- `cargo fmt -- --check` must pass.
- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` must pass — `WidgetState` and each variant are public items and need at least one-line `///` doc comments (workspace `missing_docs = "deny"`).
- `cargo build -p quartzite --no-default-features --features libm` must pass — `enumflags2` supports `no_std`; `quartzite-widgets`'s default-features posture is preserved.
- Public-field accessors (`is_visible`, `set_visible`, …) keep byte-identical signatures so external (non-workspace) callers using `WidgetExt` are not source-impacted (the workspace itself has no such callers, so this is a structural property rather than a separate AC).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `pub enum WidgetState` exists in `quartzite-widgets` with exactly 6 variants: `Visible`, `Enabled`, `PendingUpdate`, `Hovered`, `Pressed`, `Focused`. Verified by `grep -nE '^\s*(Visible\|Enabled\|PendingUpdate\|Hovered\|Pressed\|Focused)\b' quartzite-widgets/src/widget_base.rs` (or wherever placed) returning exactly 6 matches. |
| AC2 | `WidgetBase` declares `pub state: BitFlags<WidgetState>` and no longer declares the 6 bool fields. Verified by `grep -nE 'pub\s+(visible\|enabled\|pending_update\|hovered\|pressed\|focused)\s*:\s*bool' quartzite-widgets/src/widget_base.rs` returning zero matches. |
| AC3 | `#[allow(clippy::struct_excessive_bools, …)]` annotation on `WidgetBase` is removed. Verified by `grep -n 'struct_excessive_bools' quartzite-widgets/src/widget_base.rs` returning zero matches. |
| AC4 | `WidgetBase::new()` returns an instance with `state.contains(WidgetState::Enabled) == true` and `state.contains(WidgetState::Visible) == false` (and similarly false for `PendingUpdate`, `Hovered`, `Pressed`, `Focused`). Existing tests `new_widget_base_defaults`, `show_hide`, `update_sets_pending` continue to pass after their assertions are rewritten to use `state.contains(...)`. |
| AC5 | `WidgetExt` accessor surface (`is_visible`/`set_visible`/`show`/`hide`/`is_enabled`/`set_enabled`/`is_hovered`/`set_hovered`/`is_pressed`/`set_pressed`/`is_focused`/`set_focused`/`update`) preserves exact signatures and observable semantics. Verified by the existing `#[cfg(test)] mod tests` in `widget_ext.rs` passing without behavioural test edits (test bodies that read `w.widget_base().pending_update` / `.pressed` / `.focused` are updated to `state.contains(...)`, which is a read-shape change only, not a semantics change). |
| AC6 | `quartzite-style-dispatch/src/dispatch.rs` visibility checks updated from `widget.widget_base().visible` / `child.widget_base().visible` to the `BitFlags`-shape equivalent (`widget.widget_base().state.contains(WidgetState::Visible)` or call the `is_visible()` accessor). |
| AC7 | `enumflags2` declared in `quartzite-widgets/Cargo.toml` `[dependencies]` at `"0.7"` (matching the workspace's existing usage shape in `quartzite-core` / `quartzite-events`). |
| AC8 | `cargo clippy --workspace --all-targets -- -D warnings` exits 0. |
| AC9 | `cargo test --workspace` exits 0. |
| AC10 | `cargo fmt -- --check` exits 0. |
| AC11 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` exits 0. |
| AC12 | `cargo build -p quartzite --no-default-features --features libm` exits 0. |
| AC13 | `grep -rnE '\.(visible\|enabled\|pending_update\|hovered\|pressed\|focused)\b' quartzite-widgets/src quartzite-style-dispatch/src quartzite-style/src` returns zero hits **on the 6 collapsed field names accessed as fields of `WidgetBase`** (i.e. the post-collapse codebase has no surviving `widget_base().visible` / `.enabled` / `.pending_update` / `.hovered` / `.pressed` / `.focused` reads or writes — all routed through `state`). False positives on unrelated identifiers (`return_pressed` Signal in `line_edit.rs`, `MouseButton::Left.pressed()` if any, etc.) are filtered with `--type rust -g '!target/**'` and visual inspection. |
| AC14 | Closes #480. |

## Open questions

(none)

```yaml
---
status: ready
round: 1
---
```
