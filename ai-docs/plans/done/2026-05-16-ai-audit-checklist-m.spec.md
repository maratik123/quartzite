# Extend /ai-audit Phase 2 with Checklist M — agent-writing-style.md conformance

**Source:** issue #369
**Date:** 2026-05-16
**Tracked in:** #369

## Scope

1. Add **Checklist M — `agent-writing-style.md` conformance** to `.claude/skills/ai-audit/SKILL.md` Step 2.3, slotting after the existing Checklist L block. Checklist M sweeps every fail-loud block (`> **AXIOM**`, paragraphs containing `**NEVER**` / `**MUST**` / `**MUST NOT**` / `**FORBIDDEN**` / `**STOP**`) across the files Phase 2 already inventories in Step 2.2 (AGENTS.md, CLAUDE.md, every `.claude/skills/**/SKILL.md`, every `.claude/agents/**.md`, `ai-docs/{code-style,doc-convention,agent-writing-style,corrections-log}.md`) and verifies pattern-by-pattern:

   - **Pattern 1 (AXIOM blockquote)** — every `> **AXIOM —`-prefixed block is followed by an associated "if you see... → action" table.
   - **Pattern 2 (fail-loud verbs)** — at most one bold-uppercase verb (from the Pattern 2 verb set: `**NEVER**`, `**MUST**`, `**MUST NOT**`, `**FORBIDDEN**`, `**STOP**`, `**REJECT**`, `**ALWAYS**`, `**REMOVE**`, `**REPLACE**`, `**DELETE**`) per paragraph.
   - **Pattern 3 (action tables)** — in every "if X → do Y" two-column table inside a fail-loud block, the right column starts with an action verb (or a bolded uppercase action verb), not narrative prose.
   - **Pattern 4 (explicit file lists, never globs)** — in fail-loud lists that enumerate paths, each path is its own bullet; a glob is not the *entire* list (a glob inside one bullet with a parenthetical "any file under this directory" remains allowed).
   - **Pattern 5 (numbered enumeration of triggers)** — in numbered "this fires when..." / "this is authorised by..." lists inside fail-loud blocks, the OR/AND connector placement is consistent across items (all trailing, or all leading; mixing is the violation).
   - **Pattern 6 (do/not examples)** — for non-trivial rules (rules with both a positive shape and a negative shape mentioned in the body), both shapes are shown, not just one.
   - **Pattern 7 (compaction recovery callout)** — in every file carrying the callout, the body matches one of the three locked variants (A/B/C) verbatim per `ai-docs/agent-writing-style.md § Pattern 7` and the locked full bodies at `ai-docs/plans/done/2026-05-14-sonnet-skill-reentry-protocol.design.md`. No invented 4th variant.
   - **Anti-patterns table audit (Pattern-adjacent)** — for each row in `ai-docs/agent-writing-style.md § Anti-patterns`, verify the anti-pattern text does NOT appear as a positive rule anywhere in the audited file set. (Catches a fail-loud block in some skill that accidentally embodies an Anti-patterns row.)

2. Findings categorised by severity matching the existing A–L convention (`blocker` / `major` / `minor` / `nit`). Pre-resolved severity defaults are scoped under § Key decisions; design agent may refine if needed.

3. **Reciprocal cross-reference** in `ai-docs/agent-writing-style.md`: add a new `## Enforcement` section (placed between `## Citation in PRs` and `## Out of scope`) that names `/ai-audit § Checklist M` as the audit venue and links to the new Step 2.3 § M section. Keep `## Citation in PRs` focused on PR authoring (separation of concerns).

4. **AGENTS.md Propagation Rule reciprocal entry** — only if the existing `ai-docs/agent-writing-style.md` row needs amendment to reflect that editing `## Patterns` is now also enforced via `/ai-audit` Checklist M. See § Open questions.

5. **Post-merge audit run (AC5)** — running `/ai-audit phase2` against the current corpus after the merge. Surfaces zero findings OR ≥1 finding; either outcome satisfies AC5 (zero = clean corpus baseline; ≥1 = the gap was real).

## Out of scope

- Building a separate `/style-audit` skill (`/ai-audit` already loads exactly this file set; a separate skill would duplicate the inventory).
- Hook-level (pre-commit) enforcement — no `≥ 3 recurrence` threshold met; current state is zero recorded style-conformance violations, just an architectural gap.
- Auto-fixing detected style violations — Checklist M reports findings; the existing Step 2.5 severity-driven apply-or-ask pattern (`minor` / `nit` may auto-apply if mechanical; `blocker` / `major` requires user confirmation) covers application policy unchanged.
- Refactoring the existing seven Pattern bodies in `ai-docs/agent-writing-style.md` itself (separate concern; that doc is the spec, not a target of this audit).
- Generalising Checklist M to govern files outside the Step 2.2 inventory (Rust source `///` doc comments, `ai-docs/context.md`, archived plans under `done/` / `deferred/`).

