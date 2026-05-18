# Audit `clippy::doc_link_code` workspace allow

**Source:** issue #454
**Date:** 2026-05-18
**Tracked in:** #454

## Scope

1. Run the survey recipe and record per-file hit counts for `clippy::doc_link_code` across the workspace.
2. Inspect every hit site and classify its intent (legitimate use of the `[` `Foo` `](path)` adjacency form vs accidental backtick-wrapping that could be flattened to `[Foo]`).
3. **Narrow** the lint posture: remove the `doc_link_code = "allow"` entry from root `Cargo.toml`'s `[workspace.lints.clippy]` block and add a per-line `#[allow(clippy::doc_link_code, reason = "…")]` (or the appropriate `#![allow(...)]` for `//!` crate-level doc) at each remaining legitimate flagged site.
4. Each `reason = "…"` string names the adjacency-to-`(args)` pattern that justifies the local allow, so a future re-audit at the source site has the rationale in-line.
5. Run the doc gate (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`) and `cargo clippy --workspace --all-targets -- -D warnings` to confirm exit 0 after narrowing.

## Out of scope

- Mass-normalisation of `[` `Foo` `]` → `[Foo]` across non-flagged prose. The survey shows the lint flags only the *code-link-adjacent-to-code-text* pattern, not bare `[` `Foo` `]` doc references; there is no workspace-wide sweep to perform.
- Re-litigating the broader doc-convention link-form choice between inline `[Foo](path)` vs reference `[Foo][path]` — already decided by `ai-docs/doc-convention.md § Linking and code references`.
- Bundling unrelated `[workspace.lints.clippy]` allow-list audits into this PR (each allow entry gets its own audit per the audit-allows backlog).
- Extending `ai-docs/doc-convention.md` with a written policy for the adjacency-to-`(args)` pattern. The per-line `reason = "…"` strings at the two sites carry the rationale; a separate doc-convention paragraph is not required for the Narrow branch. (User picked plain `Narrow` over `Narrow + doc-convention note` in round 1.)

## Deferred

- None.

## Key decisions

| Question | Decision |
|---|---|
| What does the survey reveal? | Exactly **2** flagged sites across the workspace, both production library code: `quartzite-runtime/src/object_tree_ext.rs:16` (renders `Err(TreeAccessError)` with both names linked: `` [`Err`]`(`[`TreeAccessError`]`)` ``) and `quartzite-style/src/default_style.rs:258` (renders `disabled(color)` with `disabled` linked: `` [`disabled`]`(color)` ``). |
| Is the form `[` `Foo` `](path)` *adjacent to* `(args)` ever legitimate? | **Yes.** Both flagged sites use this pattern to render a code-styled function-call expression where one identifier inside is a clickable intra-doc link and the rest of the expression remains code-styled. Flattening to `[Foo](path)` would lose the surrounding code styling for `(args)` / `(`…`)`. The lint cannot distinguish this intentional shape from accidental backtick-wrapping. |
| Is "Fix" (mass-normalise to `[Foo]`) viable? | **No.** Both flagged sites depend on the adjacency-to-`(args)` shape; neither can be flattened without losing meaning. The "Fix" option from the issue body presupposes a normalisation target that the survey shows does not exist. The audit collapses to a Keep-vs-Narrow choice. |
| Keep vs Narrow? | **Narrow** (resolved round 1). Remove the workspace allow; add `#[allow(clippy::doc_link_code, reason = "…")]` at the 2 flagged sites. Each future accidental hit then surfaces in CI for case-by-case review; preserves global strictness, minor source noise at the 2 sites. |
| Does this audit extend `ai-docs/doc-convention.md`? | **No.** The user picked plain `Narrow`, not `Narrow + doc-convention note`. The per-line `reason = "…"` strings are the rationale-of-record; a separate paragraph in `ai-docs/doc-convention.md` is out of scope. |
| How is the audit's outcome recorded? | Per-line `reason = "…"` strings at the two flagged sites carry the survey count and adjacency rationale. The PR description and the issue-closing comment also name the count and the chosen branch. |

## Technical constraints

- Survey recipe (from issue body, verified locally on master at 2026-05-18):
  ```bash
  cargo clippy --workspace --all-targets --message-format=short -- -W clippy::doc_link_code 2>&1 \
    | rg 'doc_link_code|code link adjacent to code text' \
    | awk -F: '{ print $1 }' | sort | uniq -c | sort -rn
  ```
  Note: the recipe in the issue body filters on `doc_link_code`, but the rendered warning message reads `code link adjacent to code text`; the survey must also pass `-W clippy::doc_link_code` so the lint is **temporarily upgraded from `allow` to `warn`** for the duration of the run (otherwise zero hits surface because the workspace allow suppresses output).
- Both currently-flagged sites are `[` `Name` `](path)` adjacent to either `[` `OtherName` `]` or to a literal `(arg)` substring — flattening to `[Name]` is structurally impossible here because the link target `(path)` is itself parenthesised syntax inside the form.
- The doc gate uses `--all-features` per `ai-docs/doc-convention.md § Intra-doc links to feature-gated modules`; the audit must not introduce any new intra-doc link that breaks under that flag.
- The per-line allow at `quartzite-runtime/src/object_tree_ext.rs:16` lives inside a trait doc comment — the `#[allow]` must be attached to the trait item (or an enclosing block), not the doc line, because attributes do not nest inside `///` text. The same applies at `quartzite-style/src/default_style.rs:258`, which is on a free fn — the attribute attaches to the fn item.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Survey recipe (with the `-W clippy::doc_link_code` correction recorded in *Technical constraints*) executed on the implementation branch; per-file hit count saved into the design or progress notes and confirmed to still equal the 2 sites named in *Key decisions*. |
| AC2 | The `doc_link_code = "allow"` entry is removed from root `Cargo.toml`'s `[workspace.lints.clippy]` block (including the justifying comment above it). |
| AC3 | Each of the 2 flagged sites carries `#[allow(clippy::doc_link_code, reason = "…")]` (or the appropriate `#![allow(...)]` for `//!` crate-level doc) attached to the smallest enclosing item that suppresses the warning. |
| AC4 | Each `reason = "…"` string names the adjacency pattern (`[` `Foo` `](path)` adjacent to `(args)` / another `[` `…` `]`) that justifies the local allow. |
| AC5 | `cargo clippy --workspace --all-targets -- -D warnings` exits 0 with the workspace allow removed. |
| AC6 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` exits 0. |
| AC7 | `cargo fmt -- --check` exits 0. |
| AC8 | A re-run of the survey recipe (with `-W clippy::doc_link_code` upgraded to `warn`) returns the same 2 sites — no new accidental hits introduced by the audit's own edits. |
| AC9 | Issue #454 closed by PR merge; the closing comment names the survey count (2) and the chosen branch (Narrow). |

## Open questions

- None.
