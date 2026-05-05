# Progress: doc-convention — ACTIVE
_Updated: 2026-05-05_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-05-05-doc-convention
**base_commit:** 5ee77d67d7d48cd37143d2bc18f00efbb96b7d84
**Last build:** PARTIAL — `quartzite-macros` clippy/doc/build/fmt clean (lib + own sources); 3 new TDD tests failing as designed (locking the contract subtask 7 will satisfy); existing 142 unit tests + all 4 integration test files still pass; subtask 7 will green the new tests by emitting `# Parameters` / `# Examples` / accessor docs from the four `quote!` sites. (Transitive `cargo clippy --tests` errors from `quartzite-runtime/src/{application,connection_table,event_loop,factory,object_tree,timer}.rs` are subtask 8 work — not introduced by subtask 6.)

**Issue:** #80
**Spec:** ai-docs/plans/2026-05-05-doc-convention.spec.md
**Design:** ai-docs/plans/2026-05-05-doc-convention.design.md

## Next action

**Do this immediately:** Subtask 7 — update `quartzite-macros` codegen so the three failing TDD tests added in subtask 6 turn green. Required emissions per design § *Implementation breakdown* (row 7):

1. `emit_signal_wrappers` (`quartzite-macros/src/object/codegen.rs` ~lines 290–330): inject `# Parameters` (one bullet per `arg{i}: <ty>`) and a `no_run` `# Examples` block into the `quote!` doc comments. Test to satisfy: `object::codegen::tests::emit_wrapper_doc_contains_parameters_and_examples` (asserts `out.contains("# Parameters")` and `out.contains("# Examples")` against a `Signal<(i32, i32)>` fixture).
2. `emit_connect_auto_wrappers` (same file, ~lines 332–374): inject `# Parameters` (`receiver`, `f`) and a `no_run` `# Examples` block. Test to satisfy: `object::codegen::tests::connect_auto_wrapper_doc_contains_parameters_and_examples` (scoped to the second of three `impl Foo` blocks because `connect_queued` already emits `# Examples`).
3. `emit_connect_queued_wrappers` (same file, ~lines 376–423): already has `# Examples`; add `# Parameters` (`receiver`, `f`). No new test was added for this in subtask 6 (the existing `connect_queued_wrapper_generated_for_signal` test already covers the `no_run` doc shape), but the design row 7 lists this as part of the codegen change — implement it for symmetry.
4. `emit_root_trait_and_impl` (`quartzite-macros/src/extend/codegen.rs` ~lines 47–78): inject doc comments on the two trait-definition methods `#acc` and `#acc_mut`. Suggested summary lines: `Returns a shared reference to this object.` / `Returns a mutable reference to this object.` Test to satisfy: `extend::codegen::tests::root_trait_methods_carry_docs` (asserts `out.contains("# [doc")` for the `#[root] struct Widget { x: i32 }` case, which has zero other doc-emitting items).

Per design AC4 the trait-impl methods (`impl AsWidget for Widget`) remain exempt — only the trait-definition methods need docs.

## Subtasks

- [x] 1. Write `ai-docs/doc-convention.md`, AGENTS.md pointer, `clippy.toml` (no clippy gate yet)
- [x] 2. Add the five lint attributes to every crate's `lib.rs` (`cargo build` still green; clippy expected red)
- [x] 3. Audit & fix `quartzite-core` (clippy/doc/test gates clean for the crate)
- [x] **HANDOFF here per design** — `/context-reset` after subtask 3
- [x] 4. Audit & fix `quartzite-geometry`
- [x] 5. Audit & fix `quartzite-events` (AC13 — `MouseEvent::new` doctest)
- [x] 6. Add codegen tests for `quartzite-macros` (string-contains assertions on emitted docs)
- [ ] 7. Update `quartzite-macros` codegen — emit conforming docs at four `quote!` sites ← CURRENT
- [ ] **HANDOFF here per design** (recommended) — `/context-reset` after subtask 7
- [ ] 8. Audit & fix `quartzite-runtime` (heaviest `# Errors`/`# Panics` work)
- [ ] 9. Audit & fix `quartzite` facade (`src/lib.rs`)
- [ ] 10. Update `code-review` skill + `review-findings` + `self-review` agents (Propagation Rule)
- [ ] 11. Final workspace verification — `cargo fmt --check`, full clippy/doc/test/no_std

