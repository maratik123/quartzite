# Design: Bump MSRV to Rust 1.96.0

**Issue:** #584
**Date:** 2026-05-29

## Approach

This design has been **re-amended (2026-05-30)** to the user's final governing
policy: **`#[derive(Debug)]` ONLY where CHEAP**. The prior round's EXPENSIVE
`Debug` supertrait on the public `AsWidget` trait is **REVERSED**, and a new
**workspace-wide cheap-`Debug` sweep** (spec Scope item 7 / AC16) is added.

Definitions used throughout:
- **CHEAP** = a plain `#[derive(Debug)]` (or adding `Debug` to an existing
  `#[derive(...)]` list) compiles with **no other change** — every field /
  variant-payload type already implements `Debug`, and **no public-trait `Debug`
  supertrait** is introduced. KEEP / ADD these.
- **EXPENSIVE** = making the type `Debug` would require a `Debug` **supertrait**
  on a public trait (`AsWidget`, `Object`). REJECTED. Sites that need such a
  type to be `Debug` for `assert_matches!` are **CARVED OUT** of AC4 (they stay
  `assert!(matches!(...))`).

Four work strands. Strands 1–3 are mechanical (declaration + prose, test-macro
migration, miri-pin bump) and are **already committed** for the bulk of the
test-suite. Strand 4 replaces the rejected cascade with: **(A) unwind the
widget cascade**, **(B) renderer cheap derive + conversions**, **(C) runtime
carve-out + one partial conversion**, **(D) workspace-wide cheap-`Debug`
sweep**, **(E) miri bump + gate run**.

### Strand 1–3 (DONE — committed)

1. **Declaration + prose bump (3 lines).** `Cargo.toml:24` MSRV `1.95`→`1.96`
   (single source of truth; all members inherit via `rust-version.workspace =
   true`), `README.md:82` and `design-system/README.md:87` prose. (Subtasks 1–2
   committed: 61d657e, 71acbc4.)

2. **`assert_matches!` migration — committed crates.** Test-code
   `assert!(matches!(value, Pattern))` → std `assert_matches!(value, Pattern)`,
   crate-root macro import per crate's `#![no_std]` status: `use
   core::assert_matches;` in `#![no_std]` **quartzite-paint-api**, `use
   std::assert_matches;` in all std crates / `tests/`. The obsolete nightly
   module path `use std::assert_matches::assert_matches;` does NOT resolve on
   stable 1.96 (E0432) and MUST NOT appear. Three forms supported: plain `P`,
   guard `P if g`, message `P, "msg {x:?}"`. (Subtasks 3–5, 9 committed: 0f879f0
   paint-api, fde2691 style-dispatch, c7c6f09 core, c226b66
   signal_to_signal/meta_enum/default_style_tests.)

3. **miri.yml pinned-nightly bump (1 file).** `.github/workflows/miri.yml`
   `nightly-2026-05-01` → `nightly-2026-05-29` at both literal sites (toolchain
   pin line 53, comment line 62), then `actionlint .github/workflows/miri.yml`
   (AGENTS.md AXIOM). `coverage.yml` untouched. (Open — subtask 10.)

### Strand 4 — the re-amended `Debug` work

**(A) Unwind the widget cascade (AC14, carve-out of the 7 widget sites).**
The prior round added an EXPENSIVE `Debug` supertrait to the macro-generated
`AsWidget` trait and converted the 7 `WidgetView` sites on top of it. Both are
reversed; the **cheap** concrete-widget-struct derives are KEPT. Three pieces,
all mechanically verified (see *Revert mechanics* below):

- **Revert eb31a62** (`git revert --no-commit eb31a62`) — restores
  `quartzite-macros/src/extend/codegen.rs` to its pre-cascade state: the
  `emit_root_trait_and_impl` supertrait builder loses the `WidgetBase`-gated
  `+ ::core::fmt::Debug` branch (back to the simple `ir.base_field`-only
  `Option`), and the two codegen tests revert (the positive test drops its
  `Debug`-present assertion; the negative `non_widget_base_root_no_widget_view`
  reverts its fixture `struct BoxLayout { #[base] object_base: ObjectBase }` →
  `struct BoxLayout { x: i32 }` and drops the `AsObject`/`Debug` assertions). Net:
  `pub trait AsWidget: AsObject` carries **no** `Debug` bound (AC14). Verified:
  the revert applies cleanly with no conflict.
