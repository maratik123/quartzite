# Escalate workspace clippy `warn` → `deny` for safety/perf guards

**Source:** issue #464
**Date:** 2026-05-18
**Tracked in:** #464

## Scope

1. In root `Cargo.toml`'s `[workspace.lints.clippy]` table, change the
   `level` of three specific lints from `"warn"` to `"deny"`:
   - `undocumented_unsafe_blocks`
   - `large_stack_frames`
   - `large_stack_arrays`
2. Mirror the `undocumented_unsafe_blocks` severity bump in every
   in-tree instruction file that names the lint with its old severity
   (full sweep — Option 3 of issue #464). The following literal-string
   updates are required (all mechanical, one line each):
   - `AGENTS.md` line 97 (*Documentation* row): `clippy::undocumented_unsafe_blocks = "warn"` → `"deny"`.
   - `ai-docs/code-style.md` line 137 (*Documentation* section): same string update.
   - `ai-docs/code-style.md` line 411 (*Lints that mechanically enforce* section): same string update.
   - `.claude/agents/self-review.md` line 85: `#![warn(clippy::undocumented_unsafe_blocks)]` → `#![deny(clippy::undocumented_unsafe_blocks)]`.
   - `.claude/agents/review-findings.md` line 77: same `#![warn(…)]` → `#![deny(…)]` update.
   - `.claude/skills/task/reference.md` line 233: same `#![warn(…)]` → `#![deny(…)]` update.
3. Mirror the `large_stack_frames` / `large_stack_arrays` severity bump
   in the one in-tree prose mention that names them with their old
   severity:
   - `ai-docs/code-style.md` lines 44–46 (*Linter posture* section): the
     parenthetical `(both \`warn\`, listed separately so each survives a
     future per-group rollback)` describing `clippy::large_stack_frames`
     and `clippy::large_stack_arrays` updates to `(both \`deny\`, listed
     separately so each survives a future per-group rollback)`.
4. Verify the workspace still builds, lints, tests, formats, and
   doc-gates clean after the severity bump (full verification recipe in
   Acceptance Criteria below).

## Out of scope

- Adding new lints to `[workspace.lints.clippy]`. This is severity-only
  escalation of three already-declared entries.
- Touching the 16-entry workspace `allow`-list. Allows stay allows.
- Removing `-D warnings` from the CI clippy invocation. The CLI gate
  stays as belt-and-braces regardless of declaration-level severity.
- Escalating the `pedantic` or `nursery` lint groups. Issue #464
  classifies these as "Weak" and "Don't" respectively; `nursery`
  in particular is a footgun across toolchain bumps.
- Per-pedantic-lint promotion (Option B in the issue). Higher effort,
  separate judgement call, deferred.
- Touching `[workspace.lints.rust]` / `[workspace.lints.rustdoc]` — both
  already at `deny`.

## Deferred

- Per-pedantic-lint promotion audit (Option B) | higher effort, per-lint
  judgement, not blocking the narrow win this spec captures | yes,
  follow-up issue when prioritised.

## Key decisions

| Question | Decision |
|---|---|
| Which escalation option from issue #464? | **Option A — Narrow.** Promote only `undocumented_unsafe_blocks`, `large_stack_frames`, `large_stack_arrays` to `deny`. Issue marks this "recommended"; B is higher-effort follow-up; C escalates `nursery` and is documented as a footgun. |
| Keep `pedantic` / `nursery` groups at `warn`? | Yes. Group-level `deny` for `nursery` ties the workspace to a single toolchain build per the issue's "Don't" rationale; `pedantic` is intentionally aggressive and the 17-entry allow-list is partial protection — group-`deny` would force every new pedantic-only hit into either an in-flight fix or a per-PR allow-list edit. |
| Keep `-D warnings` on the CI clippy command line? | Yes. Declaration-level `deny` makes the policy explicit in the manifest (and lets local `cargo clippy` and IDE plugins / rust-analyzer see the real severity); the CLI flag remains as belt-and-braces so any *future* `warn`-declared lint also hard-fails CI. Issue #464 lists "Removing `-D warnings`" as out of scope. |
| Update AGENTS.md *Linter posture* row? | No per-lint detail change — the row summarises policy without enumerating severities, so it stays as-is. The *Documentation* row (separate bullet) DOES spell out `clippy::undocumented_unsafe_blocks = "warn"` and is updated to `"deny"`. |
| How wide should the `undocumented_unsafe_blocks` text sweep go (Options 1 / 2 / 3 of issue #464's open question)? | **Option 3 — Full sweep.** All in-tree instruction-file mentions of the lint's old severity are updated to match the new manifest declaration: `AGENTS.md`, `ai-docs/code-style.md` (both occurrences), `.claude/agents/self-review.md`, `.claude/agents/review-findings.md`, `.claude/skills/task/reference.md`. Rationale: the docstring drift risk from leaving five literal-string mismatches across the Review and Task/Design propagation groups outweighs the diff width — a future reader looking at the checklist would otherwise see `#![warn(…)]` and trust it as the current policy. Option 1 (narrow) and Option 2 (AGENTS siblings only) rejected after surfacing the trade-off. |
| Comment / justification policy for the three elevated entries? | None required. The `Cargo.toml` `#`-comment rule applies to `allow`-list entries (justify the allow); elevated `deny` entries restore the project's default-strict posture and need no extra justification. |

## Technical constraints

- Edit lives in `Cargo.toml` at the workspace root (no member-crate
  manifests touched). Existing `[lints] workspace = true` opt-in in each
  member crate already propagates the new severity.
- `clippy.toml` thresholds (`stack-size-threshold`,
  `array-size-threshold`) stay unchanged — those govern *when* the
  `large_stack_*` lints fire, not the severity of a hit.
- Current hit count for all three lints is 0 (per issue #464's
  candidates table, citing PR #463 and #423). Escalation must remain
  no-op for the existing codebase; any new hit surfaced by the bump is
  itself a defect to fix in-spec (not a reason to back the escalation
  out).
- Local non-flagged `cargo clippy --workspace` must hard-fail on a
  synthetic violation of any of the three lints after the change —
  this is the behavioural delta the issue motivates.

## Acceptance Criteria

| #   | Criterion                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| --- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| AC1 | Root `Cargo.toml`'s `[workspace.lints.clippy]` table has `large_stack_frames = "deny"`, `large_stack_arrays = "deny"`, and `undocumented_unsafe_blocks = "deny"`. The `pedantic` and `nursery` group entries remain at `level = "warn"` with `priority = -1`. The allow-list (lines 41–72 of the pre-change `Cargo.toml`) is unchanged — `rg -c '"allow"$' Cargo.toml` returns the same count as before the edit (currently 16; this is a live-derived assertion, not a hard-coded literal).                                                                                                |
| AC2 | `cargo build --workspace` exits 0.                                                                                                                                                                                                                                                                                                                                                                                                              |
| AC3 | `cargo clippy --workspace --all-targets -- -D warnings` exits 0 (no severity-drift discoveries; in-tree hit count remains 0).                                                                                                                                                                                                                                                                                                                   |
| AC4 | `cargo clippy --workspace --all-targets` (NO `-D warnings`) exits 0 against the unchanged tree, AND a synthetic violation of any one of the three escalated lints (e.g., an `unsafe { … }` block without a `// SAFETY:` comment in a throwaway scratch crate or via a test-only `#[allow(...)]`-removed probe) causes that same invocation to exit non-zero. Demonstrates the declaration-level severity is now load-bearing without the flag. |
| AC5 | `cargo test --workspace` exits 0.                                                                                                                                                                                                                                                                                                                                                                                                              |
| AC6 | `cargo fmt -- --check` exits 0.                                                                                                                                                                                                                                                                                                                                                                                                                |
| AC7 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` exits 0.                                                                                                                                                                                                                                                                                                                                           |
| AC8 | `cargo build -p quartzite --no-default-features --features libm` exits 0 (derive-free / no\_std path).                                                                                                                                                                                                                                                                                                                                         |
| AC9 | AGENTS.md *Code Style → Documentation* row reads `clippy::undocumented_unsafe_blocks = "deny"` (was `"warn"`); no other AGENTS.md edits required.                                                                                                                                                                                                                                                                                              |
| AC10 | `ai-docs/code-style.md` reads `clippy::undocumented_unsafe_blocks = "deny"` at every occurrence (lines 137 and 411 of the pre-change file); zero occurrences of `clippy::undocumented_unsafe_blocks = "warn"` remain in the file. Additionally, the *Linter posture* section parenthetical describing `clippy::large_stack_frames` / `clippy::large_stack_arrays` (lines 44–46 of the pre-change file) reads `(both \`deny\`, listed separately so each survives a future per-group rollback)`. The anchored multi-line grep `rg -nU 'large_stack_arrays\`\s*\n\s*\(both \`warn\`,' ai-docs/code-style.md` returns zero hits post-edit (baseline pre-edit is 1 hit). NOTE: the bare substring `(both \`warn\`,` legitimately still returns 1 hit post-edit at line 42, which describes `clippy::pedantic` / `clippy::nursery` — those groups stay at `warn` per spec § Out of scope and must NOT be touched. |
| AC11 | `.claude/agents/self-review.md`, `.claude/agents/review-findings.md`, and `.claude/skills/task/reference.md` each read `#![deny(clippy::undocumented_unsafe_blocks)]` at the previously-`#![warn(…)]` site (one occurrence per file: `self-review.md:85`, `review-findings.md:77`, `task/reference.md:233`); zero occurrences of `#![warn(clippy::undocumented_unsafe_blocks)]` remain across these three files.                                 |
| AC12 | Across **authoritative policy surfaces only**, the literal text `clippy::undocumented_unsafe_blocks = "warn"` and `#![warn(clippy::undocumented_unsafe_blocks)]` both return zero hits via `rg` — no-stragglers check covering AC9–AC11 and any in-tree text we missed. The recipe is `rg --hidden '<pattern>' --glob '!ai-docs/plans/**' --glob '!ai-docs/learnings.md' --glob '!ai-docs/context.md' --glob '!ROADMAP.md' --glob '!ai-docs/deferred/_inbox.md'`. The `--hidden` flag is required because `.claude/` is a hidden directory (leading dot) and ripgrep skips hidden dirs by default; without it, the attribute-form straggler check would silently false-pass when the three `.claude/agents/` + `.claude/skills/` mirrors are still at `#![warn(…)]`. Excluded surfaces are narrative/historical references to past states, not authoritative copies: `ai-docs/plans/**` (this spec/design themselves quote the old severity for context), `ai-docs/learnings.md` (append-only per Boundary rule 1 — historical correction entries), `ai-docs/context.md` (workspace-lints-lift narrative paragraph in the Plans list), `ROADMAP.md` (auto-generated activity log quoting completed-plan summaries), `ai-docs/deferred/_inbox.md` (triage row quoting the tighten-clippy-pedantic-nursery spec's verbatim text). |

## Open questions

None — the issue is fully self-contained with an explicit "recommended"
option, and the verification recipe in the issue maps 1:1 to the
acceptance criteria above.

## Amendment log

- **2026-05-18 (Step 7 GO-with-notes resolution).** Spec amended from
  "AGENTS.md-only mirror" (Option 1 of issue #464's open question) to
  "full sweep" (Option 3). Triggered by the design's open question on
  propagation-rule mirror drift across `ai-docs/code-style.md` (×2) and
  the three Review / Task-Design skill/agent files. Scope item 2 was
  expanded from one bullet to six; ACs AC10 / AC11 / AC12 added; Key
  decisions row added documenting the trade-off. The manifest edit
  (Scope item 1) and the gate-verification ACs (AC2–AC8) are unchanged.
- **2026-05-18 (Step 7 round-2 ITERATE).** Spec amended again after
  design-review caught two recipe-correctness defects:
  1. AC1 asserted a "17-entry" allow-list and the verifying grep
     `rg -c '"allow"$' Cargo.toml` returned **16** on the unchanged tree
     (drift; the live count is 16, lines 41–72 contain 16 entries with
     interleaved `#`-justification comments). AC1 reworded to assert the
     pre/post counts match without hard-coding the literal — invokes the
     "query live state" AXIOM in AGENTS.md § Dependency Versions. The
     out-of-scope mention of the allow-list also updated from "17-entry"
     to "16-entry".
  2. AC12 carve-out (`!ai-docs/plans/**` only) was undersized: the grep
     also returns hits in `ai-docs/learnings.md` (append-only per
     Boundary rule 1), `ai-docs/context.md`, `ROADMAP.md`, and
     `ai-docs/deferred/_inbox.md` — all historical narrative references
     to past states, not authoritative declarations. AC12 reworded to
     explicitly carve out those four additional paths, with the
     rationale stated inline.
  3. AC12 recipe also missing `--hidden` flag — ripgrep does not descend
     into hidden directories by default, and `.claude/` is hidden (leading
     dot); without `--hidden` the attribute-form straggler check would
     silently false-pass when the three `.claude/agents/` +
     `.claude/skills/` mirrors are still at `#![warn(…)]`. Caught by the
     design agent during its live recipe spot-check; folded into the same
     round-2 amendment.
- **2026-05-18 (Step 7 round-3 STOP).** Spec amended a third time —
  user-authorised round-cap override — after design-review caught a
  remaining docstring drift defect: `ai-docs/code-style.md` lines 44–46
  describe `clippy::large_stack_frames` / `clippy::large_stack_arrays`
  as "(both `warn`, listed separately so each survives a future
  per-group rollback)". The Option 3 sweep enumerated only
  `undocumented_unsafe_blocks` string mentions and missed this prose
  reference to the *other* two escalated lints. Spec changes: (a) Scope
  item 2 split into item 2 (six `undocumented_unsafe_blocks` mirrors,
  unchanged) and a new item 3 (the one `large_stack_*` prose site); old
  item 3 (verify) renumbered to item 4. (b) AC10 widened to assert the
  parenthetical reads `(both \`deny\`, ...)` and uses an anchored
  multi-line grep keyed to the `large_stack_arrays` line that precedes
  it — the bare-substring zero-hit assertion would have been
  unfulfillable because line 42 of `code-style.md` legitimately keeps
  `(both \`warn\`,` for `clippy::pedantic` / `clippy::nursery`, which
  stay at `warn` per spec § Out of scope. (c) No new ACs needed — AC10
  covers the file already. The manifest edit (Scope item 1) and the
  gate-verification ACs (AC2–AC8) remain unchanged.
