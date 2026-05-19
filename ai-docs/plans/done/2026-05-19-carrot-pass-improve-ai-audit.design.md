# Design: Carrot Pass for `/improve` and `/ai-audit` — PR 3 (Phase 5)

**Issue:** #491
**Spec:** `ai-docs/plans/2026-05-19-carrot-pass-improve-ai-audit.spec.md`
**Date:** 2026-05-19
**Prior PRs (merged):**
- #492 @ `7cc06c6` — Phase 1 (schema migration, section rename to *Learning Log*, worked-example retro-tag). Design preserved at `ai-docs/plans/2026-05-19-carrot-pass-improve-ai-audit.phase1.design.md`.
- #493 @ `ce549a6` — Phases 2+3+4 (Carrot pass parallel Step 1b/2b + 🌱 Stale-validation verdict + Checklist N + threshold reframe + AC12 worked-example block). Design preserved at `ai-docs/plans/2026-05-19-carrot-pass-improve-ai-audit.phase2-3-4.design.md`.

PR-3 is the **final PR** of the series. After merge, issue #491 closes.

## Approach

Phase 5 cross-feeds the user-local auto-memory layer (`~/.claude/projects/<project-path-encoded>/memory/feedback_*.md`) into `/improve` as a **companion signal**, not a primary one. The `self-improve` subagent gains a new **Step 1c — Auto-memory companion sweep** that runs alongside the existing Correction pass (Step 1/2a) and Carrot pass (Step 1b/2b). Step 1c is **read-only against the user-local layer** and produces candidates surfaced as a dedicated `## Auto-memory candidates` block in the subagent's report.

The decisive design choice for AC16: **all project-side writes derived from auto-memory go through an `AskUserQuestion` prompt issued by the `/improve` skill (parent) thread, NOT by the subagent.** The subagent surfaces candidates as text; the parent thread reads the subagent's report, dispatches one `AskUserQuestion` per candidate, and only on explicit `yes` proceeds to a normal Step-2 routing decision (which `## Patterns` section, which back-link form). On `no` or `defer`, the candidate is dropped from this `/improve` run — not persisted, not silently retained. This mirrors `interview/SKILL.md`'s existing pattern: subagent emits structured output; parent surfaces via `AskUserQuestion`. The consent moment becomes part of the existing skill-level surface, not a new sub-protocol.

The decisive design choice for AC16b (write guard): **enforcement is convention-only + audit-side check, NOT a hook.** The reasoning is named below in *Chosen alternatives — Write guard mechanism*; the short form is that a hook attempting to pattern-match "this edit derives from an auto-memory entry" has no reliable signal (the edit looks identical to a normal Carrot-pass edit), and adding a hook for the absence of consent would block legitimate Carrot-pass edits as a side effect. The guard relies on three layered conventions: (1) the `self-improve` agent prose explicitly forbids project-side writes from Step 1c without a `yes` from the consent prompt; (2) the `/improve` skill body documents the consent step as the gate; (3) `/ai-audit` Phase 2 already has Checklist N (bidirectional `## Patterns` ↔ `Kind: validation`) — an auto-memory-derived `## Patterns` entry without a `Kind: validation` back-link would flag, providing a passive post-hoc check.

The **privacy boundary** is enforced by three explicit prohibitions in `self-improve.md`:

1. `self-improve` reads `~/.claude/projects/<project-path-encoded>/memory/feedback_*.md` and `MEMORY.md` (index) but writes nothing to that directory.
2. `self-improve` does not paraphrase, quote, or import auto-memory content into instruction files based solely on the auto-memory entry — a matching `Kind: validation` entry in `learnings.md` must already exist, OR the user must explicitly approve creating one via the consent UX.
3. The `self-improve` report's auto-memory section is structured (filename + named primitive + cross-check verdict), not free-form excerpts. The subagent does not include verbatim auto-memory text in its report unless quoting one or two short lines is necessary for the user to recognise the candidate.

### Chosen alternatives

