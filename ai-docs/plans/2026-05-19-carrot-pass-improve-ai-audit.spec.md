# Carrot Pass for `/improve` and `/ai-audit`

**Source:** issue #491
**Date:** 2026-05-19
**Tracked in:** #491

## Problem statement

The self-learning harness today is stick-only. `ai-docs/learnings.md` is named *Corrections Log*; `/improve`'s `self-improve` subagent looks for repeating mistakes; `/ai-audit`'s `learnings-escalation-audit` subagent verifies escalation drift; the trigger threshold (*"Run `/improve` when ≥3 unescalated entries accumulate"*) presumes violations.

Two 2026-05-19 `learnings.md` entries introduced the *carrot* concept (positive-validation capture):

1. *compaction-recovery protocol in skill files works* — explicit positive validation of the recovery callout, confirmed across 4 rounds and multiple compressions.
2. *reinforce with carrot and stick: record positive validations, not only violations* — meta-rule mandating positive-validation capture going forward.

These entries land in `learnings.md` but no machinery promotes them; they sit forever unescalated, drifting away from the surface they were meant to reinforce.

This task extends the harness asymmetrically. A wrong stick costs efficiency; a wrong carrot locks in a brittle approach as a default. Carrots therefore get a **lower escalation threshold** (positive signal is rarer), **weaker promotion verbs** (*Default to* / *Prefer*, never *MUST* / *NEVER*), and a **stale-revalidation** mechanism the stick side does not need.

## Scope

The work is split into five phases. Phase 1 is a hard prerequisite for Phases 2–4. Phase 5 is independent and lands as a follow-up PR (Q5 resolved: include — see Key Decisions for the PR-slicing decision).

### Phase 1 — Schema (additive, append-only safe)

- Add `**Kind:** correction | validation` to the entry template (default `correction` when omitted; existing entries are not rewritten — Boundary Rule 1 holds, except for the named worked-example carve-out below).
- Rename `## Corrections Log` → `## Learning Log` in `AGENTS.md` and propagate to every cross-reference (`AGENTS.md`, `ai-docs/corrections-log.md`, `.claude/agents/self-improve.md`, `.claude/agents/learnings-escalation-audit.md`, plus every `Corrections Log` / `correction log` / `corrections-log` substring across `.claude/`).
- One-off Boundary-Rule-1 exception: retro-add `Kind: validation` to the 2026-05-19 *compaction-recovery protocol in skill files works* entry, recorded in that entry's `Superseded by:` line. This is named, narrow, and audit-traced — not a precedent for further bulk edits.
- Update `AGENTS.md § Propagation Rule` row for the Corrections-Log sync group so future edits to entry-format / `Kind:` / `🌱` semantics fan out to both agents.
- Sync-group fan-out (Phase 1 touchpoints):
  - `AGENTS.md` — entry format, Propagation Rule row, section rename (`Corrections Log` → `Learning Log`)
  - `ai-docs/corrections-log.md` — extracted field glossary + carve-outs (Boundary rules 1 + 2 + Exceptions); section rename reflected in any header / cross-references
  - `.claude/agents/self-improve.md` — entry-format reference in Step 5 backfill rules; section-name references updated
  - `.claude/agents/learnings-escalation-audit.md` — entry-format reference in Step 1 parse rules; section-name references updated
  - Any prose hits across `.claude/skills/**` / `.claude/agents/**` (enumerated by `grep -rn -E 'Corrections.Log|corrections-log' AGENTS.md ai-docs/ .claude/` before Phase 1 commit)

### Phase 2 — `self-improve` agent: Carrot pass

Add a second pattern-detection pass alongside the existing Correction pass (current Step 1). The Carrot pass has its own Step 2 routing table:

| Validation entries on same topic | Action |
|---|---|
| 1 | Add a `## Patterns` entry to the most-local skill / agent / AGENTS.md (mirrors `ai-docs/agent-writing-style.md § Patterns`); back-link to learning |
| ≥2 | Promote within the same `## Patterns` section in the targeted file — verb: *Default to* / *Prefer*, never *MUST* / *NEVER* (promotion is a wording / verb edit within the section, not a file relocation — see Key Decisions row *Carrot rule home*) |
| 1 + names a workflow primitive | Hold for second confirmation; surface as candidate in the report |

