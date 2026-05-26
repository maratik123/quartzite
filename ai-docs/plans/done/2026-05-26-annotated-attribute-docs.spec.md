# Documentation for annotated fields/fn's

**Source:** issue #564
**Date:** 2026-05-26
**Tracked in:** #564

## Scope

1. Inventory every annotated item in workspace library code (excludes `examples/`, `tests/`, `benches/`, `quartzite-test-helpers/src/**`) carrying one of the proc-macro attributes the object-type system recognises (see *Annotated-attribute inventory* below).
2. For each annotated item that is **not** `pub`, promote it to `pub` by default. A `// why-private: <reason>` line MAY defend the rare exception (macro-codegen invariant the public surface must not expose); the default is lift.
3. Add a `///` doc comment to every annotated item — regardless of resulting visibility — that meets the existing `ai-docs/doc-convention.md` summary-line and section-order rules. The doc requirement is independent of the visibility decision in (2); both apply.
4. Extend `ai-docs/doc-convention.md` with a new section (`## Annotated items`) that names the convention: annotated items always carry `///` docs even when private; the existing pub-only exception in § *Scope* does not apply when any of the inventoried attributes is present on the item. The section uses the live attribute names (`#[prop]`, `#[signal]`, `#[slot]`, `#[invokable]`, etc.).
5. Extend `ai-docs/code-style.md` with a sibling section (`## Annotated items`) that names the visibility-lift default: every annotated item is `pub` unless its companion `// why-private:` comment defends the exception. Defence-in-depth: the doc-convention requirement (point 4) survives a future `why-private:` exception without losing the doc requirement.
6. Update the proc-macro crate (`quartzite-macros/src/{object,object_impl,object_part,extend}/`) so each macro emits a **tri-state diagnostic** (`allow` / `warn` / `deny`) at expansion time when an annotated item is missing its `///` doc. The semantics mirror rust's native lint-level vocabulary:
   - **`allow`** — silent; no diagnostic emitted. Use cases: downstream users writing small / casual code who don't want the noise; `examples/` / `benches/` / `tests/` where doc requirements are inappropriate.
   - **`warn`** (default) — surfaced during `cargo build` as a non-error diagnostic. Humans should still fix the source — the warning is a nudge, not a gate.
   - **`deny`** — surfaced as a compile error (`compile_error!`-equivalent). Opt-IN per-item / per-invocation / globally; the spec used to flat-reject hard-error mode, but tri-state lifts that rejection — `deny` is opt-in, not the default.

   The level MUST be configurable at **three scopes**, cascading per rust's native lint precedence (most-specific wins):

   1. **Per-item** — an attribute placed directly on the annotated field / method, e.g. `#[undocumented(allow)]` (or whichever ident is chosen at design phase) on a field carrying `#[base]` / `#[signal]` / etc. Silences (or escalates) just that one site.
   2. **Per-invocation** — an attribute on the enclosing macro invocation, e.g. `#[object_impl(undocumented = "allow")]` / `#[derive(Object)] #[object(undocumented = "deny")]`. Silences (or escalates) every annotated item inside that block; per-item override still wins.
   3. **Global** — a cargo feature on `quartzite-macros` (e.g. `undocumented-deny`) and/or an environment variable read at proc-macro expansion time. Sets the workspace-wide baseline; per-invocation and per-item overrides still win.

   The exact emission vehicle for `warn` (synthesised `#[deprecated]` item with `quote_spanned!`-pointed span, `#[diagnostic::…]` attribute, custom-lint registration, or alternative) and the exact ident surface (`#[undocumented(allow)]` vs `#[quartzite(undocumented = "allow")]` vs a registered `undocumented` derive-attribute) are **design-phase choices**. The spec mandates: tri-state semantics, all three scopes, most-specific-wins cascade, default = `warn`, AC7 fixture coverage of every state at every scope.

## Out of scope