- **Revert 38c9853** (`git revert --no-commit 38c9853`) — restores the 7
  `WidgetView` test sites to `assert!(matches!(...))` and removes their `use
  std::assert_matches;` imports across `widget_base.rs` + `widgets/{button,
  label,text_edit,scroll_area,container,line_edit}.rs`. These 7 are now
  **CARVED OUT** of AC4 (their `WidgetView` scrutinee needs the rejected
  supertrait). Verified: the revert applies cleanly with no conflict.
- **One targeted edit** — delete `#[derive(Debug)]` at
  `quartzite-widgets/src/widget_base.rs:167` (the `WidgetView<'a>` enum, whose
  `Other(&'a dyn AsWidget)` variant cannot be `Debug` without the supertrait).
  This is the ONLY part of 16a4d99 that is removed; the 9 concrete-struct
  derives (`WidgetBase`, the 6 widgets, `ThirdPartyWidget`, `ClippingWidget`)
  from 16a4d99 are KEPT — they are cheap, additive, non-breaking (AC15), and
  are subsumed by the workspace sweep (D).

  *Mechanics note:* run the two `git revert --no-commit` first, then the single
  `Edit` removing the `WidgetView` enum derive. Because 38c9853 only touched the
  7 `assert` lines + imports (never the enum derive, which came from 16a4d99),
  the revert of 38c9853 does **not** conflict with the manual enum-derive
  deletion. The reverts can be squashed into one commit with the manual edit, or
  committed as a single "unwind" commit — either way the working tree must end
  in: AsWidget supertrait-free, the 7 widget sites back to `assert!(matches!)`,
  the 9 concrete widget structs still `#[derive(Debug,…)]`, and `WidgetView` NOT
  `Debug`.

**(B) Renderer cheap derive + conversions (AC13).** Add cheap
`#[derive(Debug)]` to the crate-PRIVATE enum `LocalBrushKind`
(`quartzite-renderer/src/vello_painter.rs:156`; payloads `&Color`, `&Point`,
`&[ColorStop]`, etc. are all `Debug`). With it, convert the **12** renderer
test-code sites to `assert_matches!` (`use std::assert_matches;` per `mod
tests`): `render_harness.rs` (4 — `RendererError` ✓Debug), `event_convert.rs`
(4 — `MouseEventKind`/`KeyEventKind` ✓Debug), `vello_painter.rs` (4 —
`LocalBrushKind` now Debug). (Open — was subtask 8.)

**(C) Runtime carve-out + one partial conversion (AC4).** In quartzite-runtime
the snapshot module is `#[cfg(feature = "serde")]`-gated (`lib.rs:33`):
- **CARVE OUT** (stay `assert!(matches!)`, known-allowed): `snapshot/object.rs:316`
  and `:332` (scrutinee `restore_object(&snap)` → `Result<Box<dyn Object>,
  DeserializeError>`; `Box<dyn Object>` is not Debug → the rejected `Object`
  supertrait would be required), `snapshot/tree.rs:450` and `tests/snapshot.rs:451`
  (scrutinee `restore_tree(&snap)` → `Result<ObjectTree, DeserializeError>`;
  `ObjectTree` is not cheaply Debug — see exclusion list in (D)).
- **CONVERT** the single site `snapshot/object.rs:421` (scrutinee
  `capture_object(...)` → `Result<ObjectSnapshot, SerializeError>`; **verified**
  `ObjectSnapshot` already `#[derive(Debug, …)]` at
  `quartzite-core/src/snapshot/object.rs:32` and `SerializeError` is a Debug
  error enum, so `assert_matches!` compiles). Add `use std::assert_matches;` to
  that `mod tests`. (Open — was subtask 7, now reduced to one conversion.)

