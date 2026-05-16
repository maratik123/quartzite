# Codify 40k char-cap AXIOM in agent-writing-style.md and /ai-audit Checklist M

**Source:** issue #416
**Date:** 2026-05-16
**Tracked in:** #416

## Scope

1. Add the 40,000-char file-size cap AXIOM (from `AGENTS.md § Build & Test`) to `ai-docs/agent-writing-style.md` as **Pattern 8** under `## Patterns`. The new pattern body MUST include:
   - **(i) Threshold table.** Three bands: `< 35,000` chars = OK; `35,000–39,999` = early warning (proactive extraction); `≥ 40,000` = STOP (AXIOM violation).
   - **(ii) Covered file set enumerated.** Explicit paths (no `**` globs in the enumeration itself, though the source AXIOM uses globs for convenience):
     - `AGENTS.md`
     - `CLAUDE.md`
     - every `.claude/skills/**/SKILL.md`
     - every `.claude/agents/**.md`
     - `ai-docs/code-style.md`
     - `ai-docs/doc-convention.md`
     - `ai-docs/context.md`
     - `ai-docs/agent-writing-style.md`
     - `ai-docs/corrections-log.md`
   - **(iii) Extraction model citation.** Reference PR #324 by number as the canonical extraction example (move verbose subsections into `ai-docs/<topic>.md` reference pages with anchored links from the source file). No verbatim duplication of #324's diff body.
   - **(iv) Per-commit invariant note.** Every commit boundary on a feature branch must keep every covered file under the hard cap — transient violations between intermediate commits are real violations, not "fixed at merge time".

2. Extend `.claude/skills/ai-audit/SKILL.md` Checklist M with two new sub-checks:
   - **Sub-check 9 — File-size AXIOM conformance.** Runs `wc -c` against the enumerated covered file set. Findings:
     - `35,000–39,999` chars → severity `minor` (early warning).
     - `≥ 40,000` chars → severity `blocker` (AXIOM violation).
     - `< 35,000` chars → no finding.
     The detection command and the covered-file set are inline in Checklist M (not deferred to an external script — that's #383's job).
   - **Sub-check 10 — Style-guide audit coverage map.** Reads `ai-docs/agent-writing-style.md` at audit time and parses its `## ` (level-2) headings. For each heading, matches against an existing Checklist M sub-check (1–9). If a heading has no corresponding sub-check, emits a `nit` finding: `audit coverage gap: § <heading>` with proposed action `add sub-check N+1 to /ai-audit Checklist M`. This makes future style-guide additions self-policing without requiring an audit-side edit each time.

3. Manual demonstrator: running `/ai-audit` (or just Phase 2 Checklist M) against the post-implementation corpus surfaces AGENTS.md = 39,960 chars (current size) as a `minor`-severity finding via Sub-check 9, demonstrating the early-warning state works end-to-end.

## Out of scope

- Implementation of `scripts/check-instruction-file-sizes.sh` (the pre-commit / CI mechanical gate). Already tracked separately in #383. This task's audit-side check is a back-stop, not a replacement.
- Rewriting the AXIOM body in `AGENTS.md § Build & Test`. The AXIOM stays where it is; `agent-writing-style.md` Pattern 8 references it as the source-of-truth via `AGENTS.md § Build & Test`.
- Editing `AGENTS.md` for this task. The Round-1 decision to land the AXIOM as Pattern 8 means no new propagation-rule row is needed in AGENTS.md (the existing `agent-writing-style.md` propagation row covers fan-out via `## Propagation rule for new patterns` in the style guide).
- Changing the severity bands (35k / 40k) or the covered file set — both inherited verbatim from the AGENTS.md AXIOM.
- Generalising Sub-check 10's heading-derived coverage map to other style references (e.g., `code-style.md`, `doc-convention.md`).
- Adding a Rule-5 substring blacklist entry to `.claude/agents/spec-writer.md`. The char-cap rule is a structural property of files, not a question-time pre-resolved rule.

## Deferred

- `scripts/check-instruction-file-sizes.sh` pre-commit / CI gate | long-term mechanical enforcement complements the audit-side back-stop | already tracked in #383
- Generalising Sub-check 10's heading-derived coverage map to other reference style guides (`code-style.md`, `doc-convention.md`, etc.) | reduces audit drift across multiple style references | separate issue if/when more style guides accumulate sections worth auditing

