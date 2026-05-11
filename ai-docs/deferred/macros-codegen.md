# Macros & Codegen

Items extracted from completed plans. See [index](../deferred-items.md).

## Deferred

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| `#[object_impl]` on multiple impl blocks for one type \| single impl block only for now | [macros spec](../plans/done/2026-05-01-macros.spec.md) | | #57 (closed) |
| `proc_macro_crate` based path detection in `quartzite-macros` \| would let users depend on only `quartzite` or only `quartzite-core`; non-trivial, separate task | [examples-crate spec](../plans/done/2026-05-02-examples-crate.spec.md) | | #57 (closed) |
| Generic struct support for `#[derive(Object)]` (not `#[derive(Extend)]`) — bound propagation rules unclear | [macro-codegen-improvements spec](../plans/done/2026-05-03-macro-codegen-improvements.spec.md) |  | untracked |
| `#[object_impl]` on generic types — orthogonal to this task | [macro-codegen-improvements spec](../plans/done/2026-05-03-macro-codegen-improvements.spec.md) |  | untracked |
| Manual `Debug` impl with field filtering — auto-derive is sufficient now; can be revisited if fields are added that should be hidden | [objectbase-debug-rename-factory spec](../plans/done/2026-05-03-objectbase-debug-rename-factory.spec.md) |  | #256 |
| Additional `impl Into<T>` sites — discovered during implementation | [generic-fn-split spec](../plans/done/2026-05-07-generic-fn-split.spec.md) |  | #257 |

