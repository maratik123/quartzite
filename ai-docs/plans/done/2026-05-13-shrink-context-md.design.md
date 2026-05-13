# Design: Shrink `ai-docs/context.md` below 35k-char speed bump

**Issue:** #328
**Date:** 2026-05-13

## Approach

`ai-docs/context.md` is 37,139 chars / 217 lines. Target ≤ 30,000 chars (~14% headroom under the 35k AGENTS.md speed bump). Two sections own ~69% of file mass — `## Key Design Decisions` (17,935 chars, 57 data rows) and `### Maintenance plans (cross-cutting)` (7,799 chars, 30 entries). All other sections together account for ~31% and are mostly architecturally-load-bearing (crate names, trait surfaces, mapping tables) — they stay untouched apart from the H2 skeleton AC2 already pins.

**Chosen plan — mixed mechanism (a) + (b) + (c), partitioned per row / per entry. Two new reference pages.**

1. **Maintenance plans — mechanism (c) blanket compress to one-line summaries.** Every one of the 30 entries already has a linked `plans/done/<file>.spec.md` carrying the full detail; nothing being dropped from `context.md` becomes unreachable. The one-line shape:

   ```
   - **<slug>** — <one-line architectural takeaway, ≤ ~120 chars>. [spec](plans/done/<file>)
   ```

   This is the spec's "sensible default" (Key decisions row 3) and is the lowest-risk mechanism: the spec link makes every detail one click away. Expected save: ~4.0–4.5k chars (30 entries × ~150 chars each vs. current ~260 chars/entry average).

2. **Key Design Decisions table — mechanism (a) + (c) per-row partition.** The architecturally load-bearing rows stay in `context.md` (condensed to ≤ ~250 chars / ≤ 2 lines of "Decision" cell, takeaway preserved). The implementation-detail rows extract into a new `ai-docs/key-decisions.md` reference page; their `context.md` row collapses to a one-line summary + anchored `→ ai-docs/key-decisions.md#<anchor>` link, mirroring AGENTS.md's `Code Style` extraction shape (PR #324). The partition is enumerated row-by-row in the table below.

3. **Mechanism (b) dedup pass.** Two near-verbatim repetitions identified during investigation that mechanism (b) collapses:
   - `quartzite-renderer` crate-layout entry (long-form in `## Crate Layout`) ↔ KDD rows 54–57 (Multi-window / WindowEntry / quit / AppEvent). The KDD rows are the canonical statement of *why* each detail exists; `## Crate Layout` keeps the surface-list summary and drops verbiage already restated in KDD.
   - `quartzite-style-types` / `quartzite-style` crate-layout entries (long-form) ↔ KDD rows 46–48 (cycle resolution / StyleRegistry / Style trait surface). Same treatment.

**Rejected alternatives.**

- **Pure mechanism (a) — extract the whole KDD table to `ai-docs/key-decisions.md`.** Rejected: half the table (the "Decision" axioms — ABI, codegen, inheritance, ownership, naming, …) is the workspace's load-bearing summary, and the spec hard-bars weakening grep-target wording. Moving everything one click away breaks the "find the answer inline" affordance for the most-asked-about rows.
- **Pure mechanism (c) — condense every KDD row in place.** Rejected: archival implementation-note rows (snapshot layer, multi-block accumulator details, painter/path/image impl details, multi-window slot internals) cost ~6–8k chars but rarely answer "what does the project say about X?" lookups — they answer "how exactly does X work?" which the linked done-spec already documents better than a table cell can. Extracting them to `key-decisions.md` is the cheapest route to the 30k target.
- **Mechanism (c) only on Maintenance plans, leave KDD untouched.** Rejected: 4–5k savings alone leaves the file at ~32–33k, still above the speed bump. KDD must also shrink.
- **Rename `### Maintenance plans (cross-cutting)` → `### Maintenance plans`.** Rejected: would break the inbound `INDEX.md:115` anchor. Cheaper to keep the heading verbatim than to update `INDEX.md`; AC7 explicitly permits either, design picks "keep heading" for zero-cost anchor stability.
- **Move maintenance entries to a new `ai-docs/maintenance-plans.md` reference page.** Rejected: one-line summaries with spec links are *already* short enough that an intermediate reference page adds a navigation hop without enough char savings to justify it. The done-spec is the canonical detail source; one-line `context.md` summaries link straight there.

## Extraction destinations

**One new reference page: `ai-docs/key-decisions.md`.**

- Created in the same PR as the `context.md` edit.
- Headings: one `## <Question cell verbatim>` per extracted row, body = the current "Decision" cell content (lifted verbatim, prose-format). The verbatim heading guarantees the GitHub-Flavoured-Markdown anchor (`#question-cell-text`) is grep-able and stable.
- Link-back from `context.md`: every extracted row keeps a one-line summary in the table plus a trailing anchored link:

  ```
  | <Question> | <one-line takeaway>. See [ai-docs/key-decisions.md → <Question>](key-decisions.md#<anchor>). |
  ```

- AGENTS.md `## Agent Docs` table gets one new row added in the same PR (Propagation Rule) listing `ai-docs/key-decisions.md` as "Extracted KDD detail bodies from `ai-docs/context.md` § Key Design Decisions. Read on demand." This mirrors the rows added for `ai-docs/workflow.md` / `ai-docs/corrections-log.md` in PR #324.