## Key decisions

| Question | Decision |
|---|---|
| Where does the AXIOM live in `agent-writing-style.md`? | **Pattern 8** under `## Patterns` (Round 1 answer). Inherits the existing `## Propagation rule for new patterns` automatically. No AGENTS.md edit required. Taxonomic stretch (file-size vs. textual-shape) is acceptable: the rule is binary like the other patterns. |
| Sub-check 10 sync mechanism | **Dynamic** — Sub-check 10 reads `agent-writing-style.md` at audit time, parses `## ` headings, matches each against an existing Checklist M sub-check, flags unmatched headings as `nit` "audit coverage gap" findings. Future style-guide additions auto-covered without changing `/ai-audit/SKILL.md` (Round 1 answer). |
| Sub-check 9 severity bands | `35,000–39,999` → `minor` (early warning, one `/task` cycle of headroom). `≥ 40,000` → `blocker` (AXIOM violation). No `major` band. |
| Sub-check 9 covered file set | Verbatim from the AGENTS.md AXIOM (enumerated under Scope item 1.ii). Explicit paths in Checklist M; `**` globs expanded at audit time. |
| Sub-check 10 coverage-gap finding format | Severity `nit`. Message: `audit coverage gap: § <heading>`. Proposed action: `add sub-check N+1 to /ai-audit Checklist M`. Headings inside `## Patterns` are NOT individually audited — Sub-check 10 inspects level-2 (`## `) headings only; pattern entries inside `## Patterns` are covered as a group by the existing Pattern-audit sub-check (if one exists) or by Pattern 8's own self-coverage. |
| AGENTS.md size headroom | AGENTS.md is at 39,960 chars (40 below the hard cap). This task does NOT edit AGENTS.md, so the 40-char headroom is not a binding constraint for this task. If the design phase later surfaces a reason to touch AGENTS.md, the same commit must include an extraction pass keeping AGENTS.md below 35,000 chars at every commit boundary. |
| Cross-reference between AGENTS.md AXIOM and the new Pattern 8 | Pattern 8 references `AGENTS.md § Build & Test` as the source-of-truth. No back-reference is added to AGENTS.md (no AGENTS.md edit in scope). A future incidental edit to AGENTS.md may add an anchored back-reference as a follow-up. |
| Citation of PR #324 | Pattern 8 cites PR #324 by number as the canonical extraction example. No verbatim duplication of #324's diff body. |
| Demonstrator AC | AC5 verifies that running `/ai-audit` against the post-merge corpus surfaces AGENTS.md (39,960 chars) as a `minor` Sub-check 9 finding, demonstrating the early-warning band fires end-to-end. |

## Technical constraints