## Deferred

- _None._ All deferred concerns from the issue body folded into § Out of scope above (separate `/style-audit` skill, pre-commit hook, auto-fix).

## Key decisions

| Question | Decision |
|---|---|
| Where does Checklist M sit in `.claude/skills/ai-audit/SKILL.md` Step 2.3? | After Checklist L (alphabetical), before Step 2.4. |
| Where does the reciprocal link to `/ai-audit` live in `agent-writing-style.md`? | New `## Enforcement` section between `## Citation in PRs` and `## Out of scope`. Keeps PR-citation guidance separated from audit-venue guidance. |
| Pattern detection corpus (which files Checklist M sweeps) | Exactly the Step 2.2 inventory — AGENTS.md, CLAUDE.md, every `.claude/skills/**/SKILL.md`, every `.claude/agents/**.md`, `ai-docs/{code-style,doc-convention,agent-writing-style,corrections-log}.md`. No expansion to `ai-docs/context.md`, `done/`, or `deferred/`. |
| Severity default per sub-check | Pattern 1 missing action table = `major` (rule states but is unenforceable). Pattern 2 multi-verb paragraph = `minor`. Pattern 3 non-verb-led action column = `minor`. Pattern 4 glob-as-entire-list = `major` (Rule-4 violation has caused real misreads). Pattern 5 mixed OR/AND placement = `nit`. Pattern 6 missing complementary do/not example = `minor`. Pattern 7 invented 4th variant or non-verbatim variant body = `major` (locked-body discipline). Anti-pattern-as-positive-rule = `major`. Design may refine. |
| Pattern 7 audit source-of-truth | `ai-docs/plans/done/2026-05-14-sonnet-skill-reentry-protocol.design.md` (the locked full bodies). Reading the archival design doc as a side-input is in-scope per AGENTS.md and the existing `/ai-audit` read policy. |
| Mutation scope of Checklist M | Unchanged from Phase 2 — proposed edits may touch the same file set the existing A–L checklists touch (AGENTS.md, CLAUDE.md, `.claude/skills/**`, `.claude/agents/**`, `ai-docs/{code-style,doc-convention,agent-writing-style,corrections-log}.md`). Audit reports findings; user approves `blocker` / `major` before apply. |
| AC5 "zero findings" handling | Recorded as `AC5: no drift in current corpus — baseline clean` in the post-merge `/ai-audit phase2` summary. Not a test failure. |

## Technical constraints

