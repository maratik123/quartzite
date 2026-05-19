# Design: Carrot Pass for `/improve` and `/ai-audit` — PR 2 (Phases 2 + 3 + 4 + AC12)

**Issue:** #491
**Spec:** `ai-docs/plans/2026-05-19-carrot-pass-improve-ai-audit.spec.md`
**Date:** 2026-05-19
**Prior PR (merged):** #492 — Phase 1 (schema migration, section rename to *Learning Log*, worked-example retro-tag). Phase-1 design preserved at `ai-docs/plans/2026-05-19-carrot-pass-improve-ai-audit.phase1.design.md`.

## Approach

PR-2 finishes the asymmetric Carrot/Stick harness on top of the Phase-1 schema. Four phases co-land in one PR (they cite each other: Phase 4's threshold names Phase 3's verdict; Phase 3's audit grows around Phase 2's verbs; AC12 demonstrates Phase 2's flow end-to-end on a real entry). Phase 5 stays as PR-3 — its consent-UX surface is independent from Phases 2–4.

The Carrot pass is added **alongside** the existing Correction pass in `.claude/agents/self-improve.md` (parallel — same Step-1 walk, separate Step-2 routing tables, shared apply/eval skeleton with an inverted prompt). The audit gains one new verdict (`🌱 Stale-validation`), one new Checklist row (L gets `Kind:`), one Checklist sub-check (M's cross-shape verb sweep), and one new Checklist (N — bidirectional `## Patterns` ↔ `Kind: validation`). The threshold reframe lands in lock-step in `AGENTS.md` and `.claude/skills/improve/SKILL.md`. AC12 is the worked-example demonstrator — a `## Patterns` block lands in `.claude/skills/context-reset/SKILL.md` (the most-local skill for the 2026-05-19 *compaction-recovery protocol in skill files works* entry; the entry's `Rule:` names `/task`, `/pr-commented`, `/code-review`, `/bugfix`, `/interview` as the recovery-protocol surfaces, but `/context-reset` is the canonical-rationale home and Variant-C parent — back-linking there places the pattern at the singular skill that owns the cross-link target).

### Chosen alternatives

**Carrot pass as parallel Step (not interleaved).** Two parallel passes — Correction and Carrot — each with its own Step 1 grouping rule (Correction groups by `Kind: correction` ⊕ omitted; Carrot groups by `Kind: validation`) and its own Step 2 routing table. They share Steps 3–5 skeleton (Propose / Hooks / Apply) and Step 6 with a **branching prompt** (correction = "does the violation still happen?"; carrot = "does the pattern still hold under edge case X?"). Rejected: a single fused pass that switches on `Kind` per-entry — couples reasoning across signal directions and makes the routing table illegible.

**Carrot pass: dedicated `### Step 1b: Carrot pass` section.** Inserted directly after the existing routing table in Step 2, NOT as a new Step 7. Reason: Steps 3 (Propose) / 4 (Hooks) / 5 (Apply) / 6 (Eval) all need to handle BOTH passes; a sequential Step-7 would force Step 6 to re-fire after Step-7 finishes, breaking the existing pause-and-surface protocol. Inserting the Carrot pass at Step 2 keeps it parallel — the Step-2 routing tables fork; Steps 3+ converge.

**Step 6 inverted-eval as a branching prompt, not a separate Step 6b.** The reproducer template gains a `correction | carrot` parameter selecting one of two `**Scenario:**` line forms. Same pause-and-surface protocol, same `Eval: PASS ✅` / `Eval: FAIL ❌` parent-thread emission. Rejected: separate Step 6b — duplicates the pause-and-surface contract (which already has documented primitive-absence semantics), forcing every future eval-protocol edit to fan out.

**Verb-set enumeration: dedicated `## Promotion verbs` section near the routing tables.** Block-level enumeration (not inline in prose) so the audit's Checklist-M sub-check can target a stable anchor (`#promotion-verbs`). Rejected: inline mention scattered through Step 2 prose — harder to audit, easier to drift.

**`🌱 Stale-validation` verdict: 30-day age + `Escalated? no` + ≥1 instruction-file commit since validation date.** All three conjuncts required. The age threshold matches the spec's Phase 3 body verbatim. The instruction-file-commit clause uses `git log --since=<validation-date> -- AGENTS.md ai-docs/ .claude/` (constrained to the *audited corpus*, not the whole tree — keeps the signal high-quality). Rejected: only age + `no` (too noisy — any unescalated validation auto-flags after 30 days); only ≥1 commit + `no` (no time-decay — validations escalated weeks after writing don't surface).