**Step 1c parallel to Step 1/1b (NOT interleaved into Step 2b's routing table).** Auto-memory is a separate **source** (user-local) distinct from `learnings.md` (project-side); folding it into Step 2b's table would conflate signal provenance. A parallel Step 1c + a paired routing decision at Step 2c (with a single row — "name workflow primitive AND no matching `Kind: validation` → surface as candidate; needs consent") keeps the source asymmetry visible. The Carrot pass's Step 2b table is untouched. Rejected: a fourth row inside Step 2b — couples carrot promotion (already-in-`learnings.md`) with the auto-memory cross-check (not-yet-in-`learnings.md`); the routing reasoning is genuinely different.

**Consent UX: `AskUserQuestion` from the `/improve` SKILL parent thread, one per candidate.** Three options were considered. (a) **Subagent emits `AskUserQuestion` directly** — rejected; subagents do not have the `AskUserQuestion` tool by convention (the `interview` subagent path also returns structured YAML for the orchestrator to surface). (b) **Single batched `AskUserQuestion` listing all candidates** — `AskUserQuestion` is capped at 4 questions per call (per `interview/SKILL.md` constraint), so a batched form caps out at 4 candidates per round and silently drops the rest. Rejected — the cap is a hard runtime ceiling, not a soft preference. (c) **One `AskUserQuestion` per candidate, dispatched sequentially by the parent thread after reading the subagent's report.** Chosen — preserves consent granularity (each candidate carries its own context), respects the per-call cap (each call has 1 question, well below 4), and matches `interview/SKILL.md`'s existing parent-side surfacing pattern. The `header` (≤ 12 chars) and `options` (2–4 entries) constraints fit naturally: header = `auto-memory`; options = `Surface` / `Drop` / `Defer`.

Concrete prompt wording (the literal text the `/improve` parent thread emits per candidate):

```
question: "Auto-memory entry `feedback_<name>.md` names workflow primitive `<primitive>` with no matching `Kind: validation` entry in `ai-docs/learnings.md`. Surface as a /improve candidate (would seed a `## Patterns` entry in `<target-skill-or-agent>` after the next user-approved Carrot-pass step)?"
header: "auto-memory"
options:
  - label: "Surface"
    description: "Add to this run's Carrot-pass candidate list; routes through normal Step 2b table — which may still hold for second confirmation if no matching `Kind: validation` entry is created in this turn."
  - label: "Drop"
    description: "Skip this candidate for this /improve run. Auto-memory entry is unchanged; no project-side write."
  - label: "Defer"
    description: "Skip for this run only; do not write any opt-out flag. Re-surfaces on next /improve invocation."
```

Rejected wording variants: a fourth `Never` option (would require writing an opt-out file to the user-local layer, violating the no-write-back boundary); a `Promote directly` shortcut (would skip Step 2b's normal threshold table and the Step 6 inverted-eval). The three-option shape is deliberate — `Surface` triggers the normal flow, `Drop` is the dispose-this-run choice, `Defer` is the dispose-but-resurface choice. Both `Drop` and `Defer` are local-only state (the subagent re-discovers the candidate on the next `/improve` run unless the user creates a matching `Kind: validation` entry that resolves the cross-check).

**Prompt does NOT carry timestamp.** The feedback files carry `originSessionId` and filesystem mtime, but the prompt deliberately omits both — the filename + named primitive + cross-check verdict is enough to identify the candidate. Surfacing a timestamp would inflate the prompt and add a "stale-or-not" sub-decision the user does not need at consent time. Staleness of auto-memory entries is a user-side concern; `/improve` does not own that decision (Checklist N's `🌱 Stale-validation` verdict only applies to `Kind: validation` entries, which auto-memory entries are not).

**Defer state holds for the entire `/improve` invocation; re-surfaces on next run only.** The parent thread tracks consent decisions (`Drop`, `Defer`) in its working memory for the duration of one `/improve` invocation. If `/improve` makes multiple Carrot-pass routing decisions in the same run, a candidate dropped or deferred earlier is NOT re-prompted later in that run. Re-surfacing happens on the NEXT `/improve` invocation only — no persistent state file is written to the project layer or the user-local layer (consistent with the no-write-back-to-auto-memory boundary, and no Phase-5-state file in the project either).

**Write guard mechanism: convention + audit-side passive check (NOT a hook).** Three options were considered. (a) **`PreToolUse` hook on `Edit`/`Write` matching `\.claude/` or `AGENTS\.md` paths, blocking if a marker is missing.** Rejected — there is no reliable mechanical signal to distinguish "this Edit derived from auto-memory" from "this Edit derived from a regular Carrot-pass `learnings.md` walk". A hook would either block all Carrot-pass writes (too aggressive) or rely on an opt-in marker the subagent self-attaches (no enforcement). (b) **A `settings.local.json` flag `permissions.deny` entry for auto-memory paths.** Rejected — `settings.local.json` is user-local; the project layer cannot pin a setting there reliably across contributors. (c) **Three-layered convention check: agent prose + skill prose + Checklist N audit-side validation.** Chosen — the prose contract names the rule explicitly; Checklist N (already present from PR-2) provides the post-hoc audit signal — a `## Patterns` block without a `Kind: validation` back-link is flagged at severity `major`, so any auto-memory-derived write that escaped consent would surface at the next `/ai-audit` run as a Checklist N forward-direction failure (the `## Patterns` block has no back-link because no `Kind: validation` exists for it). The audit's existing carrot-verb filter (Subtask 4 of PR-2) handles the carrier-vs-template carve-out without modification.

The convention check is named at three sites:

1. `.claude/agents/self-improve.md` § Step 1c — explicit "DO NOT write any instruction file based on a Step-1c candidate alone; routing happens only after the parent thread relays `Surface` consent" line.
2. `.claude/skills/improve/SKILL.md` body — new short section *Auto-memory consent gate* documents that the parent thread holds the consent dispatch and only `Surface` consent unlocks routing.
3. The subagent's report-template anti-pattern list — explicit "Do NOT include verbatim auto-memory text in `## Auto-memory candidates` rows beyond ≤ 2 short lines necessary for user recognition".

**Detection recipe — what "names a workflow primitive" means.** The spec leaves this phrase open; the design pins it to: a `feedback_*.md` whose **`name:`** (frontmatter) OR **`description:`** (frontmatter) OR the file's **first sentence** mentions:

- A slash command (`/task`, `/improve`, `/ai-audit`, `/pr-commented`, `/context-reset`, `/bugfix`, `/interview`, `/triage`, `/code-review`, `/master-ci-failed`, `/pr-ci-failed`, `/pr-merged`, `/next`, `/loop`, `/schedule`, `/init`, `/review`, `/security-review`, `/ui-design`, `/verify`) — i.e., the slash-command names of the skills under `.claude/skills/`.
- An agent name (`spec-writer`, `design`, `design-review`, `review-findings`, `self-review`, `self-improve`, `learnings-escalation-audit`, `triage-runner`) — i.e., file stems under `.claude/agents/`.
- An AGENTS.md section heading (`## Workflow`, `## Code Style`, `## Communication`, `## Dependency Versions`, `## Permissions`, `## API Stability`, `## API Naming`, `## Propagation Rule`, `## Learning Log`, `## Rust Test Conventions`).
- A primitive verb-phrase keyword the project already tracks: `compaction recovery`, `Boundary rule 1`, `Boundary rule 2`, `Spec Amendment recipe`, `every-group handoff`, `AskUserQuestion`, `Propagation Rule`, `Carrot pass`, `Correction pass`, `🌱 Stale-validation`.

The detection is greppable (the named slash commands + agent stems + section headings + verb-phrase keywords form a closed set). `self-improve.md` Step 1c body enumerates them in a fenced code block — same shape as the verb-set enumeration introduced in PR-2 — so the audit can target a stable anchor (`#auto-memory-primitive-keywords`). New skills / agents added later require an additive update; the detection list is **not** auto-generated from `.claude/` listings (over-broad — would match incidental references in unrelated feedback memories).

**Cross-check against `learnings.md` — match by `Kind: validation` + primitive name in `Rule:` or `description:`.** A candidate clears the "no matching entry" predicate when: there is no `Kind: validation` entry in `ai-docs/learnings.md` whose `### YYYY-MM-DD — [category] — [short description]` heading OR `Rule:` field mentions the same workflow primitive (substring match, case-insensitive). Match by the **specific primitive named in the auto-memory entry**, not by topic — a feedback memory naming `/context-reset` matches only a `Kind: validation` entry that itself names `/context-reset` somewhere. The 2026-05-19 *compaction-recovery protocol in skill files works* entry already names `/context-reset`, `/task`, `/pr-commented`, `/code-review`, `/bugfix`, `/interview`; so the existing `feedback_compaction_recovery.md` (which mentions all of them) is fully covered → no candidate. The detection is **per-primitive**, so a single auto-memory entry naming N primitives can produce up to N candidates if only some are covered by validation entries; the design recommends collapsing per-entry (1 prompt per `feedback_*.md`, not per primitive) to keep the consent UI legible — the prompt lists the uncovered primitives in its question text.

**Reporting shape — dedicated `## Auto-memory candidates` block, NOT a row in `## Carrots proposed`.** Two reasons: (1) source-asymmetry visibility (the existing two-section split — `## Corrections proposed` / `## Carrots proposed` — already encodes signal direction; adding a third section for the third source type extends the same pattern); (2) consent-gating shape — `## Carrots proposed` rows already imply user approval to land in this `/improve` run; `## Auto-memory candidates` rows are pre-consent and need different language (`needs user consent before promotion`). The subagent emits both sections in its report; the parent thread reads `## Auto-memory candidates`, dispatches one `AskUserQuestion` per row, and on `Surface` consent moves the row into `## Carrots proposed` for routing in the same `/improve` turn.

Row shape (subagent output):

```
## Auto-memory candidates

| Auto-memory file | Workflow primitive named | Cross-check verdict | Suggested `## Patterns` target |
|---|---|---|---|
| `feedback_X.md` | `<primitive-name>` | no `Kind: validation` in `learnings.md` mentions `<primitive-name>` | `<skill | agent | AGENTS.md section>` (most-local owner) |
```

The "Suggested `## Patterns` target" column applies the same most-local-target routing the Carrot pass already uses (skill > agent > AGENTS.md) — the suggestion is informational; the parent thread does NOT execute it without `Surface` consent.

**No write-back to auto-memory — explicit "do not write to `~/.claude/projects/.../memory/*`" anti-pattern.** Added to `self-improve.md` Anti-patterns list as a new bullet. The directory path is named verbatim so future contributors cannot miss it. The Defer path uses **in-memory** state — the subagent does not write a `.improve_defer` file or any other marker; deferred candidates re-surface on every `/improve` run until the user creates a `Kind: validation` entry or chooses `Drop`. Rejected (per spec § Out of scope): a `.improve_optout` flag in `~/.claude/projects/.../memory/` for `Never`-this-entry consent — the spec explicitly out-of-scopes any write-back; the `Defer` option's resurface-on-next-run behaviour replaces the need for a persistent `Never` option.

**Step 6 (eval) for auto-memory candidates — reuses existing pause-and-surface protocol, NO new reproducer-template variant.** Auto-memory candidates are surfaced as `## Auto-memory candidates` rows; on `Surface` consent they flow into `## Carrots proposed` and inherit the Kind: validation Step-6 reproducer template (the carrot variant with the "in scenario X, does the pattern still hold?" form). On `Drop` or `Defer` consent, the candidate is dropped from this run and the eval step has nothing to reproduce for that row. No new reproducer skeleton needed.

**Phase 5 NOT folded into PR-2 (already merged).** Per the spec's PR-slicing key decision, the option to fold was always available; the PR-2 design (line 46) explicitly recommended keeping Phase 5 as PR-3 because (a) the consent-UX surface is independent from Phases 2–4, (b) the privacy boundary warrants its own design-review pass. PR-2 merged @ `ce549a6` without folding, so AC17 is automatically satisfied as "follow-up PR".

**No AGENTS.md edit in this PR.** Confirmed by walking each AC + each subtask body: Phase 5 affects `self-improve.md` and `improve/SKILL.md` only. AGENTS.md's `## Learning Log` threshold line already names "≥3 unescalated correction entries, ≥2 unescalated validation entries, or a 🌱 Stale-validation flag" (Phase 4, PR-2 Subtask 5) — auto-memory candidates are NOT a fourth threshold trigger (they appear during `/improve`, they don't trigger it). Char-cap pressure (37,673 chars post-PR-2; 35,000–39,999 early-warning band) reinforces the "do not touch AGENTS.md for Phase 5" decision; recorded as the first Risk row.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | **Phase 5a — Add `### Step 1c — Auto-memory companion sweep` to `self-improve.md`.** Insert directly after the existing Step 1b (Carrot pass) section and before Step 2a (Correction pass routing). Body contains: (a) the directory path the subagent reads (`~/.claude/projects/<project-path-encoded>/memory/feedback_*.md` for full content AND `MEMORY.md` as the index) — read **both**: `MEMORY.md` first for fast filename enumeration (avoids a blind `ls`), then the individual `feedback_*.md` files because the detection rule operates on `name:` / `description:` / first-sentence, which only the per-file content carries; cost is minimal (10 small files today); (b) the **detection recipe** — fenced code block enumerating workflow primitives the sweep recognises (slash commands, agent stems, AGENTS.md sections, verb-phrase keywords) — referenced from elsewhere via the HTML-comment anchor `<!-- anchor: auto-memory-primitive-keywords -->` placed on the line immediately above the fenced block (chosen mechanism: HTML-comment anchor, NOT a heading-derived anchor, because the fenced block lives inside Step 1c's body — there is no dedicated sub-heading and adding one purely for anchoring would inflate the section's TOC weight); (c) the **cross-check** against `learnings.md` (substring match, case-insensitive, primitive-name level); (d) the **anti-pattern list** addition — three "DO NOT" lines: write to the user-local memory directory, paraphrase auto-memory text beyond ≤ 2 short lines, execute any routing without parent-thread `Surface` consent; (e) reference back to the new `### Step 2c — Auto-memory candidate surfacing` (paired routing decision, Subtask 2). | `.claude/agents/self-improve.md` | — |
| 2 | **Phase 5b — Add `### Step 2c — Auto-memory candidate surfacing` table + report-shape to `self-improve.md`.** Insert directly after Step 2b's routing table and before the `### Promotion verbs` section. Body contains: (a) the single-row routing table — `1 + named workflow primitive + no matching Kind: validation → surface as candidate; needs parent-thread consent`; (b) the report-section shape — `## Auto-memory candidates` table with columns `Auto-memory file` / `Workflow primitive named` / `Cross-check verdict` / `Suggested ## Patterns target`; (c) the per-row collapse rule (one row per `feedback_*.md`, listing all uncovered primitives in the cross-check column when multiple); (d) explicit "the parent thread holds the consent dispatch via `AskUserQuestion`; this subagent emits the table and yields" line. Add the no-write-back anti-pattern to the existing `## Anti-patterns` list — verbatim path `~/.claude/projects/<project-path-encoded>/memory/*`. | `.claude/agents/self-improve.md` | 1 |
| 3 | **Phase 5c — Add `## Auto-memory consent gate` section + literal prompt to `improve/SKILL.md`.** Insert after the existing 6-numbered list and the threshold restate line, before the `See also:` link. Body contains: (a) one-sentence explanation that auto-memory is a companion signal surfaced during `/improve` and the parent thread holds the consent dispatch; (b) the literal `AskUserQuestion` prompt — verbatim `question` / `header` / 3-option `options` block reproduced from the Approach section's *Consent UX* paragraph; (c) the consent-routing rule — `Surface` consent moves the row into `## Carrots proposed` for normal Step 2b routing; `Drop` and `Defer` drop the row from this run with no persistent state; (d) the privacy boundary — single line "Project-side `/improve` writes NEVER originate from auto-memory alone; the consent prompt is the gate". Char-cap check: this section adds ~600 chars to a 1,322-char file (file becomes ~1,900 chars) — well below the early-warning band. | `.claude/skills/improve/SKILL.md` | — |
| 4 | **Verification sweep + Rust gates.** Run AC18's four Rust gates (`cargo build`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -- --check`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`). Run `wc -c AGENTS.md` to confirm no incidental drift (expected unchanged — Phase 5 does not touch AGENTS.md). Run `wc -c .claude/agents/self-improve.md .claude/skills/improve/SKILL.md` to confirm no file exceeds the 35,000-char early-warning band. Run a precision-grep sweep to verify the privacy boundary's three prohibitions are present: `grep -n "DO NOT" .claude/agents/self-improve.md \| grep -E "user-local\|feedback_\|memory directory"` returns at least three hits; `grep -n "consent" .claude/skills/improve/SKILL.md` returns at least one hit naming the gate. Run the Propagation-Rule fan-out check: `grep -rn "auto.memory\|companion signal" .claude/ AGENTS.md ai-docs/ --exclude='2026-05-19-carrot-pass-improve-ai-audit.*'` — the exclusion drops the spec / design / progress files for this task (which would otherwise spam the output with their own mentions of "auto-memory" and "companion signal"); the only new hits should be in `self-improve.md` and `improve/SKILL.md` (no incidental references in other agents/skills/AGENTS.md). | (verification, no edits) | 1, 2, 3 |

