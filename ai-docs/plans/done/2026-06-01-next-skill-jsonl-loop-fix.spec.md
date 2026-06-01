# Fix /next auto-run jsonl-loop shell-permission failure + document the two-state `tracked` model

**Source:** user description
**Date:** 2026-06-01
**Tracked in:** (free-text; no tracking issue)

This spec has two deliverables:
- **D1 — the permission fix** (original scope): replace the shell-loop auto-run block in `next/SKILL.md` with a single literal-args `jq` invocation. (AC1–AC6.)
- **D2 — the two-state `tracked` clarification** (added by approved amendment): additive prose, documentation-only, no data / filter / behavior change, documenting that `tracked == "—"` and `tracked == "untracked"` are two deliberately-distinct non-`#N` states. (AC7–AC9.)

## Scope

### D1 — `/next` auto-run permission fix (original)

1. Rewrite ONLY the offending auto-run `!` fenced block at `.claude/skills/next/SKILL.md:29-31`:
   ```
   for f in ci-docs-workflow future-crates macros-codegen object-tree properties python signals-slots threading-runtime; do echo "== $f.jsonl =="; jq -c 'select(.tracked=="—")' ai-docs/deferred/$f.jsonl; done
   ```
   so it contains **no shell `$var` expansion and no `for`-loop**, replacing it with a single `jq -c 'select(.tracked=="—")' <8 literal .jsonl paths>` invocation. jq accepts multiple literal file-path args and streams them as one input.
2. The 8 thematic file paths must be listed literally and in full: `ai-docs/deferred/ci-docs-workflow.jsonl`, `future-crates.jsonl`, `macros-codegen.jsonl`, `object-tree.jsonl`, `properties.jsonl`, `python.jsonl`, `signals-slots.jsonl`, `threading-runtime.jsonl`.
3. One-line wording touch-up of adjacent prose if (and only if) the block shape change makes existing prose inaccurate — specifically the intro at lines 27-31 and the *Deferred-file rows* classification at lines 90-114 that reference these jq filters. Substance unchanged.
4. Verification grep confirming Triage-group siblings (`.claude/skills/triage/SKILL.md`, `.claude/agents/triage-runner.md`) share no equivalent broken `for f in` / `$f` auto-run pattern (already confirmed: they do not — propagation here is verification-only, no parallel edit).

### D2 — two-state `tracked` clarification (additive prose, documentation-only)

5. Add concise clarifying prose, in the Triage-group docs, documenting that the deferred-store `tracked` field has **two distinct, deliberately-different non-`#N` states**:
   - `tracked == "—"` (em-dash U+2014) = **UN-TRIAGED / fresh** → IS surfaced as a `/next` "Candidates needing `/triage`" item AND as a `/triage` sweep candidate (it is exactly what the `tracked=="—"` filter selects).
   - `tracked == "untracked"` (the literal word) = **already considered and consciously NOT given a GitHub issue** (declined / not-issue-tracked) → is NOT resurfaced as a candidate; it is intentionally excluded by the `tracked=="—"` filter.

   This is the **intended design**, not a bug. The prose must be additive only — it changes no jq filter, no JSONL data, and no behavior.
6. Place the clarifying prose where the `tracked=="—"` semantics are already described, in all three Triage-group files (Propagation Rule siblings — they must stay in sync):
   - `.claude/skills/next/SKILL.md` — the *Deferred-file rows* classification (around lines 90-114, esp. item 1 "Tracked vs. untracked" and the `—`/legend at lines 101-102).
   - `.claude/skills/triage/SKILL.md` — where untracked candidates + the decline-write (`tracked` set to `untracked`) are described (around lines 32, 40, 46).
   - `.claude/agents/triage-runner.md` — where the `tracked` value table / candidate filter / decline-write live (around lines 74-76, 121, 199-201).

   Exact wording and exact in-file placement are **DESIGN-phase decisions**; the spec requires only that the two-state distinction be documented clearly and consistently across the three files, without changing any filter, data, or behavior.
7. **Design-doc correction requirement (verified at design re-run).** The companion design doc `ai-docs/plans/2026-06-01-next-skill-jsonl-loop-fix.design.md` currently contains an "Out-of-scope note" that calls the `—` vs `untracked` divergence a "PRE-EXISTING latent bug". That framing is **false**. When the design is re-run for this amended spec, that note MUST be corrected to describe the two-state model as **intended, previously under-documented** (not a bug). (The spec only authors requirements; the design re-run performs the edit.)

## Out of scope

- The `tracked=="—"` filter semantics and the candidate-set definition — unchanged (D2 is prose-only; it documents the existing filter, never alters it).
- Any change to the JSONL deferred-store data, or to the meaning of `—` / `untracked` / `#N` values — the two-state model is being **documented**, not redefined.
- The widget-backlog jq blocks at lines 37-43 — they already pass; do not touch.
- Selection rules, Output sections' substance, labels handling (`blocked` / `ui-design`).
- Any Rust code or cargo tests.
- A parallel **D1** edit to `triage/SKILL.md` or `triage-runner.md` (no equivalent broken `for f in` block exists there). Note: D2 DOES touch all three Triage-group files — but only with additive clarifying prose.
- Editing the design doc in this task — the design-doc "latent bug" correction (AC9) is performed at the design re-run, not by the spec author.

## Deferred

- (none)

## Key decisions

