# Design: Doc-test fence inline comments

**Issue:** #517 (follow-up to #336 / PR #516)
**Date:** 2026-05-21
**Spec:** `ai-docs/plans/2026-05-21-doctest-fence-inline-comments.spec.md`

## Approach

The task has two halves: (a) a one-shot semantic audit of every inline `// …` line that reaches the rendered docs.rs surface, classified per Scope §3 (keep / rewrite / drop), and (b) a permanent codification + reviewer-prompt fan-out so the same gate is re-applied on every future diff. The audit pass is mechanically discoverable (two `rg`/walker passes) but the classification itself is semantic — no regex can distinguish "explains the example" from "assumes repo-internal context". This is the rationale the spec captures for *no automated regression gate*; the design carries that constraint forward unchanged.

**Chosen solution.** Run the live two-pass sweep at **design pickup** (already done — see *Live sweep delta from spec* below), pin the canonical hit set in this design doc (the per-hit table is the canonical AC2 artefact; owner approval fires at Step 7 design-review GO + user acceptance, BEFORE Step 8 code edits — see *AC2 owner-approval gate* note immediately after the table for full rationale), do a defensive re-verify at Step 8 entry (subtask 1) to catch any commit-base drift, then apply the verdicts file-by-file grouped by crate (preserves Rust idioms — one PR-shaped chunk per topical area), then codify the rule into `ai-docs/doc-convention.md` and the four review files. Verification combines `cargo test --workspace` (AC4), `cargo doc` (AC5), `cargo clippy` (AC6), a 5–10-page rendered sample (AC3), the `wc -c` post-edit budget check (AC10), and the Propagation Rule grep (AC11).

**Codification placement (AC8 → Open Question 1).** Extend the existing `## Self-sufficiency: no repo-internal references` subsection (anchor `#self-sufficiency-no-repo-internal-references`) with a **new Family C entry** parallel to Family A / Family B, NOT a sibling subsection. Rationale: Families A and B already taxonomise the same semantic concept (repo-internal coupling that fails the docs.rs reader); inline-`//` contributor comments are a third family of that same coupling, surfaced through code-fence syntax rather than prose. A sibling subsection would split the rule into two anchors and force review files to link both. Single-family extension keeps one anchor, one regex-audit block in CI's prose-side gate, and one entry point for reviewers. Non-conforming + conforming example come from the design's per-hit table (drawn from `quartzite-style/src/paint_widget.rs:20` rewrite — the most-cited contributor-perspective case).

**Reviewer-prompt placement (AC9 → Open Question 2).** The four review files already host a prose-side rule entry for the existing `Self-sufficiency` audit (`code-review/SKILL.md` row 162; `review-findings.md` line 101; `self-review.md` line 122; `design-review.md` line 35). The new Family C rule is a semantic prompt (no regex). Wording is uniform per file shape:

- `code-review/SKILL.md` (Gate checklist table) — add a second row in the `| Step 4 |` block, parallel to the existing Pattern A / Pattern B row, pointing at the `Self-sufficiency` anchor and naming the §3 classification rule for inline `//` lines inside fences.
- `review-findings.md` (§6 *Documentation conventions* bulleted list) — append a bullet beneath the existing `No repo-internal references` bullet, naming the same anchor and the §3 rule.
- `self-review.md` (§5 *Documentation conventions* bulleted list, mirroring `review-findings.md`) — append the matching bullet with the diff-touched-only scoping language the existing bullet uses.
- `design-review.md` (§3 sub-list of checks) — append a check item parallel to the existing planned-doc-comment-text check, applying the §3 rule to inline `//` lines in proposed code-fence content inside the design document.

The rule reference (`ai-docs/doc-convention.md` § Self-sufficiency anchor) is uniform across the four files; wording is tailored to each file's surrounding structure.

**Rejected alternatives.**

1. **Mechanical regression gate via regex** — rejected; spec explicitly excludes it (semantic decision, regex over- or under-matches). Already deferred in spec §Deferred row 1.
2. **One big PR-touch in a single subtask** — rejected. The per-hit table is the AC2 owner-approval artefact and fires its gate BEFORE Step 8 code edits (at Step 7 design-review GO + user acceptance); collapsing audit + edits into one step would hide the rationale that the gate validates.
3. **Crate-by-crate splitting of the edits into many smaller groups** — considered. With 69 in-scope hits spanning 18 files, a single edit-subtask per crate fragments the work into 8 groups and exceeds the 7-task ceiling. The chosen decomposition consolidates by surface (paint-API + core + style/dispatch + runtime + renderer + macros + lib.rs) and stays at 6 subtasks within the 7-task cap.
4. **New sibling subsection in `doc-convention.md` instead of Family C** — rejected per Open Question 1 above.
5. **A new dedicated rule section in each review file separate from the Pattern A/B entry** — rejected; would split the cross-file anchor and complicate the Propagation Rule grep in AC11.