## Handoff plan

4 subtasks → groups of 3 + 1.

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). The parent `/task` resumes Step 8 in Group A's fresh-context subagent.
- **Group A:** subtasks 1–3 — `self-improve.md` Step 1c + Step 2c (Subtasks 1–2) AND `improve/SKILL.md` consent gate section (Subtask 3). (3 subtasks, equal to the cap.)
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). The parent `/task` resumes Step 8 in Group B with fresh context.
- **Group B:** subtask 4 — verification sweep (Rust gates, char-cap check, prohibition-presence grep, fan-out grep). Terminal group (1 subtask; within the 1..=3 range).

## File-touch map

| File | What changes | Phase | AC coverage |
|---|---|---|---|
| `.claude/agents/self-improve.md` | New `### Step 1c — Auto-memory companion sweep` body (~600 chars: detection recipe + cross-check + prohibitions); new `### Step 2c — Auto-memory candidate surfacing` routing table + report-section shape (~500 chars); new bullet in `## Anti-patterns` for the no-write-back rule (~120 chars) | 5 | AC16a (consent UX boundary named on subagent side), AC16b (write guard prose) |
| `.claude/skills/improve/SKILL.md` | New `## Auto-memory consent gate` section (~600 chars) with the literal `AskUserQuestion` prompt + consent-routing rule + privacy-boundary one-liner | 5 | AC16a (literal `AskUserQuestion` prompt), AC16b (gate documented at orchestrator surface) |
| (none — verification only) | Rust gates + char-cap check + prohibition-presence grep + fan-out grep | AC18 | AC18 |