## Key discoveries (don't re-investigate)

- Per design: no `# Parameters` / `# Errors` / `# Panics` sections currently exist anywhere in the workspace; summary tense is mostly correct (~10–15 imperative remnants per crate); `#[doc(hidden)]` macro internals are exempt.
- Trait-impl methods (inside `impl Trait for Type {}` blocks, including derives like `From`, `Display`, `Drop`) are exempt — only inherent and trait-definition methods carry the convention.
- `clippy::doc_markdown` enabled at `warn`-level becomes a hard error via CI's `cargo clippy -- -D warnings`. `clippy.toml` `doc-valid-idents` is seeded (~60 entries) and grows during the audit as new false positives surface.
- Proc-macro `quote!` sites that emit user-facing `pub` items (per design § *Proc-macro emitted-doc audit*): `emit_signal_wrappers`, `emit_connect_auto_wrappers`, `emit_connect_queued_wrappers`, `emit_root_trait_and_impl`. These four are the entire scope of subtask 7.
- Lint insertion order in each `lib.rs`: keep `#![cfg_attr(...)]`/`#![no_std]`/`#![cfg_attr(docsrs,...)]` first, then the new lints, then `#![deny(missing_docs)]`, then `#![doc=...]`.
- Audit order (subtasks 3–9) is bottom-up by dependency: `core → geometry → events → macros (tests then codegen) → runtime → facade`.

### Subtask 3 (`quartzite-core`) notes

- `clippy.toml` did **not** need any new entries during this subtask — every `doc_markdown` site was already covered by the seed list or by clippy's defaults.
- Pre-existing `clippy::approx_constant` at `value.rs:429` fixed by replacing the `3.14f64` `rstest` case with `1.5f64` (a neutral value, with an inline comment noting the reason). Avoided `core::f64::consts::PI` so the round-trip test stays a generic non-trivial float, not a special constant.
- Enum *variant* docs were intentionally **not** rewritten for tense (the convention text targets top-level public items: `pub fn / struct / enum / trait / union / macro_rules`). Variant docs already pass `missing_docs` with their existing one-liners.
- All `pub struct` / `pub enum` / `pub trait` items now carry their own `# Examples` block in addition to the constructors' examples (extra coverage for the convention's "every public item" rule).
- Doctest count rose from 81 → 94 (13 new compiling doctests across the crate); all pass.
- No conservative `# Parameters` descriptions were left for orchestrator review — every `# Parameters` bullet is grounded in code I read directly.

### Subtask 5 (`quartzite-events`) notes

- `clippy.toml` did **not** need any new entries — every `doc_markdown` site was already covered by the seed list (the seed already contained `MouseButton`, `MouseButtons`, `MouseEvent`, `KeyEvent`, `KeyModifier`, `KeyModifiers`, `EventFilter`, `ResizeEvent`, `TimerEvent`, etc. from earlier subtasks).
- Tense audit: every existing summary line (enums `KeyEventKind` / `MouseEventKind` / `EventType` / `MouseButton` / `KeyModifier` / `Key`, structs `KeyEvent` / `MouseEvent` / `TimerEvent` / `ResizeEvent` / `CloseEvent`, traits `Event` / `EventFilter`) was already in third-person present indicative ("Returns", "Creates", "Combine", "Constructed by"). No imperative remnants found.
- `# Parameters` added to every `*::new` constructor:
  - `MouseEvent::new` — `position`, `global_position`, `event_button`, `buttons_state`, `modifiers`, `kind` (AC13 flagship).
  - `KeyEvent::new` — `key`, `text`, `modifiers`, `is_repeat`, `kind`.
  - `TimerEvent::new` — `timer_id`.
  - `ResizeEvent::new` — `old_size`, `new_size`.
  - `CloseEvent::new` — receiver-only / no args; left with the existing summary + `# Examples`.