Enumerate the allowed promotion verbs per `Kind` explicitly in agent prose (resolved via round 2 Q4):

- **Carrot promotion verbs:** *Default to*, *Prefer* (soft; ≥1 validation seeds the `## Patterns` entry, ≥2 strengthens the wording within the section).
- **Stick promotion verbs:** *MUST*, *NEVER* (hard; reserved for the existing Correction pass).
- Cross-shape (carrot rules with stick verbs, or stick rules with carrot verbs) is forbidden and audited by `/ai-audit` Checklist M sub-check (Phase 3).

Step 6 eval is **inverted** for the Carrot pass: the reproducer asks *"in scenario X (edge case from the original learning's surface), does the pattern still hold?"* rather than *"does the violation still happen?"*. PASS = pattern survives the edge; FAIL = pattern overfits, downgrade promotion verb or do not promote.

### Phase 3 — `learnings-escalation-audit`: extend verdicts + `/ai-audit` Phase 2 reframe

- Add `🌱 Stale-validation` to the four-verdict set (✅ / ⚠️ / ❌ / ❓): a `Kind: validation` entry older than 30 days whose `Escalated?` is `no` AND whose targeted surface has had ≥1 instruction-file commit since the validation date. Signal for `/improve`, not auto-fix.
- Document the new verdict in `.claude/skills/ai-audit/SKILL.md § Phase 1` orchestration prose (surfaces verdicts to the user).
- Update `/ai-audit § Step 2.3` Checklist L (Corrections-Log field coherence) to grow a `Kind:` row — same 4-location coverage discipline (AGENTS.md Boundary Rule 1+2 Exceptions, `self-improve.md` Step 5, `learnings-escalation-audit.md` Steps 2/3/4).
- Add new Checklist N — `## Patterns` ↔ `Kind: validation` coherence, severity `major` (dead-reference-class). Bidirectional:
  - Every `## Patterns` section in a skill / agent / AGENTS.md back-links to at least one `Kind: validation` entry in `learnings.md`.
  - Every `Kind: validation` entry whose `Escalated?` ≠ `no` has a corresponding `## Patterns` block in the named target.
- Add conditional Checklist M sub-check (resolved active per round 2 Q4): flag (a) carrot-shaped rules using fail-loud verbs (`MUST` / `NEVER`) AND (b) stick-shaped rules using carrot verbs (*Default to* / *Prefer*). Severity `major` — the verb asymmetry IS the asymmetric-promotion contract.

The two-phase structure of `/ai-audit`, the Step 2.1–2.6 sequence, the commit boundary in Step 3, and the gate checklist remain unchanged.

### Phase 4 — `/improve` trigger reframe

`AGENTS.md § Learning Log` (renamed from `§ Corrections Log` in Phase 1) line *"Run `/improve` when ≥3 unescalated entries accumulate"* becomes:

> Run `/improve` when **≥3 unescalated correction entries**, **≥2 unescalated validation entries**, or a `🌱 Stale-validation` flag from `/ai-audit` accumulates.

Sync-group fan-out: `.claude/skills/improve/SKILL.md` body restates the threshold and must be updated in lock-step.

### Phase 5 — Cross-feed with auto-memory (included; follow-up PR)

`self-improve` reads `~/.claude/projects/.../memory/feedback_*.md` as a *companion* signal: a `feedback` memory that names a workflow primitive AND has no matching `learnings.md` validation entry is a candidate to surface during `/improve`.

Design MUST specify, before implementation:

- **(a) Consent UX.** Explicit user approval required before any project-side write derived from auto-memory signals. Auto-memory is user-local; project agents do not import or paraphrase its content into instruction files without an in-the-loop confirmation.
- **(b) Project-side write guard.** No automatic writes to instruction files based on auto-memory alone. Auto-memory is a *companion signal* surfaced during `/improve`, never a primary source. If the user does not approve surfacing, the signal is dropped — not silently retained or escalated.

Phase 5 lands as a separate follow-up PR after the Phases 2+3+4 PR (see Key Decisions row *PR slicing*). The design agent may, at design time, recommend folding Phase 5 into the 2+3+4 PR if implementation effort proves minimal; otherwise the follow-up-PR default holds.

