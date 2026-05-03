# Design: Redesign multi-block object_impl API — #[object_part] + #[object_impl]

**Issue:** #57
**Date:** 2026-05-03

## Approach

### Chosen solution

Replace the three-value flag system (`partial` / `final` / absent) and the standalone
`#[object_meta]` macro with two orthogonal attributes:

- `#[object_part]` — a new `proc_macro_attribute` that accumulates `#[slot]`/`#[invokable]`
  methods and emits only the cleaned impl block. Accepts **no arguments**. Works for both
  inherent and trait impl blocks.
- `#[object_impl]` — no flags; auto-detects its mode by inspecting the accumulator at
  expansion time:
  - **Sole mode** (accumulator empty for this type): identical to the current plain
    `#[object_impl]` — emits cleaned impl block + MetaObject static + `impl Object`.
  - **Terminal mode** (accumulator non-empty): drains accumulated methods, merges with current
    block's methods, then emits the same full output as sole mode.

`#[object_meta]` is removed. `#[object_impl(partial)]` and `#[object_impl(final)]` are
removed.

### Structural changes

The existing `MethodKind` enum (`Sole`, `Partial`, `Final`) and the `parse_kind` function that
parses attribute flags are deleted. The `parse` function in `object_impl/parse.rs` no longer
accepts an attribute argument for selecting the kind; the attribute token stream is validated to
be empty instead (error on any argument). The `ObjectImplInput` struct loses the `kind` field.

A new top-level module `object_part` is added (`src/object_part/mod.rs`,
`src/object_part/parse.rs`, `src/object_part/codegen.rs`). Its `parse` validates that the
attribute token stream is empty and delegates to the same `parse2::<ItemImpl>` +
`extract_params` logic. Its `codegen` calls `emit_impl_block` directly.

Two helpers in `object_impl/` that are currently private must be promoted to `pub(crate)` so
`object_part` can share them:
- `emit_impl_block` in `object_impl/codegen.rs` (currently `fn`, needs `pub(crate) fn`)
- `extract_params` in `object_impl/parse.rs` (currently `fn`, needs `pub(crate) fn`)

Both promotions are explicit steps inside Task 2 (for `extract_params`) and Task 3 (for
`emit_impl_block`) respectively.

The `object_meta` module (`src/object_meta/`) is deleted in its entirety.

Mode detection in `object_impl/mod.rs` replaces the `match ir.kind` dispatch with a check on
`accumulator::peek` (a new zero-cost helper that returns `true` if the accumulator is non-empty
for the given key without consuming it) to decide between sole and terminal paths.

### Rejected alternatives

- **Keep flags, just rename** — `#[object_impl(part)]` instead of a new attribute. Rejected
  because two separate attributes are more discoverable, easier to document independently, and
  avoid conflating unrelated concerns under one macro.
- **Implicit detection without accumulator check** — always run terminal codegen (drain returns
  empty vec for sole use). Rejected because the spec requires `#[object_impl]` to be
  self-describing at expansion time, and an empty drain is indistinguishable from a type with no
  methods; using an explicit peek preserves the invariant for future diagnostics.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `accumulator::peek` — a `bool`-returning helper that checks whether any methods are staged for a type without draining | `quartzite-macros/src/object_impl/accumulator.rs` | — |
| 2 | Rewrite `object_impl/parse.rs`: remove `MethodKind`, `parse_kind`, the `kind` field; validate that attr is empty; promote `extract_params` to `pub(crate)`; keep all other parsing logic | `quartzite-macros/src/object_impl/parse.rs` | — |
| 3 | Rewrite `object_impl/mod.rs`: replace `match ir.kind` with accumulator-based auto-detection; remove `Partial`/`Final` arms; reject any non-empty attr with a compile error; promote `emit_impl_block` to `pub(crate)` | `quartzite-macros/src/object_impl/mod.rs`, `quartzite-macros/src/object_impl/codegen.rs` | 1, 2 |
| 4 | Add `object_part` module: `parse` validates empty attr + parses impl block (reuses `pub(crate) extract_params`); `codegen` calls `pub(crate) emit_impl_block` directly and emits only the cleaned impl block; `mod.rs` (`expand`) pushes extracted methods into the accumulator then calls codegen (same pattern as `object_impl/mod.rs`); delete `codegen_partial` from `object_impl/codegen.rs`; migrate its test to `object_part/codegen.rs` using `object_part::parse::parse(quote! {}, …)` for IR setup (not the old `parse(quote! { partial }, …)` form); update accumulator duplicate error message to reference `#[object_part]` instead of `#[object_impl(partial)]` | `quartzite-macros/src/object_part/mod.rs`, `quartzite-macros/src/object_part/parse.rs`, `quartzite-macros/src/object_part/codegen.rs`, `quartzite-macros/src/object_impl/codegen.rs`, `quartzite-macros/src/object_impl/accumulator.rs` | 1, 2 |
| 5 | Register `object_part` in `lib.rs`; delete `object_meta` registration and doc-comment; update `object_impl` doc-comment to describe auto-detection and remove `partial`/`final` and `#[object_meta]` mentions | `quartzite-macros/src/lib.rs` | 3, 4 |
| 6 | Delete `object_meta` module directory | `quartzite-macros/src/object_meta/` (3 files) | 5 |
| 7 | Update integration test `quartzite-macros/tests/object_impl.rs` to use `#[object_part]` / plain `#[object_impl]`; add tests for AC1–AC8 from spec | `quartzite-macros/tests/object_impl.rs` | 5 |
| 8 | Update prelude re-export in `src/lib.rs` to add `object_part`; update doc comment that lists macros | `src/lib.rs` | 5 |