- Trait *definition* methods are NOT exempt per AC4: added `# Examples` to `Event::event_type` (receiver-only, so no `# Parameters`) and added both `# Parameters` (`obj`, `event`) and `# Examples` to `EventFilter::event_filter`.
- All accessors (`position`, `global_position`, `event_button`, `buttons_state`, `modifiers`, `kind`, `key`, `text`, `is_repeat`, `timer_id`, `old_size`, `new_size`, `accepted`, `accept`) are receiver-only — they keep their existing one-line summary + `# Examples`.
- AC13 doctest design: the new second example block under `MouseEvent::new` constructs an event where `event_button = MouseButton::Right` while `buttons_state = MouseButton::Left | MouseButton::Right`, simulating "right was just pressed while left was already held". It then asserts `event.event_button()` and `event.buttons_state()` separately, plus a negative assertion that `event_button` does NOT contain `Left` — making conflation of the two fields impossible to miss for a reader.
- Symmetric `KeyEvent::new` doctest: a second example block exercises `key`, `text`, `modifiers` (Shift), `is_repeat = true`, and `kind` together — asserts each of the four observable fields independently.
- No `# Errors`, `# Panics`, `# Safety`, `# Returns`, `# Type parameters`, or `# Lifetimes` sections needed: no `Result` returns, no `unsafe`, no panics, no non-obvious generics.
- Module-level `//!` doc in `lib.rs` already conformed; no edits needed.
- Doctest count: 33 → 40 (+7 — the AC13 second example, the `KeyEvent::new` second example, and `# Examples` blocks added on the `Event` and `EventFilter` trait definitions; mouse and key constructors each have two doctests now). All pass.
- No conservative `# Parameters` descriptions were left for orchestrator review — every bullet is grounded in code I read directly. The trickiest call was distinguishing `event_button` from `buttons_state`; the prose mirrors the AC13 design intent ("the button whose state changed" vs "every button currently held") and is reinforced by the assertion structure of the new doctest.

### Subtask 6 (`quartzite-macros` audit + TDD tests) notes

- **Half 1 — own public API audit.** Two British → American spelling fixes in `quartzite-macros/src/lib.rs`:
  - `derive_object` `#[prop]` table: `"exclude from serialisation"` → `"exclude from serialization"`.
  - `object_impl` summary line: `"finalises the Object implementation"` → `"finalizes the Object implementation"`.
  - One `clippy::doc_markdown` fix in `quartzite-macros/src/util.rs:29`: `snake_case` → `` `snake_case` `` (per the audit worklist baseline — "1 × doc_markdown or similar"). The remaining four exported proc-macros (`derive_extend`, `derive_object`, `object_part`, `object_impl`, `derive_meta_enum`) all already carry summary lines in third-person present indicative ("Derive macro that generates …", "Attribute macro applied to …"), each item has an `# Examples` block (using `no_run` or `ignore` per the proc-macro `# Examples` rule), and each item with attributes documents them under an `## Attributes` subsection. The `# Parameters` rule is N/A for `proc_macro_*` exports because the function signatures are `(TokenStream) -> TokenStream` driven by the proc-macro machinery — the design's "judgment call" was to keep the existing `## Attributes` subsections as the equivalent of `# Parameters`.