### Worked-example end-to-end demonstration

Demonstrate the new flow on the 2026-05-19 *compaction-recovery protocol in skill files works* entry — promoted end-to-end through the Carrot pass. A `## Patterns` block (per round 2 Q3 resolution: Inline Patterns) is added to the relevant skill file, back-linked to the entry. The legacy entry is retro-tagged `Kind: validation` per the round-1 carve-out (Option 1), recorded in its `Superseded by:` line.

## Out of scope

- Code changes to `quartzite-*` crates — this issue only touches the instruction surface and agent / skill files.
- Schema migration of existing `learnings.md` entries — they remain `Kind: correction` implicitly per the default-when-omitted rule. No backfill commit. (The worked-example carve-out is the single named exception.)
- Auto-memory write-back from project agents to `~/.claude/projects/.../memory/*` — Phase 5 only reads auto-memory as a companion signal; no writes back to the user-local layer.
- A separate `ai-docs/patterns.md` index file — Q3 resolved to Inline Patterns in the most-local file; no central index is created.
- Heuristic prose-based legacy-entry detection in `self-improve.md` — Q3's Option B (Option 2 from the worked-example carve-out fallback list) is rejected in favour of the surgical one-off retro-add.

## Deferred

- Bulk retro-tagging of remaining historical `learnings.md` entries with `Kind: correction` | low-leverage churn vs. the default-when-omitted rule already covers them | no separate issue needed; revisit only if a future audit shows the implicit default causes confusion.
- A central `ai-docs/patterns.md` cross-skill index | rejected by Q3 (Inline Patterns chosen); revisit only if cross-skill reuse of carrot patterns proves to be a real need | new issue needed if reopened.

## Key decisions

| Question | Decision |
|---|---|
| Schema additivity | `Kind:` defaults to `correction` when omitted — existing entries need no rewrite |
| Boundary Rule 1 invariant | Append-only invariant holds for the schema migration; existing entries are NOT bulk-edited |
| Worked-example carve-out (round 1 Q1) | Option 1 — one-off retro-add of `Kind: validation` to the 2026-05-19 *compaction-recovery protocol in skill files works* entry, authorised as a named Boundary-Rule-1 exception for this single worked-example commit. Audit trail preserved by recording the retro-add in that entry's `Superseded by:` line. |
| Section rename (round 1 Q2) | Rename `## Corrections Log` → `## Learning Log` in `AGENTS.md` and propagate fan-out across all cross-references (`AGENTS.md`, `ai-docs/corrections-log.md`, `.claude/agents/self-improve.md`, `.claude/agents/learnings-escalation-audit.md`, and any `Corrections Log` / `correction log` / `corrections-log` substring across `.claude/`). Lands in Phase 1. |
| Carrot rule home (round 2 Q3) | Inline Patterns — promoted carrots live in the `## Patterns` section of the most-local skill / agent / AGENTS.md (mirrors `ai-docs/agent-writing-style.md § Patterns`). The same `## Patterns` section holds both soft *Default to*-style entries (≥1 validation) AND the promoted rule form (≥2 validations) — promotion is a wording / verb edit within the section, not a file relocation. |
| Verb-set encoding (round 2 Q4) | Encode — enumerate the allowed verb sets explicitly in `.claude/agents/self-improve.md` (carrot: *Default to* / *Prefer*; stick: *MUST* / *NEVER*). `/ai-audit` Phase 2 Checklist M gains a sub-check flagging (a) carrot-shaped rules using fail-loud verbs AND (b) stick-shaped rules using carrot verbs. Severity `major`. |
| Phase 5 scope (round 2 Q5) | Include — keep Phase 5 in scope. Design MUST specify (a) consent UX and (b) project-side write guard before implementation. Phase 5 lands as a follow-up PR after Phases 2+3+4 (see PR slicing row); design may recommend folding it into the 2+3+4 PR if effort warrants. |
| Validation supersession semantics | Bi-directional `Superseded by:` already exists. Default convention: a disconfirmed validation becomes a new `Kind: correction` entry whose `Superseded by:` references the original validation. Documented in `ai-docs/corrections-log.md` field glossary alongside the existing `Superseded by:` semantics. |
| PR slicing | Three PRs: (1) Phase 1 alone, (2) Phases 2+3+4 stacked, (3) Phase 5 as follow-up. Phases 2/3/4 are co-dependent (the trigger reframe in 4 cites the verdict from 3 and the pass from 2) and cannot be split further without artificial coupling. The design agent may, at design time, recommend folding Phase 5 into PR 2 if the implementation diff is small; otherwise the follow-up-PR default holds. |
| Phase 4 sync-group lock-step | `AGENTS.md § Learning Log` threshold line ↔ `.claude/skills/improve/SKILL.md` body restate of the threshold must update in lock-step (Propagation Rule fan-out) |

