# code-style-extraction

**Source:** user description
**Date:** 2026-05-07
**Tracked in:** #124

## Scope

1. Create `ai-docs/code-style.md` as the canonical workspace code-style reference, mirroring `ai-docs/doc-convention.md`'s structural shape: `## Scope` (what the rules apply to) → one `##` heading per rule cluster → `## Lints that mechanically enforce parts of this convention` → `## Behavioural enforcement (what lints cannot check)`. Omit `## References` (no external citations to link).
2. Inside `ai-docs/code-style.md`, regroup the rules into the 10 logical clusters below (one `##` per cluster). Body content is the verbatim rule text from AGENTS.md's existing Code Style section — **no rule changes**.

   | Order | `##` heading | Rules folded in |
   |---|---|---|
   | 1 | Source files | `.rs`-only; line length (100, rustfmt default); `cargo fmt` over `rustfmt <file>` |
   | 2 | Linter posture | strict clippy (`-D warnings`); `#[allow(...)]` policy (no blanket `#[allow(dead_code)]` / `#[allow(clippy::...)]` without justification) |
   | 3 | Rust idioms | let chains (edition 2024); comparison helpers (`.min`/`.max`/`.clamp`/`Option::or`/`Option::filter`); no GUI/UI-framework citations as design justification |
   | 4 | Library safety idioms | mutex locks (`mutex.lock().ok()?` / `.unwrap_or_else(|e| e.into_inner())`); safe primitives over raw pointers |
   | 5 | Documentation | `#![deny(missing_docs)]` + `#![warn(clippy::undocumented_unsafe_blocks)]` per crate; one-line `///` per public item; `# Examples` for new public items with single-line docs; `no_run` for proc-macro / event-loop-needing items; defer to `ai-docs/doc-convention.md` for the doc style itself |
   | 6 | Error types | `thiserror` for new error enum/struct; hand-rolled `Display`/`Error` reserved for cases the derive cannot express |
   | 7 | Tracing | `*_span!` guards on functions that meaningfully mutate application state; `debug_span!` for lifecycle, `trace_span!` for supplementary; `verbose-tracing` cargo feature gate for high-frequency paths; sibling-consistency rule |
   | 8 | `#[inline]` and the `_Simple._` doc tag | recursive "simple" definition; three markers (concrete `#[inline]` / generic `_Simple._` / trait-decl `_Simple._`); carve-out for default methods inside a `pub trait` body; marker-form decision tree (`///` vs `//`); typical targets; codegen mirroring; marker maintenance (cascade) |
   | 9 | Generic-fn split for binary size | conversion-style generic + > 3 line body → nested `fn inner(...)`; outer carries `_Simple._`; helper named `inner` not `<outer>_inner`, nested not sibling; doesn't-apply when type param used throughout body; pattern with worked example; **if the inner ends up simple, unwrap it** (re-evaluate outer's `_Simple._` per marker-maintenance) |
   | 10 | File size | 200–400 line target excl. `#[cfg(test)]`; soft 500/800; hard 1000/1500; exemptions; counter-rule (no over-splitting); per-fn `too_many_lines` |

3. Replace AGENTS.md's `## Code Style` section with a short **bulleted index** — one line per cluster (~10 bullets) summarising the rule + link to `ai-docs/code-style.md` for the canonical reference. Format chosen over the dense one-paragraph "Mandatory rules in brief" form because the rule set is too large to compress into one paragraph readably.
4. Add a `code-style.md` row to the AGENTS.md `Agent Docs` table (currently at line ~209), mirroring the existing `doc-convention.md` reference shape.
5. Propagate references: `rg 'AGENTS\.md.*Code Style'` across `.claude/agents/`, `.claude/skills/`, `ai-docs/learnings.md`, `ai-docs/plans/` — update each citation. Where a hit cites a rule body that now lives in `ai-docs/code-style.md`, point the link at the new file. Where a hit cites the AGENTS.md section as the entry point (e.g., review-findings checklist headers), update to "see `ai-docs/code-style.md`".
6. Update the **escalation taxonomy** to recognise `code-style` as a project-level escalation target (parallel to the existing `doc-convention`):
   - **`AGENTS.md` `## Corrections Log` (line ~231):** add `code-style` to the `Escalated?` enum and add an explanatory note: *"`code-style` = the rule landed in `ai-docs/code-style.md`. Use only for code-style rules that genuinely belong in the workspace code-style reference rather than in AGENTS.md or a skill."*
   - **`.claude/agents/self-improve.md` `description:` frontmatter (line 3):** add `ai-docs/code-style.md` and `ai-docs/doc-convention.md` to the list of escalation targets the agent recognises.
   - **`.claude/agents/self-improve.md` "Escalated" status list (line ~28):** add `code-style` AND `doc-convention` to the recognised set. The `doc-convention` omission is a pre-existing gap — would cause `/improve` to mis-classify a `doc-convention`-escalated entry as unescalated. Fixing in the same PR keeps the taxonomy consistent with the AGENTS.md template.
   - `.claude/skills/improve/SKILL.md` does not need changes — it delegates the target taxonomy to the self-improve agent and does not enumerate targets itself.

## Out of scope

- Any change to the rules themselves — pure extract+restructure.
- Extracting other AGENTS.md sections (API Stability, API Naming, Workflow, Communication, Permissions) — they don't have the same length problem.
- `## Dependency Versions` section in AGENTS.md — stays separate; not Code-Style content.
- Restructuring `ai-docs/doc-convention.md`.
- Changing the `## References` slot in `ai-docs/doc-convention.md` to match `code-style.md`'s skip — keep doc-convention.md as-is.

## Deferred