## Risks

- **Accumulator state leak between tests**: the `thread_local!` accumulator is shared within a
  test thread. Each unit test that calls `push` must call `drain` (or `peek` returns true for a
  subsequent test on the same type name). Mitigation: use unique type name tokens per test (same
  pattern already used in existing tests: `__TestFoo__`), and always drain after each test that
  pushes.
- **`peek` inadvertently consuming state**: `peek` must be a read-only borrow — no `remove` or
  `drain`. Mitigation: implement as `map.get(&key).map_or(false, |v| !v.is_empty())`.
- **AC7 error quality for removed flags**: when a user writes `#[object_impl(partial)]` or
  `#[object_impl(final)]`, the new `parse` rejects any non-empty attribute. The error message
  must be actionable. Mitigation: tailor the message: `` `#[object_impl]` takes no arguments — use `#[object_part]` for accumulating blocks ``.
- **AC8 error quality for `#[object_part(…)]`**: same issue. Error message: `` `#[object_part]` takes no arguments ``.
- **`object_meta` still referenced in `quartzite-core/src/meta.rs` doc comments**: doc comments
  in `meta.rs` reference `#[object_impl]` generically — no change needed there. No code
  references `object_meta` by path outside the macro crate itself.
- **`codegen_partial` rename**: the existing function is internal (`pub(crate)`) and used only
  in `object_impl/mod.rs` (partial arm) and in a codegen test. After the redesign, `object_part`
  will call `emit_impl_block` directly (already `pub(crate)`), making `codegen_partial` dead
  code. It should be deleted; its single test case migrates to `object_part/codegen.rs`.

## Test Design

### Task 1 — `accumulator::peek`

- Location: `quartzite-macros/src/object_impl/accumulator.rs` `#[cfg(test)]` module
- Entry point: `peek(type_name: &str) -> bool`
- Scenarios:
  - peek on empty accumulator returns `false`
  - peek after push returns `true`
  - peek does not consume: drain after peek still returns the methods
  - peek after drain returns `false`

### Task 2 — `object_impl/parse.rs` (attr validation)

- Location: `quartzite-macros/src/object_impl/parse.rs` `#[cfg(test)]` module
- Entry point: `parse(attr, item)`
- Scenarios:
  - empty attr parses successfully (sole path)
  - non-empty attr (`partial`, `final`, arbitrary token) returns `syn::Error` with the
    actionable message
  - all existing parse scenarios (slot, invokable, trait impl, etc.) continue to pass with
    empty attr

### Task 3 — `object_impl/mod.rs` (auto-detection)

- Location: `quartzite-macros/src/object_impl/mod.rs` `#[cfg(test)]` module
- Entry point: `expand(attr, item)`
- Scenarios:
  - AC3: empty accumulator → sole mode → output contains MetaObject static and `impl Object`
  - AC4: non-empty accumulator → terminal mode → accumulated method names appear in output,
    current block method names appear in output, MetaObject static present
  - AC6: duplicate method pushed via `object_part` then same name in `object_impl` terminal →
    compile_error in output (partial→final duplicate)
  - AC5: two `object_part` blocks with the same method name → compile_error from accumulator
    push on second block (partial→partial duplicate)
  - AC7: `expand` with non-empty attr token stream → compile_error with `#[object_part]` hint

### Task 4 — `object_part` module

- Location: `quartzite-macros/src/object_part/` (new `#[cfg(test)]` modules in each file) and
  the existing `quartzite-macros/tests/object_impl.rs` integration test
- Entry point: `object_part::expand(attr, item)`
- Scenarios:
  - AC1: inherent impl block → output contains cleaned impl block, no MetaObject static, no
    `impl Object`
  - AC2: trait impl block → output contains `impl Trait for Type { … }` with slot/invokable
    attrs stripped, no MetaObject static, no `impl Object`
  - AC8: non-empty attr → compile_error with actionable message
  - methods extracted into accumulator (verifiable via `peek` after expansion)
  - AC5/AC6 duplicate detection: second `object_part` block for same type + same method name →
    compile_error returned from `push`

### Task 7 — integration tests

- Location: `quartzite-macros/tests/object_impl.rs`
- Full end-to-end scenarios using real macro expansion:
  - sole `#[object_impl]` (no prior `#[object_part]`): `Object` trait is implemented, method
    dispatch works
  - multi-block: one `#[object_part]` + one `#[object_impl]`: methods from both blocks
    accessible via `invoke_method`
  - multi-block with trait impl `#[object_part]`: trait impl block re-emitted, methods
    dispatched
  - (compile-fail, if trybuild is available) `#[object_impl(partial)]` → compile error

## Open questions

- None.