- **Half 2 — TDD tests.** Three new `#[cfg(test)] mod tests` entries; all three intentionally **fail** until subtask 7 lands the codegen change:
  - `quartzite-macros/src/object/codegen.rs::tests::emit_wrapper_doc_contains_parameters_and_examples` — fixture `Signal<(i32, i32)>`. Failure message: `missing # Parameters in emit_<sig> wrapper doc: …` (full token stream pasted into the assertion message; current emit doc contains only summary + `Checks [...] before firing.` + `Returns immediately when blocked.`).
  - `quartzite-macros/src/object/codegen.rs::tests::connect_auto_wrapper_doc_contains_parameters_and_examples` — fixture `Signal<(i32,)>`. Scoped to the second of three `impl Foo` blocks (the connect_queued block already has `# Examples`, so the assertion would otherwise pass on the wrong block). Failure message: `missing # Parameters in connect_<sig>_auto wrapper doc: impl Foo { # [doc = r" Connects this signal to a slot with `Auto` delivery."] # [doc = r""] # [doc = r" Same-thread emits call `f` directly; cross-thread emits post to the dispatcher."] # [doc = r" The slot is silently skipped once `receiver` has been dropped."] …`.
  - `quartzite-macros/src/extend/codegen.rs::tests::root_trait_methods_carry_docs` — fixture `#[root] struct Widget { x: i32 }`. Failure message: `missing doc attribute on root-trait accessor methods: pub trait AsWidget { fn widget (& self) -> & Widget ; fn widget_mut (& mut self) -> & mut Widget ; } impl AsWidget for Widget { # [inline] fn widget (& self) -> & Widget { self } # [inline] fn widget_mut (& mut self) -> & mut Widget { self } }` — the test asserts `out.contains("# [doc")` because the `quote!` round-trip lowers `///` to `# [doc = "..."]` in the rendered token stream, and this fixture (no signals, no base, no mixin) emits zero doc-bearing items today.
- **Verify gates.** All four non-test gates green for `quartzite-macros` itself: `cargo build -p quartzite-macros` PASS; `cargo clippy -p quartzite-macros --lib -- -D warnings` PASS (no warnings in `quartzite-macros/src/`); `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc -p quartzite-macros --no-deps` PASS; `cargo fmt -- --check` PASS. `cargo clippy -p quartzite-macros --tests` shows 12 errors but **all** are from `quartzite-runtime/src/{application,connection_table,event_loop,factory,object_tree,timer}.rs` (transitive dev-dep through `quartzite` facade) — that's subtask 8 work, not introduced by this subtask.
- **Test gate.** `cargo test -p quartzite-macros --lib`: 142 passed, 3 failed (the new TDD tests). `cargo test -p quartzite-macros --tests` (integration tests `extend.rs`, `meta_enum.rs`, `object.rs`, `object_impl.rs`): all 4 binaries compile and pass — no regression.
- **Decision flagged for review:** the `extend/codegen.rs` test asserts `out.contains("# [doc")` rather than a more specific scoped assertion. Justification: for the `#[root] struct Widget { x: i32 }` fixture (no signals, no base, no mixin), `emit_root_trait_and_impl` is the **only** doc-emitting site, so any `# [doc` substring uniquely belongs to the trait-def accessor methods. If subtask 7 also emits docs on the impl-block methods (`impl AsWidget for Widget` — exempt per AC4 but harmless to document), the test still passes. Reviewer can tighten to a positional assertion if needed.

### Subtask 4 (`quartzite-geometry`) notes

- `clippy.toml` did **not** need any new entries during this subtask — `PointF` / `RectF` / `SizeF` were already in the seed list, and existing prose carried no other un-backticked CamelCase identifiers.
- Tense audit: every existing summary line was already in third-person present indicative ("Creates", "Returns", "Applies", "Converts"). No tense fixes were needed.
- `# Parameters` added to every public fn with ≥1 non-receiver argument across the four type modules:
  - `Point::new`, `PointF::new` — `x`, `y`.
  - `Size::new`, `SizeF::new` — `width`, `height` (with non-negative contract restated).
  - `Rect::new`, `RectF::new` — `origin`, `size`.
  - `Rect::contains`, `RectF::contains` — `point`.
  - `Rect::intersects`, `RectF::intersects` — `other`.
  - `Rect::united`, `RectF::united` — `other`.
  - `Rect::translated`, `RectF::translated` — `offset`.
  - `Rect::adjusted`, `RectF::adjusted` — `dx1`, `dy1`, `dx2`, `dy2`.
  - `Margins::new` — `left`, `top`, `right`, `bottom` (with positive=shrink convention restated).
  - `Margins::apply` — `rect`.