**Sync-group fan-out check.** The two touched files are NOT in a documented sync group (no Propagation-Rule row binds them). However, the spec § Phase 5 body names BOTH files; verification sweep (Subtask 4) grep-checks that both files received the corresponding edits. AGENTS.md is intentionally untouched (Phase 5 is the only phase whose subtask body explicitly excludes AGENTS.md edits, per the spec body).

**No AGENTS.md edit.** Confirmed three times across the Approach + Decomposition + this map. The 37,673-char early-warning band stays untouched.

**No `learnings-escalation-audit.md` or `ai-audit/SKILL.md` edit.** Auto-memory is NOT part of the audit corpus (the audit walks `ai-docs/learnings.md` only). Phase 5's signal is per-run, surfaced in `/improve` only — never persisted to project state, so the audit has nothing to verify on the auto-memory side.

## Risks

- **Risk 1: AGENTS.md char-cap pressure not relieved.** AGENTS.md is at 37,673 chars post-PR-2 (35,000–39,999 early-warning band). Phase 5 does not touch AGENTS.md per design, so the band stays unchanged for this PR — but the band itself is a standing concern. **Mitigation:** the design explicitly forbids AGENTS.md edits in this PR; Subtask 4's verification sweep includes `wc -c AGENTS.md` to confirm no incidental drift. Standing concern (relieving the band) is out of scope for this PR — a separate `/improve` or AGENTS.md-shrink task.
- **Risk 2: detection recipe false positives.** The "names a workflow primitive" predicate uses substring matching against the enumerated keyword set. A feedback memory whose `description:` mentions `/task` incidentally (e.g., "task" as English word in a non-workflow context) would match. **Mitigation:** the keyword set is closed and enumerated — slash commands carry a literal `/` prefix, agent stems are specific filenames, AGENTS.md sections are `## `-prefixed headings, verb-phrase keywords are multi-word strings. Pure-English "task" would not match `/task` (the `/` prefix differentiates). False-positive rate is bounded by the consent UX — the user can `Drop` any spurious candidate; spurious candidates do not persist.
- **Risk 3: detection recipe false negatives.** A feedback memory naming a workflow primitive NOT in the enumerated keyword set (e.g., a primitive the user introduced ad-hoc) is not detected. **Mitigation:** new skills / agents must update the enumeration (additive); the Anti-patterns list in `self-improve.md` adds an explicit "if a new skill / agent / section is added to the project, append its slash command / file stem / section heading to the Step 1c primitive-keyword block" line. Not auto-generated — over-broad detection would be worse than under-broad detection.
- **Risk 4: consent prompt fatigue.** A `/improve` run with N candidates triggers N `AskUserQuestion` prompts (sequential). **Mitigation:** the per-feedback-file collapse rule (one row per `feedback_*.md`, listing all uncovered primitives in the cross-check column) caps the per-run prompt count at the number of `feedback_*.md` files (currently 10). The `Defer` option lets the user batch-defer through a session; the candidates re-surface on the next `/improve` run. If prompt-count grows past ~5, a future optimization could batch into a single `AskUserQuestion` with multiple questions (per `AskUserQuestion`'s 4-question-per-call cap — exceeding requires multiple calls anyway). Deferred — not implemented in this PR.
- **Risk 5: subagent leaks verbatim auto-memory text into the report.** The anti-pattern list forbids quoting more than ≤ 2 short lines, but enforcement is convention. **Mitigation:** the report's structured row shape (filename + primitive + verdict + suggested target) does not invite quoting; the few-line carve-out is for cases where the user needs the quote to recognise the candidate. Audit-side: `/ai-audit` does NOT scan `~/.claude/projects/.../memory/*` (auto-memory is out of the audit corpus per spec); the only protection is the subagent prose contract.
- **Risk 6: `## Patterns` target column drift.** Step 1c suggests a target (`skill | agent | AGENTS.md section`) using the most-local-owner rule. If the suggestion is wrong, the parent thread on `Surface` consent routes the resulting Carrot-pass entry to the wrong file. **Mitigation:** the suggestion is informational, NOT executed without user-approved routing — Step 2b's normal routing reasoning (most-local-target) re-runs after consent. The parent thread can override the suggestion based on context.
- **Risk 7: Checklist N audit lag.** A hypothetical bypass (auto-memory-derived `## Patterns` block landing without the consent prompt firing — e.g., contributor manually copying from a feedback memory) only surfaces at the next `/ai-audit` run, which may be days or weeks later. **Mitigation:** convention enforcement is layered at three sites (self-improve.md prose, improve/SKILL.md prose, the subagent's anti-pattern list). The audit lag is acknowledged as a residual risk; a hook-based gate was considered (Approach § Write guard) and rejected for lack of reliable signal. Accept the risk per spec § AC16b — the spec says "no automatic writes based on auto-memory alone", which the convention contract satisfies.

## Test Design

Instruction-surface changes only — no Rust code changes — so "tests" are grep + structural checks. AC18 is the four Rust gates which must pass even though no `.rs` file changed.

### Subtask 1 (Step 1c body in `self-improve.md`)
- **Location:** `.claude/agents/self-improve.md` § Step 1c (new section between existing Step 1b and Step 2a)
- **Verification:**
  - `grep -n "Step 1c — Auto-memory companion sweep" .claude/agents/self-improve.md` returns exactly 1 hit.
  - `grep -n "feedback_\*.md\|~/.claude/projects" .claude/agents/self-improve.md` returns hits inside Step 1c (path enumeration present).
  - `grep -nE "DO NOT" .claude/agents/self-improve.md | grep -E "user-local|feedback_|memory directory"` returns ≥3 hits (the three prohibitions).
  - The primitive-keyword fenced block contains the closed set named in the Approach (slash commands + agent stems + section headings + verb-phrase keywords); `grep -n "auto-memory-primitive-keywords" .claude/agents/self-improve.md` returns 1 hit (anchor comment present).
- **Scenarios:**
  - Happy path: a `feedback_X.md` whose `description:` names `/foo`; Step 1c's detection rule fires → candidate emitted.
  - Negative: a `feedback_X.md` whose `description:` is generic English ("about X feature"); detection rule does not fire → no candidate.
  - Cross-check positive: a `feedback_X.md` names `/context-reset` AND a `Kind: validation` entry in `learnings.md` mentions `/context-reset` → cross-check passes → no candidate (already covered).
  - Cross-check negative: a `feedback_Y.md` names `/foo` AND no `Kind: validation` entry mentions `/foo` → cross-check fails → candidate emitted.

### Subtask 2 (Step 2c + report shape + Anti-patterns in `self-improve.md`)
- **Location:** `.claude/agents/self-improve.md` § Step 2c (between Step 2b and `### Promotion verbs`); § Anti-patterns (new bullet)
- **Verification:**
  - `grep -n "Step 2c — Auto-memory candidate surfacing\|## Auto-memory candidates" .claude/agents/self-improve.md` returns ≥2 hits (heading + report shape).
  - The routing table contains exactly 1 row (the `1 + named primitive + no matching Kind: validation` row).
  - The report shape table has exactly 4 columns: Auto-memory file / Workflow primitive named / Cross-check verdict / Suggested `## Patterns` target.
  - `grep -n "parent thread holds the consent dispatch" .claude/agents/self-improve.md` returns 1 hit.
  - Anti-patterns: `grep -nA1 "Do NOT" .claude/agents/self-improve.md | grep "~/.claude/projects/.*memory"` returns 1 hit (no-write-back rule present).
- **Scenarios:**
  - Happy path: subagent emits a `## Auto-memory candidates` row for an uncovered primitive; parent thread reads the row and dispatches a single `AskUserQuestion`.
  - Collapse: a `feedback_X.md` naming three primitives (one covered, two uncovered) produces ONE row in the table listing the two uncovered primitives in the cross-check column.

### Subtask 3 (Consent gate in `improve/SKILL.md`)
- **Location:** `.claude/skills/improve/SKILL.md` § Auto-memory consent gate (new section between the 6-numbered list and `See also:`)
- **Verification:**
  - `grep -n "Auto-memory consent gate" .claude/skills/improve/SKILL.md` returns 1 hit.
  - The literal `AskUserQuestion` prompt is reproduced verbatim — `grep -nE "header.*auto-memory|Surface|Drop|Defer" .claude/skills/improve/SKILL.md` returns ≥4 hits (header + 3 options).
  - `grep -n "consent" .claude/skills/improve/SKILL.md` returns ≥2 hits (gate + routing rule).
  - The privacy-boundary one-liner is present: `grep -n "NEVER originate from auto-memory alone" .claude/skills/improve/SKILL.md` returns 1 hit.
- **Scenarios:**
  - `Surface` consent: row moves into `## Carrots proposed` for routing.
  - `Drop` consent: row dropped, no project-side write, no auto-memory write.
  - `Defer` consent: row dropped from THIS run, no auto-memory write, will re-surface on next `/improve`.

### Subtask 4 (Verification sweep)
- **Entry point:** Rust gates + char-cap + prohibition-presence + fan-out grep; no edits, just confirmations.
- **Scenarios:**
  - All four AC18 commands return 0.
  - `wc -c AGENTS.md` returns the same value as pre-PR-3 (37,673 chars) — no incidental edit drift.
  - `wc -c .claude/agents/self-improve.md` is below 35,000 chars (current 15,799; post-PR-3 expected ~17,000).
  - `wc -c .claude/skills/improve/SKILL.md` is below 35,000 chars (current 1,322; post-PR-3 expected ~1,900).
  - `grep -rn "auto.memory\|companion signal" .claude/ AGENTS.md ai-docs/ --exclude='2026-05-19-carrot-pass-improve-ai-audit.*'` — new hits ONLY in `self-improve.md` and `improve/SKILL.md` (verify by diffing the output against pre-PR-3 baseline). The `--exclude` drops the spec / design / progress files for this task (which themselves repeatedly mention "auto-memory" and "companion signal"), eliminating false-positive noise.

## Open questions

All three notes from design-review round 1 (GO with 3 notes + 3 recommendations) resolved; ready for Step 8.

## Phase 5 finalisation note (for /task Step 12, NOT subtask content)

The orchestrator (`/task`) handles these at finalise time:

- **Move to done.** Move spec + .design.md (this file) + .phase1.design.md + .phase2-3-4.design.md to `ai-docs/plans/done/`. The full carrot-pass series completes.
- **INDEX.md row update.** Change the current `🟢 Phases 1+2+3+4 implemented (Phase 5 stacked for PR-3)` row to `✅ implemented (Closes #491)`. The implementation-detail tail of the row gains a Phase-5 sentence (e.g., *PR-3 (#<N>): Phase 5 cross-feed with user-local auto-memory — Step 1c + Step 2c in `self-improve.md` with detection recipe + cross-check + privacy-boundary prohibitions; consent gate in `improve/SKILL.md` with literal `AskUserQuestion` prompt; convention-only write guard backed by Checklist N audit. AC16, AC17, AC18 PASS.*).
- **PR body.** Use `Closes #491` (not `Refs`) so the tracking issue closes on merge.
- **context.md — REQUIRED.** Add a Maintenance plans row. Phase 5 alters how `/improve` works in a user-visible way (the consent prompt issued by the parent thread per candidate), so a one-line row similar to the existing instruction-surface plans (e.g., `escalate-clippy-warns-deny`, `code-style-extraction`) is required to keep the project narrative current. Suggested wording: *carrot-pass-improve-ai-audit — added Carrot pass parallel to the existing Correction pass in `self-improve` + `🌱 Stale-validation` verdict in `learnings-escalation-audit` + Checklist N (bidirectional `## Patterns` ↔ `Kind: validation`) in `/ai-audit` + threshold reframe in AGENTS.md `§ Learning Log` + Phase 5 cross-feed with user-local auto-memory as a companion signal via `AskUserQuestion` consent gate; `## Patterns` block in `/context-reset/SKILL.md` back-linked to the 2026-05-19 worked-example validation entry.*
- **README.md — REQUIRED.** Maintenance plans (cross-cutting) list at lines 184–210 — add a row in lock-step with context.md. The user-visible UX change (consent prompt) makes the README row a hard requirement, not an option. Suggested wording: same as context.md.

These items are at Step 12, not subtask content.

## Quality checklist self-verification

- **Completeness:** all four ACs in scope (AC16, AC17, AC18) are mapped to subtasks. AC16a covered by Subtask 3 (literal `AskUserQuestion` prompt); AC16b covered by Subtasks 1 + 2 + 3 (prose contract at three sites — agent prose + skill prose + anti-pattern). AC17 is automatically satisfied (PR-3 is the follow-up PR; PR-2 merged without folding). AC18 covered by Subtask 4. Two target files listed in File-touch map; no AGENTS.md edit (verified three times). Tasks are atomic — each subtask touches at most 1 file, each is a logically complete edit.
- **Correctness:** the parallel Step-1c shape mirrors the Carrot-pass Step-1b shape already in `self-improve.md` (PR-2) — same parallel-Step pattern, different signal source. The consent UX uses the `AskUserQuestion` shape already in use by `interview/SKILL.md` (`header` ≤ 12 chars, 2–4 options) — verified by reading `interview/SKILL.md` lines 161–164. The convention-only write guard is acknowledged as weaker than a hook; the design rationale (no reliable mechanical signal to detect auto-memory provenance on an `Edit`/`Write`) is named explicitly so design-review can challenge if the trade-off is wrong.
- **Tests:** every non-trivial edit has a verification recipe in the Test Design section (grep keywords, scenarios per AC); AC18 is the Rust-gate suite which fires unconditionally.
- **Risks:** seven risks identified (char-cap drift, detection false positives / negatives, consent fatigue, auto-memory verbatim leak, suggested-target drift, audit lag) — all with mitigations. The convention-only guard (Risk 7) is the residual risk most likely to surface in design-review; the design rationale for choosing convention over hook is reproduced inline.
- **Economy:** no new files, no new agents, no new tools, no new hooks. The consent UX reuses `AskUserQuestion` (already available to the parent thread). The cross-check reuses `learnings.md` parsing (already done by `self-improve` Step 1 + Step 1b). The audit-side passive check reuses Checklist N (already present from PR-2). YAGNI honoured — Phase 5 adds the cross-feed without expanding the surface beyond two files.
- **Handoff plan:** 4 subtasks → 3 + 1, all groups within `1..=3`, non-terminal Group A is exactly 3, Group B is 1 (terminal, in range). `/context-reset` named at every boundary including Group A entry. Compliant with the design's handoff-grouping contract per `.claude/skills/task/SKILL.md` Step 8.

## References

- Spec: `ai-docs/plans/2026-05-19-carrot-pass-improve-ai-audit.spec.md`
- PR-1 design (historical): `ai-docs/plans/2026-05-19-carrot-pass-improve-ai-audit.phase1.design.md`
- PR-2 design (historical): `ai-docs/plans/2026-05-19-carrot-pass-improve-ai-audit.phase2-3-4.design.md`
- `.claude/agents/self-improve.md` (15,799 chars post-PR-2) — Carrot-pass + Step-6 inverted-prompt host (PR-2 Subtasks 1–2); Phase 5 inserts Step 1c + Step 2c after Step 1b/2b.
- `.claude/skills/improve/SKILL.md` (1,322 chars post-PR-2) — orchestrator skill body with the 6-numbered subagent task list + threshold restate; Phase 5 adds the `## Auto-memory consent gate` section.
- `.claude/skills/interview/SKILL.md` lines 161–164 — the `AskUserQuestion` constraint reference (`header` ≤ 12 chars, 2–4 options per `options` list, 4 questions per call cap) the consent UX honours.
- `.claude/skills/ai-audit/SKILL.md` Checklist N (post-PR-2) — the audit-side passive check the write-guard convention relies on; auto-memory-derived `## Patterns` block without a `Kind: validation` back-link → severity `major` flag.
- `~/.claude/projects/<project-path-encoded>/memory/` — the user-local auto-memory directory Step 1c reads. Currently contains `MEMORY.md` (index) + 10 `feedback_*.md` files (currently) plus 1 `project_*.md` pinning entry.
- `ai-docs/learnings.md` § 2026-05-19 *compaction-recovery protocol in skill files works* — the validation entry that PR-2 worked-example back-linked to `/context-reset/SKILL.md`; Phase 5 design verifies the existing `feedback_compaction_recovery.md` is fully covered (mentions `/task`, `/pr-commented`, `/code-review`, `/bugfix`, `/interview`, `/context-reset`; matching `Kind: validation` entry exists; cross-check passes → no candidate emitted).
