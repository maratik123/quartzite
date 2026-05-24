# Design: Fix AC# references that leak into generated docs

**Issue:** #557
**Date:** 2026-05-24

## Approach

The task is a mechanical docs-only sweep: every `///` doc-comment under `**/src/**/*.rs` that mentions `AC<digit>` is rewritten to describe the underlying behaviour directly, without losing useful test intent. One site (`quartzite-renderer/src/vello_painter.rs:27`) also requires a factual correction — the doc currently claims gradient brushes are unsupported, but `LocalBrushKind::from_brush_kind` (lines 186–216 of the same file) shows `LinearGradient`, `RadialGradient`, and `Custom` are explicitly matched and forwarded to vello; only the catch-all `_ => Self::Unknown` arm (covering future `#[non_exhaustive]` variants) falls through to the no-brush sink.

The spec's §2 enumerates the cited sites from round-1 grep. A live re-run (`rg -n '^\s*///.*\bAC[0-9]+\b' --type rust 2>&1 | grep '/src/'`) confirms the same sites plus two extras the spec did not list explicitly but AC3's catch-all grep requires we also strip:

- `quartzite-style/src/default_style_tests.rs:2126` — `/// AC2 — read_only overlays …`
- `quartzite-style/src/default_style_tests.rs:2368` — `/// AC8 — asserts on the recorded painter-call argument shape …`

These two are in scope because AC3 mandates **zero** hits on the workspace-wide grep — the spec's explicit enumeration is non-exhaustive; AC3 is authoritative.

`//!`, `Cargo.toml`, `examples/**`, and `quartzite-macros/src/**` already grep clean (preventative ACs); no edits there.

**Grouping rationale for decomposition:** affected files cluster into three crates (`quartzite-renderer`, `quartzite-core`, `quartzite-style`/`quartzite-style-types`). One subtask per crate keeps each change atomic and reviewable, isolates the factual rewrite (`vello_painter.rs:27`) into the renderer subtask alongside its sibling site, and lets each subtask run `cargo build -p <crate>` for fast local feedback before the workspace-wide verification gate. A fourth subtask creates the deferred-guard gh issue (AC11), and a fifth runs the workspace gates (AC3–AC10).

**Rejected alternatives:**

1. **Single mega-edit subtask covering all 8 files at once.** Rejected — harder to bisect if `cargo doc` regresses; reviewer comments would land scattered across one giant diff.
2. **One subtask per file (8 subtasks).** Rejected — exceeds the 7-task ceiling, and several files (`wrapped_handler.rs` 4 sites; `default_style_tests.rs` 9 sites) require multiple edits with no inter-file dependency that would justify per-file granularity. Per-crate is the right unit.
3. **Defer the factual correction at `vello_painter.rs:27` to a separate PR.** Rejected — the spec's AC1 explicitly bundles strip + correct as a single criterion ("contains zero `AC<digit>` substrings AND accurately reflects current behaviour"); splitting would force two PRs for one logical fix.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Rewrite `///` AC# refs in `quartzite-renderer/src/**`, including the factual correction at `vello_painter.rs:27` (strip `AC10`; rewrite the doc paragraph to state that gradient variants — `LinearGradient`, `RadialGradient`, `Custom` — are supported via vello, and only future unknown `#[non_exhaustive]` `BrushKind` variants fall through to the no-brush sink). Also rewrite `vello_painter.rs:184`, `render_harness.rs:91–92`, `window_registry.rs:209–210`, `wrapped_handler.rs:332,366,398,432`. Cross-check the new wording at `vello_painter.rs:27` against the `match` arms in `LocalBrushKind::from_brush_kind` in the same file (lines 186–216) to satisfy AC1's accuracy clause. | `quartzite-renderer/src/vello_painter.rs`, `quartzite-renderer/src/render_harness.rs`, `quartzite-renderer/src/window_registry.rs`, `quartzite-renderer/src/wrapped_handler.rs` | — |
| 2 | Rewrite `///` AC# refs in `quartzite-core/src/**`. Two sites: `traits.rs:289` (`ObjectExt::downcast_ref` — strip `(AC8)`, keep the rest of the existing sentence), `signal.rs:68` (`ConnectionType::Auto` — strip `(see AC5)`, optionally rephrase as "Changing the receiver's thread affinity after connecting does not update the stored `ThreadId`."). | `quartzite-core/src/traits.rs`, `quartzite-core/src/signal.rs` | — |
| 3 | Rewrite `///` AC# refs in `quartzite-style-types/src/dark_palette.rs` (lines 109, 142, 160 — all are `///` on `#[cfg(test)]` test fns inside the `mod tests` block; replace the `AC8 — …` prefix with a behavioural one-liner that preserves test intent). | `quartzite-style-types/src/dark_palette.rs` | — |
| 4 | Rewrite `///` AC# refs in `quartzite-style/src/default_style_tests.rs`. Nine sites: lines 945, 994, 1604, 1663, 1718, 1737, 1762 (spec's explicit enumeration) plus lines 2126 and 2368 (extras surfaced by the live AC3 grep — spec's enumeration is non-exhaustive; AC3 is authoritative). Each `///` rewrite strips the `AC<digit>` token and keeps or rewords the behavioural part to preserve test intent. | `quartzite-style/src/default_style_tests.rs` | — |
| 5 | Create the deferred-guard gh issue titled approximately "Add automated guard for AC# rustdoc leaks" with a body describing the deferred recurrence-guard work (LLM-based, CI script, or doc-test bash/Python check). Capture the issue number so it can be referenced in the PR body to satisfy AC11. | (no source edits — `gh issue create` invocation only) | — |
| 6 | Run the workspace verification gates: `rg -n '^\s*///.*\bAC[0-9]+\b' --type rust -g '**/src/**'` returns zero (AC3); `rg -n '^\s*//!.*\bAC[0-9]+\b' --type rust -g '**/src/**'` returns zero (AC4); `rg -n 'AC[0-9]+' quartzite-macros/src/` returns only `//` line comments, not `///` / `#[doc=…]` / `quote!`-spliced `///` (AC4 second clause); `rg -n 'AC[0-9]+' -g 'Cargo.toml'` returns zero (AC5); `rg -n 'AC[0-9]+' -g 'examples/**'` returns zero (AC6); `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` passes (AC7); `cargo clippy --workspace --all-targets -- -D warnings` passes (AC8); `cargo fmt -- --check` passes (AC9); `cargo test --workspace` passes (AC10). | (no source edits — verification only) | 1, 2, 3, 4 |
| 7 | Open the PR and ensure its body says `Closes #557` (AC12) and references the new deferred-guard issue created in subtask 5 (AC11). | (PR body via `gh pr create`) | 5, 6 |