- Refactoring widget public APIs beyond the visibility-lift mechanical change (no behavioural changes to `WidgetBase`, `Container`, etc.).
- **Renaming existing macro attribute identifiers** (`#[prop]` / `#[invokable]` stay as-is — the Spec Amendment 2026-05-26 dropped the original rename plan, see § Spec Amendment history below). No attribute identifiers are renamed in this PR.
- Adding net-new attribute identifiers to the macro vocabulary beyond the new helper `#[undocumented]` (per scope item 6). No other new attributes (e.g. an additional `#[notify]` alongside the existing `notify` sub-attribute) are introduced.
- Doc rewriting on non-annotated items (the standard `missing_docs = "deny"` workspace lint already covers public non-annotated items).
- The audit / lift / doc-add work is library `src/` only; `examples/` / `tests/` / `benches/` are exempt.
- Cross-references between the four convention sources cited in `doc-convention.md`'s § *References*.
- Making `deny` the default — the default remains `warn`. `deny` is opt-in only, per scope item 6. (Workspace CI MAY enable `deny` via the global cargo feature, but the spec does not mandate that decision.)

## Deferred

- Whether `#[prop]` sub-attributes (`notify` / `read_only` / `constant` / `stored` / `designable` / `user`) deserve a per-sub-attribute doc-comment paragraph template | low-leverage, design phase can choose | no separate issue needed
- Whether `#[base]` / `#[widget_children]` deserve a synthesized `# Inheritance` rustdoc section emitted by the `Extend` derive | low-leverage; design phase can choose | no separate issue needed

## Annotated-attribute inventory

Attributes recognised by the workspace proc-macros (`quartzite-macros/src/lib.rs`).

| Attribute | Defined by macro | Site | Notes |
|---|---|---|---|
| `#[signal]` | `derive(Object)` | field of type `Signal<Args>` | recorded in `MetaObject`; codegen synthesises `emit_*` / `connect_*` methods |
| `#[slot]` | `#[object_impl]` / `#[object_part]` | method, return type `()` | callable via `Object::invoke_method` |
| `#[invokable]` | `#[object_impl]` / `#[object_part]` | method, any return type implementing `IntoValue` | callable via `Object::invoke_method` |
| `#[prop]` | `derive(Object)` | field | readable/writable property; sub-attrs `notify`, `read_only`, `constant`, `stored`, `designable`, `user` |
| `#[root]` | `derive(Extend)` | struct attr | marks hierarchy root |
| `#[base]` | `derive(Extend)` | field | parent-object delegation target |
| `#[mixin]` | `derive(Extend)` | field | additional delegation target |
| `#[widget_view(variant = ...)]` | `derive(Extend)` | struct attr | only meaningful for `AsWidget` subtypes |
| `#[widget_children(slice\|optional)]` | `derive(Extend)` | field | overrides `AsWidget::children` default |

## Concrete cases of non-`pub` annotated items found in workspace `src/`

(Comprehensive scan against `quartzite-widgets/src/**`, `quartzite-runtime/src/**`, `quartzite-core/src/**`, `src/**`; other workspace crates carry no annotations.)

| File | Item | Attribute | Currently `pub`? |
|---|---|---|---|
| `quartzite-widgets/src/widget_base.rs` | `WidgetBase::object` field | `#[base]` | no |
| `quartzite-widgets/src/widgets/{label,button,line_edit,text_edit,scroll_area,container}.rs` | `<Widget>::widget_base` field | `#[base]` | no (×6) |
| `quartzite-widgets/src/layout/{box_layout,grid_layout}.rs` | `<Layout>::object` field | `#[base]` | no (×2) |
| `quartzite-widgets/src/widgets/container.rs` | `Container::children: Vec<ObjectId>` | `#[widget_children(slice)]` | no |
| `src/lib.rs` doctest fixture | `Counter::object_base` | `#[base]` | no (doc-fixture only — out of scope) |

**Already-`pub`-and-documented annotated items (no action required — listed for completeness so the audit can skip them):**

| File:line | Item | Attribute |
|---|---|---|
| `quartzite-runtime/src/timer.rs` | `Timer::base: ObjectBase` | `#[base]` |
| `quartzite-runtime/src/event_loop.rs:58` | `EventLoop::base: ObjectBase` | `#[base]` |
| `quartzite-widgets/src/widgets/scroll_area.rs:46` | `ScrollArea::content_widget: Option<ObjectId>` | `#[widget_children(optional)]` |

The first three already-`pub`-and-documented sites confirm the visibility lift is feasible without breaking macro codegen.