**No `ai-docs/maintenance-plans.md` created.** Mechanism (c) compression alone hits target on the maintenance side.

**Inbound-anchor preservation.**
- `ai-docs/plans/INDEX.md:115` → `context.md#maintenance-plans-cross-cutting`: heading stays verbatim ⇒ anchor stable, zero cross-file edit.
- No other inbound anchors found (`grep -rn 'context.md#' .claude/ ai-docs/ AGENTS.md CLAUDE.md` returns only this one).
- Outbound anchors from `context.md` introduced by this PR all point into `ai-docs/key-decisions.md` headings whose text is verbatim from the row's "Question" cell — anchors deterministic.

## Per-row partition for `## Key Design Decisions`

Per AGENTS.md `## API Stability` ("Pre-publish: clean breaks") none of these rows is API contract — they document design choices and implementation notes. None are dead, so no DROP partition.

Estimates: a current row averages ~310 chars (17,935 chars / 57 rows). After partition, each KEEP-INLINE row targets ≤ 180 chars (architectural takeaway only) and each EXTRACT row targets ≤ 150 chars in `context.md` (one-line summary + anchored link). Heading + extracted body in `key-decisions.md` costs ~50 chars markup; the bodies themselves move 1:1 so they don't count against `context.md`.

Legend: **K** = KEEP-INLINE (condense in place), **E** = EXTRACT to `key-decisions.md`.

| #  | Question cell                              | Verdict | Why                                                                                  |
|----|--------------------------------------------|---------|--------------------------------------------------------------------------------------|
| 1  | ABI compatibility                          | K       | Foundational axiom (3 chars value). Already minimal.                                  |
| 2  | Code generation                            | K       | Foundational axiom. Already minimal.                                                  |
| 3  | Inheritance model                          | K       | Foundational architecture, single-line.                                               |
| 4  | Hierarchy root marker                      | K       | Foundational; single-line.                                                            |
| 5  | Multiple bases                             | K       | Foundational; single-line.                                                            |
| 6  | `AsObject` vs `Object`                     | K       | Trait-layer contract (grep target).                                                   |
| 7  | Crate naming                               | K       | Repo-structure axiom.                                                                 |
| 8  | Python interop                             | K       | One-liner; high-level scope statement.                                                |
| 9  | Macro codegen paths                        | E       | Implementation detail (proc_macro_crate fallback chain). Done in `quartzite-macros`. |
| 10 | `derive` feature                           | K       | Feature-gate contract; grep target for `no_std` builds.                              |
| 11 | Object ownership                           | K       | Foundational; named in `## Core Architecture` already.                                |
| 12 | `ConnectionType::Auto`                     | E       | Implementation detail of auto-connection; full design in `auto-connection.spec.md`.   |
| 13 | Signal slot storage                        | E       | `IndexMap` impl detail; full design in `lookup-perf.spec.md`.                         |
| 14 | `ObjectBase: Debug`                        | E       | Implementation note (derive + thread_id cfg-gate).                                    |
| 15 | `ObjectBase::name`                         | K       | Contract row — `None` vs `Some("")` semantics (grep target for callers).             |
| 16 | `ObjectTree::rename` no-op                 | E       | Behavioural detail; full design in `objectbase-debug-rename-factory.spec.md`.        |
| 17 | `ObjectFactory` singleton                  | E       | Implementation note; full design in `objectbase-debug-rename-factory.spec.md`.       |
| 18 | `ObjectTree::find_by_name`                 | K       | API contract — return type + ordering (grep target).                                  |
| 19 | `ObjectTree::find_by_name_in`              | K       | API contract — BFS order (grep target).                                               |
| 20 | `ObjectBase::name_changed`                 | E       | Signal-payload encoding detail.                                                       |
| 21 | `name_changed` codegen                     | E       | Codegen mechanics — proc-macro internal note.                                         |
| 22 | `Option<String>` Value encoding            | E       | `IntoValue` impl detail.                                                              |
| 23 | Process-global tree accessor               | E       | `try_with_tree` implementation note (AtomicPtr).                                      |
| 24 | `ObjectBase::new_with_id`                  | E       | Caller-contract footnote.                                                             |
| 25 | Snapshot layer (`serde` feature)           | E       | ~1400-char block; the biggest single row; full design lives in snapshot-layer specs.  |
| 26 | MetaObject / EnumMeta lookup               | E       | O(1) lookup mechanics — implementation note.                                          |
| 27 | `ObjectBase::signals_blocked`              | K       | API contract — `emit!` macro is the canonical way (grep target).                     |
| 28 | `emit!` macro                              | E       | Macro internals — `__blocked` binding sequence; full body in `signal-emit-rename`.    |
| 29 | `emit_<signal>` codegen                    | E       | Codegen-internal note.                                                                |
| 30 | `connect_<signal>_auto` codegen            | E       | Codegen-internal note.                                                                |
| 31 | Multi-block `#[object_impl]`               | E       | `thread_local!` HashMap key construction — codegen internal.                          |
| 32 | Generic `#[derive(Extend)]`                | E       | `split_for_impl` mechanics — codegen internal.                                        |
| 33 | `connect_<signal>_queued` codegen          | E       | Codegen-internal note.                                                                |
| 34 | Per-thread `LoopRegistry`                  | E       | `LazyLock<RwLock<HashMap…>>` + `RegistryGuard` mechanics.                              |
| 35 | `QueuedDispatcher::post` signature         | K       | API contract — `target: ThreadId` parameter shape (grep target).                     |
| 36 | `quartzite-geometry` no_std                | K       | Crate-level constraint (no_std + no alloc + `libm`).                                  |
| 37 | `ObjectId`/`ConnectionId` ordering         | E       | `#[derive(PartialOrd, Ord)]` + `BinaryHeap` use case — implementation note.          |
| 38 | `Timer` pluggable driver                   | K       | API surface — `TimerDriver` trait + 3 built-in drivers (grep target).                |
| 39 | `Timer::tick` signal isolation             | E       | `Arc<Mutex<Signal>>` sharing mechanics — implementation detail.                       |
| 40 | `signals_blocked` two-copy sync            | E       | `AtomicBool` mirror + documented limitation.                                          |
| 41 | Signal-to-signal connections               | E       | API surface for two connect variants — 700+ char block, suits reference page.        |
| 42 | `Timer` implements `Object`                | E       | Manual-impl note (`Object` without macro dep).                                        |
| 43 | `quartzite-events` no_std                  | K       | Crate-level constraint.                                                               |
| 44 | `EventType<T>` shape                       | E       | Generic-param mechanics + winit alignment — implementation note.                      |
| 45 | `PropertyFlags` representation             | E       | `enumflags2` + `make_bitflags!` codegen detail; full body in `enumflags2-property-flags.spec.md`. |
| 46 | `style ↔ widgets` Cargo cycle resolution   | K       | Foundational — explains why two style crates exist (grep target for `quartzite-style-types`). |
| 47 | `StyleRegistry` storage                    | E       | `Box::leak` + non-panicking accessor — implementation detail.                         |
| 48 | `Style` trait surface                      | K       | API contract — single `draw_widget` method, generic-only (grep target).              |
| 49 | `Painter` trait shape (paint-api)          | K       | 11-method contract + pass-through nature (grep target — heavily referenced).         |
| 50 | `Color` representation                     | K       | API contract — `f32` channels (grep target).                                          |
| 51 | `Path` representation                      | E       | Method-by-method builder detail.                                                       |
| 52 | `Image` representation                     | E       | Field-by-field plus error variants.                                                   |
| 53 | `Alignment` location                       | E       | Re-export note + `TypeId` equality assertion — implementation detail.                |
| 54 | Multi-window `ActiveEventLoop` slot        | E       | `Cell<*const>` + guard mechanics — implementation detail.                            |
| 55 | `WindowEntry` drop order                   | E       | Drop-order soundness note — load-bearing but archival.                               |
| 56 | `quit_on_last_window_closed` policy        | E       | Behavioural detail; controllable from builder — already a one-liner-able takeaway.   |
| 57 | `AppEvent` / proxy exit                    | E       | Proxy-based exit mechanics — implementation detail.                                  |

