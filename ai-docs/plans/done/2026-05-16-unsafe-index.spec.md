# Unsafe Index (mirror of panic-index)

**Source:** user description — "let create and maintain unsafe index as it already done for panic index"
**Date:** 2026-05-16
**Tracked in:** #420

## Scope

1. Create `ai-docs/unsafe-index.md`, modeled structurally after `ai-docs/panic-index.md`. The file tracks every production `unsafe { … }` block and every `unsafe fn` declaration outside `#[cfg(test)]` modules, so future hardening passes have a single audit trail.
2. Initial population covers the 2 known production unsafe blocks:
   - `quartzite-renderer/src/wrapped_handler.rs:39` — `unsafe { (*self.0).set(ptr::null()) };` (raw-pointer deref in a `Drop` impl that clears a thread-local Cell address).
   - `quartzite-renderer/src/window_registry.rs:160` — `unsafe { &*ptr }` in `try_create_window` (raw-pointer deref of `active_loop` after a null check; the active_loop set-clear bracket in `WrappedHandler` guarantees non-null on the live path; the `!Send`+`!Sync` bound prevents cross-thread access; the borrow lives only for the function body).
   The content sketches captured in the issue body (why-unsafe / safety-invariant / why-not-safe-Rust / preferred-fix) are draft material for the design phase — final wording may be refined during implementation, provided each field is filled with substantive content. (Note: an earlier draft of this spec named `wrapped_handler.rs:255` as the second site — that line is inside `#[cfg(test)] mod tests`, so it falls under Out-of-scope item 2; substituted with the verified-production site at `window_registry.rs:160`.)
