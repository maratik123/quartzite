# Design: Fix /next auto-run jsonl-loop shell-permission failure + document the two-state `tracked` model

**Issue:** (free-text; no tracking issue) — spec `ai-docs/plans/2026-06-01-next-skill-jsonl-loop-fix.spec.md`
**Date:** 2026-06-01

This design covers two deliverables from the amended spec:

- **D1 — the `/next` auto-run permission fix** (original scope, already GO'd): replace the
  shell-loop `!` block at `.claude/skills/next/SKILL.md:29-31` with a single literal-args
  `jq` invocation. (AC1–AC6.)
- **D2 — the two-state `tracked` clarification** (added by approved amendment): additive,
  documentation-only prose across all three Triage-group files, documenting that
  `tracked=="—"` and `tracked=="untracked"` are two deliberately-distinct non-`#N` states.
  No data / filter / behavior change. (AC7–AC9.)

---

## Approach

### D1 — `/next` auto-run permission fix (unchanged — already GO'd)

`.claude/skills/next/SKILL.md` lines 29–31 carry an auto-run `!` fenced block:

```
for f in ci-docs-workflow future-crates macros-codegen object-tree properties python signals-slots threading-runtime; do echo "== $f.jsonl =="; jq -c 'select(.tracked=="—")' ai-docs/deferred/$f.jsonl; done
```

The harness shell-permission check rejects it with `Contains simple_expansion` because
of the `$f` variable expansion inside the `for`-loop. The fix is purely mechanical and
instruction-file-only: replace the loop with a single `jq` invocation that takes all 8
thematic `.jsonl` files as **literal** path arguments. jq streams multiple literal
file-path args as one concatenated input, so the emitted candidate set (every
`tracked=="—"` row across the 8 files) is the byte-for-byte union of the original loop's
per-file outputs — minus the `== <file> ==` echo headers.

Replacement block (lines 29–31):

```!
jq -c 'select(.tracked=="—")' ai-docs/deferred/ci-docs-workflow.jsonl ai-docs/deferred/future-crates.jsonl ai-docs/deferred/macros-codegen.jsonl ai-docs/deferred/object-tree.jsonl ai-docs/deferred/properties.jsonl ai-docs/deferred/python.jsonl ai-docs/deferred/signals-slots.jsonl ai-docs/deferred/threading-runtime.jsonl
```

This contains no `$`-expansion and no `for`-loop, satisfying AC1/AC2.

**Key decision — drop the per-file echo headers; do NOT use `input_filename` (recommended).**
The spec's Key-decisions table leaves latitude between (a) dropping the `== $f.jsonl ==`
headers entirely vs. (b) preserving per-file provenance inline via jq's `input_filename`
builtin. Recommendation: **(a) drop the headers, plain single-jq.** Justification, verified
against the live skill and store:

- **Every row already carries `.source_path`** (verified: 358/358, 90/90, 44/44, 10/10,
  23/23, 6/6, 30/30, 59/59 across the 8 files). The skill's downstream prose is the only
  consumer of provenance, and it reads `.source_path` exclusively — *Deferred-file rows*
  item 3 (line 113: "read `.item` … and `.source_path` for the source citation") and
  *Output (both modes)* (line 120: "cite the source via `.source_path`"). No prose reads
  the originating filename for thematic rows. Per-file provenance is therefore already
  preserved without headers; the headers were purely visual grouping (spec Key-decisions).
- **`input_filename` would inject noise no consumer reads.** Verified: the builtin emits
  the *absolute* path passed on the command line (e.g.
  `/tmp/.../a.jsonl`), so threading it into each object (`. + {_file: input_filename}`)
  adds a redundant, verbose key that contradicts `.source_path` (which points at the
  originating spec, not the deferred file) and bloats each candidate line. It buys nothing
  the Output section uses.
- **Minimal diff.** Option (a) replaces exactly the 3-line block body with a one-line jq;
  option (b) would also require a prose touch-up to document the new `_file` key. (a) is the
  smaller, lower-risk change.

**Rejected alternatives (D1):**
- *`input_filename` provenance variant* — rejected per the decision above (noise, no consumer).
- *Keep headers via a literal `echo` per file* — would re-introduce an 8×(echo+jq) block,
  larger and uglier than the loop it replaces, for grouping no downstream prose consumes.
- *xargs / find-based iteration* — reintroduces shell machinery the permission check may
  object to and adds zero value over literal args.

**Live-state note (informational, not a contract):** as of 2026-06-01 all 8 thematic files
have **zero** `tracked=="—"` rows, so both the original loop and the replacement currently
emit nothing. Byte-equivalence was verified by `diff`-ing the original loop output against
the replacement (`diff /tmp/orig.txt /tmp/new.txt` → IDENTICAL). The equivalence is
structural (jq concatenation = loop union), not dependent on the current empty state.

### D2 — two-state `tracked` clarification (additive prose, documentation-only)

**Correction of this design's prior false note (AC9).** An earlier revision of this design
contained an "Out-of-scope note" calling the `tracked=="—"` vs `tracked=="untracked"`
divergence a "PRE-EXISTING latent bug shared by `/next`, `/triage`, and `triage-runner.md`"
that hides "437 currently-invisible rows". **That framing was false and has been removed.**
The two values are an **intended, previously under-documented two-state model**, not a bug:

- `tracked=="—"` (em-dash U+2014) = **un-triaged / fresh** → IS a `/next` "Candidates needing
  `/triage`" item and IS a `/triage` sweep candidate. It is exactly what the
  `tracked=="—"` filter selects.
- `tracked=="untracked"` (the literal word) = **consciously considered and declined** — the
  row was seen, judged not worth a GitHub issue, and the triage decline-write set
  `tracked` to `untracked` (see `triage-runner.md` Phase 6 decline-write, line 199). It is
  **intentionally excluded** by the `tracked=="—"` filter so declined rows are never
  resurfaced as candidates.

The 437 live `untracked` rows are therefore correctly excluded **by design** — they are
declined, not hidden. This is the candidate-set-unchanged invariant the #596 migration design
proves (see *Source of truth* below). D2 adds prose documenting this; it does **not** change
the filter.

**Source of truth for the two-state model** (cited in the prose, not duplicated into it):

- `ai-docs/plans/done/2026-05-31-triage-deferred-jsonl.design.md` (the #596 migration design)
  — **line 70** (`tracked` stored as the cell-4 verbatim string, which already held both `—`
  and `untracked` as distinct values: `—`, `#48`, `#49 (closed)`, `untracked`, …) and
  **lines 236–240** (the AC8 candidate-set-unchanged proof: `untracked` / `#N` rows surface
  under NEITHER the old `Status`-emoji rule NOR the new `tracked=="—"` rule, em-dash count
  verified 0 → candidate set provably unchanged).
- The triage decline-write semantics: `triage-runner.md` Phase 6 (line 199, `tracked ← untracked`
  on decline) and `triage/SKILL.md` cell-iteration sweep (line 46, "On decline … `tracked`
  set to `untracked`").

**Placement decision — one canonical definition + two brief cross-references (recommended).**
To avoid three-way drift across Propagation-Rule siblings (AGENTS.md Triage group), the design
fixes **one canonical two-state definition** in `triage-runner.md` (the subagent that *writes*
both states, hence the natural source-of-truth) and **brief one-to-two-sentence cross-references**
in the other two files, anchored where each already mentions the `—` filter. Exact wording and
exact insertion points (design's call per spec item 6):

1. **`.claude/agents/triage-runner.md` — canonical definition.** Insert directly under the
   Phase 3 candidate-rules table (after line 78, the `_inbox.jsonl`-routes-to-drain note), a
   short paragraph:

   > **Two-state `tracked` (non-`#N`) — intended, not a bug.** The `tracked` field has two
   > deliberately-distinct non-`#N` states: `—` (em-dash U+2014) = **un-triaged / fresh** → a
   > candidate (selected by the `tracked=="—"` rule above); `untracked` (literal word) =
   > **consciously declined** — the row was seen and judged not worth a GitHub issue, so the
   > Phase 6 decline-write set `tracked` to `untracked` (line 199; the approval/promote half of
   > that same action table lands in Phase 7.5). Declined rows are
   > **intentionally excluded** by the `tracked=="—"` filter and are never resurfaced. This is
   > by design — the #596 migration design's AC8 candidate-set proof
   > (`ai-docs/plans/done/2026-05-31-triage-deferred-jsonl.design.md` lines 236–240) treats
   > both as non-candidate states.

   Rationale: this is adjacent to both the candidate rule (line 74) and the decline-write
   (line 199 — same file), so the definition sits beside both anchors it reconciles.

2. **`.claude/skills/next/SKILL.md` — fix the conflated legend + cross-reference.** Line 101
   currently reads `\`—\` ⇒ untracked` — this **conflates the two states** (it labels the
   em-dash "untracked", the exact misread the amendment corrects). Edit it to distinguish
   them and add a one-line cross-reference. Target wording for the line-101 bullet:

   > field `tracked`: `#N` ⇒ tracked; `—` (em-dash) ⇒ **un-triaged / fresh** ⇒ candidate
   > (the `jq 'select(.tracked=="—")'` filter above); `untracked` (literal word) ⇒
   > **consciously declined**, intentionally NOT a candidate. These two non-`#N` states are
   > a deliberate two-state model (not a bug) — see `triage-runner.md` Phase 3. Emoji-status
   > legend (`🟡 v2` etc.): …

   This is the only D2 edit that also corrects an existing inaccuracy (the `—` ⇒ untracked
   conflation); it remains additive in candidate-set terms (the `—` filter is unchanged) and
   touches no jq filter or data.

3. **`.claude/skills/triage/SKILL.md` — cross-reference at the "Unhandled" definition.**
   After the "Unhandled" sentence (line 32, which already pins `tracked=="—"`), append one
   sentence:

   > (`tracked=="untracked"` rows are **consciously declined**, not un-triaged — they are
   > intentionally excluded from this count and from `/next`, the deliberate non-`#N`
   > counterpart of `—`; see `triage-runner.md` Phase 3. Not a bug.)

**D2 invariants (binding):** the prose is additive only. It introduces NO new jq filter,
alters NO existing filter, changes NO JSONL data, and alters NO control flow. The single
line-101 edit in `next/SKILL.md` *rewords* an existing (inaccurate) legend bullet but leaves
the `tracked=="—"` filter and the candidate set byte-identical. AC8's byte-equivalence proof
(re-run the candidate-set diff) must still pass after D2.

**Char-cap check (AGENTS.md 35k early-warning / 40k hard cap).** Current `wc -c` of the three
targets and the projected post-D2 deltas:

| File | Current `wc -c` | D2 added prose (approx) | Projected | Status |
|---|---|---|---|---|
| `.claude/skills/next/SKILL.md` | 9,015 | +~250 (line-101 reword + 1-line xref) | ~9,265 | OK (≪ 35k) |
| `.claude/skills/triage/SKILL.md` | 20,541 | +~280 (1 sentence) | ~20,821 | OK (≪ 35k) |
| `.claude/agents/triage-runner.md` | 38,941 | +~600 (canonical paragraph) | ~39,541 | **minor — 35k–40k band** |

`triage-runner.md` is **already** at 38,941 chars — in the 35,000–39,999 `minor` early-warning
band **before** D2, and the canonical paragraph pushes it to ~39,541, still under the 40,000
hard cap but deeper into the warning band. **This is a pre-existing condition, not introduced
by D2**, but D2 must keep its `triage-runner.md` addition minimal (the canonical paragraph
above is ~600 chars) and MUST NOT cross 40,000. The implementer MUST re-run
`wc -c .claude/agents/triage-runner.md` after the edit and confirm `< 40000`; if the final
count lands in 39,900–40,000, trim the paragraph (it is the only file with headroom risk).
Surface the pre-existing 35k+ state of `triage-runner.md` to the orchestrator as a follow-up
extraction candidate for the next `/ai-audit` pass (out of scope for this task).

**Rejected alternatives (D2):**
- *Full two-state definition repeated verbatim in all three files* — rejected: triples the
  drift surface across Propagation-Rule siblings and adds ~1.8k chars, worsening the
  `triage-runner.md` char-cap pressure. One canonical definition + two cross-references keeps
  the siblings in sync with a single edit point.
- *New "two-state model" subsection / standalone reference doc* — rejected as YAGNI for a
  two-sentence concept; the canonical paragraph beside the candidate-rule table is enough.

---

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | **D1 — permission fix.** Replace the `for`-loop `!` block (lines 29–31) with the single literal-args `jq` invocation; touch up adjacent prose (intro lines 27–31; *Deferred-file rows* lines 90–114) ONLY if the block-shape change made it inaccurate; then verify (a) the new block runs with no permission error and is byte-equivalent to the original loop's union, and (b) AC6 propagation grep over `.claude/skills/triage/SKILL.md` + `.claude/agents/triage-runner.md` confirms no sibling `for f in` / `$f` block. | `.claude/skills/next/SKILL.md` | — |
| 2 | **D2 — canonical two-state definition.** Add the canonical "Two-state `tracked` (non-`#N`) — intended, not a bug" paragraph under the Phase 3 candidate-rules table (after line 78), citing the #596 AC8 proof + the Phase 6 decline-write (line 199; the approval/promote half of that same action table lands in Phase 7.5). Re-run `wc -c .claude/agents/triage-runner.md` and confirm `< 40000`. No jq/filter/data change. | `.claude/agents/triage-runner.md` | 1 |
| 3 | **D2 — cross-references + legend fix.** In `next/SKILL.md`: reword the line-101 `—` ⇒ untracked legend bullet to distinguish `—` (fresh candidate) from `untracked` (declined, not a candidate) + one-line `triage-runner.md` Phase 3 cross-reference. In `triage/SKILL.md`: append the one-sentence two-state cross-reference after the "Unhandled" definition (line 32). Then verify AC7 (all three files document the two-state model and say "not a bug") and AC8 (re-run the candidate-set diff → byte-identical; `git diff` shows only additive/reword prose, no `select(` / filter / data changes outside the D1 block). | `.claude/skills/next/SKILL.md`, `.claude/skills/triage/SKILL.md` | 2 |

Three atomic subtasks (M = 3). D1 (subtask 1) is independent and already GO'd; D2 splits into
the canonical definition (subtask 2, the source-of-truth file) then the two cross-references
(subtask 3, which must reference the canonical paragraph and so depends on it). All edits are
instruction-file-only — no Rust code, no cargo gate.

## Handoff plan

Per `.claude/skills/task/SKILL.md` Step 8 + `.claude/agents/design.md` § Rules → handoff-grouping,
the `## Handoff plan` is mandatory for **every M ≥ 1**. Group-sizing contract: non-terminal
groups MUST be exactly 3 consecutive subtasks; the terminal group may be 1..=3.

- **Group A:** subtasks 1–3 — terminal group (3 subtasks; within the 1..=3 range, and exactly
  the 3-subtask cap). Entry into Group A spawns `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry); the single group
  completes Step 8 in its own `/context-reset` subagent. No handoff between groups (there is
  only one group).

## Risks

- **D2 mis-framed as a behavior change (AC8):** the line-101 reword in `next/SKILL.md` edits
  an existing legend bullet, which could be mistaken for a filter change. Mitigation: the
  `tracked=="—"` filter string is untouched; only the prose *describing* `—`/`untracked` is
  reworded. Re-run the AC3/AC8 candidate-set diff (original loop vs. live store, and pre/post
  D2) → must stay byte-identical. `git diff` must show no `select(` change outside the D1 block.
- **Three-way prose drift (Propagation Rule):** documenting the two-state model in three files
  risks divergent wording. Mitigation: ONE canonical paragraph in `triage-runner.md`; the other
  two files carry brief cross-references that point at it rather than restating it.
- **`triage-runner.md` char-cap (35k early-warning):** the file is already 38,941 chars (in the
  35k–40k `minor` band before D2). Mitigation: keep the canonical paragraph ≤ ~600 chars; re-run
  `wc -c` after the edit and confirm `< 40000`; trim if it lands in 39,900–40,000. Surface the
  pre-existing 35k+ state as an `/ai-audit` extraction follow-up (out of scope here).
- **Prose drift (AC5, D1):** the *Deferred-file rows* classification (lines 90–114) and intro
  (lines 27–31) describe the jq filter, not the loop syntax — they say "via a baked-in `jq`
  one-liner" and "the `jq 'select(.tracked=="—")'` filter above". Replacing a loop-of-jq with a
  single-jq keeps that description accurate, so **no D1-driven prose edit is expected** (the
  line-101 edit is a D2 change, not a D1 shape fix). Mitigation: re-read lines 27–31 and 90–114
  after the edit; touch up wording ONLY if a sentence now mis-describes the block shape.
- **Accidental edit of the 4 passing `!` blocks (AC4):** lines 11, 17, 38, 42 already pass.
  Mitigation: the D1 edit targets only the lines 29–31 block; the two widget-backlog jq blocks
  (lines 37–43) and the `gh issue list` / `cat INDEX.md` blocks must remain byte-identical.
- **Non-byte-equivalent candidate set (AC3):** mitigated structurally (jq multi-file = loop
  union) and empirically (`diff` → IDENTICAL during design verification). Re-confirm at
  implementation time by running both forms and diffing.
- **Em-dash literal corruption:** the filter value is U+2014 `—`. The replacement (D1) and all
  D2 prose must copy the em-dash byte-for-byte — do not retype it as a hyphen `-` or en-dash
  `–`. Mitigation: copy the `'select(.tracked=="—")'` fragment / the `—` glyph verbatim from the
  existing block.

## Test Design

Instruction-file-only change — "tests" are runtime command verifications, not Rust `#[cfg(test)]`.

- **Location:** ad-hoc Bash verification during implementation (no test file).
- **Entry point:** the rewritten `!` block command (D1) + grep/diff over the three files (D2).

### D1 scenarios

- *Happy path / permission (AC1):* run the rewritten one-liner directly via Bash; it must exit
  0 with no `Contains simple_expansion` / permission error. Confirm the source no longer contains
  `for f in` or `$f` (`grep -nE 'for f in|\$f' .claude/skills/next/SKILL.md` → no match in the
  auto-run block).
- *Single-jq shape (AC2):* the block is one `jq -c 'select(.tracked=="—")'` over all 8 literal
  `.jsonl` paths.
- *Candidate-set equivalence (AC3):* capture original-loop output
  (`for f in <8 names>; do jq -c 'select(.tracked=="—")' ai-docs/deferred/$f.jsonl; done`) and
  replacement output to temp files; `diff` them → must be IDENTICAL. Each emitted row must remain
  readable via `.item` and `.source_path` (both verified present on 100% of rows).
- *Empty-state edge case:* the live store legitimately has **437 `tracked=="untracked"`
  (consciously-declined) rows and 0 `tracked=="—"` (fresh) rows** across the 8 thematic files
  (verified 2026-06-01: `jq 'select(.tracked=="untracked")' | wc -l` → 437; `select(.tracked=="—")`
  → 0). Only fresh `—` rows are candidates, so **both the old loop and the new single-jq emit
  EMPTY output — this is correct, not a regression and not a "hidden-candidate" bug.** The 437
  `untracked` rows are correctly excluded by design (the two-state model D2 documents). To exercise
  the non-empty path for AC3, inject a temp `{…,"tracked":"—"}` row into two scratch files under a
  `mktemp -d` dir and confirm the multi-file jq emits both rows in file order — that injected-scratch
  run is the real byte-equivalence proof, not the empty live-store run.
- *Untouched-blocks (AC4):* `git diff` must show D1 changes confined to the lines 29–31 block;
  the 4 other `!` blocks and the widget-backlog blocks unchanged.
- *Propagation verification (AC6):*
  `grep -nE 'for f in|\$f' .claude/skills/triage/SKILL.md .claude/agents/triage-runner.md` → no
  match (verified during design: siblings use per-`<theme>` single-file jq, no loop). Verification
  only; no parallel D1 edit.

### D2 scenarios

- *Two-state prose present + "not a bug" (AC7):* grep each of the three files for the two-state
  framing and the explicit "not a bug" / "intended" / "by design" assertion:
  - `grep -niE 'two-state|intended|not a bug|consciously declined' .claude/agents/triage-runner.md`
    → matches the canonical paragraph.
  - `grep -niE 'two-state|not a bug|consciously declined|un-triaged' .claude/skills/next/SKILL.md`
    → matches the reworded line-101 legend + cross-reference.
  - `grep -niE 'two-state|not a bug|consciously declined|deliberate' .claude/skills/triage/SKILL.md`
    → matches the appended sentence.
  All three must mention BOTH `—` (fresh / un-triaged / candidate) and `untracked` (declined /
  not a candidate) and assert the model is intended.
- *Additive-prose-only + byte-identical candidate set (AC8):* `git diff` over the three files must
  show only added prose lines plus the single reworded line-101 bullet in `next/SKILL.md` — NO new
  or altered `select(` / jq filter outside the D1 block, NO `.jsonl` data-file change. Re-run the
  AC3 candidate-set diff (8-file `select(.tracked=="—")` against the live store) BEFORE and AFTER
  the D2 edits → both runs must produce byte-identical output (currently both empty; the structural
  invariant holds regardless).
- *Design-doc no longer frames this as a bug (AC9):* `grep -niE 'latent bug|PRE-EXISTING.*bug'
  ai-docs/plans/2026-06-01-next-skill-jsonl-loop-fix.design.md` → the only matches are inside this
  AC9 / correction narrative describing the *removal* of the false framing; the design no longer
  *asserts* the divergence is a bug. (This design revision already performs that correction — see
  the D2 Approach "Correction of this design's prior false note" paragraph.)
- *Char-cap (AGENTS.md):* `wc -c .claude/skills/next/SKILL.md .claude/skills/triage/SKILL.md
  .claude/agents/triage-runner.md` after the D2 edits → all three `< 40000`; `triage-runner.md`
  re-checked specifically (pre-D2 38,941 → projected ~39,541; must stay `< 40000`, trim if it lands
  39,900–40,000).

- **Fixtures / helpers:** two scratch `.jsonl` files under a `mktemp -d` dir for the non-empty-path
  AC3 check; otherwise the live `ai-docs/deferred/*.jsonl` store and the three Triage-group
  instruction files.

## Open questions

- (none)