**(D) Workspace-wide cheap-`Debug` sweep (AC16) — the new big scope.** Add
`#[derive(Debug)]` to every **public** struct/enum that derives `Debug` cheaply.
The concrete per-crate set below was **empirically derived**: each candidate had
`#[derive(Debug)]` inserted and the owning crate compiled (`cargo build -p
<crate> --all-features`); CHEAP = compiled, NOT-CHEAP = E0277. Interdependent
pairs were re-tested together (e.g. `ConnectionTable` only compiles once
`ConnectionRecord` is also Debug). The cumulative runtime cheap set was built
together and is **GREEN**. This sweep SUBSUMES the AC15 widget-struct derives
and the AC13 `LocalBrushKind` derive (don't double-count those).

**(E) miri bump + gate run.** miri.yml nightly bump (B-strand 3 above) + the
full gate matrix, with the residual `assert!(matches!` grep whitelisting the
carve-outs and `cargo test` run with `--all-features` so the serde-gated runtime
sites compile.

---

### Workspace cheap-`Debug` sweep — the enumerated set (the deliverable)

Method: 113 `pub`/`pub(crate)` struct+enum declarations across 15 crates were
listed; 38 lacked any `Debug` (derive or manual impl). Each was empirically
compile-tested. Manual `Debug` impls already present (NOT in scope, no-op):
`Signal`, `MetaObject`, `EnumMeta`, `RenderHarness`. The **EMPHASIS is public
types**; `pub(crate)` cheap derives are included as secondary (spec item 7).

**CHEAP — gain `#[derive(Debug)]` (the AC16 set). Count: 20 types.**

| Crate | Type | Vis | File:line | Derive mechanics |
|---|---|---|---|---|
| quartzite-style | `StyleRegistry` | pub | `src/registry.rs:59` | unit struct; add `#[derive(Debug)]` |
| quartzite-widgets | `WidgetChildren<'a>` | pub | `src/widget_base.rs:216` | enum; `&[ObjectId]`/`Option<ObjectId>` ✓ |
| quartzite-widgets | `WidgetChildrenIter<'a>` | pub | `src/widget_base.rs:241` | enum; `slice::Iter`/`option::IntoIter` ✓ |
| quartzite-widgets | `BoxLayout` | pub | `src/layout/box_layout.rs:44` | `#[derive(Extend)]`→add `Debug`; `ObjectBase`/`Direction`/`Vec<(ObjectId,i32)>` ✓ |
| quartzite-widgets | `GridLayout` | pub | `src/layout/grid_layout.rs:112` | `#[derive(Extend)]`→add `Debug`; `ObjectBase`/`Vec<GridCell>` ✓ |
| quartzite-runtime | `ApplicationBuilder` | pub | `src/application_builder.rs:34` | `Option<Duration>` ✓ |
| quartzite-runtime | `ConnectionRecord` | pub | `src/connection_table.rs:31` | `ObjectId`/`SignalIndex` ✓ |
| quartzite-runtime | `ConnectionTable` | pub | `src/connection_table.rs:55` | needs `ConnectionRecord` Debug (paired) |
| quartzite-runtime | `EventLoop` | pub | `src/event_loop.rs:55` | std `Sender`/`Mutex<Receiver>` are Debug ✓ |
| quartzite-runtime | `ThreadPool` | pub | `src/thread_pool.rs:27` | `Option<Sender<Task>>`/`Vec<JoinHandle>` ✓ |
| quartzite-runtime | `ThreadDriver` | pub | `src/timer_drivers.rs:47` | `Arc<AtomicBool>`/`Mutex<Option<(Thread,JoinHandle)>>` ✓ |
| quartzite-runtime | `AppDriver` | pub | `src/timer_drivers.rs:139` | same shape as `ThreadDriver` ✓ |
| quartzite-runtime | `RegistryGuard` | pub(crate) | `src/loop_registry.rs:37` | unit struct ✓ |
| quartzite-runtime | `LoopRegistry` | pub(crate) | `src/loop_registry.rs:50` | unit struct ✓ |
| quartzite-runtime | `TimerState` | pub(crate) | `src/timer.rs:70` | `Arc<Mutex<Signal<…>>>` (Signal manual-Debug) / atomics ✓ |
| quartzite-renderer | `WindowedApplicationBuilder` | pub | `src/application_builder.rs:53` | `bool`/`Option<Duration>`/cfg bool ✓ |
| quartzite-renderer | `RenderHarnessBuilder` | pub | `src/render_harness.rs:52` | `u32`/`u32`/`f32` ✓ |
| quartzite-renderer | `ActiveLoopGuard` | pub(crate) | `src/wrapped_handler.rs:31` | trivially Debug ✓ |
| quartzite-renderer | `LocalBrushKind<'a>` | private | `src/vello_painter.rs:156` | AC13 instance; `&Color`/`&Point`/… ✓ |
| quartzite-widgets | `WidgetBase` + 6 widgets + `ThirdPartyWidget` + `ClippingWidget` | pub / test | (16a4d99) | AC15 instances — **already committed**, KEPT |

**Count discipline:** the table's last two rows are the AC13/AC15 instances
(already present or added in strands A/B) — they are members of the AC16 set,
NOT new work in subtask D. The **net-new** sweep derives applied by subtask D
are the **18** rows above the last two: `StyleRegistry`, `WidgetChildren`,
`WidgetChildrenIter`, `BoxLayout`, `GridLayout`, `ApplicationBuilder`,
`ConnectionRecord`, `ConnectionTable`, `EventLoop`, `ThreadPool`,
`ThreadDriver`, `AppDriver`, `RegistryGuard`, `LoopRegistry`, `TimerState`,
`WindowedApplicationBuilder`, `RenderHarnessBuilder`, `ActiveLoopGuard`.

**NOT CHEAP — explicitly EXCLUDED (with the empirically-observed blocker).**

| Crate | Type | File:line | Why excluded (E0277 / inspection) |
|---|---|---|---|
| quartzite-paint-util | `TranslateGuard<'a>` | `src/lib.rs:100` | field `&mut dyn Painter` — `dyn Painter` not Debug |
| quartzite-runtime | `Application` | `src/application.rs:70` | `Arc<ApplicationInner>`; `ApplicationInner` not Debug (holds `Mutex<ObjectTree>`/`Arc<EventLoop>`/`Arc<ConnectionTable>`) |
| quartzite-runtime | `ObjectFactory` | `src/factory.rs:40` | `HashMap<String, dyn Fn()->Box<dyn Object>>` constructor — `dyn Fn` not Debug |
| quartzite-runtime | `ObjectTree` | `src/object_tree.rs:28` | `SlotMap<_, Box<dyn Object>>` — `dyn Object` not Debug (rejected `Object` supertrait) |
| quartzite-runtime | `Timer` | `src/timer.rs:165` | field `Box<dyn TimerDriver>` not Debug (even with `TimerState` Debug) |
| quartzite-runtime | `PoolDriver` | `src/timer_drivers.rs:276` | `Arc<PoolInner>`; `PoolInner` not Debug |
| quartzite-renderer | `WindowedApplication` | `src/application.rs:33` | embeds `Application` (not Debug) + `wgpu::Instance` |
| quartzite-renderer | `FontCache` | `src/font.rs:16` | `parley::FontContext`/`LayoutContext` not Debug |
| quartzite-renderer | `VelloPainter<'a>` | `src/vello_painter.rs:45` | `vello::Scene`/`kurbo::Affine` stack/`Box<ParleyCaretCursor>` not Debug |
| quartzite-renderer | `WindowEntry` | `src/window_registry.rs:27` | `dyn WidgetRoot` not Debug |
| quartzite-renderer | `WindowRegistry` | `src/window_registry.rs:59` | contains `WindowEntry` (not Debug) |
| quartzite-renderer | `WrappedHandler` | `src/wrapped_handler.rs:48` | contains `WindowRegistry` (not Debug) |

**Out of the sweep on visibility/test grounds (NOT public API, no-op-ish):**
`TestDispatcher` (quartzite-core, `pub(crate)`, `#[cfg(test)]`-only, holds
`Mutex<Vec<Box<dyn FnOnce()>>>` — not Debug anyway); the proc-macro internal
`pub(crate)` IR types in `quartzite-macros/src/extend/parse.rs`
(`ExtendInput`/`WidgetChildrenField`/`WidgetChildrenKind`/`BaseField`/…) already
carry `#[cfg_attr(test, derive(Debug))]` and are compiler-internal, not exported
API — left as-is.

**Rejected alternatives:**
- *Re-adding the `Debug` supertrait to `AsWidget`/`Object`* — REJECTED by the
  user's final policy (EXPENSIVE; forces `Debug` on every implementor). The 7
  widget sites + the 4 runtime snapshot sites are carved out instead.
- *Converting the 7 `WidgetView` sites / the 4 runtime snapshot carve-outs* —
  REJECTED; their scrutinees can't be cheaply `Debug`.
- *Deriving `Debug` on `Timer`/`ObjectTree`/`Application`/renderer painters* —
  REJECTED; empirically E0277 (non-Debug `dyn`/external fields).
- *External `assert_matches` crate* / *nightly module path
  `use std::assert_matches::assert_matches;`* — REJECTED (AC4 forbids the crate;
  the module path is E0432 on stable 1.96).
- *Converting doc-example (`///`) `matches!` occurrences* — REJECTED (Out of
  scope): `quartzite-paint-api/src/{brush,image,path}.rs` and
  `quartzite-core/src/snapshot/tree.rs` doc lines stay as-is.
- *Single sweeping commit* — REJECTED; per-crate grouping keeps each
  `/context-reset` group build-/test-verifiable.

## Decomposition

Subtasks **1–5, 9** are DONE + COMMITTED (61d657e, 71acbc4, 0f879f0, fde2691,
c7c6f09, c226b66); they remain for AC traceability. The **open work** is
subtasks **6, 7, 8, 10, 11, 12, 13** below. All `matches!` counts are test-code
sites only (doc-`///` excluded).

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | **DONE (61d657e).** MSRV `1.95`→`1.96` in `[workspace.package] rust-version` (line 24) | `Cargo.toml` | — |
| 2 | **DONE (71acbc4).** Prose MSRV refs `1.95`→`1.96` (README:82; design-system:87) | `README.md`, `design-system/README.md` | — |
| 3 | **DONE (0f879f0).** `assert_matches!` in quartzite-paint-api (`#![no_std]` → `use core::assert_matches;`) | `quartzite-paint-api/src/{path,brush}.rs` | 1 |
| 4 | **DONE (fde2691).** `assert_matches!` in quartzite-style-dispatch (guard 516, message 742) | `quartzite-style-dispatch/src/dispatch.rs` | 1 |
| 5 | **DONE (c7c6f09).** `assert_matches!` in quartzite-core (guard 1084) | `quartzite-core/src/value.rs` | 1 |
| 9 | **DONE (c226b66).** `assert_matches!` in `tests/signal_to_signal.rs`, `quartzite-macros/tests/meta_enum.rs`, `quartzite-style/src/default_style_tests.rs` | (those 3 files) | 1 |
| 6 | **NEW — UNWIND the widget cascade (AC14, carve-out of 7 widget sites).** `git revert --no-commit eb31a62` (AsWidget supertrait + codegen tests back to pre-cascade) and `git revert --no-commit 38c9853` (7 `WidgetView` sites back to `assert!(matches!)`, drop their `use std::assert_matches;`). Then ONE `Edit`: delete `#[derive(Debug)]` on the `WidgetView<'a>` enum at `widget_base.rs:167`. KEEP the 9 concrete widget-struct derives from 16a4d99 (`WidgetBase`/6 widgets/`ThirdPartyWidget`/`ClippingWidget`). Verify: `cargo test -p quartzite-macros -p quartzite-widgets`; confirm `pub trait AsWidget` has no `Debug` bound and `WidgetView` is not `Debug`; the 7 sites are `assert!(matches!)` again | `quartzite-macros/src/extend/codegen.rs`, `quartzite-widgets/src/widget_base.rs`, `quartzite-widgets/src/widgets/{button,label,text_edit,scroll_area,container,line_edit}.rs`, `quartzite-style/tests/third_party_paint.rs`, `quartzite-style-dispatch/src/dispatch.rs` | 1 |
| 7 | **NEW — renderer cheap derive + conversions (AC13).** Add `#[derive(Debug)]` to crate-private `LocalBrushKind` (`vello_painter.rs:156`), then convert the 12 renderer test sites to `assert_matches!`: `render_harness.rs` (4), `event_convert.rs` (4), `vello_painter.rs` (4). `use std::assert_matches;` per `mod tests`. Verify `cargo test -p quartzite-renderer` | `quartzite-renderer/src/{vello_painter,render_harness,event_convert}.rs` | 1 |
| 8 | **NEW — runtime carve-out + 1 partial convert (AC4).** Convert ONLY `snapshot/object.rs:421` (scrutinee `Result<ObjectSnapshot, SerializeError>` — both Debug) to `assert_matches!`; add `use std::assert_matches;` to that `mod tests`. LEAVE `snapshot/object.rs:316`, `:332`, `snapshot/tree.rs:450`, `tests/snapshot.rs:451` as `assert!(matches!)` (carved out). Verify with serde feature: `cargo test -p quartzite-runtime --all-features` | `quartzite-runtime/src/snapshot/object.rs` | 1 |
| 10 | Bump pinned nightly `nightly-2026-05-01`→`nightly-2026-05-29` in `.github/workflows/miri.yml` at BOTH sites (toolchain pin line 53; comment line 62). Then `actionlint .github/workflows/miri.yml` (AGENTS.md AXIOM — before `git add`). `coverage.yml` unchanged (AC12) | `.github/workflows/miri.yml` | — |
| 11 | **NEW — cheap-`Debug` sweep, group 1: quartzite-runtime + quartzite-style.** Add `#[derive(Debug)]` to: runtime `ApplicationBuilder`, `ConnectionRecord`, `ConnectionTable` (pair — Record first), `EventLoop`, `ThreadPool`, `ThreadDriver`, `AppDriver`, `RegistryGuard`, `LoopRegistry`, `TimerState`; style `StyleRegistry`. Verify `cargo build -p quartzite-runtime --all-features && cargo test -p quartzite-runtime --all-features -p quartzite-style` | `quartzite-runtime/src/{application_builder,connection_table,event_loop,thread_pool,timer_drivers,loop_registry,timer}.rs`, `quartzite-style/src/registry.rs` | 6,7,8 |
| 12 | **NEW — cheap-`Debug` sweep, group 2: quartzite-renderer + quartzite-widgets.** Add `#[derive(Debug)]` to: renderer `WindowedApplicationBuilder`, `RenderHarnessBuilder`, `ActiveLoopGuard`; widgets `WidgetChildren`, `WidgetChildrenIter`, and add `Debug` to the `#[derive(Extend)]` lines of `BoxLayout`, `GridLayout`. (The 9 widget structs + `LocalBrushKind` are AC15/AC13 — already present from 6/7, not re-applied.) Verify `cargo test -p quartzite-renderer -p quartzite-widgets` | `quartzite-renderer/src/{application_builder,render_harness,wrapped_handler}.rs`, `quartzite-widgets/src/widget_base.rs`, `quartzite-widgets/src/layout/{box_layout,grid_layout}.rs` | 6,7,8 |
| 13 | **Gate run + verification (AC1, AC5–AC12, AC16).** `cargo build`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt -- --check`; `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`; `cargo build -p quartzite --no-default-features --features libm`; `cargo test --all-features` (exercises serde-gated runtime sites) **and** `cargo test` (default features); `actionlint .github/workflows/miri.yml`; refresh + stage `Cargo.lock`. Residual greps: `rg '1\.95' -g 'Cargo.toml' . README.md design-system/README.md` (empty); `rg 'nightly-2026-05-01' .github/workflows/miri.yml` (empty); `rg 'assert!\(matches!' --type rust` MUST return ONLY the known-allowed set — 8 doc-`///` lines (paint-api brush/image/path, core/snapshot/tree) + 7 widget carve-outs + 4 runtime carve-outs (object.rs:316/332, tree.rs:450, tests/snapshot.rs:451) = 19 lines; any other hit is a defect | `Cargo.lock` (+ verification across all above) | 6,7,8,10,11,12 |

## Handoff plan

Per `.claude/agents/design.md` § Rules → handoff-grouping: **(a)** a `## Handoff
plan` is required for every M ≥ 1; **(b)** non-terminal groups MUST be exactly
**3 consecutive subtasks**; **(c)** each boundary hands off to **`/context-reset`**
per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry);
**(d)** the terminal group MUST be sized within **1..=3**.