All `#[signal]` / `#[slot]` / `#[invokable]` / `#[prop]` sites observed in workspace library code are already `pub` — the visibility-lift workload concentrates on the `#[base]` / `#[widget_children]` pattern above.

## Key decisions

| Question | Decision |
|---|---|
| Which crates are in scope for the audit? | Every workspace crate's `src/` directory. `examples/`, `tests/`, `benches/`, and `quartzite-test-helpers/src/**` are excluded (already standing convention from `doc-convention.md` § *Local enforcement*). |
| Doc-convention scope for annotated items | Annotated items always require `///` docs, regardless of visibility — the existing pub-only exemption is overridden when any of the inventoried attributes is present. (Pre-resolved per the issue body: "should be documented … even if they are not in pub api".) |
| Visibility-lift default | Promote to `pub` unless a `// why-private:` rationale defends the exception. (Pre-resolved per the issue body: "need to promote field/fn to pub, if nothing blocked".) |
| Naming of new convention sections | `## Annotated items` in `doc-convention.md`; `## Annotated items` in `code-style.md`. |
| Macro attribute renames (`#[prop]` → `#[property]`, `#[invokable]` → `#[invoke]`) | **Dropped 2026-05-26 (Spec Amendment 3).** Original plan was to rename both; user reverted before PR merge. Macros continue to recognise `prop` and `invokable` as their attribute idents; no use-site rename happens in this PR. See § Spec Amendment history. |
| Macro-level enforcement | Tri-state diagnostic — `allow` (silent) / `warn` (default, non-error build diagnostic) / `deny` (hard `compile_error!`-equivalent). Mirrors rust's native lint-level vocabulary. Configurable at three scopes with most-specific-wins cascade: per-item attribute on the annotated field/method, per-invocation attribute on the enclosing macro, global via cargo feature on `quartzite-macros` and/or env var. Default = `warn`; `deny` is opt-in only (workspace CI MAY enable it globally). Specific emission vehicle for `warn` + ident surface for the opt level deferred to design phase. AC7 fixture coverage spans every level at every scope. |

## Technical constraints

- `cargo build` + `cargo clippy --workspace --all-targets -- -D warnings` + `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` must continue to pass.
- Visibility lift on `#[base]` fields must not break `Extend` codegen: the codegen reads the field by name (`self.<base-ident>`) and never assumes a particular visibility, so the change is mechanical. Confirmed by `Timer::base` already being `pub`.
- Two convention files (`doc-convention.md`, `code-style.md`) edited together. Both are below the 35 000-char early-warning cap and the change adds ~1 KB at most; no extraction PR triggered.
- No new dependency added.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | Every annotated item in workspace `src/` (per the *Annotated-attribute inventory* table) carries a `///` doc comment whose summary line conforms to `doc-convention.md` § *Summary line*. |
| AC2 | Every annotated item in workspace `src/` is either `pub` or carries an immediately-preceding `// why-private: <reason>` line (one of: macro-codegen requirement, internal-only invariant the macro relies on, etc.). |
| AC3 | `ai-docs/doc-convention.md` has a new `## Annotated items` section stating that the inventoried attributes (table reproduced from this spec) override the pub-only exemption and list the per-attribute summary-line patterns. |
| AC4 | `ai-docs/code-style.md` has a new `## Annotated items` section stating the visibility-lift default (`pub` unless `// why-private:` defends the exception). |
| AC5 | `cargo build` + `cargo clippy --workspace --all-targets -- -D warnings` + `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` all pass. |
| AC6 | **Dropped 2026-05-26 (Spec Amendment 3).** The original rename criterion has been retired; macros continue to recognise `prop` and `invokable`. `grep -rn '#\[prop\b' --include='*.rs' .` SHOULD NOT be empty — pre-existing use-sites remain. |
| AC7 | The proc macros emit a tri-state diagnostic (`allow` / `warn` / `deny`) at expansion time per scope item 7. Default level is `warn`. A workspace test under `quartzite-macros/tests/` (`trybuild` recommended for the `deny`-engages-error path; warnings-captured fixture or compile-only test capturing rustc stderr for the others) covers **every level at every scope** — at minimum: (a) `warn` fires when doc is missing (default, no override); (b) silent when doc is present (default, no override); (c) per-item `allow` silences a missing doc; (d) per-invocation `allow` silences a block of missing docs; (e) global `allow` (via cargo feature or env var) silences all; (f) per-item `deny` escalates a missing doc to compile error (`trybuild` compile-fail fixture); (g) per-invocation `deny` escalates a block; (h) most-specific-wins cascade (per-item `allow` overrides per-invocation `deny`, etc. — at least one fixture demonstrating the precedence). `cargo build` succeeds for paths (a)–(e) and (h)-allow-cases; paths (f)–(g) and (h)-deny-cases are compile-fail fixtures. All tri-state surfaces (per-item attribute name, per-invocation attribute name, cargo feature name, env var name) are documented in `doc-convention.md` § *Annotated items*. |