- **AGENTS.md size headroom (per AGENTS.md § Build & Test 40,000-char AXIOM):** AGENTS.md currently sits at 39,775 chars / 40,000 hard cap (225-char headroom). AC6 may push AGENTS.md by ≤ 200 chars if a Propagation Rule amendment is made. Design must `wc -c AGENTS.md` after any AGENTS.md edit and abort + extract before commit if it crosses 40,000. The existing 35,000–39,999 warning band already requires a proactive extraction pass on the next `/task` cycle; this task does not have to perform that extraction, but it must not push AGENTS.md *over* 40k.
- **`.claude/skills/ai-audit/SKILL.md` size:** currently 16,927 chars / ~260 lines. Adding Checklist M as prose + grep recipes likely adds 1.5–2.5k chars; total stays well under the 40k cap and under the 500-line Checklist-I soft-limit.
- **`ai-docs/agent-writing-style.md` size:** currently 9,247 chars. Adding a `## Enforcement` section adds ~300–500 chars; total stays well under cap.
- **Grep recipe discipline:** Checklist M's per-pattern detection recipes should be `rg` / `grep -E` / `awk` invocations — no new external tooling — so the existing `/ai-audit` `allowed-tools` frontmatter (`Bash(rg *) Bash(grep *) Bash(awk *) …`) covers the audit without amendment.
- **Pattern detection is heuristic, not semantic.** The audit catches *shape* drift (missing action table, multi-verb paragraph, glob-as-entire-list, non-verbatim Pattern 7 body, anti-pattern-body-as-positive-rule). It cannot judge whether a rule is *correct*; that remains the author's job. Findings phrasing should reflect "Pattern N shape violation: …" not "rule is wrong".
- **Propagation Rule co-evolution:** if Checklist M amends the AGENTS.md `## Propagation Rule` row for `ai-docs/agent-writing-style.md`, the row's downstream consumers (the style guide itself, every `.claude/skills/**`, every `.claude/agents/**`) do not need parallel edits in *this* PR — they are already the targets the existing row points at. Only the AGENTS.md row text and the style guide's `## Enforcement` link are reciprocally edited.
- **Audit re-runs against the changed corpus produced by Checklist M itself must not loop.** Pattern 7's "variant body matches A/B/C verbatim" must be checked by hash/string-equality against the archival doc, not by re-parsing the *currently committed* skill files — otherwise an in-flight edit can mask its own drift.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `.claude/skills/ai-audit/SKILL.md` Step 2.3 contains a new `#### M. agent-writing-style.md conformance` block with the 7 Pattern sub-checks (numbered 1–7 matching Pattern 1–7 in the style guide) plus the 8th Anti-patterns table sub-check. Each sub-check names the detection mechanism (grep/awk recipe shape or text-equality check) and the severity. |
| AC2 | Severity labels in Checklist M's sub-checks come from the existing A–L convention vocabulary (`blocker` / `major` / `minor` / `nit`); no new severity tier is introduced. Default mapping per § Key decisions is applied; design may refine. |
| AC3 | Mutation scope of Checklist M is the same file set Step 2.2 inventories (AGENTS.md, CLAUDE.md, `.claude/skills/**/SKILL.md`, `.claude/agents/**.md`, `ai-docs/{code-style,doc-convention,agent-writing-style,corrections-log}.md`); the existing severity-driven apply-or-ask policy at Step 2.5 applies unchanged. Audit cannot mutate `ai-docs/context.md`, `done/`, `deferred/`, or Rust source. |
| AC4 | `ai-docs/agent-writing-style.md` gains a new `## Enforcement` section (between `## Citation in PRs` and `## Out of scope`) that names `/ai-audit § Checklist M` as the audit venue and links to `.claude/skills/ai-audit/SKILL.md#m-agent-writing-stylemd-conformance` (or the heading slug the design produces). |
| AC5 | A `/ai-audit phase2` invocation against the post-merge corpus completes successfully. If ≥ 1 finding surfaces, the finding is recorded in the merge PR's body or a follow-up issue. If zero findings surface, the result is recorded verbatim as "AC5: no drift in current corpus — baseline clean" (not a failed AC). |
| AC6 | Only if the design phase concludes amendment is required (see § Open questions): AGENTS.md `## Propagation Rule` row for `ai-docs/agent-writing-style.md` is updated to note the `/ai-audit § Checklist M` enforcement path. Resulting AGENTS.md char count is verified `< 40,000` via `wc -c` before commit. If the design concludes no amendment is needed, AC6 is recorded as "no amendment — existing row already names the enforcement surface via Propagation Rule fan-out and AC4's reciprocal link". |
| AC7 | `actionlint` is not relevant (no `.github/workflows/*.yml` touched). `cargo build` / `cargo test` / `cargo clippy --workspace -- -D warnings` are not relevant (no `.rs` touched). No quality-gate regression introduced (no source changes). |

## Open questions

- **AC6 amendment scope.** Should AGENTS.md `## Propagation Rule` row for `ai-docs/agent-writing-style.md` be amended to reference `/ai-audit § Checklist M` explicitly?
  - **Default the design agent should take if not revisited:** *No amendment.* The existing row names the fan-out target on edits-to-the-style-guide. The audit is the reverse direction (catches drift in downstream consumers), which is captured by AC4's reciprocal link from the style guide itself. Amending the AGENTS.md row would consume ~150–200 of the 225-char headroom for a redundant pointer.
  - **Conditions under which the user might want to revisit:** if the project later adopts a convention that every audit-enforced rule must be discoverable from the AGENTS.md Propagation Rule table, the row gains a short suffix. Not blocking design.
- **Pattern 2 verb set lock-in.** The Pattern 2 detection recipe must enumerate the bold-uppercase verb set explicitly. The style guide's § Pattern 2 lists `**NEVER**`, `**MUST NOT**`, `**FORBIDDEN**`, `**ALWAYS**`, `**MUST**`, `**STOP**`, `**REJECT**`, `**REMOVE**`, `**REPLACE**`, `**DELETE**`. Whether to extend that set when new fail-loud verbs appear in instruction files (e.g., `**HALT**`, `**ABORT**`) is a future-style-guide decision, not a Checklist M decision. Design takes the current verb set as-is.
- **Pattern 6 "non-trivial rule" detector.** Pattern 6's "for non-trivial rules" is judgement-laden; a fully mechanical detector is impossible. Design should pick a heuristic — e.g., "if the fail-loud body mentions both 'this' and 'instead' / 'not' / 'do' / 'NOT' in close proximity AND no code block follows" — or downgrade Pattern 6 severity to `nit` if the heuristic produces too much noise on first run. Recorded for design rather than user.
