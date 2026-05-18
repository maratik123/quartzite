# Design: Audit `clippy::doc_link_code` workspace allow — Narrow to per-site

**Issue:** #454
**Date:** 2026-05-18

## Approach

The spec resolves the audit to the **Narrow** branch: remove the workspace-level
`doc_link_code = "allow"` entry from root `Cargo.toml`'s
`[workspace.lints.clippy]` block and add per-site `#[allow(clippy::doc_link_code,
reason = "…")]` at exactly the 2 currently-flagged production-doc sites.

Verified survey (clean `cargo clean -p` on each affected crate, full long-format
clippy run on `2026-05-18`):

- `quartzite-runtime/src/object_tree_ext.rs:16:5` — inside the trait-level doc
  of `pub trait ObjectTreeExt` (doc spans lines 9–30, trait item at line 31).
  The exact span is the substring
  `` [`Err`]`(`[`TreeAccessError`]`)` ``
  inside the line `` /// [`Err`]`(`[`TreeAccessError`]`)` when called outside an active ``.
- `quartzite-style/src/default_style.rs:258:13` — outer doc comment of the free
  fn `maybe_disabled` (line 259). The exact span is the substring
  `` [`disabled`]`(color)` ``
  inside the line `` /// Returns [`disabled`]`(color)` when `enabled` is `false`; otherwise `color` unchanged. ``

Both warnings reproduce verbatim against `rust-1.95.0`; both render the
adjacency-to-code-text shape the spec's *Key decisions* identify as legitimate.

**Attribute placement.** `#[allow(...)]` is an outer attribute on the smallest
enclosing item:

- Site 1 → attaches to `pub trait ObjectTreeExt` (before the `pub trait` line,
  immediately after the closing `///` doc line at 30). The outer-attribute form
  is required because the warning fires on the trait's own doc comment, not on
  a method's doc inside the trait body; an inner `#![allow(...)]` would have to
  live at the crate root, which over-scopes.
- Site 2 → attaches to `fn maybe_disabled` (between the doc line at 258 and the
  `fn` keyword at 259).

Neither item carries any other outer attribute today, so the new `#[allow]` is
the only attribute above each item. No attribute-ordering concerns.

**Rejected alternatives:**

1. **Keep** (workspace-wide allow stays): rejected by spec round-1 resolution.
   Loses CI surfacing of any future accidental backtick-wrapping.
2. **Fix / mass-normalise to `[Foo]`**: the spec's *Key decisions* row 3 records
   the survey-confirmed result that both sites depend on the
   adjacency-to-`(args)` shape and cannot be flattened. Not viable.
3. **Apply clippy's own suggestion** (`<code>[Err]([TreeAccessError])</code>`):
   the rendered help text proposes wrapping the whole group in `<code>` HTML
   tags. Rejected because (a) the spec mandates `Narrow` (per-site `#[allow]`
   with `reason = "…"`), not a rewrite of the doc prose, and (b) introducing
   raw HTML in rustdoc cuts against the project's plain-Markdown doc
   convention; the per-line allow keeps the source readable as Markdown.
4. **Block-scoped `#![allow(...)]`** at the crate-root of `quartzite-runtime`
   and `quartzite-style`: rejected — over-scopes to two whole crates and
   silently suppresses any future accidental hit anywhere in those crates,
   defeating the audit's stated goal of surfacing accidents case-by-case.
5. **Module-scoped `#![allow(...)]`** at the top of each of the two `.rs`
   files: still over-scopes relative to the per-item `#[allow]`, and adds a
   crate-root-style inner attribute pattern that the file does not otherwise
   use. Per-item placement is the minimum-blast-radius option.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Confirm survey on the implementation branch: run the corrected recipe (with `-W clippy::doc_link_code` upgrade) and record the per-file hit count in the progress notes; verify it equals the 2 sites named in *Key decisions* (AC1, AC8 baseline). | (no source edit; progress notes only) | — |
| 2 | Remove `doc_link_code = "allow"` from root `Cargo.toml`'s `[workspace.lints.clippy]` block, **including the justifying comment line above it** (AC2). After the edit, build to refresh `Cargo.lock` if anything moves (`cargo build`). Note: `rustfmt` does not touch `Cargo.toml`; the edit is by hand. After removing the row (last entry in the block), verify there is no trailing blank-line drift. | `Cargo.toml` | 1 |
| 3 | Add `#[allow(clippy::doc_link_code, reason = "adjacency-to-(args) pattern: renders Err(TreeAccessError) with both identifiers intra-doc-linked; flattening to [Err]([TreeAccessError]) would drop the surrounding code styling")]` above `pub trait ObjectTreeExt` (between line 30 and line 31). | `quartzite-runtime/src/object_tree_ext.rs` | 2 |
| 4 | Add `#[allow(clippy::doc_link_code, reason = "adjacency-to-(args) pattern: renders disabled(color) with disabled intra-doc-linked; flattening to [disabled](path) would drop the surrounding code styling on (color)")]` above `fn maybe_disabled` (between line 258 and line 259). | `quartzite-style/src/default_style.rs` | 2 |
| 5 | Run the three gates and a re-run of the survey recipe: `cargo clippy --workspace --all-targets -- -D warnings` (AC5), `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` (AC6), `cargo fmt -- --check` (AC7), survey re-run (AC8). All four must exit 0 / show the same 2 sites. | (no source edit) | 3, 4 |