**Checklist L `Kind:` row — 4-location selection (spec-leaving-open reconciliation).** Spec AC7 wording says "Checklist L grows a `Kind:` row enforcing 4-location coverage"; the 4 locations are not enumerated in AC7. Spec Phase 3 body explains the *existing* Checklist L's 4 locations (for `Escalated?` / `Superseded by:`) and says the same coverage discipline must apply to `Kind:`. The reconciliation chosen here is: `Kind:` is a declared-schema field with no `/improve`-time mutation; the Exception-body 4-location list for `Escalated?` / `Superseded by:` does not literally apply (Boundary Rule 1+2 Exception bodies cover `/improve`-mutated fields, which `Kind:` is not). The Checklist L `Kind:` row uses an analogous 4-location list — entry-format declaration site (AGENTS.md `## Learning Log` Entry-format block) + each backfill/parse site (`self-improve.md` Step 5 Commit-B backfill / `learnings-escalation-audit.md` Steps 2/3/4 parse / `ai-docs/corrections-log.md` field glossary). Spec AC7's "4-location coverage" wording is preserved; the location selection is an implementation choice the spec leaves open. This is NOT a spec amendment — same Checklist L PATTERN of cross-location tracking, applied to a different relevant set. Subtask 4's body carries the four locations verbatim.

**Checklist N (carrot-side bidirectional analog of Checklist C).** Severity `major` per spec; mirrors C's "every reference resolves AND every target has a back-reference" shape. Detection: a forward sweep grepping every `## Patterns` section in skills/agents/AGENTS.md for `learnings.md` anchor links AND a reverse sweep walking every `Kind: validation` entry whose `Escalated?` ≠ `no` and confirming the named target contains a `## Patterns` block. Rejected: one-way Checklist (only `## Patterns` → `learnings.md`) — the asymmetry would let escalated validations rot without back-links and the audit would never catch it.

**Checklist M sub-check 11 (cross-shape verbs).** Severity `major` per spec. Added as sub-check 11 (current max is 10). Detection: in every audited corpus file, find `## Patterns` sections AND grep their bodies for `**MUST**` / `**NEVER**` / `**MUST NOT**` / `**FORBIDDEN**` (carrot-shaped rule with stick verb = flag); separately, walk every fail-loud AXIOM blockquote outside `## Patterns` sections and grep for `Default to` / `Prefer` (stick-shaped rule with carrot verb = flag). Both directions flagged at the same severity per spec. Rejected: a single-direction sub-check — the cross-shape asymmetry only protects half the contract.

**Sub-check 10 coverage-map update.** `agent-writing-style.md`'s `## Patterns` heading maps to "sub-checks 1–7" in the current map. After PR-2, Checklist M has sub-check 11; the map needs no edit — Pattern entries themselves are still audited by sub-checks 1–7, and sub-check 11 audits **carrier files** (skills/agents/AGENTS.md), not the style-guide itself. Verified explicitly to avoid a phantom coverage-gap finding.

**AC12 home: `.claude/skills/context-reset/SKILL.md`.** Three candidates were considered:

| Candidate | Pro | Con | Decision |
|---|---|---|---|
| `.claude/skills/context-reset/SKILL.md` | Owns the canonical rationale (`## Compaction recovery (re-entry)` cross-link target); Variant-C parent; AGENTS.md's Propagation Rule already names it as the context-reset sync-group member | None — singular natural home | **Chosen** |
| `.claude/skills/task/SKILL.md` | One of the orchestrating skills the validation entry names | Validation isn't `/task`-specific — would arbitrarily privilege one orchestrator | Rejected |
| Spread across all six callout-carrying skills (`/task`, `/pr-commented`, `/code-review`, `/bugfix`, `/interview`, `/context-reset`) | Maximum coverage | Six near-identical `## Patterns` blocks; pattern-promotion churn on every validation | Rejected — `/context-reset`'s ownership of the canonical rationale is the singular point |

**`## Patterns` block shape — mirrors `ai-docs/agent-writing-style.md § Patterns`.** Heading-level discipline: `## Patterns` (level 2) → `### N. <Name>` (level 3, numbered). Per-entry body: one-sentence rule (carrot-shaped using *Default to* / *Prefer* verbs), back-link to the validation entry, and a short "why" or "what to do" paragraph. Back-link form: `See [`ai-docs/learnings.md`](../../../ai-docs/learnings.md) 2026-05-19 *compaction-recovery protocol in skill files works*` (path-relative + date-slug — same style as existing cross-references in skill files; anchor links are fragile since `learnings.md` headings auto-collide on shared dates, so a path + date + quoted-slug citation is the durable form).

**AGENTS.md threshold reframe — char-cap impact.** AGENTS.md is at 37,564 chars post-PR-1. The threshold line rewrite adds ~150 chars (line 336: 51 chars → ~200 chars). Net post-PR-2 AGENTS.md: ~37,714 chars — still in the 35,000–39,999 early-warning band, but no breach of the 40,000 hard cap. No extraction needed in this PR. Recorded as a Risk row.