## Out of scope

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Runtime-specific code in macros (macros emit code that calls quartzite-core APIs) | [macros spec](../plans/done/2026-05-01-macros.spec.md) | ✅ done | |
| `#[derive(Extend)]` on enums or tuple structs (named fields only) | [macros spec](../plans/done/2026-05-01-macros.spec.md) | | #212 |
| Test-only `AsObject` impls inside `#[cfg(test)]` modules | [inline-simple-fns spec](../plans/done/2026-05-02-inline-simple-fns.spec.md) | | #213 |
| Any generated or hand-written function with branches, loops, or more than one call to a non-simple function | [inline-simple-fns spec](../plans/done/2026-05-02-inline-simple-fns.spec.md) (superseded by AGENTS.md Code Style → `#[inline]` recursive rule) | | untracked |
| Codegen emission of `/// _Simple._` doc tag for generated generic simple fns and for generated trait-method docs whose impls are always simple | AGENTS.md Code Style → `#[inline]` and the `_Simple._` doc tag | | #117 (closed) |
| Changing `quartzite-macros` codegen (deferred — `proc_macro_crate` is a separate future task) | [examples-crate spec](../plans/done/2026-05-02-examples-crate.spec.md) | | #57 (closed) |
| `#[derive(Extend)]` on enums or tuple structs (named fields only — existing constraint, unchanged) | [macro-codegen-improvements spec](../plans/done/2026-05-03-macro-codegen-improvements.spec.md) |  | untracked |
| Generic functions or blanket-impl trait methods — not affected by codegen improvements | [macro-codegen-improvements spec](../plans/done/2026-05-03-macro-codegen-improvements.spec.md) |  | untracked |
| Changing the public API surface of `quartzite-macros` (attribute names, derive names) | [macro-codegen-improvements spec](../plans/done/2026-05-03-macro-codegen-improvements.spec.md) |  | untracked |
| Multi-crate / cross-crate `#[object_impl]` collection (same compilation unit only) | [macro-codegen-improvements spec](../plans/done/2026-05-03-macro-codegen-improvements.spec.md) |  | untracked |
| Changing the `thread_local!` accumulator mechanism or its key format (`CARGO_PKG_NAME::type_name`) | [object-part-redesign spec](../plans/done/2026-05-03-object-part-redesign.spec.md) |  | untracked |
| Changing duplicate-detection behaviour or error messages | [object-part-redesign spec](../plans/done/2026-05-03-object-part-redesign.spec.md) |  | untracked |
| Any other proc-macro (`#[derive(Object)]`, `#[derive(Extend)]`, `#[meta_enum]`) | [object-part-redesign spec](../plans/done/2026-05-03-object-part-redesign.spec.md) |  | untracked |
| `dynamic_properties` — tracked by a separate issue | [objectbase-debug-rename-factory spec](../plans/done/2026-05-03-objectbase-debug-rename-factory.spec.md) |  | untracked |
| Any other `ObjectTree` changes beyond `rename` semantics | [objectbase-debug-rename-factory spec](../plans/done/2026-05-03-objectbase-debug-rename-factory.spec.md) |  | untracked |
| Hand-written `impl Trait for Type` outside `quartzite-macros` (rule is in AGENTS.md; reviewer audit covers it) | [codegen-inline-concrete-trait-impls spec](../plans/done/2026-05-07-codegen-inline-concrete-trait-impls.spec.md) |  | untracked |
| `// _Simple._` codegen (Rust strips comments from token streams before parsing) | [codegen-inline-concrete-trait-impls spec](../plans/done/2026-05-07-codegen-inline-concrete-trait-impls.spec.md) |  | untracked |
| `/// _Simple._` on trait declarations (already emitted by PRs #120/#127) | [codegen-inline-concrete-trait-impls spec](../plans/done/2026-05-07-codegen-inline-concrete-trait-impls.spec.md) |  | untracked |
| `where_clause` predicate inspection (bounds don't affect per-impl symbol count) | [codegen-inline-concrete-trait-impls spec](../plans/done/2026-05-07-codegen-inline-concrete-trait-impls.spec.md) |  | untracked |
| Non-simple methods: `read_property`, `write_property`, `invoke_method`, `from_value`, `__lookup_*`, `__connect_signal_dynamic_*` | [codegen-inline-concrete-trait-impls spec](../plans/done/2026-05-07-codegen-inline-concrete-trait-impls.spec.md) |  | untracked |
| Part 1 (hand-written trait declarations) — closed by PR #120. | [codegen-simple-marker spec](../plans/done/2026-05-07-codegen-simple-marker.spec.md) |  | untracked |
| Adding any `_Simple._` marker form to codegen — the trait-declaration `/// _Simple._` tag (PR #120) is the canonical signal; generated impls inherit it by Rust's rustdoc inheritance rules. | [codegen-simple-marker spec](../plans/done/2026-05-07-codegen-simple-marker.spec.md) |  | untracked |
| Other `impl Into<T>` / `impl AsRef<T>` / `impl ToString` sites beyond the four listed targets. If additional > 3-line-body sites are found during implementation, file a follow-up issue. | [generic-fn-split spec](../plans/done/2026-05-07-generic-fn-split.spec.md) |  | untracked |
| Binary-size measurement (`cargo bloat`). Optional: if available locally, note before/after numbers in PR body. | [generic-fn-split spec](../plans/done/2026-05-07-generic-fn-split.spec.md) |  | #258 |
| **Marker stripping for no-longer-simple fns.** Removing `#[inline]` / `_Simple._` from fns whose bodies became non-simple (separate concern; this PR is annotation-only). | [recursive-inline-annotations spec](../plans/done/2026-05-07-recursive-inline-annotations.spec.md) |  | #259 |
| **Refactoring** any fn body to make it simple. Annotate only what already qualifies. | [recursive-inline-annotations spec](../plans/done/2026-05-07-recursive-inline-annotations.spec.md) |  | untracked |
| **Codegen-output marker mirroring** changes in `quartzite-codegen` beyond what naturally falls out of the sweep — i.e. if codegen *emits* fns that should now carry the marker, the codegen itself is updated; but no broader codegen restructuring. | [recursive-inline-annotations spec](../plans/done/2026-05-07-recursive-inline-annotations.spec.md) |  | #260 |
| **API renames / signature changes.** Annotation-only PR. | [recursive-inline-annotations spec](../plans/done/2026-05-07-recursive-inline-annotations.spec.md) |  | untracked |

## Open questions

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Should `Base` suffix be stripped when forming `As{Name}` trait? (`WidgetBase` → `AsWidget` or `AsWidgetBase`?) | [macros spec](../plans/done/2026-05-01-macros.spec.md) | ✅ done | |
| Should `#[derive(Object)]` require `#[derive(Extend)]` to be present, or are they independent? | [macros spec](../plans/done/2026-05-01-macros.spec.md) | ✅ done | |
| How to handle generic structs with `#[derive(Extend)]`? | [macros spec](../plans/done/2026-05-01-macros.spec.md) | | #57 (closed) |
| Should `MetaObject::new()` be extended with fn pointer params or replaced by struct literal construction in the macro? Decide in design phase. | [lookup-perf spec](../plans/done/2026-05-02-lookup-perf.spec.md) | ✅ done | |
| What mechanism enables collecting `#[object_impl]` methods across multiple independent macro invocations? Proc-macro calls are stateless between invocations. Candidates: `thread_local!` accumulation within one compilation, aggregating terminal attribute (`#[object_meta]`), or `linkme`/`inventory`-style distributed slices. (Design phase decision.) | [macro-codegen-improvements spec](../plans/done/2026-05-03-macro-codegen-improvements.spec.md) |  | untracked |
| `ObjectFactory::register` and `ObjectBase::named` have bodies shorter than the "~3 lines" threshold. Spec-mandated for this PR; if deemed noise, a follow-up can revert these two. | [generic-fn-split design](../plans/done/2026-05-07-generic-fn-split.design.md) |  | untracked |
