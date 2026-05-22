# Facade `style-dispatch` feature in `quartzite/Cargo.toml`

**Source:** issue #393
**Date:** 2026-05-22
**Tracked in:** #393

> Surfaced by `/triage` from [`ai-docs/deferred/future-crates.md`](../deferred/future-crates.md). Source spec / design: [`2026-05-13-renderer-style-dispatch.design.md`](done/2026-05-13-renderer-style-dispatch.design.md) § *Workspace registration* (closing paragraph) and § *Open questions* row 5 deferred this exact item.

Add a new `style-dispatch` Cargo feature to the **`quartzite`** facade crate (root `Cargo.toml`) that gates an optional `quartzite-style-dispatch` dependency and re-exports it through a new `quartzite::style_dispatch` module. Pattern mirrors the existing `style` and `widgets` features: `feature → optional dep entry → cfg-gated module in `src/lib.rs` with `#[cfg_attr(docsrs, doc(cfg(feature = "..." )))]`.

> Naming note: the issue body says *"re-exporting the `quartzite-renderer-style-dispatch` bridge crate"*; the bridge crate's actual name is **`quartzite-style-dispatch`** (final name pinned in the design — see `2026-05-13-renderer-style-dispatch.design.md` § *Why `quartzite-style-dispatch` (and not `quartzite-renderer-style`)*). This spec uses the live crate name.

## Scope

1. Add `quartzite-style-dispatch = { path = "quartzite-style-dispatch", optional = true }` to the facade's `[dependencies]` block in `Cargo.toml` (workspace root, lines 93–103 today).
2. Add a `style-dispatch` entry under `[features]` in the same file. The feature MUST be `optional = true`-gated via `dep:quartzite-style-dispatch`, mirroring `style = ["dep:quartzite-style"]` and `widgets = ["dep:quartzite-widgets"]`. The feature chains the sibling facade features required for end-to-end usage — see § *Key decisions* row "Feature chaining".
3. Append `"style-dispatch"` to the `features = [...]` list under `[package.metadata.docs.rs]` (currently line 149) so docs.rs builds the new module's rustdoc.
4. Add a new `pub mod style_dispatch { pub use quartzite_style_dispatch::*; }` block to `src/lib.rs`, gated on `#[cfg(feature = "style-dispatch")]` and annotated with `#[cfg_attr(docsrs, doc(cfg(feature = "style-dispatch")))]`. The module's `///` doc comment names the two public items (`dispatch_paint`, `WidgetResolver`), states the dep, and links to the `quartzite-style-dispatch` crate.
5. Update the `## Feature flags` section in the facade's crate-level rustdoc (already auto-rendered by `document_features::document_features!()`). The feature's `##` doc comment in `Cargo.toml` is what `document_features` picks up — the doc comment must be present and informative.
6. Add a one-line bullet under the `# Ecosystem` heading in `src/lib.rs` for the new sub-crate, matching the format of existing `- [`core`] (`quartzite-core`) — …` entries.
7. Verify the matrix builds cleanly:
   - `cargo build` (default features — no behaviour change, `style-dispatch` is off).
   - `cargo build --features style-dispatch`.
   - `cargo build --no-default-features --features std,style-dispatch`.
   - `cargo doc --no-deps --workspace --all-features` (doc gate from AGENTS.md).
   - `cargo clippy --workspace --all-targets -- -D warnings`.

## Out of scope

- Any change to the `quartzite-style-dispatch` crate itself. The bridge crate's public surface, deps, and tests stay as they are — this task only adds a facade-level re-export.
- Renaming or expanding the bridge crate's public API.
- A facade-level convenience wrapper around `dispatch_paint` (e.g. `quartzite::paint_widget_tree(...)`). The re-export is verbatim; any ergonomic facade helper is a separate plan.
- Adding new sibling features (e.g. `style-dispatch-test-support`). The bridge crate has no features today and does not expose the `test-support` shape that `quartzite-style` does.
- Adding the bridge crate's `WidgetResolver` to `quartzite::prelude`. The trait has a name collision with `quartzite_widgets::layout::WidgetResolver` (design doc § *Name collision with `quartzite_widgets::WidgetResolver`*); pulling either into the prelude under a single name would force a rename. Path-qualified imports remain the documented shape.
- Touching the facade crate's example files (`examples/`). No existing example uses the styling or widget layer; adding one is a separate task.
- Modifying the workspace `Cargo.toml` `[workspace] members` list. `quartzite-style-dispatch` is already registered (line 14 today).
- Backporting the feature to a published version. Per AGENTS.md § *API Stability*, the facade is pre-publish; no compat layer needed.

## Deferred

- Convenience facade fn `quartzite::paint_widget_tree(...)` that wraps `dispatch_paint` + a default palette + a `StyleRegistry::try_style()`-aware no-op shape | low immediate leverage (`dispatch_paint` already handles the `None` case) | new issue if the four-arg signature proves clunky for typical callers.
- Adding `WidgetResolver` to `quartzite::prelude` | name collision with `quartzite_widgets::layout::WidgetResolver` requires a rename | new issue when one of the two traits becomes prelude-worthy.
- An end-to-end example under `examples/style_dispatch.rs` showing facade-level usage | requires a running painter backend, which the facade does not pull in today | new issue when a `quartzite-renderer`-using example lands.