Subtasks 1–5 and 9 are ALREADY COMMITTED and are NOT re-executed. The grouping
covers the open subtasks 6, 7, 8, 10, 11, 12, 13. **Open subtask count M = 7.**

`.claude/skills/context-reset/SKILL.md` caps a task at **3 design-defined
groups**. M = 7 fits cleanly in **3 groups (3 + 3 + 1)** within that cap — **no
overflow, no flag to the orchestrator required.** (The workspace-wide sweep was
deliberately split into exactly two coarse per-crate-cluster subtasks — 11 and
12 — precisely to keep the open count at 7 and the grouping within 3×3.)

**NOTE on already-done work:** subtasks 1–5, 9 are COMMITTED. When /task enters
Group A it first verifies these six are committed (`git log --oneline -12`
showing 61d657e, 71acbc4, 0f879f0, fde2691, c7c6f09, c226b66), then begins open
work at subtask 6.

- **Entry into Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
- **Group A:** subtasks 6, 7, 8 — unwind the widget cascade (6), renderer cheap
  derive + conversions (7), runtime carve-out + the one partial conversion (8)
  (3 subtasks; independent of each other, all depend only on committed subtask 1
  — order 6 first so the tree reaches the clean cheap-only state before the
  sweep).
- **Handoff after Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
  Parent /task resumes in Group B with fresh context.