## Handoff plan

`M = 7`. Three groups: 3 + 3 + 1.

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md`. Parent `/task` enters Group A with fresh context.
- **Group A:** subtasks 1–3 — per-crate `///` sweeps for `quartzite-renderer`, `quartzite-core`, and `quartzite-style-types`. Includes the factual rewrite at `vello_painter.rs:27`. Three subtasks.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md`. Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — subtask 5 (gh issue creation) runs first (no source-code dependency), then subtask 4 (`default_style_tests.rs` sweep), then subtask 6 (workspace verification gates). Three subtasks.
- **Handoff after Group B:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md`. Parent `/task` resumes in Group C with fresh context.
- **Group C:** subtask 7 — terminal group (1 subtask). PR open with `Closes #557` and reference to the new deferred-guard issue.

## Risks

- **Risk: A `///` rewrite at one of the cited sites accidentally drops information that downstream readers rely on (test intent, behavioural nuance).** Mitigation: for each site, preserve the substantive part of the existing sentence and strip only the `AC<digit>` token. Where the entire sentence is "AC<N> — <behaviour>", rephrase to start with the behaviour. Subtask 6's `cargo test --workspace` gate catches any unintended logic edit; subtask 6's `cargo doc` gate catches any broken intra-doc links introduced during rewriting.
- **Risk: The factual rewrite at `vello_painter.rs:27` misrepresents the actual `match` behaviour in `LocalBrushKind::from_brush_kind`.** Mitigation: subtask 1 explicitly mandates a cross-check of the new wording against the four explicit `match` arms (`Solid`, `LinearGradient`, `RadialGradient`, `Custom`) and the catch-all `_ => Self::Unknown` arm (lines 186–216). The wording must enumerate the supported variants and call out the catch-all behaviour.
- **Risk: A future macro-generated `///` referencing `AC#` slips into `quartzite-macros/src/**` between this PR and the deferred-guard PR.** Mitigation: AC4's second clause re-asserts grep cleanliness on `quartzite-macros/src/` at the verification step; the recurrence guard (deferred to AC11's gh issue) is the durable fix.
- **Risk: `default_style_tests.rs` is `#[cfg(test)]`-wrapped, so `cargo doc` already does not surface the `///` lines.** Mitigation: spec AC3 is the authoritative gate, requiring zero `///.*\bAC[0-9]+\b` hits across the entire workspace `src/**` regardless of whether `cargo doc` happens to surface them; the sweep is uniform and preventative.
- **Risk: A site cited in §2 of the spec has shifted line numbers since the round-1 grep.** Mitigation: subtask 6's live AC3 grep is the gating mechanism — it greps **all** `src/**` and requires zero hits, so any drift is caught regardless of the cited line numbers.
- **Risk: Subtask 5's gh-issue creation runs out of order if subtask 6's verification fails repeatedly.** Mitigation: subtask 5 has no source-code dependency on subtasks 1–4; it can be created at any point in Group B.

## Test Design

The spec's "Key decisions" row explicitly states **no new behavioural test** is added. Subtask-level verifications:

- **Subtask 1 (`quartzite-renderer`):** `cargo test -p quartzite-renderer` passes; accuracy gate (AC1) — manual cross-check of new `vello_painter.rs:27` doc block against `LocalBrushKind::from_brush_kind` match arms (lines 186–216); `cargo doc -p quartzite-renderer --no-deps` builds clean.
- **Subtask 2 (`quartzite-core`):** `cargo test -p quartzite-core` passes; `cargo doc -p quartzite-core --no-deps` builds clean.
- **Subtask 3 (`quartzite-style-types`):** `cargo test -p quartzite-style-types` passes; `cargo doc -p quartzite-style-types --no-deps` builds clean.
- **Subtask 4 (`quartzite-style`):** `cargo test -p quartzite-style` passes; `cargo doc -p quartzite-style --no-deps` builds clean.
- **Subtask 5 (gh-issue creation):** `gh issue view <new-id>` confirms title/body conformance to AC11.
- **Subtask 6 (workspace verification gates):** Runs all five `rg` commands plus `cargo doc`, `cargo clippy`, `cargo fmt --check`, `cargo test --workspace`.
- **Subtask 7 (PR open):** PR body contains `Closes #557` (AC12) and reference to the new guard issue (AC11).

## Open questions

_(None — the spec's "Open questions" section is closed; Q1, Q2, Q3 were resolved in spec round 2, and the live re-grep in this design phase surfaced two additional sites at `default_style_tests.rs:2126,2368` that are now folded into subtask 4 under AC3's catch-all authority.)_