**Partition count.** KEEP-INLINE: 22 rows (1–8, 10, 11, 15, 18, 19, 27, 35, 36, 38, 43, 46, 48, 49, 50). EXTRACT: 35 rows. DROP: 0. (Authoritative — matches the per-row verdict column above row-for-row.)

**Char-budget estimate.**
- Current `## Key Design Decisions` block: 17,935 chars.
- After: 22 KEEP rows × ~180 chars = ~3,960 chars + 35 EXTRACT rows × ~150 chars (one-line summary + anchored link) = ~5,250 chars + heading + table header/divider ≈ ~9,300 chars in `context.md`. (22 × 180 + 35 × 150 = 9,210, plus ~90 chars header/divider.)
- **Save in `context.md` from KDD: ~8,600 chars.** Content moves to `ai-docs/key-decisions.md` (~13,500 chars in the new file).

## Maintenance plans compression plan

Every entry in the current list links to a `plans/done/<file>.spec.md`. I cross-checked each linked spec against the verbose `context.md` text:

- **All 30 specs exist** under `ai-docs/plans/done/`.
- **All 30 specs carry the detail that the verbose `context.md` text duplicates** — the maintenance-plan blurb in `context.md` is, in every case, a restatement of the spec's `## Scope` + `## Acceptance Criteria` boiled into prose. Spot-checked the largest entries: `cleanup-progress-issue-derive`, `shrink-agents-md`, `project-docs`, `tracing-itertools`, `signal-to-signal`, `per-thread-event-loops`, `enumflags2-property-flags` — every architectural takeaway in the verbose blurb is already in the linked spec.

Therefore: **mechanism (c) applies uniformly to all 30 entries.** No entry needs to retain verbose text in `context.md`; none requires augmenting the (out-of-scope-to-edit) historical spec. The one-line-summary template is:

```
- **<slug>** — <one-line takeaway emphasising what changed at the architectural level, ≤ ~120 chars>. [spec](plans/done/<file>)
```

**Proposed one-liners (30 entries):**