| Question | Decision |
|---|---|
| How to replace the for-loop without `$var`? | Single `jq -c 'select(.tracked=="—")'` over the 8 literal `.jsonl` paths as multiple file args. No shell loop. |
| Keep the per-file `== $f.jsonl ==` echo headers? | Drop them — they were purely visual grouping. Each emitted row already carries `.source_path` for citation (per *Deferred-file rows* / *Output (both modes)*), so per-file provenance is preserved without headers. Design phase MAY instead use jq's `input_filename` builtin if it judges per-file provenance worth keeping inline; either is acceptable since the spec only requires the candidate set + `.source_path` readability to survive. |
| Must emitted rows stay readable as candidates? | Yes — the same JSON objects (readable via `.item` / `.source_path`) must be emitted, identical candidate set to the original loop. |
| Is the `—` vs `untracked` divergence a bug? | **No — intended two-state model, previously under-documented.** `—` = un-triaged/fresh (a candidate); `untracked` = consciously declined (deliberately excluded). The exclusion of `untracked` rows by the `tracked=="—"` filter is correct by design, asserted as correct in the #596 migration design's AC8 candidate-set-unchanged proof. It was misread as a "437-hidden-candidates bug" during this very task; D2 documents it to prevent recurrence. The design doc's prior "PRE-EXISTING latent bug" note is therefore false and must be corrected at design re-run (AC9). |
| Where does D2's wording / placement get decided? | Design phase. The spec fixes only the **content** (the two-state distinction) and the **target files** (all three Triage-group docs); exact sentences + insertion points are design's call. |

## Technical constraints

- Block must pass the harness shell-permission check: the error was `Contains simple_expansion`, triggered by `$f` expansion inside the for-loop. The rewrite must contain no shell variable expansion.
- jq streams multiple literal file-path args as one concatenated input — the candidate set (every `tracked=="—"` row across all 8 thematic files) must be byte-equivalent in content to the original loop's union of outputs.
- The other 4 `!` blocks in the file (lines 11, 17, 38, 42) already pass and must remain untouched.
- Instruction-file-only change; no `cargo` build/test gate applies, but the edited `!` block must be a valid auto-runnable command. D1 touches only `.claude/skills/next/SKILL.md`; D2 touches all three Triage-group files (`next/SKILL.md`, `triage/SKILL.md`, `triage-runner.md`) with additive prose.
- **D2 is documentation-only.** It must introduce NO new jq filter, alter NO existing filter, change NO JSONL data, and alter NO control flow. A `git diff` for the D2 portion must show only added prose lines (plus any one-line cross-reference). The candidate set produced by `/next` and `/triage` must be byte-identical before and after D2.
- **Source of truth for the two-state model:** `ai-docs/plans/done/2026-05-31-triage-deferred-jsonl.design.md` (the #596 migration design) — line 70 (`tracked` copied verbatim from source markdown, which already held both `—` and `untracked`) and lines 236-240 (`untracked` rows "never produce a candidate under the `tracked=="—"` rule", asserted correct in the AC8 candidate-set-unchanged proof); plus the triage-rewrite semantics (`tracked` rewritten to `#N` on promote, or to `untracked` on decline — see `triage-runner.md` decline-write).
- **Propagation Rule (AGENTS.md Triage group):** `/next` is in the Triage group with `triage/SKILL.md` and `triage-runner.md`; any edit to one must propagate to the siblings in the same PR. D2's prose must therefore land in all three files consistently.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | The block at `.claude/skills/next/SKILL.md:29-31` no longer contains a `for`-loop or any `$`-prefixed shell variable expansion. |
| AC2 | The rewritten block is a single `jq -c 'select(.tracked=="—")'` invocation taking all 8 thematic `.jsonl` paths as literal file arguments. |
| AC3 | Running the rewritten block emits the same candidate set (every `tracked=="—"` row across the 8 thematic files) as the original loop, with each row readable via `.item` and `.source_path`. |
| AC4 | The `tracked=="—"` filter semantics, the widget-backlog blocks (lines 37-43), selection rules, Output-section substance, and labels handling are all unchanged. |
| AC5 | Any prose referencing the changed block (lines 27-31 intro; *Deferred-file rows* lines 90-114) remains accurate after the change — wording touched only where the old block shape was described, with no substantive change. |
| AC6 | A verification grep confirms `.claude/skills/triage/SKILL.md` and `.claude/agents/triage-runner.md` contain no equivalent `for f in` / `$f` auto-run block (Triage-group propagation for **D1** = verification only). |
| AC7 | The two-state distinction — `tracked=="—"` (un-triaged/fresh, IS a candidate) vs `tracked=="untracked"` (consciously declined, NOT a candidate) — is documented clearly and consistently in all three Triage-group docs: `.claude/skills/next/SKILL.md`, `.claude/skills/triage/SKILL.md`, and `.claude/agents/triage-runner.md`. The prose states explicitly that this is the intended design, not a bug. |
| AC8 | D2 changes nothing but prose: beyond the AC1–AC6 D1 block edit, `git diff` shows only additive prose lines (and at most one-line cross-references) for the clarification. No jq filter is added or altered, no JSONL data changes, and the `/next` + `/triage` candidate set is byte-identical before and after. |
| AC9 | (Verified at design re-run, not in this task's edits.) The companion design doc `2026-06-01-next-skill-jsonl-loop-fix.design.md`'s false "PRE-EXISTING latent bug" note is corrected to describe the `—` vs `untracked` divergence as an **intended two-state model, previously under-documented**. The spec records this as a required outcome of the deliverable; the design re-run performs the edit. |

## Open questions

- (none)