**Phase 5 NOT folded into PR-2.** Per the spec's PR-slicing key decision, the design agent may recommend folding Phase 5 if the implementation diff is small. Phase 5's consent-UX (AskUserQuestion shape; opt-in flag in `~/.claude/projects/.../memory/` or `.claude/settings.local.json`) is non-trivial to design AND has a privacy boundary that warrants its own design-review pass. Recommend: keep Phase 5 as PR-3. AC17 satisfied by this design recommendation being recorded here.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | **Phase 2a — Carrot pass body in `self-improve.md`.** Insert a `### Step 1b — Carrot pass` section directly after the existing Step 2 (Determine actions) routing table. The Carrot section contains (a) Step 1 grouping rule for `Kind: validation` entries, (b) the asymmetric routing table (1 → seed `## Patterns` entry + back-link; ≥2 → promote within the same `## Patterns` section using stronger verb wording; 1 + workflow-primitive → hold for second confirmation), (c) explicit prose naming the most-local-target routing (skill > agent > AGENTS.md). Header the existing Step 2 as `### Step 2a — Correction pass routing` for symmetry. Carrot routing table goes immediately after as `### Step 2b — Carrot pass routing`. | `.claude/agents/self-improve.md` | — |
| 2 | **Phase 2b — Promotion-verb enumeration + Step 6 inverted-eval prompt in `self-improve.md`.** Add a new `## Promotion verbs` section near the routing tables (between Step 2b and Step 3): two tables — *Carrot promotion verbs* (`Default to` / `Prefer`) and *Stick promotion verbs* (`MUST` / `NEVER` / `MUST NOT` / `FORBIDDEN`) — with one-sentence rationale. Update Step 6's reproducer template skeleton to carry both `Scenario:` line variants (Correction: *"You are about to violate rule X — what's the expected behaviour?"*; Carrot: *"In scenario X (edge case from the original learning's surface), does pattern P still hold?"*). PASS / FAIL criteria split per direction. | `.claude/agents/self-improve.md` | 1 |
| 3 | **Phase 3a — `🌱 Stale-validation` verdict in `learnings-escalation-audit.md`.** Add `🌱 Stale-validation` to the verdict set in Step 2 (immediately after the existing `❓ Ambiguous` row). Document the trigger: `Kind: validation` entry whose entry-date is > 30 days old AND `Escalated? no` AND `git log --since=<entry-date> -- AGENTS.md ai-docs/ .claude/` returns ≥1 commit. Add a new sub-step under Step 2 (between current `Step 2` body and `Step 3 — Categorise + propose fixes`) titled *"Step 2b — Stale-validation sweep"* with the trigger logic verbatim. Update Step 6 (Report) template to include a new line `- 🌱 Stale-validation: N (surfaced)` under the summary. | `.claude/agents/learnings-escalation-audit.md` | — |
| 4 | **Phase 3b — `🌱` verdict prose + Checklist L `Kind:` row + Checklist M sub-check 11 + new Checklist N in `ai-audit/SKILL.md`.** (a) `§ Phase 1` orchestration prose: add one paragraph after the existing "If the subagent left any entry as **needs user judgment**" item describing how `🌱 Stale-validation` items surface to the user (signal for `/improve`, not auto-fix). (b) Checklist L gains a fourth row for `Kind:` with an analogous 4-location list (per the *Chosen alternatives* reconciliation — Spec AC7 leaves location selection open): **AGENTS.md `## Learning Log` Entry-format declaration block** + `self-improve.md` Step 5 Commit-B backfill (parse site) + `learnings-escalation-audit.md` Steps 2/3/4 (parse site) + `ai-docs/corrections-log.md` field glossary (declaration mirror). The Exception-body locations used by the `Escalated?` / `Superseded by:` rows do not literally apply since `Kind:` is a declared-schema field with no `/improve`-time mutation. (c) Checklist M adds sub-check 11 (cross-shape verbs); update the table row count and severity table after the row. (d) New Checklist N added after Checklist M with severity `major`; bidirectional detection (forward: `## Patterns` → `Kind: validation` back-link; reverse: `Kind: validation` with `Escalated?` ≠ `no` → `## Patterns` in named target). **Forward-sweep carrier-vs-template exemption:** the forward sweep filters by entry-body verb content — only `## Patterns` entries whose body uses carrot verbs (*Default to* / *Prefer*) are required to back-link to a `Kind: validation` entry. Explicitly exempt: `ai-docs/agent-writing-style.md § Patterns` (template source, not a promoted-from-validation carrier). The audit recipe greps for carrot-verb presence within each `### N. <Name>` block before requiring a back-link; entries without carrot verbs (template scaffolding, non-promoted prose) are out of scope. | `.claude/skills/ai-audit/SKILL.md` | 3 |
| 5 | **Phase 4 — Threshold reframe in AGENTS.md + `improve/SKILL.md` lock-step.** Edit AGENTS.md line 336 from `Run /improve when ≥3 unescalated entries accumulate.` to the spec's rewrite: `Run /improve when ≥3 unescalated correction entries, ≥2 unescalated validation entries, or a 🌱 Stale-validation flag from /ai-audit accumulates.` In `.claude/skills/improve/SKILL.md`, update the body to add a one-line restate consistent with the new threshold (current SKILL.md body has no threshold restate; add as a new line after the 6-numbered list, before the `See also:` link). | `AGENTS.md`, `.claude/skills/improve/SKILL.md` | 2, 4 |
| 6 | **AC12 — `## Patterns` block in `.claude/skills/context-reset/SKILL.md`.** Add a new `## Patterns` section near the end of the file (after `## Rules`, before EOF if EOF is plain — verify position during implementation). Single entry: `### 1. Trust the compaction-recovery callout`. One-paragraph rule using carrot verb (*Default to following the callout exactly — locate, read end-to-end, re-enter from the top of the body — even when context feels thin*). Back-link to `ai-docs/learnings.md` 2026-05-19 *compaction-recovery protocol in skill files works* entry. | `.claude/skills/context-reset/SKILL.md` | — |
| 7 | **Verification sweep + Rust gates.** Run AC18's four Rust gates (`cargo build`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -- --check`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`). Run `wc -c AGENTS.md` to confirm the threshold-line rewrite did not breach 40,000 chars. Run the Checklist N bidirectional check by hand: forward `grep -rn "## Patterns" .claude/skills/ .claude/agents/ AGENTS.md` finds the context-reset block; reverse `grep -A2 "Kind: validation" ai-docs/learnings.md` finds the compaction-recovery entry — confirm `Escalated?` remains `no` per Open Question 1 Resolution (a); Checklist N's reverse direction predicate (`Escalated? ≠ no`) is therefore false for this entry, and the lack of reverse-direction firing is the expected pass condition. Run the AGENTS.md Propagation Rule grep: `grep -rn "≥3 unescalated" .claude/ AGENTS.md ai-docs/` — confirm AGENTS.md + `improve/SKILL.md` are both updated; no stale matches. | (verification, no edits) | 1–6 |

## Handoff plan

7 subtasks → groups of 3 + 3 + 1.

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). The parent `/task` resumes Step 8 in Group A's fresh-context subagent.
- **Group A:** subtasks 1–3 — Carrot-pass body + verb enumeration + Step 6 inverted prompt in `self-improve.md`, AND `🌱 Stale-validation` verdict in `learnings-escalation-audit.md`. (3 subtasks, equal to the cap.)
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). The parent `/task` resumes Step 8 in Group B with fresh context.
- **Group B:** subtasks 4–6 — `ai-audit/SKILL.md` checklist edits (Phase 3b), AGENTS.md + `improve/SKILL.md` threshold reframe (Phase 4), AC12 worked-example `## Patterns` block in `/context-reset`. (3 subtasks, equal to the cap.)
- **Handoff after Group B:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). The parent `/task` resumes Step 8 in Group C with fresh context.
- **Group C:** subtask 7 — verification sweep (Rust gates, char-cap check, Propagation-Rule grep). Terminal group (1 subtask; within the 1..=3 range).