- **Group B:** subtasks 10, 11, 12 — miri.yml nightly bump + actionlint (10),
  cheap-`Debug` sweep group 1 (runtime + style, 11), cheap-`Debug` sweep group 2
  (renderer + widgets, 12) (3 subtasks). 11 and 12 depend on 6/7/8 (the tree
  must be in the clean cheap-only state); 10 is independent.
- **Handoff after Group B:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
- **Group C:** subtask 13 — terminal group (the full gate matrix +
  `Cargo.lock` refresh + the three residual greps incl. the 19-line
  known-allowed `assert!(matches!` whitelist; 1 subtask, within the 1..=3
  range). No handoff after; the single terminal group completes Step 8 in its
  own `/context-reset` subagent.

## Risks

- **Revert conflict in A.** `git revert --no-commit eb31a62` and `… 38c9853`
  were both **test-applied and abort-verified clean** (no conflict) against the
  current `HEAD`. Mitigation: if a later commit on the branch touches
  `codegen.rs` or the 7 widget test sites before subtask 6 runs, re-verify the
  revert applies; on conflict, fall back to targeted `Edit`s reproducing the
  pre-cascade `codegen.rs` supertrait builder (the simple `ir.base_field`-only
  `Option`) and restoring `assert!(matches!)` on the 7 sites.
