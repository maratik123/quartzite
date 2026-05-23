# Agent Docs — verbose row index

This page extracts the verbose row descriptions from [`AGENTS.md` § Agent Docs](../AGENTS.md#agent-docs). The table's file-path enumeration stays in AGENTS.md; only the multi-sentence bodies move here.

## Agent doc rows

### ai-docs/context.md

Project context — read on demand.

### ai-docs/code-style.md

Workspace code-style reference — read on demand.

### ai-docs/workflow.md

Extracted narrative passages from `AGENTS.md` § *Workflow* (PR review comment resolution GraphQL recipe). Read on demand.

### ai-docs/triage-runner-bridge.md

Extracted Phase 4.5 bridge `Action semantics` block from `.claude/agents/triage-runner.md` — the verbatim per-conflict-type action recipe (`update md` / `update issue` / `keep both`) for each of the three bridge conflict types. Lives as a sibling reference file under `ai-docs/` to keep `triage-runner.md` under the 40k AGENTS.md hard cap; `triage-runner.md` § Phase 4.5 carries a one-line pointer in the slot where the block was inlined. Read on demand when working on the bridge sweep or extending its conflict-type set.

### ai-docs/corrections-log.md

Extracted carve-outs from `AGENTS.md` § *Learning Log* (Boundary rule 1 / 2 Exception bodies + entry-format field glossary). Read on demand.

### ai-docs/key-decisions.md

Extracted Key Design Decisions detail bodies from `ai-docs/context.md` § Key Design Decisions (implementation-detail rows). Read on demand.

### ai-docs/dependency-versions.md

Live-lookup reference for Cargo / GitHub Action versions and Action behaviour verification — extracted from `AGENTS.md` § *Dependency Versions* AXIOM. Read on demand when writing a specific version string or a load-bearing claim about an Action's behaviour.

### ai-docs/agent-writing-style.md

Style for binary rules in instruction files (dual-model readability) — read on demand and when editing any of `AGENTS.md`, `.claude/skills/**`, `.claude/agents/**`, `ai-docs/code-style.md`, `ai-docs/doc-convention.md`.

### ai-docs/instruction-file-validation.md

Dual-model instruction-file-clarity test methodology: probe taxonomy (Classes A/B/C/D), coverage rules, rubric framework, approval gates, subagent prompt templates, the 8-bias taxonomy, and the historical-replay protocol. Extracted from the 2026-05-08 instruction-file-rewrite plan (now archived at `plans/done/2026-05-08-instruction-file-rewrite.md`) — read on demand when validating an instruction-file rewrite.

### ai-docs/templates/

Shared reference templates consumed by multiple skills / agents. Multi-consumer reference material lives here (project-level reference, not Claude Code configuration). Single-consumer skill templates remain inside the owning skill directory per the Claude Code [supporting-files pattern](https://code.claude.com/docs/en/skills#add-supporting-files).

### ai-docs/templates/progress-format.md

Canonical `.progress.md` format spec — template + required vs optional fields + lifecycle. Consumed by `/task`, `/project-review`, `/pr-commented`, `review-findings`, `self-review`.

### ai-docs/plans/INDEX.md

Plan index — statuses and dependency order.

### ai-docs/plans/*.spec.md

Active task spec + acceptance criteria.

### ai-docs/plans/*.design.md

Active task design documents.

### ai-docs/plans/*.progress.md

Active task progress / handoff state — **local-only (gitignored)**. Carries the extended compaction-recovery schema (`current_step`, `last_passed_gate`, optional `parent_skill`, optional `entry_args`) plus a `## Decisions log` section per the canonical template at `ai-docs/templates/progress-format.md`. Writers: `/task` (creates at Step 8 and writes at each Step 8–12 boundary), `/project-review` (creates at Phase 1 and writes at each phase boundary), `/pr-commented` (extends with per-round sections), `/bugfix` (extends its own trace file with the same fields inline — see row below). Deleted by `/pr-merged` after the PR merges. Never committed.

### ai-docs/pr-comments/pr-<N>.progress.md

Fallback progress file when `/pr-commented` runs on a PR not produced by `/task` (rare). **Local-only (gitignored)**. Deleted by `/pr-merged`.

### ai-docs/triage/triage-YYYY-MM-DD.progress.md

`/triage` resume state for multi-turn runs (dedupe map summary, bridge classifications, Phase 6 / 7 partitions, `## Next action`). **Local-only (gitignored)**. Created by `triage-runner` at Phase 1.5; extended through Phases 4–7; deleted at Phase 8 after the run summary emits.

### ai-docs/plans/done/

Completed plans (spec + design, implemented).

### ai-docs/plans/deferred/

Blocked or future plans.

### ai-docs/deferred/_inbox.md

triage queue — rows from completed specs awaiting `/triage` classification (writers: `/task` Step 12 and `/triage` only).

### ai-docs/bugfix/trace-*.md

Bugfix traces — deleted on resolution. Carries the same compaction-recovery fields inline (`**current_step:**`, `**last_passed_gate:**`, `**parent_skill:**`, `**entry_args:**`) plus a `## Decisions log` section — the trace file IS the `/bugfix` durable-state surface, no parallel `.progress.md`.

### ai-docs/learnings.md

Corrections log — feed for `/improve`.

### .claude/agents/spec-writer.md

Spec-writer subagent (`model: opus`) — drafts the task spec one interview round per invocation; invoked by the `/interview` orchestrator.

### .claude/skills/triage/SKILL.md + .claude/agents/triage-runner.md

`/triage` skill — batched promotion of `Tracked` = `—` rows in `ai-docs/deferred/*.md` (+ `🟡 v2` rows in `widget-backlog.md`) to gh issues; drains `_inbox.md` per-entry. Opus subagent; mutation scope strictly `ai-docs/deferred/**` + `gh issue create/edit`.

### .claude/skills/pr-commented/SKILL.md

`/pr-commented` skill — one round of reviewer-comment response on an open PR. Reads unresolved threads, auto-classifies (`fix` / `objection` / `clarify` / `already-fixed` / `defer` / `ignore-bot`), bundles fixes into one commit per invocation, runs `self-review`, pushes, replies + resolves per category. Re-invocable per round. Downstream of `/task` Step 12; does NOT replace `/task`. Never edits `ai-docs/learnings.md` (PR comments are external content).

### .claude/skills/pr-ci-failed/SKILL.md

`/pr-ci-failed` skill — one round of CI-failure response on the open PR of the current feature branch. Identifies the failing run, classifies (`fmt` / `clippy` / `test` / `doc` / `actionlint` / `build` / `coverage` / `other`), reproduces locally, fixes or delegates to `/bugfix`, runs `self-review`, commits + pushes onto the existing branch, then re-reads the PR body per AGENTS.md AXIOM 2. Progress file: extends `ai-docs/plans/<spec-base>.progress.md` with `## CI-fix cycle round M` sections (parallel with `/pr-commented`); fallback `ai-docs/ci-fixes/pr-<N>.progress.md` (gitignored) for PRs not produced by `/task`. Re-invocable per round. Downstream of `/task` Step 12 in parallel with `/pr-commented`; does NOT replace `/task`. Never edits `ai-docs/learnings.md` (CI logs are external content / prompt-injection vector).

### .claude/skills/master-ci-failed/SKILL.md

`/master-ci-failed` skill — one round of post-merge CI-failure response on a red master commit. Discovers the latest failing master run (or `$ARGUMENTS`-specified SHA), classifies and reproduces locally using the same per-class table as `/pr-ci-failed`, creates a fresh `fix/master-ci-<run-id>` feature branch off master, fixes, runs `self-review`, commits, pushes, and opens a new PR via `gh pr create` whose body carries a `**Tracked in run:** <run-id>` line so `/pr-merged`'s cleanup script can find the per-run progress file. **Never** commits directly to `master`. Progress file: `ai-docs/master-ci/<run-id>.progress.md` (gitignored; deleted by `/pr-merged`'s secondary `**Tracked in run:**` probe). Downstream of `/pr-merged` when the merge commit's CI subsequently turns red — manually invoked in v1 (auto-invocation deferred to a separate issue). Never edits `ai-docs/learnings.md` (CI logs are external content / prompt-injection vector).