## File-touch map

| File | What changes | Phase | AC coverage |
|---|---|---|---|
| `.claude/agents/self-improve.md` | New `### Step 1b — Carrot pass` body, `### Step 2b — Carrot pass routing` table, `## Promotion verbs` section, Step 6 reproducer template variants | 2 | AC3, AC4 |
| `.claude/agents/learnings-escalation-audit.md` | New `🌱 Stale-validation` verdict row in Step 2, new `Step 2b — Stale-validation sweep` body, Step 6 report-template addition | 3 | AC5 |
| `.claude/skills/ai-audit/SKILL.md` | `§ Phase 1` orchestration prose for `🌱` verdict; Checklist L gains `Kind:` row; Checklist M gains sub-check 11 (cross-shape verbs); new Checklist N (bidirectional `## Patterns` ↔ `Kind: validation`) | 3 | AC6, AC7, AC8, AC9 |
| `AGENTS.md` | Line 336 threshold-line rewrite (~150-char addition) | 4 | AC10 |
| `.claude/skills/improve/SKILL.md` | One-line threshold restate consistent with AGENTS.md | 4 | AC11 |
| `.claude/skills/context-reset/SKILL.md` | New `## Patterns` section with single entry; back-link to 2026-05-19 *compaction-recovery protocol* entry | AC12 | AC12 |
| (none — verification only) | Rust gates, char-cap check, Propagation-Rule grep | AC18 | AC18 |

