# Fix AC# references that leak into generated docs

**Source:** issue #557
**Date:** 2026-05-24
**Tracked in:** #557

## Scope

The issue surfaced a single `///`-comment leak (`AC10` in
`quartzite-renderer/src/vello_painter.rs:27`), but the round-1 sweep
revealed several more sites across the workspace. The PR fixes **every
real or potential AC# leak that `cargo doc` could pick up** — across all
crates — and corrects the factual error at the original site. No
behaviour change; no new dependency; no new automated guard in this PR
(the guard is split into a separate gh issue — see § *Deferred*).

### 1. Original site — strip + correct

`quartzite-renderer/src/vello_painter.rs:27` — the `pub struct
VelloPainter<'a>` doc block currently says "skip drawing silently for
unsupported brush kinds (e.g. gradient brushes — AC10); text methods …".
Two problems:

- `AC10` references the spec acceptance criterion — leaks into rustdoc.
- The factual claim is wrong: gradient brushes (`LinearGradient`,
  `RadialGradient`, `Custom`) **are** supported in the renderer
  (issue #281 / spec `2026-05-14-paint-brush-gradient-variants`); only
  future unknown `#[non_exhaustive]` `BrushKind` variants fall through
  to the no-brush sink.

Replacement wording must (a) remove `AC10`, (b) accurately describe
current behaviour — gradient variants paint via vello; only future
unknown `BrushKind` variants are silently skipped.

### 2. Workspace-wide `///` rustdoc sweep on `src/**`

Rewrite every `///`-comment AC# reference in `**/src/**/*.rs` to
describe the behaviour directly. Sites found by the round-1 grep
(`rg -n '^\s*///.*\bAC[0-9]+\b' --type rust -g '**/src/**'`):

- `quartzite-renderer/src/vello_painter.rs:27` — `pub struct
  VelloPainter<'a>` (covered above; the canonical "original site").
- `quartzite-renderer/src/vello_painter.rs:184` — private
  `LocalBrushKind::from_brush_kind` ("satisfying AC4").
- `quartzite-renderer/src/render_harness.rs:91–92` — `pub` builder
  method `scale_factor` ("see AC11 / AC12 in the renderer-painter-impls
  spec").
- `quartzite-renderer/src/window_registry.rs:209–210` — `pub(crate)
  insert_root_for_test` ("(AC3, AC5)").
- `quartzite-renderer/src/wrapped_handler.rs:332,366,398,432` — `///`
  on `#[cfg(test)]` test functions ("AC3", "AC5").
- `quartzite-core/src/traits.rs:289` — `ObjectExt::downcast_ref`
  ("(AC8)").
- `quartzite-core/src/signal.rs:68` — `ConnectionType::Auto`
  ("(see AC5)").
- `quartzite-style-types/src/dark_palette.rs:109,142,160` — `///`
  on `#[cfg(test)]` test functions ("AC8 — …").
- `quartzite-style/src/default_style_tests.rs:945,994,1604,1663,1718,
  1737,1762` — `///` on `#[cfg(test)]` test functions
  ("AC10 — …", "AC2 / #407 fold-in …", "AC5 — …", "AC4 — …", "AC4 /
  #407 fold-in …").

The acceptance verification re-runs the grep against `**/src/**` and
requires zero hits (see AC3).

### 3. Workspace-wide `//!` module-doc sweep on `src/**`

Rewrite every `//!`-comment AC# reference in `**/src/**/*.rs`. The
round-1 grep (`rg -n '^\s*//!.*\bAC[0-9]+\b' --type rust -g '**/src/**'`)
returned **zero hits** — sweep is preventative; AC verifies cleanliness.

### 4. Macro-generated doc surface (`quartzite-macros/`)

Every AC# reference inside `quartzite-macros/src/**` is a `//` line
comment (verified by `rg -n 'AC[0-9]+' quartzite-macros/src/`); no
`#[doc = "…"]` attribute, no string literal spliced into a `///`
comment, no `quote!` block emitting `///`-prefixed lines mentioning
`AC#`. Macro-generated doc surface is therefore already clean and no
edits are required in this crate. The PR's verification AC re-asserts
that property via grep (see AC4).

### 5. Cargo.toml feature docs (`document-features`)

`document-features` propagates `#!` / `##` comments in `[features]`
sections of every workspace `Cargo.toml` into the rendered crate
rustdoc. Round-1 grep (`rg -n 'AC[0-9]+' -g 'Cargo.toml'`) returned
**zero hits**. Sweep is preventative; AC verifies cleanliness (see AC5).

### 6. Examples (`examples/**`)

`cargo doc` includes example sources under `--examples` (or by default
when an example has `doc = true`). Round-1 grep (`rg -n 'AC[0-9]+' -g
'examples/**'`) returned **zero hits**. Sweep is preventative; AC
verifies cleanliness (see AC6).

### 7. Deferred guard — separate gh issue

The user explicitly deferred the automated recurrence guard
(LLM-based, CI script, or doc-test bash/Python check) to a follow-up
issue. The PR for #557 **creates the gh issue** as part of its scope;
the guard's implementation is not part of this PR. See § *Deferred*.

## Out of scope

- Any behaviour change, public-API rename, or new dependency. Docs-only fix.
- `//` line comments **inside function bodies** (test setup, anchor
  banners like `// ── AC10a: resolver miss mid-tree …`, traceability
  callouts). These do not reach `cargo doc` output and remain useful
  AC-pinned-test traceability anchors. Sites confirmed in scope of
  this exclusion include `quartzite-widgets/src/widget_ext.rs:649,696`,
  `quartzite-core/src/signal.rs` lines 270 / 744 / 767 / 788 / 816 /
  857 / 993 / 1041 / 1099 / 1147 / 1273 / 1314 / 1350,
  `quartzite-core/src/connect.rs` lines 812 / 879 / 951 / 971 / 1167 /
  1210 / 1214 / 1221, every AC# in `quartzite-macros/src/**`, every AC#
  in `quartzite-style-dispatch/src/dispatch.rs`, and every AC# under
  `quartzite-style/src/default_style_tests.rs` that is a `//` (not `///`)
  line comment.
- Anything outside `**/src/**`, `**/examples/**`, `Cargo.toml`, or
  macro-generated doc surface — i.e. `tests/*.rs` integration tests are
  NOT touched regardless of comment style. Integration-test files are
  not part of crate rustdoc.
- Implementing the recurrence guard itself (deferred to the separate
  issue this PR opens).

## Deferred

| Item | Why | Separate issue? |
|---|---|---|
| Automated AC# leak guard (LLM-based, CI script, or doc-test bash/Python) | User explicitly deferred in round-2 Q3 answer; design needs its own discussion | **Yes** — PR for #557 creates a new gh issue ("Add automated guard for AC# rustdoc leaks") and links it from the PR body; the implementation lands separately. |

## Key decisions

| Question | Decision |
|---|---|
| Scope of the AC# cleanup (round-2 Q1) | Broadest: every site that `cargo doc` could include — `///`, `//!`, macro-generated docs, `Cargo.toml` `document-features` blocks, and `examples/**`. All four real or potential surfaces, even when current grep returns zero hits. |
| Replacement wording at `vello_painter.rs:27` (round-2 Q2) | Strip the `AC10` token AND correct the factual claim: gradient brushes (linear / radial / custom) **are** supported in the renderer; only future unknown `#[non_exhaustive]` `BrushKind` variants fall through to the no-brush sink. |
| Recurrence guard (round-2 Q3) | Defer to a separate gh issue; this PR creates that issue but does not implement the guard. |
| `///` vs `//` distinction | Only `///`, `//!`, `#[doc=…]`, `document-features`-propagated `##` / `#!`, and example source lines reach `cargo doc` output. Plain `//` line comments (function bodies, banners) are internal-only and stay as test-traceability anchors. |
| Test approach for the docs-only edit | No new behavioural test. The fix is verified by (a) re-running the round-1 greps and requiring zero hits, (b) `cargo doc` clean under workspace doc gate, (c) `cargo clippy` + `cargo fmt --check` + `cargo test --workspace` clean. |

## Technical constraints

- `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` MUST pass.
- `cargo clippy --workspace --all-targets -- -D warnings` MUST pass.
- `cargo fmt -- --check` MUST pass.
- `cargo test --workspace` MUST pass (regression sanity; no behaviour change expected).
- No new dependency.
- Existing `//` line-comment AC anchors (function bodies, banner lines,
  test-section delimiters) MUST NOT be touched.
- Test files under `**/tests/**` MUST NOT be touched.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `quartzite-renderer/src/vello_painter.rs` — the `pub struct VelloPainter<'a>` doc block contains zero `AC<digit>` substrings AND accurately reflects current behaviour (gradient brushes — linear, radial, custom — are supported; only future unknown `#[non_exhaustive]` `BrushKind` variants fall through to the no-brush sink). Verified by reading the doc block and cross-checking the wording against the `match` arms in `LocalBrushKind::from_brush_kind` in the same file. |
| AC2 | `quartzite-renderer/src/vello_painter.rs:184` — the `///` on `LocalBrushKind::from_brush_kind` contains zero `AC<digit>` substrings. |
| AC3 | Workspace-wide: `rg -n '^\s*///.*\bAC[0-9]+\b' --type rust -g '**/src/**'` returns **zero matches** across the entire workspace (covers `quartzite-renderer`, `quartzite-core`, `quartzite-style-types`, `quartzite-style`, every `src/**/*.rs` in the repo). |
| AC4 | Workspace-wide: `rg -n '^\s*//!.*\bAC[0-9]+\b' --type rust -g '**/src/**'` returns **zero matches**. (Currently zero — sweep is preventative; AC verifies the invariant holds after the edit.) Also `rg -n 'AC[0-9]+' quartzite-macros/src/` either returns zero matches OR every remaining match is a plain `//` line comment (not `///`, not inside a `#[doc=…]` attribute, not inside a `quote!`-spliced `///` line). |
| AC5 | `rg -n 'AC[0-9]+' -g 'Cargo.toml'` returns **zero matches** across the workspace. (Currently zero — preventative.) |
| AC6 | `rg -n 'AC[0-9]+' -g 'examples/**'` returns **zero matches**. (Currently zero — preventative.) |
| AC7 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` passes. |
| AC8 | `cargo clippy --workspace --all-targets -- -D warnings` passes. |
| AC9 | `cargo fmt -- --check` passes. |
| AC10 | `cargo test --workspace` passes. |
| AC11 | A new gh issue exists titled approximately "Add automated guard for AC# rustdoc leaks" (or equivalent), describing the deferred recurrence-guard work (LLM-based, CI script, or doc-test bash/Python). The PR description for #557 references this new issue. |
| AC12 | The PR description for this work closes issue #557 (`Closes #557` in the body). |

## Open questions

_(None — Q1, Q2, Q3 resolved in round 2; the spec is design-ready.)_
