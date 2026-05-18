# Design: Escalate workspace clippy `warn` → `deny` for safety/perf guards

**Issue:** #464
**Date:** 2026-05-18

## Approach

Single-commit, two-class change: a manifest severity bump plus a workspace-wide
text-mirror sweep covering both the `undocumented_unsafe_blocks` mentions (per
the spec's Option 3 amendment) and the one `large_stack_*` prose mention
surfaced by the round-3 amendment.

1. **`Cargo.toml` (workspace root) — three one-line severity bumps.** In
   `[workspace.lints.clippy]` (lines 34–39 of the current root manifest),
   change `large_stack_frames = "warn"` → `"deny"` (line 37),
   `large_stack_arrays = "warn"` → `"deny"` (line 38), and
   `undocumented_unsafe_blocks = "warn"` → `"deny"` (line 39). The
   `pedantic` / `nursery` group entries (lines 35–36, both
   `{ level = "warn", priority = -1 }`) and the 16-entry allow-list
   (lines 42–72, with interleaved `#`-justification comments) are not
   touched. Each member crate already carries `[lints] workspace = true`,
   so the bumped severity propagates automatically — no per-crate
   `Cargo.toml` edits needed.

2. **Full instruction-file mirror sweep — seven literal-string updates across
   five files (spec Scope items 2 and 3).** Six edits cover the
   `undocumented_unsafe_blocks` Option-3 sweep; one edit covers the
   `large_stack_*` prose site surfaced by the round-3 amendment. All seven are
   mechanical, no surrounding text changes:
   - `AGENTS.md` line 97 (*Code Style → Documentation* row):
     `` `clippy::undocumented_unsafe_blocks = "warn"` `` →
     `` `clippy::undocumented_unsafe_blocks = "deny"` ``.
   - `ai-docs/code-style.md` line 137 (*Documentation* section):
     `clippy::undocumented_unsafe_blocks = "warn"` →
     `clippy::undocumented_unsafe_blocks = "deny"`.
   - `ai-docs/code-style.md` line 411 (*Lints that mechanically enforce*
     section): same string update.
   - `ai-docs/code-style.md` lines 44–46 (*Linter posture* section,
     parenthetical describing `clippy::large_stack_frames` /
     `clippy::large_stack_arrays`): the literal substring
     `` (both `warn`, listed separately so each survives a future per-group rollback) ``
     updates to
     `` (both `deny`, listed separately so each survives a future per-group rollback) ``.
     Adjacent `pedantic`/`nursery` parenthetical on line 42 (also "(both
     `warn`, ...)" but for the two group entries that legitimately stay at
     `warn`) is NOT touched.
   - `.claude/agents/self-review.md` line 85:
     `#![warn(clippy::undocumented_unsafe_blocks)]` →
     `#![deny(clippy::undocumented_unsafe_blocks)]`.
   - `.claude/agents/review-findings.md` line 77: same `#![warn(…)]` →
     `#![deny(…)]` update.
   - `.claude/skills/task/reference.md` line 233: same `#![warn(…)]` →
     `#![deny(…)]` update.

**Why this approach.** The spec's manifest portion is mechanical and
pre-validated: issue #464's candidates table (cited in spec § Technical
constraints) measured 0 in-tree hits for all three lints under the pre-change
tree, so the escalation is a no-op for existing code. The behavioural delta is
forward-protection (any *future* hit hard-fails the local non-flagged
`cargo clippy --workspace` invocation as well as CI, instead of being swallowed
when a developer forgets `-D warnings` locally).

The mirror sweep is mandated by the spec amendments (2026-05-18 Step 7
GO-with-notes resolution → Option 3, plus the round-3 amendment adding the
`large_stack_*` prose site). It is still mechanical — six edits are literal
`"warn"` → `"deny"` or `warn(…)` → `deny(…)` substitutions; the seventh is a
literal `` `warn` `` → `` `deny` `` substitution inside a parenthetical, no
prose rewording. Folding all seven edits into the single commit alongside the
manifest change preserves the design's "documented severity matches the
manifest at every git revision" invariant and avoids triggering a stale
intermediate-commit state where some docstrings still read `warn`.