**Sync-group fan-out (Phase 4 lock-step):** AGENTS.md `§ Learning Log` threshold line ↔ `improve/SKILL.md` body restate are paired in Subtask 5 (both edits in one subtask + one commit, per the Propagation Rule).

**Sync-group fan-out (Learning-Log group, AGENTS.md line 216):** Phase-3 edits to `self-improve.md` (Subtask 1–2) AND `learnings-escalation-audit.md` (Subtask 3) are sister-file edits under the Learning-Log group. AGENTS.md `§ Learning Log` section was already amended in PR-1 to declare the `Kind:` field; no further AGENTS.md edit is triggered by Subtasks 1–3 except the Phase-4 threshold line in Subtask 5. The two agent files co-evolve in this PR but do not require an AGENTS.md mirror edit beyond Subtask 5's threshold line — the Propagation Rule row at AGENTS.md line 216 *already* anticipates this fan-out shape (it names both agents as the sync targets, not AGENTS.md itself).

**No char-cap breach.** Pre-PR: 37,564 chars in AGENTS.md. Post-Subtask-5 estimate: ~37,714 (still in early-warning band; no breach of 40,000). All other touched files (`self-improve.md` 10,778; `learnings-escalation-audit.md` 9,650; `ai-audit/SKILL.md` 27,073; `improve/SKILL.md` 1,058; `context-reset/SKILL.md` 8,286) gain at most ~2,000 chars each — none approach 35,000.

## Risks

- **Risk:** AGENTS.md char-cap drift — Subtask 5 adds ~150 chars to a 37,564-char file. **Mitigation:** Subtask 7 explicitly runs `wc -c AGENTS.md` and confirms < 40,000. If a later subtask in this PR (e.g., a Phase-3 sync edit) inflates AGENTS.md unexpectedly, the verification sweep flags it before commit.
- **Risk:** Checklist N forward-direction false positives — a `## Patterns` section may carry an entry that references a `Kind: correction` entry (legitimately — patterns can reference correction history), OR may be a template-source file (e.g., `ai-docs/agent-writing-style.md § Patterns`) that defines the pattern shape rather than carrying a promoted-from-validation carrot. **Mitigation:** Checklist N's forward direction is scoped to `## Patterns` entries whose body uses carrot verbs (*Default to* / *Prefer*); correction-shaped references and non-carrot entries are out of scope. The audit greps for carrot-verb presence within each `### N. <Name>` block before requiring a back-link. `ai-docs/agent-writing-style.md § Patterns` is explicitly exempt as a template source. Operationalised in Subtask 4's Checklist N edit shape.
- **Risk:** `🌱` verdict false-positive rate — a `Kind: validation` entry > 30 days old with `Escalated? no` and an unrelated commit to AGENTS.md will flag. **Mitigation:** the per-validation surface (skill/agent named in the `Rule:` line) narrows the commit-search path; if no specific surface is named, fall back to whole-corpus `git log` and surface as `❓ Ambiguous` instead of `🌱`. Recorded in Subtask 3's body.
- **Risk:** Step 6 inverted-eval primitive-absence — the `self-improve` subagent still lacks the `Agent` primitive (per the 2026-05-15 *primitive genuinely lacks* entry). The Carrot-pass eval reuses the existing pause-and-surface protocol → no new primitive dependency. **Mitigation:** Subtask 2 emits the reproducer block in the SAME `## Step 6 handoff` block (one block per pattern across BOTH passes), preserving the protocol.
- **Risk:** AC12 back-link rot — if the 2026-05-19 compaction-recovery entry is later superseded, the `## Patterns` back-link in `/context-reset` becomes stale. **Mitigation:** Checklist N's reverse direction catches this (the validation entry would still exist by Boundary Rule 1; if `Escalated?` later changes to remove `skill:context-reset`, Checklist N flags the orphan). No additional guard needed.
- **Risk:** PR-2 size — 7 subtasks across 6 files; could feel "wide". **Mitigation:** Subtasks are small (each file gains < 100 net lines); the per-group cap of 3 subtasks fits the design's handoff contract.
- **Risk:** Concurrent edit conflict if `/improve` runs on the branch after PR-2 lands but before PR-3 (Phase 5). **Mitigation:** out of scope — PR-3 starts from PR-2's merged master commit; no in-flight branch sharing.

## Test Design

Instruction-surface changes — no Rust code changes — so "tests" are grep + structural checks. AC18 is the four Rust gates which must pass even though no `.rs` file changed.