1. `cleanup-progress-issue-derive` — `.claude/skills/pr-merged/scripts/cleanup-progress.sh` now derives the spec-lookup key from the issue number resolved from the merged PR body (not the PR number); idempotent, with stderr warnings on miss.
2. `shrink-agents-md` — `AGENTS.md` reduced 40,572 → ~31,466 chars; long narrative extracted to `ai-docs/workflow.md` and `ai-docs/corrections-log.md`; all 8 AXIOMs preserved verbatim.
3. `project-docs` — README description block, facade-crate `lib.rs` rustdoc, `CONTRIBUTING.md`, auto-generated `ROADMAP.md` via `scripts/gen-roadmap.sh`, `roadmap-sync` CI gate.
4. `macro-object-bench` — criterion benchmarks for macro-derived objects in the root `quartzite` facade; mirrors the `quartzite-core` 6-group fixture using `#[derive(Extend, DeriveObject)]` + `#[object_impl]`.
5. `criterion-benchmarks` — `quartzite-core/benches/signal_property.rs` + `quartzite-runtime/benches/object_tree.rs` + 3 Bencher CI workflows.
6. `codegen-inline-concrete-trait-impls` — restored `#[inline]` on concrete-struct trait-impl method emissions across all three codegen modules; branches on user-struct generics.
7. `codegen-simple-marker` — dropped `#[inline]` from generated trait-impl methods; trait-declaration `/// _Simple._` tag is the sole canonical signal, generated impls inherit via rustdoc.
8. `code-style-extraction` — lifted AGENTS.md `## Code Style` body into `ai-docs/code-style.md`; AGENTS.md retains a bullet index; `code-style` added as a new `Escalated?` target.
9. `generic-fn-split` — applied "Generic-fn split for binary size" to `ObjectTree::rename`, `ObjectFactory::register`, `Timer::named`, `ObjectBase::named`; outer `_Simple._`, body in nested `fn inner`.
10. `recursive-inline-annotations` — annotation-only sweep applying the recursive `#[inline]` / `_Simple._` rule across the workspace; cascade verified quiescent.
11. `tracing-spans` — `*_span!` guards replace bare `debug!`/`trace!` in `object_tree`, `timer`, `event_loop`, `timer_drivers`; high-frequency emit/post gated on `verbose-tracing` feature.
12. `signal-to-signal` — `quartzite-core::connect` module: `Object::emit_signal`, `ArgsToValues`, `connect_signal_to_signal`, `connect_signals`, `SignalConnectionError`; std-only.
13. `per-thread-event-loops` — `LoopRegistry` singleton + `EventLoop::install/uninstall_for_current_thread` + `spawn` + RAII `RegistryGuard`; `QueuedDispatcher::post` gains `target: ThreadId`.
14. `doc-convention` — workspace doc-comment convention (RFC 1574 + deterministic.space); clippy `missing_errors_doc` / `missing_panics_doc` / `missing_safety_doc` / `doc_markdown` enabled across all crates.
15. `thiserror-migration` — `ApplicationError`, `FactoryAlreadySet`, `DispatcherAlreadySet`, `TypeError` migrated to `#[derive(thiserror::Error)]`.
16. `tracing-itertools` — `tracing` added to `quartzite-core` + `quartzite-runtime` (`tracing/log` feature); `itertools` dev-dep; `env_logger` in examples.
17. `signal-emit-rename` — `Signal::emit` is now unconditional; `Signal::emit_unless_blocked` removed; `emit!` macro owns the `signals_blocked` guard.
18. `receiver-guard-auto` — `Weak<ReceiverGuard>` for Auto connections + `connect_<signal>_auto` codegen.
19. `connect-queued-codegen` — `connect_<signal>_queued` typed wrappers from `#[derive(Object)]`.
20. `enumflags2-property-flags` — `PropertyFlags` replaced by `BitFlags<PropertyFlag>` backed by `u8` via `enumflags2`; `make_bitflags!` codegen; `enumflags2` re-exported `#[doc(hidden)]` from `quartzite-core`. [spec](plans/done/2026-05-03-enumflags2-property-flags.spec.md)
21. `objectbase-debug-rename-factory` — `ObjectBase: Debug`, `rename` no-op, `ObjectFactory` global singleton.
22. `code-quality-cleanup` — code-quality sweep across `quartzite-macros`, `quartzite-runtime`, `quartzite-core`.
23. `docs-and-facade` — facade-crate documentation pass.
24. `public-api-docs` — public-API doctest pass; 47 new doctests.
25. `lookup-perf` — O(1) signal disconnect, name index, match-based meta lookup; 21 new tests.
26. `inline-simple-fns` — `#[inline]` on simple non-generic fns.
27. `examples-crate` — `quartzite-examples` crate with runnable API examples.
28. `signals-blocked` — typed emit wrappers + `signals_blocked` guard.
29. `auto-connection` — signal/slot Auto connection extension.
30. `geometry-events` — `quartzite-geometry` + `quartzite-events` crates.

Each one-liner's char budget ≈ 150 chars including the `[spec](plans/done/<file>)` link. **Total Maintenance plans block after compression: ~4,500 chars (was 7,799). Save: ~3,300 chars (30 × ~150).**

## Total char budget reconciliation