- All accessors (`x()`, `y()`, `width()`, `height()`, `origin()`, `size()`, `left()`, `top()`, `right()`, `bottom()`, `is_empty()`) are receiver-only, so they keep their existing one-line summary + `# Examples` per design.
- Free-standing `From<...>` impl docs (e.g. `impl From<PointF> for Point`) carry doc-on-impl block-level comments only — these are trait-impl methods (AC4 exemption) and were left as-is.
- No `# Errors`, `# Panics`, `# Safety`, `# Returns`, `# Type parameters`, or `# Lifetimes` sections needed: no `Result` returns, no `unsafe`, no panicking arithmetic on caller input (overflow on `i32` arithmetic in `adjusted`/`united` panics only in debug mode, considered an internal invariant rather than a caller-facing precondition — matches stdlib convention of not documenting every arithmetic-overflow possibility).
- No conservative `# Parameters` descriptions were left for orchestrator review — every bullet is grounded in code read directly.
- Module-level `//!` doc in `lib.rs` already conformed; no edits needed.
- Doctest count: 59 (unchanged — every existing accessor / constructor / combinator already had its own doctest; adding `# Parameters` doesn't add or remove tests).

## AC Status

| AC | Status |
|----|--------|
| AC1 | PASS (subtask 1 — `ai-docs/doc-convention.md` written) |
| AC2 | PASS (subtask 1 — AGENTS.md Code Style updated) |
| AC3 | PARTIAL (subtasks 3+4+5+6 — `quartzite-core`, `quartzite-geometry`, `quartzite-events`, `quartzite-macros` own API audited; subtasks 8/9 cover `quartzite-runtime` and the `quartzite` facade) |
| AC4 | NOT_TESTED |
| AC5 | PASS (subtask 2 — five lints in every `lib.rs`) |
| AC6 | PASS (subtask 1 — `clippy.toml` seeded; no new entries needed during subtask 3) |
| AC7 | NOT_TESTED |
| AC8 | NOT_TESTED |
| AC9 | NOT_TESTED |
| AC10 | PARTIAL (subtasks 4+5 — `quartzite-geometry --no-default-features` and `quartzite-events --no-default-features` PASS; `quartzite` facade still NOT_TESTED — subtasks 9/11) |
| AC11 | NOT_TESTED (subtask 6 added the locking string-contains tests in `quartzite-macros/src/object/codegen.rs` + `quartzite-macros/src/extend/codegen.rs` — currently failing as designed; subtask 7 will green them by updating the four `quote!` sites) |
| AC12 | NOT_TESTED |
| AC13 | PASS (subtask 5 — `MouseEvent::new` carries `# Parameters` for `event_button` and `buttons_state` plus a doctest constructing an event where `event_button = Right` while `buttons_state = Left | Right`, asserting `event_button()` and `buttons_state()` independently — readers cannot conflate the two fields) |

## Files touched

- `ai-docs/doc-convention.md` (new) — canonical doc convention reference
- `AGENTS.md` — Documentation Conventions pointer paragraph in *Code Style*
- `clippy.toml` (new) — seed `doc-valid-idents` allowlist (~60 entries)
- `quartzite-core/src/lib.rs` — added 5 lint attrs
- `quartzite-events/src/lib.rs` — added 5 lint attrs
- `quartzite-geometry/src/lib.rs` — added 5 lint attrs
- `quartzite-macros/src/lib.rs` — added 5 lint attrs
- `quartzite-runtime/src/lib.rs` — added 5 lint attrs
- `src/lib.rs` — added 5 lint attrs
- `quartzite-core/src/meta.rs` — tense fixes; `# Parameters` on every `*::new`; `# Examples` added to `PropertyMeta`/`ParamMeta`/`SignalMeta`/`MethodMeta`/`EnumEntry`/`EnumMeta`/`MetaObject`; `# Parameters`/`# Examples` on six `noop_lookup_*` helpers
- `quartzite-core/src/object_base.rs` — `# Parameters` on `named` and `set_name_raw`
- `quartzite-core/src/receiver_guard.rs` — tense fix on `new_pair`
- `quartzite-core/src/signal.rs` — `# Errors` on `set_queued_dispatcher`; tense fixes; `# Parameters` on every multi-arg `Signal::*`/`QueuedDispatcher::post`; `# Panics` on `connect_typed`; `# Examples` on `Signal`, `ConnectionType`, `DispatcherAlreadySet`, `QueuedDispatcher`
- `quartzite-core/src/traits.rs` — `# Parameters` + `# Examples` (no_run) on every method declared in `Object`
- `quartzite-core/src/value.rs` — `# Errors` on `FromValue::from_value`; `# Parameters` on it; tense fixes on `IntoValue`/`CustomValue`; `# Examples` on `WeakObjectRef`, `Value`, `TypeError`; doctest fix `3.14f64` → `1.5f64` (also rewrote `IntoValue` summary doctest from `3.14` to `1.5`); `# Examples` on each `CustomValue` method
- `quartzite-geometry/src/point.rs` — `# Parameters` on `Point::new`, `PointF::new`
- `quartzite-geometry/src/size.rs` — `# Parameters` on `Size::new`, `SizeF::new`
- `quartzite-geometry/src/rect.rs` — `# Parameters` on `Rect::new`, `Rect::contains`, `Rect::intersects`, `Rect::united`, `Rect::translated`, `Rect::adjusted`, and the matching `RectF::*` methods
- `quartzite-geometry/src/margins.rs` — `# Parameters` on `Margins::new`, `Margins::apply`
- `quartzite-events/src/event.rs` — `# Examples` added to trait-definition method `Event::event_type`; `# Parameters` (`obj`, `event`) + `# Examples` added to trait-definition method `EventFilter::event_filter`
- `quartzite-events/src/keyboard.rs` — `# Parameters` on `KeyEvent::new` plus a second `# Examples` doctest exercising `key`, `text`, `modifiers`, `is_repeat`, `kind` together
- `quartzite-events/src/mouse.rs` — `# Parameters` on `MouseEvent::new` (AC13 flagship) plus a second `# Examples` doctest where `event_button = Right` while `buttons_state = Left | Right`, asserting both accessors independently
- `quartzite-events/src/timer.rs` — `# Parameters` on `TimerEvent::new`
- `quartzite-events/src/window.rs` — `# Parameters` on `ResizeEvent::new`
- `quartzite-macros/src/lib.rs` — British → American spelling fixes (`finalises` → `finalizes` on `object_impl`; `serialisation` → `serialization` in `derive_object` `#[prop(stored = false)]` table)
- `quartzite-macros/src/util.rs` — `clippy::doc_markdown` fix on `accessor_name` doc (`snake_case` → `` `snake_case` ``)
- `quartzite-macros/src/object/codegen.rs` — added two TDD tests inside the `#[cfg(test)] mod tests` block: `emit_wrapper_doc_contains_parameters_and_examples`, `connect_auto_wrapper_doc_contains_parameters_and_examples` (both currently fail; subtask 7 lands the codegen change)
- `quartzite-macros/src/extend/codegen.rs` — added one TDD test inside the `#[cfg(test)] mod tests` block: `root_trait_methods_carry_docs` (currently fails; subtask 7 lands the codegen change)

## Audit worklist (from subtask 2 baseline clippy run)

`cargo clippy --workspace --all-targets` output: 28 warnings + 2 errors.

- `quartzite-core (lib)`: 2 × `missing_errors_doc`.
- `quartzite-runtime (lib)`: 12 warnings — mix of `missing_errors_doc`, `missing_panics_doc`, `doc_markdown` (2 backtick fixes).
- `quartzite-macros (lib)`: 1 × `doc_markdown` or similar.
- `quartzite-runtime (lib test)`: same 12 (duplicates).
- `quartzite-runtime (test "object_tree")`: 1 × `methods called 'new' usually return Self` — **investigate scope**.
- `quartzite-core (lib test) ERROR`: `clippy::approx_constant` at `quartzite-core/src/value.rs:429` — pre-existing, fix in subtask 3.

Run `cargo clippy --workspace --all-targets 2>&1` again at the start of each crate-audit subtask to refresh the worklist after each fix.
