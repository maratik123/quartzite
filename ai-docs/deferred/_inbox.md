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
| `BrushKind::LinearGradient` / `RadialGradient` rendering — variant already exists on `BrushKind` as `#[non_exhaustive]`; backend support tracked under #281. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | #281 |
| `Image` source-rect cropping — tracked under #291. | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | out-of-scope | #291 |
| `BrushKind::LinearGradient` / `RadialGradient` rendering — needs gradient-stop API + peniko `Gradient` wiring | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | deferred | #281 |
| `Image` source-rect cropping — trait surface lacks a source rect; would require a `Painter` method addition | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | deferred | #291 |
| Per-test perceptual-diff tolerance tuning — calibration once real pixels exist (mentioned in `gpu-snapshot-tests-ci` open questions) | ai-docs/plans/done/2026-05-12-renderer-painter-impls.spec.md | deferred | #286 |
| Per-platform variants (macOS-flavoured, Windows-flavoured) — the source paint-style spec defers these and tracks the work via #284. | ai-docs/plans/done/2026-05-13-default-style-content.spec.md | out-of-scope | #284 |
| Refactoring or removing the existing call-level `mod tests` block in `quartzite-style/src/default_style.rs`. Those tests stay; they cover invariants the pixel goldens can't. | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | out-of-scope | — |
| Adding snapshot tests for widgets `DefaultStyle` does not yet support (`Container`, `LineEdit`) — those fall through the unknown-widget arm and are already covered by the call-level no-op test. | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | out-of-scope | — |
| Per-platform `DefaultStyle` variants (macOS / Windows flavours). Tracked separately under #284. | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | out-of-scope | — |
| Scrollbar track / thumb rendering for `ScrollArea`. Deferred in the prior spec; chrome only here too. | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | out-of-scope | — |
| Hover / pressed / focused button states. Deferred in the prior spec; only `checked` and `enabled` are wired. | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | out-of-scope | — |
| TextEdit caret / selection / scroll offset rendering. Deferred in the prior spec. | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | out-of-scope | — |
| Extracting the snapshot-assert helper into a shared dev-only crate. The helper is duplicated between `quartzite-widgets/tests/support/mod.rs` and `quartzite-style/tests/support/mod.rs` in v1 — this keeps the change surface small. Extraction is recorded as a deferred follow-up. | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | out-of-scope | — |
| Snapshot goldens for the unknown-widget fall-through (empty PNG = harness clear colour; adds no signal). | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | out-of-scope | — |
| Snapshot golden for the registry round-trip. Already covered at the call level (AC10 of the prior spec); a pixel golden would be byte-identical to `button_idle`. | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | out-of-scope | — |
| Shared `quartzite-test-support` dev-only crate that hosts `snapshot_assert` + `harness_or_skip` for every crate that wants pixel goldens — duplicating the helper between `quartzite-widgets/tests/support/mod.rs` and `quartzite-style/tests/support/mod.rs` is fine for two consumers but starts to drift at three+ | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | deferred | — |
| Per-backend (vulkan / dx12 / metal) override goldens for `DefaultStyle` — the `shared/` fallback handles every backend today because no real rasterization drift has surfaced | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | deferred | — |
| Snapshot tests for `Container` / `LineEdit` under `DefaultStyle` — both fall through the unknown-widget arm today; testing requires extending `DefaultStyle` itself first | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | deferred | — |
| Whether the seven goldens look "right" enough for v1, or whether `DefaultStyle`'s visual choices (1 px outline, flat fill, palette-direct text colours) want a follow-up styling pass *before* goldens are committed. Default: commit the v1 goldens as-is; revisit via a follow-up issue if review surfaces "the chrome looks too flat" feedback. Not blocking. | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | open-question | — |
| Whether `harness_or_skip` should live alongside `snapshot_assert` in `tests/support/mod.rs` from the outset (so the `quartzite-widgets` copy can adopt it too via the snapshot-helper sync group). Default: yes — lift it into support during this PR so both copies stay symmetric. Design may revisit if the lift creates churn the reviewer flags. | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | open-question | — |
| Whether per-backend goldens land in this PR or in a follow-up once drift is observed. Default: ship `shared/` only; per-backend overrides happen reactively when CI on a new backend flags a FLIP-mean breach. Not blocking. | [default-style-snapshot-tests spec](../plans/done/2026-05-13-default-style-snapshot-tests.spec.md) | open-question | — |
| Editing instruction files other than `AGENTS.md` and the **new or existing** `ai-docs/*.md` files that absorb extracted content. (Other instruction files are touched only as required by the Propagation Rule, e.g., updating cross-references that pointed to a now-moved AGENTS.md section.) | [shrink-agents-md spec](../plans/done/2026-05-13-shrink-agents-md.spec.md) | out-of-scope | — |
| Editing `ai-docs/learnings.md` in the same turn as AGENTS.md edits (Corrections Log Boundary Rule 2). Existing entries' `Escalated?` / `Superseded by:` fields are also out of scope for this PR. | [shrink-agents-md spec](../plans/done/2026-05-13-shrink-agents-md.spec.md) | out-of-scope | — |
| Rewording any AXIOM, MUST, NEVER, DENY, or ASK directive. Only relocation (extraction into a referenced page) and dedup (AXIOM-vs-redundant-prose) are permitted. | [shrink-agents-md spec](../plans/done/2026-05-13-shrink-agents-md.spec.md) | out-of-scope | — |
| Restructuring the H2 section ordering of `AGENTS.md` (the `## Project / ## Permissions / ## Build & Test / ...` sequence). Sections may be shortened or their content moved out, but the H2 skeleton stays so existing memory of "look in `## Workflow`" continues to resolve. | [shrink-agents-md spec](../plans/done/2026-05-13-shrink-agents-md.spec.md) | out-of-scope | — |
| Re-running `/improve` or `/ai-audit` as part of this task. | [shrink-agents-md spec](../plans/done/2026-05-13-shrink-agents-md.spec.md) | out-of-scope | — |
| Editing the `CLAUDE.md` `@AGENTS.md` import line. | [shrink-agents-md spec](../plans/done/2026-05-13-shrink-agents-md.spec.md) | out-of-scope | — |
| Migration of *other* `.claude/skills/**` and `.claude/agents/**` files that are also approaching their own perf thresholds — track separately when measured. — independent perf chore; not blocking this one | [shrink-agents-md spec](../plans/done/2026-05-13-shrink-agents-md.spec.md) | deferred | — |
| A `scripts/check-agents-md-size.sh` precommit / CI gate that would mechanically prevent regression past the threshold. — nice-to-have automation, not required by this task | [shrink-agents-md spec](../plans/done/2026-05-13-shrink-agents-md.spec.md) | deferred | — |
