# Doc-test fence inline comments

**Source:** issue #517 (follow-up to #336 / PR #516; explicit scope reconsideration of #336's *Out of scope* item 7 "Doc-test code fences").
**Date:** 2026-05-21
**Tracked in:** #517

## Scope

1. Sweep every inline `// …` comment that reaches the published rustdoc surface (workspace crate library targets — same surface definition as #336: `src/**` excluding `tests/`, `benches/`, `#[cfg(test)]` regions, and `quartzite-test-helpers/src/**` which is already excluded via `[lib] doc = false`). The sweep has **two passes** — both deliver inline `// …` lines into the rendered docs.rs surface and both are in scope:

   **Pass A — line-prefixed doc comments (`///` / `//!`).** Pattern:

   ```bash
   rg --type rust -n '^\s*(///|//!)\s*//\s'
   ```

   Matches inline `// …` lines that sit directly inside `///`-fenced or `//!`-fenced rustdoc blocks. As of spec time this returns **65 hits** across the workspace.

   **Pass B — raw-string / string-literal doc attributes (`#[doc = r#"..."#]`, `#[doc = "..."]`, `#![doc = ...]`).** As of spec time only one file in the workspace uses this syntax — verified by `rg --type rust -ln 'doc\s*=\s*r#'` returning a single hit: `src/lib.rs`. Inside `src/lib.rs`'s `#![cfg_attr(feature = "derive", doc = r#"..."#)]` Quickstart fence, the following inline `// …` lines render to docs.rs:

   - `src/lib.rs:44-46` — three-line "Explicit `fn main` ..." contributor-perspective comment about `#[derive]` codegen and rustdoc's implicit-main wrapper.
   - `src/lib.rs:54` — `// Connect a slot to the count_changed signal.`
   - `src/lib.rs:57` — `// Writing the property emits count_changed automatically.`
   - `src/lib.rs:60` — `// Invoke the slot dynamically through the reflection layer.`

   That's **6 Pass-B hits** as of spec time. **Combined candidate set: 65 + 6 = 71.** The design agent re-runs both passes at implementation pickup and treats the live result as the canonical surface; if any new `#[doc = r#"..."#]` / `#[doc = "..."]` / `#![doc = ...]` site has landed between spec time and pickup, Pass B must re-walk it.

   **Pass B recipe.** Pass B has no single-line `rg` form because the inline `//` lines do not begin with `///` / `//!`; they live in the body of an attribute string. The walker is:

   1. `rg --type rust -ln 'doc\s*=\s*r#'` to enumerate files containing raw-string doc attributes (extend the pattern to `doc\s*=\s*"` if non-raw string-literal `#[doc = "..."]` sites land later).
   2. For each hit file, locate every `#[doc = r#"..."#]` / `#![doc = r#"..."#]` / `#[doc = "..."]` / `#![doc = "..."]` block and extract the attribute-string body (terminator: matching `"#` for raw-string forms, matching unescaped `"` for plain-string forms).
   3. Inside each extracted body, grep for `^\s*//\s` to surface inline `// …` lines and record file:line (line numbers refer to the source file, not the extracted body).
   4. Add the resulting (file:line, text) rows to the per-hit table.

   A practical implementation: `awk '/doc = r#"/,/"#$/' <file> | grep -nE '^\s*//\s'` works for the current `src/lib.rs` shape; design picks the final walker form.

2. **Fence-language scope.** The rule applies to inline `//` comments inside **every fence kind that rustdoc renders to docs.rs readers** — `\`\`\`` (default Rust), `\`\`\`rust`, `\`\`\`no_run`, `\`\`\`ignore`, and `\`\`\`text`. All of these render the code block verbatim on docs.rs; a contributor-perspective `// …` line is equally visible regardless of whether the block is compiled, ignored, or text-only. (Hits in the workspace span `text`, `ignore`, and `no_run` fences as well as default Rust fences — see e.g. `quartzite-style/src/paint_widget.rs:20` inside `\`\`\`text` and `quartzite-style-dispatch/src/lib.rs:19+22+33+47` inside `\`\`\`ignore`.)

3. Apply the classification rule from the issue body to each hit. For each inline `// …` comment:

   1. **Does it help a docs.rs reader?** (Explains what the code does, or states a Rust ecosystem fact, or labels an input / expected output / variable / branch / step.) — **keep.**
   2. **Does it assume repo-internal architecture, workspace structure, or contributor convention?** (Points at `quartzite-widgets`, `DefaultStyle::draw_widget`, the runtime/tree contract, `#[widget_view]` attribute behaviour, "in practice we use `#[derive(Extend)]`", etc., in a way that requires the reader to already know the codebase.) — **rewrite or drop.**

   "Rewrite or drop" means: replace the line with a self-contained explanation a docs.rs reader can act on, OR delete the line if the surrounding code already conveys the same information without it. No mid-state preservation of repo-internal phrasing.

4. Concrete classification hints from the issue body (the design agent treats these as **starting recommendations**, not final verdicts — re-classify per Scope §3 during implementation and document the final per-hit verdict in the design doc):

   **Issue body marks as useful (likely keep, verify against §3):**
   - `quartzite-paint-api/src/color.rs:180` — value-computation comment.
   - `quartzite-core/src/object_base.rs:29` — Debug-impl behaviour.
   - `quartzite-style/src/default_style.rs:55` — object-safety claim (borderline).
   - `quartzite-paint-api/src/image.rs:55+59` — input-labelling.

   **Issue body marks as contributor-perspective (likely rewrite or drop, verify against §3):**
   - `src/lib.rs:127` — "Given a tree populated by the runtime:".
   - `quartzite-style/src/paint_widget.rs:59-60` — repo-internal architecture pointer about `#[widget_view]`.
   - `quartzite-style/src/paint_widget.rs:63` — contributor-convention pointer to `#[derive(Extend)]`.
   - `quartzite-style-dispatch/src/lib.rs:19+22+33+47` — numbered tutorial (check rendering before final verdict; numbered steps inside an `ignore` fence may be fine).
   - `quartzite-style-dispatch/src/dispatch.rs:119` — "In a `WidgetRoot::paint` implementation:".
   - `quartzite-style/src/paint_widget.rs:20` — "Inside `DefaultStyle::draw_widget`:".
   - `quartzite-core/src/traits.rs:240` — "`obj` is any type that implements `AsObject`" (probably fine; verify).

   **Pass-B starting recommendations (from `src/lib.rs` raw-string Quickstart fence — verify against §3):**
   - `src/lib.rs:44-46` — three-line contributor-perspective comment about `#[derive]` codegen path resolution under rustdoc's implicit-main wrapper. Mentions `#[derive]` codegen + implicit-main wrapper — the kind of repo-internal / rustdoc-internals reasoning a docs.rs reader has no context for. **Starting recommendation: rewrite or drop.** (Design verifies; if the explicit `fn main` form is load-bearing for the example, a shorter user-facing reason — e.g. "explicit `fn main` so the derive macros see the type definitions at module scope" — may be enough.)
   - `src/lib.rs:54` — `// Connect a slot to the count_changed signal.` — **likely useful label** for a step in the example; verify against §3.
   - `src/lib.rs:57` — `// Writing the property emits count_changed automatically.` — **likely useful**: labels observable behaviour for the reader; verify.
   - `src/lib.rs:60` — `// Invoke the slot dynamically through the reflection layer.` — **likely useful label**; verify.

   The full 71-hit table (65 Pass A + 6 Pass B) is reproducible via the recipes in Scope §1. Hits not pre-classified above remain entirely up to the design agent's §3 application.

5. Re-run **both passes** (Pass A `rg` + Pass B walker) after edits and confirm that every remaining hit is justifiable under Scope §3 rule (i). The final hit-count is allowed to be non-zero — useful inline comments survive — but every survivor must pass the rule.

6. `cargo test --workspace` continues to pass after the rewrites (no doc-test breakage). Default-fence doc-tests are still compiled and run by `cargo test --doc`; `text` / `ignore` / `no_run` fences are not compiled but the rendered markdown must remain syntactically intact.

## Out of scope

- **Mechanical regression gate.** The issue body explicitly excludes this: the classification is semantic ("does this help a reader?") and a regex either over-matches (every inline `//` becomes a finding) or under-matches (no token reliably identifies "scene-setting"). Human review is the gate; the script gate from #336 (PR #516) already catches the prose-side leak, which IS mechanisable.

- **Re-opening the prose-side rule from #336.** The Pattern A / Pattern B audits from #336 cover doc-comment **prose** (text outside code fences). They are settled in PR #516 and remain unchanged by this issue. This issue strictly extends scope to code-fence inline comments.

- **Non-doc `//` comments.** `//` comments that are not inside any doc-delivery surface (neither a `///` / `//!` line-prefixed block nor the body of a `#[doc = ...]` / `#![doc = ...]` attribute) do not reach rustdoc and are not in scope. Inline `//` comments inside raw-string / string-literal doc attributes ARE in scope — see Scope §1 Pass B and Key Decisions.

- **`tests/` integration files, `benches/`, `#[cfg(test)]` modules, `quartzite-test-helpers/src/**`.** Same surface exclusions as #336 — not part of the published rustdoc.

- **Fence content other than `// …` comments.** Use statements, function bodies, struct definitions, asserts, etc. inside code fences are not in scope — the sweep specifically targets lines that begin (after `///` / `//!` and optional whitespace) with `//`.

- **Renaming or restructuring code-fence examples.** Only the in-fence inline `//` comments change; surrounding API-use code stays as-is unless an inline-comment edit makes a structural rewrite trivially better (judgement call by design).

## Deferred

- Mechanising the rule via a regression script | semantic classification cannot be reliably encoded in regex per issue body | no separate issue.
- Broader audit of inline comments inside fence content beyond `// …`-prefixed lines (e.g. `let x = 3; // comment` trailing comments) | not flagged by the issue's `rg` sweep; would require a different pattern and a fresh classification pass | no separate issue — revisit if the docs.rs surface shows similar friction.

## Key decisions

| Question | Decision |
|---|---|
| Should `text` / `ignore` / `no_run` fences be in scope, or only default Rust fences? | **All rendered fences in scope.** `text`, `ignore`, `no_run`, and default Rust fences all render verbatim on docs.rs; a contributor-perspective `// …` line is equally visible in any of them. The candidate set spans multiple fence kinds (e.g. `paint_widget.rs:20` in `\`\`\`text`, style-dispatch hits in `\`\`\`ignore`). Restricting to default Rust fences would miss the majority of the named contributor-perspective hits. |
| Pre-classify per-hit verdicts in the spec, or defer to the design agent? | **Defer to design.** The issue body provides starting-recommendation classifications (Scope §4) but explicitly says "check rendering" for several borderline cases and leaves the other ~50 hits unenumerated. The design agent produces the full per-hit table during the design phase and surfaces it for owner review before edits. |
| Should this PR codify the rule in `ai-docs/doc-convention.md`? | **Yes (codify).** #336 codified the prose-side rule in `doc-convention.md` § *Self-sufficiency: no repo-internal references* (anchor `#self-sufficiency-no-repo-internal-references`) with Family A (internal-artefact citations) and Family B (contributor-tooling instructions). This PR extends the same section with a parallel Family C-style entry — or adjacent subsection — covering inline `// …` comments inside `///` / `//!` code fences. Design picks the exact placement (extend Family list vs new sibling subsection) — both options keep the doc-convention discoverable from one anchor. |
| Should this PR propagate the rule to the Review group (`code-review/SKILL.md`, `review-findings.md`, `self-review.md`) and to `design-review.md`? | **Yes (all four files).** Since the gate is semantic (no regex), the propagation is a checklist / prompt addition that asks reviewers to read code-fence inline comments through the §3 classification rule. #336 added the prose-side regex gate to all four files; this issue adds the matching semantic prompt. The Review-group sync-group requirement under the Propagation Rule (AGENTS.md → *Propagation Rule*, Review group: `code-review/SKILL.md` ↔ `review-findings.md` ↔ `self-review.md`) is satisfied because all three Review-group files are edited in the same PR; `design-review.md` is the fourth file, edited alongside. |
| Treatment of doc-comment lines that begin with `//` but are **outside** a fence (i.e. matched by the sweep but living in prose, not fence content) | Out of scope of this rule; the prose-side audit patterns from #336 / PR #516 cover that surface. The sweep `^\s*(///\|//!)\s*//\s` is a starting filter; design verifies each match sits inside an open fence before applying §3. |
| How to verify "the rewrites make sense to a downstream reader without repo context"? | **Render the affected pages locally with `cargo doc --no-deps --workspace --all-features` and inspect a sample** (5–10 of the rewritten items, weighted toward the most-contributor-perspective edits). Document the sampled URLs / item paths in the design's Test plan. No automated visual check. |
| Should inline `// …` comments inside raw-string / string-literal doc attributes (`#[doc = r#"..."#]`, `#[doc = "..."]`, `#![doc = ...]`) be in scope, alongside `///` / `//!` line-prefixed doc comments? | **Yes — widen the sweep.** Round-2 review surfaced `src/lib.rs:44-46` (the "Explicit `fn main`..." contributor-perspective comment about `#[derive]` codegen + rustdoc's implicit-main wrapper) which lives inside `#![cfg_attr(feature = "derive", doc = r#"..."#)]` and IS rendered to docs.rs. The §3 classification rule applies to inline `// …` lines regardless of which syntax delivers the doc comment. As of spec time only `src/lib.rs` matches `rg --type rust -ln 'doc\s*=\s*r#'`, contributing 6 Pass-B hits (lib.rs:44-46, 54, 57, 60). Combined candidate set: 65 (Pass A) + 6 (Pass B) = 71. The walker recipe lives in Scope §1; Technical Constraints flags its fragility (any new `#[doc = ...]` site must be re-scanned). |

## Technical constraints

- Pre-publish project — free to reword any code-fence example without compat concerns.
- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` is the canonical doc gate and must stay clean after the rewrites.
- Default-fence doc-tests (`\`\`\`` / `\`\`\`rust`) are compiled and run by `cargo test --doc`; any rewrite of an inline `// …` line inside such a fence must preserve compilability of the surrounding code.
- `text` / `ignore` / `no_run` fences are not compiled; rewrites here only affect rendered markdown.
- The Pass A sweep `rg --type rust -n '^\s*(///|//!)\s*//\s'` is a starting filter — it picks up doc-comment-internal `// …` lines regardless of fence context. The design agent must verify each match is **inside a code fence** before applying the §3 rule (matches outside fences belong to #336's surface, not this issue).
- The Pass B walker (Scope §1 recipe — `rg ... 'doc\s*=\s*r#'` + per-file attribute-body extraction + `grep -nE '^\s*//\s'`) is fragile: any new `#[doc = r#"..."#]` / `#[doc = "..."]` / `#![doc = ...]` site landing in the workspace after spec time must be re-scanned at implementation pickup. The Scope §3 classification rule applies regardless of which syntax (line-prefixed `///` / `//!` or attribute-string `#[doc = ...]`) delivers the doc comment — the docs.rs render surface is identical.
- Surface exclusions match #336 exactly: `tests/`, `benches/`, `#[cfg(test)]` regions, and `quartzite-test-helpers/src/**` are out of the published rustdoc and out of scope here.
- Workspace lints `missing_docs = "deny"` and `rustdoc::broken_intra_doc_links = "deny"` stay clean.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Every inline `// …` comment returned by EITHER Pass A (`rg --type rust -n '^\s*(///\|//!)\s*//\s'`, line-prefixed `///` / `//!` doc comments) OR Pass B (inline `//` lines inside the body of `#[doc = r#"..."#]` / `#[doc = "..."]` / `#![doc = ...]` doc attributes, currently only `src/lib.rs`) that sits inside a rustdoc code fence in the published rustdoc surface (workspace `src/**`, excluding `tests/`, `benches/`, `#[cfg(test)]` modules, `quartzite-test-helpers/src/**`) has been classified per Scope §3. Useful hits remain; contributor-perspective hits are rewritten or dropped. |
| AC2 | The design doc contains the full per-hit classification table (file:line \| verdict: keep / rewrite / drop \| one-line rationale) covering every in-scope hit from BOTH passes of the live sweep at implementation pickup (Pass A + Pass B combined). The owner approves this table before any code edits land. |
| AC3 | After implementation, a manual sample of 5–10 affected rendered doc pages (built locally via `cargo doc --no-deps --workspace --all-features`) confirms the rewritten / kept code blocks make sense to a downstream rustdoc reader without repo context. The sampled URLs / item paths are listed in the implementation PR description (or a checklist comment) so a reviewer can re-verify. |
| AC4 | `cargo test --workspace` continues to pass after the edits (no doc-test breakage from the rewrites). |
| AC5 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` passes cleanly. |
| AC6 | `cargo clippy --workspace --all-targets -- -D warnings` remains clean. |
| AC7 | Audit hits in `tests/`, `benches/`, `#[cfg(test)]` modules, and `quartzite-test-helpers/src/**` are left untouched. Hits in prose (outside any code fence) are also left to #336's Pattern-A / Pattern-B gates and not modified here. |
| AC8 | `ai-docs/doc-convention.md` is updated: the existing § *Self-sufficiency: no repo-internal references* subsection (anchor `#self-sufficiency-no-repo-internal-references`) gains coverage of inline `// …` comments inside `///` / `//!` code fences, either as a third family entry (parallel to existing Family A / Family B) or as a clearly-anchored sibling subsection. The codification names the §3 classification rule and provides at least one non-conforming-vs-conforming inline-`//` example (drawn from the Scope §4 starting set or design's per-hit table). |
| AC9 | All four review files gain a checklist / prompt line directing the reviewer to apply Scope §3 to inline `// …` comments inside doc-comment code fences: `.claude/skills/code-review/SKILL.md`, `.claude/agents/review-findings.md`, `.claude/agents/self-review.md`, `.claude/agents/design-review.md`. The added line parallels the prose-side prompt added by #336 / PR #516 in the same files. |
| AC10 | The instruction-file-size AXIOM stays satisfied: every file edited under AC8 / AC9 reports `wc -c` below the 35,000-char early-warning threshold post-edit. Pre-edit baselines verified at spec time: `doc-convention.md` 30,323; `code-review/SKILL.md` 9,560; `review-findings.md` 20,985; `self-review.md` 24,080; `design-review.md` 6,012 — all five have headroom (the largest, `doc-convention.md`, has 4,677 chars of headroom before the 35k warning; `self-review.md` has 10,920). |
| AC11 | The Propagation Rule grep (`grep -rn "<changed-keyword>" .claude/agents/ .claude/skills/ AGENTS.md ai-docs/agent-writing-style.md`) is run after the Review-group + `design-review.md` edits land, and any further sites that reference the same rule are reconciled in the same PR. |

## Open questions

- Whether the doc-convention codification (AC8) lands as a third family entry inside the existing § *Self-sufficiency: no repo-internal references* subsection (Family A — internal-artefact citations; Family B — contributor-tooling instructions; new Family C — code-fence inline contributor comments) or as a sibling subsection with its own anchor. Both are discoverable; design picks during the design phase. Not design-blocking.
- Exact wording of the Scope §3 prompt added to each review file (AC9). The four files have different review-step structures (`code-review/SKILL.md` checklist, `review-findings.md` agent prompts, `self-review.md` agent prompts, `design-review.md` agent prompts); design tailors the prompt per file while keeping the rule reference uniform. Not design-blocking.
