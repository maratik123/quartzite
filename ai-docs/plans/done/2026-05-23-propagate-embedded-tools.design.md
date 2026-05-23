# Design: Propagate embedded tools to harnesses

**Issue:** #549
**Date:** 2026-05-23

> **POST-MERGE AMENDMENT (PR #550 round 1).** Class C subtasks (4–7) and Class D subtasks D3/D4 were **rolled back**; runtime probe on 2026-05-23 confirmed the harness elides `Agent` from a Subagent's runtime tool list (mirrors `ai-docs/learnings.md` 2026-05-15 `self-improve` finding). See `done/2026-05-23-propagate-embedded-tools.spec.md` post-merge note. The per-AC verification mapping below for AC6/AC7/AC8/AC9/AC11 is obsoleted; remaining ACs (1–5, 10, 12–16) stand.

## Approach

The spec already enumerates six concrete classes of edit (A–F) with per-site
"Today → After" tables, the round-1/2/3 decisions table, the technical
constraints block, and the 16 ACs. Class E (audit-only) and Class F
(`Bash`-retention) deliver as prose tables already living inside the spec
itself — no Subagent-file edits this PR for those two classes.

The design's job, given how concrete the spec is, is therefore narrow:

1. **Per-AC verification mapping** — one `AC<N> verified by: <command>` line
   per AC, so Step 11 self-review and Step 11.5 review-findings can grep
   each AC mechanically without rederiving the shell from the spec body.
2. **Decompose Classes A–D into atomic implementation subtasks** sized to
   the every-group handoff contract (3 / 3 / … / 1..=3).
3. **Pre-compute the `## Handoff plan` group boundaries** so /task Step 8
   reads them rather than re-deriving per turn.
4. **Enumerate risks** (URL availability, Explore exclusion-list, prompt
   shape for `claude-code-guide`).
5. **Record Propagation Rule firings** Class D (Tools/Subagents anchor
   row → already discharged by Class D itself, AC11+AC12 verify) and
   Class C (Interview group → no-op, AC13 verifies byte-for-byte).
6. **Sequencing.** Classes A / B / C / D are independent for editing
   purposes (different files); D3/D4 *report* the post-rewrite state of
   C1/C3, but Class D rows can be written from the spec's planned values
   without waiting on C to land. Group ordering in the Handoff plan is
   chosen for thematic locality (A together, B together, C together, D
   together) rather than dependency — there is no cross-class blocker.

**Rejected alternatives:**

- *Land Class E rewrites in this PR.* Spec round 3 promoted Class E to
  audit-only explicitly. All six landed ✅ / ⛔; zero 🟡 candidates means
  no rewrite payload exists.
- *Drop `Bash` from `design-review` / `spec-writer` "while we're here".*
  Spec F1/F2 explicitly keep `Bash`; F2 requires it for the Rule-5
  mechanical-check (`printf … | grep -iE …` — entry C5), the rewritten
  C4 (`grep -nE … AGENTS.md`), and `gh issue view`; F1 keeps it for
  `cargo check` / `cargo doc` / `gh pr view` shell-outs. Dropping
  `Bash` would either re-spawn nested for every shell-out (cost) or
  break C5 outright (breakage). Same-PR scope rejects both.
- *Spawn `Explore` for the Class C4 single-pattern `Bash grep` on
  AGENTS.md.* Spec key-decisions row applies the same rationale that
  keeps `triage-runner` Phase 4.5's bounded one-file Read inline: one
  shell-grep against a known path is lighter than a nested `Explore`
  spawn.
- *Persist `claude-code-guide` output to a temp file under `ai-docs/`.*
  Spec key-decisions row inlines the spawn output into orchestrator
  context the same way today's `WebFetch` results are consumed; no
  parallel-fallback retained per Q2 ("Replace").

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Rewrite `.claude/skills/ai-audit/SKILL.md` Phase 2 (Step 2.1) to spawn `claude-code-guide` via the `Agent` Tool instead of inline `WebFetch`. Rewrite all four sites in spec entry A1: line 66 ("Fetch the three primary references via WebFetch …"), lines 68–70 (the three bullet URLs become the spawn `prompt`'s URL list), line 74 ("fetch additional pages from `code.claude.com` on demand" → "ask `claude-code-guide` for additional pages from `code.claude.com` on demand"). The spawn `prompt` cites the three URLs verbatim and asks for the verbatim shape contracts for skill frontmatter / Subagent file structure / Hook event matchers. Returned text is inlined into orchestrator context for the rest of the run (no temp-file persistence). | `.claude/skills/ai-audit/SKILL.md` | — |
| 2 | Rewrite `.claude/skills/ai-audit/SKILL.md` Anti-patterns line 186 (spec entry A2): replace `Do **not** skip the `WebFetch` Tool call in Phase 2.` with `Do **not** skip the `claude-code-guide` spawn in Phase 2.` Same prohibition, updated to point at the new primitive. | `.claude/skills/ai-audit/SKILL.md` | 1 |
| 3 | Rewrite `.claude/skills/bugfix/SKILL.md` Step 1 (spec entry B1): the "Using only read-only tools (`rg`, `Read`, logs, test output), trace the actual execution path" guidance line becomes an `Agent` spawn of `Explore`. Spawn `prompt` instructs `Explore` to trace the execution path, draw the ASCII diagram per the existing template (preserved in the body for the orchestrator to fill from `Explore`'s output), and return both the diagram AND the file:line citations supporting each arrow. The `prompt` MUST include the verbatim `ast-index.md § Rules for subagents` block (per the existing rule in `.claude/rules/ast-index.md`). Step 1's other inner steps (artefact creation, `current_step` tracking, user confirmation) remain unchanged — `Explore`'s output feeds into the existing trace template. | `.claude/skills/bugfix/SKILL.md` | — |
| 4 | Rewrite `.claude/agents/design-review.md` frontmatter (spec entry C1): `tools: Read, Grep, Glob, Bash` → `tools: Read, Bash, Agent`. Drop `Grep` + `Glob`; add `Agent`; retain `Read` (known-path inline reads per the spec's "default applied silently" key-decision row) and `Bash` (per Class F1: `cargo` / `gh pr view` shell-outs). | `.claude/agents/design-review.md` | — |
| 5 | Rewrite `.claude/agents/design-review.md` body line 20 (spec entry C2): `Every suspicion — **investigate via Read/grep**, don't guess and don't give benefit of the doubt.` → `Every suspicion — investigate via `Read` for known paths, or spawn `Explore` via `Agent` for code search across the workspace; don't guess and don't give benefit of the doubt.` The `Explore` spawn `prompt` MUST include the verbatim `ast-index.md § Rules for subagents` block (per the existing rule). | `.claude/agents/design-review.md` | 4 |
| 6 | Rewrite `.claude/agents/spec-writer.md` frontmatter (spec entry C3): `tools: Read, Write, Edit, Grep, Glob, Bash` → `tools: Read, Write, Edit, Bash, Agent`. Drop `Grep` + `Glob`; add `Agent`; retain `Read`, `Write`, `Edit`, `Bash` (per Class F2 mandatory-retention rationale: Rule-5 mechanical-check `printf … \| grep -iE …` block (entry C5), Class C4 `grep -nE … AGENTS.md` rewrite, and `gh issue view` lookups). | `.claude/agents/spec-writer.md` | — |
| 7 | Rewrite `.claude/agents/spec-writer.md` body line 30 (spec entry C4): replace `Use \`Grep\` against AGENTS.md for any rule that might affect the spec under consideration.` with a `Bash` `grep` invocation (`grep -nE '<keyword>' AGENTS.md` against the pre-resolved-rules table). AGENTS.md is a known path; `Bash` `grep` is lighter than spawning `Explore` for a single-file regex search and parity with C5's existing pattern. Spec entry C5 (the Rule-5 substring blacklist `printf '%s\n' … \| grep -iE …` shell snippet) is byte-for-byte unchanged — AC10 verifies. | `.claude/agents/spec-writer.md` | 6 |
| 8 | Add Class D rows to `ai-docs/claude-tools-hierarchy.md` §1a (spec entries D1, D2) + sync §2b rows for `design-review` and `spec-writer` (spec entries D3, D4). D1: `Grep` row in §1a with 🟦 origin, purpose "Regex search across files", parameters `pattern`, `path`, `glob`, `-i`, `-n`, `-l`, `-A`/`-B`/`-C`, `output_mode`, `type`, `head_limit`, `multiline`. D2: `Glob` row in §1a with 🟦 origin, purpose "Fast filename pattern matching", parameters `pattern`, `path`. D3: §2b `design-review` row `Tools` column from `Read`, `Grep`, `Glob`, `Bash` → `Read`, `Bash`, `Agent`. D4: §2b `spec-writer` row `Tools` column from `Read`, `Write`, `Edit`, `Grep`, `Glob`, `Bash` → `Read`, `Write`, `Edit`, `Bash`, `Agent`. Preserve existing column order; no row reordering; no stable-anchor changes. | `ai-docs/claude-tools-hierarchy.md` | — |
| 9 | Verify Propagation-Rule fan-out and char-budget gate. (a) Interview group no-op: confirm AC13's two checks fire on the post-edit tree — `! grep -qE 'backward.compat\|compat.shim\|deprecat\|legacy' .claude/skills/interview/SKILL.md` exits 0 (blacklist tokens stay absent — the blacklist is **not** mirrored into `interview/SKILL.md` by design, per its own anti-pattern note) AND `git diff --name-only $(git merge-base HEAD master)..HEAD -- .claude/skills/interview/SKILL.md` produces no output (file untouched by this PR). (b) Stable anchors intact in `ai-docs/claude-tools-hierarchy.md`: `grep -E 'stable-anchor: #(1-tools\|2-subagents\|3-skills\|4-hooks\|mental-model\|stable-anchors)' ai-docs/claude-tools-hierarchy.md` returns 6 matches (AC12). (c) Char budget: `wc -c` each touched instruction file — every count < 40 000 (AC14). (d) Walk every AC1–AC16 verification command in the spec — confirm each fires the expected outcome on the post-edit tree. Surface any failure as a blocker before commit. | (verification-only; no file edits) | 1, 2, 3, 4, 5, 6, 7, 8 |

## Handoff plan

M = 9 (three groups, 3 + 3 + 3).

- **Group A:** subtasks 1–3 — Class A (ai-audit Phase 2 + Anti-pattern note) + Class B (bugfix Step 1) initial chunk. Non-terminal (exactly 3).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent /task resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — Class C frontmatter + body line 20 rewrite of `design-review`, plus Class C frontmatter rewrite of `spec-writer`. Non-terminal (exactly 3).
- **Handoff after Group B:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent /task resumes in Group C with fresh context.
- **Group C:** subtasks 7–9 — `spec-writer` body line 30 rewrite (C4), Class D hierarchy-doc edits (D1–D4), and the propagation/char-budget/AC-walk verification subtask. Terminal group (3 subtasks; within the 1..=3 range).

The first group's entry also runs under a `/context-reset` subagent per the every-group handoff contract (`.claude/skills/task/SKILL.md` Step 8 + `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry)).

## Per-AC verification mapping

| AC | Verification command |
|---|----------------------|
| AC1 | `grep -E 'WebFetch' .claude/skills/ai-audit/SKILL.md` returns no matches AND `grep -E 'claude-code-guide' .claude/skills/ai-audit/SKILL.md` returns ≥ 1 match |
| AC2 | `grep -F 'Do **not** skip the \`claude-code-guide\`' .claude/skills/ai-audit/SKILL.md` matches |
| AC3 | `grep -E 'subagent_type.*Explore' .claude/skills/bugfix/SKILL.md` matches AND `grep -F 'ast-index search' .claude/skills/bugfix/SKILL.md` matches |
| AC4 | `grep -E '^\| \`Grep\` \| 🟦' ai-docs/claude-tools-hierarchy.md` matches |
| AC5 | `grep -E '^\| \`Glob\` \| 🟦' ai-docs/claude-tools-hierarchy.md` matches |
| AC6 | `head -10 .claude/agents/design-review.md \| grep -E '^tools:'` shows `Read`, `Bash`, `Agent` (order tolerant) AND does NOT match `Grep` or `Glob` |
| AC7 | `head -10 .claude/agents/spec-writer.md \| grep -E '^tools:'` shows `Read`, `Write`, `Edit`, `Bash`, `Agent` (order tolerant) AND does NOT match `Grep` or `Glob` |
| AC8 | `grep -F 'investigate via Read/grep' .claude/agents/design-review.md` returns no matches AND `grep -E 'spawn \`Explore\`' .claude/agents/design-review.md` matches |
| AC9 | `grep -F 'Use \`Grep\` against AGENTS.md' .claude/agents/spec-writer.md` returns no matches AND `grep -E 'grep -[nE].* AGENTS.md' .claude/agents/spec-writer.md` returns ≥ 1 match |
| AC10 | `grep -F "printf '%s\n'" .claude/agents/spec-writer.md` matches AND `grep -F "backward.compat" .claude/agents/spec-writer.md` matches |
| AC11 | `grep -E '^\| \`design-review\` \|' ai-docs/claude-tools-hierarchy.md` shows `Read`, `Bash`, `Agent` and not `Grep`/`Glob` AND `grep -E '^\| \`spec-writer\` \|' ai-docs/claude-tools-hierarchy.md` shows `Read`, `Write`, `Edit`, `Bash`, `Agent` and not `Grep`/`Glob` |
| AC12 | `grep -E 'stable-anchor: #(1-tools\|2-subagents\|3-skills\|4-hooks\|mental-model\|stable-anchors)' ai-docs/claude-tools-hierarchy.md` returns 6 matches |
| AC13 | `! grep -qE 'backward.compat\|compat.shim\|deprecat\|legacy' .claude/skills/interview/SKILL.md` exits 0 (blacklist tokens stay absent) AND `git diff --name-only $(git merge-base HEAD master)..HEAD -- .claude/skills/interview/SKILL.md` produces no output (file untouched in this PR) |
| AC14 | `wc -c .claude/skills/ai-audit/SKILL.md .claude/skills/bugfix/SKILL.md .claude/agents/design-review.md .claude/agents/spec-writer.md ai-docs/claude-tools-hierarchy.md` — every line's char count < 40000 |
| AC15 | `grep -cE '^\| E[1-6] \|' ai-docs/plans/2026-05-23-propagate-embedded-tools.spec.md` returns `6` AND every row carries one of ✅ / 🟡 / ⛔ |
| AC16 | `grep -cE '^\| F[1-3] \|' ai-docs/plans/2026-05-23-propagate-embedded-tools.spec.md` returns `3` AND rows F1 / F2 explicitly state "Keep \`Bash\`" with a justification line each |

AC15 and AC16 are no-ops for the implementation phase — the spec already
ships with the E and F tables in place; the ACs verify they survived the
PR untouched. Subtask 9 walks all 16 ACs including these two.

## Affected files

| Path | Class(es) | Subtasks |
|------|-----------|----------|
| `.claude/skills/ai-audit/SKILL.md` | A1, A2 | 1, 2 |
| `.claude/skills/bugfix/SKILL.md` | B1 | 3 |
| `.claude/agents/design-review.md` | C1, C2 | 4, 5 |
| `.claude/agents/spec-writer.md` | C3, C4 (+ C5 untouched) | 6, 7 |
| `ai-docs/claude-tools-hierarchy.md` | D1, D2, D3, D4 | 8 |
| `ai-docs/plans/2026-05-23-propagate-embedded-tools.spec.md` | E1–E6, F1–F3 (already in spec — no edit this PR) | — (AC15, AC16 verify presence only) |
| `.claude/skills/interview/SKILL.md` | (Interview group no-op) | — (AC13 verifies untouched diff) |

All edits stay below the 40 000-char per-file cap (current sizes: ai-audit
13.6 KB, bugfix 14.1 KB, design-review 6.9 KB, spec-writer 16.1 KB,
claude-tools-hierarchy 10.8 KB — net change per file is bounded by a few
hundred chars; subtask 9 verifies via AC14).

## Propagation Rule firings

Per AGENTS.md § Propagation Rule:

1. **Tools / Subagents / Skills / Hooks anchor row.** Class C changes the
   `tools:` contract of two project Subagents (`design-review`,
   `spec-writer`). The anchor row requires `ai-docs/claude-tools-hierarchy.md`
   to be updated in the same PR — Class D IS that update. D3/D4 sync §2b
   `Tools` columns with the C1/C3 frontmatter rewrite; D1/D2 add the
   §1a embedded-tool rows for `Grep` and `Glob` (rows were missing despite
   both Subagents' pre-rewrite `tools:` frontmatter citing them; same-PR
   land documents the embedded tool regardless of the drop). AC11 + AC12
   verify (§2b rows match post-rewrite frontmatter; stable anchors intact).

2. **Interview group row.** Class C edits `.claude/agents/spec-writer.md`;
   the row requires `.claude/skills/interview/SKILL.md` to be checked
   and updated in the same PR. **The check is a no-op** — Class C
   touches only the `tools:` line + body line 30 of `spec-writer.md`;
   entry C5 (the Rule-5 substring blacklist `printf … \| grep -iE …`
   block) is byte-for-byte unchanged, and `interview/SKILL.md` does
   not mirror the blacklist (per its own anti-pattern note forbidding
   the inline) so no edit is required. AC13 verifies by **negation
   plus untouched-file diff**: `! grep -qE 'backward.compat\|compat.shim\|deprecat\|legacy'`
   confirms the blacklist tokens stay absent from `interview/SKILL.md`,
   and `git diff --name-only <merge-base>..HEAD -- .claude/skills/interview/SKILL.md`
   produces no output (the file is untouched in this PR).

3. **No other group fires.** Classes A, B (orchestrator Skill rewrites)
   do not change any Subagent / Tool / Skill / Hook contract — they
   consume existing contracts (`Agent` Tool calls on already-documented
   embedded Subagents). Class E is audit-only inside this spec; none of
   the six audited Subagent files are edited, so the Review group
   (`self-review` / `review-findings`), Triage group (`triage-runner`),
   Learning-Log group (`self-improve` / `learnings-escalation-audit`),
   and Task/Design group (`design`) all stay quiescent.

4. **Embedded-name clash AXIOM.** This task is the *dual* direction —
   dropping project simulations in favour of embedded primitives. No
   project rename is triggered. `Explore`, `claude-code-guide`, `Agent`,
   `Grep`, `Glob` are all embedded; no project name collides post-rewrite.

## Sequencing

No cross-class blocker exists. Classes A, B, C, D touch disjoint files
(except Class C splits across two `.claude/agents/*.md` files); D3/D4
*report* C1/C3's post-rewrite state but can be written from the spec's
planned values without waiting on C to land. The Handoff plan's group
ordering is chosen for thematic locality (A+B / C / D+verify), not
sequencing necessity. Subtask 9 (verification) is the only true
dependent — it must run after every edit subtask, which is why it lands
in the terminal group.

Within each class, ordering matters slightly:

- Class A: subtask 1 (Step 2.1 rewrite — Phase 2 body) → subtask 2
  (Anti-patterns line 186 — the prohibition line must point at the
  primitive subtask 1 just installed).
- Class B: single subtask (3) — no internal ordering.
- Class C / design-review: subtask 4 (`tools:` frontmatter drop) →
  subtask 5 (body line 20 rewrite — the directive must reference the
  `Agent`-spawned `Explore` that subtask 4's frontmatter authorises).
- Class C / spec-writer: subtask 6 (`tools:` frontmatter drop) →
  subtask 7 (body line 30 rewrite — the directive must shell out via
  the `Bash` retained in subtask 6's frontmatter).
- Class D: single subtask (8) — four rows, but mechanically one file
  edit.

## Risks

- **`code.claude.com` URL availability.** Class A1's `claude-code-guide`
  spawn `prompt` cites three URLs (`/docs/en/{skills,sub-agents,hooks-guide}`).
  Spec § Technical constraints explicitly opts out of re-validating
  them during this task — if any 404 in practice, `claude-code-guide`
  surfaces that and the failure is treated as a docs-publisher change,
  not a regression of this task. *Mitigation:* AC1 verifies the
  `WebFetch` references are gone and the spawn is wired; URL
  availability is `claude-code-guide`'s problem at run time, not this
  PR's.

- **`Explore` exclusion-list contract for the `/bugfix` Step 1 spawn.**
  Per `claude-tools-hierarchy.md` §2a, `Explore`'s tools are "All except
  `Agent`, `Edit`, `Write`, `NotebookEdit`, `ExitPlanMode`" — so it
  cannot nest a further `Agent` spawn, cannot edit files, and cannot
  write artefacts. Step 1's artefact creation (the `trace-*.md` file)
  is orchestrator-side, NOT in the spawn — `Explore` returns the
  diagram + citations, orchestrator writes the trace. *Mitigation:*
  subtask 3's prompt shape MUST preserve the existing inner-steps
  ordering (artefact creation, `current_step` tracking, user
  confirmation) in the orchestrator, with `Explore` returning only the
  diagram text. The trace-file template stays in the body of
  `bugfix/SKILL.md` for the orchestrator to fill from `Explore`'s
  output.

- **`ast-index.md § Rules for subagents` verbatim block.** Classes B and
  C spawn `Explore` from instruction surfaces; the spawn `prompt` MUST
  include the verbatim block (per `.claude/rules/ast-index.md`).
  *Mitigation:* subtasks 3 and 5 must each embed the verbatim block in
  the spawn `prompt`. AC3 specifically checks `grep -F 'ast-index
  search' .claude/skills/bugfix/SKILL.md` matches (the verbatim block
  contains this command); no equivalent grep exists for
  `design-review.md`'s body line 20, so reviewer attention there falls
  on Step 11 self-review.

- **`design-review.md` GO-with-notes pre-Step-8 round-trip.** Body's
  GO-verdict block currently mandates a round-trip of GO-with-notes
  back into the design doc. Subtask 5 rewrites line 20 only — the
  GO-with-notes contract on lines 64–68 is untouched and continues to
  fire if this very design's review emits notes.

- **`spec-writer.md` Rule-5 mechanical-check block (entry C5)
  byte-for-byte integrity.** Subtask 7 rewrites body line 30, which is
  ~90 lines before the Rule-5 block (line 121 onward in the current
  file). The two edits don't overlap, but Edit must use a unique
  `old_string` for line 30 — the `printf '%s\n'` block on line 128
  contains its own `grep -iE` substring. *Mitigation:* AC10's
  twin `grep -F` checks (`"printf '%s\n'"` for the snippet anchor
  AND `"backward.compat"` for the surrounding blacklist literal)
  catch any accidental damage; subtask 9 walks AC10 explicitly.

- **40 000-char cap on instruction files.** Net change per file is
  bounded by a few hundred chars (each subtask is a frontmatter line
  swap, a body line replacement, or a tablerow addition); current sizes
  are 6.9–16.1 KB. *Mitigation:* AC14's `wc -c` walk in subtask 9.

- **Class E future-PR conditional re-fire.** Spec § Deferred lists one
  conditional: "if a future Class E re-run flips any verdict to 🟡, the
  rewrite is its own PR per the user's round-3 promotion directive".
  This PR codifies the current verdicts (all ✅ / ⛔); no follow-up issue
  is created at merge time. *Mitigation:* no design action — the Class
  E table inside the spec is the durable record; a future
  reclassification re-fires Class E from scratch.

## Test Design

Instruction-surface PR — no Rust code changes — therefore no
`#[cfg(test)]` modules or `tests/` files. Verification is grep-based per
AC; subtask 9 walks every AC verifier on the post-edit tree.

Per non-trivial subtask, the verification entry point is the AC mapping
table above:

- Subtask 1 (Class A1, Phase 2 rewrite): AC1.
- Subtask 2 (Class A2, Anti-patterns line): AC2.
- Subtask 3 (Class B1, bugfix Step 1): AC3.
- Subtask 4 (Class C1, design-review frontmatter): AC6.
- Subtask 5 (Class C2, design-review body line 20): AC8.
- Subtask 6 (Class C3, spec-writer frontmatter): AC7.
- Subtask 7 (Class C4, spec-writer body line 30): AC9 + AC10.
- Subtask 8 (Class D1–D4, hierarchy-doc edits): AC4 + AC5 + AC11 + AC12.
- Subtask 9 (verification): AC12 + AC13 + AC14 + AC15 + AC16 + full
  re-walk of AC1–AC11.

No fixtures or helpers needed — every verifier is a `grep` / `wc` /
`diff` over the working tree. Test "scenarios" per AC reduce to: does the
mandated string match (positive) AND does the forbidden string not match
(negative). AC10's "byte-for-byte unchanged" check pairs the positive
match of `printf '%s\\\\n'` with the negative check that the surrounding
blacklist literal is preserved.

## Open questions

_(none — spec rounds 1–3 resolved every design-affecting question; spec
§ Open questions confirms.)_