**Rejected alternatives.**

- **Option 1 — Narrow (`AGENTS.md` only) (rejected per spec amendment).**
  Originally chosen in the pre-amendment design. Five other in-tree locations
  (`ai-docs/code-style.md` ×2, `.claude/agents/self-review.md`,
  `.claude/agents/review-findings.md`, `.claude/skills/task/reference.md`)
  would have continued to spell the lint as `"warn"` / `#![warn(…)]`,
  presenting a future reader with conflicting authoritative-looking copies of
  the current policy. Rejected during Step 7 GO-with-notes review because the
  drift risk across the Review and Task/Design propagation groups outweighed
  the diff-width saving.
- **Option 2 — AGENTS.md siblings only (`code-style.md` lines 137 + 411)
  (rejected per spec amendment).** Updates the two `code-style.md` lines that
  the AGENTS.md *Documentation* row directly cross-references, but leaves the
  three checklist-style `#![warn(…)]` mentions in `self-review.md`,
  `review-findings.md`, and `task/reference.md` stale. Asymmetric — splits the
  docstring-drift risk into "fixed for prose" / "still drifting for
  checklist attributes". Rejected for the same reason as Option 1: leaves
  half the surface still inconsistent with the manifest.
- **Option B (per-pedantic-lint promotion to `deny`).** Higher effort,
  per-lint judgement, deferred by the spec (§ Out of scope; § Deferred).
- **Option C (escalate the `nursery` group to `deny`).** Documented as
  a footgun in issue #464 (`nursery` ties the workspace to a single
  toolchain build); the spec § Key decisions explicitly keeps `nursery`
  at `warn`.
- **Removing `-D warnings` from CI clippy.** Spec § Out of scope. The
  CLI flag stays as belt-and-braces so any *future* `warn`-declared lint
  also hard-fails CI.
- **Splitting the manifest edit and the text sweep into two commits.** Adds
  no review value; leaves the intermediate commit where the documented
  severity disagrees with the manifest (the same docstring-drift state Option
  3 is intended to close). Single commit keeps every source synchronised at
  every revision.
- **Splitting the text sweep itself across multiple subtasks (e.g., one per
  file or one per Propagation group).** All six edits are independent literal
  substitutions with no inter-file ordering constraint. Splitting them would
  multiply gate-suite executions without changing semantics. See *Decomposition*
  below for the explicit grouping justification.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Edit `[workspace.lints.clippy]` in root `Cargo.toml`: flip `large_stack_frames`, `large_stack_arrays`, and `undocumented_unsafe_blocks` from `"warn"` to `"deny"`. Apply the seven instruction-file mirror edits: `AGENTS.md` line 97; `ai-docs/code-style.md` lines 137, 411, and 44–46 (the `large_stack_*` parenthetical — flip `` `warn` `` → `` `deny` `` in the second of the two adjacent parentheticals; do NOT touch the line-42 `pedantic`/`nursery` parenthetical); `.claude/agents/self-review.md` line 85; `.claude/agents/review-findings.md` line 77; `.claude/skills/task/reference.md` line 233. Run the AC2–AC8 gate suite (build, clippy with `-D warnings`, clippy without `-D warnings`, test, fmt, doc gate, no_std path) and the AC4 behavioural probe. Verify AC1, AC9, AC10 (both halves), AC11, and AC12 via the grep recipes in *Test Design*. Commit. | `Cargo.toml`, `AGENTS.md`, `ai-docs/code-style.md`, `.claude/agents/self-review.md`, `.claude/agents/review-findings.md`, `.claude/skills/task/reference.md` | — |

**Why a single subtask (justification).** The eight file edits (manifest +
seven mirror locations) are all literal-substitution one-liners with zero
logic content, zero inter-file ordering dependency, and a single shared
verification gate suite. Splitting into per-file or per-Propagation-group
subtasks would:

- Multiply the gate-suite cost (each split would re-run AC2–AC8 against an
  identical tree state, since none of the text edits influence build / test /
  clippy / doc / fmt output — they all live in instruction Markdown).
- Create intermediate-commit states where the documented severity disagrees
  with the manifest (the exact docstring-drift class Option 3 is closing).
