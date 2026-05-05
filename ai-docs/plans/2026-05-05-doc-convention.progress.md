# Progress: doc-convention — ACTIVE
_Updated: 2026-05-05_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-05-05-doc-convention
**base_commit:** 5ee77d67d7d48cd37143d2bc18f00efbb96b7d84
**Last build:** PASS — review-group instruction files updated together (Propagation Rule). `.claude/skills/code-review/SKILL.md` gained a *Doc convention conformance* item in the Step 4 verify checklist; `.claude/agents/review-findings.md` gained a new §6 *Documentation conformance* checklist; `.claude/agents/self-review.md` gained the parallel block under §6 Documentation. All three reference `ai-docs/doc-convention.md` via relative links (verified with `realpath`). Grep cross-check returns exactly the four expected files (the three above + `AGENTS.md`). `cargo build --workspace` PASS (no source touched, sanity confirmed).

**Issue:** #80
**Spec:** ai-docs/plans/2026-05-05-doc-convention.spec.md
**Design:** ai-docs/plans/2026-05-05-doc-convention.design.md

## Next action

**Do this immediately:** Subtask 11 — final workspace verification. Run all gates green from a clean state:

- `cargo fmt -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`
- `cargo test --workspace` (all unit + integration + doctests)
- `cargo build -p quartzite --no-default-features` (AC10 final re-confirmation)

After all five gates green: PR is ready (subtask 11 closes the task).

## Subtasks

- [x] 1. Write `ai-docs/doc-convention.md`, AGENTS.md pointer, `clippy.toml` (no clippy gate yet)
- [x] 2. Add the five lint attributes to every crate's `lib.rs` (`cargo build` still green; clippy expected red)
- [x] 3. Audit & fix `quartzite-core` (clippy/doc/test gates clean for the crate)
- [x] **HANDOFF here per design** — `/context-reset` after subtask 3
- [x] 4. Audit & fix `quartzite-geometry`
- [x] 5. Audit & fix `quartzite-events` (AC13 — `MouseEvent::new` doctest)
- [x] 6. Add codegen tests for `quartzite-macros` (string-contains assertions on emitted docs)
- [x] 7. Update `quartzite-macros` codegen — emit conforming docs at four `quote!` sites
- [x] **HANDOFF here per design** (recommended) — `/context-reset` after subtask 7
- [x] 8. Audit & fix `quartzite-runtime` (heaviest `# Errors`/`# Panics` work)
- [x] 9. Audit & fix `quartzite` facade (`src/lib.rs`)
- [x] 10. Update `code-review` skill + `review-findings` + `self-review` agents (Propagation Rule)
- [ ] 11. Final workspace verification — `cargo fmt --check`, full clippy/doc/test/no_std ← CURRENT

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

### Subtask 7 (`quartzite-macros` codegen update) notes

