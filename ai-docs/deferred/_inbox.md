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