| Section                              | Before  | After   | Δ        |
|--------------------------------------|---------|---------|----------|
| `## Key Design Decisions` block      | 17,935  |  ~9,300 | −8,600   |
| `### Maintenance plans` block        |  7,799  |  ~4,500 | −3,300   |
| `## Crate Layout` (dedup pass — b)   |  ~3,400 |  ~2,800 |   −600   |
| Other sections                       | ~8,005  | ~8,005  |      0   |
| **Total `ai-docs/context.md`**       | **37,139** | **~24,600** | **−12,500** |
| Target                               |         | ≤ 30,000 | ~5,400-char buffer |

The buffer (≥ 5,000 chars beyond the target) absorbs:
- Whitespace and link-mark-up the per-row estimates can't perfectly predict.
- A future EXTRACT-row that turns out to need slightly more context than estimated; trivially reclassify-able as KEEP without exceeding target.

If the actual post-edit size lands ≥ 28,000 chars (closer to target than expected) → no rework. If ≥ 30,000 chars → reclassify 2–3 of the borderline KEEP-INLINE rows (e.g., 14 `ObjectBase: Debug`, 56 `quit_on_last_window_closed` policy if previously kept) to additional extraction; their one-line summaries shrink further. The ~5,000-char headroom makes this contingency cheap.

## Decomposition