### Subtask 1 (Carrot-pass body)
- **Location:** `.claude/agents/self-improve.md` § Step 1b / Step 2b
- **Verification:** `grep -n "Step 1b — Carrot pass\|Step 2b — Carrot pass routing" .claude/agents/self-improve.md` returns ≥2 hits; the routing table contains exactly three rows matching the spec (`1`, `≥2`, `1 + workflow primitive`); the asymmetric-promotion preconditions are spelled out.
- **Scenarios:** happy path (Carrot pass triggers on `Kind: validation` entry); edge (`Kind:` omitted → falls through to Correction pass); mixed (entry with both `Kind: validation` + recurrence ≥ 2 → routes via Carrot ≥2 row, not Correction's recurrence row).

### Subtask 2 (Promotion-verb enumeration + Step 6 inverted prompt)
- **Location:** `.claude/agents/self-improve.md` § Promotion verbs / § Step 6
- **Verification:** `grep -E '(Default to|Prefer)' .claude/agents/self-improve.md` returns hits inside the Carrot promotion verb block; `grep -E '(MUST|NEVER)' .claude/agents/self-improve.md` returns hits inside the Stick promotion verb block; Step 6 reproducer template contains both `Scenario:` line variants.
- **Scenarios:** Step 6 emits a Carrot reproducer for a `Kind: validation` pattern; the reproducer's PASS criterion names "pattern survives edge case X" not "violation absent".

### Subtask 3 (`🌱` verdict)
- **Location:** `.claude/agents/learnings-escalation-audit.md` § Step 2 / § Step 2b / § Step 6
- **Verification:** `grep -n "🌱 Stale-validation" .claude/agents/learnings-escalation-audit.md` returns ≥3 hits (Step 2 verdict table, Step 2b sweep body, Step 6 report template); the trigger logic names all three conjuncts (age > 30d, `Escalated? no`, ≥1 instruction-file commit).
- **Scenarios:** age-only (≤ 30d → no flag); `Escalated?` ≠ `no` (→ no flag); both conjuncts but zero commits since validation date (→ no flag); all three → `🌱` flag emitted; ambiguous surface (no specific skill/agent named in `Rule:`) → `❓ Ambiguous` fallback per the Risk mitigation.

### Subtask 4 (`ai-audit/SKILL.md` checklists)
- **Location:** `.claude/skills/ai-audit/SKILL.md` § Phase 1 orchestration; § Step 2.3 Checklists L / M / N
- **Verification:**
  - **Phase 1 prose:** `grep -n "🌱 Stale-validation" .claude/skills/ai-audit/SKILL.md` returns ≥1 hit in the Phase 1 section.
  - **Checklist L:** the table has 4 rows (`Escalated?`, `Superseded by:`, `Kind:`, plus the existing fourth — confirm count post-edit); the `Kind:` row names the four coverage locations from the Subtask-4 reconciliation (AGENTS.md Entry-format block + `self-improve.md` Step 5 + `learnings-escalation-audit.md` Steps 2/3/4 + `ai-docs/corrections-log.md` field glossary).
  - **Checklist M:** sub-check 11 exists; `grep -nE 'sub-check 11|cross-shape' .claude/skills/ai-audit/SKILL.md` returns hits; severity `major`.
  - **Checklist N:** appears between Checklist M and Checklist L (verify ordering matches existing alphabetical sequence); bidirectional detection mechanism documented; severity `major`.
- **Scenarios:**
  - Forward Checklist N: a `## Patterns` entry whose body uses a carrot verb (*Default to* / *Prefer*) with no `learnings.md` back-link → flag. A `## Patterns` entry without any carrot verb (template scaffolding) → no flag (carrier-vs-template exemption). `ai-docs/agent-writing-style.md § Patterns` is the named exempt template source.
  - Reverse Checklist N: a `Kind: validation` entry whose `Escalated?` names `skill:foo` but `.claude/skills/foo/SKILL.md` has no `## Patterns` block → flag.
  - Sub-check 11: a `## Patterns` block using `MUST` in an entry body → flag (carrot rule, stick verb); a fail-loud AXIOM blockquote using `Default to` → flag (stick rule, carrot verb).

### Subtask 5 (threshold reframe)
- **Location:** `AGENTS.md` line 336; `.claude/skills/improve/SKILL.md` body
- **Verification:** `grep -n '≥3 unescalated correction\|≥2 unescalated validation\|🌱 Stale-validation flag' AGENTS.md .claude/skills/improve/SKILL.md` returns ≥1 hit per phrase per file (i.e., both files updated); `wc -c AGENTS.md` reports < 40,000.
- **Scenarios:** count three accumulated `Kind: correction` entries → `/improve` triggers per the new line; count two accumulated `Kind: validation` entries → `/improve` triggers; emit a single `🌱` flag from `/ai-audit` → `/improve` triggers.

### Subtask 6 (AC12 `## Patterns` block)
- **Location:** `.claude/skills/context-reset/SKILL.md` § Patterns (new section)
- **Verification:**
  - `grep -n '^## Patterns' .claude/skills/context-reset/SKILL.md` returns exactly 1 hit.
  - The single entry uses a carrot verb (*Default to* / *Prefer*); a `MUST` / `NEVER` in the entry body → grep flags.
  - Back-link line resolves: `grep -F '2026-05-19' .claude/skills/context-reset/SKILL.md` returns the back-link AND `grep -n 'compaction-recovery protocol in skill files works' ai-docs/learnings.md` returns the entry.
- **Scenarios:** Checklist N's reverse-direction predicate requires `Escalated? ≠ no`; per Open Question 1 Resolution (a), the field stays `no`, so Checklist N does not fire on this entry. The `🌱 Stale-validation` verdict requires age > 30d (entry is < 30d post-AC12 landing) so cannot fire until ~2026-06-18 regardless of `Escalated?` state. Both audit gates are inert by design — that is the expected behaviour for the worked example until the next `/improve` cycle backfills `Escalated?`.

### Subtask 7 (verification sweep)
- **Entry point:** Rust gates + grep recipes; no edits, just confirmations.
- **Scenarios:** all four AC18 commands return 0; AGENTS.md char count < 40,000; `grep -rn "## Patterns" .claude/skills/ .claude/agents/ AGENTS.md` returns the context-reset block as the only `## Patterns` block in the audited surface (since other skills/agents will gain `## Patterns` blocks as carrots accumulate; in PR-2, only `/context-reset` has one).

## Open questions

All three resolved during this design's design-review round (GO with 4 notes, folded back). Resolutions captured inline below; design-review verdict carries Resolution (a) for Q1, "yes" for Q2, "no" for Q3 as the binding decisions Subtasks 4 / 6 / 7 implement.

1. **Should Subtask 6 (the AC12 `## Patterns` block landing) ALSO update the 2026-05-19 *compaction-recovery protocol in skill files works* entry's `Escalated?` field from `no` to `skill:context-reset` in the same PR?** **RESOLVED — (a)**: leave `Escalated? no`, document the deferred update as a follow-up `/improve` invocation. Three considerations:
   - **Pro:** Without this update, the 2026-05-19 entry remains `Escalated? no` after the `## Patterns` block lands — which means `/improve`'s ≥2-validation threshold would fire on the next unescalated validation (since this one stays `no`). The AGENTS.md threshold reframe in Subtask 5 says "≥2 unescalated validation entries" — leaving this entry `no` while a back-link exists in `/context-reset/SKILL.md` is a logical inconsistency.
   - **Con:** Per AGENTS.md Boundary Rule 1 Exception, `Escalated?` updates are `self-improve`-agent-driven (via `/improve`'s Commit B backfill) OR `learnings-escalation-audit`-agent-driven (via `/ai-audit` Phase 1). A PR-2 implementer manually editing the field would technically violate the Exception's "agent-driven only" clause. The worked-example carve-out in PR-1 covered the `Kind:` retro-add — it did NOT extend to `Escalated?` mutation.
   - **Resolution candidates:** (a) leave `Escalated? no`, document the deferred update as a follow-up `/improve` invocation; (b) extend the Boundary-Rule-1 carve-out one more time to cover the `Escalated?` update as part of the worked-example completion (would require an AGENTS.md text edit clarifying the extended carve-out — adds ~80 chars); (c) trigger a one-shot `/improve` invocation as part of the PR-2 acceptance flow, recording the Commit-B backfill on the same branch as a separate commit. **Design recommendation: (a)** — keep the carve-out narrow, do NOT extend; document the deferred `/improve` invocation as a one-line note at the bottom of Subtask 6's body.

2. **Should Checklist N's reverse direction account for `Escalated?` field listing multiple targets (e.g., `skill:context-reset, AGENTS.md`)?** **RESOLVED — yes**: the reverse direction iterates each comma-separated `Escalated?` value independently. A validation entry escalated to two targets should have `## Patterns` blocks in BOTH. The forward direction (every `## Patterns` block needs a back-link) is unaffected. Captured in Subtask 4's body.

3. **Should the AGENTS.md Propagation Rule row (line 216, Learning-Log group) be amended to name `## Patterns` ↔ `Kind: validation` coherence as a fan-out trigger?** **RESOLVED — NO**: adding it would conflate the entry-format sync (which is about the two agents reading `learnings.md`) with the carrier-file ownership (which is about whatever skill claims a carrot). Today's row names "entry format incl. `Kind:`, `Escalated?` semantics, 🌱 verdict from `/ai-audit`". The `## Patterns` ↔ `Kind: validation` link is one level removed — it's a carrier-file relationship, not an entry-format change. Checklist N is the audit-side gate; the Propagation Rule row stays at its current scope.

## Phase 5 sketch (NOT part of PR-2)

For continuity only. PR-3 implements Phase 5 (cross-feed with `~/.claude/projects/.../memory/feedback_*.md`). PR-3's design must specify:

- **(a) Consent UX.** `AskUserQuestion` shape: "auto-memory entry `feedback_X.md` names a workflow primitive (`<primitive>`) and has no matching `Kind: validation` in `learnings.md`. Surface as a `/improve` candidate? [yes / no / never-this-entry]". The third option writes an opt-out flag to `~/.claude/projects/.../memory/.improve_optout` (user-local, not project-side).
- **(b) Project-side write guard.** `self-improve` MUST NOT write to AGENTS.md / skills / agents based solely on an auto-memory entry; auto-memory is a *companion signal* surfaced during `/improve`'s Step 2 routing, never a primary source. If consent is declined, the signal is dropped — not silently retained.

PR-3 adds (a) a new `### Step 1c — Auto-memory companion sweep` to `self-improve.md` AND (b) a new `## Privacy boundary` section to `improve/SKILL.md`. No changes to `learnings-escalation-audit.md` or `ai-audit/SKILL.md` — auto-memory is not in the audit corpus.

## Quality checklist self-verification

- **Completeness:** all six target files listed in the File-touch map; AC1–AC2 already covered by PR-1; AC3–AC12 + AC18 covered by PR-2 subtasks 1–7; AC16–AC17 covered by the Phase 5 sketch above (AC17 explicitly recommends the follow-up-PR default). Tasks are atomic — each subtask touches at most 2 files, each subtask is a logically complete edit.
- **Correctness:** the Carrot pass sits parallel to the Correction pass (verified by reading the existing `self-improve.md` Step 2 routing table — adding `Step 1b` / `Step 2b` parallel sections is structurally clean); the `🌱` verdict slots into the existing 4-verdict set without disrupting the existing flow (verified by reading `learnings-escalation-audit.md` § Step 2 — verdicts are a flat list, not a state machine); Checklist N follows Checklist C's bidirectional-resolution shape (verified by reading `ai-audit/SKILL.md` § C — same severity, same bidirectional language).
- **Tests:** every non-trivial edit has a verification recipe in the Test Design section (grep keywords, scenarios per AC); AC18 is the Rust-gate suite which fires unconditionally.
- **Risks:** char-cap breach, false-positive rates on Checklist N + `🌱`, primitive-absence in Step 6 inverted eval, AC12 back-link rot — all identified with mitigations.
- **Economy:** no new files, no new abstractions, no new agents. The Carrot pass reuses the existing `self-improve.md` skeleton (Steps 3 / 4 / 5 / 6); the `🌱` verdict reuses the existing verdict-emission protocol; AC12 reuses the existing `## Patterns` shape from `agent-writing-style.md`. YAGNI honoured.
- **Handoff plan:** 7 subtasks → 3 + 3 + 1, all groups within `1..=3`, non-terminal groups exactly 3, `/context-reset` named at every boundary including Group A entry. Compliant with the design's handoff-grouping contract.

## References

- Spec: `ai-docs/plans/2026-05-19-carrot-pass-improve-ai-audit.spec.md`
- PR-1 design (historical): `ai-docs/plans/2026-05-19-carrot-pass-improve-ai-audit.phase1.design.md`
- AGENTS.md § Learning Log (lines 279–336) — the schema home for the threshold reframe (Subtask 5).
- AGENTS.md § Propagation Rule (line 216) — Learning-Log sync-group row (already amended in PR-1; no further edit in PR-2).
- `ai-docs/corrections-log.md` § Entry format — field glossary (`Kind:` line added in PR-1) — no edit in PR-2.
- `.claude/agents/self-improve.md` (10,778 chars) — Carrot-pass + Step-6 inverted-prompt host (Subtasks 1–2).
- `.claude/agents/learnings-escalation-audit.md` (9,650 chars) — `🌱 Stale-validation` host (Subtask 3).
- `.claude/skills/ai-audit/SKILL.md` (27,073 chars) — Phase 1 prose + Checklist L/M/N edits (Subtask 4).
- `.claude/skills/improve/SKILL.md` (1,058 chars) — threshold restate (Subtask 5, lock-step with AGENTS.md).
- `.claude/skills/context-reset/SKILL.md` (8,286 chars) — AC12 `## Patterns` host (Subtask 6).
- `ai-docs/agent-writing-style.md` § Patterns (lines 23–186) — the per-pattern block shape AC12 mirrors.
- `ai-docs/agent-writing-style.md` § Pattern 8 (40k char-cap) — the cap the AGENTS.md edit in Subtask 5 must respect.
- `ai-docs/learnings.md` lines 1299–1309 — the 2026-05-19 *compaction-recovery protocol in skill files works* entry that AC12 back-links.