Five subtasks. Within the 7-subtask budget.

## Handoff plan

Five subtasks → one non-terminal group of 3 (subtasks 1–3) followed by one
terminal group of 2 (subtasks 4–5). Boundary respects the 3-subtask cap and
keeps the terminal group within the `1..=3` range.

- **Entry into Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry);
  proceed with subtasks 1–3.
- **Group A:** subtasks 1–3 — survey re-confirmation, root-`Cargo.toml` edit,
  first per-site `#[allow]` on `quartzite-runtime` (3 subtasks; equals the
  non-terminal cap).
- **Handoff after Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
  Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–5 — second per-site `#[allow]` on `quartzite-style`,
  followed by the gate suite + survey re-run (2 subtasks; terminal group;
  within the `1..=3` range).

## Risks

- **Reason-string Markdown rendering in compiler output**: clippy / rustc render
  attribute `reason = "…"` strings as plain text in diagnostics; the backticks
  inside the string are preserved verbatim. No risk of `reason`-string
  re-tripping the lint (the lint inspects rendered doc comments, not attribute
  string literals). Mitigation: none required — confirmed by reading the lint
  source pointer (`rust-lang/rust-clippy` `doc_link_code` lints the rendered
  Markdown tree of `///` / `//!` doc comments; attribute string literals are
  not parsed as Markdown).
- **Re-rendering of `(args)` adjacency in future doc edits**: if the doc text
  on either flagged item is rewritten and the adjacency form is dropped, the
  per-site allow becomes dead. Mitigation: the `reason = "…"` string names the
  exact pattern; a future re-audit at that site will surface the dead allow as
  reviewer-visible noise and prompt removal. No mechanical guard added (out of
  scope per spec's *Out of scope* row 4).
- **Doc-gate breakage from intra-doc-link resolution under `--all-features`**:
  the audit does not introduce or remove any intra-doc link target — both
  sites already used `[`Err`]` / `[`TreeAccessError`]` / `[`disabled`]` before
  the audit. Mitigation: AC6 re-runs the doc gate with `--all-features` per
  `ai-docs/doc-convention.md § Intra-doc links to feature-gated modules`.
- **Workspace `Cargo.lock` churn**: removing a `[workspace.lints.clippy]` entry
  does not alter dependency resolution. `cargo build` re-run in Subtask 2 is a
  defensive check, not an expected lock-file change. Mitigation: if `Cargo.lock`
  somehow updates, stage it with the commit per `AGENTS.md § Build & Test` /
  `§ Workflow`.
- **Adjacent lint suppression**: `clippy::doc_link_code` is the only lint named
  in each `#[allow]`. No risk of incidentally widening the suppression to
  neighbouring lints. Mitigation: explicit single-lint form (`clippy::doc_link_code`,
  not a group).

## Test Design

This audit has no behavioural surface — the changes are (a) one workspace-lint
config entry removed and (b) two source-attribute additions on items whose
runtime behaviour is unaffected. **No new `#[cfg(test)]` tests are added.**

The audit's correctness is fully captured by the three existing gates plus the
survey recipe:

- **Gate A — Clippy** (`cargo clippy --workspace --all-targets -- -D warnings`):
  exits 0 (AC5). Verifies that removing the workspace allow plus adding the two
  per-site allows yields zero residual warnings, i.e. both per-site allows are
  correctly placed and the audit has not surfaced any third accidental hit.
- **Gate B — Doc** (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`):
  exits 0 (AC6). Verifies no intra-doc-link breakage (the audit touches doc
  prose only via attributes above the doc, not via doc-text edits — this gate
  is a defence-in-depth check, not a likely failure surface).
- **Gate C — Fmt** (`cargo fmt -- --check`): exits 0 (AC7). Verifies that the
  two new long-line attributes either fit under 100 cols or are accepted by
  rustfmt's attribute-formatting rules.
- **Gate D — Survey re-run**: re-runs the corrected recipe from the spec's
  *Technical constraints*, confirms the same 2 sites still surface when the
  lint is upgraded to `warn`, and no third (AC8). Implicit confirmation that
  the audit's own edits introduced no new accidental hit.

The two `#[allow]` attributes are themselves the "test fixture" — their
`reason = "…"` strings encode the rationale and are inspected by any future
re-audit.

## Open questions

- None. The spec's *Key decisions* fully resolves Keep-vs-Narrow, the
  doc-convention extension question, and the survey count; the design only
  needed to fix the attribute placement and draft the `reason` strings.