**Live sweep delta from spec.** Spec claimed 65 Pass-A hits (workspace total) + 6 Pass-B = 71. Re-run at design pickup confirms 65 raw Pass-A hits, but **2 of them live in `quartzite-test-helpers/src/lib.rs`** (`[lib] doc = false` — excluded from rustdoc per the spec's own exclusion list); 0 hits in `tests/` / `benches/` / `#[cfg(test)]` regions. In-scope Pass A is therefore **63**, and combined in-scope is **63 + 6 = 69**. Pass-B raw-string sites unchanged at one file (`src/lib.rs`). Live sweep verified to match spec time apart from the test-helpers exclusion accounting. The per-hit table below covers all 69.

**Per-hit classification table (AC2 — full audit; owner-approval gate fires at Step 7 design-review GO + user acceptance, before Step 8 code edits — see *AC2 owner-approval gate* note immediately following the table).**

Verdict legend: **K** = keep (rule (i) — useful to a docs.rs reader); **R** = rewrite (rule (ii) — repo-internal phrasing, but the labelled content has value); **D** = drop (rule (ii) — line conveys nothing the surrounding code does not already say).

Fence-kind column: `rust` (default, compiled); `no_run` (compiled, not executed); `ignore` (not compiled); `text` (raw text). All four fence kinds render verbatim on docs.rs.

### Pass A — line-prefixed `///` / `//!` doc comments (63 in-scope hits)

#### `quartzite-paint-api` (4 hits)

| file:line | fence | original | verdict | rationale | proposed rewrite |
|---|---|---|---|---|---|
| `quartzite-paint-api/src/color.rs:180` | `rust` | `// 25 % blend toward sky-blue: r stays 0.75, g moves from 1.0 to 0.875` | K | labels the computation the assertions below verify — useful for a reader scanning the example | — |
| `quartzite-paint-api/src/brush.rs:36` | `rust` | `// BrushKind is Clone` | K | states a Rust trait-impl fact verified by the next two lines; helpful to a reader of `BrushKind`'s doc-page | — |
| `quartzite-paint-api/src/image.rs:55` | `rust` | `// Valid 2x2 RGBA buffer.` | K | labels an input case (`ok`) in a `try_new` example — directly helpful | — |
| `quartzite-paint-api/src/image.rs:59` | `rust` | `// Wrong length is rejected.` | K | labels the error case (`err`) — directly helpful | — |

#### `quartzite-core` (8 hits)

| file:line | fence | original | verdict | rationale | proposed rewrite |
|---|---|---|---|---|---|
| `quartzite-core/src/snapshot.rs:43` | `ignore` | `//     // ...` | K | body-placeholder in a `typetag::serde` impl skeleton; widely-recognised Rust convention for "omitted impl body"; useful to the reader | — |
| `quartzite-core/src/object_base.rs:29` | `no_run` | `// Debug output includes id, name, and (on std) thread_id.` | K | labels Rust-ecosystem behaviour (`Debug` impl shape); helpful for a reader reading `ObjectBase`'s page | — |
| `quartzite-core/src/traits.rs:49` | `no_run` | `// To rename an object, use ObjectTree::rename so the name index stays consistent.` | R | mentions `ObjectTree::rename` as advice in `AsObject::object_base_mut`'s `# Examples`; the API hint belongs in prose above the fence, not buried in a code comment that doesn't compile a call | move the advice up into the prose paragraph (re-wording: "Note: prefer [`ObjectTree::rename`] for renames so the name index stays consistent."); drop the in-fence line |
| `quartzite-core/src/traits.rs:65` | `no_run` | `//     // use concrete` | K | placeholder for "use the downcast" in `as_any` example; widely-recognised "elided body" convention | — |
| `quartzite-core/src/traits.rs:81` | `no_run` | `//     // mutate concrete` | K | matching mutable variant; same justification as :65 | — |
| `quartzite-core/src/traits.rs:240` | `no_run` | `// obj is any type that implements AsObject` | K | labels the input variable in an `ObjectExt::id()` example, conveying a Rust trait-bound fact rather than a repo-internal one | — |
| `quartzite-core/src/traits.rs:298` | `no_run` | `//     // use concrete` | K | placeholder for the downcast-ref body; matches :65 | — |
| `quartzite-core/src/traits.rs:316` | `no_run` | `//     // mutate concrete` | K | matching `downcast_mut`; matches :81 | — |

#### `quartzite-core/src/signal.rs` (3 hits)

| file:line | fence | original | verdict | rationale | proposed rewrite |
|---|---|---|---|---|---|
| `quartzite-core/src/signal.rs:180` | `no_run` | `// Normally called by Application::new(); shown here for illustration.` | R | mentions `Application::new()` (a `quartzite-runtime` type) as the production caller — contributor-perspective. `quartzite-core` does **not** depend on `quartzite-runtime`, so no intra-doc link is available from this site; a docs.rs reader has nothing to follow. Useful content is "the dispatcher slot is normally set up automatically by the higher-level runtime", which can be stated without naming any specific cross-crate type. | replace with `// The dispatcher slot can be set once per process; the higher-level runtime does this automatically.` (no cross-crate type name; no orphan intra-doc reference) |
| `quartzite-core/src/signal.rs:181` | `no_run` | `// set_queued_dispatcher(Arc::new(my_dispatcher));` | K | commented-out call sketches the API shape — useful illustration; the line cannot run because `my_dispatcher` is a placeholder, hence the comment-out | — |
| `quartzite-core/src/signal.rs:512` | `rust` | `// Weak::new() stands in for a real receiver guard here.` | K | labels the doctest fixture choice — directly helpful for a reader reading the example | — |

#### `quartzite-style-dispatch` (8 hits, 5 logical edits)

| file:line | fence | original | verdict | rationale | proposed rewrite |
|---|---|---|---|---|---|
| `quartzite-style-dispatch/src/lib.rs:19` | `rust` | `// 1. Install a style (once per process, before painting).` | K | step label in a numbered Quick-start tutorial — directly helpful and structural | — |
| `quartzite-style-dispatch/src/lib.rs:22` | `rust` | `// 2. Build a widget tree.` | K | step label | — |
| `quartzite-style-dispatch/src/lib.rs:33` | `rust` | `// 3. Implement WidgetResolver over your backing store.` | K | step label | — |
| `quartzite-style-dispatch/src/lib.rs:47` | `rust` | `// 4. Call dispatch_paint from inside WidgetRoot::paint (or a test).` | R | step label and useful structurally, but "inside `WidgetRoot::paint` (or a test)" leans on the runtime contract; the step is just "call dispatch_paint" — the call-site choice is up to the reader | rewrite as `// 4. Call dispatch_paint with the root id, resolver, painter, and palette.` |
| `quartzite-style-dispatch/src/dispatch.rs:119` | `rust` | `// In a WidgetRoot::paint implementation:` | R | scene-setting that names a repo-internal trait (`WidgetRoot::paint`); the three pseudo-code lines that follow only restate the call. Collapse the four-line block to a single self-contained comment that names the API shape and **keeps the example self-explanatory** — the fence is a default `rust` doctest whose `use` block at :109 imports `dispatch_paint`, and the visible body above :119 only defines `MapResolver`; round-1 design-review flagged that dropping every readable trace of the imported name leaves the example opaque (and may also raise `unused_imports` in doctest builds, though comment lines are not code references; if the lint fires the fallback per Risks section is to drop the import too). | rewrite to `// Then call dispatch_paint(root, &resolver, painter, palette).` (single visible commented-out call — names the function the import brings in, names the four parameters by role, no repo-internal trait reference) |
| `quartzite-style-dispatch/src/dispatch.rs:120` | `rust` | `// fn paint(&self, painter: &mut dyn quartzite_paint_api::Painter) {` | D | merged into the :119 rewrite — the rewritten one-liner subsumes the pseudo-`fn paint` wrapper | — |
| `quartzite-style-dispatch/src/dispatch.rs:121` | `rust` | `// dispatch_paint(self.root_id, &self.resolver, painter, &self.palette);` | D | merged into the :119 rewrite — same content, expressed by-role rather than by-`self`-field | — |
| `quartzite-style-dispatch/src/dispatch.rs:122` | `rust` | `// }` | D | merged into the :119 rewrite — pseudo-`fn paint` brace closure no longer needed | — |

(Count above is 8 visible rows but :119 / :120 / :121 / :122 are one logical edit — the four-line block in `dispatch.rs` collapses to a single rewritten line at :119 that keeps the example self-explanatory (the rewritten one-liner names `dispatch_paint`, the imported function, so a docs.rs reader sees what the import is for). Pass-A in-scope total still 63.)

#### `quartzite-style` (12 hits)

| file:line | fence | original | verdict | rationale | proposed rewrite |
|---|---|---|---|---|---|
| `quartzite-style/src/style.rs:42` | `ignore` | `//         // draw MyWidget` | K | placeholder body in a `Paint<MyWidget>::paint` impl skeleton; widely-recognised "elided body" convention | — |
| `quartzite-style/src/style.rs:104` | `rust` | `// Trait is object-safe — boxing through \`dyn Style\` compiles.` | K | labels the assertion immediately below (`let _boxed: Box<dyn Style>`); states a Rust object-safety fact useful for the reader | — |
| `quartzite-style/src/style.rs:121` | `text` | `///         // … other built-ins …` | K | placeholder inside a `text`-fenced pseudo-`match`; ellipsis-comment is a widely-recognised "more variants omitted" convention | — |
| `quartzite-style/src/default_style.rs:55` | `no_run` | `// DefaultStyle implements Style — it can be boxed as a trait object.` | K | states a Rust trait-impl + object-safety fact that the line below (`Box::new(DefaultStyle)`) verifies — labels the assertion | — |
| `quartzite-style/src/registry.rs:105` | `no_run` | `// Before any set_style: returns None.` | K | labels the temporal state of the doctest fixture (no prior `set_style` call) — directly helpful | — |
| `quartzite-style/src/paint_widget.rs:20` | `text` | `// Inside DefaultStyle::draw_widget:` | R | scene-setter mentioning `DefaultStyle::draw_widget` (a sibling-crate type) — the prose sentence two lines up already says "Inside `Style::draw_widget`" via an intra-doc link; the fence label is contributor-shorthand for that. Replace with a fence-internal heading-style comment that names the API shape, not the concrete type. | rewrite as `// In a Style::draw_widget body:` (links the trait method shown above the fence, no crate-specific type) |
| `quartzite-style/src/paint_widget.rs:39` | `rust` | `//         // draw Button here` | K | placeholder body in a `Paint<Button>::paint` skeleton; widely-recognised convention | — |
| `quartzite-style/src/paint_widget.rs:43` | `rust` | `// Paint<W> is object-safe — &dyn Paint<Button> compiles.` | K | labels the assertion below (`&dyn Paint<Button>`) — states a Rust object-safety fact | — |
| `quartzite-style/src/paint_widget.rs:59` | `ignore` | `// A widget defined outside quartzite-widgets.  No #[widget_view] attribute` | R | mentions `quartzite-widgets` and `#[widget_view]` — repo-internal architecture. The example demonstrates the *open-set extensibility* pattern; the useful content is "a user-defined widget type with no `widget_view` variant gets `WidgetView::Other(self)`" | rewrite as `// A user-defined widget without a registered WidgetView variant defaults to WidgetView::Other(self).` (drops the crate-internal cite + macro name) |
| `quartzite-style/src/paint_widget.rs:60` | `ignore` | `// → widget_view() returns WidgetView::Other(self).` | D | merged into the :59 rewrite — that single sentence now covers both lines | — |
| `quartzite-style/src/paint_widget.rs:63` | `ignore` | `// (AsWidget impl omitted for brevity — use #[derive(Extend)] in practice)` | R | "in practice we use `#[derive(Extend)]`" is contributor-convention phrasing; the *content* the reader needs is "the `AsWidget` impl is omitted for example brevity". `#[derive(Extend)]` is a workspace-internal macro mention | rewrite as `// (AsWidget impl elided for example brevity.)` |
| `quartzite-style/src/paint_widget.rs:69` | `rust` | `//         // draw MyWidget here` | K | placeholder body in `paint` impl — convention | — |

#### `quartzite-widgets` (2 hits)

| file:line | fence | original | verdict | rationale | proposed rewrite |
|---|---|---|---|---|---|
| `quartzite-widgets/src/widget_ext.rs:479` | `rust` | `// The default impl sets pressed=true; the input-plumbing pass calls this.` | R | "the input-plumbing pass calls this" cites the contributor model of how the framework drives the widget — repo-internal phrasing. The useful content for a reader of `WidgetExt::on_mouse_press` is "the default impl sets pressed=true". | rewrite as `// The default impl sets pressed=true via set_pressed().` |
| `quartzite-widgets/src/widget_ext.rs:480` | `rust` | `// Directly verify the accessor:` | K | labels the verification approach — useful for a reader scanning the doctest body | — |

#### `quartzite-runtime` (15 hits)

| file:line | fence | original | verdict | rationale | proposed rewrite |
|---|---|---|---|---|---|
| `quartzite-runtime/src/lib.rs:16` | `rust,no_run` | `// … create Application, run event loop` | K | placeholder body after the `env_logger::init();` setup; widely-recognised "elided remainder" convention | — |
| `quartzite-runtime/src/application.rs:160` | `no_run` | `// post a quit event immediately so the loop exits right away in tests` | R | "in tests" is contributor-perspective — a docs.rs reader isn't writing tests, they're learning the API. The substantive content is "post a quit event before `exec()` so the loop exits". | rewrite as `// Post a quit before exec() so the loop exits immediately.` |
| `quartzite-runtime/src/timer.rs:160` | `no_run` | `// … later …` | K | placeholder between `start` and `stop` calls — convention | — |
| `quartzite-runtime/src/snapshot/tree.rs:39` | `no_run` | `// let (tree, root_id): (ObjectTree, ObjectId) = ...;` | K | sketches the precondition of a `capture_tree` call — useful for a reader who needs to know the input types; commented-out because the placeholder `...` cannot compile | — |
| `quartzite-runtime/src/snapshot/tree.rs:40` | `no_run` | `// let snap = capture_tree(&tree, root_id).unwrap();` | K | sketches the call — useful API shape | — |
| `quartzite-runtime/src/snapshot/tree.rs:98` | `no_run` | `// let snap: TreeSnapshot = ...;` | K | sketches the precondition of `restore_tree` — useful | — |
| `quartzite-runtime/src/snapshot/tree.rs:99` | `no_run` | `// let (tree, root_id) = restore_tree(&snap).unwrap();` | K | sketches the call | — |
| `quartzite-runtime/src/snapshot/object.rs:29` | `no_run` | `// let obj: Box<dyn quartzite_core::Object> = ...;` | K | sketches the precondition of `capture_object` | — |
| `quartzite-runtime/src/snapshot/object.rs:30` | `no_run` | `// let snap = capture_object(obj.as_ref()).unwrap();` | K | sketches the call | — |
| `quartzite-runtime/src/snapshot/object.rs:82` | `no_run` | `// let snap: ObjectSnapshot = ...;` | K | sketches the precondition | — |
| `quartzite-runtime/src/snapshot/object.rs:83` | `no_run` | `// let obj = restore_object(&snap).unwrap();` | K | sketches the call | — |
| `quartzite-runtime/src/object_tree.rs:71` | `no_run` | `// tree.insert(Box::new(my_object), None);` | K | sketches an `insert` call — useful API shape | — |
| `quartzite-runtime/src/object_tree.rs:172` | `no_run` | `//     // use parent_id` | K | placeholder body inside an `if let Some(parent_id) = …` block; widely-recognised convention | — |
| `quartzite-runtime/src/object_tree.rs:279` | `no_run` | `// ids[0] is the shallowest match; ids.last() is the deepest.` | K | states a behaviour fact about the returned `Vec<ObjectId>` ordering — directly useful | — |
| `quartzite-runtime/src/factory.rs:121` | `no_run` | `// factory.register("MyObject", || Box::new(MyObject::new()));` | K | sketches the `register` call — useful API shape; commented-out because `MyObject` is a placeholder | — |

(All 15 rows above are distinct in-scope hits — no sub-clustering; the subheading count matches the row count.)

#### `quartzite-events` (2 hits)

| file:line | fence | original | verdict | rationale | proposed rewrite |
|---|---|---|---|---|---|
| `quartzite-events/src/mouse.rs:134` | `rust` | `// event_button names only the button whose state just changed.` | K | labels the assertion below — useful behavioural fact | — |
| `quartzite-events/src/mouse.rs:138` | `rust` | `// buttons_state holds every button currently pressed.` | K | labels the assertion below — useful | — |

#### `quartzite-renderer` (6 hits)

| file:line | fence | original | verdict | rationale | proposed rewrite |
|---|---|---|---|---|---|
| `quartzite-renderer/src/window_id.rs:21` | `rust` | `// WindowId is Copy, Clone, Eq, Hash — verify those bounds compile.` | K | labels the assert-trait-bounds doctest pattern; states Rust trait facts | — |
| `quartzite-renderer/src/application_builder.rs:21` | `no_run` | `// In another thread or deferred callback:` | K | scene-setter for the `proxy.send_event(...)` call — useful context for a reader | — |
| `quartzite-renderer/src/application.rs:82` | `no_run` | `// From another thread:` | K | matching scene-setter for `event_proxy` example | — |
| `quartzite-renderer/src/vello_painter.rs:38` | `no_run` | `// pass to widget.paint(&mut painter)` | R | mentions "widget.paint" as a follow-up call — `widget` is undefined in the example. Useful content is "use `painter` as `&mut dyn Painter`". | rewrite as `// Use painter as a &mut dyn Painter argument.` |
| `quartzite-renderer/src/render_harness.rs:219` | `no_run` | `// 64x64 RGBA8 offscreen target.` | K | labels the `RenderHarnessBuilder::new(64, 64)` constructor inputs — useful | — |
| `quartzite-renderer/src/windowed_app_handler.rs:25` | `no_run` | `//         // create initial windows here` | K | placeholder body inside an `on_start` impl skeleton — convention | — |

#### `quartzite-macros` (2 hits)

| file:line | fence | original | verdict | rationale | proposed rewrite |
|---|---|---|---|---|---|
| `quartzite-macros/src/lib.rs:48` | `no_run` | `// Root of the hierarchy: generates AsWidget + AsObject delegation.` | K | labels the `#[root]` example struct — useful since the line below shows `#[derive(Extend) #[root]` | — |
| `quartzite-macros/src/lib.rs:56` | `no_run` | `// Derived type: extends Widget, so AsButton, AsWidget, and AsObject are all available.` | K | labels the non-root example struct — useful since it explains the impl-fan-out the derive emits | — |

#### `quartzite` facade crate — `src/lib.rs` line-prefixed `//!` (1 hit; distinct from the Pass-B raw-string fence below)

| file:line | fence | original | verdict | rationale | proposed rewrite |
|---|---|---|---|---|---|
| `src/lib.rs:127` | `ignore` | `// Given a tree populated by the runtime:` | R | scene-setter naming the runtime layer as the actor — contributor-perspective ("the runtime" implies the contributor model of `Application` driving the tree). The useful content is "assuming `tree` is an `ObjectTree` with some objects in it". | rewrite as `// Given an ObjectTree containing named objects:` (drops the "by the runtime" coupling; states the precondition the example needs) |

### Pass B — `#[doc = ...]` / `#![doc = ...]` raw-string attribute bodies (6 in-scope hits)

All in `src/lib.rs`'s `#![cfg_attr(feature = "derive", doc = r#"..."#)]` Quickstart fence.

| file:line | fence | original | verdict | rationale | proposed rewrite |
|---|---|---|---|---|---|
| `src/lib.rs:44` | `no_run` (inside `cfg_attr`-gated Quickstart) | `// Explicit \`fn main\` keeps the type definitions at module level — the` | R | three-line contributor / rustdoc-internals reasoning about `#[derive]` codegen path resolution under rustdoc's implicit-main wrapper. A docs.rs reader does not need to know what the implicit-main wrapper is. The substantive content is "the derive macros emit paths assuming top-level type definitions, so the example uses an explicit `fn main`". | replace all three lines (:44 + :45 + :46) with a single one-liner: `// Explicit fn main keeps the derived types at module scope.` |
| `src/lib.rs:45` | `no_run` | `// \`#[derive]\` codegen emits paths that resolve relative to the containing` | D | merged into :44 rewrite | — |
| `src/lib.rs:46` | `no_run` | `// module, which would not resolve under rustdoc's implicit-main wrapper.` | D | merged into :44 rewrite | — |
| `src/lib.rs:54` | `no_run` | `// Connect a slot to the count_changed signal.` | K | step label for `c.count_changed.connect(...)` — directly useful for a docs.rs reader following the Quickstart | — |
| `src/lib.rs:57` | `no_run` | `// Writing the property emits count_changed automatically.` | K | labels observable behaviour — directly useful (states a property-system fact) | — |
| `src/lib.rs:60` | `no_run` | `// Invoke the slot dynamically through the reflection layer.` | K | step label — useful | — |

### Hit summary

- **Pass A in-scope total: 63** (65 raw − 2 test-helpers excluded under `[lib] doc = false`).
  - **Keep (48):** labels, assertions, placeholders, Rust-ecosystem facts, structural step labels.
  - **Rewrite (11):** `quartzite-core/src/traits.rs:49`, `quartzite-core/src/signal.rs:180`, `quartzite-style-dispatch/src/lib.rs:47`, `quartzite-style-dispatch/src/dispatch.rs:119`, `quartzite-style/src/paint_widget.rs:20`, `:59`, `:63`, `quartzite-widgets/src/widget_ext.rs:479`, `quartzite-runtime/src/application.rs:160`, `quartzite-renderer/src/vello_painter.rs:38`, `src/lib.rs:127`.
  - **Drop (4):** `quartzite-style-dispatch/src/dispatch.rs:120`, `:121`, `:122`, `quartzite-style/src/paint_widget.rs:60`.

- **Pass B in-scope total: 6** (all in `src/lib.rs`'s `#![cfg_attr(feature = "derive", doc = r#"..."#)]` Quickstart fence).
  - **Keep (3):** `src/lib.rs:54`, `:57`, `:60`.
  - **Rewrite (1):** `src/lib.rs:44` (collapses :45 + :46 into one line).
  - **Drop (2):** `src/lib.rs:45`, `src/lib.rs:46` (merged into the :44 rewrite).

- **Combined in-scope total: 69 hits — K = 51, R = 12, D = 6.**

**AC2 owner-approval gate — satisfied at design-approval time, NOT inside Step 8.** The per-hit table above is the canonical AC2 artefact. The live two-pass sweep was run at design pickup (see *Live sweep delta from spec* above); the table reflects current workspace state. Owner approval of this table is satisfied by:

1. Step 7 `design-review` issuing GO on this design doc, AND
2. The user accepting the GO (the standard `/task` Step 7 handoff to Step 8).

Once both conditions hold, the table is approved and no further in-Step-8 owner-approval pause is required. This placement keeps the AC2 gate inside the existing Step 6 → Step 7 → Step 8 transition (no novel mid-Step-8 user pause), and avoids creating a non-terminal handoff group of size 1 (which would itself violate `.claude/agents/design.md` § Rules → handoff-grouping (b) "non-terminal groups MUST be exactly 3").

If the Step 8 defensive re-verify (subtask 1, below) surfaces hits the design-time sweep missed, the Design Amendment recipe (`.claude/skills/task/reference.md` § Design Amendment recipe) re-runs Step 6 → Step 7 with the amended table — the new owner-approval gate is the re-approval of the amended design, not an ad-hoc in-Step-8 pause.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | **Defensive re-verify** of the design-time live sweep at Step 8 entry (Pass A `rg --type rust -n '^\s*(///\|//!)\s*//\s'` + Pass B walker per spec §1 recipe). The per-hit table is **already pinned and owner-approved in this design doc** (see *AC2 owner-approval gate* note above — the gate fired at Step 7 design-review GO + user acceptance, not here). If the re-verify finds **zero deltas** vs the design table, proceed to subtask 2. If it finds **any delta** (new hit not in the table, or a hit that has moved file/line because of an intervening commit), STOP and trigger the **Design Amendment** recipe (`.claude/skills/task/reference.md` § Design Amendment recipe) — re-run Step 6 → Step 7 on the amended design before any code edit. No mid-group user pause is required for the zero-delta path. | (read-only sweep; no file edits) | — |
| 2 | Apply Pass A rewrites in the **paint-API + style core** surface: `quartzite-core/src/traits.rs:49`, `quartzite-core/src/signal.rs:180`, `quartzite-style-dispatch/src/lib.rs:47`, `quartzite-style-dispatch/src/dispatch.rs:119–122` (rewrite + 3 drops), `quartzite-style/src/paint_widget.rs:20`, `:59`, `:60` (drop), `:63`. Run `cargo build` + `cargo test --doc` + `cargo doc --no-deps --workspace --all-features` after this subtask. | `quartzite-core/src/traits.rs`, `quartzite-core/src/signal.rs`, `quartzite-style-dispatch/src/lib.rs`, `quartzite-style-dispatch/src/dispatch.rs`, `quartzite-style/src/paint_widget.rs` | 1 |
| 3 | Apply Pass A rewrites in the **widgets + runtime + renderer** surface: `quartzite-widgets/src/widget_ext.rs:479`, `quartzite-runtime/src/application.rs:160`, `quartzite-renderer/src/vello_painter.rs:38`. Run the same three verification commands after the subtask. | `quartzite-widgets/src/widget_ext.rs`, `quartzite-runtime/src/application.rs`, `quartzite-renderer/src/vello_painter.rs` | 1 |
| 4 | Apply the `src/lib.rs` rewrites: (a) Pass-A rewrite at `src/lib.rs:127` (the `Object tree` `ignore` fence — rewrite per the table); (b) Pass-B rewrite collapsing `src/lib.rs:44-46` to one line. Verify `cargo test --doc` (the default-feature Quickstart fence is `no_run` but still compile-checked; `:127` is in an `ignore` fence so it is markdown-only) + `cargo doc`. | `src/lib.rs` | 1 |
| 5 | Codify the rule (AC8): extend `ai-docs/doc-convention.md` § *Self-sufficiency: no repo-internal references* with a **new Family C entry** parallel to Family A / B, naming inline `// …` lines inside `///` / `//!` code fences AND inside `#[doc = ...]` attribute bodies, plus the §3 classification rule. **Also update the section's lead-in sentence at line 22** — change "Two families of repo-internal references are forbidden:" to "Three families of repo-internal references are forbidden:" so the lead-in counts the new entry. Include the non-conforming + conforming `paint_widget.rs:20` rewrite as the worked example. Pre-edit `wc -c` = 30,323. Verify post-edit `wc -c` < 35,000 (AC10). | `ai-docs/doc-convention.md` | 2, 3, 4 |
| 6 | Propagate the rule to the four review files (AC9): one tailored line per file as described in *Approach → Reviewer-prompt placement*. After all four edits, run `grep -rn '<changed-keyword>' .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md` (AC11) and reconcile any further sites. Verify each file's post-edit `wc -c` < 35,000 (AC10). Verify final: `cargo test --workspace` (AC4), `cargo doc` (AC5), `cargo clippy --workspace --all-targets -- -D warnings` (AC6), `actionlint` on any touched workflow (none expected). Render-and-sample 5–10 affected rustdoc pages (AC3) and list the sampled URLs in the PR body. | `.claude/skills/code-review/SKILL.md`, `.claude/agents/review-findings.md`, `.claude/agents/self-review.md`, `.claude/agents/design-review.md` | 5 |

**M = 6.** Scope ≤ 7-task ceiling per `.claude/agents/design.md` § Rules.

## Handoff plan

`M = 6` → two groups, 3 + 3 (terminal group sized 3 — within the `1..=3` range; non-terminal Group A sized exactly 3 per the maximum-group-size rule).

- **Handoff into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). `/task` Step 8 makes this the entry into the first group per the every-group handoff contract.
- **Group A:** subtasks 1–3 — defensive re-verify of the design-time sweep (subtask 1; AC2 owner-approval gate already cleared at Step 7 GO + user acceptance, so no mid-group user pause is required on the zero-delta path), then apply Pass A rewrites across the paint-API + style core surface (subtask 2) and the widgets + runtime + renderer surface (subtask 3). Exactly 3 consecutive subtasks per the maximum group size of 3.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent /task resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — apply the Pass B rewrite (subtask 4), codify in `doc-convention.md` including the line-22 lead-in update from "Two families" to "Three families" (subtask 5), propagate to the four review files plus final verification + AC10 + AC11 + AC3 (subtask 6). Terminal group with 3 subtasks (within the `1..=3` range).

## Risks

- **Risk: a "Keep" verdict on a hit that a reviewer disagrees with.** Mitigation: the per-hit table is pinned in this design doc and owner-approval is gated by Step 7 design-review GO + user acceptance (see *AC2 owner-approval gate* note above). If a reviewer later flags a kept hit as too contributor-perspective in PR review, the change is one inline-comment rewrite — minimal blast radius. The 12 rewrites + 6 drops cover every line the issue body or its starting-recommendations called out.
- **Risk: an edit inside a compiled (`rust` / `no_run`) fence breaks `cargo test --doc`.** Mitigation: subtasks 2 / 3 / 4 explicitly run `cargo test --doc` after the edit subtask. All 12 rewrites are pure-comment edits (replace one comment line with another comment line); no rewrite alters code state. The `dispatch.rs:119–122` edit (1 rewrite at :119 + 3 drops at :120–:122) is the highest-blast-radius single block; design-review round 1 flagged the original four-line drop as risking an `unused_imports` doctest failure on a default `rust` fence (the `use` block at `dispatch.rs:109` imports `dispatch_paint`; the visible fence body above :119 defines `MapResolver` only and does not call `dispatch_paint`, so dropping every line that mentions the name removes the only readable evidence the import is in use). The chosen rewrite (option (c) from the round-1 review — single rewritten visible commented-out call at :119) avoids the risk by side-stepping it: the rewrite keeps the example self-explanatory (the reader sees the function the import names) and removes the contributor-perspective `WidgetRoot::paint` reference at the same time. Verification: `cargo test --doc -p quartzite-style-dispatch` after subtask 2; if a doctest-side `unused_imports` warning is raised regardless (a comment is not a code-reference from the compiler's standpoint), the fallback is to drop the `dispatch_paint` import from the `use` line at :109 too (option (a) from round-1) and re-run.
- **Risk: an edit inside an `ignore` / `text` fence renders as malformed markdown.** Mitigation: the rewrites in `paint_widget.rs:20` (`text`), `:59` (`ignore`), `:60` (`ignore`), `:63` (`ignore`), and `style.rs:121` (`text` — keep) are pure single-line-comment replacements; markdown structure is preserved. AC3's render-and-sample step explicitly inspects the affected pages.
- **Risk: `doc-convention.md` post-edit exceeds the 35,000-char early-warning threshold (AC10).** Mitigation: pre-edit baseline is 30,323 chars; the new Family C entry is one bullet (estimated 350–500 chars including the worked example). Post-edit ceiling = ~30,820, well below the 35k warning. Verified mechanically in subtask 5.
- **Risk: a fifth review file references the same `Self-sufficiency` anchor and is missed.** Mitigation: AC11's `grep -rn` over `.claude/agents/` + `.claude/skills/` + `AGENTS.md` + `ai-docs/agent-writing-style.md` is mandatory in subtask 6; any further site is reconciled in the same commit.
- **Risk: re-sweep at Step 8 implementation pickup surfaces new Pass-A or Pass-B hits not in the design-time table** (a commit between Step 7 GO and Step 8 entry could have added a new `///` / `//!` doc-comment or a new `#[doc = ...]` site). Mitigation: subtask 1 is a defensive re-verify that handles this — if the live result diverges from the table, the **Design Amendment recipe** (`.claude/skills/task/reference.md` § Design Amendment recipe) re-runs Step 6 → Step 7 on the amended (spec, design) pair before any code edit lands. The table remains canonical for the pre-amendment commit base; the amended table covers the new base. No mid-Step-8 ad-hoc user pause — the Design Amendment recipe is the single canonical re-approval gate.
- **Risk: a contributor adds new `#[doc = r#"..."#]` / `#![doc = ...]` sites between merge of this PR and the next sweep.** Mitigation: the Family C codification (AC8) names both syntaxes explicitly, and the reviewer prompt (AC9) directs reviewers to apply §3 to every inline `// …` line in any doc-delivery surface. No mechanical gate is added (spec deferral).

## Test Design

This task does not add Rust code; it edits doc-comment content + instruction files. No new `#[cfg(test)]` modules are added. Verification is via existing toolchain gates and a manual rendered-doc sample.

**Per-subtask verification:**

- **Subtask 2 / 3 / 4 (code-fence edits):**
  - Command: `cargo test --doc` (post-subtask)
  - Command: `cargo doc --no-deps --workspace --all-features` with `RUSTDOCFLAGS="-D warnings -D missing-docs"` (AC5)
  - Scenarios: every edited `rust` / `no_run` fence still compiles; every edited `ignore` / `text` fence still parses as markdown
  - Edge case: `dispatch.rs:119–122` collapse (1 rewrite at :119 + 3 drops at :120–:122) — verify the surrounding default `rust` fence still compiles under `cargo test --doc`; specifically verify the `dispatch_paint` import at `dispatch.rs:109` is not flagged `unused_imports` (the rewritten one-liner at :119 names the function in a comment but is not a code reference — if rustdoc-doctest defaults raise an `unused_imports` warning under `RUSTDOCFLAGS="-D warnings"`, drop the `dispatch_paint` import from the `use` line at :109 too as the round-1 (a) fallback). Also verify the fence still reads as a complete, parseable example (the `MapResolver` impl + the rewritten one-liner at :119 remain).
  - Edge case: `paint_widget.rs:60` drop — verify the merged :59 rewrite reads cleanly as a single comment without leaving an orphan arrow

- **Subtask 5 (`doc-convention.md` codification):**
  - Command: post-edit `wc -c ai-docs/doc-convention.md` < 35,000 (AC10)
  - Manual: render the section locally (Markdown preview) to confirm the Family C entry parallels Family A / B in shape
  - Manual: the worked non-conforming / conforming example uses a real line from the per-hit table (`paint_widget.rs:20` rewrite)

- **Subtask 6 (review-file propagation + final gates):**
  - Command: `wc -c` on each of the four review files post-edit; each < 35,000 (AC10)
  - Command: `grep -rn '<changed-keyword>' .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md` (AC11) — keyword = "Self-sufficiency" + "inline `//` in code fences"
  - Command: `cargo test --workspace` (AC4)
  - Command: `cargo doc --no-deps --workspace --all-features` with `RUSTDOCFLAGS="-D warnings -D missing-docs"` (AC5)
  - Command: `cargo clippy --workspace --all-targets -- -D warnings` (AC6)
  - Manual (AC3): 5–10 rendered doc pages — recommended sample (weighted toward rewrites):
    1. `quartzite_style/paint_widget/trait.Paint.html` — covers the 4 `paint_widget.rs` edits (largest concentration of rewrites/drops, `ignore` + `text` fences).
    2. `quartzite_style_dispatch/fn.dispatch_paint.html` — covers the `dispatch.rs:119–122` collapse (1 rewrite at :119 + 3 drops); confirm the fence renders the single rewritten commented-out call and reads cleanly.
    3. `quartzite_style_dispatch/index.html` (crate-root Quick-start) — covers `lib.rs:47` rewrite.
    4. `quartzite_core/trait.ObjectExt.html` (or `quartzite_core/traits/trait.ObjectExt.html` — actual path verified at render time) — covers the `traits.rs:49` rewrite + prose move.
    5. `quartzite_core/signal/fn.set_queued_dispatcher.html` — covers `signal.rs:180` rewrite.
    6. `quartzite_widgets/trait.WidgetExt.html` (`on_mouse_press` doc) — covers `widget_ext.rs:479` rewrite.
    7. `quartzite_runtime/struct.Application.html#method.exec` — covers `application.rs:160` rewrite.
    8. `quartzite_renderer/struct.VelloPainter.html` — covers `vello_painter.rs:38` rewrite.
    9. `quartzite/index.html` (facade crate root) — covers Pass-B `src/lib.rs:44-46` rewrite (Quickstart fence).
    10. `quartzite_style/struct.DefaultStyle.html` — spot-check of an unchanged Keep verdict (`default_style.rs:55`).
  - The sampled URLs/paths are listed in the PR body or a checklist comment so reviewers can re-verify (AC3).

## Open questions

Both spec-listed open questions are decided in this design; carrying them forward as resolved decisions (no design-blocker remaining):

- **Q1 (AC8 placement — Family C entry vs sibling subsection).** **Decision: new Family C entry inside the existing `## Self-sufficiency` subsection (single anchor).** Rationale: keeps one anchor for all four review files to reference; same regex audit block in the existing prose-side gate (which the present codification does not extend mechanically — semantic-only — but the section locality is the same); reviewers reading the rule encounter all three families in one read. The non-conforming + conforming example is drawn from the design's per-hit table (`paint_widget.rs:20` rewrite — most-cited contributor-perspective line). See *Approach → Codification placement*.

- **Q2 (AC9 wording per review file).** **Decision: one tailored line per file, parallel in placement to each file's existing Pattern A / B entry, with uniform rule-reference (`ai-docs/doc-convention.md` § Self-sufficiency anchor).** Per-file placement and wording:
  - `code-review/SKILL.md` line ~162 (Gate checklist table, Step 4 row block): add `| Step 4 | inline \`//\` comments inside doc-comment code fences in changed published-surface files pass the Family C §3 classification rule from [doc-convention](../../../ai-docs/doc-convention.md#self-sufficiency-no-repo-internal-references) (the standard \`review-findings.md\` §6 finding fires when violated) |`
  - `review-findings.md` line ~101 (after existing `No repo-internal references in doc-comments` bullet): add bullet `**No repo-internal inline \`//\` comments inside doc-comment code fences** ([doc-convention.md → Self-sufficiency: no repo-internal references → Family C](../../ai-docs/doc-convention.md#self-sufficiency-no-repo-internal-references)). For every inline \`//\` line inside a \`///\` / \`//!\` code fence or \`#[doc = ...]\` attribute body in this diff (workspace \`src/**\` excluding \`tests/\`, \`benches/\`, \`quartzite-test-helpers/src/**\`, and \`#[cfg(test)]\` regions), apply the §3 classification rule (rule (i): useful to a docs.rs reader → keep; rule (ii): assumes repo-internal architecture / contributor convention → rewrite or drop). Any non-test rule-(ii) match → \`major\`.`
  - `self-review.md` line ~122 (after existing `No repo-internal references in doc-comments` bullet): add the matching bullet with the diff-touched-only scoping language the existing self-review bullet uses (parallels the `review-findings.md` wording above with `For every \`///\` / \`//!\` / \`#[doc = ...]\` line added or modified by this diff`).
  - `design-review.md` line ~35 (in the §3 checks sub-list, after existing `No repo-internal references in planned doc-comment text` check): add `**No repo-internal inline \`//\` comments inside planned doc-comment fence content** — when the design document contains inline rustdoc snippets with code fences, apply the Family C §3 rule from [doc-convention.md § Self-sufficiency](../../ai-docs/doc-convention.md#self-sufficiency-no-repo-internal-references) to every inline \`//\` line inside those proposed fences. Any rule-(ii) match in planned fence content = \`major\`.`

  Both Q1 and Q2 ship as-decided; final wording may be tightened during implementation as long as the anchor link and the §3-rule reference stay uniform.

## Spec amendments

None. The live sweep at design pickup confirmed the spec's hit counts (modulo the 2 test-helpers Pass-A hits which the spec's exclusion list already excludes; the design notes the in-scope total of 63 + 6 = 69 vs the spec's pre-exclusion total of 65 + 6 = 71). No structural amendment to the spec is needed.