## Technical constraints

- **Append-only invariant.** Boundary Rule 1 prohibits editing existing entries except `Escalated?` / `Superseded by:` lines (per Exception). The worked-example carve-out for the 2026-05-19 compaction-recovery entry is an authorised, named, one-off exception — retro-add of `Kind: validation` recorded in the entry's `Superseded by:` line. The carve-out MUST be explicitly named in this spec AND surfaced in `ai-docs/corrections-log.md § Boundary rule 1` (or its referenced section) so the audit trail is clear.
- **Propagation Rule discipline.** Every edit to a sync-group member fans out to the other members in the same PR. The Corrections-Log sync group expands in Phase 1 to cover entry-format / `Kind:` / `🌱` semantics; new fan-out rules land alongside in Phase 1's PR.
- **Asymmetric verb sets enforced semantically + audited.** Carrot promotions use *Default to* / *Prefer* (soft); stick promotions use *MUST* / *NEVER* (hard). Verb choice is not enforceable by hook. `.claude/agents/self-improve.md` enumerates the allowed verb sets explicitly; `/ai-audit` Phase 2 Checklist M sub-check flags cross-shape violations (severity `major`).
- **`/ai-audit` Phase 2 structure unchanged.** Step 2.1–2.6 sequence, the commit boundary in Step 3, and the gate checklist remain unchanged. Only Checklist L grows a row, Checklist M gains a sub-check, and Checklist N is added.
- **Validation entry shape unchanged.** `Kind: validation` is a new field on the same entry shape; no new file, no new section in `learnings.md`.
- **Phase 5 privacy boundary.** Project agents read user-local auto-memory only as a companion signal during `/improve`, never write back to the user-local layer, and surface candidates only with explicit user approval. The privacy boundary is a design constraint, not just an AC; design agent must produce a concrete consent-UX shape (e.g., AskUserQuestion prompt + opt-in flag) before implementation.
- **`## Patterns` section convention.** The same `## Patterns` section holds both `Default to`-style soft entries (≥1 validation) AND promoted rule-form entries (≥2 validations). Promotion is an edit within the section, not a relocation across files.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `Kind:` field is defined in `AGENTS.md § Learning Log` *Entry format* AND in `ai-docs/corrections-log.md`'s field glossary, with default `correction` when omitted |
| AC2 | `AGENTS.md § Propagation Rule` table row for the Corrections-Log sync group names entry-format / `Kind:` / `🌱` semantics as triggers for fan-out |
| AC3 | `.claude/agents/self-improve.md` has a Carrot pass alongside the Correction pass, with the asymmetric threshold table (1 → `## Patterns` + back-link; ≥2 → promotion within `## Patterns`; 1 + workflow primitive → hold) AND the inverted Step 6 eval prompt |
| AC4 | `.claude/agents/self-improve.md` enumerates the allowed promotion verbs per `Kind` explicitly (carrot: *Default to* / *Prefer*; stick: *MUST* / *NEVER*) |
| AC5 | `.claude/agents/learnings-escalation-audit.md` emits `🌱 Stale-validation` flags per the documented trigger (entry age > 30d AND `Escalated? no` AND ≥1 instruction-file commit since validation date) |
| AC6 | `.claude/skills/ai-audit/SKILL.md § Phase 1` orchestration prose documents the `🌱 Stale-validation` verdict and how it surfaces to the user |
| AC7 | `.claude/skills/ai-audit/SKILL.md § Step 2.3` Checklist L grows a `Kind:` row enforcing 4-location coverage |
| AC8 | `.claude/skills/ai-audit/SKILL.md § Step 2.3` Checklist M gains a sub-check flagging cross-shape verb usage (carrot rules with fail-loud verbs, stick rules with carrot verbs), severity `major` |
| AC9 | `.claude/skills/ai-audit/SKILL.md § Step 2.3` Checklist N exists (carrot-side analog of Checklist C), bidirectional `## Patterns` ↔ `Kind: validation`, severity `major` |
| AC10 | `AGENTS.md § Learning Log` `/improve` trigger line covers both `Kind`s and the stale-validation flag (≥3 unescalated corrections, OR ≥2 unescalated validations, OR a 🌱 flag) |
| AC11 | `.claude/skills/improve/SKILL.md` body restates the new threshold consistently with `AGENTS.md` (sync-group lock-step) |
| AC12 | The 2026-05-19 *compaction-recovery protocol in skill files works* entry is promoted end-to-end as a worked example — `## Patterns` block exists in the targeted skill (Inline-Patterns home per Key Decisions), back-linked to the entry, demonstrating the new flow. The entry itself is retro-tagged `Kind: validation` per the round-1 carve-out, with a new `Superseded by:` line on the entry recording the carve-out commit hash + a one-line reason. |
| AC13 | The Boundary-Rule-1 carve-out for the worked-example retro-add is explicitly named in this spec AND in `ai-docs/corrections-log.md § Boundary rule 1` (or its referenced section) — the audit trail makes it clear that the retro-add was an authorised exception, not a precedent for further bulk edits to historical entries. |
| AC14 | Section rename `## Corrections Log` → `## Learning Log` is applied to every cross-reference. Enumeration of cross-references is performed programmatically before the Phase 1 commit: `grep -rn -E 'Corrections.Log|corrections-log' AGENTS.md ai-docs/ .claude/` returns no stale hits after Phase 1 lands. |
| AC15 | All sync-group fan-outs verified: any change to entry format / `Kind:` / 🌱 semantics appears in every member of the Corrections-Log sync group (`AGENTS.md`, `ai-docs/corrections-log.md`, both agents) |
| AC16 | Phase 5 design specifies (a) consent UX (explicit user approval before any project-side write derived from auto-memory signals) AND (b) project-side write guard (no automatic instruction-file writes based on auto-memory alone; auto-memory is a companion signal) |
| AC17 | Phase 5 lands as a follow-up PR after the Phases 2+3+4 PR — OR, if the design agent recommends folding Phase 5 into PR 2 based on implementation effort, that recommendation is recorded in the design document with rationale |
| AC18 | `cargo build`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -- --check`, and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` all pass — no Rust code changed, but the gates are part of the merge contract |

