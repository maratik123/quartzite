# Rustdoc strip internal refs

**Source:** issue #336 (incl. 2026-05-14 owner comment broadening the rule, 2026-05-21 user feedback adding `quartzite-test-helpers` exclusion + `design-system/` token + `doc-convention.md` codification + Review-group + design-review gates + bare `#NN` defence-in-depth token).
**Date:** 2026-05-21
**Tracked in:** #336

## Scope

1. Rewrite every `///` / `//!` / `#[doc = "..."]` doc comment in the **published rustdoc surface** (workspace crate library targets — i.e. anything not behind `#[cfg(test)]` and not under `tests/` or `benches/`) so that it does not reference any of the following. Two families, both forbidden:

   **Family A — internal-artefact citations** (original issue body, broadened in round 3 to cover repo-internal directories that PR #515's narrower audit missed):
   - GitHub issue or PR numbers (`#NN`) or URLs (`github.com/.../issues/N`, `github.com/.../pull/N`).
   - Repo-internal paths: `ai-docs/...`, `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `design-system/...`, anything under `.claude/`, and any other repo-only folder not present in published rustdoc (e.g. `scripts/`, `.github/`, `target/`). Note: `scripts/` and `.github/` overlap with Family B's contributor-tooling token list — the duplication is intentional defence-in-depth so a path-shaped hit fires under Family A regardless of whether the surrounding prose is a contributor instruction.
   - Internal-artefact citations: "plan #N", "spec AC<n>", "the X spec", "tracked in the Y backlog", "deferred to a future plan / spec / follow-up", "per AGENTS.md", or any equivalent paraphrase that only makes sense to a contributor.

   **Family B — contributor-tooling instructions** (added per 2026-05-14 owner comment on #336):
   - Verify-locally / how-to-verify command incantations directed at a contributor (e.g. "Verify locally with `cargo build -p quartzite --no-default-features`", "Run `cargo test -p X` to confirm Y", "Re-run `RUSTDOCFLAGS=… cargo doc --no-deps` after editing").
   - Build / test / lint / fmt / doc / publish command lines included as how-to-verify or how-to-reproduce notes rather than as worked examples of API use.
   - References to repo-internal scripts (`scripts/<name>.sh`, `scripts/<name>.py`, etc.) or workflow files (`.github/workflows/...`).
   - References to "this PR", "this commit", "this implementation", "in the same change", and other coupling-to-the-development-process language.

   A downstream rustdoc reader cannot run `cargo build` in this workspace, has no access to the scripts directory, and is not party to the development process — anything they cannot act on does not belong in the published surface.

2. Each rewritten doc-comment must stand alone for a downstream rustdoc reader: replace the internal reference with either (a) plain-English behaviour wording, (b) an intra-doc link to an in-workspace item, or (c) a docs.rs / spec-document URL. If the only purpose of the sentence was a contributor instruction with no behaviour content, drop the sentence entirely.

3. Concrete rewrite surface as of task pickup (re-run BOTH audit patterns in *Audit patterns* below at start of implementation; the diff is the canonical set):
   - `quartzite-style-dispatch/src/dispatch.rs` — "per `AGENTS.md` § *API Stability*" rationale block (Family A).
   - `quartzite-core/src/snapshot.rs` — `signals_blocked` row citing `[#39](github.com/.../issues/39)` (Family A).
   - `quartzite-widgets/src/widget_base.rs` — `event_filters` field doc citing "plan #47" (Family A).
   - `quartzite-widgets/src/layout/mod.rs` — `WidgetResolverMut` doc citing "plan #47" (Family A).
   - `quartzite-renderer/src/render_harness.rs` — `render_once` (or sibling) citing "spec AC1's 'or equivalent' escape hatch" (Family A).
   - `quartzite-style-types/src/dark_palette.rs:5` — module `//!` citing "`design-system/README.md` § *Dark theme*" (Family A, `design-system/` path token).
   - `quartzite-style-types/src/dark_palette.rs:12` — `DARK_PALETTE` `///` citing "Every `ColorRole` slot is set to the RGBA value that `design-system/README.md` ..." (Family A, `design-system/` path token).
   - `src/lib.rs:203-204` — Design notes `//!` section: "Verify locally with `cargo build -p quartzite --no-default-features`" (Family B).

4. Exclude `quartzite-test-helpers` from the published-rustdoc surface entirely. Add `[lib] doc = false` to `quartzite-test-helpers/Cargo.toml` so the crate is skipped by `cargo doc --workspace`. The crate is a workspace-internal test-serialisation helper consumed only via `dev-dependencies` (`quartzite-core`, `quartzite-style`, `quartzite-runtime`, `quartzite-style-dispatch`); it is not user-facing and should not appear in published rustdoc. This is the cleanest exclusion (single-line manifest change vs `--exclude quartzite-test-helpers` on every `cargo doc` invocation in CI + AGENTS.md). With the exclusion in place, any audit-pattern hits inside `quartzite-test-helpers/src/**` are out of the published surface and ignored — the round-2 borderline case at `quartzite-test-helpers/src/lib.rs:19` becomes moot.

5. Re-run BOTH audit patterns after edits to confirm the published-surface match-set is empty (test-only matches remaining is fine and expected — see *Out of scope*).

6. Land a regression-prevention gate (decision deferred to design — see *Key decisions*): a script invoked from the `Docs` CI workflow (and optionally a pre-commit hook) that runs **both** audit patterns scoped to non-test source files and fails on any hit.

7. Doc-gate command must remain clean: `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` succeeds after the rewrite (no intra-doc links broken by the prose edits) and emits no `target/doc/quartzite_test_helpers/` directory.

8. Update `ai-docs/doc-convention.md` to codify the rule so future doc-writing does not reintroduce the leak. Add a new subsection — proposed title *Self-sufficiency: no repo-internal references* — placed under `## Scope` (parallel to the existing "Applies to / Does NOT apply to" subsection; i.e. before `## Section order (strict)`). The subsection must:
   - Name both Family A (internal-artefact citations) and Family B (contributor-tooling instructions) explicitly, mirroring the Scope §1 definitions in this spec.
   - Include at least one concrete example per family showing a non-conforming sentence and a conforming rewrite (a Family-A example using one of the `dark_palette.rs` / `widget_base.rs` rewrites, and a Family-B example using the `src/lib.rs` "Verify locally" rewrite, are natural picks but design may select different examples as long as one per family appears).
   - Cite both audit patterns from this spec's *Audit patterns* section as the local enforcement mechanism (inline copy or named link — design picks the form).

9. Update the Review group (`.claude/skills/code-review/SKILL.md` + `.claude/agents/review-findings.md` + `.claude/agents/self-review.md`) and the design-review side of the Task/Design group (`.claude/agents/design-review.md`) so each gate fires a finding when in-scope doc-comment text matches either audit pattern:
   - `.claude/agents/self-review.md` (diff-based reviewer): add a checklist item that runs both patterns against the published-surface diff (doc-comments in `src/**` excluding `tests/`, `benches/`, `#[cfg(test)]` regions, and `quartzite-test-helpers/src/**`).
   - `.claude/agents/review-findings.md` (whole-branch / whole-codebase scanner): add a finding-section that runs both patterns across the same published-surface glob.
   - `.claude/skills/code-review/SKILL.md` (orchestrator): add a one-line bullet noting the new gate is part of the standard finding set.
   - `.claude/agents/design-review.md` (design-doc reviewer): add a checklist item that scans planned doc-comment text inside the design doc itself for either pattern's tokens (the gate fires before any code exists). All four edits propagate per the AGENTS.md *Propagation Rule* — Review group and Task/Design group are already listed there, so no new propagation-rule entry is required.

## Out of scope

- `tests/` integration test files (`quartzite-style/tests/support/mod.rs`, `quartzite-widgets/tests/snapshots.rs`, `quartzite-widgets/tests/support/mod.rs`) — not part of published rustdoc; internal refs there are fine. Audit hits in these files are accepted as-is.
- `#[cfg(test)]` sibling modules / inline test modules — also not part of published rustdoc. The Family-B pattern in particular fires on `cargo test` / `cargo build` mentions inside test-module doc comments (e.g. `quartzite-runtime/src/timer_drivers.rs:450` enclosed by the `#[cfg(test)]` at line 425, and `quartzite-renderer/src/render_harness.rs:545` enclosed by the `#[cfg(test)]` at line 442); these are explicit rewrite-skips, not bugs.
- `quartzite-test-helpers/src/**` — once Scope §4 lands `[lib] doc = false`, the entire crate is outside the published surface. Audit-pattern hits inside this crate (including the previously-borderline `lib.rs:19` line) are accepted as-is and require no rewrite.
- `benches/` — not in rustdoc.
- `README.md`, `CONTRIBUTING.md`, `ROADMAP.md`, `CLAUDE.md`, `AGENTS.md`, `ai-docs/**`, `design-system/**` — project / contributor surface, not docs.rs.
- `CHANGELOG.md` (if/when added) — versioning artefact; references PRs / issues by design.
- Non-doc `//` comments — never reach rustdoc.
- Doc-test code fences (the `///`-fenced Rust block itself) — code, not prose; prose around the fence is in scope, the fenced code is not. A `cargo test` mention inside a code fence is the runnable example, not a how-to-verify instruction.
- Renaming the underlying internal artefacts (issues, plans, ACs) — only the rustdoc-visible text is changed; tracking issue numbers stay in their issues / specs / `learnings.md`.
- Other AGENTS.md / propagation-rule edits beyond the four review-group / Task-Design-group files named in Scope §9. The propagation rule itself already lists both groups; this PR exercises existing rule entries rather than adding new ones.

PR #515 (merged 2026-05-21) had previously cleaned a stale `design-system/colors_and_type.css` reference in `quartzite-style-types/src/dark_palette.rs`, but its audit scope did not catch the two remaining `design-system/README.md` references in the same file (lines 5 and 12). This spec's broadened Family-A pattern (with the `design-system/` token) is the correction-pass for exactly that class of miss; the two `dark_palette.rs` lines are now in Scope §3.

## Deferred

- Pre-commit hook for the regression gate | adds setup cost on every contributor's first commit; CI gate alone covers the regression risk for the published surface | no separate issue — revisit if doc-comment leakage recurs after the CI gate lands.
- Broader audit of `//` non-doc comments for internal references | non-doc comments don't reach the published surface; this issue is scoped to rustdoc | no separate issue.

## Key decisions

| Question | Decision |
|---|---|
| Does this PR also land the regression-prevention CI gate? | **Deferred to design phase.** Issue body explicitly flags this as a "consider — evaluate cost vs value" item; design agent decides whether the gate ships in this PR or as a fast-follow. Both options remain in scope of issue #336 by the issue's own wording. |
| Where does the regression-prevention script live, if it lands? | Defer to design. Likely `scripts/check-rustdoc-internal-refs.sh` consistent with existing `scripts/` shell scripts (`gen-roadmap.sh`, `update-ast-index.sh`, `update-snapshots.sh`). |
| Replacement strategy for "per `AGENTS.md` § *API Stability*" rationale | Drop the citation and keep the substantive behavioural claim ("this blanket impl can be removed without breaking direct trait impls"). The pre-publish-no-shims rationale is a contributor concern, not a downstream-reader concern. |
| Replacement strategy for "plan #47" / "spec AC1" / "issue #39" / "design-system/README.md" citations | Replace with self-contained behaviour wording. Use intra-doc links to in-workspace items where the link adds reader value; otherwise plain prose. For the two `quartzite-style-types/src/dark_palette.rs` hits, the substantive content (Dark theme RGBA values) is the palette itself — the citation can be dropped without information loss, since the constant's RGBA values ARE the dark-theme specification as far as a downstream reader is concerned. |
| Replacement strategy for the `src/lib.rs` "Verify locally with `cargo build -p quartzite --no-default-features`" instruction | **Drop the sentence entirely.** The `no_std + derive`-free build is exercised by the workspace CI matrix; a docs.rs reader will not run the command, and the surrounding sentence ("disable the `std` feature for the `no_std + derive`-free path") already conveys the behavioural content. No replacement prose needed. |
| How to exclude `quartzite-test-helpers` from `cargo doc --workspace`? | Add `[lib] doc = false` to `quartzite-test-helpers/Cargo.toml`. **Rationale:** single-line manifest change, no CI workflow change, no AGENTS.md doc-gate change — the canonical doc gate command stays as-is and Cargo's own target-level flag does the exclusion. Downstream test users do not need rustdoc for a `dev-dependencies`-only helper. This supersedes the round-2 borderline-case discussion for `quartzite-test-helpers/src/lib.rs:19` — the file is no longer in the published surface. |
| Section placement of the new subsection in `ai-docs/doc-convention.md` | Under `## Scope`, before `## Section order (strict)`. **Rationale:** parallel to the existing `## Scope` "Applies to / Does NOT apply to" subsection — the existing wording says what doc-comments must apply to; the new wording says what doc-comments must NOT contain. The two are the same kind of rule (what's in / what's out) and belong adjacent. Design picks the exact insertion line within the `## Scope` block. |
| Phrasing of the new `self-review.md` checklist item | Defer exact wording to design. Constraint: must be terse + binary like the other checklist items already in `self-review.md` (a reviewer reads it once and answers pass / fail without re-deriving the rule). Linking to the audit-patterns subsection of `doc-convention.md` rather than inlining the regex is acceptable as long as the linked anchor is stable. |
| Should the `design-review.md` check scan the design doc itself or the resulting code? | Scan the design doc itself. **Rationale:** the design-review gate fires before Step 8 / 9 of `/task`, when no code exists yet. Any planned doc-comment text inside a design document — typically inline rustdoc snippets the design author drafted — is the actionable surface at that stage; running the patterns over the design doc catches the leak before implementation. The published-surface code scan remains the job of `self-review.md` + `review-findings.md` after Step 8 / 9. |

## Technical constraints

- Pre-publish project (no crates.io release, no downstream clients) — free to reword any public-item doc comment without compat concerns.
- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` is the canonical doc gate and must stay clean.
- Workspace declares `missing_docs = "deny"` and `rustdoc::broken_intra_doc_links = "deny"` in `[workspace.lints.*]` — rewrites must preserve doc coverage and not introduce broken intra-doc links.
- The audit patterns (below) are the authoritative discovery tool; matches in published-surface files are the rewrite surface, matches in test / bench files are not.
- The regression gate (if it ships in this PR) must distinguish published-surface from test surface — e.g. by excluding paths matching `*/tests/*`, `*/benches/*`, `quartzite-test-helpers/src/**` (now `[lib] doc = false`), and files / modules behind `#[cfg(test)]`, OR by using `cargo doc --no-deps`'s own list of compiled inputs. Both audit patterns must be wired into the gate, not just Pattern A.
- Any new CI workflow / workflow change must pass `actionlint` per AGENTS.md AXIOM.

## Audit patterns

Two patterns. Both must be re-run at implementation pickup; both must return empty against the published surface on completion. Both apply to the same file glob: `--type rust` excluding `tests/`, `benches/`, and `#[cfg(test)]` regions (the latter cannot be expressed in `rg` alone — manual context check per match, as the issue body instructs).

**Pattern A — internal-artefact citations** (issue body, broadened in round 3 to add `design-system/` and `CONTRIBUTING.md` after PR #515 missed two `dark_palette.rs` hits; further broadened in round 5 with a bare `\b#[0-9]{1,4}\b` defence-in-depth token after a 2026-05-21 audit surfaced that the existing `[#39]` hit at `quartzite-core/src/snapshot.rs:27` was caught only because companion tokens (`tracked in`, GitHub URL) happened to share the line; further broadened post-Step-12 with `tree|blob|commit|raw` GitHub subpath tokens after the user surfaced that `src/lib.rs:167` cited `github.com/.../tree/master/examples/timer.rs` and the narrower `(issues|pull)/` token did not catch it):

```bash
rg --type rust -n '^\s*(///|//!).*(\bissue #[0-9]|\bPR #[0-9]|github\.com/.+/(issues|pull|tree|blob|commit|raw)/|ai-docs/|AGENTS\.md|CLAUDE\.md|CONTRIBUTING\.md|design-system/|\.claude/|\bspec AC[0-9]|\bplan #[0-9]|tracked in|deferred to a future (plan|spec)|\b#[0-9]{1,4}\b)'
```

**Pattern B — contributor-tooling instructions** (added per 2026-05-14 owner comment):

```bash
rg --type rust -n '^\s*(///|//!).*(\bVerify locally|\bcargo build -p|\bcargo test\b|\bcargo clippy\b|\bcargo fmt\b|RUSTDOCFLAGS|cargo doc --|scripts/[a-z]|\bthis PR\b|\bthis commit\b|\bthis implementation\b)'
```

False-positive notes:
- Pattern A: `design`, `spec`, and `specific` are common English words; `#` followed by hex digits is a colour code (`#000000ff`), not an issue ref. `design-system/` as a token will not collide with the word "design" in prose — the trailing `/` anchors it to the directory shape. The `.claude/` and `design-system/` tokens are repo-internal directory shapes that a downstream reader has no access to; both are explicit Family-A hits.
- Pattern A bare `\b#[0-9]{1,4}\b` token (round-5 addition): a 2026-05-21 audit ran the looser screening regex `rg -n --type rust '^\s*(///|//!).*#[0-9]+'` and surfaced exactly two non-test categories of hits. (i) Ten RGBA colour codes in `quartzite-paint-api/src/color.rs:46–82` (`` `#000000ff` ``, `` `#00ff00ff` ``, `` `#0000ffff` ``, `` `#0080ffff` ``, `` `#00000000` ``, `` `#2b2b2bff` ``, `` `#3c3c3cff` ``, `` `#1e1e1eff` ``, `` `#1e90ffff` ``, `` `#5bb0ffff` ``) — legitimate per the issue body allow-list ("`#` followed by hex digits is a colour code"), all 6- or 8-char hex inside backticks. (ii) The three existing Scope §3 Family-A hits at `quartzite-core/src/snapshot.rs:27` (`[#39]`), `quartzite-widgets/src/widget_base.rs:100` ("plan #47"), and `quartzite-widgets/src/layout/mod.rs:17` ("plan #47"), each already caught by the word-anchored `tracked in` / `github\.com/...` / `\bplan #[0-9]` tokens. The bare `\b#[0-9]{1,4}\b` token is defence-in-depth against future bare `#NN` references whose companion tokens (`tracked in`, GitHub URL, `\bissue` / `\bplan` / `\bPR` / `\bspec AC` prefix) might not be on the same doc-comment line. Colour hex codes do NOT collide: the `{1,4}` digit bound + `\b` word boundary excludes 6- or 8-char hex like `#000000ff` (5 + digits past the bound) and contiguous-token shapes like `quartzite-2024` (no `#` prefix). Five `quartzite-widgets/tests/snapshots.rs` "issue #281" matches surfaced by the audit are test-surface and out of scope per `## Out of scope`.
- Pattern B: `cargo test` / `cargo build` inside a `///` doc-test code fence is the runnable example, not a how-to-verify instruction — out of scope (the prose around the fence is in scope; the fenced code is not). Plain-English behaviour mentions of Rust's test harness behaviour (e.g. "each test binary runs as a separate process") may legitimately contain `cargo test` as a noun phrase — read each match in context. `#[cfg(test)]`-enclosed matches are out of scope. With `quartzite-test-helpers` excluded from rustdoc (Scope §4), any Pattern-B hits inside that crate's `src/**` are also automatically out of scope.

Neither pattern is foolproof — each match must be read in context.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | After implementation, BOTH audit patterns (Pattern A — including the round-3 `design-system/`, `CONTRIBUTING.md`, and `.claude/` tokens AND the round-5 bare `\b#[0-9]{1,4}\b` token — *and* Pattern B) run against the published-rustdoc surface (workspace `src/**`, excluding `#[cfg(test)]` modules / `tests/` / `benches/` / `quartzite-test-helpers/src/**`) return zero matches. |
| AC2 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` passes cleanly with the rewritten doc-comments. |
| AC3 | The `Docs` CI workflow continues to pass on the implementation PR. |
| AC4 | Each rewritten doc-comment, read on its own, conveys the same behavioural information to a downstream rustdoc reader as the pre-rewrite version did to a contributor — i.e. no information loss, only loss of internal-process pointers and unactionable how-to-verify instructions. |
| AC5 | Audit hits in `tests/` and `#[cfg(test)]` modules are left untouched (verifies the rewrite was correctly scoped). Specifically, the `#[cfg(test)]`-enclosed hits at `quartzite-runtime/src/timer_drivers.rs:450` and `quartzite-renderer/src/render_harness.rs:545` remain unmodified. |
| AC6 | `quartzite-test-helpers` is excluded from the published rustdoc surface. After a clean `cargo doc --no-deps --workspace --all-features` build, `target/doc/quartzite_test_helpers/` does not exist and the generated `target/doc/` index page does not list the crate. The exclusion is achieved via `[lib] doc = false` in `quartzite-test-helpers/Cargo.toml` (no CI workflow change, no AGENTS.md doc-gate change). |
| AC7 | If the regression-prevention gate ships in this PR (decision per *Key decisions*): a new CI step or pre-commit hook fails when EITHER audit pattern matches a published-surface file. Demo this by introducing a deliberate violation locally (one for Pattern A — e.g. an `ai-docs/...` or `design-system/...` reference — one for Pattern B) and observing the failure, then reverting. The gate's path-exclusion logic must mirror AC1's scoping (i.e. skip `quartzite-test-helpers/src/**` along with `tests/` / `benches/` / `#[cfg(test)]`). |
| AC8 | If the regression-prevention gate ships: any new workflow file (or modified workflow file) passes `actionlint`. |
| AC9 | `cargo clippy --workspace --all-targets -- -D warnings` remains clean. |
| AC10 | `cargo test --workspace` continues to pass after the `quartzite-test-helpers` exclusion (verifies `[lib] doc = false` does not affect the crate's role as a `dev-dependencies` test helper — only its rustdoc-target visibility). |
| AC11 | `ai-docs/doc-convention.md` contains a new subsection (proposed title *Self-sufficiency: no repo-internal references*) that (a) names Family A and Family B explicitly with the same definitions used in Scope §1 of this spec, (b) includes at least one concrete non-conforming-vs-conforming example per family, and (c) cites both audit patterns from this spec's *Audit patterns* section. The subsection is placed under `## Scope`, before `## Section order (strict)`, per *Key decisions*. |
| AC12 | All four review-gate files — `.claude/skills/code-review/SKILL.md`, `.claude/agents/review-findings.md`, `.claude/agents/self-review.md`, and `.claude/agents/design-review.md` — reference both audit patterns (inline copy, named anchor, or stable link to the `doc-convention.md` subsection added in AC11) such that grepping any of the four files for a shared named anchor chosen by design (e.g. `Pattern A` / `Pattern B`, or `internal-refs gate`, or similar) returns a hit in each file. Each file's check fires at the right surface per Scope §9: diff-based for `self-review.md`, whole-codebase for `review-findings.md`, orchestrator-bullet for `code-review/SKILL.md`, design-doc-self-scan for `design-review.md`. |
| AC13 | After the edits land, `wc -c` on each of `ai-docs/doc-convention.md`, `.claude/skills/code-review/SKILL.md`, `.claude/agents/review-findings.md`, `.claude/agents/self-review.md`, and `.claude/agents/design-review.md` reports a size strictly below the 35,000-char early-warning threshold from the AGENTS.md instruction-file-size AXIOM. Pre-edit baselines (verified at spec time): 27,595 / 9,293 / 20,543 / 23,605 / 5,475 — all five files have ample headroom. |

## Open questions

None at spec time. The remaining design-space ambiguity (whether the regression gate ships in this PR or as a fast-follow) is captured in *Key decisions* as a deferred-to-design call. Round 3 resolved the round-2 `quartzite-test-helpers/src/lib.rs:19` borderline case by excluding the crate from the published surface entirely via `[lib] doc = false` (Scope §4) — that ambiguity is gone. Round 4 added the doc-convention codification (Scope §8) and Review-group + design-review gate propagation (Scope §9); placement, checklist-phrasing, and design-doc-vs-code-scan questions are pinned in *Key decisions*. Round 5 closed the round-4 cap-extension request with a single regex tightening — appending the bare `\b#[0-9]{1,4}\b` defence-in-depth token to Pattern A — after a 2026-05-21 `#[0-9]+` doc-comment audit confirmed colour codes (6- or 8-char hex) and existing Scope §3 hits as the only non-test categories present. No further design-affecting ambiguity remains.