- **Keeping vs. removing 16a4d99 pieces in A.** The unwind must remove ONLY the
  `WidgetView` enum derive while KEEPING the 9 concrete-struct derives.
  Mitigation: subtask 6 does NOT revert 16a4d99 (it surgically deletes one line);
  `cargo test -p quartzite-widgets` + the residual grep (subtask 13) catch a
  stray removal or a leftover `WidgetView` derive.
- **`assert_matches!` needs `WidgetView` Debug (re-introduced blocker).** After
  A, the 7 widget sites are `assert!(matches!)` again and CARVED OUT —
  converting them would re-fail to compile. Mitigation: subtask 13's residual
  grep treats them as known-allowed; no conversion is attempted.
- **Interdependent sweep pair (`ConnectionRecord`→`ConnectionTable`).**
  `ConnectionTable` only compiles with `ConnectionRecord` also Debug
  (empirically verified — paired test GREEN). Mitigation: subtask 11 adds the
  derive to `ConnectionRecord` **before/together with** `ConnectionTable`; the
  cumulative runtime cheap-set build was verified GREEN.
- **A non-cheap type sneaking into the sweep.** Every CHEAP entry was
  compile-verified; every NOT-CHEAP entry has a recorded E0277/inspection
  reason. Mitigation: `cargo build`/`cargo test` (subtask 13) hard-fails on any
  mis-classified derive, naming the non-Debug field.