| # | Task                                                                                                                                            | Files                                                                       | Depends on |
|---|-------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------|------------|
| 1 | Create `ai-docs/key-decisions.md` with verbatim-question H2 headings + lifted "Decision" cell bodies for all 35 EXTRACT rows. No edits to `context.md` yet — file is self-consistent (the new reference page is just available). | `ai-docs/key-decisions.md` (new)                                            | —          |
| 2 | Add `ai-docs/key-decisions.md` row to `AGENTS.md § Agent Docs` table (Propagation Rule). Phrase matches PR #324 style: "Extracted KDD detail bodies from `ai-docs/context.md` § Key Design Decisions. Read on demand." | `AGENTS.md`                                                                  | 1          |
| 3 | Rewrite `## Key Design Decisions` table in `context.md`: KEEP-INLINE rows condensed to ≤ 180-char "Decision" cells; EXTRACT rows collapsed to one-line summary + `See [ai-docs/key-decisions.md → <Question>](key-decisions.md#<anchor>).` | `ai-docs/context.md`                                                         | 1, 2       |
| 4 | Rewrite `### Maintenance plans (cross-cutting)` block in `context.md` to 30 one-liners (heading text verbatim — preserves `INDEX.md:115` anchor). | `ai-docs/context.md`                                                         | —          |
| 5 | Apply mechanism (b) crate-layout dedup pass: shorten the `quartzite-renderer`, `quartzite-style-types`, `quartzite-style` rows by removing clauses already canonicalised in KEEP-INLINE KDD rows 46 / 48 / 49 (and renderer mechanics in EXTRACT rows 54–57 reachable via the new anchor). | `ai-docs/context.md`                                                         | 3          |
| 6 | Verify and finalise: run AC1–AC10 checks (`wc -c ai-docs/context.md`, `grep -nE '^## '` H2 ordering, `grep -nE '^\| \`quartzite' crate set, KDD row enumeration, slug enumeration, INDEX.md anchor resolution, `grep -rn 'context.md#' …`, `cargo fmt --check` / `cargo clippy --workspace -- -D warnings` / `cargo build` / `cargo test` no-op pass). | (verification only — no file writes)                                         | 1, 2, 3, 4, 5 |

Each task leaves the tree self-consistent:
- After Task 1: new reference page added, `context.md` still references nothing in it (no broken links).
- After Task 2: AGENTS.md links to the new reference page; reachable; no `context.md` outbound link yet (still 37k).
- After Task 3: KDD rows now link into `key-decisions.md` headings created in Task 1 — every anchor resolves.
- After Task 4: Maintenance plans compressed; all `[spec](plans/done/<file>)` links still resolve.
- After Task 5: Crate-layout dedup pass — only shortens existing rows, removes no rows (AC3 preserved).
- After Task 6: verification + finalisation only.

6 tasks total — within the ≤ 7 ceiling.

## Risks

- **Risk:** The new `ai-docs/key-decisions.md` reference page (**created by this PR**) contains Question-cell headings with backticks (e.g., `` `ObjectBase::name_changed` ``); GFM strips backticks for anchor generation and may produce non-obvious anchors. **Mitigation:** GFM strips backticks for anchor text; predictable anchor for ``ObjectBase::name_changed`` is `objectbasename_changed`. Task 1 author MUST verify each generated anchor by opening the rendered file in a viewer (or use `gh-md-toc` / a quick `grep` against the produced anchors). Where backticks produce a confusing anchor, add a manual `<a name="explicit-anchor"></a>` adjacent to the heading and use that anchor in `context.md`'s link. Risk surfaces are limited to ~35 anchors; verification cost is bounded.

- **Risk:** A KDD row classified KEEP-INLINE is implicitly cited by a future PR review as still being too verbose, OR an EXTRACT row turns out to be more architecturally load-bearing than estimated. **Mitigation:** The classification table in this design document is reviewable; design-review can flip any K↔E assignment before implementation. Char-budget headroom (~5,400 chars below target) absorbs misclassification of 2–3 rows in either direction without exceeding target.

- **Risk:** An agent (`/task`, `/triage`, `/improve`, `/code-review`, …) greps `ai-docs/context.md` for a now-extracted phrase and silently misses. **Mitigation:** Spec § Technical constraints calls out the grep-target list (crate names, trait names, type names). All 22 KEEP-INLINE rows in the partition contain at least one such grep target, deliberately. AC10 enforces `grep -rn` across `.claude/`, `ai-docs/`, `AGENTS.md`, `CLAUDE.md` for every moved phrase. The investigation phase already ran the `context.md` reverse-reference grep — only `ai-docs/plans/INDEX.md:115` cites a specific `context.md` anchor; nothing else cites a specific quoted phrase from KDD.

- **Risk:** Inbound anchor `context.md#maintenance-plans-cross-cutting` (from `INDEX.md:115`) breaks if someone misreads the design and renames the heading. **Mitigation:** Decomposition Task 4 explicitly pins the heading text verbatim. AC7 enforces. Design notes the heading rename as a rejected alternative.

- **Risk:** Spec hard-bars editing `ai-docs/plans/done/*.spec.md`. If a maintenance-plan one-liner needs detail not in the linked spec, design must keep that entry verbose in `context.md`. **Mitigation:** Investigation phase cross-checked each entry's linked spec — all 30 specs already carry the architectural takeaway. Reconciliation: no augmentation needed; mechanism (c) applies uniformly.

- **Risk:** New reference page itself crosses the 35k speed bump in the future (the same problem `context.md` has). **Mitigation:** Out of scope for this PR — the new page is born at ~13,500 chars, well below the threshold. The deferred precommit script (spec § Deferred) is the mechanical guard against future regression across all instruction files at once.

- **Risk:** `Tracked in: #328` cross-link integrity. **Mitigation:** Not touched by this design — the spec → issue cross-link lives in `2026-05-13-shrink-context-md.spec.md:5` and is not affected by `context.md` edits. `/task` Step 12's spec→done/ move runs as normal on PR merge.

- **Risk:** Mechanism (b) dedup pass mis-identifies a "near-verbatim repetition" and removes a clause whose canonical statement was only in the verbose-but-now-deleted location. **Mitigation:** Mechanism (b) only fires for the three crate-rows named (`quartzite-renderer`, `quartzite-style-types`, `quartzite-style`), and only removes clauses that are restated in KDD rows that the partition kept either KEEP-INLINE (canonical statement remains) or EXTRACT with anchored link (canonical statement remains, one click away). Cross-check is part of Task 5.

- **Risk:** Slug-extraction regex `grep -oE '\*\*[a-z][a-z0-9-]+\*\*'` is too loose — it matches `**word**` anywhere in the file body, not just at the start of a bullet, so mid-prose `**emphasis**` runs become false-positive slugs in V5 / V10. **Real-corpus hazard, verified:** the current `cleanup-progress-issue-derive` Maintenance plans entry contains the literal text `the **issue** body` (prose emphasis on the noun "issue"); live measurement against master yields **31** unique tokens, not 30 — the extra token is the spurious slug `issue`. After compression to one-liners, the `**issue**` emphasis is dropped, so the "after" set correctly contains 30 slugs — V5's "empty diff" check would then report a spurious mismatch, and V10's per-slug loop would iterate `slug=issue` and produce 232 stale hits across the repo (every documentation reference to the word "issue" — `gh issue`, "issue body", "issue #N", etc.), drowning the real signal. **Mitigation:** Anchor the regex to bullet-list shape — `grep -oE '^- \*\*[a-z][a-z0-9-]+\*\*'` followed by `sed 's/^- \*\*//; s/\*\*$//'`. Applied in both V5 (`before` and `after` slug extraction) and V10 (slug-derivation pipeline). V1–V4, V6–V9 audited and confirmed unaffected (no other slug-shape extraction in the suite — see V10's "Audit" note).

- **Risk:** Boundary Rule 2 (`learnings.md` not in same turn as instruction-file edits). **Mitigation:** AC9 enforces. No `learnings.md` edit is in-scope for this PR; the design records this as a hard constraint and the implementer must respect it. Pre-existing working-tree changes to `learnings.md` from prior turns MAY be staged with the commit per AGENTS.md `## Workflow` ("stage `learnings.md` together with the related code changes — learnings are part of the task deliverable").

## Test Design

This is a documentation-only change — no Rust tests apply (AC8 explicitly enforces `cargo fmt` / `cargo clippy` / `cargo build` / `cargo test` pass as a sanity check that no `.rs` file was touched). Verification is grep-based, performed in Task 6.

**Verification recipe (Task 6 entry point — runs `bash -c` from repo root):**

### V1 — AC1: file size

```bash
wc -c ai-docs/context.md
# Expected: ≤ 30000
[ "$(wc -c < ai-docs/context.md)" -le 30000 ] && echo "AC1 PASS" || echo "AC1 FAIL"
```

### V2 — AC2: H2 heading sequence

```bash
git show master:ai-docs/context.md | grep -nE '^## ' > /tmp/h2-before.txt
grep -nE '^## ' ai-docs/context.md | sed 's/^[0-9]*://' > /tmp/h2-after-headings.txt
git show master:ai-docs/context.md | grep -nE '^## ' | sed 's/^[0-9]*://' > /tmp/h2-before-headings.txt
diff /tmp/h2-before-headings.txt /tmp/h2-after-headings.txt
# Expected: empty diff (heading text and order identical)
```

Expected output (the 8 lines, in order): `## Purpose`, `## Crate Layout`, `## Concept Mapping`, `## Out of Scope`, `## Core Architecture`, `## Key Design Decisions`, `## Plans (Implementation Order)`, `## Open Questions`.

### V3 — AC3: crate-row set unchanged

```bash
git show master:ai-docs/context.md | grep -E '^\| `quartzite' | sed -E 's/`(quartzite[a-z-]*)`.*/\1/' | sort > /tmp/crates-before.txt
grep -E '^\| `quartzite' ai-docs/context.md | sed -E 's/`(quartzite[a-z-]*)`.*/\1/' | sort > /tmp/crates-after.txt
diff /tmp/crates-before.txt /tmp/crates-after.txt
# Expected: empty diff. The set: quartzite-core, quartzite-macros, quartzite-runtime,
# quartzite-geometry, quartzite-event-types, quartzite-events, quartzite-paint-api,
# quartzite-paint, quartzite-renderer, quartzite-style-types, quartzite-style,
# quartzite-widgets, quartzite (the facade).
```

### V4 — AC4: every KDD row's "Question" still present (inline OR extracted)

Trim trailing / leading whitespace on both sides so the diff is robust against the awk-cell padding (`| Foo |` → ` Foo `) vs the sed-stripped H2 form (`## Foo` → ` Foo`).

```bash
# Before set:
git show master:ai-docs/context.md \
  | awk '/^## Key Design Decisions/,/^## Plans/' \
  | grep -E '^\| ' \
  | grep -vE '^\| Question \| Decision \|$|^\|---\|---\|$' \
  | awk -F'|' '{gsub(/^ +| +$/, "", $2); print $2}' | sort > /tmp/kdd-questions-before.txt
# After set: union of context.md inline questions + key-decisions.md H2 headings.
{
  awk '/^## Key Design Decisions/,/^## Plans/' ai-docs/context.md \
    | grep -E '^\| ' \
    | grep -vE '^\| Question \| Decision \|$|^\|---\|---\|$' \
    | awk -F'|' '{gsub(/^ +| +$/, "", $2); print $2}'
  grep -E '^## ' ai-docs/key-decisions.md | sed 's/^## //; s/^ *//; s/ *$//'
} | sort -u > /tmp/kdd-questions-after.txt
diff /tmp/kdd-questions-before.txt /tmp/kdd-questions-after.txt
# Expected: empty diff (57 unique Question cells, each appears either in context.md inline or as an H2 in key-decisions.md).
```

### V5 — AC5: every Maintenance-plan slug + spec link still present

**Bullet-anchored slug extraction.** The slug regex MUST anchor to the bullet-list shape (`^- **slug**`) — an unanchored `\*\*…\*\*` match would pick up mid-prose `**emphasis**` runs as false-positive slugs. Verified live against the master baseline: the `cleanup-progress-issue-derive` maintenance entry contains the literal phrase `the **issue** body`, which an unanchored regex captures as a spurious slug `issue`, yielding 31 unique tokens on master instead of the 30 real slugs. After compression, mid-prose emphasis disappears, so the "after" set legitimately contains 30 slugs — and the unanchored form would falsely report a one-token mismatch. Anchoring to `^- ` shape eliminates this hazard. The slug regex below also passes through a trim step before sorting, mirroring V4's robustness.

```bash
git show master:ai-docs/context.md \
  | awk '/^### Maintenance plans/,/^## Open Questions/' \
  | grep -oE '^- \*\*[a-z][a-z0-9-]+\*\*' \
  | sed 's/^- \*\*//; s/\*\*$//; s/^ *//; s/ *$//' | sort -u > /tmp/slugs-before.txt
awk '/^### Maintenance plans/,/^## Open Questions/' ai-docs/context.md \
  | grep -oE '^- \*\*[a-z][a-z0-9-]+\*\*' \
  | sed 's/^- \*\*//; s/\*\*$//; s/^ *//; s/ *$//' | sort -u > /tmp/slugs-after.txt
diff /tmp/slugs-before.txt /tmp/slugs-after.txt
# Expected: empty diff (30 unique slugs).

# And every [spec](plans/done/<file>) link resolves:
awk '/^### Maintenance plans/,/^## Open Questions/' ai-docs/context.md \
  | grep -oE 'plans/done/[a-z0-9.-]+\.spec\.md' \
  | sort -u \
  | xargs -I {} test -f ai-docs/{}
echo "AC5 link resolution PASS"
# Expected: no error from xargs (all 30 specs exist).
```

### V6 — AC6: 13 numbered crate-level plans preserved

```bash
git show master:ai-docs/context.md \
  | awk '/^Crate-level plans:/,/^### Maintenance plans/' \
  | grep -cE '^[0-9]+\. '
# Expected before: 13
awk '/^Crate-level plans:/,/^### Maintenance plans/' ai-docs/context.md \
  | grep -cE '^[0-9]+\. '
# Expected after: 13
awk '/^Crate-level plans:/,/^### Maintenance plans/' ai-docs/context.md \
  | grep -cE '^[0-9]+\. .*✅'
# Expected: 13 (every numbered entry retains its ✅ marker).
```

### V7 — AC7: inbound anchor stable

```bash
grep -nE '^### Maintenance plans \(cross-cutting\)$' ai-docs/context.md
# Expected: exactly one match (in context.md).

grep -rn 'context.md#' .claude/ ai-docs/ AGENTS.md CLAUDE.md 2>/dev/null
# Expected: exactly one hit at ai-docs/plans/INDEX.md:115 → context.md#maintenance-plans-cross-cutting.
# That anchor resolves because the heading was preserved verbatim above.
```

### V8 — AC8: Rust toolchain pass (no .rs touched)

```bash
cargo fmt -- --check
cargo clippy --workspace -- -D warnings
cargo build
cargo test
# All four expected to pass. Trivially true since the PR touches only ai-docs/*.md + AGENTS.md.
```

### V9 — AC9: no new `learnings.md` entry this turn

```bash
git diff master -- ai-docs/learnings.md | grep -E '^\+### ' && echo "AC9 FAIL: new entry detected" || echo "AC9 PASS"
# Expected: AC9 PASS (no new "### YYYY-MM-DD" entry header in the diff).
```

### V10 — AC10: Propagation Rule

The slug list is derived from the **master baseline** (the pre-PR `context.md` § Maintenance plans block) so a future addition of a new maintenance plan does not silently drift this verification. The current set as of design time is the 30 slugs proposed in the "Proposed one-liners (30 entries)" list above (`cleanup-progress-issue-derive`, `shrink-agents-md`, `project-docs`, `macro-object-bench`, `criterion-benchmarks`, `codegen-inline-concrete-trait-impls`, `codegen-simple-marker`, `code-style-extraction`, `generic-fn-split`, `recursive-inline-annotations`, `tracing-spans`, `signal-to-signal`, `per-thread-event-loops`, `doc-convention`, `thiserror-migration`, `tracing-itertools`, `signal-emit-rename`, `receiver-guard-auto`, `connect-queued-codegen`, `enumflags2-property-flags`, `objectbase-debug-rename-factory`, `code-quality-cleanup`, `docs-and-facade`, `public-api-docs`, `lookup-perf`, `inline-simple-fns`, `examples-crate`, `signals-blocked`, `auto-connection`, `geometry-events`) — kept here as a human-readable reference; the script below is authoritative.

```bash
# Every quoted phrase / slug / anchor moved out of context.md was searched for elsewhere:
# (a) Slugs — derived from the master baseline so the verification cannot silently drift
#     as new maintenance plans land between design draft and implementation. The slug
#     regex is bullet-anchored (^- **slug**) so that mid-prose **emphasis** runs cannot
#     leak in as false positives (verified hazard: the cleanup-progress-issue-derive
#     entry on master contains "the **issue** body", which an unanchored regex would
#     capture as a spurious slug `issue`, producing 232 stale hits across the repo).
awk '/^### Maintenance plans/,/^## Open Questions/' <(git show master:ai-docs/context.md) \
  | grep -oE '^- \*\*[a-z][a-z0-9-]+\*\*' \
  | sed 's/^- \*\*//; s/\*\*$//' \
  | sort -u \
  | while read slug; do
      grep -rn "$slug" .claude/agents/ .claude/skills/ ai-docs/ AGENTS.md CLAUDE.md 2>/dev/null \
        | grep -vE '^ai-docs/(context\.md|key-decisions\.md|plans/)' || true
    done
# Expected: no hit pointing at a stale reference outside context.md / key-decisions.md / plans/.

# (b) Anchor: only one inbound anchor exists (INDEX.md:115); covered by V7.
```

**Audit of other slug-extraction sites in the verification suite.** Reviewed V1–V10 for the same `grep -oE '\*\*…\*\*'` hazard:
- **V1** (`wc -c`): byte count, no slug extraction. Unaffected.
- **V2** (H2 sequence): `grep -nE '^## '` — heading-anchored, no `**…**`. Unaffected.
- **V3** (crate-row set): `sed -E 's/`(quartzite[a-z-]*)`.*/\1/'` — backtick-anchored on `\`crate\``, no `**…**`. Unaffected.
- **V4** (KDD row diff): `awk -F'|' '{… print $2}'` — cell-positional extraction from the table-row pipe-delimited form; Question-cell text does not contain `**…**` emphasis in the current corpus, and the awk approach would not be tricked by mid-prose emphasis even if it did. Unaffected.
- **V5**, **V10**: fixed above.
- **V6** (numbered plans): `grep -cE '^[0-9]+\. '` — numeric-anchored. Unaffected.
- **V7**, **V8**, **V9**: no slug regex.

**Fixtures needed.** None — the verification suite is shell-only, runs against the working tree + `git show master:ai-docs/context.md` (the baseline). The design author commits to running V1–V10 as the final action of Task 6 before the `git push` that creates the PR.

## Open questions

_None._ Every design-time choice the spec deferred to the design agent is recorded above with rationale:
- Per-row K/E partition: enumerated.
- Maintenance-plans one-liner template + 30 proposed summaries: enumerated.
- Extraction destination: single new page `ai-docs/key-decisions.md` (no `maintenance-plans.md`).
- Anchor-stability story: heading preserved verbatim; new outbound anchors point to verbatim-question H2 headings; one-shot grep documented.
- Decomposition (6 atomic tasks): ordered so every intermediate state is self-consistent.
- Verification (V1–V10): one shell-runnable check per AC.
