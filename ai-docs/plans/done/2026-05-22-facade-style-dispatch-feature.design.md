# Design: Facade `style-dispatch` feature in `quartzite/Cargo.toml`

**Issue:** #393
**Date:** 2026-05-22

## Approach

The work is a faithful additive transplant of the existing `style` and `widgets` facade-feature pattern to a third sibling: `style-dispatch`. The bridge crate (`quartzite-style-dispatch`) is already a workspace member (root `Cargo.toml` line 14), and its public surface is two items — `dispatch_paint` (free fn) and `WidgetResolver` (trait) — both re-exported from `crate::dispatch` at the bridge crate's root.

**Chosen shape:**

1. **`Cargo.toml`** (workspace root, the facade's package manifest, single file at lines 1–152):
   - Add `quartzite-style-dispatch = { path = "quartzite-style-dispatch", optional = true }` to `[dependencies]` (line 102 area, immediately after the existing `quartzite-widgets` entry so the optional deps cluster).
   - Add a `## ` doc-commented `style-dispatch` entry to `[features]`. Value chains both prerequisites: `style-dispatch = ["dep:quartzite-style-dispatch", "style", "widgets"]`. This matches the spec § *Key decisions* "Feature chaining" row and AC1's exact literal.
   - Append `"style-dispatch"` to `[package.metadata.docs.rs] features` array (currently line 149).

2. **`src/lib.rs`** (lines 1–395 — under the 500-line outline threshold so direct Read was used; design phase confirmed there is no `style_dispatch` mention today):
   - Add a new `pub mod style_dispatch { pub use quartzite_style_dispatch::*; }` block gated on `#[cfg(feature = "style-dispatch")]` + `#[cfg_attr(docsrs, doc(cfg(feature = "style-dispatch")))]`. Place it immediately after the existing `widgets` module (line 334 area), before the `prelude` module. Carry a `///` doc comment naming `dispatch_paint` and `WidgetResolver`, stating the bridge dep, and noting the `style-dispatch` feature requirement.
   - Add one `# Ecosystem` bullet for the bridge crate, placed after the existing `- [`paint`] (`quartzite-paint-api`) — …` bullet (line 183–184) and before the `Add quartzite to your Cargo.toml:` paragraph (line 186). Format: ``- [`style_dispatch`] (`quartzite-style-dispatch`) — widget-tree paint dispatcher. Requires the `style-dispatch` feature.`` Bullet uses unqualified `[`style_dispatch`]` (module link) so rustdoc resolves it via the same-crate `style_dispatch` module — and only when the feature is enabled, which `docs.rs` arranges via the metadata array. **Risk note:** under the default-features build the link target does not exist; `broken_intra_doc_links = "deny"` could fail. Mitigation in § *Risks* row "Ecosystem bullet broken-link risk".
   - Add a `#[cfg(all(test, feature = "style-dispatch"))]` test inside the existing `mod tests` (line 387). This is the AC6 mechanical re-export check: name `quartzite::style_dispatch::dispatch_paint`, `quartzite::style_dispatch::WidgetResolver`, `quartzite::style::Palette`, and `quartzite::widgets::WidgetBase`, demonstrating the chained features are wired through.

**Alternatives considered & rejected:**

- *Independent (non-chaining) `style-dispatch` feature.* Rejected per spec § *Key decisions* row "Feature chaining": a caller enabling `style-dispatch` must already have `style` + `widgets` compilable to use `dispatch_paint`; an independent feature forces "enable, fail, enable, fail" iteration. Documented departure from the single-crate-feature precedent set by `style` and `widgets`.
- *Sub-module `style::dispatch` shape.* Rejected per § *Open questions* row 1: the top-level `style_dispatch` mirrors the crate-name convention (`paint`, `widgets`, `runtime`) and decouples the module path from the chain decision. A future ergonomic prelude / facade helper can pick `style::dispatch` if needed; the top-level path is more stable.
- *Selective re-export `pub use quartzite_style_dispatch::{dispatch_paint, WidgetResolver};`.* Rejected per § *Key decisions* row "Re-export shape": the bridge crate's public surface is intentionally minimal (two items today). Using `pub use ::*` is the verbatim pattern of the sibling modules (`style`, `widgets`, `paint`, `runtime`, `geometry`, `events`) and survives the bridge crate adding more public items in the future without a facade-side edit.
- *Facade-level doc-test inside the `style_dispatch` module.* Rejected per § *Key decisions* row "Self-contained doc-test inside the facade `style_dispatch` module": the bridge crate's `lib.rs` already carries the AC9 end-to-end doc-test of the design that birthed the bridge crate. A facade-side doc-test would duplicate it verbatim and force facade dev-deps growth.
- *Adding `WidgetResolver` to `quartzite::prelude`.* Rejected per spec § *Out of scope* row "Adding the bridge crate's `WidgetResolver` to `quartzite::prelude`": `quartzite_widgets::layout::WidgetResolver` already exists (verified via direct read of `quartzite-widgets/src/lib.rs` line 25); pulling either into the prelude under one bare `WidgetResolver` name would force a rename.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `style-dispatch` feature entry + `quartzite-style-dispatch` optional dep to facade `Cargo.toml`. Add the `## ` doc-comment expected by `document_features` (two sentences per spec § *Open questions* row 2 default wording; plain prose, no `[link]` syntax). Append `"style-dispatch"` to `[package.metadata.docs.rs] features`. | `Cargo.toml` | — |
| 2 | Add the `pub mod style_dispatch { pub use quartzite_style_dispatch::*; }` block to `src/lib.rs` with the `#[cfg(feature = "style-dispatch")]` + `#[cfg_attr(docsrs, doc(cfg(feature = "style-dispatch")))]` annotations and a `///` doc comment naming `dispatch_paint` and `WidgetResolver`. Place between the `widgets` module and the `prelude` module. | `src/lib.rs` | 1 |
| 3 | Add the `- [`style_dispatch`] (`quartzite-style-dispatch`) — …` bullet under `# Ecosystem`, after the `paint` bullet and before the `Add quartzite to your Cargo.toml:` paragraph. | `src/lib.rs` | 2 |
| 4 | Add the AC6 compile-only re-export test inside `mod tests`, gated on `#[cfg(all(test, feature = "style-dispatch"))]`. Test names `quartzite::style_dispatch::dispatch_paint`, `quartzite::style_dispatch::WidgetResolver`, `quartzite::style::Palette`, `quartzite::widgets::WidgetBase` so failure of the chain at the type level surfaces as a compile error. | `src/lib.rs` | 2 |
| 5 | Run the AC matrix: `cargo build`, `cargo build --features style-dispatch`, `cargo build --no-default-features --features std,style-dispatch`, `cargo build -p quartzite --no-default-features --features libm`, `cargo test --features style-dispatch`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo clippy --workspace --all-targets -- -D warnings`. Inspect the rendered facade rustdoc for the `style-dispatch` feature-flag row and the new module page. | — (verification only) | 1, 2, 3, 4 |

(5 atomic tasks — under the 7-task split threshold; no decomposition split.)

## Handoff plan

`M = 5` — two groups, **3 + 2**.

- **Group A:** subtasks 1–3 — Cargo manifest changes + facade module block + Ecosystem bullet. Initial-implementation chunk (3 subtasks; non-terminal group MUST be exactly 3 per design contract).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–5 — AC6 compile-only test + AC matrix verification. Terminal group (2 subtasks; within the 1..=3 range).

## Risks

- **`broken_intra_doc_links = "deny"` against the `- [`style_dispatch`] …` Ecosystem bullet under default features.** The workspace lints declare `rustdoc::broken_intra_doc_links = "deny"`, and the doc-gate command from AGENTS.md is `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` (note `--all-features` — `style-dispatch` is on for the doc gate). The bullet `[`style_dispatch`]` resolves under `--all-features`, satisfying the gate. The default-features build does NOT invoke `cargo doc`, so the disappeared link target does not produce a lint hit at build time. **Mitigation:** none required — the doc gate runs with `--all-features`, and `cargo build` (no `cargo doc`) under default features does not run rustdoc lints. **If the matrix grows a `cargo doc --no-default-features` step**, switch the bullet to use a markdown reference rather than an intra-doc link, or wrap the bullet (and the analogous `style`/`widgets`/`snapshot` bullets) in `#[cfg_attr(feature = "...", doc = "...")]` per the existing Quickstart precedent (lines 18–65). **Confirmation step:** during AC matrix verification, capture the `cargo doc` output and ensure no `broken_intra_doc_links` warning fires.

- **`document_features` doc-comment wording drift.** The `## ` Cargo-feature doc comment is what `document_features::document_features!()` (line 213) renders into the `# Feature flags` rustdoc section. The spec leaves the body open (§ *Open questions* row 2) with a sensible default. Implementer uses the default; PR reviewer may tighten. **Mitigation:** keep the wording within ~2 sentences, plain prose, no intra-doc links — matches the sibling entries (`style`, `widgets`, `serde`). If the reviewer pushes for tighter wording, the change is single-line and contained.

- **Feature-chain triggers transitive recompilation when only `style-dispatch` is requested.** Enabling `style-dispatch` pulls in `style` + `widgets`, which compile `quartzite-style` + `quartzite-widgets` + (transitively) `quartzite-style-types`. This is the intended trade per spec § *Key decisions* row "Feature chaining" — single-switch end-to-end usability. **Mitigation:** none — it is the chosen design point.

- **`cargo build --no-default-features --features libm` regression.** AC8 explicitly requires the no-std + derive-free + `style-dispatch`-off path to keep building. Because the new dep is `optional = true` and `style-dispatch` is off by default, no new code enters the no-std build. **Mitigation:** AC matrix runs `cargo build -p quartzite --no-default-features --features libm` — this is the canonical regression guard from AGENTS.md § *Build & Test*.

- **Module-name collision risk (`WidgetResolver`).** The bridge crate exports `WidgetResolver` (the resolver trait); `quartzite-widgets` also exports a `WidgetResolver` (layout's `Layout::WidgetResolver`, line 25 of `quartzite-widgets/src/lib.rs`). Both are re-exported by the facade — under different module paths (`quartzite::style_dispatch::WidgetResolver` vs `quartzite::widgets::WidgetResolver`). Glob-imports `use quartzite::{style_dispatch::*, widgets::*};` would conflict. **Mitigation:** none required in this PR — neither name reaches the prelude (spec § *Out of scope*); path-qualified imports remain the documented shape. The AC6 test uses path-qualified `quartzite::style_dispatch::WidgetResolver` and `quartzite::widgets::WidgetBase` (NOT `WidgetResolver` from `widgets`), so no shadowing in-test.

## Test Design

For each non-trivial task:

- **Task 4 — AC6 compile-only re-export test.**
  - Location: `src/lib.rs` existing `#[cfg(test)] mod tests` block (line 387).
  - Entry point: a `#[test] fn style_dispatch_re_exports_resolve()` gated on `#[cfg(all(test, feature = "style-dispatch"))]`.
  - Scenarios:
    - Names `quartzite::style_dispatch::dispatch_paint` via a typed binding `let _: fn(_, _, _, _) = quartzite::style_dispatch::dispatch_paint;` — confirms the fn symbol resolves through the facade path. (Exact signature shape is verified by the bridge crate's own tests; the facade test only confirms re-export reachability, so an `_` -placeholder fn-pointer cast suffices. If the cast is too strict, fall back to `let _ = quartzite::style_dispatch::dispatch_paint as fn(_, _, _, _);` or `core::mem::size_of_val(&(quartzite::style_dispatch::dispatch_paint as fn(_, _, _, _)));`.)
    - Names `quartzite::style_dispatch::WidgetResolver` via a `let _: Option<&dyn quartzite::style_dispatch::WidgetResolver> = None;` binding.
    - Names `quartzite::style::Palette` via `let _: quartzite::style::Palette = quartzite::style::Palette::default();` — confirms the chained `style` feature surfaces `Palette` through the facade.
    - Names `quartzite::widgets::WidgetBase` via `let _ = core::mem::size_of::<quartzite::widgets::WidgetBase>();` — confirms the chained `widgets` feature surfaces `WidgetBase` through the facade.
  - Fixtures / helpers: none — the test is type-name-only; no widget tree, no painter, no palette state.
  - **Why compile-only and not a runtime invocation:** the bridge crate's own test suite already covers `dispatch_paint` runtime behaviour. The facade test answers "do these four paths resolve when `style-dispatch` is on" — failure indicates the feature chain dropped a link, not a logic bug.

- **Task 5 — AC matrix verification (no code, only commands).**
  - `cargo build` — default features (AC5).
  - `cargo build --features style-dispatch` (first half of AC6) AND `cargo test --features style-dispatch` (so the AC6 compile-only test actually runs and the cfg-gated test fires).
  - `cargo build --no-default-features --features std,style-dispatch` (AC7).
  - `cargo build -p quartzite --no-default-features --features libm` (AC8, regression guard).
  - `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` (AC9). Then inspect `target/doc/quartzite/index.html` for the `# Feature flags` `style-dispatch` row and `target/doc/quartzite/style_dispatch/index.html` existence.
  - `cargo clippy --workspace --all-targets -- -D warnings` (AC10).
  - AC11 is satisfied as a corollary of AC6 — if the type-level names resolve, the bridge crate's surface is byte-for-byte re-exported (the `pub use ::*;` shape leaves no room for divergence).

## Open questions

- *Resolved by spec.* The spec § *Open questions* section resolves all three questions in-line (module-name spelling, doc-comment wording default, no `tests/` integration test). The implementer adopts the spec's defaults; reviewer-driven tweaks on PR (especially to the `document_features` doc-comment body) are single-line edits inside Task 1's scope.