- Inflate the decomposition cost-of-context for no review value: the diff is
  small enough (8 lines across 6 files) to fit on one screen, and the spec's
  Amendment log itself characterises every mirror edit as "all mechanical,
  one line each".

The Spec Amendment recipe (`.claude/skills/task/SKILL.md` Step 7) explicitly
allows scope expansions to fold into the existing decomposition when the
expansion is mechanically homogeneous with the original edit. This is that
case.

## Handoff plan

`M = 1` (one group, terminal):

- **Group A:** subtask 1 — terminal group (1 subtask; within the 1..=3 range). No handoff between groups; the single group completes Step 8 in its own `/context-reset` subagent. Per `.claude/skills/task/SKILL.md` Step 8 and the every-group handoff contract in `.claude/skills/task/reference.md` § *Every-group handoff (rationale)*, the entry into Group A spawns `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).

## Risks

- **Risk:** Issue #464 / spec § Technical constraints assert "0 hits"
  for all three lints under the current tree, but the count is sourced
  from PR #463 / #423 measurements taken before this branch existed.
  Drift since then (a new `unsafe { … }` block, a stack-large fn, or a
  stack-large array) would surface as a clippy hard-fail under AC3.
  **Mitigation:** Subtask 1 runs `cargo clippy --workspace --all-targets`
  (without `-D warnings`) AND `cargo clippy --workspace --all-targets -- -D warnings`
  against the post-edit tree. If either fires on one of the three
  escalated lints, the surfaced hit is itself a defect to fix in this
  same spec (per spec § Technical constraints: "any new hit surfaced by
  the bump is itself a defect to fix in-spec — not a reason to back the
  escalation out"). If the count of fixes exceeds 2–3 small edits, raise
  it as a Spec Amendment per `.claude/skills/task/SKILL.md` Step 7
  *Spec Amendment recipe*; otherwise fold the fixes into the same
  commit.
- **Risk:** The AC4 behavioural probe (synthetic-violation hard-fail
  without `-D warnings`) is a *one-shot* manual check, not a regression
  test. **Mitigation:** Document the probe outcome in the commit message
  (e.g., "AC4 probed via `unsafe { core::ptr::null::<u8>().is_null() }`
  in a scratch `tests/` file — clippy exited 1 with
  `error: unsafe block missing a safety comment` — probe reverted").
  The probe verifies the declaration-level severity is load-bearing
  without the flag; a `warn` declaration would have surfaced as a
  warning (exit 0) under the same invocation.
- **Risk:** `clippy.toml` thresholds (`stack-size-threshold = 524288`,
  `array-size-threshold = 524288`) are unchanged; under `deny`, the
  current 512 KiB ceiling becomes load-bearing. **Mitigation:** Spec
  § Technical constraints explicitly retains the thresholds. A future
  legitimate hit (e.g., a 600 KiB stack frame in optimised code) would
  be a separate spec change (raise the threshold, add an allow, or
  refactor the offender). No mitigation needed at this revision (hit
  count is 0).
- **Risk (RESOLVED by spec amendment).** Propagation Rule mirror drift.
  The pre-amendment design called out that the literal string
  `clippy::undocumented_unsafe_blocks = "warn"` also lived in
  `ai-docs/code-style.md` lines 137 + 411, and that the
  `#![warn(clippy::undocumented_unsafe_blocks)]` attribute phrasing lived
  in `.claude/agents/self-review.md` line 85,
  `.claude/agents/review-findings.md` line 77, and
  `.claude/skills/task/reference.md` line 233. The Step 7 GO-with-notes
  spec amendment (logged in spec § Amendment log, dated 2026-05-18)
  resolved this risk by adopting Option 3 — full sweep — which folds all
  five additional locations into spec § Scope item 2 and adds ACs AC10
  / AC11 / AC12 to enforce the no-stragglers state. The risk is therefore
  no longer mitigated-but-deferred — it is closed: the design action that
  used to be deferred is now in-scope and verified by the AC10–AC12 grep
  recipes in *Test Design*.