- what | why | separate issue needed?
- Same extraction treatment for other AGENTS.md sections if they grow comparably in future | will judge case-by-case at the time | yes — file at the time the threshold is hit, not pre-emptively |

## Key decisions

| Question | Decision |
|---|---|
| Mirror `doc-convention.md` structure or use a leaner one-`##`-per-rule layout? | Mirror — `## Scope` + meta-sections (`## Lints that mechanically enforce`, `## Behavioural enforcement`) genuinely apply to Code Style; consistency aids navigation across the two reference files. |
| Skip `## References`? | Yes — no external citations exist for these rules (rustfmt defaults, clippy lint pages are folklore). Empty section would be padding. |
| Preserve AGENTS.md rule order verbatim or regroup logically? | Regroup into 10 clusters (see Scope #2 table). Existing order is ad-hoc; logical clustering improves discoverability. No rule body content changes. |
| Brief-summary format in AGENTS.md — dense paragraph or bulleted index? | Bulleted index (~10 bullets, one per cluster). Rule set is too large for the dense paragraph form `doc-convention.md` uses. |
| Migrate `## Dependency Versions` into `code-style.md`? | No — keep separate. Cargo.toml mechanics, not source style. |
| Update propagation references everywhere or leave AGENTS.md citations as a redirect? | Update everywhere. Citations should point at the file containing the rule body to keep `rg` audits accurate. |
| Update escalation taxonomy in AGENTS.md and self-improve agent? | Yes — `code-style` becomes a recognised escalation target parallel to `doc-convention`. Also fix pre-existing gap where `doc-convention` is in the AGENTS.md template but missing from the self-improve agent's "Escalated" status list. |

## Technical constraints

- **Doc-only PR.** No `.rs` source files change. No `Cargo.toml` change. `cargo build` / `clippy` / `test` run as a sanity check (must remain clean) but no behavioural change is expected.
- **Audit grep `rg '_Simple\._'` and similar must continue to match the same set of source sites** — propagation step is purely about prose references, not inline source markers.
- **Markdown link integrity.** All relative links from `ai-docs/code-style.md` must resolve (`../AGENTS.md`, `./doc-convention.md`). All AGENTS.md links pointing at `ai-docs/code-style.md` must resolve. Trace at least one link by hand or with `realpath` before committing per AGENTS.md `Workflow` rule.
- **Propagation Rule (AGENTS.md).** When editing AGENTS.md, grep `.claude/agents/`, `.claude/skills/`, `AGENTS.md` for any reference to the changed terminology and apply the same change there. The current task IS a propagation across the instruction tree — the Propagation Rule is the framework for executing it.
- **Don't write content directly into MEMORY.md** (memory rule) — not applicable here, but flagged because both files live under `ai-docs/`.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ai-docs/code-style.md` exists and is structured as: `## Scope` → one `##` per cluster (clusters 1–10 in spec table, in that order) → `## Lints that mechanically enforce parts of this convention` → `## Behavioural enforcement (what lints cannot check)`. No `## References` section. |
| AC2 | Every Code Style rule currently in AGENTS.md (lines 52–141, all bullets) has its rule body present verbatim in `ai-docs/code-style.md` under the cluster it was assigned to. Diff of the original rule prose between AGENTS.md and `code-style.md` (after extraction) is empty modulo whitespace and Markdown header reformatting. |
| AC3 | AGENTS.md's `## Code Style` section is replaced by a bulleted index of ~10 lines (one per cluster), each line one sentence + a link to the cluster anchor in `ai-docs/code-style.md`. Section ends with "see [`ai-docs/code-style.md`](ai-docs/code-style.md) for the canonical reference." |
| AC4 | AGENTS.md `Agent Docs` table includes a row for `ai-docs/code-style.md` with purpose "Workspace code-style reference — read on demand". |
| AC5 | `rg 'AGENTS\.md.*Code Style'` across `.claude/agents/`, `.claude/skills/`, `ai-docs/learnings.md`, `ai-docs/plans/` returns no hits, OR each remaining hit is intentionally pointing at the now-trimmed AGENTS.md section header (not at a rule body that moved). |
| AC6 | All relative Markdown links from `ai-docs/code-style.md` resolve (verified by hand or via `realpath`). All new AGENTS.md links to `ai-docs/code-style.md` resolve. |
| AC7 | `cargo build && cargo clippy --all-targets -- -D warnings && cargo fmt -- --check && cargo test` clean (sanity check — doc-only PR should not affect any of these). |
| AC8 | `cargo build -p quartzite --no-default-features` clean. |
| AC9 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace` clean. |
| AC10 | The `## Lints that mechanically enforce parts of this convention` section in `code-style.md` enumerates every clippy lint that backstops a rule (e.g., `clippy::too_many_lines`, `clippy::undocumented_unsafe_blocks`, `clippy::missing_errors_doc`, etc.). The `## Behavioural enforcement` section enumerates every reviewer-judgment rule (marker-maintenance cascade, `_Simple._` decision tree, unwrap-simple-`inner`, no-foreign-framework citation, etc.). Lists are not exhaustive of the rule body — they index pointers into earlier `##` sections. |
| AC11 | `ai-docs/plans/INDEX.md` updated: row added under Active plans pointing at this spec; on completion (Step 12), row moves to ✅ implemented. |
| AC12 | Escalation taxonomy updated: AGENTS.md `## Corrections Log` `Escalated?` enum (line ~231) includes `code-style` with an explanatory note. `.claude/agents/self-improve.md` recognises both `code-style` AND `doc-convention` as escalation targets — in the `description:` frontmatter (line 3) and in the "Escalated" status list (line ~28). |

## Open questions

None — all interview rounds resolved.