- **Four `quote!` sites updated.** All emitted user-facing items now carry convention-conforming docs (Summary → prose → `# Parameters` → `# Examples`); trait-impl methods inside the matching `impl Block` remain exempt per AC4.
- **`emit_signal_wrappers` (`quartzite-macros/src/object/codegen.rs`).** Emits `pub fn emit_<sig>(&mut self, arg0..argN)`. Doc shape now: 4-line summary/prose (`Emits this signal …`) → conditional `# Parameters` block (omitted entirely for zero-arg signals — keeps the existing `emit_wrappers_zero_arg_signal` invariant of "no `arg0` token" intact) → `# Examples` `no_run` block whose hidden setup uses placeholder type `Emitter` (matches the existing `connect_<sig>_queued` pattern; rustdoc only collects emitted-doc doctests when a downstream library crate uses these macros, none currently do, so the placeholder is dormant). Each `# Parameters` bullet reads `` - `arg{i}`: the {i}-th positional argument forwarded to slots. ``
- **`emit_connect_auto_wrappers` (same file).** Doc shape: 4-line summary/prose (`Connects this signal to a slot with `Auto` delivery.` …) → `# Parameters` (`receiver`, `f` — same prose conventions used by `emit_connect_queued_wrappers` for symmetry) → `# Examples` `no_run` block (placeholder `Emitter` type). Section ordering matches the convention.
- **`emit_connect_queued_wrappers` (same file).** The existing `# Examples` block was preserved verbatim (still uses placeholder `Receiver` type — left unchanged so the existing `connect_queued_wrapper_generated_for_signal` test's `out.contains("no_run")` assertion keeps passing); `# Parameters` (`receiver`, `f`) inserted between the prose and the `# Examples` block per the convention's strict order.
- **`emit_root_trait_and_impl` (`quartzite-macros/src/extend/codegen.rs`).** The two trait-definition methods `#acc(&self)` and `#acc_mut(&mut self)` now each carry a single-line `#[doc = " Returns a {shared,mutable} reference to this object."]` attribute. Receiver-only signatures, so no `# Parameters` block is required by the convention. The matching `impl As<Self> for <Self>` impl methods stay undocumented (AC4 exemption).
- **All emitted docs use `#[doc = "…"]` attribute syntax** rather than `///` so the macro can interpolate dynamic content (e.g. per-signal `# Examples` body referencing `obj.{fn_name_str}(...)`). Per the convention, both syntaxes are equivalent for rustdoc.
- **`#[inline]` attributes preserved on every emitted fn**, immediately before the `pub fn` keyword (after the doc-attribute block). No emitted function signatures changed.
- **Test results.** `cargo test -p quartzite-macros --lib`: 145 passed (was 142 + 3 failing TDD; now all green). All 4 integration test binaries (`extend`, `meta_enum`, `object`, `object_impl`) still pass — no regression. Workspace `cargo test --workspace` PASS (every doctest compiles, every unit test passes, every integration test passes).
- **Verify gates.** `cargo build --workspace` PASS; `cargo clippy -p quartzite-macros --lib -- -D warnings` PASS; `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc -p quartzite-macros --no-deps` PASS; `cargo fmt -- --check` PASS; `cargo test --workspace --doc` PASS. (`cargo clippy --workspace --all-targets` still surfaces the same 12 transitive `quartzite-runtime` errors that were present before subtask 7 — those are subtask 8 work.)

### Subtask 8 (`quartzite-runtime`) notes

- **`clippy.toml`** did **not** need any new entries — the only `doc_markdown` warning (`object_tree.rs:28` on the bare `ObjectIds` token in a private field comment) was fixed by inline backticking, not allowlisting. Same treatment applied to the four other `ObjectId` / `MetaObject` / `RwLock` / `Arc<Mutex<...>>` mentions in field comments and module docs.
- **`# Errors`** added to all three flagship `Result`-returning sites:
  - `Application::new` — `ApplicationError::AlreadyExists` if an `Application` is already installed in this process.
  - `ConnectionTable::install_as_dispatcher` — `DispatcherAlreadySet` if a queued dispatcher is already registered.
  - `ObjectFactory::install` — `FactoryAlreadySet` if a factory is already installed.
- **`# Panics`** added to every site clippy flagged. All cases are `Mutex`/`RwLock` poisoning (panic only if a previous holder panicked while emitting / mutating). Sites:
  - `Application::exec` (transitively, through `EventLoop::run`'s receiver-mutex lock).
  - `EventLoop::run` (receiver mutex; secondary precondition: only one thread may be inside `run` at a time).
  - `ConnectionTable::register`, `remove`, `remove_by_receiver`, `receivers_for_signal` (all four `RwLock` accessors).
  - `Timer::connect_timeout`, `disconnect_timeout`, `start` (signal mutex).
  - `Timer::stop` is **not** flagged — it only does `running.store(false)` and `handle.join()`; no `unwrap` on a lock. Left without `# Panics`.
  - `ThreadPool::new` already had `# Panics` for the `assert!(size > 0)` precondition; left as-is and added `# Parameters`.
- **Tense fixes throughout:** `Create` → `Creates`, `Insert` → `Inserts`, `Remove` → `Removes`, `Run` → `Runs`, `Post` → `Posts`, `Stop` → `Stops`, `Find` → `Returns the slice…`, `Move` → `Moves`, `Rename` → `Renames`, `Clear` → `Clears`, `Submit` → `Submits`, `Connect` → `Connects`, `Disconnect` → `Disconnects`, `Wrap` → `Wraps`, `Convert` → `Converts`, `Register` → `Registers`, `Install` → `Installs`, `Signal` → `Signals`, `Access the global` → `Returns a handle to the global`. `Clone the sender` → `Returns a clone of the sender`.
- **`# Parameters`** added to every public fn with ≥1 non-receiver arg. Conservative descriptions ground every bullet in code I read directly — no orchestrator-review flags.
- **`# Examples`** added at the type level for `Application`, `ApplicationError`, `ConnectionRecord`, `ConnectionTable`, `EventLoop`, `FactoryAlreadySet`, `ObjectFactory`, `ObjectRef<T>`, `WeakRef<T>`, `ObjectTree`, `ThreadPool`, `Timer` (some had only constructor-level examples; convention requires one on every `pub` item).
- **Doctest fix on `ApplicationError`:** the first attempt used `Application::new().unwrap_err()` which requires `Application: Debug` (not implemented). Rewrote as a `match` arm that asserts `Err(ApplicationError::AlreadyExists)` — semantically equivalent, no `Debug` requirement on the `Ok` variant.
- **Trait-impl exemption (AC4) honored:** the `impl Default for EventLoop`, `impl QueuedDispatcher for ConnectionTable`, `impl Drop for ThreadPool`, `impl Drop for Timer`, and the various `Copy`/`Clone`/`PartialEq`/`Eq`/`Hash` impls on `ObjectRef<T>` / `WeakRef<T>` were all left undocumented per the convention's trait-impl skip rule. Same for `impl std::error::Error` and `impl std::fmt::Display` for `ApplicationError` / `FactoryAlreadySet`.
- **Test-only `new_ret_no_self` warning disposition:** Renamed `LogObj::new` → `LogObj::boxed` in `quartzite-runtime/tests/object_tree.rs` (test helper that returns `Box<dyn Object>`, not `Self`). Followed the existing `Stub::named` precedent in the same file (`fn named(name: &str) -> Box<dyn Object>` — no `new` name, no warning). The four call sites in `destroy_is_depth_first_post_order` updated. **No `#[allow(clippy::new_ret_no_self)]` was needed** — the rename is the cleaner fix and matches the file's existing style.
- **Doctest count:** 63 doctests in `quartzite-runtime` (was 0 before subtask 2 lints landed; substantial jump from this subtask). All pass. Plus 40 unit tests + 25 integration tests across 5 binaries.

### Subtask 9 (`quartzite` facade) notes

- **Scope confirmed minimal.** The facade is module re-exports plus a `prelude` module. Nothing inherent to document beyond the crate-level `//!` and the six `pub mod { pub use … }` declarations. No `pub use` items needed touching (re-exports inherit docs from the source crate per AC4 spirit).
- **Crate-level `//!` doc.** Added a third-person summary line directly after the `document_features::document_features!()` insertion: `Provides a single facade crate that re-exports the workspace member crates ([`core`], [`events`], [`geometry`], [`macros`], [`runtime`]) plus a curated [`prelude`] for one-line imports.` Renamed the existing `## Getting started` section to `# Examples` so the convention's always-present rule is satisfied at the crate root; the body (`use quartzite::prelude::*;` plus the `MetaEnum` note) was preserved verbatim. Reworded the lead-in to `Imports the prelude …` (third person present indicative). Section ordering is now: summary line → free-form prose (none) → `# Examples` — convention-conforming.
- **Six module-level `///` docs.** Each `pub mod` declaration's summary line was rewritten from a noun-phrase fragment (`Core object model, signals, …`) to a third-person present indicative verb phrase (`Re-exports the core object model, signals, …`). Same treatment applied to `macros`, `runtime`, `geometry`, `events`, `prelude`. Trailing prose paragraphs (`Prefer …`, `Provides integer …`, etc.) preserved as-is. No `# Parameters`, `# Errors`, `# Panics`, or `# Examples` blocks needed at module level — convention scope per `ai-docs/doc-convention.md` lists `pub fn / struct / enum / trait / union / macro_rules`, not `pub mod`, and design row 9 explicitly limits the facade audit to "tense pass; reorder if any heading exists; ensure module docs do not raise `doc_markdown` warnings".
- **`clippy.toml`** did **not** need any new entries — every CamelCase identifier in the facade prose (`MetaEnum`, `PointF`, `SizeF`, `RectF`, `MouseEvent`, `KeyEvent`, `ResizeEvent`, `CloseEvent`, `TimerEvent`, `EventFilter`, `EventType`) was already on the seed list, and the rest are already wrapped in backticks (`` `Extend` ``, `` `Object` ``, `` `object_impl` ``, `` `object_part` ``, `` `Point` ``, `` `Size` ``, `` `Rect` ``, `` `Margins` ``).
- **No conservative descriptions left for orchestrator review** — every reworded line is grounded in the visible re-export contents.
- **Test gate:** 1 unit test (`prelude_compiles`) + 1 doctest (the `# Examples` block) = 2 tests, both pass. No regression.
- **AC10 milestone hit:** `cargo build -p quartzite --no-default-features` PASS, completing AC10 across all three crates that exercise the no_std / derive-free path (`quartzite-geometry`, `quartzite-events`, `quartzite`).
- **AC3 milestone hit:** the audit is now complete for every public item across all six workspace crates (`quartzite-core` ✓ `quartzite-geometry` ✓ `quartzite-events` ✓ `quartzite-macros` ✓ `quartzite-runtime` ✓ `quartzite` facade ✓). AC4 trait-impl exemption was honored throughout (no facade-level trait impls exist to skip).

### Subtask 10 (review skill + agents) notes

- **Three files updated together per Propagation Rule.** `.claude/skills/code-review/SKILL.md`, `.claude/agents/review-findings.md`, `.claude/agents/self-review.md` — all in the same operation. Each cites `ai-docs/doc-convention.md` (relative path adjusted per file depth — `../../../` from the skill, `../../` from the two agent files; both forms verified with `realpath` to resolve to `/home/syt/RustroverProjects/quartzite/ai-docs/doc-convention.md`).
- **Checklist item shape (parallel across both agent files).** Each lists the same actionable triggers a reviewer can spot: imperative summary line ("Returns" required, not "Return"); missing `# Parameters` on a public fn with ≥1 non-receiver arg; section ordering violation (canonical order pasted in full so reviewers don't have to re-open the convention); missing `# Errors` on a `Result`-returning public fn; missing `# Panics` on a fn that calls `unwrap`/`expect`/indexing/asserts/overflowing arithmetic; missing `# Safety` on every `unsafe fn`; ad-hoc sections (only canonical headings allowed). Each restates the AC4 trait-impl exemption explicitly so reviewers don't flag exempt items. Each includes the design's mechanical heading-scan as a copy-pasteable `rg` one-liner.
- **Skill placement** (`.claude/skills/code-review/SKILL.md`). New item 6 in the Step 4 final-verify checklist, sitting between the workspace doc gate and the progress-file update. The Gate checklist row was updated from "all five checks pass" → "all six checks pass (build, test, clippy, fmt, doc, doc convention)" so the gate description stays accurate. The skill body cites the convention via `[ai-docs/doc-convention.md](../../../ai-docs/doc-convention.md)`.
- **Findings-agent placement** (`.claude/agents/review-findings.md`). New §6 *Documentation conformance* section appended after §5 Style, before the existing *What you do NOT check* section. Numbering matches `self-review.md`'s §6 Documentation, which keeps the structure parallel between findings producer and fix validator (per the Propagation Rule sync group).
- **Self-review-agent placement** (`.claude/agents/self-review.md`). The new block was appended to the existing §6 Documentation section rather than promoted to §7, because the existing §6 already covers `cargo doc` lint-level checks — the doc-convention block extends the same documentation theme with the convention's behavioural-only checks (tense, ordering, trait-impl exemption). Keeping it inside §6 avoids re-numbering §7 *Objection quality* and keeps the agent's checklist contiguous.
- **Grep cross-check after edits.** `grep -rn "doc-convention" .claude/agents/ .claude/skills/ AGENTS.md` returns exactly four lines: the three files updated in this subtask plus `AGENTS.md` line 60 (the *Code Style* pointer added in subtask 1). No other instruction file references the convention — Propagation Rule procedure step 1 satisfied.
- **Verify gate.** No source touched, but `cargo build --workspace` PASS confirms the tree still compiles. Markdown links traced via `realpath`: both relative forms resolve correctly to the canonical convention doc.

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
| AC3 | PASS (subtasks 3+4+5+6+8+9 — every workspace crate audited: `quartzite-core`, `quartzite-geometry`, `quartzite-events`, `quartzite-macros` own API, `quartzite-runtime`, and the `quartzite` facade) |
| AC4 | NOT_TESTED |
| AC5 | PASS (subtask 2 — five lints in every `lib.rs`) |
| AC6 | PASS (subtask 1 — `clippy.toml` seeded; no new entries needed during subtask 3) |
| AC7 | NOT_TESTED |
| AC8 | NOT_TESTED |
| AC9 | NOT_TESTED |
| AC10 | PASS (subtasks 4+5+9 — `quartzite-geometry --no-default-features`, `quartzite-events --no-default-features`, and `quartzite --no-default-features` (no_std / derive-free path) all build clean) |
| AC11 | PASS (subtask 7 — codegen now emits `# Parameters` + `# Examples` on `emit_<sig>`, `connect_<sig>_auto`, `connect_<sig>_queued` wrappers, plus single-line summaries on the two `As<Self>` trait-definition accessor methods; all three subtask-6 TDD lock tests now green) |
| AC12 | PASS (subtask 10 — `.claude/skills/code-review/SKILL.md`, `.claude/agents/review-findings.md`, `.claude/agents/self-review.md` updated together (Propagation Rule); each cites `ai-docs/doc-convention.md` and lists the actionable convention checks) |
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
- `src/lib.rs` — added 5 lint attrs (subtask 2); subtask 9 added a third-person crate-level summary line, renamed `## Getting started` to `# Examples`, and rewrote each of the six `pub mod` (`core`, `macros`, `runtime`, `geometry`, `events`, `prelude`) summary lines to third-person present indicative (`Re-exports …`)
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
- `quartzite-macros/src/object/codegen.rs` — subtask 6 added two TDD tests (`emit_wrapper_doc_contains_parameters_and_examples`, `connect_auto_wrapper_doc_contains_parameters_and_examples`); subtask 7 updated three `quote!` sites (`emit_signal_wrappers`, `emit_connect_auto_wrappers`, `emit_connect_queued_wrappers`) to emit `# Parameters` + `# Examples` (`no_run`) doc blocks on every user-facing wrapper, in convention-mandated order
- `quartzite-macros/src/extend/codegen.rs` — subtask 6 added one TDD test (`root_trait_methods_carry_docs`); subtask 7 added single-line `#[doc = "…"]` attributes to both trait-definition accessor methods (`#acc`, `#acc_mut`) inside `emit_root_trait_and_impl` (the matching `impl As<Self> for <Self>` impl methods stay undocumented per AC4 exemption)
- `quartzite-runtime/src/application.rs` — `# Errors` on `Application::new`; `# Panics` on `Application::exec`; `# Parameters` on `post_event`; tense fixes throughout; `# Examples` on `Application` and `ApplicationError` (the latter rewritten as a `match` arm to avoid requiring `Application: Debug` for `unwrap_err`)
- `quartzite-runtime/src/connection_table.rs` — `# Errors` on `install_as_dispatcher`; `# Panics` on `register`, `remove`, `remove_by_receiver`, `receivers_for_signal`; `# Parameters` on every multi-arg accessor; tense fixes; `# Examples` on `ConnectionRecord` and `ConnectionTable`
- `quartzite-runtime/src/event_loop.rs` — `# Panics` on `run`; `# Parameters` on `post`; tense fixes (`Run`/`Post`/`Signal`/`Clone` → 3rd-person present); `# Examples` on `EventLoop` itself
- `quartzite-runtime/src/factory.rs` — `# Errors` on `install`; `# Parameters` on `install`, `register`, `create`; tense fixes throughout; `# Examples` on `ObjectFactory` and `FactoryAlreadySet`; minor markdown cleanup of `global` description (trailing periods + comma list)
- `quartzite-runtime/src/object_ref.rs` — `# Parameters` on all four `new` / `is_valid` constructors; `# Examples` added at the type level on both `ObjectRef<T>` and `WeakRef<T>`; `#[inline]` added to the simple non-generic `new`/`id`/`downgrade`/`is_valid` getters; tense fixes (`Wrap`/`Convert` → 3rd-person)
- `quartzite-runtime/src/object_tree.rs` — `# Parameters` on every public method (`insert`, `contains`, `with`, `with_mut`, `parent_of`, `children_of`, `reparent`, `find_by_name`, `rename`, `clear_name`, `destroy`); `doc_markdown` fixes on every `ObjectId` mention in field comments; tense fixes throughout; `# Examples` added on the type itself; `contains` doctest upgraded from `no_run`-with-fake-helper to a real compiling assertion
- `quartzite-runtime/src/thread_pool.rs` — `# Parameters` on `new` and `spawn`; tense fixes (`Submit` → `Submits`, `Create` → `Creates`); `# Examples` on `ThreadPool` itself
- `quartzite-runtime/src/timer.rs` — `# Panics` on `connect_timeout`, `disconnect_timeout`, `start`; `# Parameters` on `new`, `connect_timeout`, `disconnect_timeout`, `start`; tense fixes throughout (`Create`/`Connect`/`Disconnect`/`Start`/`Stop`); `# Examples` on `Timer` itself; doctest on `new` and `is_running` upgraded from `no_run` → compiling
- `quartzite-runtime/tests/object_tree.rs` — renamed `LogObj::new` → `LogObj::boxed` (clippy `new_ret_no_self` — test helper that returns `Box<dyn Object>`, not `Self`); updated 4 call sites in `destroy_is_depth_first_post_order`
- `.claude/skills/code-review/SKILL.md` — Step 4 verify checklist gained item 6 (*Doc convention conformance*), citing `ai-docs/doc-convention.md` via the `../../../` relative path; gate-checklist row updated from "all five checks pass" → "all six checks pass (build, test, clippy, fmt, doc, doc convention)"
- `.claude/agents/review-findings.md` — added §6 *Documentation conformance* between §5 Style and *What you do NOT check*; cites `ai-docs/doc-convention.md` via `../../`; lists the actionable triggers (imperative summary, missing `# Parameters`, missing `# Errors`/`# Panics`/`# Safety`, section-ordering violation, ad-hoc sections); explicitly restates the AC4 trait-impl exemption; includes the `rg` heading-scan one-liner
- `.claude/agents/self-review.md` — appended a parallel *Doc convention conformance* block at the end of §6 Documentation; same shape, same triggers, same `rg` heading-scan, same AC4 exemption; uses the same `../../` relative path

## Audit worklist (from subtask 2 baseline clippy run)

`cargo clippy --workspace --all-targets` output: 28 warnings + 2 errors.

- `quartzite-core (lib)`: 2 × `missing_errors_doc`.
- `quartzite-runtime (lib)`: 12 warnings — mix of `missing_errors_doc`, `missing_panics_doc`, `doc_markdown` (2 backtick fixes).
- `quartzite-macros (lib)`: 1 × `doc_markdown` or similar.
- `quartzite-runtime (lib test)`: same 12 (duplicates).
- `quartzite-runtime (test "object_tree")`: 1 × `methods called 'new' usually return Self` — **investigate scope**.
- `quartzite-core (lib test) ERROR`: `clippy::approx_constant` at `quartzite-core/src/value.rs:429` — pre-existing, fix in subtask 3.

Run `cargo clippy --workspace --all-targets 2>&1` again at the start of each crate-audit subtask to refresh the worklist after each fix.