- **serde-gated runtime sites not exercised (AC4/AC10).** The snapshot module is
  `#[cfg(feature = "serde")]`. Mitigation: subtask 8 verifies with `--all-features`
  and subtask 13 runs BOTH `cargo test --all-features` and `cargo test`.
- **Residual `assert!(matches!` grep miscount.** The known-allowed set is
  exactly 19 lines (8 doc-`///` + 7 widget carve-outs + 4 runtime carve-outs).
  Mitigation: subtask 13 enumerates them; any extra hit is a defect to fix or
  re-classify.
- **`cargo fmt` reflow of converted multi-line asserts.** Mitigation: run `cargo
  fmt` (not just `--check`) during conversion subtasks; AC7 `--check` verifies.
- **Wrong `assert_matches!` import path (E0432).** Mitigation: each conversion
  subtask prescribes the crate-root re-export form (`std` for all open-work
  crates — none of the open conversions are in the `#![no_std]` paint-api);
  `cargo build`/`cargo test` fail loudly on a stray module-path import.
- **Stale nightly literal in miri.yml (AC12).** Mitigation: subtask 10 edits
  both sites; subtask 13 greps `nightly-2026-05-01` (must be empty) and runs
  `actionlint` (must pass before `git add`).
- **Residual-`1.95` grep silently not scanning member manifests (AC1/AC11).**
  Mitigation: subtask 13 uses `rg '1\.95' -g 'Cargo.toml' . README.md
  design-system/README.md` (walks all member manifests + both prose files); a
  non-empty result is a defect.