## Key decisions

| Question | Decision |
|---|---|
| Feature default state | **Off by default** — matches `style`, `widgets`, `serde`, `verbose-tracing`. The `default` feature set stays `["std", "derive"]`; `style-dispatch` is purely opt-in. |
| Facade module name | **`style_dispatch`** (snake_case of the crate name). The module path `quartzite::style_dispatch::{dispatch_paint, WidgetResolver}` parallels `quartzite::style::*`, `quartzite::widgets::*`, `quartzite::paint::*`. |
| Re-export shape | **`pub use quartzite_style_dispatch::*;`** inside the cfg-gated module — verbatim mirror of how `style`, `widgets`, `paint`, `runtime`, `geometry`, `events` are re-exported. No selective re-export list; the bridge crate's public surface is already minimal (`dispatch_paint` + `WidgetResolver`). |
| docsrs annotation | **`#[cfg_attr(docsrs, doc(cfg(feature = "style-dispatch")))]`** on the `style_dispatch` module — matches the `style`, `widgets`, `serde` modules. |
| Feature chaining — does `style-dispatch` imply `style` + `widgets`? | **Chain both** (round-1 answer to Q1). Final feature value: `style-dispatch = ["dep:quartzite-style-dispatch", "style", "widgets"]`. Rationale: the bridge crate's whole purpose is to dispatch `Style` calls across the widget tree, so a caller enabling `style-dispatch` needs both prerequisite vocabularies (`quartzite::style::*` and `quartzite::widgets::*`) immediately compilable. A single `--features style-dispatch` switch gives end-to-end usability; the alternative (independent features, mirroring the current `style`/`widgets` posture) forced "enable feature, hit compile error, enable next feature" iteration. Departs from the workspace-existing single-crate feature precedent deliberately — `style-dispatch` is the first facade feature gating a *bridge* crate, and the bridge-crate kind chains its prerequisites. |
| Ecosystem-bullet placement in `src/lib.rs` | New bullet goes **after the `paint` bullet** (current last bullet under `# Ecosystem`) and **before** the `Add quartzite to your Cargo.toml:` paragraph. Format mirrors existing bullets: ``- [`style_dispatch`] (`quartzite-style-dispatch`) — widget-tree paint dispatcher. Requires the `style-dispatch` feature.`` |
| `document_features` doc-comment shape | The feature's `##` doc comment in `Cargo.toml` mirrors the existing `style` / `widgets` entries: one summary sentence, then a second sentence explaining what the module re-exports. Plain prose; no `[link]` syntax (the auto-rendered doc lives outside an item-doc context). |
| Bridge crate name in spec/design vs issue body | The issue body uses the stale name `quartzite-renderer-style-dispatch`; the live crate name is `quartzite-style-dispatch` (pinned by the design's § *Why `quartzite-style-dispatch` (and not `quartzite-renderer-style`)*). This spec and the implementation use the live name. |
| Workspace `members` change needed? | **No** — `quartzite-style-dispatch` is already a workspace member (root `Cargo.toml` line 14). The facade re-export is the only addition. |
| Self-contained doc-test inside the facade `style_dispatch` module? | **No new doc-test.** The bridge crate's `lib.rs` already carries the AC9 end-to-end doc-test (`2026-05-13-renderer-style-dispatch.spec.md` AC9). The facade's re-export module gets a one-line `///` summary + a `#[doc(inline)]`-style behaviour (transparent re-export) — adding a second doc-test would duplicate the bridge crate's example and force the facade to pull `quartzite-style` + `quartzite-widgets` into its own doc dep cone via the `style-dispatch` feature chain. |

## Technical constraints

- The facade's `[dependencies]` entry MUST use `optional = true` (matches `quartzite-macros`, `quartzite-style`, `quartzite-widgets`). Without `optional = true`, the dep would always compile regardless of the feature flag, defeating the gate.
- The new `pub mod style_dispatch` block in `src/lib.rs` MUST carry `#[cfg(feature = "style-dispatch")]` (so the module disappears when the feature is off — keeps `cargo doc` clean for default-feature builds) AND `#[cfg_attr(docsrs, doc(cfg(feature = "style-dispatch")))]` (so docs.rs renders the feature badge on the module page).
- The `##` Cargo-feature doc comments (consumed by `document_features::document_features!()` at line 213 of `src/lib.rs`) MUST start with `## ` (two `#` + space) for a top-level entry — mirroring the existing `style`, `widgets`, `serde`, etc. entries. The doc-comment body becomes the rendered feature-flags table row.
- The `[package.metadata.docs.rs] features = [...]` array MUST include `"style-dispatch"` so docs.rs builds with the feature on. The existing entry (line 149) already lists `["std", "derive", "widgets", "serde", "style", "verbose-tracing"]`; the new entry appends `"style-dispatch"`.
- Module-name underscore vs crate-name hyphen: Rust module names are snake_case; the facade module `style_dispatch` mirrors the crate `quartzite-style-dispatch`'s hyphenated name with the conventional `-`→`_` translation. The `pub use quartzite_style_dispatch::*;` reference inside the module body uses the snake_case crate-path identifier (Cargo's `pkg-name → pkg_name` rule).
- Lint gate (`cargo clippy --workspace --all-targets -- -D warnings`) and doc gate (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`) MUST pass — same matrix the workspace already enforces. The new module's `///` doc satisfies `missing_docs`.
- `actionlint` is not engaged for this change (no workflow files touched), but if any CI workflow is updated in parallel (e.g. a build matrix that needs `style-dispatch` added), the AGENTS.md `actionlint` AXIOM applies.
- The addition is purely additive — no aliases, no wrappers (AGENTS.md § *API Stability*: pre-publish, clean breaks; nothing to break here since the feature does not exist yet).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | The facade's root `Cargo.toml` contains a `style-dispatch` entry under `[features]` (with the `##` doc comment expected by `document_features`) and a `quartzite-style-dispatch = { path = "quartzite-style-dispatch", optional = true }` entry under `[dependencies]`. The feature value is exactly `style-dispatch = ["dep:quartzite-style-dispatch", "style", "widgets"]` — chain both prerequisites per § *Key decisions* row "Feature chaining". |
| AC2 | `[package.metadata.docs.rs] features` includes `"style-dispatch"`. |
| AC3 | `src/lib.rs` defines `pub mod style_dispatch { pub use quartzite_style_dispatch::*; }` gated on `#[cfg(feature = "style-dispatch")]` with the `#[cfg_attr(docsrs, doc(cfg(feature = "style-dispatch")))]` annotation. The module carries a `///` doc comment summarising the re-export (≥ 1 line, names `dispatch_paint` and `WidgetResolver`). |
| AC4 | `src/lib.rs` `# Ecosystem` section gains a bullet for `quartzite-style-dispatch`, placed after the `paint` bullet and before the `Add quartzite to your Cargo.toml:` paragraph, mirroring the existing bullet shape and noting the `style-dispatch` feature requirement. |
| AC5 | `cargo build` (default features) succeeds. |
| AC6 | `cargo build --features style-dispatch` succeeds AND `cargo check --features style-dispatch` resolves all of: `quartzite::style_dispatch::dispatch_paint`, `quartzite::style_dispatch::WidgetResolver`, `quartzite::style::Palette` (chained `style` feature), and `quartzite::widgets::WidgetBase` (chained `widgets` feature). Verified by a `#[cfg(all(test, feature = "style-dispatch"))]` test in `src/lib.rs` `mod tests` that names all four items (compile-only; e.g. `let _: &dyn WidgetResolver; let _: Palette; let _: WidgetBase;` patterns or `core::mem::size_of::<...>()` calls). This AC is the mechanical check that the feature chain pinned in AC1 propagates to user-visible module access. |
| AC7 | `cargo build --no-default-features --features std,style-dispatch` succeeds (sanity-check the feature doesn't accidentally rely on `derive`). |
| AC8 | `cargo build -p quartzite --no-default-features --features libm` still succeeds (the no-std + derive-free path from AGENTS.md § *Build & Test* is unaffected — `style-dispatch` is off and the new dep entry is `optional`, so it must not contribute to the no-std build). |
| AC9 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` is clean. The auto-rendered `## Feature flags` section in the facade's crate-level rustdoc contains a `style-dispatch` row (verified by `cargo doc` output inspection or by reading the generated HTML for the facade crate). |
| AC10 | `cargo clippy --workspace --all-targets -- -D warnings` is clean. |
| AC11 | The two public items re-exported through the facade (`dispatch_paint`, `WidgetResolver`) match the bridge crate's surface byte-for-byte — verified by the AC6 compile-only test, which would fail at type level if either item changed shape. |

## Open questions

- **Module-name spelling** — `style_dispatch` vs `dispatch` vs `style::dispatch` (sub-module under `style`). Decision: **`style_dispatch`** (top-level module, matches crate-name convention; consistent with `paint` / `widgets` / `runtime`). The chained-feature value (`style` is implied) would also permit a `style::dispatch` sub-module shape, but the top-level shape stays independent of the chain decision and matches the crate-name convention.

- **`document_features` doc-comment body** — the exact wording of the `##` Cargo-feature doc comment. Sensible default: ``## Enable widget-tree paint dispatch (`quartzite-style-dispatch` crate).`` + a second sentence: ``When enabled, `quartzite::style_dispatch` re-exports the `dispatch_paint` free function and the `WidgetResolver` trait, which walk a widget subtree and invoke `Style::draw_widget` once per visible node. Implies `style` and `widgets`.`` Reviewer may tighten on PR.

- **Whether to add a facade-level integration test** under `tests/`. Decision: **no** — AC6's `#[cfg(all(test, feature = "style-dispatch"))]` compile-only test inside `src/lib.rs` `mod tests` is sufficient. Adding a `tests/style_dispatch.rs` integration test would force the test to construct a widget tree + recording painter, which duplicates the bridge crate's own unit tests verbatim. The facade test's job is *re-export resolves*, not *bridge logic works*.