- **Risk (RESOLVED by spec round-2 amendment).** Recipe correctness drift
  between design and live state. The pre-amendment AC1 hard-coded the
  allow-list count as "17" but live `rg -c '"allow"$' Cargo.toml` returns
  **16**, and the pre-amendment AC12 carve-out only excluded
  `ai-docs/plans/**`, leaving live hits in `ai-docs/learnings.md` (×2),
  `ai-docs/context.md` (×1), `ROADMAP.md` (×1), and
  `ai-docs/deferred/_inbox.md` (×1) — all historical / auto-generated
  narrative surfaces, not authoritative policy declarations. The spec
  round-2 ITERATE amendment (2026-05-18) reworded AC1 to assert pre/post
  count equality (live-derived) and extended AC12's carve-out to all 5
  paths. The corrected recipes are exercised against the live unchanged
  tree below; see *Test Design* table for the per-AC live-state baseline.
- **Risk (RESOLVED by spec round-3 amendment, observation for future
  similar work).** The spec's mirror-sweep scope was originally defined by
  exact-string enumeration around the literal substring
  `clippy::undocumented_unsafe_blocks` (Option 3, six sites). That framing
  is fragile in two ways exposed by the round-3 amendment: (a) it scopes
  on one of three escalated lints, so prose mentions of the *other two*
  (`large_stack_frames`, `large_stack_arrays`) are invisible to a literal
  grep — the `code-style.md` lines 44–46 parenthetical fell through this
  gap until design-review round 3 surfaced it; (b) "all mentions of the
  severity" is a semantic predicate, not a string predicate, so any
  paraphrase ("both `warn`, …") slips past a substring search.
  **Implication for future similar work.** When a spec needs to keep
  documented severity in lockstep with manifest severity, the audit should
  enumerate the *semantic* surface (every prose paragraph that names any
  of the affected lints, regardless of the surrounding spelling) rather
  than scoping by string-match on one lint name. A future spec could
  alternatively run a broader `rg` over `"warn"` (or `\`warn\``) within a
  narrowed glob first and triage from there. The mitigation already in
  place for *this* spec is the round-3 amendment itself plus the widened
  AC10. No further design action required.

## Test Design

No new automated tests added. The change has no source-code impact
under the (verified-zero) current hit count, so a `#[cfg(test)] mod
tests` block addition would carry no behaviour to assert. Validation is
gate-execution per the spec's AC2–AC8 suite, the AC4 synthetic probe,
and grep-driven verification for AC1 + AC9–AC12.

**Gate suite (matches `AGENTS.md § Build & Test` + spec AC2–AC8).** All
commands run from the workspace root against the post-edit tree:

| AC | Command | Pass criterion |
|----|---------|----------------|
| AC2 | `cargo build --workspace` | exit 0 |
| AC3 | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no severity-drift discoveries |
| AC4 (positive half) | `cargo clippy --workspace --all-targets` (NO `-D warnings`) | exit 0 against the unchanged tree |
| AC4 (negative half, manual probe) | Introduce a synthetic `unsafe { /* no SAFETY */ … }` in a temporary scratch file or test, run `cargo clippy --workspace --all-targets` (NO `-D warnings`) | exit non-zero with `error: unsafe block missing a safety comment` — revert the synthetic violation immediately after observing the failure |
| AC5 | `cargo test --workspace` | exit 0 |
| AC6 | `cargo fmt -- --check` | exit 0 |
| AC7 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` | exit 0 |
| AC8 | `cargo build -p quartzite --no-default-features --features libm` | exit 0 |

**AC1 / AC9–AC12 verification (grep-driven).** All recipes assume `cwd` is
the workspace root. Each recipe's `live unchanged tree` column documents what
the recipe returned when run against the pre-edit tree at design time
(2026-05-18); this is the per-AC baseline the implementor uses to detect drift
before applying any edits.

| AC | Recipe | Live unchanged tree (baseline) | Post-edit pass criterion |
|----|--------|--------------------------------|--------------------------|
| AC1 (deny-level) | `rg -n '^(large_stack_frames\|large_stack_arrays\|undocumented_unsafe_blocks)\s*=\s*"deny"$' Cargo.toml` | 0 hits | exactly 3 hits |
| AC1 (pedantic/nursery) | `rg -n '^(pedantic\|nursery)\s*=\s*\{\s*level\s*=\s*"warn"' Cargo.toml` | exactly 2 hits (lines 35 + 36) | exactly 2 hits (unchanged) |
| AC1 (allow-list count, baseline) | `rg -c '"allow"$' Cargo.toml` (run before any edit) | **16** | record this number — call it `N_pre` |
| AC1 (allow-list count, post-edit) | `rg -c '"allow"$' Cargo.toml` (run after the edit) | n/a | must equal `N_pre` (live-derived assertion per spec AC1) |
| AC9 | `rg -n 'clippy::undocumented_unsafe_blocks = "(warn\|deny)"' AGENTS.md` | 1 hit on line 97 with `"warn"` | exactly 1 hit on line 97 with `"deny"`; zero `"warn"` hits |
| AC10 (undocumented_unsafe_blocks half) | `rg -n 'clippy::undocumented_unsafe_blocks = "(warn\|deny)"' ai-docs/code-style.md` | exactly 2 hits (lines 137, 411) with `"warn"` | exactly 2 hits at the same lines with `"deny"`; zero `"warn"` hits |
| AC10 (large_stack parenthetical, anchored — required) | `rg -nU 'large_stack_arrays\`\s*\n\s*\(both \`warn\`,' ai-docs/code-style.md` | **exactly 1 hit** anchored at line 44 (the `large_stack_arrays` line) with `(both \`warn\`,` on the immediately-following line (45) | zero hits |
| AC10 (large_stack parenthetical, post-edit positive form) | `rg -nU 'large_stack_arrays\`\s*\n\s*\(both \`deny\`,' ai-docs/code-style.md` | 0 hits on the unchanged tree | exactly 1 hit at the same anchor (line 44 → line 45) |
| AC10 (bare-substring sanity, do NOT use as the pass criterion) | `rg -n '\(both \`warn\`,' ai-docs/code-style.md` | **2 hits** on the unchanged tree (line 42 — `pedantic`/`nursery`, legitimately stays at `warn`; line 45 — `large_stack_*`, target of this edit) | exactly 1 hit remaining at line 42 — line 45 must be gone. **Do not use as a zero-hit assertion; the line-42 hit is correct and load-bearing.** |
| AC11 | `rg -n '#!\[(warn\|deny)\(clippy::undocumented_unsafe_blocks\)\]' .claude/agents/self-review.md .claude/agents/review-findings.md .claude/skills/task/reference.md` | exactly 3 hits (self-review.md:85, review-findings.md:77, task/reference.md:233), each spelling `warn(…)` | exactly 3 hits at the same lines, each spelling `deny(…)`; zero `warn(…)` hits |
| AC12 (string-form) | `rg -n --hidden 'clippy::undocumented_unsafe_blocks = "warn"' --glob '!ai-docs/plans/**' --glob '!ai-docs/learnings.md' --glob '!ai-docs/context.md' --glob '!ROADMAP.md' --glob '!ai-docs/deferred/_inbox.md'` | **exactly 3 hits** — `AGENTS.md:97`, `ai-docs/code-style.md:137`, `ai-docs/code-style.md:411` (the same surfaces AC9/AC10 cover) | zero hits |
| AC12 (attribute-form) | `rg -n --hidden '#!\[warn\(clippy::undocumented_unsafe_blocks\)\]' --glob '!ai-docs/plans/**' --glob '!ai-docs/learnings.md' --glob '!ai-docs/context.md' --glob '!ROADMAP.md' --glob '!ai-docs/deferred/_inbox.md'` | **exactly 3 hits** — `.claude/agents/self-review.md:85`, `.claude/agents/review-findings.md:77`, `.claude/skills/task/reference.md:233` (the same surfaces AC11 covers) | zero hits |

**Critical recipe note — `--hidden` flag is mandatory for AC12.** `ripgrep`
by default does not descend into hidden directories, and `.claude/` is
hidden. Without `--hidden`, the AC12 attribute-form recipe would return 0
hits even on a *pre-edit* tree (false pass), making the no-stragglers check
silently inert. The recipe was validated against the live unchanged tree
during design (2026-05-18): `--hidden` produced the 3-hit baseline shown
above; the same recipe without `--hidden` produced 0 hits. The AC9 / AC10 /
AC11 recipes do not require `--hidden` because they name their target files
explicitly on the command line (ripgrep does not skip explicitly-named
hidden files).

**AC12 carve-out rationale (matches spec AC12 verbatim).** The five
`--glob '!path'` exclusions are narrative/historical references to past
states, not authoritative copies:

- `ai-docs/plans/**` — this spec/design themselves quote the old severity
  for context.
- `ai-docs/learnings.md` — append-only per Boundary rule 1; two entries
  (lines 595, 597) document the historical `#![warn(…)]` directive's
  placement-across-crates rule.
- `ai-docs/context.md` — workspace-lints-lift narrative paragraph in the
  Plans list (line 190) quotes the old severity verbatim.
- `ROADMAP.md` — auto-generated activity log (line 42) quotes a completed
  plan summary that contains the old severity.
- `ai-docs/deferred/_inbox.md` — triage row (line 88) quotes the
  tighten-clippy-pedantic-nursery spec's verbatim text.

If a stray hit appears anywhere outside these five paths after the edit,
AC12 fails and the sweep is incomplete.

**Spot-check summary against live unchanged tree (2026-05-18).** Every grep
recipe in this Test Design section was run against the live tree before this
design was finalised (round 3); the *Live unchanged tree (baseline)* column
above records what each one returned. Findings:

- AC1 allow-list is **16** (matches spec round-2 amendment).
- AC9 surface: **1 hit** at `AGENTS.md:97` with `"warn"` (matches spec).
- AC10 undocumented_unsafe_blocks half: **2 hits** at lines 137, 411 with
  `"warn"` (matches spec).
- AC10 large_stack anchored recipe: **1 hit** at line 44→45 with
  `(both \`warn\`,` on line 45 (matches spec's intent).
- AC10 *bare-substring sanity*: **2 hits** at lines 42 + 45 on the
  unchanged tree (post-edit: 1 hit, line 42 only). Line 42 is the
  pedantic/nursery parenthetical that legitimately stays at `warn` per
  spec § Out of scope; line 45 is the `large_stack_*` parenthetical
  flipped by this spec. Documented as informational baseline only —
  the spec's authoritative AC10 assertion uses the anchored multi-line
  recipe above, not this bare substring.
- AC11 surface: **3 hits** at `self-review.md:85`, `review-findings.md:77`,
  `task/reference.md:233`, each spelling `warn(…)` (matches spec).
- AC12 (with `--hidden`): **3+3 hits** on the unchanged tree (matches
  spec round-2 amendment).

**No fixture or helper additions.** The AC4 probe is a manual one-shot;
codifying it as an automated test would require a `compile_fail` doctest
or a `trybuild` harness — both out of scope per the spec's "0 new tests"
posture (the design phase reads the spec literally; if review wants
automation, raise it as a Spec Amendment).

## Open questions

None — the recipe-correctness defect surfaced during round-3 live-validation
(spec AC10's bare-substring `(both \`warn\`,` zero-hit assertion was
unfulfillable because the legitimate line-42 pedantic/nursery parenthetical
also matches) has been resolved by a same-turn spec follow-up amendment
applying **Resolution 1**: the spec AC10 now uses the anchored multi-line
recipe `rg -nU 'large_stack_arrays\`\s*\n\s*\(both \`warn\`,' ai-docs/code-style.md`
(baseline 1 hit pre-edit, asserting 0 hits post-edit). The bare-substring
sanity-row in *Test Design* documents the legitimate 1-hit post-edit residual
at line 42 explicitly.

All earlier open questions remain closed: Option 1 / 2 / 3 propagation-sweep
resolved by Step 7 GO-with-notes (Option 3 + round-3 prose addition);
AC1 allow-list count + AC12 carve-out width + `--hidden` flag resolved by the
spec round-2 ITERATE amendment. The verification recipes for AC1, AC9, AC10
(both halves), AC11, and AC12 have been live-validated against the unchanged
tree.