- **No-std path regression (AC9).** The sweep + conversions are
  test-/non-`libm`-path only; AC9 (`cargo build -p quartzite
  --no-default-features --features libm`) confirms.

## Test Design

The `assert_matches!` conversions and the `Debug` derives add no behaviour; the
proof is compilation + the existing test suite. The macro-codegen revert is the
one strand touching test *assertions*.

- **Macro codegen revert (subtask 6) — codegen tests revert, no new logic.**
  - Location: `quartzite-macros/src/extend/codegen.rs` `#[cfg(test)] mod tests`.
  - Entry point: `emit_root_trait_and_impl` via the `emit(...)` string-snapshot
    helper.
  - Scenarios after revert: `as_widget_root_emits_widget_view_and_children`
    (positive) no longer asserts a `Debug` supertrait; `non_widget_base_root_no_widget_view`
    reverts its fixture to `struct BoxLayout { x: i32 }` and drops the
    `AsObject`/`Debug` assertions. `cargo test -p quartzite-macros` is GREEN with
    `pub trait AsWidget: AsObject` (no `Debug`).
  - Fixtures: none new — reuse the existing `emit(quote! { … })` helper.
- **`Debug` derive sweep (subtasks 6 KEEP-set, 7, 11, 12) — no unit test; proven
  by compilation.** The owning crate building + `cargo test` passing IS the
  proof every swept type is `Debug`. The cheap set was pre-validated by inserting
  each derive and compiling (CHEAP/NOT-CHEAP recorded); the cumulative runtime
  set built GREEN. No new fixtures.
- **Conversion correctness (subtasks 7, 8):** converted assertions are
  semantically identical to their `assert!(matches!(...))` predecessors;
  `cargo test` (default + `--all-features`) is the regression oracle. No new
  fixtures.
- **Workflow gate (subtask 10):** `actionlint .github/workflows/miri.yml` after
  the nightly-pin bump (re-confirmed in subtask 13).
- **Gate matrix (subtask 13):** build, clippy `-D warnings`, fmt `--check`, doc
  `-D warnings -D missing-docs`, no-std `libm` build, `cargo test --all-features`
  + `cargo test`, `actionlint`, residual-`1.95` grep (AC1/AC11),
  residual-nightly grep (AC12), residual-`assert!(matches!` grep against the
  19-line known-allowed whitelist (AC4/AC16).

## Open questions

- (none)