## Open questions

None — all five open questions from the issue body (Q1 worked-example carve-out, Q2 section rename, Q3 carrot rule home, Q4 verb-set encoding, Q5 Phase 5 scope) are resolved and captured in Key Decisions.

## References

- `ai-docs/learnings.md` 2026-05-19 *compaction-recovery protocol in skill files works* — the validation that motivated this issue
- `ai-docs/learnings.md` 2026-05-19 *reinforce with carrot and stick: record positive validations, not only violations* — meta-rule
- `AGENTS.md § Corrections Log` (Boundary rules 1 + 2 + exceptions) — the contract the schema change must keep intact (renamed § Learning Log in Phase 1)
- `.claude/agents/self-improve.md` Step 6 *Primitive-absence statement* — the existing pause-and-surface pattern the inverted-eval phase reuses
- `.claude/skills/ai-audit/SKILL.md § Step 2.3` Checklists C + L + M — the integrity gates that grow to accommodate `Kind:` and `## Patterns` back-links; new Checklist N is the carrot-side analog of C
- `ai-docs/agent-writing-style.md § Patterns` — the precedent for the `## Patterns` section convention adopted for promoted carrots (Q3 resolution)
- System prompt § *auto memory* — the `feedback` type already covers carrots at the user-local layer; Phase 5 cross-feeds this signal into `/improve`