## Open questions

- The specific `warn`-emission vehicle inside `quartzite-macros` for the default level — candidates (non-exclusive):
  - synthesised `#[deprecated]` item with `quote_spanned! { field.ident.span() => … }` so rustc points at the user's field (caveat: collides with user `#[allow(deprecated)]` — known false-suppression risk that the design must record in § Risks);
  - `#[diagnostic::on_unimplemented]`-family attribute (or similar `diagnostic::*` namespace tools);
  - custom-lint registration (heavier, requires lint-registration plumbing).

  Constraint: warning surfaces during `cargo build` without `-D warnings`-gating the whole workspace, AND the warning's span points at the user-visible field/method, not at the macro-synthesised mod.

- The tri-state ident surface — three independent design-phase choices:
  1. **Per-item attribute** — `#[undocumented(allow)]` / `#[undocumented(warn)]` / `#[undocumented(deny)]` on the annotated field/method. Requires `undocumented` (or chosen ident) registered in every consuming derive's `attributes(...)` list (`derive(Extend)`, `derive(Object)`) and parsed by the `#[object_impl]` / `#[object_part]` attribute macros. Alternative surface: `#[quartzite(undocumented = "allow")]` if a single `quartzite` namespace is preferred — also needs `quartzite` registered in every `attributes(...)` list.
  2. **Per-invocation attribute** — `#[object_impl(undocumented = "allow")]` on the attribute macro itself, or `#[derive(Object)] #[object(undocumented = "deny")]` as a sibling attribute. The per-invocation level applies to every annotated item inside the block; per-item overrides still win (most-specific-wins cascade).
  3. **Global** — cargo feature on `quartzite-macros` (e.g. `undocumented-deny` enables workspace-wide `deny`; `undocumented-allow` disables the lint entirely) and/or an env var read at expansion time (e.g. `QUARTZITE_UNDOCUMENTED=allow|warn|deny`).

  Default for design: combine all three with the precedence `per-item > per-invocation > global > built-in default (warn)`. Each scope mapped to one concrete ident / feature / env var; AC7 fixtures cover every level at every scope.

- Whether the `#[signal]` / `#[slot]` / `#[invokable]` / `#[prop]` summary lines deserve a per-attribute rustdoc template (e.g. "Signal emitted when …" / "Slot invoked when …") in `doc-convention.md` § *Annotated items*, or whether a single common template suffices. Sensible default: single common template; revisit if reviewers want per-attribute prose.

## Spec Amendment history

- **Amendment 1 (2026-05-26, post design-review R1)** — dropped two stale rows from § *Concrete cases* table (`EventLoop::base` and `ScrollArea::content_widget` already `pub` + documented; auditing confirmed).
- **Amendment 2 (2026-05-26, post-user-followup)** — expanded macro enforcement from warn-only with one opt-out to full tri-state `allow / warn / deny` with per-item + per-invocation + global scopes (most-specific-wins cascade).
- **Amendment 3 (2026-05-26, post-PR-#572-open)** — user reverted the `#[prop]`→`#[property]` and `#[invokable]`→`#[invoke]` rename. Dropped: scope item 4 (rename clean break); AC6 (rename grep empty); rename mapping table; key-decisions row for the rename. Updated: inventory table reverted to live attribute names (`#[prop]`, `#[invokable]`); doc-convention § *Annotated items* uses the live names; technical-constraint line for the rename removed. Out-of-scope row added enforcing "no identifier rename". Tri-state `#[undocumented]` helper attribute + opt scopes + cascade preserved.