- **AGENTS.md size headroom (informational, non-binding for this task).** AGENTS.md is at 39,960 chars (40 below the hard cap). This task does NOT edit AGENTS.md. If the design phase surfaces a reason to touch AGENTS.md, the editing commit MUST be paired with an extraction pass keeping AGENTS.md below 35,000 chars at every commit boundary.
- **`ai-docs/agent-writing-style.md` size budget.** Currently 10,992 chars. Pattern 8 will add roughly 30–60 lines (threshold table + covered file enumeration + extraction-model citation + per-commit note). Comfortably below 35k post-edit.
- **`.claude/skills/ai-audit/SKILL.md` size budget.** Currently 22,482 chars. Sub-check 9 + Sub-check 10 will add roughly 40–80 lines (table rows + supporting prose + the heading-parse recipe). Comfortably below 35k post-edit.
- **Propagation Rule.** Edits to `ai-docs/agent-writing-style.md`'s `## Patterns` section MUST be paired with the grep sweep documented in `agent-writing-style.md § Propagation rule for new patterns`. Pattern 8 itself does NOT mandate per-consumer adoption (the char-cap is mechanical, not a writing pattern that consumers must mirror) — but the grep MUST still run to confirm no consumer file half-uses or contradicts the new pattern.
- **No new pre-resolved-rule entries for `.claude/agents/spec-writer.md`.** The char-cap is a structural property of files, not a question-time pre-resolved rule.
- **Pattern 8 self-conformance.** The new Pattern 8 entry MUST itself respect the existing patterns in `agent-writing-style.md` (AXIOM blockquote per Pattern 1, action table per Pattern 1's tabular format, explicit file enumeration per Pattern 4, bold-uppercase fail-loud verbs per Pattern 2). Self-violation in the introducing entry would be ironic and audit-detectable.
- **Sub-check 9 detection is self-contained.** The `wc -c` command and the covered-file set are inline in Checklist M. The audit does NOT shell out to `scripts/check-instruction-file-sizes.sh` (which doesn't exist yet — that's #383).
- **Sub-check 10 heading parser.** The audit-time parser MUST handle: (a) ATX-style level-2 headings only (`## `); (b) headings inside fenced code blocks MUST be ignored (false-positive class); (c) heading text after a `# ` prefix is the lookup key against Checklist M's sub-check titles. Implementation detail (parser strictness, fuzzy matching) is design-phase scope.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ai-docs/agent-writing-style.md` carries the 40k char-cap rule as **Pattern 8** under `## Patterns`. The entry includes: (i) the threshold table (`< 35k` / `35k–39,999` / `≥ 40k`), (ii) the explicitly enumerated covered file set (per Scope 1.ii), (iii) the extraction-model citation of PR #324, and (iv) the per-commit invariant note. |
| AC2 | `.claude/skills/ai-audit/SKILL.md` Checklist M is extended with **Sub-check 9** (file-size AXIOM conformance). Sub-check 9 enumerates the covered file set, specifies `wc -c` as the detection mechanism, and assigns `minor` to `35,000–39,999` and `blocker` to `≥ 40,000`. |
| AC3 | `.claude/skills/ai-audit/SKILL.md` Checklist M is extended with **Sub-check 10** (style-guide audit coverage map). Sub-check 10 reads `ai-docs/agent-writing-style.md` at audit time, parses `## ` headings, matches each against an existing Checklist M sub-check (1–9), and emits a `nit` finding `audit coverage gap: § <heading>` with proposed action `add sub-check N+1 to /ai-audit Checklist M` for any unmatched heading. |
| AC4 | Propagation Rule satisfied: the grep sweep documented in `agent-writing-style.md § Propagation rule for new patterns` is run against `.claude/agents/`, `.claude/skills/`, `AGENTS.md`, and `ai-docs/agent-writing-style.md` itself; no consumer file half-uses or contradicts Pattern 8. |
| AC5 | Manual demonstrator: running `/ai-audit` (Phase 2, Checklist M) against the post-merge corpus produces a `minor`-severity Sub-check 9 finding for AGENTS.md = 39,960 chars (current size). The finding text identifies AGENTS.md by path and reports its char count. |
| AC6 | Every covered file is below the hard cap (`< 40,000` chars) at every commit boundary in the implementation PR. Verified by running `wc -c` on the covered file set at HEAD of each commit on the feature branch (not just merge-time HEAD). |
| AC7 | Doc gate clean: `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` passes; `cargo fmt -- --check` and `cargo clippy --workspace -- -D warnings` pass (no-op expected — this task touches markdown only). |
| AC8 | `actionlint` not required (no workflow files touched in expected diff). If the implementation discovers a workflow file needs editing, `actionlint <file>` MUST pass before `git add` per AGENTS.md. |

## Open questions

- **Q1 — Should Pattern 8 cross-reference the deferred `scripts/check-instruction-file-sizes.sh` (issue #383)?** The pre-commit / CI mechanical gate is the long-term enforcement; the audit-side Sub-check 9 is a back-stop. Pattern 8 could either (a) mention #383 by issue number as the planned mechanical gate, leaving the audit as the interim back-stop; or (b) stay silent on #383 and let the deferred script's own implementation PR add the cross-reference when it lands. **Default if not asked:** option (a) — one-line forward reference (`Mechanical pre-commit gate planned in #383; this audit-side check is the interim back-stop`). Cheap, signposts the symbiosis between the two enforcement layers, and doesn't lock in #383's scope. The user may revisit at design-review time or after #383 lands.