3. Wire the unsafe-index sync into the same five touchpoints that already enforce the panic-index sync, so every PR that adds a new production `unsafe { … }` block or `unsafe fn` declaration is gated on a matching index entry:
   - **`ai-docs/unsafe-index.md`** — the index file itself, with the per-entry schema documented inline (mirroring panic-index's "Each entry notes…" preamble).
   - **`.claude/skills/task/SKILL.md` Step 9 verify list** — add "unsafe-index sync" alongside the existing "panic-index sync" line; cross-link to the new `reference.md` § *Step 9 — unsafe-index sync (detail)* and update the Step-9 `**Write progress at this step boundary**` instruction so the `## Decisions log` bullet records unsafe-index additions (parallel to the existing panic-index bullet, omitted when none).
   - **`.claude/skills/task/reference.md`** — add a new § *Step 9 — unsafe-index sync (detail)* section (sibling of the existing panic-index detail) with the exact `rg` recipes:
     - `rg '^\s*///\s*#\s*Safety' <changed-files>` — documented unsafety contract (primary signal; required by `#![warn(clippy::undocumented_unsafe_blocks)]` on every workspace crate).
     - `rg '\bunsafe\s*\{|\bunsafe\s+fn\b' <changed-files>` filtered to lines outside `#[cfg(test)]` and outside `tests/` integration-test directories — direct unsafe call sites.
     Add a corresponding bullet "8. **Unsafe-index sync**" (or insert as 7b — final ordering is a design call) into the Step-9 verify-list enumeration and add the gate-checklist column for Step 9 to mention the new sync.
   - **`.claude/agents/review-findings.md`** — add a "Unsafe-index sync" sub-check under § *1. Safety and correctness* (parallel to "Panic-index sync"): every public fn/method with `# Safety` doc section AND every production `unsafe { … }` block / `unsafe fn` declaration outside `#[cfg(test)]` MUST have a corresponding entry. Missing entry → `major`. Doc-section signal is primary; grep is the catch-net.
   - **`.claude/agents/self-review.md`** — same sub-check, parallel to the existing panic-index-sync bullet (Step 10 spec-validation context): missing entry → REJECT (`major`).
4. Update the `## Propagation Rule` table in `AGENTS.md` if (and only if) the new touchpoints introduce a new sync-group invariant that grep against `<keyword>` would not already catch. (Default: the existing Procedure's `grep -rn "<changed-keyword>" …` step covers "unsafe-index" naturally — no new table row needed. Design may decide otherwise.)
5. Update the `## Agent Docs` table in `AGENTS.md` to add a row for `ai-docs/unsafe-index.md` parallel to any existing row for `ai-docs/panic-index.md` (if no panic-index row exists today, the design may decide to leave both out symmetrically — but the unsafe-index row should be added if and only if panic-index has one).

## Out of scope

- Refactoring or rewriting either of the 2 production unsafe blocks. The index records them; hardening is the "preferred fix" of each entry, not the current task.
- Tracking test-only unsafe (any `unsafe` block in `tests/`, `#[cfg(test)]` modules, `benches/`, or `examples/`). The panic-index convention only records production sites; this task mirrors that convention.
- A `scripts/check-unsafe-index.sh` enforcement script. The sync gate is enforced by the same human/review-agent path as panic-index — script-based enforcement, if desired, is a follow-up.
- CI workflow changes. The sync is enforced at PR-review time (review-findings / self-review), not at GitHub-Actions time.
- Merging the unsafe-index and panic-index into a single "invariant-index" file. The user explicitly asked for a parallel file.

## Deferred

- Automated `scripts/check-unsafe-index.sh` parity with any future `scripts/check-panic-index.sh` | mechanical CI gate enforcement | separate issue if added
- Per-entry severity tagging (e.g. "structurally unavoidable" vs "removable with refactor") | beyond the panic-index template's surface area | revisit if the index grows beyond ~20 entries

## Key decisions

| Question | Decision |
|---|---|
| New file or merge into panic-index? | New file. User explicitly asked for a parallel file ("as it already done for panic index"). |
| What counts as a "production unsafe site"? | Mirror panic-index convention: any `unsafe { … }` block or `unsafe fn` declaration outside `#[cfg(test)]` modules, `tests/` integration-test directories, `benches/`, and `examples/`. |
| Per-entry schema fields | Mirror panic-index 5-field per-entry table, adapted for unsafety: **Location**, **Why unsafe**, **Safety invariant**, **Why not safe Rust**, **Preferred fix**. (Schema may be refined in design — these field names are draft.) |
| Sync gate severity | Same as panic-index: missing entry for a new production unsafe site is `major` in review-findings and REJECT in self-review. |
| Primary detection signal | The `# Safety` doc section on `unsafe fn` / `unsafe trait` declarations (analogous to `# Panics` for panic-index). The `unsafe`-keyword grep is the secondary catch-net. |
| Documentation lint relationship | `#![warn(clippy::undocumented_unsafe_blocks)]` (already workspace-wide per AGENTS.md *Code Style → Documentation*) guarantees a `// SAFETY: …` comment on every `unsafe { … }` block. The unsafe-index entry is the per-site narrative record; the `// SAFETY:` comment is the in-source one-liner. Both exist; neither replaces the other. |

## Technical constraints

- `ai-docs/unsafe-index.md` must be ≤ 40,000 chars (workspace file-size axiom). With ~2 initial entries × ~600 chars/entry, plus preamble, the file starts well under 5,000 chars — comfortable headroom.
- Touchpoint instruction files must remain under the 35,000-char early-warning band after the edit:
  - `AGENTS.md` is at 33,328 chars (1,672 chars of headroom). The touchpoint #4/#5 edits add ≲ 200 chars combined; verify with `wc -c AGENTS.md` after the edit and apply the early-warning extraction model from AGENTS.md *Build & Test* if the file crosses 35,000.
  - `.claude/skills/task/SKILL.md` (21,455), `.claude/skills/task/reference.md` (28,867), `.claude/agents/review-findings.md` (19,308), `.claude/agents/self-review.md` (22,196) — all have ≥ 6,000 chars of headroom; no extraction expected.
- Initial-population content for the 2 sites must use the schema verbatim; no field may be left blank or filled with placeholder text — every entry must give the human reader and the review-findings agent enough context to evaluate "is this still the right shape?" on subsequent diffs.
- `unsafe-index.md` is a regular committed artefact (NOT gitignored). It is staged alongside the implementation commit on every PR that adds an unsafe site, parallel to `panic-index.md`'s staging rule in `reference.md` § Step 9 — panic-index sync (detail).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ai-docs/unsafe-index.md` exists, is committed (NOT gitignored), opens with a preamble paragraph mirroring panic-index's ("Production `unsafe` sites — tracked here for future hardening…"), and documents the per-entry schema (Location / Why unsafe / Safety invariant / Why not safe Rust / Preferred fix) before the `## Active entries` section. |
| AC2 | `ai-docs/unsafe-index.md` § *Active entries* contains substantive entries for both `quartzite-renderer/src/wrapped_handler.rs:39` (raw-pointer Cell deref in `Drop` impl) and `quartzite-renderer/src/window_registry.rs:160` (raw-pointer deref of `active_loop` in `try_create_window`). Every field is filled with substantive content; no placeholders. |
| AC3 | `.claude/skills/task/SKILL.md` Step 9 verify-list line names "unsafe-index sync" alongside "panic-index sync" and cross-links to the new `reference.md` § *Step 9 — unsafe-index sync (detail)*; the Step-9 progress-write instruction additionally records unsafe-index additions in the `## Decisions log` (parallel to the panic-index bullet, omitted when none). |
| AC4 | `.claude/skills/task/reference.md` contains a new § *Step 9 — unsafe-index sync (detail)* section with the two `rg` recipes (one for `^\s*///\s*#\s*Safety` doc sections, one for `\bunsafe\s*\{|\bunsafe\s+fn\b` filtered to production code) and the staging instruction ("Stage `unsafe-index.md` with the implementation commit. Skip when this task added no new production unsafe sites."). The Step-9 verify-list enumeration and the gate-checklist Step-9 row both reference the new sync. |
| AC5 | `.claude/agents/review-findings.md` § *1. Safety and correctness* contains a "Unsafe-index sync" bullet (parallel shape to the existing "Panic-index sync" bullet): every public fn/method with `# Safety` doc section AND every production `unsafe { … }` / `unsafe fn` outside `#[cfg(test)]` MUST have an `ai-docs/unsafe-index.md` entry; missing → `major`. Doc-section is primary signal; grep is the catch-net. |
| AC6 | `.claude/agents/self-review.md` contains a "Unsafe-index sync" bullet parallel to the existing panic-index-sync bullet: missing entry for a new production unsafe site → REJECT (`major`). |
| AC7 | Running `rg '\bunsafe\s*\{|\bunsafe\s+fn\b' --type rust -g '!**/tests/**' -g '!**/*test*.rs'` (or the equivalent recipe captured in AC4) against the workspace returns at least the 2 sites listed in AC2, and every returned site outside `#[cfg(test)]` is covered by an entry in `ai-docs/unsafe-index.md`. (Catch-net audit: no production unsafe goes untracked at merge time.) |
| AC8 | All instruction files edited by AC3–AC6 remain under 35,000 chars (`wc -c` check). No file crosses the 35,000-char early-warning band. |
| AC9 | `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test`, and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` all pass on the final commit. (Sanity floor — the changes are doc-only, so all four must remain green.) |

## Open questions

- Whether to add a row for `ai-docs/unsafe-index.md` to `AGENTS.md` § *Agent Docs* depends on whether `ai-docs/panic-index.md` has one today. The design agent should grep for a panic-index row and mirror its presence/absence. Defensible default: if panic-index has no row, neither does unsafe-index, and the user can request both in a follow-up.
- Whether the per-entry schema's **Why unsafe** field should distinguish "FFI/ABI", "raw-pointer deref", "transmute", and "unsynchronised shared mutability" as a tagged taxonomy is deferred until the index grows past ~5 entries. For 2 entries, free-form prose is enough — defer until taxonomy patterns are visible.
- Whether to add a `scripts/check-unsafe-index.sh` CI gate (parallel to any panic-index parity) is left to a follow-up if/when manual review fails to catch a regression — the current panic-index relies on review-findings + self-review with no script gate, so the unsafe-index follows the same enforcement model in v1.
