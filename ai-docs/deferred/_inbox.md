# Inbox

Untriaged rows extracted from completed plans' *Out of scope* / *Deferred* /
*Open questions* sections. Every row here is awaiting classification by
`/triage` (Issue B, [#204](https://github.com/maratik123/quartzite/issues/204))
— do not hand-edit.

This file is the universal landing zone for both forward-going propagation
(`/task` Step 12 appends one row per spec section after merging a plan) and
the one-shot backfill that seeded it. `/triage` drains rows by sorting each
into a thematic file (`signals-slots.md`, `ci-docs-workflow.md`, etc.),
promoting to a GitHub issue, or dropping with the literal `untracked`
decline-marker token written into the `Tracked` cell.

**Write discipline.** Hand-edits to this file are FORBIDDEN per the
`AGENTS.md` AXIOM (*Workflow* section, anchor `_inbox.md`) — only `/task`
Step 12 and `/triage` may write here.

**Schema.** 4-column markdown table. `Section` records which spec heading
the row was pulled from (`out-of-scope` / `deferred` / `open-question`).
`Tracked` mirrors cell 4 of the 8 thematic files — initially `—`,
rewritten to `#N` on promotion or `untracked` on decline by `/triage`.

| Item | Source | Section | Tracked |
|------|--------|---------|---------|
| Built-in image decoders — `Image` is still consumed as the raw RGBA8 buffer set up by `paint-style`. File / byte-stream decoding stays tracked under #282. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | #282 |
| `BrushKind::LinearGradient` / `RadialGradient` rendering — variant already exists on `BrushKind` as `#[non_exhaustive]`; backend support tracked under #281. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | #281 (closed) |
| `Image` source-rect cropping — tracked under #291. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | #291 |
| `BrushKind::LinearGradient` / `RadialGradient` rendering — needs gradient-stop API + peniko `Gradient` wiring | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | deferred | #281 (closed) |
| `Image` source-rect cropping — trait surface lacks a source rect; would require a `Painter` method addition | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | deferred | #291 |
| Per-test perceptual-diff tolerance tuning — calibration once real pixels exist (mentioned in `gpu-snapshot-tests-ci` open questions) | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | deferred | #286 |
| Per-platform variants (macOS-flavoured, Windows-flavoured) — the source paint-style spec defers these and tracks the work via #284. | ai-docs/plans/done/2026-05-13-default-style-content.spec.md | out-of-scope | #284 |
| `Container` content clipping (clip-rect to `geometry()` so overflow children are cut) — requires renderer-side dispatch decisions outside `Style::draw_widget`'s contract; tracked under #312. | [container-lineedit-rendering spec](../plans/done/2026-05-15-container-lineedit-rendering.spec.md) | deferred | #312 (closed) |
| Per-crate coverage targets in `codecov.yml` — tracked in issue #268; explicitly deferred from this task. | [increase-project-coverage spec](../plans/done/2026-05-15-increase-project-coverage.spec.md) | out-of-scope | #268 |
| Tightening the Codecov drop threshold below 1% or flipping `informational: true` to a hard gate — tracked in issue #267; explicitly deferred until coverage stabilises. | [increase-project-coverage spec](../plans/done/2026-05-15-increase-project-coverage.spec.md) | out-of-scope | #267 |
| Implementation of `scripts/check-instruction-file-sizes.sh` (the pre-commit / CI mechanical gate). Already tracked separately in #383. This task's audit-side check is a back-stop, not a replacement. | [ai-audit-charcap-axiom spec](../plans/done/2026-05-16-ai-audit-charcap-axiom.spec.md) | out-of-scope | — |
| Rewriting the AXIOM body in `AGENTS.md § Build & Test`. The AXIOM stays where it is; `agent-writing-style.md` Pattern 8 references it as the source-of-truth via `AGENTS.md § Build & Test`. | [ai-audit-charcap-axiom spec](../plans/done/2026-05-16-ai-audit-charcap-axiom.spec.md) | out-of-scope | — |
| Editing `AGENTS.md` for this task. The Round-1 decision to land the AXIOM as Pattern 8 means no new propagation-rule row is needed in AGENTS.md (the existing `agent-writing-style.md` propagation row covers fan-out via `## Propagation rule for new patterns` in the style guide). | [ai-audit-charcap-axiom spec](../plans/done/2026-05-16-ai-audit-charcap-axiom.spec.md) | out-of-scope | — |
| Changing the severity bands (35k / 40k) or the covered file set — both inherited verbatim from the AGENTS.md AXIOM. | [ai-audit-charcap-axiom spec](../plans/done/2026-05-16-ai-audit-charcap-axiom.spec.md) | out-of-scope | — |
| Generalising Sub-check 10's heading-derived coverage map to other style references (e.g., `code-style.md`, `doc-convention.md`). | [ai-audit-charcap-axiom spec](../plans/done/2026-05-16-ai-audit-charcap-axiom.spec.md) | out-of-scope | — |
| Adding a Rule-5 substring blacklist entry to `.claude/agents/spec-writer.md`. The char-cap rule is a structural property of files, not a question-time pre-resolved rule. | [ai-audit-charcap-axiom spec](../plans/done/2026-05-16-ai-audit-charcap-axiom.spec.md) | out-of-scope | — |
| `scripts/check-instruction-file-sizes.sh` pre-commit / CI gate — long-term mechanical enforcement complements the audit-side back-stop. | [ai-audit-charcap-axiom spec](../plans/done/2026-05-16-ai-audit-charcap-axiom.spec.md) | deferred | #383 |
| Generalising Sub-check 10's heading-derived coverage map to other reference style guides (`code-style.md`, `doc-convention.md`, etc.) — reduces audit drift across multiple style references. | [ai-audit-charcap-axiom spec](../plans/done/2026-05-16-ai-audit-charcap-axiom.spec.md) | deferred | — |
| Should Pattern 8 cross-reference the deferred `scripts/check-instruction-file-sizes.sh` (issue #383)? — Resolved at design phase: adopted the spec's default (one-line forward reference: "Mechanical pre-commit gate planned in #383; this audit-side back-stop fires in the meantime."). | [ai-audit-charcap-axiom spec](../plans/done/2026-05-16-ai-audit-charcap-axiom.spec.md) | open-question | — |
| Mechanical pre-commit / CI gate for instruction-file size enforcement — tracked separately in #383. | [agents-md-extraction spec](../plans/done/2026-05-16-agents-md-extraction.spec.md) | out-of-scope | #383 |
| Substantive rewriting of extracted content — extraction is a move + stub, not a content edit; binding semantics preserved verbatim. Editorial micro-tightening permitted only when meaning-preserving. | [agents-md-extraction spec](../plans/done/2026-05-16-agents-md-extraction.spec.md) | out-of-scope | — |
| Extraction of `AGENTS.md § Workflow` — second-highest extraction value, second-highest context risk (multiple downstream skills depend on rule placement). Revisit if a future edit pushes AGENTS.md back into the 35k early-warning band. | [agents-md-extraction spec](../plans/done/2026-05-16-agents-md-extraction.spec.md) | deferred | — |
| Further extraction of `AGENTS.md § Code Style` — already largely cross-referenced to `ai-docs/code-style.md`; diminishing returns. | [agents-md-extraction spec](../plans/done/2026-05-16-agents-md-extraction.spec.md) | out-of-scope | — |
| Further extraction of `AGENTS.md § Propagation Rule` — PR #376 already extracted the agent-writing-style.md row; remaining rows are mostly one-liners. | [agents-md-extraction spec](../plans/done/2026-05-16-agents-md-extraction.spec.md) | out-of-scope | — |
| Extraction of `AGENTS.md § Build & Test` — the 40k AXIOM itself lives in that section; recursive extraction would be confusing. | [agents-md-extraction spec](../plans/done/2026-05-16-agents-md-extraction.spec.md) | out-of-scope | — |
| Edits to `.claude/skills/**` or `.claude/agents/**` files — they have their own per-file budgets, audited independently. | [agents-md-extraction spec](../plans/done/2026-05-16-agents-md-extraction.spec.md) | out-of-scope | — |
| Substantive Rust source changes — none expected; this PR is markdown-only. | [agents-md-extraction spec](../plans/done/2026-05-16-agents-md-extraction.spec.md) | out-of-scope | — |
| Second-round extraction analysis after this PR lands — defer to the next PR that hits the early-warning band; the AXIOM itself prescribes the action. | [agents-md-extraction spec](../plans/done/2026-05-16-agents-md-extraction.spec.md) | deferred | — |
| Refactoring or rewriting either of the 2 production unsafe blocks. The index records them; hardening is the "preferred fix" of each entry, not the unsafe-index creation task itself. | [unsafe-index spec](../plans/done/2026-05-16-unsafe-index.spec.md) | out-of-scope | — |
| Tracking test-only unsafe (any `unsafe` block in `tests/`, `#[cfg(test)]` modules, `benches/`, or `examples/`). Index records production sites only, mirroring panic-index. | [unsafe-index spec](../plans/done/2026-05-16-unsafe-index.spec.md) | out-of-scope | — |
| `scripts/check-unsafe-index.sh` enforcement script. Sync gate is enforced by review-findings + self-review, not by a CI script. | [unsafe-index spec](../plans/done/2026-05-16-unsafe-index.spec.md) | out-of-scope | — |
| CI workflow changes. Sync enforced at PR-review time, not at GitHub-Actions time. | [unsafe-index spec](../plans/done/2026-05-16-unsafe-index.spec.md) | out-of-scope | — |
| Merging unsafe-index and panic-index into a single "invariant-index" file. User explicitly asked for a parallel file. | [unsafe-index spec](../plans/done/2026-05-16-unsafe-index.spec.md) | out-of-scope | — |
| Automated `scripts/check-unsafe-index.sh` parity with any future `scripts/check-panic-index.sh` — mechanical CI gate enforcement. | [unsafe-index spec](../plans/done/2026-05-16-unsafe-index.spec.md) | deferred | — |
| Per-entry severity tagging (e.g. "structurally unavoidable" vs "removable with refactor") — revisit if the index grows beyond ~20 entries. | [unsafe-index spec](../plans/done/2026-05-16-unsafe-index.spec.md) | deferred | — |
| AGENTS.md § Agent Docs symmetry — add rows for BOTH `ai-docs/panic-index.md` AND `ai-docs/unsafe-index.md`, or leave both out symmetrically. Current state: neither has a row. Spec conditioned the unsafe-index row on the panic-index row's presence; consistent treatment (add or leave-out) is the follow-up. | [unsafe-index spec](../plans/done/2026-05-16-unsafe-index.spec.md) | open-question | — |
| AGENTS.md § Propagation Rule symmetry — confirm the existing Procedure's grep step covers "unsafe-index" naturally (default assumption); add a new sync-group row if grep misses the unsafe-index in practice. | [unsafe-index spec](../plans/done/2026-05-16-unsafe-index.spec.md) | open-question | — |
